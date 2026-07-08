# CHAT improvement backlog - master table

Generated 2026-07-06 from docs/strategy/chat-enterprise-grade-plan-2026-07-06.md (C1-C147).
Status changes happen in THIS file only (vocabulary in README.md). Detailed specs: tasks/phase-N.md.
Depends lists task ids that must be done first; the implementation prompt enforces top-to-bottom order
within a phase on top of that. blocked:input rows need something only Stephen can provide.

## Phase 0 - safety rails (target: week 1)

| Task | Pri | Eff | Title | C-refs | Depends | Status |
|---|---|---|---|---|---|---|
| T-001 | P0 | M | Rate limiting (subject + IP layers) | C46 | - | ready |
| T-002 | P0 | S | JWT hardening: aud always, iss pin, JWKS refetch on unknown kid | C47, C48 | - | ready |
| T-003 | P0 | S | Account-wide socket kill on revoke (<60 s) | C49 | - | ready |
| T-004 | P0 | S | BYTEA attachment closeout (fs-only prod) | C28 | - | ready |
| T-005 | P0 | S | Edit-history revisions table | C29 | - | ready |
| T-006 | P0 | S | WS heartbeat + idle reaper | C16 | - | ready |
| T-007 | P0 | S | External synthetic probe (login-send-receive) | C74 | - | ready |
| T-008 | P0 | M | Metrics baseline + core alerts (Prom/Grafana/AM) | C69 | - | ready |
| T-009 | P0 | S | Staging compose profile + seeded tenant | C139 | - | ready |
| T-010 | P0 | S | Go-live checklist doc (living) | C147 | - | ready |

## Phase 1 - sync core (target: weeks 2-4)

| Task | Pri | Eff | Title | C-refs | Depends | Status |
|---|---|---|---|---|---|---|
| T-011 | P0 | M | chat_events log + per-tenant seq (transactional) | C1 | - | ready |
| T-012 | P0 | S | Seq-stamped ws frames + client gap detection | C3 | T-011 | ready |
| T-013 | P0 | M | POST /v1/chat/sync v1 (pos, subscriptions, initial/limited, reset) + log retention | C2, C10 | T-011 | ready |
| T-014 | P0 | M | Idempotent send: client_msg_id + unique index + echo reconcile | C4 | - | ready |
| T-015 | P0 | M | chat-core package skeleton + web SQLite/OPFS adapter (IndexedDB fallback) | C6 | - | ready |
| T-016 | P0 | M | Persistent outbox (FIFO, backoff, restart-safe, visible states) + connection-state contract | C5, C22 | T-014, T-015 | ready |
| T-017 | P0 | L | Store-driven rendering: messages/channels/read state + drafts from chat-core | C6, C7 | T-013, T-015 | ready |
| T-018 | P0 | M | Single account-scoped multiplexed socket (subscribe frames) | C15 | T-012 | ready |
| T-019 | P1 | M | Socket hardening: resume hint, backpressure, envelope v, origin/frame caps | C17, C18, C19, C20 | T-018 | ready |
| T-020 | P0 | M | Convergence + outbox property suites | C76, C77 | T-016, T-017 | ready |
| T-021 | P0 | M | Offline end-to-end Playwright suite | C79 | T-017 | ready |
| T-022 | P1 | S | chat-core as CI citizen (typecheck/unit/property gates) | C83 | T-015 | ready |

## Phase 2 - push + desktop GA (target: weeks 5-7)

| Task | Pri | Eff | Title | C-refs | Depends | Status |
|---|---|---|---|---|---|---|
| T-023 | P0 | L | Push relay worker (FCM v1 + APNs) + collapse keys + badges | C38, C40 | T-008 | blocked:input (FCM svc acct + APNs key) |
| T-024 | P1 | S | Web push: VAPID + declarative payload | C39, C90 | T-023 | ready |
| T-025 | P1 | S | Quiet hours / DND schedule server-side | C42 | - | ready |
| T-026 | P0 | M | Desktop: bundle SPA into Tauri + CSP (web + shell) | C92, C99, C50 | - | ready |
| T-027 | P0 | M | Desktop: tauri-plugin-sql adapter + native notifications | C93, C94 | T-026, T-015 | ready |
| T-028 | P1 | M | Desktop: tray/badge, deep links + single instance, window-state/shortcut/autostart | C95, C96, C98 | T-026 | ready |
| T-029 | P1 | M | Desktop: updater keys/channels + crash reporting + CI matrix | C97, C100, C101 | T-026 | blocked:input (signing keys, RELEASE.md) |
| T-030 | P1 | S | Version-skew policy: min_supported_client + blocking banner | C142 | - | ready |
| T-031 | P1 | S | Error tracking client + server (GlitchTip/Sentry) | C73 | - | ready |
| T-032 | P1 | S | Per-tenant feature flags + canary rollout habit | C140, C141 | - | ready |

