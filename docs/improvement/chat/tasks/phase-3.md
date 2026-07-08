# Phase 3 - mobile, storage, compliance, scale seam (T-033..T-047)

Exit bar (report section 6): internal TestFlight/Play-track installs with working push, PDPL dossier +
DSAR path done, attachments on object storage, calls survive carrier NAT.

## T-033 Capacitor init + release lanes live

- C-refs C102 | P0/M | depends: none; blocked:input (Apple Developer + Play Console accounts, signing
  keys/keystore - exact list in RELEASE.md)
- Touch: apps/web (cap add ios/android; commit native projects), release.yml MOBILE_RELEASE lanes,
  RELEASE.md updates.
- Spec: generate and commit ios/ + android/ from the existing capacitor.config.ts (webDir already set);
  app ids world.cyberskill.cyberos; icons/splash from brand assets; CI lanes produce signed .aab and
  .ipa on tags once secrets exist; local build documented for both.
- Accept: debug builds run on simulator/emulator with login + chat round trip; CI dry-run passes to the
  signing step (full signing once inputs land).
- Review: confirm bundle ids + store listing names before first upload (renames are painful).

## T-034 Mobile: sqlite adapter + push + deep links

- C-refs C103, C104 | P0/M | depends: T-033, T-015, T-023
- Spec: chat-core adapter on @capacitor-community/sqlite (never the wasm path inside the native webview -
  assert at runtime); @capacitor/push-notifications registers FCM/APNs tokens into /v1/chat/devices with
  platform + app version; tap-through routes to channel/message; badge count sync; foreground pushes
  suppressed when the channel is open.
- Accept: adapter passes the conformance tests; staging push received on a real Android device and iOS
  (sandbox APNs) with app killed; tap opens the right channel.
- Review: notification sound/vibration defaults per platform.

## T-035 Mobile UX pass + store readiness

