# CHAT enterprise-grade plan: offline-online sync + web/desktop/mobile go-live (2026-07-06)

Purpose: make CHAT the first module of CyberOS to go live as a daily tool for the whole company, on web,
desktop, and mobile, with smooth online-offline sync and an enterprise-grade reliability, security, and
compliance bar. This report is the execution source: investigate here once, then build from the numbered
recommendations (C1-C147) and the phased roadmap without re-auditing.

Relation to prior work. CHAT-AUDIT-2026-07-02.md (feature inventory) and CHAT-DEEP-AUDIT-2026-07-03.md
(bug/UX overhaul, backlog cleared on main) are both absorbed; nothing here repeats a shipped item. The
platform-wide plan docs/strategy/cyberos-deep-audit-and-auto-evolution-plan-2026-07-06.md (R1-R52) stays
authoritative for cross-service work; where a C-item depends on an R-item it says so. Method: two
code-reading passes over services/chat (all 21 src modules, migrations 0001-0013, tests), apps/web,
apps/desktop, apps/web/capacitor.config.ts, deploy/vps, plus primary-source research (Matrix MSC4186
simplified sliding sync, Tauri 2 plugin set, WebKit declarative web push, ElectricSQL/PowerSync,
MLS RFC 9420, Vietnam PDP Law 91/2025/QH15). Disputed agent findings were re-verified against the code
before inclusion (two false claims rejected: token refresh and browser notifications both exist).

## 1. Verified current state (what we build on)

Backend (services/chat, Rust/axum, ~5,500 lines, 46 endpoints, 13 migrations):

- Channels (group/dm kinds), messages with threads (parent_id), soft delete (deleted_at), edits
  (overwrite, no history), reactions, mentions, multi-attachment junction, read receipts, unread + mention
  counts, notify prefs with mute, devices registry, VN accent-insensitive search, translate + AI passthrough
  to ai-gateway, consent-gated capture to BRAIN, hash-chained audit log read.
- Realtime: one websocket per open channel plus a notify socket; events for message/edited/deleted/
  reaction_changed/read/typing/presence/signal/kicked. In-process Hub, Notifier, Presence: single replica
  enforced and documented in deploy/vps/docker-compose.p0*.yml.
- Auth: RS256 verify via JWKS (boot fetch + interval refresh), audience check opt-in, per-transaction RLS
  GUC for tenant scoping. Attachments: db|fs backend, 50 MB cap, safe content-type allowlist, nosniff,
  reference-counted purge.
- Ordering and sync today: messages are ordered and paginated by created_at (uuid PK, no monotonic
  sequence). No idempotency key on POST message. No delta endpoint (no events?since=). Reconnect
  correctness comes from the client refetching the live tail and merging by id.
- Push: device registry + intent logging only; no APNs/FCM delivery. Offline detection via in-process
  presence.
- No rate limiting. Tracing logs but no metrics, no OTel export, no error tracker. Fixed DB pools.

Web client (apps/web, React/Vite, Chat.tsx ~1,240 lines):

