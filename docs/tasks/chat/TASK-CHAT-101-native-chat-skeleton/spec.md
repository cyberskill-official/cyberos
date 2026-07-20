---
id: TASK-CHAT-101
title: "CyberOS-native chat - slice 1 (skeleton: channels, messages, live delivery, CyberOS-token auth)"
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
eu_ai_act_risk_class: not_ai
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-06-29T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: CHAT
priority: p0
status: done
verify: T
phase: P4
milestone: P4 - native chat slice 1
slice: 1
owner: Stephen Cheng (CTO)
created: 2026-06-29
supersedes: [TASK-CHAT-001, TASK-CHAT-002, TASK-CHAT-003, TASK-CHAT-004, TASK-CHAT-005, TASK-CHAT-006, TASK-CHAT-007, TASK-CHAT-008, TASK-CHAT-009, TASK-CHAT-010, TASK-CHAT-011, TASK-CHAT-012, TASK-CHAT-013]
related_tasks: [TASK-AUTH-110, TASK-AUTH-004, TASK-AUTH-005, TASK-AUTH-101]
depends_on: [TASK-AUTH-110, TASK-AUTH-004]
blocks: []

language: Rust (axum + sqlx + tokio)
service: cyberos/services/chat
new_files:
  # new workspace member, package cyberos-chat
  - services/chat/Cargo.toml
  # bin entrypoint (bind, load JWKS, run)
  - services/chat/src/main.rs
  # router + state
  - services/chat/src/lib.rs
  # CyberOS-token verifier (RS256 JWKS, HS256 for tests)
  - services/chat/src/auth.rs
  # create/list channels + membership
  - services/chat/src/channels.rs
  # post/list messages
  - services/chat/src/messages.rs
  # per-channel broadcast + websocket handler
  - services/chat/src/realtime.rs
  # pool + tenant-GUC helper
  - services/chat/src/db.rs
  # channels, messages, channel_members (+ RLS)
  - services/chat/migrations/0001_chat_core.sql
modified_files:
  # add "chat" to workspace members
  - services/Cargo.toml
  # retire TASK-CHAT-001..013, point at the native series
  - docs/tasks/chat/README.md

retires:
  - "The Mattermost-fork CHAT module (TASK-CHAT-001..012) and its OIDC config (TASK-CHAT-013). services/chat is repurposed from the Mattermost fork to the native cyberos-chat crate; the old Mattermost scaffolding is archived, not built."

effort_hours: 10
---

## §1 - Description (BCP-14 normative)

CHAT is rebuilt as a CyberOS-native service - a first-party Rust service on the existing identity, database, and audit chain - replacing the Mattermost fork entirely. Slice 1 is the walking skeleton that proves the spine end to end. Each requirement:

1. **MUST** be a new first-party Rust service `cyberos-chat` at `services/chat` (axum + sqlx + tokio), a new member of the `services/` Cargo workspace. No third-party chat server, no Mattermost.

2. **MUST** authenticate every request with the CyberOS access token the TASK-AUTH-110 provider issues, verified against the TASK-AUTH-004 JWKS (RS256) - the same verification `obs-compliance-view` and the MCP gateway already do. No separate chat login, no chat-local password store. An HS256 secret path is allowed for tests and local dev only.

3. **MUST** take the tenant and the caller identity from the verified token (`tenant_id`, `sub`), never from a request parameter. A cross-tenant request is impossible because every query runs under the tenant.

4. **MUST** persist to Postgres with per-tenant row-level security, using the established GUC idiom (`app.current_tenant_id`; the nil tenant bypasses for admin paths) - identical to the auth and sessions tables. Slice 1 uses a dedicated `cyberos_chat` database.

5. **MUST** expose channels: `POST /v1/chat/channels` (create; the creator becomes a member) and `GET /v1/chat/channels` (list the channels the caller is a member of, in their tenant).

6. **MUST** expose messages: `POST /v1/chat/channels/{id}/messages` (post; the caller must be a member) and `GET /v1/chat/channels/{id}/messages?before=&limit=` (most-recent-first, paged; members only).

7. **MUST** deliver messages live over a websocket: `GET /v1/chat/ws?channel={id}` (token in the `Authorization` header or an `access_token` query param). When any client posts to a channel, every websocket subscribed to that channel receives the message. Slice 1 uses an in-process per-channel broadcast; Redis pub/sub for multi-instance fan-out is a later slice.

8. **MUST** write an audit row to the memory chain for `chat.channel_created` and `chat.message_posted` (via `cyberos-audit-chain`), best-effort and non-blocking, the way the other services emit.

9. **MUST** serve `GET /healthz` (200 when the pool is reachable) so the deploy stack healthcheck passes.

10. **MUST** refuse an absent, malformed, or expired token at every HTTP endpoint and at the websocket handshake (401), and refuse a non-member posting to or reading a channel (403).

Slice-1 non-goals (named, not gaps): threads, direct messages, presence and typing, read receipts, file attachments, search, multi-tenant team mapping, federation, and the production client UI. A minimal test client (or a websocket CLI) is enough to prove slice 1; the web client is its own slice.

## §2 - Why this design (rationale for humans)

CyberOS is a from-scratch, owned platform. Chat was the one surface leaning on a third party (Mattermost); this makes it first-party too. It costs less than it looks because it stands on what already exists: identity from the TASK-AUTH-110 provider, Postgres with the tenant-RLS pattern, the audit chain, the obs SDK, and the same service shape as auth and memory. Slice 1 deliberately builds only the spine - authenticated channels and messages with live delivery - so there is a running, testable chat before any of the heavier features.

