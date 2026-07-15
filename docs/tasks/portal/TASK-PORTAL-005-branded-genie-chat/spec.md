---
id: TASK-PORTAL-005
title: "PORTAL branded Genie chat — CUO scope-narrowed by JWT scope_grants + per-Engagement brand pack + IdP-auth session integration + cross-tenant boundary enforcement"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-05-17T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: PORTAL
priority: p1
status: draft
verify: T
phase: P4
milestone: P4 · slice 2
slice: 2
owner: Stephen Cheng (CCO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-PORTAL-001, TASK-PORTAL-002, TASK-PORTAL-003, TASK-PORTAL-004, TASK-CUO-101, TASK-AUTH-004, TASK-AUTH-101, TASK-CHAT-005, TASK-AI-003, TASK-MEMORY-111, TASK-OBS-005]
depends_on: [TASK-PORTAL-003, TASK-CUO-101]
blocks: [TASK-EMAIL-008]

source_pages:
  - website/docs/modules/portal.html#branded-genie
  - website/docs/modules/cuo.html#scope-grants

source_decisions:
  - DEC-1170 2026-05-17 — Genie chat embedded in client-tenant portal (via TASK-PORTAL-001 scoped views); narrows CUO answers to ONLY data the JWT's scope_grants allow (a client user sees their own projects + docs, not the consulting firm's full org)
  - "DEC-1171 2026-05-17 — Scope_grants schema in JWT: array of `{ resource_type: enum, resource_ids: [uuid], permissions: [enum] }` per TASK-AUTH-004 extension; closed enum for resource_type per DEC-1172"
  - DEC-1172 2026-05-17 — Closed `cuo_scope_resource_type` enum = {project, document, invoice, channel, engagement}; CI cardinality test asserts 5
  - DEC-1173 2026-05-17 — Genie query path: client UI POST → PORTAL handler validates JWT + extracts scope_grants → invokes CUO with narrowed context → CUO returns answer scoped to allowed resources only → audit row records query + answer hash
  - DEC-1174 2026-05-17 — Out-of-scope answer attempt: CUO returns a `boundary_violation_detected` signal when its retrieval would touch scope-disallowed data; PORTAL converts to user-friendly response "I can only see resources in this engagement"
  - DEC-1175 2026-05-17 — Brand pack applied to Genie UI per TASK-PORTAL-002: colours, logo, font; default-pack used if tenant hasn't configured one
  - DEC-1176 2026-05-17 — Session integration with TASK-PORTAL-003: Genie session inherits IdP-auth subject + Engagement scope from the JWT; SCIM deprovision (TASK-PORTAL-004) cascades to active Genie sessions
  - DEC-1177 2026-05-17 — Per-Engagement Genie persona override: tenant_admin can set per-Engagement persona (e.g. "FormalLegal" vs "FriendlyOnboarding") via tenant config
  - DEC-1178 2026-05-17 — Conversation history per `(engagement_id, caller_subject_id)`; persisted in `portal_genie_sessions` table; default retention 90 days
  - DEC-1179 2026-05-17 — Rate limit: 60 messages/min/caller; sustained spikes trigger sev-2 alert (potential bot)
  - DEC-1180 2026-05-17 — memory audit kinds: portal.genie_query_issued, portal.genie_answer_emitted, portal.genie_boundary_violation_detected, portal.genie_session_created, portal.genie_session_archived
  - DEC-1181 2026-05-17 — Streaming responses via SSE (Server-Sent Events) so client UI sees tokens as they generate; falls back to JSON-once on non-SSE clients
  - DEC-1182 2026-05-17 — Cross-tenant boundary: CUO MUST refuse to retrieve from tenants other than the JWT's tenant_id; verified by TASK-CUO-101's doctor invariant `cuo.boundary_test` (mentioned in DEC-RESEARCH-REVIEW-RESPONSE.md §2.2)
  - DEC-1183 2026-05-17 — Audit-row PII: query + answer SHA256 hashes only in chain; raw text in `portal_genie_messages` (RLS-scoped, 90-day retention)
  - DEC-1184 2026-05-17 — IdP-auth subjects (`auth_method='external_idp'`) get 8h JWT TTL per TASK-PORTAL-003 DEC-879; Genie session re-auths at expiry; conversation history preserved across sessions