## Phase 3 - mobile, storage, compliance, scale seam (target: weeks 8-10)

| Task | Pri | Eff | Title | C-refs | Depends | Status |
|---|---|---|---|---|---|---|
| T-033 | P0 | M | Capacitor init (ios/ + android/ committed) + release lanes live | C102 | - | blocked:input (store accounts + signing) |
| T-034 | P0 | M | Mobile: sqlite adapter + push plugin + deep-link tap-through | C103, C104 | T-033, T-015, T-023 | ready |
| T-035 | P1 | M | Mobile UX pass + store readiness (privacy forms, deletion path) | C105, C106, C108 | T-034 | ready |
| T-036 | P1 | M | Attachments to object storage (presigned) | C30 | decision D3 | blocked:D3 |
| T-037 | P1 | M | Upload pipeline: resumable, EXIF/thumbs, AV scan, orphan GC | C31, C32, C33, C34 | T-036 | ready |
| T-038 | P1 | M | Fan-out seam trait + LISTEN/NOTIFY backend + shared presence + push offline fix | C23, C24, C41 | - | ready |
| T-039 | P0 | M | Calls: coturn TURN + ring via account socket + push wake + call log + quality telemetry | C125, C126, C127 | T-018, T-023 | ready |
| T-040 | P1 | M | PDPL ops: DSAR export, retention jobs, legal hold | C63, C65, C67 | T-005 | ready |
| T-041 | P1 | M | PDPL governance: data mapping/DPIA, transfer assessment, breach playbook | C62, C64, C66 | - | blocked:input (counsel) |
| T-042 | P1 | M | Web: unread from event log (kill polls) + Chat.tsx refactor + virtualized list | C11, C84, C85, C86 | T-017 | ready |
| T-043 | P1 | M | Web: PWA completion, a11y AA + axe CI, perf budget, QoL batch | C87, C88, C89, C91 | T-017 | ready |
| T-044 | P1 | M | Sync v1.1: cold-start snapshot, lists/windows, offline local search, long-poll fallback | C8, C9, C12, C21 | T-013, T-017 | ready |
| T-045 | P1 | M | Ops drills: zero-downtime deploy, off-site backup + restore drill, runbooks, dep cadence | C143, C144, C145, C146, C37 | T-038 | ready |
| T-046 | P1 | L | Load/fuzz/migration-gate/chaos/soak suites | C75, C78, C80, C81, C82 | T-009 | ready |
| T-047 | P1 | M | Observability completion: OTel traces, capacity dashboard, SLOs + error budget | C70, C71, C72 | T-008 | ready |

## Phase 4 - product depth, ecosystem, scale (target: the following quarter)

| Task | Pri | Eff | Title | C-refs | Depends | Status |
|---|---|---|---|---|---|---|
| T-048 | P1 | M | Product wave 1: pins, saved-for-later, forward/quote, notif-override UI, edit-history view | C109, C110, C111, C120, C118 | T-017 | ready |
| T-049 | P1 | M | Group DMs + scheduled send + reminders | C112, C113 | T-017 | ready |
| T-050 | P1 | M | Link previews (SSRF-safe fetcher + cards) | C52, C114 | T-001 | ready |
| T-051 | P2 | M | Product wave 2: slash commands, custom emoji, status/DND states, jump-to-date/sections/mark-unread, templates | C115, C116, C117, C119, C124 | T-048 | ready |
| T-052 | P1 | L | AI set: catch-me-up/summaries/action items (streaming), semantic search, @lumi assistant | C121, C122, C123 | T-017 | ready |
| T-053 | P1 | M | API contracts: error envelope, OpenAPI, generated TS client, ws frame catalog | C130, C131, C132, C133 | - | ready |
| T-054 | P2 | M | Ecosystem: incoming/outgoing webhooks, bot accounts, MCP tool surface | C134, C135, C136, C137 | T-053 | ready |
| T-055 | P2 | L | Imports: Slack then Zalo (idempotent, checkpointed) | C138 | - | ready |
| T-056 | P2 | L | Scale wave: Redis/NATS backend, load-shed order, partitioning, quotas, cold storage | C25, C26, C27, C35, C36 | T-038, T-047 | ready |
| T-057 | P1 | M | Security completion: upload hardening, device/session UI, secrets scan, supply chain, retention policy object | C51, C53, C54, C55, C56 | - | ready |
| T-058 | P1 | M | External penetration test (scope: chat + auth + uploads + ws) | C57 | T-057 | blocked:input (vendor) |
| T-059 | P2 | S | Encryption posture: honest security page + attachment encryption at rest | C58, C59 | - | ready |
| T-060 | P3 | L | E2EE channel class on MLS (capture/AI/search off, labeled) | C60, C61 | decision D1 | blocked:D1 |
| T-061 | P2 | S | Attachment offline cache policy + sync conformance fixture | C13, C14 | T-017, T-020 | ready |
| T-062 | P2 | M | Calls extended: 1:1 screen share; SFU spike for group calls | C128, C129 | T-039, decision D4 | blocked:D4 |
| T-063 | P2 | S | Notification extras: offline email digest, delivery-ticks decision, Zalo/SMS bridge spike | C43, C44, C45 | T-023 | ready |
| T-064 | P2 | M | Share-to-chat intent (mobile share sheet) | C107 | T-035 | ready |
| T-065 | P2 | M | Admin data-governance panel (retention, holds, DSAR, consent, capture) | C68 | T-040 | ready |
| T-066 | P1 | S | Decision memos D1-D6 prepared for Stephen (one page each) | - | - | ready |