- Optimistic send with temp client id + pending/failed flags (in memory only), reconnect refetch + merge,
  15s unread polling, token refresh via refresh_token grant, rich text (Slack-scale hand-written parser,
  XSS-safe by construction), mentions, emoji picker, quick switcher, thread panel, i18n en/vi, browser
  Notification API wired, update banner via version.json polling, service worker that is app-shell only
  (network-first, /v1/* excluded), manifest for installable PWA.
- No client message store (no IndexedDB/SQLite), drafts are component state and die on switch/reload,
  no persistent outbox: a message sent while offline or mid-crash is lost silently after the in-memory
  retry state goes away.
- Calls: 1:1 WebRTC with signaling over the channel socket, Google STUN only, no TURN, ring only if the
  callee has that channel open, no call history.

Desktop (apps/desktop): Tauri 2 shell that loads https://os.cyberskill.world/web/ remotely. Updater plugin
declared; no tray, no native notifications, no deep links, no offline behavior, csp: null.

Mobile: apps/web/capacitor.config.ts exists; ios/ and android/ were never generated; release.yml jobs are
gated behind MOBILE_RELEASE and signing secrets. Nothing installable today.

Deploy: single-origin p0 compose (auth 7700, chat 7720, Caddy TLS), Supabase Postgres, attachment volume,
no Prometheus/Loki/OTel/Sentry, no external synthetic probes.

FR state: FR-CHAT-101 done (native skeleton). Drafts: 102 Slack import, 103 Zalo import, 104 real push,
105 DSAR export, 106 @lumi assistant.

Strengths to protect: single-binary simplicity, per-tenant RLS everywhere, hash-chained audit, consent-gated
capture, XSS-safe rich text, VN-first search and i18n, and a working optimistic-send UX. None of the work
below should regress these.

## 2. The bar: what "enterprise-grade daily tool" means here

Measurable exit bar for company-wide go-live (referenced by the roadmap in section 6):

- Correctness: no message loss or duplication under crash, offline, retry, or reconnect. A message accepted
  by the server exactly once regardless of how many times the client retries (idempotent send). Client
  state converges to server state after any disconnect length.
- Offline: the app opens instantly offline showing full cached history for subscribed channels; composing,
  sending (queued), reading, and searching cached content all work; the queue drains automatically on
  reconnect in order, with per-message status visible.
- Latency SLOs (in-region): p95 send-to-echo under 250 ms; p95 catch-up sync after a 7-day gap under 2 s
  for the active channel window; cold app open to usable under 1.5 s from cache.
- Availability: 99.9% monthly for send/receive; error budget tracked; externally probed.
- Security: rate-limited, abuse-resistant, audited, key rotation safe, device/session revocation effective
  within 60 s on live sockets.
- Compliance: Vietnam PDP Law 91/2025/QH15 (in force since 2026-01-01) duties mapped and operational:
  consent records, DSAR export, retention, breach process, cross-border transfer assessment for Supabase.
- Platforms: installable web/PWA, signed desktop builds with tray + native notifications + deep links,
  store-distributable mobile builds with real push.

## 3. Core design: the sync layer (read this before any C-item in group A)

The single biggest gap between today and the bar is a real synchronization protocol. Today correctness
depends on "refetch tail and merge by id", which cannot express edits/deletes/reactions that happened in
unopened channels, cannot resume cheaply after long gaps, and gives the client no durable position. The
design below is deliberately small, server-authoritative, and native to the existing Rust service. It
borrows the proven shape of Matrix simplified sliding sync (MSC4186) - position token, windowed lists,
room subscriptions, initial/limited flags - without adopting Matrix.

3.1 Server-side event log and sequence:

- Add a per-tenant monotonic change log table chat_events(tenant_id, seq bigint, channel_id, kind,
  payload jsonb, created_at), seq allocated from a per-tenant sequence, with (tenant_id, seq) primary
  ordering and an index on (tenant_id, channel_id, seq). Every mutation that clients must converge on
  writes one row in the same transaction as the mutation itself: message_created, message_edited,
  message_deleted, reaction_set (absolute snapshot), read_marker_moved, member_added/removed/role_changed,
  channel_created/updated/archived, attachment_linked, pin_set, pref_changed. Presence and typing never
  enter the log (ephemeral only).
- Expose the head position as pos (an opaque string wrapping tenant seq). All realtime frames carry the
  seq of the event they describe so a client can advance its cursor from the socket alone.
- Compaction and reset semantics: keep the log N days (start 30). If a client presents a pos older than
  the horizon, reply with a typed reset error (mirror of M_UNKNOWN_POS) and the client falls back to a
  windowed full resync. This caps storage and makes the failure mode explicit instead of silent.

3.2 The sync endpoint:

- POST /v1/chat/sync with body { pos?, lists?, subscriptions?, timeout_ms? }. Lists select channels by
  rule (all-mine sorted by last activity, range [0..n], per-channel timeline_limit and a fields filter);
  subscriptions pin specific channels the UI has open. Response: new pos, per-channel deltas since the
  client's pos (or a snapshot with initial:true when first sent or after reset), a limited flag plus
  prev_batch-style cursor when the gap exceeds the timeline_limit, and account-level items (channel list
  changes, prefs, unread summary). With timeout_ms > 0 it long-polls, which doubles as the degraded
  transport when websockets are blocked by a corporate proxy.
- This one endpoint serves: cold start (fast first paint of the top-N channels), reconnect catch-up (delta
  by pos), long-offline recovery (bounded windows + limited), and multi-device consistency. The websocket
  remains the low-latency push path; sync is the source of truth for convergence.

3.3 Idempotent, durable send (the outbox):

- Client generates a UUIDv7 client_msg_id when the user hits send and persists the message to a local
  outbox before any network attempt. POST /v1/chat/channels/:id/messages carries client_msg_id; server
  enforces UNIQUE (channel_id, sender_subject_id, client_msg_id) and on conflict returns the existing row
  (200, not 409), making retries harmless. The echo event and the POST response both carry client_msg_id
  so any device of the sender reconciles the optimistic row.
- Outbox drain: strictly per-channel FIFO, exponential backoff with jitter, capped attempts before
  surfacing "failed - retry", survives app restart, visible per-message states queued/sending/sent/failed.
  UUIDv7 also gives the outbox a stable local sort before the server assigns seq.

3.4 Client store, shared across all three surfaces:

- Extract a platform-neutral TypeScript package (suggested: apps/web/src/chat-core or packages/chat-core)
  owning: schema, sync loop, outbox, merge rules, unread math, draft persistence. UI layers (React web,
  the same React bundle in Tauri, Capacitor) consume it through one interface.
- Storage adapter per platform: web = SQLite-in-WASM over OPFS (wa-sqlite or SQLocal) with an IndexedDB
  fallback; desktop = tauri-plugin-sql (sqlx/SQLite, official plugin); mobile = @capacitor-community/sqlite.
  Same SQL schema everywhere: messages, channels, members, reactions, read_state, outbox, kv(pos, ...).
- Merge rules are trivial by design because the server is authoritative: apply events in seq order,
  last-writer-wins per field, deletes are tombstones, reactions arrive as absolute snapshots (already the
  case since the 07-03 fix). No CRDTs needed for an append-mostly timeline; keep that complexity out.

3.5 Build vs adopt (decision D2 in section 7, recommendation: build):

- Evaluated: ElectricSQL 1.x (GA since 2025-03, read-path sync of Postgres shapes over HTTP, writes stay
  yours, has a Supabase integration; now oriented toward its Durable Streams/TanStack DB stack) and
  PowerSync (SQLite-first sync service + client SDKs, Supabase-friendly, self-hostable). Both are credible;
  both add a second moving service, a second auth surface to your RLS model, and a generic-sync abstraction
  the chat timeline does not need. Zero (Rocicorp) and LiveStore remain younger. CRDT engines
  (Automerge/Yjs) solve concurrent text editing, not server-ordered timelines; keep them out of chat
  (they may later fit a docs module).
- Chat's log is the easy case for hand-rolled sync: one authority, one ordering key, snapshot-able state.
  The native path above is roughly: one table, one endpoint, one unique index, one client package - and it
  keeps the single-binary deploy story that is currently a CyberOS strength. Adopt an engine later only if
  CyberOS-wide local-first (beyond chat) becomes a goal; the chat-core package isolates that swap.

## 4. Recommendations, group A-H (sync, transport, scale, data, delivery, security, E2EE, compliance)

Priorities: P0 = before the team lives in it daily; P1 = before company-wide go-live; P2 = scale and polish
after go-live; P3 = later bet. Effort: S under a day, M days, L a week+ (one engineer + agent loop).

### A. Sync and offline core

- C1 [P0/M] Add chat_events log + per-tenant seq exactly as 3.1, written transactionally with each
  mutation. Backfill is unnecessary (start the log at deploy time; older history serves via existing
  pagination).
- C2 [P0/M] Implement POST /v1/chat/sync per 3.2 with pos, subscriptions, initial, limited, reset error.
  Ship lists support in a second pass (C9); subscriptions + account stream are enough for v1.
- C3 [P0/S] Stamp every websocket frame with seq (and channel_id) so the socket advances the same cursor
  the sync endpoint uses; on any gap detected (next seq != last+1 per channel) trigger a sync call instead
  of trusting the frame stream.
- C4 [P0/M] client_msg_id idempotent send + UNIQUE index + echo reconciliation per 3.3. This is the
  highest-value single change for perceived reliability.
- C5 [P0/M] Persistent outbox in the client store with per-channel FIFO drain, backoff, restart survival,
  and visible message states. Acceptance: kill the app mid-send, reopen offline, reconnect - the message
  arrives exactly once.
- C6 [P0/L] chat-core package + SQLite storage adapters per 3.4; move message/channel/read state and
  drafts into it; render from the store, network only updates the store (unidirectional).
- C7 [P0/S] Draft persistence per channel in the store (survives reload and app restart, syncs nothing).
- C8 [P1/M] Cold-start snapshot path: first sync (no pos) returns top-N channels by activity with
  timeline_limit messages each, then the client widens the window in the background (MSC4186 range-growing
  pattern). Target: usable UI in one round trip.
- C9 [P1/M] Lists in sync (windowed channel collections with filters: all, dms, unread-only) so large
  tenants never pay full-account sync; count returned per list for scrollbar math.
- C10 [P1/S] Event-log retention job (30 days) + typed reset + client full-resync fallback path, tested.
- C11 [P1/M] Multi-device read-state convergence: read markers flow as events (they already exist as
  receipts) and the store reconciles unread badges from the log, replacing the 15 s polling loop.
- C12 [P1/S] Search over the local store when offline (SQLite FTS5 on cached messages), clearly labeled
  as cached results; server VN search remains the online path.
- C13 [P2/M] Attachment offline policy: thumbnails cached always; full files on tap with size budget and
  LRU eviction; queued uploads in the outbox with resumable upload (pairs with C31).
- C14 [P2/S] Sync conformance fixture: a recorded event-log scenario replayed against chat-core in CI
  asserting byte-identical final store state (the convergence contract, see C76).

### B. Realtime transport

- C15 [P0/M] Replace per-channel sockets + notify socket with ONE account-scoped multiplexed socket per
  device: subscribe/unsubscribe frames per channel, server filters by membership. Fixes at the root: calls
  that only ring when the channel is open, presence blind spots, N sockets per tab, and per-channel
  reconnect storms. Keep the old paths during migration behind a version flag (C142).
- C16 [P0/S] Application-level heartbeat (ping/pong with deadline) + server idle reaper; today a dead
  uplink lingers and presence lies.
- C17 [P1/S] Resume-aware reconnect: exponential backoff with jitter and a resume hint (last seq) so the
  server can push the delta immediately on reattach, before the client even calls sync.
- C18 [P1/S] Backpressure policy: bound per-connection send queues; on overflow, drop-and-mark (force the
  client into sync catch-up) rather than unbounded buffering; expose a lagged counter metric.
- C19 [P1/S] Envelope versioning: add v and type to every frame now, define unknown-field tolerance, and
  document the frame catalog in the OpenAPI/AsyncAPI artifact (C133). Protocol changes stop being breaking.
- C20 [P1/S] Origin allowlist + max frame size + message-rate cap per socket (pairs with C46).
- C21 [P2/M] Long-poll fallback transport via sync timeout_ms for networks that kill websockets (some VN
  corporate/hotel networks do); auto-detected, surfaced in the connection indicator.
- C22 [P2/S] Connection-state UI contract in chat-core: online / connecting / syncing / offline with
  timestamps, one source of truth for every surface's indicator.

### C. Scale-out and fan-out

- C23 [P1/M] Implement the R4 seam now: route Hub/Notifier publishes through a trait; first backend stays
  in-process, second backend Postgres LISTEN/NOTIFY. This unblocks 2+ replicas and zero-downtime deploys
  (C143) without new infra.
- C24 [P2/M] Shared presence in Postgres (or Redis when adopted): per-connection rows with TTL heartbeat,
  so push offline-detection (C41) stays correct across replicas.
- C25 [P2/M] Redis (or NATS) pub/sub backend behind the C23 trait once message volume or replica count
  makes LISTEN/NOTIFY payload limits (8 KB) or single-DB fan-out a bottleneck; measured, not assumed.
- C26 [P2/S] Load-shed order documented and implemented: typing first, then presence, then receipts;
  message events never shed.
- C27 [P3/M] Partition chat_messages and chat_events by tenant hash or month when row counts approach
  10^8; decision gated on the C71 capacity dashboard, not calendar.

### D. Data model and storage

- C28 [P0/S] Migrate any residual BYTEA attachment rows to the fs store and drop the db backend in prod
  config (explicitly deferred on 07-03; close it before go-live).
- C29 [P0/S] Edit history table chat_message_revisions(message_id, rev, body, edited_at, editor) written
  on every edit; retention aligned with C56. Enterprise expectation and a PDPL/DSAR completeness need.
- C30 [P1/M] Object storage for attachments behind the existing storage trait: S3-compatible (Supabase
  Storage or R2/MinIO), presigned upload + download, keep fs as dev backend. Unblocks multi-replica (C23)
  and off-VPS durability.
- C31 [P1/M] Resumable uploads (tus or S3 multipart) for VN mobile networks; chunk size tuned for 3G/4G.
- C32 [P1/S] Server-side thumbnail + image normalization pipeline (strip EXIF/GPS, re-encode, cap
  dimensions), original kept per policy; stops location leaks from photos.
- C33 [P1/S] Antivirus hook on upload (ClamAV container in compose; async scan, quarantine state on the
  attachment row) - a security company shipping unscanned file sharing is a story we do not want.
- C34 [P1/S] Attachment retention + orphan GC job (reference counting already exists; add the sweeper and
  a metric for orphaned bytes).
- C35 [P2/S] Per-tenant storage quotas + per-message attachment count/size limits enforced server-side.
- C36 [P2/M] Cold storage tier: messages older than N months move to an archive table (or partition) that
  sync never touches and search hits lazily; keeps the hot working set small.
- C37 [P2/S] Nightly logical dump of chat DB + attachment store to off-site (extends R33), with a restore
  drill that reconstructs a tenant end-to-end (C144).

### E. Delivery and notifications

- C38 [P0/L] Real push relay (FR-CHAT-104): small worker consuming push intents; FCM HTTP v1 for
  Android/web, APNs token-based for iOS/macOS. Payload stays privacy-preserving (title + sender, no body,
  per the FR). Device tokens already registered via devices.rs.
- C39 [P1/S] Web push for the PWA: standard VAPID web push, and adopt the declarative web push JSON shape
  (shipped in iOS/iPadOS 18.4 and macOS 15.5) so Apple-platform PWA pushes work without service-worker
  execution and fall back cleanly elsewhere.
- C40 [P1/S] Collapse keys + per-channel coalescing so a burst becomes one updated notification, not
  twenty; unread-count badge push (APNs badge, FCM notification count).
- C41 [P1/S] Offline detection for push moves from in-process presence to the shared presence set (C24
  dependency); until then document the single-replica constraint in the push worker.
- C42 [P1/S] Quiet hours / DND schedule per user (extends prefs.rs mute model), evaluated server-side at
  fan-out so every surface obeys it; default VN work calendar template.
- C43 [P2/S] Missed-activity email digest for users offline more than N hours (via existing mail path
  from the lead fan-out work), per-user opt-out.
- C44 [P2/S] Delivery receipts per device (delivered state between sent and read) if the team wants
  WhatsApp-style ticks; the event log makes this cheap - product call, off by default for calm.
- C45 [P3/S] Zalo/SMS bridge notification for critical mentions as a VN-market differentiator, opt-in.

### F. Security hardening

- C46 [P0/M] Rate limiting at two layers: per-subject token bucket in chat (send, search, upload,
  channel-create) and per-IP at Caddy; 429 with Retry-After; metrics + alerts on limiter hits. Today there
  is none.
- C47 [P0/S] Enforce JWT audience always (currently opt-in) and add issuer pinning; reject tokens minted
  for other services.
- C48 [P0/S] JWKS rotation hardening: on unknown kid, force an immediate JWKS refetch (bounded) before
  rejecting - closes the deferred key-rotation break window without restarts.
- C49 [P0/S] Kill-switch coverage: on subject revoke or tenant suspend (AUTH already emits), close all
  live sockets for that subject within 60 s (extend the Kicked control-event mechanism account-wide).
- C50 [P1/S] Set a real CSP for the web app and the Tauri shell (csp is null today in tauri.conf.json);
  connect-src pinned to the app origin + ws; frame-ancestors none.
- C51 [P1/S] Upload hardening beyond C32/C33: magic-byte sniffing must match declared type, SVG served as
  download-only, HTML/JS types forbidden, archive bombs capped by decompressed-size probe.
- C52 [P1/M] Link previews (C114) fetched only by a server-side SSRF-safe fetcher: private-range IP deny,
  redirect cap, content-type + size caps, per-domain rate limit, previews cached and stripped of cookies.
- C53 [P1/S] Session/device management UI: list active devices/sockets (devices.rs + presence), revoke
  one, sign-out-everywhere; pairs with AUTH sessions.
- C54 [P1/S] Secrets hygiene: push credentials, TURN secrets, and AI keys via env-injected secrets only;
  add gitleaks to CI for the repo (aligns with the platform audit).
- C55 [P1/M] Dependency + supply chain: cargo audit/deny and npm audit gating CI for chat + web; SBOM
  emitted per release; Tauri updater keys stored offline.
- C56 [P1/S] Data minimization defaults: retention policy per tenant (message TTL optional), attachment
  TTL, capture already consent-gated - wire all three to one policy object the admin console can edit.
- C57 [P2/M] External penetration test of chat + auth before company-wide go-live; scope includes ws
  auth, RLS bypass attempts, file upload, SSRF via previews.

### G. Encryption roadmap (decision D1 in section 7)

- C58 [P1/S] Now: document the honest model - TLS in transit, Postgres/Supabase encryption at rest,
  server-readable by design because BRAIN capture, translate, AI assist, search, and compliance export all
  read plaintext. Publish this in the product security page; ambiguity here hurts a security brand more
  than the absence of E2EE.
- C59 [P2/M] Add application-layer encryption at rest for attachment bytes (per-tenant keys, envelope
  encryption) - cheap, no product tension.
- C60 [P3/L] E2EE track: MLS (RFC 9420, the IETF standard adopted by Wire and Cisco Webex, used by
  Discord's DAVE for calls, and the base of IETF MIMI interop work) for designated E2EE DM/channel types
  where capture, AI, and server search are structurally disabled and the UI says so. Per-channel policy,
  default off in workspace channels. Only start after D1 is decided; do not hand-roll crypto.
- C61 [P3/M] If E2EE ships, add key transparency/backup UX (device verification, recovery) as its own
  project; half-shipped E2EE is worse than none.

### H. Compliance and governance (Vietnam PDP Law 91/2025/QH15, in force 2026-01-01)

- C62 [P0/S] Map chat data to PDPL categories with counsel: basic vs sensitive personal data in messages,
  consent records for capture (exists) and for processing employee comms (BRAIN plan overlap); produce the
  DPIA-style processing dossier the law expects. (Legal review required; this report is not legal advice.)
- C63 [P1/M] DSAR export (FR-CHAT-105): per-subject export of authored messages + attachments + audit
  slice, admin-triggered, rendered to a signed archive; deletion/anonymization workflow honoring the
  hash-chained audit (tombstone content, keep chain integrity).
- C64 [P1/S] Cross-border transfer assessment: chat data currently sits in Supabase (region per project) -
  record the region, run the PDPL transfer impact assessment, and decide whether a VN-region or self-hosted
  Postgres is required for employee data; revisit the AGE-removal note that any managed Postgres now works.
- C65 [P1/S] Retention operationalized (extends R45): default retention windows per data class (messages,
  attachments, events log C10, push intents, presence traces), enforced by jobs, surfaced in admin.
- C66 [P1/S] Breach playbook: detection sources (C69 alerts), 72h-style notification steps per PDPL,
  responsible owner, dry-run once.
- C67 [P1/S] Legal hold: per-subject or per-channel hold flag that suspends retention/deletion, logged to
  the audit chain.
- C68 [P2/S] Admin data-governance panel in the console app module: retention, holds, DSAR, consent
  states, capture policy per tenant - one screen, backed by the C56 policy object.

## 5. Recommendations, group I-Q (observe, test, web, desktop, mobile, product, calls, API, ops)

### I. Observability and SLOs

- C69 [P0/M] Metrics + alerts (extends R29/R38): axum middleware exporting RED per route, ws connection
  gauge, per-channel fan-out latency histogram, outbox-age gauge (from client telemetry), limiter hits,
  push relay success/failure; Prometheus + Grafana + Alertmanager in the p0 compose; page on send-error
  rate, ws-connect failure, and DB pool saturation.
- C70 [P1/M] OTel tracing (R37): trace ids from Caddy through chat to Postgres and ai-gateway; sample
  100% of errors, 1% of ok.
- C71 [P1/S] Capacity dashboard: message and event rows per tenant, storage bytes, growth rate, largest
  channels; the trigger source for C27 partitioning and C36 archiving.
- C72 [P1/S] SLOs written down (R39) with the section-2 targets, an error budget, and a weekly review
  habit; releases pause when the budget is spent (see C141).
- C73 [P1/S] Error tracking both sides (R40): GlitchTip or Sentry for the web/desktop/mobile clients and
  Rust panics; release-tagged so a bad build is visible within minutes.
- C74 [P0/S] External synthetic probe (R30): login, open socket, send, receive round trip against prod
  every minute from outside the VPS; alert to the team channel and email.
- C75 [P2/M] Load tests in CI (nightly): k6 or artillery scenario with 10k idle sockets + 200 msg/s burst
  on one tenant; regression gate on p95 send-to-echo and memory ceiling.

### J. Testing discipline

- C76 [P0/M] Convergence property test: drive chat-core with randomized event interleavings, duplicates,
  reorders within the socket-vs-sync race, and crash/restart points; assert the local store equals a
  reference reduction of the event log every time. This is the contract that makes offline trustworthy.
- C77 [P0/S] Outbox property test: random network failure injection around POST; assert exactly-one
  server row per client_msg_id and eventual sent state.
- C78 [P1/S] Fuzz the ws + sync inputs (arbitrary JSON frames, truncated bodies, absurd pos values);
  service must never panic and always answer typed errors.
- C79 [P0/M] Offline end-to-end suite (Playwright): toggle offline mid-send, reload while queued, sleep
  the tab through a token expiry, reconnect after 3 days of simulated events; run against a seeded stack
  in CI (dev-up.sh already provides one).
- C80 [P1/S] Migration gate: apply 0001..latest on a throwaway DB in CI (exists in spirit; make it a
  required check) plus a reverse compatibility test that the previous release runs against the new schema
  (expand/contract discipline, see C143).
- C81 [P1/M] Chaos drill: kill the chat container under active traffic; assert zero loss/duplication by
  ledger comparison between client stores and the event log.
- C82 [P2/M] Soak: 24 h continuous traffic in staging with heap and map-size metrics flat (Hub reaping
  proved by numbers, not trust).
- C83 [P1/S] Make the chat-core package a first-class CI citizen: typecheck, unit, property suites gate
  merges the same way clippy gates the service.

### K. Web client

- C84 [P1/M] Split Chat.tsx (~1,240 lines) into feature modules over chat-core (timeline, composer,
  sidebar, panels); state moves to the store; components become renderers. Do this after C6, not before.
- C85 [P1/M] Virtualized message list with stable scroll anchoring (day of history without jank on a
  mid-range VN Android in Chrome); lazy-decode images; cap DOM nodes.
- C86 [P1/S] Remove residual polling: unread badges come from the event log (C11), version.json check
  stays but only on visibilitychange; no interval timers left in Chat.
- C87 [P1/S] PWA completion: offline start from cache (app shell exists; data arrives via chat-core),
  offline banner, install prompt on supported browsers, iOS add-to-home-screen hint.
- C88 [P1/S] Accessibility to WCAG 2.1 AA: keep the existing focus-trap/live-region work, add axe-core to
  CI and fix findings; full keyboard path for every action added in group N.
- C89 [P1/S] Performance budget: initial JS under a set cap, code-split AI/thread/call panels, LCP under
  2 s on a throttled profile in CI (Lighthouse assertion).
- C90 [P1/S] Web push wiring: VAPID subscription UI, declarative web push payload from the C38 relay so
  installed PWAs on iOS/iPadOS 18.4+ and macOS get notifications without service-worker execution;
  classic SW push everywhere else.
- C91 [P2/S] Daily-feel batch: paste-image upload, drag-drop target on the whole timeline, optional send
  sound (off by default), unread count in favicon and title, Vietnamese spellcheck attribute on the
  composer.

### L. Desktop (Tauri 2)

- C92 [P0/M] Bundle the built SPA into the app instead of loading the remote URL (remote stays as a
  fallback route); this is the precondition for any offline desktop behavior and removes the
  blank-window-when-offline failure.
- C93 [P0/S] Wire chat-core to tauri-plugin-sql (official, sqlx/SQLite) for the local store.
- C94 [P0/S] Native notifications via the notification plugin, with click-to-focus routing to the right
  channel; respect C42 quiet hours.
- C95 [P1/S] Tray icon with unread badge, close-to-tray, launch-minimized option.
- C96 [P1/S] Deep links (cyberos://channel/..., cyberos://message/...) via the deep-link plugin +
  single-instance plugin so links from email/browser land in the running app.
- C97 [P1/S] Finish the updater path already scaffolded in docs/deploy/RELEASE.md: signing keys generated and stored
  offline, stable + beta channels, update-available UX reusing UpdateBanner.
- C98 [P2/S] window-state persistence, global shortcut for the quick switcher, autostart opt-in
  (autostart plugin), macOS dock badge parity.
- C99 [P1/S] Security posture: set a strict CSP in tauri.conf.json (null today), narrow capability
  permissions to exactly the plugins used, keep withGlobalTauri false.
- C100 [P1/S] Native crash reporting (sentry-tauri community plugin or equivalent) feeding C73.
- C101 [P1/M] Desktop CI matrix: build + smoke (launch, login against staging, send/receive) on
  macOS/Windows/Linux via tauri-action on tags; artifacts attached to releases.

### M. Mobile (Capacitor first)

- C102 [P0/M] Initialize the platforms for real: cap add ios/android from apps/web (config exists),
  commit the native projects, wire the existing release.yml lanes, produce the first internal builds.
  Blocked only on the signing accounts/keys listed in docs/deploy/RELEASE.md - flag to Stephen early.
- C103 [P0/S] chat-core storage adapter on @capacitor-community/sqlite; verify OPFS-wasm fallback is
  never used inside the native webview.
- C104 [P0/M] Push: @capacitor/push-notifications registering FCM/APNs tokens into the existing
  /v1/chat/devices, delivered by the C38 relay; deep-link tap-through to channel; badge counts.
- C105 [P1/M] Mobile UX pass: safe-area insets, keyboard-aware composer, pull-to-refresh triggering sync,
  haptics on long-press actions (already designed for touch), 44 px targets audit.
- C106 [P1/S] Store readiness: privacy policy pages (PDPL-aligned, C62), Apple privacy nutrition labels
  and Play data-safety forms, account-deletion path (C63 dependency), TestFlight + internal track rollout.
- C107 [P2/M] Share-to-chat intent (share sheet -> channel picker) once the basics are stable.
- C108 [P3/S] Re-evaluate Tauri 2 mobile as a later consolidation (one Rust shell for all four targets);
  decision D5 - not now, Capacitor is the shortest path with the current React bundle.

### N. Product completeness for a daily driver

- C109 [P1/S] Pin messages per channel (pins list in the header, pin_set event exists in the C1 log).
- C110 [P1/S] Saved for later (private bookmarks) with a dedicated sidebar view.
- C111 [P1/S] Forward/share a message to another channel + quote-reply with backlink.
- C112 [P1/M] Ad-hoc group DMs (3-9 people, unnamed, kind='dm_group') reusing the advisory-lock create
  pattern from create_dm; today groups force named channels.
- C113 [P1/S] Scheduled send + remind-me-about-this (outbox rows with not_before; server job releases).
- C114 [P1/M] Link previews: server-side unfurl through the C52 SSRF-safe fetcher, stored once per URL,
  rendered as a compact card; per-channel toggle.
- C115 [P2/S] Slash commands (/mute, /call, /translate, /remind, /lumi) parsed client-side against a
  server-declared registry, so bots (C136) can register their own.
- C116 [P2/S] Custom emoji per tenant + skin-tone variants in the picker.
- C117 [P1/S] User status (focus/away/off-sick/on-leave with emoji + auto-clear) and DND that maps to
  C42; presence states beyond online/offline.
- C118 [P1/S] Edit-history viewer ("edited" affordance opens revisions from C29) - trust feature for a
  workplace tool.
- C119 [P2/S] Jump-to-date, sidebar sections (favorites/custom groups), mark-as-unread from any message.
- C120 [P1/S] Per-channel notification overrides UI (all/mentions/nothing exists server-side in prefs;
  surface it per channel and per device).
- C121 [P1/M] AI daily-driver set via ai-gateway: catch-me-up per channel (summarize since my last read),
  thread summary, action-item extraction posted as a checklist message; streaming responses (ai.rs is
  non-streaming today).
- C122 [P2/M] Semantic search alongside VN FTS: embed messages via the embed service into pgvector,
  hybrid rank, opt-in per tenant (capture-consent aware).
- C123 [P1/M] @lumi assistant (FR-CHAT-106): mention triggers CUO routing with channel context, replies
  in-thread, capture-consent respected; this is the differentiator no Slack clone has.
- C124 [P2/S] Message templates/snippets for support and sales replies (per-user + per-tenant shared).

### O. Calls

- C125 [P0/S] Deploy coturn (TURN/TLS) next to the stack and ship ICE config from the server; without
  TURN, calls fail on exactly the networks a VN company hits daily (carrier NAT). STUN-only today.
- C126 [P1/S] Ring path via the account socket (C15) + push wake (C38) so calls ring with the app closed
  or another channel open; missed-call chip + call log rows in the timeline.
- C127 [P1/S] Call quality telemetry: periodic getStats samples to the metrics pipe (C69) so "calls are
  bad" becomes a graph, not a vibe.
- C128 [P2/S] In-call screen share (getDisplayMedia) for 1:1 - cheap now that TURN exists.
- C129 [P3/L] Group calls/huddles via a self-hosted SFU (LiveKit is the pragmatic default; decision D4);
  do not attempt mesh group WebRTC.

### P. API and ecosystem

- C130 [P1/S] Adopt the unified error envelope (R1) in chat first: {code, message, retryable, details};
  clients branch on code, never on prose.
- C131 [P1/M] OpenAPI for all chat routes (utoipa or hand-written spec checked in CI against the router,
  R8); publish to the wiki.
- C132 [P1/M] Generated TypeScript client from the spec consumed by chat-core - removes hand-written
  fetch drift across web/desktop/mobile.
- C133 [P1/S] AsyncAPI-style frame catalog documenting every ws event and sync payload with version notes
  (the C19 artifact).
- C134 [P2/S] Incoming webhooks per channel (token URL, rate-limited, rendered as app messages with
  distinct styling).
- C135 [P2/S] Outgoing event subscriptions (HMAC-signed, retried with backoff, dead-letter after N) for
  integrations like CRM and CI notifications.
- C136 [P2/M] Bot/service accounts: AUTH-issued service tokens, bot badge in UI, slash-command
  registration (C115), replies via the normal message API.
- C137 [P2/M] Chat as MCP tools on the existing MCP gateway (post_message, search, summarize_channel,
  create_channel) so any agent in the CyberOS ecosystem can act in chat under tenant policy - unique
  CyberOS advantage, cheap to expose.
- C138 [P2/L] Imports: Slack export importer first (FR-CHAT-102, idempotent checkpointed), Zalo bundle
  second (FR-CHAT-103) - decisive for migrating the company's history in and making chat the only tool.

### Q. Ops, release, and rollout

- C139 [P0/S] Staging profile of the p0 compose (separate domain + DB) so gate-then-prod stops being the
  only path; seeded demo tenant for the e2e suites (C79, C101).
- C140 [P1/S] Per-tenant feature flags (tiny table + cached read) gating every group-N feature and the
  C15 transport migration; flags flip without deploys.
- C141 [P1/S] Canary rollout habit: CyberSkill tenant first, watch C69/C73 for a day, then everyone;
  release pauses automatically when the C72 error budget is spent.
- C142 [P1/S] Version-skew policy: /version exposes min_supported_client; older clients get the existing
  UpdateBanner in blocking mode; ws frames carry v (C19) so old clients degrade to sync-only instead of
  misparsing.
- C143 [P1/S] Zero-downtime deploys once C23 lands: two replicas, drain-on-SIGTERM finishes in-flight
  sends, sockets reconnect to the survivor; until then, deploys stay in the low-traffic window.
- C144 [P1/M] Backup/restore drill (extends C37/R33): quarterly restore of DB + attachments into staging,
  timed, documented; an untested backup is a hope, not a plan.
- C145 [P1/S] Runbooks for the five likely incidents (ws storm, DB failover, push outage, storage full,
  bad deploy) + an incident comms template posted in chat itself and mirrored to email if chat is the
  casualty.
- C146 [P2/S] Dependency update cadence: weekly automated PRs (Renovate-style) through the caf gate;
  SBOM per release (pairs with C55).
- C147 [P0/S] Go-live checklist as a living doc in docs/deploy: every P0/P1 C-item with owner + status;
  go-live is a checklist state, not a feeling.

## 6. Roadmap to first go-live

Phase 0 - safety rails (week 1). C46-C49 (limits, aud, JWKS, kill-switch), C28, C29, C16, C74, C69
minimal, C139, C147. Exit: rate limiter live, revoke closes sockets under 60 s, external probe green,
staging exists.

Phase 1 - the sync core (weeks 2-4). C1-C7 server+client core, C3 seq-stamped frames, C15 single socket,
C76/C77/C79 test contracts. Exit: the section-2 offline acceptance scenario passes on web (kill mid-send,
reopen offline, reconnect - exactly once), convergence suite green in CI, p95 send-to-echo measured.

Phase 2 - push + desktop GA (weeks 5-7). C38-C42 push relay + quiet hours, C90, C92-C97 + C99 desktop,
C142, C73. Exit: signed desktop builds auto-updating, notifications on closed app, the team defaults to
desktop daily; this is the "first go live" moment for web + desktop.

Phase 3 - mobile beta + storage + compliance (weeks 8-10). C102-C106 mobile, C30-C34 attachments to
object storage + hygiene, C23/C24 fan-out seam + shared presence, C63-C67 PDPL operational items, C125
TURN, C11/C84-C89 web polish. Exit: internal TestFlight/Play track installs with working push, PDPL
dossier + DSAR path done, calls survive carrier NAT.

Phase 4 - depth and scale (quarter). Group N product batch by daily-pain order (start: C109-C114, C117,
C118, C120, C123), the C121/C122 AI set, C134-C138 ecosystem + imports, C75/C81/C82 load-chaos-soak,
C25/C27/C36 as C71 signals demand, D1 then possibly C59-C61.

Sequencing rule: nothing in groups K-N ships to everyone before Phase 1 exits - features stacked on an
unconverged sync layer create bug reports that all read "chat lost my message".

## 7. Decisions that are Stephen's to make (genuine forks)

- D1. Recording policy vs E2EE: BRAIN capture, AI assist, search, and compliance export require
  server-readable content; E2EE structurally removes it. Recommendation: default workspace channels stay
  recorded-with-consent (current model, stated plainly per C58), plus a later optional E2EE channel class
  (C60) where capture/AI/search are off. Needed before any E2EE work starts.
- D2. Build the thin sync layer natively (section 3 recommendation) vs adopting PowerSync/Electric.
  Revisit trigger: if CyberOS later wants generic local-first across modules, swap behind chat-core.
- D3. Attachment object store: Supabase Storage (least ops, ties residency to Supabase region), MinIO on
  the VPS (full control, VN residency, more ops), or R2 (cheap egress, foreign region). Interacts with D6.
- D4. Group-call SFU: self-hosted LiveKit when group calls become real demand, or defer group calls
  entirely for v1 (1:1 + screen share only).
- D5. Mobile track: Capacitor now (recommended, reuses the React bundle) with a later look at Tauri 2
  mobile consolidation.
- D6. Data residency for employee communications under PDPL: keep Supabase (record region + transfer
  assessment, C64) vs migrate chat DB to VN-hosted Postgres. Counsel input required; the AGE removal
  already made the DB portable.

## 8. Sources consulted (primary)

- Matrix MSC4186 simplified sliding sync (pos token, lists/subscriptions, initial/limited semantics):
  https://github.com/matrix-org/matrix-spec-proposals/blob/main/proposals/4186-simplified-sliding-sync.md
- WebKit, "Meet Declarative Web Push" (shipped iOS/iPadOS 18.4, macOS 15.5 beta):
  https://webkit.org/blog/16535/meet-declarative-web-push/
- Tauri 2 official plugin set (SQL, notification, updater, deep-link, single-instance, tray, window-state,
  autostart, biometric): https://v2.tauri.app/plugin/
- ElectricSQL (1.0 GA 2025-03, read-path Postgres sync, Supabase integration, Durable Streams):
  https://electric-sql.com/
- PowerSync (SQLite-first sync service, Supabase-friendly, self-hostable): https://www.powersync.com/
- MLS, RFC 9420 (IETF Messaging Layer Security; adopted by Wire, Cisco Webex; base of IETF MIMI):
  https://www.rfc-editor.org/rfc/rfc9420
- Vietnam Personal Data Protection Law No. 91/2025/QH15, in force 2026-01-01 (verify specifics with
  counsel; this report is engineering guidance, not legal advice).

Report author: Claude (Cowork deep-dive session, 2026-07-06), verified against the code at the paths named
in section 1. Execute from this report; re-audit only what a C-item's acceptance check fails.
