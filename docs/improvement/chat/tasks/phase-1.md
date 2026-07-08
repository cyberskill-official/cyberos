# Phase 1 - sync core (T-011..T-022)

Exit bar (report section 6): the offline acceptance scenario passes on web - kill the app mid-send,
reopen offline, reconnect, message arrives exactly once; convergence suite green in CI; p95 send-to-echo
measured. Design authority: report section 3 (read it before starting any task here).

## T-011 chat_events log + per-tenant seq

- C-refs C1 | P0/M | depends: none
- Touch: migration 00NN_chat_events.sql; new services/chat/src/events.rs; every mutating handler in
  messages.rs, reactions.rs, read.rs, members.rs, channels.rs, attachments.rs, prefs.rs.
- Spec: chat_events(tenant_id, seq bigint, channel_id nullable, kind text, payload jsonb, created_at);
  seq from a per-tenant allocator (sequence table row with UPDATE ... RETURNING inside the tx is fine at
  this scale; document the choice); PK (tenant_id, seq); index (tenant_id, channel_id, seq); RLS by
  tenant. Event kinds and payload shapes exactly as report 3.1 (reaction events carry ABSOLUTE snapshots;
  no presence/typing ever). Every mutation writes its event in the same transaction or the whole tx fails.
- Accept: unit test proves atomicity (forced failure after mutation write rolls back both); seq strictly
  monotonic per tenant under 32 concurrent writers (test); migration clean; smokes green.
- Review: eyeball the kind list vs report 3.1 - this taxonomy is the protocol, renames later are painful.

## T-012 Seq-stamped ws frames + gap detection

- C-refs C3 | P0/S | depends: T-011
- Touch: services/chat/src/realtime.rs (frame envelope), apps/web/src/lib chat socket layer.
- Spec: every event frame carries {seq, channel_id}; client tracks last seq per channel + account; on
  next-seq mismatch it marks the channel dirty and schedules a sync call (T-013) instead of trusting the
  stream. Until T-013 lands, dirty triggers the existing refetch-merge path.
- Accept: unit test on the client cursor logic (dup frame ignored, gap flags dirty); frame shape
  documented inline pending T-053's catalog.
- Review: none beyond diff read; this is plumbing for everything after.

## T-013 POST /v1/chat/sync v1 + log retention

- C-refs C2, C10 | P0/M | depends: T-011
- Touch: new services/chat/src/sync.rs; lib.rs route; retention job (tokio interval or SQL cron note);
  tests/smoke_sync.py.
- Spec: body {pos?, subscriptions?[channel_id, timeline_limit], timeout_ms?}; response {pos, channels:
  {id: {initial?, events[] | snapshot, limited?, prev_batch?}}, account: {channel_list_changes,
  prefs_changes, unread_summary}}. pos = opaque wrap of tenant seq. No pos -> bounded snapshot for the
  subscribed channels. Gap beyond timeline_limit -> limited:true + prev_batch cursor into existing
  pagination. pos older than horizon -> 410-style typed reset error; client falls back to snapshot.
  Retention: delete events older than CHAT_EVENTS_RETENTION_DAYS (default 30), daily. timeout_ms>0
  long-polls on the tenant seq (this is the C21 fallback transport later - keep the door open, do not
  build the loop yet).
- Accept: smoke covers: fresh snapshot, delta after known pos, limited on big gap, reset on ancient pos,
  two devices converge to identical state; retention job deletes only past-horizon rows (test).
- Review: read the response shape once against MSC4186 vocabulary in the report - naming should stay
  parallel (pos/initial/limited) so future readers can lean on the prior art.

## T-014 Idempotent send

- C-refs C4 | P0/M | depends: none (merges cleanly before or after T-011)
- Touch: migration (client_msg_id uuid column + UNIQUE(channel_id, sender_subject_id, client_msg_id));
  messages.rs post handler; realtime echo payload; apps/web send path.
- Spec: client generates UUIDv7 at send; POST carries it; on unique-violation the handler SELECTs and
  returns the existing row with 200 + {deduplicated:true}; echo event and POST response both include
  client_msg_id; client reconciles optimistic row by client_msg_id first, id second. Nullable column;
  legacy senders (none today) would still work.
- Accept: property-style test: fire the same send 10x concurrently -> exactly one row, all callers get
  the same id; client double-send test shows single bubble; smokes green.
- Review: confirm UUIDv7 generation lands in chat-core (single implementation), not copied per surface.

## T-015 chat-core package + web storage adapter

- C-refs C6 (part 1 of 2) | P0/M | depends: none
- Touch: new packages/chat-core (pnpm workspace member; wire into apps/web build); adapters/web-sqlite.
- Spec: TypeScript package owning schema (messages, channels, members, reactions, read_state, outbox,
  kv), StorageAdapter interface (open/tx/query/migrate), sync-loop skeleton, merge rules (seq order, LWW
  per field, tombstones), unread math, draft table. Web adapter: SQLite-wasm over OPFS (wa-sqlite or
  SQLocal), IndexedDB-backed VFS fallback when OPFS unavailable; feature-detect, one code path above the
  adapter. No UI imports here; node-runnable for tests (in-memory adapter included).
