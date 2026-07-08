# Phase 4 - product depth, ecosystem, scale (T-048..T-066)

Order inside this phase follows daily-pain: report section 6 phase 4 names the starting set. Every
feature here ships behind a T-032 tenant flag and rides the canary habit. Nothing here starts before the
phase-1 exit bar is met.

## T-048 Product wave 1

- C-refs C109, C110, C111, C120, C118 | P1/M | depends: T-017
- Spec: pins per channel (pin/unpin permissioned like edits, pins list in header, pin_set event already
  in the T-011 taxonomy); saved-for-later private bookmarks with sidebar view; forward-to-channel +
  quote-reply with backlink chip; per-channel notification override UI (all/mentions/nothing exists in
  prefs - surface per channel + per device); edit-history viewer on the "edited" affordance reading
  T-005 revisions (permission: channel members).
- Accept: each feature has ws/sync events flowing through chat-core (offline-correct), keyboard path,
  VN+EN strings, and a smoke; property suite untouched-green.
- Review: 15-minute product pass; kill anything that feels bolted-on before it ships tenant-wide.

## T-049 Group DMs + scheduled send + reminders

- C-refs C112, C113 | P1/M | depends: T-017
- Spec: ad-hoc group DMs (3-9 people, unnamed, kind='dm_group', name derived from members) using the
  create_dm advisory-lock pattern on the sorted member-set key; scheduled send = outbox rows with
  not_before surfaced in UI (badge on channel, editable before release) + server-side release job for
  cross-device correctness; remind-me-about-this creates a scheduled self-DM with a backlink.
- Accept: concurrent group-DM create for the same set yields one channel (test); scheduled message
  survives client shutdown and sends on time server-side; reminder round trip.
- Review: pick the group-DM display-name rule (first names vs counts).

## T-050 Link previews

- C-refs C52, C114 | P1/M | depends: T-001
- Spec: server-side unfurl worker: private-range IP deny (post-DNS re-check), redirect cap 3, content
  types html/og only, 2 MB fetch cap, per-domain rate limit, no cookies, cache per URL (tenant-shared)
  with TTL; renders compact card (title, description, image via image proxy with same SSRF rules);
  per-channel toggle + per-message opt-out (trailing flag); unfurls arrive as attachment-style events so
  offline clients converge.
- Accept: SSRF suite (169.254.x, 10.x, redirect-to-internal, DNS rebind case) all blocked in tests; a
  normal article and a YouTube link render; toggle respected.
- Review: none - but this task must not start before T-001 limits exist.

## T-051 Product wave 2

- C-refs C115, C116, C117, C119, C124 | P2/M | depends: T-048
- Spec: slash commands parsed against a server-declared registry (/mute /call /translate /remind /lumi;
  bots extend via T-054); custom emoji per tenant (upload -> T-037 pipeline; picker section + skin
  tones); user status (emoji + text + auto-clear presets: focus/away/off-sick/on-leave) shown in
  directory/DM headers, DND state maps to T-025; jump-to-date, sidebar sections (favorites/custom),
  mark-as-unread from any message; message templates/snippets (per-user + per-tenant shared) in the
  composer.
- Accept: registry-driven command list renders from server data; every item offline-correct through
  chat-core; a11y + i18n pass per feature.
- Review: sequence within the wave is yours to reorder by team demand.

## T-052 AI set (streaming)

- C-refs C121, C122, C123 | P1/L | depends: T-017 (streaming needs ai.rs upgrade)
- Spec: upgrade ai.rs passthrough to support SSE streaming from ai-gateway; catch-me-up per channel
  (summarize since my read marker), thread summary, action-item extraction posted as a checklist message
  (bot-authored, T-054 account); semantic search: embed messages via embed service into pgvector,
  hybrid rank with VN FTS, tenant flag + capture-consent aware (only consented content embeds); @lumi
  assistant (FR-CHAT-106): mention triggers CUO routing with channel context, streams reply in-thread,
  consent respected, rate-limited per T-001 class.
- Accept: streamed tokens render incrementally; catch-me-up on a 200-message fixture is accurate spot-
  checked; embeddings respect consent flags (test with a non-consented channel); lumi round trip on
  staging with the local model.
- Review: sign off the lumi persona/system prompt; watch one real catch-me-up before tenant-wide.

## T-053 API contracts