## Decisions (from report section 7; resolve before their gated tasks)

| Id | Decision | Gates | Status |
|---|---|---|---|
| D1 | Recording-with-consent vs E2EE channel classes | T-060 | open |
| D2 | Build thin sync natively vs adopt PowerSync/Electric | (report recommends build; revisit trigger noted) | resolved:build |
| D3 | Attachment object store: Supabase Storage vs MinIO-on-VPS vs R2 | T-036 | open |
| D4 | Group-call SFU (LiveKit self-host) vs defer group calls | T-062 | open |
| D5 | Mobile track: Capacitor now, Tauri-mobile later | (report recommends Capacitor; T-035 carries the C108 re-check) | resolved:capacitor |
| D6 | Data residency: Supabase region vs VN-hosted Postgres | T-041 output informs | open |

## Coverage map (every report item -> task)

C1:T-011 C2:T-013 C3:T-012 C4:T-014 C5:T-016 C6:T-015,T-017 C7:T-017 C8:T-044 C9:T-044 C10:T-013
C11:T-042 C12:T-044 C13:T-061 C14:T-061 C15:T-018 C16:T-006 C17:T-019 C18:T-019 C19:T-019 C20:T-019
C21:T-044 C22:T-016 C23:T-038 C24:T-038 C25:T-056 C26:T-056 C27:T-056 C28:T-004 C29:T-005 C30:T-036
C31:T-037 C32:T-037 C33:T-037 C34:T-037 C35:T-056 C36:T-056 C37:T-045 C38:T-023 C39:T-024 C40:T-023
C41:T-038 C42:T-025 C43:T-063 C44:T-063 C45:T-063 C46:T-001 C47:T-002 C48:T-002 C49:T-003 C50:T-026
C51:T-057 C52:T-050 C53:T-057 C54:T-057 C55:T-057 C56:T-057 C57:T-058 C58:T-059 C59:T-059 C60:T-060
C61:T-060 C62:T-041 C63:T-040 C64:T-041 C65:T-040 C66:T-041 C67:T-040 C68:T-065 C69:T-008 C70:T-047
C71:T-047 C72:T-047 C73:T-031 C74:T-007 C75:T-046 C76:T-020 C77:T-020 C78:T-046 C79:T-021 C80:T-046
C81:T-046 C82:T-046 C83:T-022 C84:T-042 C85:T-042 C86:T-042 C87:T-043 C88:T-043 C89:T-043 C90:T-024
C91:T-043 C92:T-026 C93:T-027 C94:T-027 C95:T-028 C96:T-028 C97:T-029 C98:T-028 C99:T-026 C100:T-029
C101:T-029 C102:T-033 C103:T-034 C104:T-034 C105:T-035 C106:T-035 C107:T-064 C108:T-035 C109:T-048 C110:T-048
C111:T-048 C112:T-049 C113:T-049 C114:T-050 C115:T-051 C116:T-051 C117:T-051 C118:T-048 C119:T-051 C120:T-048
C121:T-052 C122:T-052 C123:T-052 C124:T-051 C125:T-039 C126:T-039 C127:T-039 C128:T-062 C129:T-062 C130:T-053
C131:T-053 C132:T-053 C133:T-053 C134:T-054 C135:T-054 C136:T-054 C137:T-054 C138:T-055 C139:T-009 C140:T-032
C141:T-032 C142:T-030 C143:T-045 C144:T-045 C145:T-045 C146:T-045 C147:T-010