Real-time starts in-process (a `tokio::sync::broadcast` per channel) because a single instance is enough to prove the model and to run for the team early; the move to Redis pub/sub is a contained change behind the same `realtime` interface when more than one instance runs.

## §3 - Architecture

- Service `cyberos-chat` (`services/chat`): axum router, sqlx Postgres pool, tokio. Binds `CHAT_LISTEN_ADDR` (default `0.0.0.0:7720`). Reads `DATABASE_URL` and the provider issuer `CHAT_AUTH_ISSUER` (to fetch `/.well-known/jwks.json` at boot, cache the keys by `kid`).
- Auth (`src/auth.rs`): an `Authenticator` mirroring `obs-compliance-view::auth` - `from_jwks` (RS256) or `from_hs256_secret` (tests), `verify(token) -> Claims { sub, tenant_id, roles, exp }`. A small extractor turns the `Authorization: Bearer` header (or the `access_token` ws query param) into verified `Claims`.
- DB (`src/db.rs`): the pool plus a helper that, per request, opens a transaction and runs `SELECT set_config('app.current_tenant_id', $tenant, true)` before the query, so RLS scopes every read and write to the caller's tenant.
- Realtime (`src/realtime.rs`): a process-global map `channel_id -> broadcast::Sender<MessageEvent>`. The websocket handler verifies the token, checks membership, subscribes to the channel sender, and forwards each event to the socket. `messages::post` publishes to the sender after the row commits.

## §4 - Schema (`migrations/0001_chat_core.sql`)

```sql
CREATE TABLE chat_channels (
  id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id    UUID NOT NULL,
  name         TEXT NOT NULL,
  created_by   UUID NOT NULL,
  created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE chat_channel_members (
  channel_id   UUID NOT NULL REFERENCES chat_channels(id) ON DELETE CASCADE,
  tenant_id    UUID NOT NULL,
  subject_id   UUID NOT NULL,
  role         TEXT NOT NULL DEFAULT 'member',
  joined_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (channel_id, subject_id)
);

CREATE TABLE chat_messages (
  id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id          UUID NOT NULL,
  channel_id         UUID NOT NULL REFERENCES chat_channels(id) ON DELETE CASCADE,
  sender_subject_id  UUID NOT NULL,
  body               TEXT NOT NULL,
  created_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX chat_messages_channel_created_idx ON chat_messages (channel_id, created_at DESC);

-- RLS by tenant on all three (the 0021_sessions idiom: tenant match OR nil bypass).
-- ALTER TABLE ... ENABLE ROW LEVEL SECURITY + FORCE; policy USING/WITH CHECK
--   tenant_id::text = current_setting('app.current_tenant_id', true)
--   OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
```

## §5 - HTTP + websocket contract

```
POST /v1/chat/channels            {"name":"general"}            -> 201 {channel}
GET  /v1/chat/channels                                          -> 200 [{channel}]
POST /v1/chat/channels/{id}/messages  {"body":"hello"}          -> 201 {message}
GET  /v1/chat/channels/{id}/messages?before=<iso>&limit=50     -> 200 [{message}]  (newest first)
GET  /v1/chat/ws?channel={id}     (Bearer or ?access_token=)    -> websocket; server pushes {message} events
GET  /healthz                                                   -> 200
```

All HTTP routes require `Authorization: Bearer <CyberOS access token>`; tenant + sender come from the token.

## §6 - Acceptance criteria

1. A user with a valid CyberOS token creates a channel and lists it; the creator is a member.
2. Posting a message persists it and `GET messages` returns it; a non-member posting or reading is 403.
3. Two websocket clients subscribed to the same channel both receive a posted message live.
4. A token from tenant A sees none of tenant B's channels or messages (RLS proven, not just app logic).
5. An absent, malformed, or expired token is refused 401 at every endpoint and at the ws handshake.
6. `chat.message_posted` and `chat.channel_created` audit rows are written to the memory chain.
7. `cargo test -p cyberos-chat` is green; the service boots healthy in the deploy stack (`/healthz` 200).

## §7 - Slice roadmap (native chat)

1. **This task** - skeleton: auth, channels, messages, live delivery, one tenant.
2. Threads and replies; channel membership and roles; message history paging and edits/deletes.
3. Vietnamese search (the TASK-CHAT-004 goal, re-homed natively) and file attachments.
4. Presence, typing, read receipts, mobile push.
5. Voice and video; mobile clients. The heavy, later items.

The web client lands alongside slice 1-2 inside the existing console app (its own task under the APP module).

## §8 - Dependencies and what this retires

Upstream: TASK-AUTH-110 (the provider that issues the identity token), TASK-AUTH-004 (the JWKS chat verifies against). Reuses the `cyberos-audit-chain` and `cyberos-obs-sdk` shared crates and the tenant-RLS idiom.

Retires: the Mattermost-fork CHAT module - TASK-CHAT-001 through TASK-CHAT-012 and TASK-CHAT-013 - is superseded by this native series. `services/chat` is repurposed from the Mattermost fork to the `cyberos-chat` crate; the old Mattermost scaffolding (Dockerfile, patches, plugins, the python helpers, the deploy OIDC config) is archived out of the build, not deleted, so its history and any reusable logic remain available.

---

*End of TASK-CHAT-101.*