- C-refs C130, C131, C132, C133 | P1/M | depends: none
- Spec: unified error envelope {code, message, retryable, details} adopted across chat routes (R1 shape;
  map internal errors to stable codes); OpenAPI spec for all chat routes (utoipa preferred; CI check
  that router and spec agree on paths); generate the TS client consumed by chat-core (kills hand-written
  fetch drift); ws + sync frame catalog doc (AsyncAPI-style) with version notes - the C19/T-019 contract
  formalized.
- Accept: spec-vs-router CI check green; chat-core compiles against the generated client only; error
  codes asserted in existing tests (no prose matching anywhere).
- Review: skim the error-code list once; these become support vocabulary.

## T-054 Ecosystem: webhooks + bots + MCP

- C-refs C134, C135, C136, C137 | P2/M | depends: T-053
- Spec: incoming webhooks per channel (token URL, T-001 rate class, rendered as app messages with
  distinct styling); outgoing subscriptions (HMAC-signed, backoff retries, dead-letter after N, replay
  tool); bot/service accounts via AUTH service tokens with bot badge + slash-command registration; MCP
  tools on the existing gateway: chat.post_message, chat.search, chat.summarize_channel,
  chat.create_channel under tenant policy + audit emits.
- Accept: CI notification into a channel via incoming webhook; outgoing delivery survives a dead
  receiver (retry -> dead-letter -> replay); an MCP client posts a message on staging with audit row.
- Review: approve the MCP tool scopes before exposure beyond staging.

## T-055 Imports: Slack then Zalo

- C-refs C138 | P2/L | depends: none
- Spec: revive FR-CHAT-102 (Slack export importer: users/channels/messages/threads/files/reactions with
  idempotent 8-step checkpoints, dry-run report first) mapped to the current schema incl. chat_events
  backfill policy (import events enter the log with a synthetic epoch so sync clients do not storm);
  then FR-CHAT-103 Zalo bundle with VN-Unicode normalization. Real company export as the acceptance
  fixture.
- Accept: dry-run report matches expectation; re-run after interrupt resumes at checkpoint; imported
  history searchable + thread-intact; decision-signal metric (FR-CHAT-010 ratio) noted.
- Review: you provide the Slack export + call the cutover date.

## T-056 Scale wave (signal-gated)

- C-refs C25, C26, C27, C35, C36 | P2/L | depends: T-038, T-047 signals
- Spec: only what T-047's capacity dashboard justifies, in order: Redis (or NATS) Fanout backend behind
  the trait when LISTEN/NOTIFY saturates; documented load-shed order implemented (typing -> presence ->
  receipts, never messages); per-tenant quotas (storage bytes, attachment size/count, channels) with
  admin overrides; partition chat_messages + chat_events (tenant hash or month) near 10^8 rows; cold
  archive tier (older than N months out of sync reach, lazy search).
- Accept: each sub-item has a before/after benchmark in the ledger; rollback path per sub-item.
- Review: approve each sub-item start from the dashboard evidence, not calendar.

## T-057 Security completion

- C-refs C51, C53, C54, C55, C56 | P1/M | depends: none
- Spec: upload hardening beyond T-037 (magic-byte sniff must match declared type, SVG download-only,
  html/js types forbidden, archive decompressed-size probe); session/device management UI (list devices
  + live sockets from devices/presence, revoke one, sign-out-everywhere - pairs with AUTH sessions);
  gitleaks in CI repo-wide; cargo audit/deny + npm audit gating chat paths + SBOM per release; the
  retention/consent/capture policy object unifying C56 (single JSON per tenant read by retention jobs +
  capture + admin panel).
- Accept: each control has a red test (bad file rejected, revoked device socket dies, planted secret
  caught in CI, vulnerable dep blocks); policy object round-trips through admin API.
- Review: none special.

## T-058 External penetration test (human-led)

- C-refs C57 | P1/M | depends: T-057; blocked:input (vendor selection + window)
- Spec: scope: chat + auth + uploads + ws + link-preview fetcher + RLS bypass attempts; agent prepares
  the scoping doc, test accounts, staging environment hardening parity note; findings triaged into this
  backlog as T-058-Fn rows.
- Accept: report received, criticals fixed + retested before company-wide go-live.
- Review: vendor pick + budget; VN or regional firm fine.

## T-059 Encryption posture (honest now)