build_envelope:
  language: rust 1.81
  service: cyberos/services/portal/
  new_files:
    - services/portal/migrations/0012_portal_genie_sessions.sql        # session per (engagement, caller)
    - services/portal/migrations/0013_portal_genie_messages.sql        # message log with raw text
    - services/portal/src/genie/mod.rs                                 # orchestrator
    - services/portal/src/genie/query_handler.rs                       # POST /v1/portal/genie/query
    - services/portal/src/genie/scope_narrowing.rs                     # JWT scope_grants → CUO context filter
    - services/portal/src/genie/boundary_check.rs                      # cross-tenant + out-of-scope detection
    - services/portal/src/genie/sse_stream.rs                          # SSE response streaming
    - services/portal/src/genie/session.rs                             # session create/archive
    - services/portal/src/genie/persona.rs                             # per-Engagement persona resolver
    - services/portal/src/audit/genie_events.rs                        # 5 memory row builders
    - services/portal/src/handlers/genie_routes.rs
    - services/portal/tests/genie_query_happy_test.rs
    - services/portal/tests/genie_scope_narrowing_test.rs
    - services/portal/tests/genie_boundary_violation_test.rs
    - services/portal/tests/genie_cross_tenant_refused_test.rs
    - services/portal/tests/genie_session_persistence_test.rs
    - services/portal/tests/genie_persona_override_test.rs
    - services/portal/tests/genie_sse_streaming_test.rs
    - services/portal/tests/genie_brand_applied_test.rs
    - services/portal/tests/genie_rate_limit_test.rs
    - services/portal/tests/genie_scope_resource_type_enum_test.rs
    - services/portal/tests/genie_scim_deprovision_cascade_test.rs
    - services/portal/tests/genie_audit_emission_test.rs

  modified_files:
    - services/portal/src/lib.rs                                       # mount genie routes
    - services/auth/src/jwt/mint.rs                                    # add scope_grants claim to JWT mint
    - services/cuo/src/orchestrator.rs                                 # accept scope_grants context + enforce boundary

  allowed_tools:
    - file_read: services/portal/**
    - file_read: services/cuo/src/**
    - file_read: services/auth/src/jwt/**
    - file_write: services/portal/{src,tests,migrations}/**
    - file_write: services/auth/src/jwt/mint.rs
    - file_write: services/cuo/src/orchestrator.rs
    - bash: cd services/portal && cargo test genie

  disallowed_tools:
    - bypass scope_grants check on CUO invocation (per DEC-1173)
    - allow cross-tenant retrieval (per DEC-1182)
    - store raw query/answer in memory chain (per DEC-1183 — SHA only)
    - allow non-tenant_admin to set per-Engagement persona (per DEC-1177)
    - skip rate-limit check (per DEC-1179)
    - serve Genie without brand pack lookup (per DEC-1175 — default-pack ok, but lookup required)

effort_hours: 6
subtasks:
  - "0.4h: 0012_portal_genie_sessions.sql + 0013_portal_genie_messages.sql + RLS"
  - "0.4h: genie/mod.rs + scope_resource_type enum"
  - "0.5h: genie/scope_narrowing.rs — JWT scope_grants → CUO context filter"
  - "0.5h: genie/boundary_check.rs — pre/post-flight checks"
  - "0.5h: genie/sse_stream.rs — SSE token streaming"
  - "0.4h: genie/session.rs — session lifecycle"
  - "0.3h: genie/persona.rs — per-Engagement persona"
  - "0.4h: genie/query_handler.rs — orchestrator"
  - "0.3h: audit/genie_events.rs (5 builders)"
  - "0.3h: handlers/genie_routes.rs"
  - "0.4h: auth/jwt/mint.rs — scope_grants claim"
  - "0.4h: cuo/orchestrator.rs — scope_grants context + boundary enforcement"
  - "1.0h: tests — 12 test files covering happy + scope-narrowing + boundary + cross-tenant + session + persona + SSE + brand + rate-limit + enum + SCIM-cascade + audit"
  - "0.2h: integration smoke with brand pack + IdP-auth subject"

risk_if_skipped: "Without Branded Genie, client users in PORTAL see CyberSkill-branded Genie answering CyberSkill-related questions — totally wrong UX for white-label B2B2C. Without DEC-1170's scope narrowing, a client user could ask Genie about the consulting firm's other clients' data → catastrophic data leak. Without DEC-1182's cross-tenant boundary, Genie retrieves freely across tenant boundary → multi-tenant SaaS breaks. Without DEC-1174's boundary-violation response, the user sees raw errors instead of clear 'I can only see your engagement' messaging. Without DEC-1176's SCIM-cascade integration, deprovisioned users can still chat with Genie until their JWT expires (up to 8h). Without DEC-1178's conversation history, each interaction is amnesic — no context across messages. Without DEC-1181's SSE streaming, perceived latency is 5-15s vs <1s for first token. The 6h effort lands the white-label Genie experience that makes the PORTAL primitive a complete client-facing product."
---

## §1 — Description (BCP-14 normative)

The PORTAL service **MUST** ship branded Genie chat at `services/portal/src/genie/` integrating TASK-CUO-101 with PORTAL's IdP-auth subjects, scope_grants narrowing per JWT, TASK-PORTAL-002 brand application, cross-tenant boundary enforcement, SCIM-cascade revocation per TASK-PORTAL-004, SSE streaming, 5 memory audit kinds.

1. **MUST** define the closed `cuo_scope_resource_type` enum at migration `0012`: `('project','document','invoice','channel','engagement')` per DEC-1172. CI cardinality test asserts 5.

2. **MUST** define `portal_genie_sessions` table at migration `0012`: `(session_id UUID PRIMARY KEY, tenant_id UUID NOT NULL, engagement_id UUID NOT NULL, caller_subject_id UUID NOT NULL, persona_override TEXT, created_at TIMESTAMPTZ NOT NULL DEFAULT now(), last_activity_at TIMESTAMPTZ NOT NULL DEFAULT now(), archived_at TIMESTAMPTZ, revoked_at TIMESTAMPTZ, revoked_reason TEXT)`. Partial unique `(engagement_id, caller_subject_id) WHERE archived_at IS NULL AND revoked_at IS NULL` — one active session per (caller, engagement).

3. **MUST** define `portal_genie_messages` table at migration `0013`: `(id BIGSERIAL PRIMARY KEY, session_id UUID NOT NULL REFERENCES portal_genie_sessions(session_id), role TEXT NOT NULL CHECK (role IN ('user','assistant','system')), content_kms_blob BYTEA NOT NULL, content_sha256 CHAR(64) NOT NULL, scope_grants_snapshot JSONB NOT NULL, boundary_violations JSONB DEFAULT '[]'::jsonb, created_at TIMESTAMPTZ NOT NULL DEFAULT now())`. Append-only; raw content KMS-encrypted; 90-day retention via scheduled prune.

4. **MUST** enforce RLS with USING and WITH CHECK on both tables: `tenant_id = current_setting('auth.tenant_id')::uuid AND caller_subject_id = current_setting('auth.subject_id')::uuid` (per-caller scope).

5. **MUST** extend TASK-AUTH-004 JWT mint with `scope_grants` claim per DEC-1171. Claim shape:
    ```json
    "scope_grants": [
      { "resource_type": "project",    "resource_ids": ["uuid1","uuid2"], "permissions": ["read"] },
      { "resource_type": "document",   "resource_ids": ["uuid3"],         "permissions": ["read","comment"] },
      { "resource_type": "engagement", "resource_ids": ["uuid_eng"],      "permissions": ["read"] }
    ]
    ```
   JWT mint resolves the caller's TASK-AUTH-101 RBAC role → scope_grants from `engagement_memberships` table.

6. **MUST** expose `POST /v1/portal/genie/query` per DEC-1173. Body: `{ session_id?, message }`. Handler:
    - Validates JWT + extracts scope_grants.
    - Creates session if `session_id` absent (UUIDv7) or loads existing.
    - Rate-limit check per DEC-1179 (60/min/caller).
    - Pre-flight boundary check per §1 #7.
    - Invokes CUO with `{ message, scope_grants, persona }` context.
    - Post-flight boundary check on CUO response per §1 #8.
    - Returns SSE stream per DEC-1181; on success persists user + assistant messages.

7. **MUST** perform pre-flight boundary check at `services/portal/src/genie/boundary_check.rs::pre_flight(scope_grants, message)`:
    - Heuristic check on message intent (regex + keyword match for known cross-tenant query patterns like "all clients", "other engagements").
    - If suspicious: don't block; flag for stricter post-flight check.
    - Always: ensure CUO query context is scoped to `(tenant_id, engagement_id from JWT)`.

8. **MUST** perform post-flight boundary check on CUO response per DEC-1174:
    - CUO returns `retrieval_sources: [{type, id, tenant_id}]` metadata.
    - For each source: verify `source.tenant_id == jwt.tenant_id` AND `source.id ∈ scope_grants[source.type].resource_ids`.
    - Violation found: replace response with user-friendly message "I can only see resources in this engagement"; emit `portal.genie_boundary_violation_detected` sev-1 (security-critical); persist violation details in `portal_genie_messages.boundary_violations` JSONB.

9. **MUST** apply per-Engagement brand from TASK-PORTAL-002 per DEC-1175. The Genie UI inlines per-Engagement brand colours + logo via the `portal_brand_pack_active` lookup; default pack if none configured.

10. **MUST** apply per-Engagement persona override per DEC-1177. `portal_genie_sessions.persona_override` populated from `engagements.genie_persona` (slice 3 extension; slice 2 reads from in-memory config). Persona name passed to CUO as context modifier.

11. **MUST** stream SSE response per DEC-1181. Server-Sent Events at `/v1/portal/genie/query` with `Content-Type: text/event-stream`. Each event = one token. Client opens persistent connection; CUO emits tokens; server forwards as SSE events. Final event = `{ done: true, message_id, final_text }`. Non-SSE clients (Accept: application/json) get full response after CUO completes.

12. **MUST** invalidate Genie sessions on TASK-PORTAL-004 SCIM deprovision per DEC-1176. The cascade target `Genie sessions` from TASK-PORTAL-004 §1 #6 UPDATE `portal_genie_sessions SET revoked_at=now(), revoked_reason='scim_deprovision' WHERE caller_subject_id=$1 AND engagement_id=$2`. Active SSE connections close per TASK-PORTAL-004 §1 #5 (WebSocket close 4001 equivalent for SSE = abort stream + emit `error` event).

13. **MUST** rate-limit at 60 messages/min/caller per DEC-1179. Sliding-window via Redis. Excess returns SSE event `{ error: "rate_limited", retry_after_seconds: 60 }` + closes stream.

14. **MUST** persist conversation history per DEC-1178. Messages stored with role (user/assistant/system), KMS-encrypted content, scope_grants snapshot (audit forensic — what permissions did the caller have when this answer was generated). Default 90-day retention via daily prune job.

15. **MUST** emit 5 memory audit row kinds per DEC-1180:
    - `portal.genie_query_issued` (sev-3 — high-volume; sampled at 1% via TASK-OBS-006)
    - `portal.genie_answer_emitted` (sev-3 — sampled)
    - `portal.genie_boundary_violation_detected` (sev-1 — security-critical, ALWAYS emitted not sampled)
    - `portal.genie_session_created` (sev-3)
    - `portal.genie_session_archived` (sev-3)

16. **MUST** PII-scrub per DEC-1183. Query + answer SHA256 hashes only in memory chain; raw text in `portal_genie_messages.content_kms_blob` (RLS-scoped, 90-day retention).

17. **MUST** enforce cross-tenant boundary at CUO invocation per DEC-1182. CUO orchestrator receives `scope_grants` context + `tenant_id` from JWT; CUO MUST NOT retrieve from any other tenant_id. Verified by `cuo.boundary_test` doctor invariant (TASK-CUO-101 derivative). Violation = `boundary_violation_detected` audit row + caller sees safe error.

18. **MUST** thread W3C trace_id end-to-end per task-audit skill rule 22-24. Genie query trace_id propagates through CUO invocation + audit rows + SSE events.

19. **MUST** expose session list at `GET /v1/portal/genie/sessions?engagement_id=...` per caller. Returns recent sessions (last 30 days; max 50). Tenant_admin sees all sessions for the Engagement via separate handler.

20. **MUST** support session archival at `POST /v1/portal/genie/sessions/{id}/archive`. Sets `archived_at=now()`; conversation history retained per 90-day retention but no longer surfaced in default list.

21. **MUST** detect and refuse cross-tenant queries at the boundary check per DEC-1182 + §1 #8. Test fixture: client user in tenant X attempts to query about tenant Y's data → CUO retrieval blocked (returns 0 sources from tenant Y) + boundary_violation_detected audit.

22. **SHOULD** observe per-Engagement query volume + latency via OTel histogram `portal_genie_query_duration_seconds{engagement_id}` for operator visibility.

23. **SHOULD** auto-archive idle sessions after 30 days of no activity (daily job) per DEC-1178 implementation note.

---

## §2 — Why this design (rationale for humans)

**Why scope_grants in JWT (§1 #5, DEC-1171)?** The JWT is presented at every Genie query; reading scope_grants is free. Database lookup per query = latency + DB load. JWT mint computes scope_grants once from RBAC + Engagement membership; CUO consumes signed scope without re-verification. Standard JWT-as-credential pattern.

**Why closed enum cuo_scope_resource_type (§1 #1, DEC-1172)?** Free-form types invite mis-spellings + scope-creep. 5 resource types cover the slice-2 use cases (project, document, invoice, channel, engagement). Adding a 6th = schema migration + audit. Forces deliberate scope expansion.

**Why pre-flight + post-flight boundary checks (§1 #7 + §1 #8)?** Defense-in-depth. Pre-flight catches obvious cross-tenant intent ("show me all clients"); post-flight catches CUO over-retrieval (a CUO bug that touches disallowed data). Either alone misses cases; both together approach zero leakage.

**Why "I can only see X" user-friendly message (§1 #8, DEC-1174)?** Raw errors ("403 boundary_violation_detected") confuse end-users. Friendly framing avoids "support escalation about confusing error" while still security-correct (we don't reveal what they're not allowed to see).

**Why per-caller session scope (§1 #2 partial unique, DEC-1178)?** Multi-session per caller per engagement = confusion (which conversation is the user in?). One active session = clean UX; user can explicitly archive to start fresh.

**Why SSE streaming (§1 #11, DEC-1181)?** Perceived latency dominates UX; first-token latency < 1s feels instant; 5s feels slow. CUO can take 5-15s for complex answers; SSE shows tokens as they're generated. Standard pattern (ChatGPT, Claude.ai, Copilot Chat all use SSE).

**Why 60 msg/min rate limit (§1 #13, DEC-1179)?** Genie is a chat — typing 60 msgs/min = ~1/sec which is faster than humans type. Above = bot or stuck retry loop; bound the abuse.

**Why brand pack inheritance from TASK-PORTAL-002 (§1 #9, DEC-1175)?** Single source of truth for tenant branding; Genie shouldn't have its own brand config. PORTAL-002 brand pack applies everywhere PORTAL renders, including Genie.

**Why SCIM-cascade (§1 #12, DEC-1176)?** Active Genie chat to a deprovisioned user is a 8h window of unauthorized chat. PORTAL-004 cascade target keeps the deprovision SLO tight (30s) across Genie too.

**Why 90-day retention on messages (§1 #14, DEC-1178)?** Long enough for the user to come back next sprint + re-find a useful answer; bounded to limit storage growth. Matches TASK-TEN-101 signup_sessions pattern.

**Why scope_grants_snapshot per message (§1 #3, DEC-1183 derivative)?** Forensic-critical. Audit question: "Did this user have permission to ask about X at the time the answer was generated?" Snapshot answers it exactly; reconstructing post-hoc (RBAC + Engagement membership changes over time) is unreliable.

**Why cross-tenant boundary at CUO level (§1 #17, DEC-1182)?** PORTAL is one layer; CUO is the retrieval layer. Defense-in-depth means BOTH must enforce. CUO's `cuo.boundary_test` doctor invariant validates this regardless of which surface (PORTAL, CHAT, CLI) invokes CUO.

**Why per-Engagement persona override (§1 #10, DEC-1177)?** Different client tones — a law firm wants formal Genie; a marketing agency wants friendly. One-Genie-fits-all = brand-mismatch for at least half the tenants.

---

## §3 — API contract

### 3.1 Postgres schema

```sql
-- 0012_portal_genie_sessions.sql
CREATE TYPE cuo_scope_resource_type AS ENUM ('project','document','invoice','channel','engagement');

CREATE TABLE portal_genie_sessions (
  session_id UUID PRIMARY KEY,
  tenant_id UUID NOT NULL,
  engagement_id UUID NOT NULL,
  caller_subject_id UUID NOT NULL,
  persona_override TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  last_activity_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  archived_at TIMESTAMPTZ,
  revoked_at TIMESTAMPTZ,
  revoked_reason TEXT
);
CREATE UNIQUE INDEX uniq_active_genie_session
  ON portal_genie_sessions(engagement_id, caller_subject_id)
  WHERE archived_at IS NULL AND revoked_at IS NULL;
CREATE INDEX idx_genie_sessions_caller ON portal_genie_sessions(caller_subject_id, last_activity_at DESC);
ALTER TABLE portal_genie_sessions ENABLE ROW LEVEL SECURITY;
CREATE POLICY portal_genie_sessions_rls ON portal_genie_sessions
  USING (tenant_id = current_setting('auth.tenant_id')::uuid
         AND caller_subject_id = current_setting('auth.subject_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid
              AND caller_subject_id = current_setting('auth.subject_id')::uuid);
REVOKE DELETE ON portal_genie_sessions FROM cyberos_app;
GRANT UPDATE (last_activity_at, archived_at, revoked_at, revoked_reason, persona_override)
  ON portal_genie_sessions TO cyberos_app;

-- 0013_portal_genie_messages.sql
CREATE TABLE portal_genie_messages (
  id BIGSERIAL PRIMARY KEY,
  session_id UUID NOT NULL REFERENCES portal_genie_sessions(session_id),
  role TEXT NOT NULL CHECK (role IN ('user','assistant','system')),
  content_kms_blob BYTEA NOT NULL,
  content_sha256 CHAR(64) NOT NULL,
  kms_key_id TEXT NOT NULL,
  scope_grants_snapshot JSONB NOT NULL,
  boundary_violations JSONB NOT NULL DEFAULT '[]'::jsonb,
  trace_id CHAR(32),
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_genie_messages_session ON portal_genie_messages(session_id, created_at);
ALTER TABLE portal_genie_messages ENABLE ROW LEVEL SECURITY;
CREATE POLICY portal_genie_messages_rls ON portal_genie_messages
  USING (session_id IN (SELECT session_id FROM portal_genie_sessions
                          WHERE caller_subject_id = current_setting('auth.subject_id')::uuid))
  WITH CHECK (session_id IN (SELECT session_id FROM portal_genie_sessions
                               WHERE caller_subject_id = current_setting('auth.subject_id')::uuid));
REVOKE UPDATE, DELETE ON portal_genie_messages FROM cyberos_app;
GRANT DELETE ON portal_genie_messages TO cyberos_pruner;  -- 90-day retention
```

### 3.2 Rust types

```rust
// services/portal/src/genie/scope_narrowing.rs
#[derive(Copy, Clone, Eq, PartialEq, Debug, sqlx::Type, serde::Serialize, serde::Deserialize)]
#[sqlx(type_name = "cuo_scope_resource_type", rename_all = "snake_case")]
pub enum CuoScopeResourceType { Project, Document, Invoice, Channel, Engagement }

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ScopeGrant {
    pub resource_type: CuoScopeResourceType,
    pub resource_ids: Vec<Uuid>,
    pub permissions: Vec<String>,
}

// Tool-side CUO invocation context
#[derive(serde::Serialize, Debug)]
pub struct CuoQueryContext {
    pub tenant_id: Uuid,
    pub engagement_id: Uuid,
    pub caller_subject_id: Uuid,
    pub scope_grants: Vec<ScopeGrant>,
    pub persona: Option<String>,
    pub trace_id: String,
}
```

### 3.3 REST endpoints

```text
POST   /v1/portal/genie/query                                (SSE or JSON; caller-owned)
GET    /v1/portal/genie/sessions?engagement_id=...           (caller-owned list)
GET    /v1/portal/genie/sessions/{id}/messages               (caller-owned history)
POST   /v1/portal/genie/sessions/{id}/archive                (caller-owned)
```

---

## §4 — Acceptance criteria

1. **cuo_scope_resource_type cardinality** — enum = exactly `{project, document, invoice, channel, engagement}`.
2. **Happy query path** — POST `{message: "show my projects"}` → CUO returns scoped list → SSE stream tokens → final message persisted.
3. **Scope narrowing** — JWT scope_grants = `[{type:'project', ids:[uuid1]}]`; CUO retrieval returns ONLY uuid1; uuid2 (different project) NOT in retrieval.
4. **Boundary violation** — CUO retrieves a document not in scope_grants → post-flight detects + replaces answer with safe message + sev-1 audit.
5. **Cross-tenant refused** — caller in tenant X queries; CUO retrieval attempts tenant Y document → 0 results from tenant Y + boundary_violation_detected audit.
6. **Session persistence** — second query with `session_id` from first → same session row updated, conversation history accumulates.
7. **Per-Engagement persona override** — engagement A configured persona "FormalLegal"; tenant_admin sees Genie response styled accordingly.
8. **SSE streaming** — Accept: text/event-stream → response is SSE; tokens stream as generated; final event has `done:true`.
9. **Non-SSE JSON fallback** — Accept: application/json → full response after CUO completes.
10. **Brand pack applied** — Genie UI fetches brand pack from TASK-PORTAL-002; falls back to default if none.
11. **Rate limit 60/min** — 61st query in 60s → SSE event `{error: "rate_limited"}`.
12. **SCIM-cascade revoke** — TASK-PORTAL-004 SCIM DELETE cascades to active Genie session → `revoked_at` set + active SSE aborted.
13. **Conversation history view** — GET `/sessions/{id}/messages` returns decrypted messages in order.
14. **Session archive** — POST archive → `archived_at` set + no longer in default list.
15. **Idle session auto-archive** — fixture session with `last_activity_at` 31 days ago → daily job archives.
16. **Trace_id threaded** — single trace_id across query + CUO invocation + audit rows + SSE events.
17. **5 memory audit kinds emitted** — happy path covers query_issued + answer_emitted + session_created; failure paths cover boundary_violation + session_archived.
18. **PII scrub** — audit row carries `content_sha256` only; raw text in DB.
19. **scope_grants in JWT** — minted JWT contains scope_grants claim; validator passes through to handler.
20. **90-day retention prune** — fixture message > 90d old → daily prune sets content_kms_blob=NULL.

---

## §5 — Verification

### 5.1 `genie_query_happy_test.rs`

```rust
#[tokio::test]
async fn happy_query_returns_scoped_response() {
    let ctx = TestContext::with_engagement_subject_with_scope(vec![
        ScopeGrant { resource_type: CuoScopeResourceType::Project, resource_ids: vec![ctx.proj_a], permissions: vec!["read".into()] }
    ]).await;
    let r = ctx.post_genie_query("show my projects").await;
    let messages = ctx.collect_sse_messages(r).await;
    assert!(messages.iter().any(|m| m.contains(&ctx.proj_a.to_string())));
    assert!(!messages.iter().any(|m| m.contains(&ctx.proj_b.to_string())));
}
```

### 5.2 `genie_scope_narrowing_test.rs`

```rust
#[tokio::test]
async fn cuo_retrieval_narrowed_to_jwt_scope() {
    let ctx = TestContext::with_scoped_subject(vec![ScopeGrant {
        resource_type: CuoScopeResourceType::Document, resource_ids: vec![ctx.doc1], permissions: vec!["read".into()]
    }]).await;
    let r = ctx.post_genie_query("summarize all documents").await;
    let cuo_call = ctx.last_cuo_invocation().await;
    assert_eq!(cuo_call.context.scope_grants.len(), 1);
    assert_eq!(cuo_call.context.scope_grants[0].resource_ids, vec![ctx.doc1]);
    assert!(!cuo_call.retrieval_sources.iter().any(|s| s.id == ctx.doc2));
}
```

### 5.3 `genie_boundary_violation_test.rs`

```rust
#[tokio::test]
async fn boundary_violation_replaces_response() {
    let ctx = TestContext::with_scoped_subject(vec![]).await;
    ctx.force_cuo_to_retrieve_unauthorized_doc(ctx.unauthorized_doc).await;
    let r = ctx.post_genie_query("explain x").await;
    let msgs = ctx.collect_sse_messages(r).await;
    let full = msgs.join("");
    assert!(full.contains("I can only see resources in this engagement"));

    let audit = ctx.memory_rows().await;
    assert!(audit.iter().any(|r| r.kind == "portal.genie_boundary_violation_detected" && r.severity == 1));
}
```

### 5.4 `genie_cross_tenant_refused_test.rs`

```rust
#[tokio::test]
async fn cross_tenant_query_blocked_at_cuo() {
    let ctx = TestContext::with_two_tenants().await;
    let r = ctx.as_caller("a").post_genie_query("what does tenant b have").await;
    let _ = ctx.collect_sse_messages(r).await;

    let cuo_call = ctx.last_cuo_invocation().await;
    assert_eq!(cuo_call.context.tenant_id, ctx.tenant_a_id);
    assert!(cuo_call.retrieval_sources.iter().all(|s| s.tenant_id == ctx.tenant_a_id));
}
```

### 5.5 `genie_session_persistence_test.rs`

```rust
#[tokio::test]
async fn second_query_continues_session() {
    let ctx = TestContext::new().await;
    let r1 = ctx.post_genie_query_new_session("hello").await;
    let session_id: Uuid = r1.json::<serde_json::Value>().await.unwrap()["session_id"].as_str().unwrap().parse().unwrap();

    let r2 = ctx.post_genie_query_in_session(session_id, "follow up").await;
    let messages: Vec<(String,String)> = sqlx::query_as(
        "SELECT role, encode(content_sha256, 'escape') FROM portal_genie_messages WHERE session_id=$1 ORDER BY id"
    ).bind(session_id).fetch_all(&ctx.pool).await.unwrap();
    assert!(messages.len() >= 4);  // 2 user + 2 assistant
}
```

### 5.6 `genie_scim_deprovision_cascade_test.rs`

```rust
#[tokio::test]
async fn scim_delete_revokes_active_genie_session() {
    let ctx = TestContext::with_engagement_subject_with_jwt("alice@acme.com").await;
    let session_id = ctx.create_active_genie_session().await;
    ctx.scim_delete_user(ctx.subject_id).await;
    tokio::time::sleep(Duration::from_secs(2)).await;  // async cascade

    let revoked: Option<DateTime<Utc>> = sqlx::query_scalar(
        "SELECT revoked_at FROM portal_genie_sessions WHERE session_id=$1"
    ).bind(session_id).fetch_one(&ctx.pool).await.unwrap();
    assert!(revoked.is_some());
}
```

### 5.7 `genie_persona_override_test.rs`

```rust
#[tokio::test]
async fn engagement_persona_passed_to_cuo() {
    let ctx = TestContext::new().await;
    ctx.set_engagement_persona(ctx.eng_id, "FormalLegal").await;
    let r = ctx.post_genie_query("hello").await;
    let _ = ctx.collect_sse_messages(r).await;
    let cuo_call = ctx.last_cuo_invocation().await;
    assert_eq!(cuo_call.context.persona, Some("FormalLegal".into()));
}
```

### 5.8 `genie_sse_streaming_test.rs`

```rust
#[tokio::test]
async fn sse_streams_tokens_progressively() {
    let ctx = TestContext::new().await;
    let r = ctx.post_genie_query_sse("explain x").await;
    let mut events = ctx.sse_event_iter(r).await;
    let first_token = events.next().await.unwrap();
    assert!(first_token.data.contains("data:"));
    let final_event = events.last().await.unwrap();
    assert!(final_event.data.contains("\"done\":true"));
}
```

### 5.9 `genie_rate_limit_test.rs`

```rust
#[tokio::test]
async fn rate_limit_at_60_per_min() {
    let ctx = TestContext::new().await;
    for _ in 0..60 {
        let _ = ctx.post_genie_query("hi").await;
    }
    let r = ctx.post_genie_query("one more").await;
    let msgs = ctx.collect_sse_messages(r).await;
    assert!(msgs.iter().any(|m| m.contains("rate_limited")));
}
```

### 5.10 `genie_scope_resource_type_enum_test.rs`

```rust
#[tokio::test]
async fn cuo_scope_resource_type_has_5_values() {
    let ctx = TestContext::new().await;
    let labels: Vec<String> = sqlx::query_scalar(
        "SELECT unnest(enum_range(NULL::cuo_scope_resource_type))::text"
    ).fetch_all(&ctx.pool).await.unwrap();
    let mut labels = labels; labels.sort();
    assert_eq!(labels, vec!["channel","document","engagement","invoice","project"]);
}
```

---

## §6 — Implementation skeleton

### 6.1 Genie query orchestrator

```rust
// services/portal/src/genie/query_handler.rs
pub async fn query(ctx: AppCtx, jwt: JwtClaims, req: GenieQueryReq) -> Result<SseResponse, GenieError> {
    // Rate limit
    if !rate_limit_check(&ctx, jwt.subject_id).await? { return Err(GenieError::RateLimited); }

    // Load or create session
    let session_id = match req.session_id {
        Some(id) => { ctx.repo.genie_sessions.touch(id, jwt.subject_id).await?; id }
        None => create_session(&ctx, &jwt, &req.engagement_id).await?,
    };

    // Persona resolution
    let persona = persona::resolve(&ctx, jwt.engagement_id).await?;

    // CUO context
    let cuo_ctx = CuoQueryContext {
        tenant_id: jwt.tenant_id,
        engagement_id: jwt.engagement_id,
        caller_subject_id: jwt.subject_id,
        scope_grants: jwt.scope_grants.clone(),
        persona,
        trace_id: ctx.trace_id.clone(),
    };

    // Persist user message
    let scope_snapshot = serde_json::to_value(&jwt.scope_grants)?;
    let user_msg_id = persist_message(&ctx, session_id, "user", &req.message, &scope_snapshot, vec![]).await?;

    emit_audit(&ctx, "portal.genie_query_issued", json!({
        "session_id": session_id, "message_sha256": sha256_hex(req.message.as_bytes()),
    })).await;

    // Invoke CUO; stream response
    let (stream, retrieval_sources_handle) = ctx.cuo.invoke_streaming(cuo_ctx, req.message.clone()).await?;
    let (tx, rx) = tokio::sync::mpsc::channel(32);
    let ctx_clone = ctx.clone();
    tokio::spawn(async move {
        let mut accumulated = String::new();
        while let Some(token) = stream.next().await {
            accumulated.push_str(&token);
            let _ = tx.send(SseEvent::data(json!({"token": token}))).await;
        }

        // Post-flight boundary check on retrieval_sources
        let sources = retrieval_sources_handle.await.unwrap_or_default();
        let violations = boundary_check::post_flight(&ctx_clone, &jwt, &sources);
        if !violations.is_empty() {
            accumulated = "I can only see resources in this engagement. If you need broader access, please contact your administrator.".into();
            emit_audit(&ctx_clone, "portal.genie_boundary_violation_detected", json!({
                "session_id": session_id, "violations": violations,
            })).await;
        }

        let assistant_scope_snapshot = serde_json::to_value(&jwt.scope_grants).unwrap();
        let assistant_msg_id = persist_message(&ctx_clone, session_id, "assistant", &accumulated, &assistant_scope_snapshot, violations).await.ok();

        emit_audit(&ctx_clone, "portal.genie_answer_emitted", json!({
            "session_id": session_id, "message_id": assistant_msg_id,
            "answer_sha256": sha256_hex(accumulated.as_bytes()),
        })).await;

        let _ = tx.send(SseEvent::data(json!({"done": true, "message_id": assistant_msg_id}))).await;
    });

    Ok(SseResponse::from_rx(rx))
}
```

### 6.2 Post-flight boundary check

```rust
// services/portal/src/genie/boundary_check.rs
pub fn post_flight(ctx: &AppCtx, jwt: &JwtClaims, sources: &[RetrievalSource]) -> Vec<BoundaryViolation> {
    let mut violations = Vec::new();
    for source in sources {
        if source.tenant_id != jwt.tenant_id {
            violations.push(BoundaryViolation::CrossTenant {
                expected: jwt.tenant_id, actual: source.tenant_id, source_id: source.id,
            });
            continue;
        }
        let allowed = jwt.scope_grants.iter().any(|g| {
            g.resource_type == source.resource_type && g.resource_ids.contains(&source.id)
        });
        if !allowed {
            violations.push(BoundaryViolation::OutOfScope {
                resource_type: source.resource_type, resource_id: source.id,
            });
        }
    }
    violations
}
```

---

## §7 — Dependencies

**Upstream (depends_on):**
- **TASK-PORTAL-003** External IdP + SCIM JIT — IdP-auth subjects + JWT scope_grants pattern.
- **TASK-CUO-101** LangGraph supervisor — CUO orchestrator invoked here; receives scope_grants context + enforces boundary via doctor invariant.

**Cross-module (related_tasks):**
- **TASK-PORTAL-001** Scoped read-only views — same scope_grants apply.
- **TASK-PORTAL-002** Brand pack — Genie UI inherits brand.
- **TASK-PORTAL-004** SCIM deprovision — cascade target.
- **TASK-AUTH-004** JWT mint — scope_grants claim extension here.
- **TASK-AUTH-101** RBAC — scope_grants derived from RBAC + Engagement membership.
- **TASK-CHAT-005** CHAT — Genie may surface inline in chat.
- **TASK-AI-003** memory audit-row bridge — 5 new kinds.
- **TASK-MEMORY-111** PII scrubbing — content SHA only.
- **TASK-OBS-005** Trace correlation — trace_id end-to-end.

**Downstream (blocks):** None.

---

## §8 — Example payloads

### 8.1 Genie query request

```json
{
  "session_id": null,
  "engagement_id": "0190f7c0-8b3c-7a4f-aaaa-000000000001",
  "message": "Summarize the documents for the Q2 review"
}
```

### 8.2 SSE event stream

```text
event: message
data: {"token": "Based"}

event: message
data: {"token": " on"}

event: message
data: {"token": " the"}
...
event: message
data: {"done": true, "message_id": 12345, "final_text": "Based on the 3 documents in your scope..."}
```

### 8.3 `portal.genie_boundary_violation_detected` memory row

```json
{
  "kind": "portal.genie_boundary_violation_detected",
  "severity": 1,
  "tenant_id": "8a2f...",
  "actor_id": "user.subject.456",
  "trace_id": "0af7651916cd43dd8448eb211c80319c",
  "occurred_at": "2026-05-17T09:14:32.847Z",
  "payload": {
    "session_id": "0190f7c0-...",
    "engagement_id": "0190f7c0-...",
    "caller_subject_id_hash16": "f8a1b2c3d4e5f607",
    "violations": [
      { "type": "OutOfScope", "resource_type": "document", "resource_id": "9c4e7a8b-..." }
    ],
    "user_message_sha256": "ab12cd34..."
  }
}
```

---

## §9 — Open questions

All resolved for slice 2. Deferred:

- **Deferred:** Multi-modal Genie input (image upload, file attach) — slice 3.
- **Deferred:** Voice input/output for Genie — slice 4.
- **Deferred:** Conversation export (PDF transcript) — slice 3.
- **Deferred:** Per-message reaction (thumbs up/down for feedback) — slice 3.
- **Deferred:** Cross-session search ("find when I asked about X") — slice 3.
- **Deferred:** Genie-to-tool action (Genie suggesting tool call requiring user confirm) — slice 4, requires TASK-MCP-008 integration.
- **Deferred:** Tenant_admin view of all caller sessions — slice 3.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| CUO retrieves cross-tenant data | post-flight check | Response replaced with safe message; sev-1 audit | Operator investigates CUO; TASK-CUO-101 doctor invariant should have caught |
| CUO retrieves out-of-scope (same tenant, wrong resource) | post-flight | Same: safe message + audit | Operator reviews CUO's retrieval logic |
| Pre-flight heuristic false-positive (legit query flagged) | not blocked; only flag | Post-flight still runs; query proceeds | Inherent — pre-flight is hint not gate |
| Rate limit hit | Redis counter | SSE event rate_limited; stream closes | Caller backs off |
| CUO error (model unavailable) | CUO returns Err | SSE event error; session message logged with error | Caller retries |
| SSE connection drops mid-stream | TCP reset | Client sees partial response; reconnect with session_id loads accumulated history | Inherent |
| SCIM deprovision mid-query | revoked_at set during query | Active SSE aborted (error event); subsequent queries 401 | Inherent cascade |
| Brand pack lookup fails | KMS or DB error | Default pack used + sev-3 log | Inherent fallback |
| Persona override invalid | unknown persona name | CUO uses default persona + sev-3 log | Admin sets valid persona |
| Conversation history > 1000 messages | length cap | Truncate context window to recent 50 + summary | Inherent |
| Cross-tenant subject ID in scope_grants | RLS rejects | Query fails with 403 + sev-1 audit | Investigate JWT mint bug |
| KMS unavailable for message decrypt (history view) | KMS timeout | 503 + temp unavailable | AWS KMS recovery |
| Idle session > 30 days | daily archive job | session.archived_at set | Inherent |
| 90-day message retention prune | daily job | content_kms_blob set NULL; metadata retained | Inherent |
| Session_id provided but belongs to different caller | RLS rejects | 403 | Caller uses own session_id |
| CUO emits sources without tenant_id metadata | post-flight needs the field | Sev-1 audit; conservative reject (assume violation) | TASK-CUO-101 schema fix |
| Scope_grants empty (no permissions) | JWT mint produces empty array | All queries return safe message ("no access") | Tenant_admin assigns membership |
| SSE backpressure from slow client | tokio channel full | CUO stream pauses; client should catch up | Inherent |
| Multiple concurrent queries on same session | last writer wins on last_activity_at | UX may show interleaved responses; client should serialize | UI-side serialisation |
| Boundary violation in pre-flight (cross-tenant intent keyword) | regex hit | Not blocked at pre-flight; post-flight still runs | Inherent — heuristic |
| Genie answer contains hallucinated tenant boundary leak (model says "client B did X") | not detectable at retrieval layer | Sev-3 informational; future enhancement: answer-content filter | Slice 4 LLM-side hallucination check |
| Persona prompt injection by malicious user message | prompt-injection guard | Stripped from context before CUO; sev-3 log | Inherent CUO defense |

---

## §11 — Implementation notes

**§11.1** SSE Content-Type `text/event-stream`; uses `axum::response::sse::Sse` wrapper.

**§11.2** Rate limiter is sliding-window Redis (same pattern as TASK-TEN-101 + TASK-MCP-007).

**§11.3** scope_grants in JWT may grow large at high-RBAC users (tenant_admin with 50 projects). JWT size budget: keep < 8 KB to fit cookie limits. Mitigation: tenant_admin scopes use wildcard `resource_ids: ["*"]` semantic (slice 3); slice 2 enumerates IDs (acceptable for typical 5-20 projects).

**§11.4** Wildcard `*` in resource_ids interpretation deferred to slice 3; slice 2 = literal UUID list only.

**§11.5** SSE event format follows W3C spec: `event:` + `data:` + blank line per event.

**§11.6** Pre-flight heuristic keywords: deny-list of intent phrases ("all clients", "other engagements", "tenant X"); future ML-based intent classifier (slice 4).

**§11.7** Post-flight enforcement: defense-in-depth — CUO's `cuo.boundary_test` invariant SHOULD prevent cross-tenant retrieval; this task catches CUO bugs.

**§11.8** Persona resolution: per-Engagement override > tenant default > CyberOS default. Cached in memory at handler startup; reload on engagement config change.

**§11.9** Conversation history retention: 90 days; daily prune via `cyberos_pruner` role (same as TASK-MCP-007 pattern).

**§11.10** scope_grants_snapshot per message critical for audit: "what permissions did caller have when answering at T?" reconstruct-from-snapshot only.

**§11.11** Idle session archive: 30 days no activity; daily job; caller can resume by creating new session_id (history view via `?include_archived=true`).

**§11.12** Boundary-violation audit always sev-1 + ALWAYS emitted (not sampled per OBS tail-sampling); security-critical.

**§11.13** Cross-session search (slice 3) requires full-text index on `portal_genie_messages.content` — gated on KMS-decrypt cost.

**§11.14** Genie response streaming uses CUO's streaming endpoint (TASK-CUO-101 derivative); non-streaming CUO models accumulate then emit single-event.

**§11.15** UI integration: `/v1/portal/genie/widget.html` (slice 3) serves the chat widget; slice 2 = API-only, frontend integrated by PORTAL-002 brand pack templates.

**§11.16** Cross-caller session_id access blocked by RLS; cross-Engagement same-caller blocked by `(engagement_id, caller_subject_id)` partial unique + the engagement_id check at handler.

**§11.17** Per-tool MCP integration via TASK-MCP-008 elicitation (slice 4): Genie suggests tool calls; user confirms via elicitation; tool runs; response continued.

**§11.18** OBS metric `portal_genie_query_duration_seconds` includes label `tenant_id` (cardinality OK at 100s of tenants).

**§11.19** SSE keepalive: send `: keepalive` comment every 30s to prevent intermediate proxy timeout.

**§11.20** Boundary violation kind always at sev-1 because security-critical; even sampled-OBS doesn't drop.

---

*End of TASK-PORTAL-005 spec.*