- Accept: package builds + unit tests run in CI (T-022 formalizes); demo script hydrates a store from a
  fixture event log; both web backends pass the same adapter conformance tests.
- Review: approve the package location/name once - everything after depends on it.

## T-016 Persistent outbox + connection-state contract

- C-refs C5, C22 | P0/M | depends: T-014, T-015
- Touch: packages/chat-core (outbox module + connection state machine); apps/web composer wiring.
- Spec: outbox rows {client_msg_id, channel_id, body, attachments?, created_at, attempts, state
  queued|sending|sent|failed}; strict per-channel FIFO; exponential backoff + jitter, cap then failed
  with manual retry; drain resumes on app start and on reconnect; sent rows deleted after echo reconcile.
  Connection state machine (online/connecting/syncing/offline + since) exposed as the single source for
  every surface's indicator.
- Accept: T-020's outbox property test passes; manual: toggle offline, send 3, kill tab, reopen offline
  (3 visible queued), reconnect -> exactly 3 delivered in order; indicator states verified in Playwright.
- Review: try to break it by hand for five minutes; this is the feature users will feel most.

## T-017 Store-driven rendering + drafts

- C-refs C6 (part 2), C7 | P0/L | depends: T-013, T-015
- Touch: apps/web/src/pages/Chat.tsx + lib/chat.ts (rendering reads move to chat-core selectors);
  drafts to store.
- Spec: unidirectional: socket/sync/REST write into the store, React renders from store subscriptions;
  the in-memory merge code in lib/chat.ts becomes chat-core's; drafts persist per channel across reload
  and restart (local only, never synced). Cold open offline renders cached channels + timelines. Do NOT
  restructure components yet (that is T-042); keep the diff mechanical.
- Accept: offline cold open shows history; Playwright offline suite (T-021) core scenarios pass; no
  regression in the existing browser flows (login -> channel -> send -> edit -> react).
- Review: click through the app for feel: nothing should look different online - that is the point.

## T-018 Single account-scoped multiplexed socket

- C-refs C15 | P0/M | depends: T-012
- Touch: services/chat/src/realtime.rs (accept account socket + subscribe/unsubscribe frames; membership
  filter server-side), notify.rs (fold into account socket), apps/web socket layer, call ring path.
- Spec: one ws per device: client subscribes the visible channel(s); server pushes subscribed channel
  events + always-on account events (dm/channel membership, notify summaries, call signals - fixes ring
  requiring the channel open). Keep legacy per-channel path alive behind the T-030 version gate until
  desktop/mobile ship, then remove.
- Accept: two-tab test: DM ring arrives with the channel closed; event delivery equivalence vs old path
  (same fixture, same store state); socket count per tab == 1 in metrics.
- Review: watch prod ws gauge halve; approve the removal date for the legacy path.

## T-019 Socket hardening

- C-refs C17, C18, C19, C20 | P1/M | depends: T-018
- Spec: resume hint (client sends last account seq on connect; server pushes the delta immediately);
  bounded per-connection send queue with drop-and-mark -> client forced into sync (lagged counter
  metric); {v, type} on every frame + unknown-field tolerance documented; Origin allowlist, max frame
  size, per-socket message-rate cap (shares T-001 buckets).
- Accept: lag test (slow consumer) triggers drop-and-mark and full recovery via sync; fuzz basics from
  T-046 reserved; origin reject test.
- Review: none special; read the lagged-counter dashboard after a week.

## T-020 Convergence + outbox property suites

- C-refs C76, C77 | P0/M | depends: T-016, T-017
- Touch: packages/chat-core tests (fast-check or hand-rolled PRNG cases).
- Spec: generator produces random event logs + delivery schedules (dup frames, gaps forcing sync, crash/
  restart points, interleaved outbox sends with injected network failures); invariant 1: final store ==
  reference reduction of the log; invariant 2: exactly one server row per client_msg_id, no lost sends.
  Seeded + reproducible; failure prints the schedule.
- Accept: 1,000 cases green in CI under a minute-ish; a deliberately introduced merge bug is caught
  (prove once, then revert).
- Review: none - but never allow this suite to be skipped "temporarily".

## T-021 Offline end-to-end Playwright suite

- C-refs C79 | P0/M | depends: T-017
- Touch: apps/web/e2e (new), CI workflow job against the staging/dev stack.
- Spec: scenarios: offline mid-send; reload while queued; token expiry during sleep then wake; 3-day gap
  (fixture-injected events) reconnect -> converged timeline; connection indicator transitions. Uses
  Playwright network interception + context.setOffline.
- Accept: suite green in CI on the seeded stack; each scenario asserts store state, not just pixels.
- Review: watch the recording of the mid-send kill scenario once - satisfying and convincing.

## T-022 chat-core as CI citizen

- C-refs C83 | P1/S | depends: T-015
- Spec: package gets its own required CI job (typecheck, unit, property) wired into the existing gate
  path next to cargo/clippy and apps/web tsc/vite; caf gate config updated so chat-core failures block
  exactly like service failures.
- Accept: a red chat-core test blocks the branch; job time budget < 3 min.
- Review: none.