- C-refs C58, C59 | P2/S | depends: none
- Spec: product security page (VN + EN) stating the real model: TLS in transit, encrypted at rest,
  server-readable by design for capture/AI/search/compliance, consent-gated capture, E2EE class on the
  roadmap pending D1; envelope encryption for attachment bytes at rest (per-tenant data keys wrapped by
  a master key in env/KMS-lite) - server can still read (keys server-side), protects the storage layer.
- Accept: page published; attachments unreadable via raw store access without keys (test); key rotation
  procedure documented + drilled once.
- Review: wording sign-off - this page is brand-sensitive for a security company.

## T-060 E2EE channel class on MLS (gated on D1)

- C-refs C60, C61 | P3/L | depends: decision D1
- Spec: only after D1: designated E2EE DM/channel types on MLS (RFC 9420) via a maintained library
  (openmls) - never hand-rolled; capture/AI/server-search structurally disabled and labeled in-UI;
  device verification, key backup/recovery UX ships in the same release or the class does not ship;
  sync layer carries opaque ciphertext events (chat-core stores ciphertext + local plaintext cache).
- Accept: separate design doc first (this task starts as a spike + doc, not code); the report's C61
  warning stands: half-shipped E2EE is worse than none.
- Review: D1 is the gate; also confirm PDPL/BRAIN implications with counsel.

## T-061 Attachment offline policy + sync fixture

- C-refs C13, C14 | P2/S | depends: T-017, T-020
- Spec: thumbnails always cached; full files on tap with per-device size budget + LRU eviction; queued
  attachment uploads ride the outbox (multipart resume via T-037); the recorded golden event-log
  scenario fixture replayed in CI asserting byte-identical final store state (the convergence contract
  as a fixed regression fixture, complementing T-020's randomized suite).
- Accept: eviction respects budget (test); golden fixture green and REQUIRED in CI.
- Review: pick the default cache budget per platform (suggest 512 MB desktop, 256 MB mobile).

## T-062 Calls extended (gated on D4)

- C-refs C128, C129 | P2/M | depends: T-039, decision D4
- Spec: 1:1 screen share via getDisplayMedia (TURN exists after T-039); group calls/huddles spike on
  self-hosted LiveKit (or defer per D4) - spike output is a one-page recommendation with resource cost,
  not production code; no mesh group WebRTC ever.
- Accept: screen share works cross-network on staging; spike doc filed.
- Review: D4 call after reading the spike doc.

## T-063 Notification extras

- C-refs C43, C44, C45 | P2/S | depends: T-023
- Spec: offline email digest (offline > N hours -> summary via existing mail path, per-user opt-out);
  delivery-ticks decision memo (per-device delivered state is cheap post-event-log - product call,
  default OFF for calm; memo to Stephen, only build if yes); Zalo/SMS bridge spike for critical mentions
  (VN differentiator, opt-in) - spike doc only.
- Accept: digest received in staging; two memos filed.
- Review: the two product calls are yours.

## T-064 Share-to-chat intent

- C-refs C107 | P2/M | depends: T-035
- Spec: mobile share sheet -> channel picker -> composer prefilled (files ride T-037 uploads); Android
  SEND/SEND_MULTIPLE intents + iOS share extension.
- Accept: share a photo from Photos to a channel on both platforms in staging builds.
- Review: none.

## T-065 Admin data-governance panel

- C-refs C68 | P2/M | depends: T-040
- Spec: console app-module screen over the T-057 policy object + T-040 operations: retention windows,
  legal holds, DSAR trigger + download, consent states, capture policy per tenant; every action audited;
  role-gated (owner/admin).
- Accept: each control round-trips and emits an audit row; screen usable on the existing console stack.
- Review: 10-minute walkthrough; this is the screen you show enterprise customers.

## T-066 Decision memos D1-D6

- C-refs none | P1/S | depends: none (do early; unblocks T-036/T-060/T-062)
- Spec: one page per decision (D1 recording vs E2EE classes; D2 build-vs-adopt standing answer + revisit
  trigger; D3 object store; D4 SFU; D5 mobile track standing answer; D6 residency) with options, costs,
  recommendation, and the tasks each unblocks; filed in docs/improvement/chat/decisions/ and linked from
  BACKLOG.md.
- Accept: six memos exist; BACKLOG decision rows link them.
- Review: you resolve D1/D3/D4/D6 on your schedule; D2/D5 just need a nod.