- C-refs C105, C106, C108 | P1/M | depends: T-034
- Spec: safe-area insets, keyboard-aware composer (no jumpy viewport), pull-to-refresh triggers sync,
  haptics on long-press action sheet, 44 px touch-target audit; store packet: privacy policy pages
  (PDPL-aligned, from T-041 language), Apple privacy nutrition labels + Play data-safety forms, account
  deletion path (links the T-040 DSAR/delete flow), screenshots; TestFlight + internal track rollout to
  the team. C108: one-page note re-evaluating Tauri-mobile consolidation with what we learned (input to
  D5's standing answer).
- Accept: team installs via TestFlight/internal track; store pre-review checklists pass; the C108 note
  filed in docs/improvement/chat/notes/.
- Review: you approve the store listings + privacy claims before any submission.

## T-036 Attachments to object storage

- C-refs C30 | P1/M | depends: decision D3 (store choice)
- Touch: services/chat/src/storage.rs (new backend behind the trait), attachments.rs (presigned upload +
  download), migration for storage_key/backend column, mover tool from T-004 pattern.
- Spec: S3-compatible backend (whichever D3 picks) with presigned PUT upload (client -> store directly,
  then confirm to chat) and presigned GET or streamed download; fs stays the dev default; size caps and
  content-type checks preserved server-side at confirm time; bucket lifecycle rules noted for T-045
  backups.
- Accept: upload/download round trip byte-identical; presigned URLs expire (test); mover migrates fs ->
  object store idempotently on staging; multi-replica note removed from the storage section of the
  compose comment.
- Review: cost + residency sanity per D3.

## T-037 Upload pipeline: resumable, EXIF/thumbs, AV, GC

- C-refs C31, C32, C33, C34 | P1/M | depends: T-036
- Spec: resumable uploads (S3 multipart with part size tuned for 3G/4G; tus only if multipart proves
  awkward through the proxy); server-side image normalization on confirm: strip EXIF/GPS, cap dimensions,
  generate thumbnail + blurhash; ClamAV container in compose, async scan, attachment states
  pending_scan|clean|quarantined (download blocked unless clean; UI shows scanning); orphan GC sweeper
  (unreferenced N days -> delete) + orphaned-bytes metric.
- Accept: kill an upload mid-way and resume; a GPS-tagged JPEG comes out clean (exiftool check in test);
  EICAR file quarantines; GC deletes a planted orphan and nothing else.
- Review: quarantine notification wording (VN + EN).

## T-038 Fan-out seam + shared presence + push offline fix

- C-refs C23, C24, C41 | P1/M | depends: none
- Touch: services/chat/src/realtime.rs + notify.rs (publish through a Fanout trait), new backend
  postgres-notify; presence to a table with TTL heartbeat; push relay offline check reads shared
  presence.
- Spec: trait with two impls: in-process (default today) and Postgres LISTEN/NOTIFY (payloads are
  {tenant, seq} pointers, receivers fetch from chat_events - stays under the 8 KB NOTIFY limit by
  design); shared presence rows (subject, device, last_seen) with sweep; compose comment about single
  replica updated to "2 replicas supported behind flag"; actual 2-replica smoke on staging.
- Accept: two chat containers on staging: message from a socket on A arrives to a socket on B; presence
  correct across both; push fires for a subject offline on both replicas; flag rollback path proven.
- Review: keep single replica in prod until T-045's zero-downtime drill passes; this task only unlocks it.

## T-039 Calls: TURN + ring + log + telemetry

- C-refs C125, C126, C127 | P0/M | depends: T-018, T-023
- Touch: deploy/vps (coturn service + secrets + Caddy/udp notes), server ICE-config endpoint, apps/web
  lib/call.ts + CallOverlay, timeline call rows, metrics.
- Spec: coturn with static-auth-secret + TLS; GET /v1/chat/ice returns time-limited TURN credentials;
  ring travels the account socket to all devices + push wake when offline (payload: call invite,
  privacy-preserving); call rows in timeline (started/ended/missed + duration); missed-call chip;
  periodic getStats samples (rtt, packet loss, codec) to metrics.
- Accept: call connects between two networks that fail today (mobile 4G <-> office NAT) on staging TURN;
  ring reaches a device with the app closed (push) and a device on another channel (socket); call rows
  render; stats visible in Grafana.
- Review: try one real call VN mobile <-> home fiber.

## T-040 PDPL ops: DSAR, retention, legal hold

- C-refs C63, C65, C67 | P1/M | depends: T-005
- Touch: services/chat export module + admin route, retention jobs, hold flags (subject/channel), audit
  chain emits for all three; console admin hooks minimal (full panel is T-065).
- Spec: DSAR export per subject: authored messages + revisions + attachments + audit slice -> signed
  archive (tar + manifest hash recorded to audit chain); deletion/anonymization workflow: tombstone
  content, keep chain integrity (hash of removed content retained); retention: per-data-class windows
  (messages, attachments, chat_events per T-013, push intents, presence traces) enforced by jobs with
  dry-run mode; legal hold flag suspends retention/deletion for its scope, audited.
- Accept: export of a seeded subject verifies against fixtures (complete + nothing extra); delete leaves
  chain verifiable; retention dry-run report matches expectation then live run deletes exactly that;
  hold blocks both paths.
- Review: sign off the retention window defaults before the first live run.

## T-041 PDPL governance pack (human-led, agent-drafted)

- C-refs C62, C64, C66 | P1/M | depends: none; blocked:input (counsel review)
- Spec: agent drafts, Stephen + counsel finalize: (1) processing dossier/DPIA-style mapping of chat data
  categories, purposes, bases, storage, processors (Supabase, Vultr, Google login, push providers); (2)
  cross-border transfer assessment recording the Supabase region and the D6 options; (3) breach playbook
  with roles, clocks, notification templates (VN + EN) + one dry-run. Filed under docs/compliance/chat/.
- Accept: three documents exist and are marked draft-for-counsel; dry-run of the breach playbook logged.
- Review: this one is yours + counsel's; agent output is a starting point, not the deliverable.

## T-042 Web: store-first cleanups + refactor + virtualization

- C-refs C11, C84, C85, C86 | P1/M | depends: T-017
- Spec: unread/mention badges derive from the event log in chat-core (delete the 15 s poll); split
  Chat.tsx (~1,240 lines) into feature modules (timeline, composer, sidebar, panels) rendering from
  chat-core; virtualized message list with stable scroll anchoring + day separators preserved; remove
  remaining interval timers (version check moves to visibilitychange-only).
- Accept: zero polling requests in the network tab during idle; scroll through 5k cached messages at
  60 fps-ish on a mid-range profile; all existing e2e + property suites green after the refactor.
- Review: feel pass - scroll, jump to unread, open threads; report anything that got heavier.

## T-043 Web: PWA completion, a11y, perf budget, QoL

- C-refs C87, C88, C89, C91 | P1/M | depends: T-017
- Spec: offline start from cache with offline banner + queued composer; install prompt flow + iOS
  add-to-home-screen hint; axe-core in CI with a zero-serious-violations gate; performance budget in CI
  (Lighthouse: LCP < 2 s on throttled profile, initial JS cap, code-split AI/thread/call panels); QoL:
  paste-image upload, whole-timeline drag-drop, optional send sound (off by default), favicon/title
  unread badge, lang="vi" spellcheck on composer when VN.
- Accept: airplane-mode reload shows usable app; axe + Lighthouse gates green; QoL items each demoed in
  the ledger entry.
- Review: 10-minute daily-driver pass on your Mac + phone browser.

## T-044 Sync v1.1: windows, cold start, offline search, fallback

- C-refs C8, C9, C12, C21 | P1/M | depends: T-013, T-017
- Spec: lists in sync (windowed channel collections: all/dms/unread-only with range + count) so big
  tenants never full-sync; cold-start returns top-N by activity with timeline_limit then background
  widening (MSC4186 range-growing pattern); SQLite FTS5 over cached messages for offline search (results
  labeled cached; online VN search unchanged); long-poll fallback transport using sync timeout_ms when
  ws fails repeatedly (auto-detect, indicator shows degraded).
- Accept: cold start on a 200-channel fixture usable in one round trip; offline search returns cached
  hits; forced ws-block still delivers messages via long-poll in Playwright.
- Review: none special.

## T-045 Ops drills: zero-downtime, backups, runbooks, cadence

- C-refs C143, C144, C145, C146, C37 | P1/M | depends: T-038
- Spec: two-replica rolling deploy with drain-on-SIGTERM (finish in-flight sends, close sockets politely)
  proven on staging then adopted in deploy.yml; nightly logical dump + attachment store sync off-site +
  QUARTERLY restore drill reconstructing a tenant into staging (timed, documented); runbooks for the five
  incidents (ws storm, DB failover, push outage, storage full, bad deploy) + incident comms template
  (post in chat + email mirror); dependency update cadence: weekly automated PRs through the caf gate,
  SBOM per release (pairs with T-057).
- Accept: mid-deploy message loss test = zero across 3 runs; restore drill report #1 committed; runbooks
  reviewed by doing one tabletop each.
- Review: you time the restore drill - that number goes in the go-live checklist.

## T-046 Load, fuzz, migration gate, chaos, soak

- C-refs C75, C78, C80, C81, C82 | P1/L | depends: T-009
- Spec: k6 (or artillery) nightly: 10k idle sockets + 200 msg/s burst single tenant, regression gate on
  p95 send-to-echo + RSS ceiling; fuzz ws/sync inputs (cargo-fuzz or a proptest rig: arbitrary frames,
  truncated bodies, absurd pos) - no panics, typed errors only; CI migration gate required (0001..latest
  on throwaway + previous-release-against-new-schema check); chaos drill: kill chat container under
  traffic, ledger-compare client stores vs chat_events (zero loss/dup); 24 h soak with flat heap + map
  metrics.
- Accept: all five suites runnable by one command each + wired to schedule; baseline numbers recorded in
  the ledger for future regressions.
- Review: read the first nightly report; set the regression thresholds with real numbers.

## T-047 Observability completion: OTel, capacity, SLOs

- C-refs C70, C71, C72 | P1/M | depends: T-008
- Spec: OTLP traces Caddy -> chat -> Postgres/ai-gateway (100% errors, ~1% ok) into a compose-local
  Tempo/Jaeger; capacity dashboard (rows + bytes per tenant, growth rate, largest channels, ws counts) -
  the trigger source for T-056 partitioning/archive decisions; SLO doc with the section-2 targets, burn-
  rate alerts, error budget + weekly review ritual; release-pause rule wired to T-032 canary habit.
- Accept: a traced send shows spans across services; SLO dashboard live with budget remaining; one week
  of data reviewed in the first ritual.
- Review: attend the first weekly review; keep or kill the ritual consciously.
