---
id: TASK-MCP-008
title: "MCP Elicitation — server-initiated structured prompts for mid-call user input (clarifications, confirmations, missing args) with timeout + cancellation"
module: MCP
priority: MUST
status: implementing
verify: T
phase: P0
milestone: P0 · slice 3
slice: 3
owner: Stephen Cheng (CTO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-MCP-001, TASK-MCP-004, TASK-MCP-006, TASK-MCP-007, TASK-AUTH-004, TASK-AI-003, TASK-MEMORY-111, TASK-CHAT-005, TASK-OBS-005, TASK-OBS-007]
depends_on: [TASK-MCP-001, TASK-MCP-004]
blocks: []

source_pages:
  - website/docs/modules/mcp.html#elicitation
  - https://modelcontextprotocol.io/specification/2025-11-25/server/utilities/elicitation
  - https://datatracker.ietf.org/doc/html/rfc8259  # JSON

source_decisions:
  - DEC-1140 2026-05-17 — Elicitation per MCP 2025-11-25 spec: tool invoked → mid-execution server emits structured `elicitation/request` to caller → caller responds with `elicitation/response` → tool resumes with the answer; alternative to "fail with missing-arg-error and have caller retry"
  - DEC-1141 2026-05-17 — Closed enum `elicitation_type` = {string_input, single_choice, multi_choice, confirmation, file_upload}; CI cardinality test asserts 5
  - DEC-1142 2026-05-17 — Elicitation request carries a JSON Schema describing expected response shape; server validates the response against the schema before resuming the tool
  - DEC-1143 2026-05-17 — Elicitation timeout default 5 min; configurable per request (max 30 min); on timeout → tool errors with `elicitation_timeout`
  - DEC-1144 2026-05-17 — Elicitation pending request stored in `mcp_elicitations` table with TTL = timeout_seconds; expired pending → status='expired' + tool resumes with timeout error
  - DEC-1145 2026-05-17 — Elicitation transport: HTTP polling (caller GETs pending elicitations for their session) + NATS push (optional); polling REQUIRED, push OPTIONAL — mirrors TASK-MCP-007 progress polling pattern
  - DEC-1146 2026-05-17 — Per-tool registration declares `supports_elicitation: bool`; tools without this flag MAY NOT emit elicitation requests (fail loudly if attempted)
  - DEC-1147 2026-05-17 — Elicitation in sync tool calls = anti-pattern (sync tools should return on missing args, not block waiting for user); ALLOWED for long-running tasks (TASK-MCP-007) ONLY at slice 3
  - DEC-1148 2026-05-17 — Cancellation: caller can cancel pending elicitation via `POST /v1/mcp/elicitations/{id}/cancel`; tool resumes with `elicitation_cancelled` error
  - DEC-1149 2026-05-17 — Per-elicitation memory audit: emits `mcp.elicitation_requested`, `mcp.elicitation_responded`, `mcp.elicitation_timeout`, `mcp.elicitation_cancelled`, `mcp.elicitation_validation_failed`
  - DEC-1150 2026-05-17 — Response validation: invalid JSON → 400; schema validation failure → 422 + `validation_errors` array; tool MAY re-elicit (max 3 retries per logical prompt)
  - DEC-1151 2026-05-17 — TASK-MCP-006 integration: destructive tool with `confirmation` elicitation_type satisfies the TASK-MCP-006 confirm-mode requirement (`elicit` mode in policy)
  - "DEC-1152 2026-05-17 — Confirmation elicitation MUST present clear action description; per DEC-1141 confirmation type has fixed schema `{ confirmed: boolean, reason?: string }`"
  - DEC-1153 2026-05-17 — File-upload elicitation_type returns a presigned S3 URL (TASK-DOC-001) for the caller to upload to; max 100 MiB; ttl = elicitation timeout
  - DEC-1154 2026-05-17 — Rate limit on elicitation creation: 100/min/task (one task shouldn't elicit 100+ times); over-limit → tool errors with `elicitation_rate_limited`
  - DEC-1155 2026-05-17 — UI integration: CHAT (TASK-CHAT-005) surfaces elicitations as inline prompts in the conversation; outside-CHAT contexts (CLI, API) use polling
  - DEC-1156 2026-05-17 — Idempotency: re-submitting the same `elicitation_id` + `response_payload` returns cached response (avoids double-side-effect on tool resume)
  - DEC-1157 2026-05-17 — Response payload PII-scrubbed: hash16 in memory chain; full response in `mcp_elicitations` (RLS-scoped, 30-day retention)
  - DEC-1158 2026-05-17 — Trace_id end-to-end: elicitation share trace_id with parent task; allows forensic reconstruction
  - DEC-1159 2026-05-17 — Cross-tenant elicitation read DENIED via RLS + explicit check; attempt = 403 + sev-1 audit
  - "DEC-1160 2026-05-17 — Single-choice + multi-choice elicitation_type have fixed schema with `choices: [{value, label}]` array"

build_envelope:
  language: rust 1.81
  service: cyberos/services/mcp/
  new_files:
    - services/mcp/migrations/0012_mcp_elicitations.sql               # pending + completed elicitations
    - services/mcp/src/elicitation/mod.rs                             # orchestrator
    - services/mcp/src/elicitation/request.rs                         # tool-side API to emit elicitation
    - services/mcp/src/elicitation/response.rs                        # caller-side response handler
    - services/mcp/src/elicitation/poll.rs                            # caller polling endpoint
    - services/mcp/src/elicitation/cancel.rs                          # cancellation handler
    - services/mcp/src/elicitation/timeout_job.rs                     # daily timeout sweep
    - services/mcp/src/elicitation/validate.rs                        # JSON Schema response validation
    - services/mcp/src/elicitation/file_upload.rs                     # presigned S3 URL generation
    - services/mcp/src/elicitation/nats_push.rs                       # NATS publish for push transport
    - services/mcp/src/audit/elicitation_events.rs                    # 5 memory row builders
    - services/mcp/src/handlers/elicitation_routes.rs                 # REST routes
    - services/mcp/tests/elicitation_request_response_test.rs
    - services/mcp/tests/elicitation_timeout_test.rs
    - services/mcp/tests/elicitation_cancellation_test.rs
    - services/mcp/tests/elicitation_type_enum_cardinality_test.rs
    - services/mcp/tests/elicitation_schema_validation_test.rs
    - services/mcp/tests/elicitation_re_elicit_after_invalid_test.rs
    - services/mcp/tests/elicitation_confirmation_integration_test.rs
    - services/mcp/tests/elicitation_file_upload_test.rs
    - services/mcp/tests/elicitation_idempotency_test.rs
    - services/mcp/tests/elicitation_cross_tenant_denied_test.rs
    - services/mcp/tests/elicitation_rate_limit_test.rs
    - services/mcp/tests/elicitation_audit_emission_test.rs

  modified_files:
    - services/mcp/src/server_registry.rs                              # add supports_elicitation field
    - services/mcp/src/lib.rs                                          # mount elicitation routes
    - services/mcp/src/gating/elicit.rs                                # de-stub the elicit mode placeholder from TASK-MCP-006
    - services/mcp/src/tasks/mod.rs                                    # task ctx exposes elicit() API

  allowed_tools:
    - file_read: services/mcp/**
    - file_write: services/mcp/{src,tests,migrations}/**
    - bash: cd services/mcp && cargo test elicitation

  disallowed_tools:
    - allow elicitation from sync tools at slice 3 (per DEC-1147)
    - skip schema validation on response (per DEC-1142 + DEC-1150)
    - bypass rate limit on elicitation creation (per DEC-1154)
    - persist file uploads cross-tenant (S3 keys scoped per tenant)
    - allow re-elicitation > 3 retries on same prompt (per DEC-1150)

effort_hours: 6
subtasks:
  - "0.4h: 0012_mcp_elicitations.sql + RLS + closed enums"
  - "0.4h: elicitation/mod.rs + closed-enum types"
  - "0.5h: elicitation/request.rs — tool-side API (TaskCtx::elicit)"
  - "0.5h: elicitation/response.rs — caller response handler"
  - "0.3h: elicitation/poll.rs — caller polling"
  - "0.3h: elicitation/cancel.rs"
  - "0.3h: elicitation/timeout_job.rs"
  - "0.4h: elicitation/validate.rs — JSON Schema validate with json-schema-validator crate"
  - "0.4h: elicitation/file_upload.rs — presigned S3 URL"
  - "0.3h: elicitation/nats_push.rs"
  - "0.3h: audit/elicitation_events.rs (5 builders)"
  - "0.3h: handlers/elicitation_routes.rs"
  - "1.2h: tests — 12 test files covering all types + timeout + cancel + schema + re-elicit + confirmation + file + idempotency + cross-tenant + rate-limit + audit"
  - "0.3h: de-stub TASK-MCP-006 gating/elicit.rs placeholder"
  - "0.3h: wire-up server_registry + tasks/mod.rs ctx.elicit() API"

risk_if_skipped: "Without Elicitation, tools requiring missing args at runtime must fail with `missing_arg` errors + caller retries with more info — multi-turn UX nightmare. Destructive tools requiring confirmation fall back to the simpler TASK-MCP-006 `confirm` mode (returns 403 + caller posts confirm token); `elicit` mode is the richer pattern (server explains WHAT it wants to do + WHY) and is more usable. Without DEC-1142's JSON Schema validation, malformed responses crash tool handlers. Without DEC-1147's sync-tool ban, blocking on elicit in a sync handler exhausts request workers. Without DEC-1150's re-elicit retry cap, a buggy tool could elicit forever. Without DEC-1153's file-upload type, file-aware tools have no clean upload primitive. Without DEC-1151's TASK-MCP-006 integration, the `elicit` mode in MCP-006 policy stays as a 503 placeholder forever. The 6h effort lands the interactive primitive that completes the long-running-tool UX story + de-stubs TASK-MCP-006's elicit mode."
---

## §1 — Description (BCP-14 normative)

The MCP service **MUST** ship Elicitation primitive at `services/mcp/src/elicitation/` per MCP 2025-11-25 spec — server-initiated structured prompts mid-tool-execution, response polling + cancellation + timeout, JSON Schema validation, 5 type-specific schemas, TASK-MCP-006 confirmation integration, 5 memory audit kinds.

1. **MUST** define the closed `elicitation_type` enum at migration `0012`: `('string_input','single_choice','multi_choice','confirmation','file_upload')` per DEC-1141. CI cardinality test asserts 5.

2. **MUST** define the closed `elicitation_status` enum: `('pending','responded','expired','cancelled','validation_failed')`. CI cardinality test asserts 5.

3. **MUST** define `mcp_elicitations` table at migration `0012`: `(elicitation_id UUID PRIMARY KEY, tenant_id UUID NOT NULL, task_id UUID, caller_subject_id UUID NOT NULL, tool_id TEXT NOT NULL, elicitation_type elicitation_type NOT NULL, status elicitation_status NOT NULL DEFAULT 'pending', prompt JSONB NOT NULL, response_schema JSONB NOT NULL, response_payload_kms_blob BYTEA, response_payload_sha256 CHAR(64), validation_errors JSONB, created_at TIMESTAMPTZ NOT NULL DEFAULT now(), responded_at TIMESTAMPTZ, timeout_seconds INT NOT NULL CHECK (timeout_seconds BETWEEN 1 AND 1800), expires_at TIMESTAMPTZ NOT NULL, trace_id CHAR(32), retry_count INT NOT NULL DEFAULT 0 CHECK (retry_count BETWEEN 0 AND 3))`. Per-task elicitations linked via task_id (NULL for sync-tool elicitations which are forbidden per DEC-1147 — defensive column).

4. **MUST** enforce RLS with USING and WITH CHECK on the table: `tenant_id = current_setting('auth.tenant_id')::uuid AND caller_subject_id = current_setting('auth.subject_id')::uuid` (per-caller scope; tenant_admin can see all via separate handler).

5. **MUST** expose tool-side API at `services/mcp/src/elicitation/request.rs::TaskCtx::elicit(elicitation_type, prompt, response_schema, timeout_seconds) -> Result<ElicitationResponse, ElicitationError>` per DEC-1140. The API:
    - Validates tool's `supports_elicitation=true` per DEC-1146.
    - Validates `timeout_seconds ≤ 1800` (30 min max) per DEC-1143.
    - Generates UUIDv7 elicitation_id.
    - INSERTs `mcp_elicitations` row with status='pending', `expires_at=now()+timeout_seconds`.
    - Publishes to NATS `tenant.<slug>.mcp.elicitations.<elicitation_id>.requested` per DEC-1145 (push transport).
    - Awaits caller response via Postgres LISTEN/NOTIFY on `elicitation_responded:<elicitation_id>` channel with timeout.
    - On NOTIFY: loads row, returns response per type.
    - On timeout: transitions status='expired', emits `mcp.elicitation_timeout`, returns Err.
    - On cancellation: transitions status='cancelled', emits `mcp.elicitation_cancelled`, returns Err.

6. **MUST** expose caller polling at `GET /v1/mcp/elicitations?task_id=...&status=pending` per DEC-1145. Returns array of pending elicitations for the caller's session. Rate-limit 60/min/caller (1/sec sufficient).

7. **MUST** expose caller response at `POST /v1/mcp/elicitations/{elicitation_id}/respond` with body `{ response_payload }`. Handler:
    - Validates RLS + caller ownership.
    - Validates `response_payload` against `mcp_elicitations.response_schema` via JSON Schema validator per DEC-1142 + DEC-1150.
    - On validation success: KMS-encrypt + persist `response_payload_kms_blob`; transition status='responded'; `NOTIFY elicitation_responded:<elicitation_id>` to wake the tool.
    - On validation failure: increment `retry_count`; if `retry_count > 3` → status='validation_failed' (terminal); else return 422 + validation_errors array (caller re-submits within the same elicitation_id).
    - Emit `mcp.elicitation_responded` (on success) or `mcp.elicitation_validation_failed` (on failure).

8. **MUST** expose cancellation at `POST /v1/mcp/elicitations/{id}/cancel` per DEC-1148. Handler:
    - Transitions status='cancelled'; NOTIFY to wake the tool with cancellation signal.
    - Tool's `elicit()` returns Err(`elicitation_cancelled`); tool MAY decide to fail-task or retry-elicit.

9. **MUST** apply per-type fixed schemas per DEC-1141 + DEC-1152 + DEC-1160:
    - **string_input**: schema `{ type: "object", properties: { value: { type: "string", maxLength: 4096 } }, required: ["value"] }`.
    - **single_choice**: schema `{ type: "object", properties: { value: { enum: [...prompt.choices.map(c => c.value)] } }, required: ["value"] }` (choices declared in prompt).
    - **multi_choice**: schema `{ type: "object", properties: { values: { type: "array", items: { enum: [...] }, uniqueItems: true } }, required: ["values"] }`.
    - **confirmation**: schema `{ type: "object", properties: { confirmed: { type: "boolean" }, reason: { type: "string", maxLength: 512 } }, required: ["confirmed"] }`.
    - **file_upload**: schema `{ type: "object", properties: { s3_key: { type: "string", pattern: "^elicitations/[a-f0-9-]{36}/[a-zA-Z0-9_.-]+$" }, sha256: { type: "string", pattern: "^[a-f0-9]{64}$" }, size_bytes: { type: "integer", minimum: 1, maximum: 104857600 } }, required: ["s3_key", "sha256", "size_bytes"] }`.

10. **MUST** support file_upload elicitation per DEC-1153 via TASK-DOC-001 presigned S3 URLs. Handler at elicit creation for file_upload type:
    - Generates per-elicitation S3 key `elicitations/{elicitation_id}/{filename}`.
    - Generates presigned PUT URL with `expires_at` matching elicitation timeout.
    - Returns URL in elicitation prompt payload (`prompt.upload_url`).
    - Caller uploads; then POSTs `respond` with `{s3_key, sha256, size_bytes}` per the schema.
    - Server verifies S3 object exists + size matches + SHA256 matches the uploaded object's ETag (or computed); on mismatch → 422.

11. **MUST** integrate with TASK-MCP-006 gating per DEC-1151. When TASK-MCP-006 policy says `mode=elicit` for a tool:
    - Gating layer creates a `confirmation` elicitation with prompt describing the action.
    - Tool execution blocks until elicitation resolved.
    - `confirmed=true` → tool proceeds; `confirmed=false` → tool fails with `user_rejected`.
    - De-stubs TASK-MCP-006's 503 placeholder (`elicitation_not_yet_supported`); TASK-MCP-006 `services/mcp/src/gating/elicit.rs` modified here.

12. **MUST** enforce 5-min default timeout + 30-min max per DEC-1143. Tool API enforces upper bound at create time. Daily timeout job per DEC-1144 sweeps expired pending elicitations.

13. **MUST** rate-limit elicitation creation at 100/min/task per DEC-1154. Excess returns Err(`elicitation_rate_limited`) to the tool.

14. **MUST** be idempotent on response submission per DEC-1156. Re-POSTing `respond` with same `elicitation_id` + `response_payload_sha256` returns cached response (200 + existing state); avoids double-side-effect on tool resume.

15. **MUST** apply re-elicit retry cap per DEC-1150. The `retry_count` column tracks retries; CHECK constraint enforces ≤ 3. Tool can call `ctx.elicit()` after a validation_failed elicitation but only 3 times for the same logical prompt (tool determines uniqueness; framework tracks count per elicitation_id chain).

16. **MUST** scope elicitation to caller_subject_id per DEC-1159. Cross-caller-subject GET/POST returns 403 + `cross_caller_access_denied` + sev-1 audit.

17. **MUST** emit 5 memory audit row kinds per DEC-1149 (task-audit skill rule 6):
    - `mcp.elicitation_requested` (sev-3 — informational; can be high-volume)
    - `mcp.elicitation_responded` (sev-3)
    - `mcp.elicitation_timeout` (sev-3)
    - `mcp.elicitation_cancelled` (sev-3)
    - `mcp.elicitation_validation_failed` (sev-2 — indicates bad tool design or compromised client)

18. **MUST** preserve trace_id end-to-end per DEC-1158. Parent task trace_id propagates to elicitation row + caller poll responses + audit rows + NATS publishes.

19. **MUST** PII-scrub per DEC-1157 + task-audit skill rule 18: `response_payload_sha256` only in memory chain; raw payload in `mcp_elicitations.response_payload_kms_blob` (RLS-scoped, 30-day retention post-completion).

20. **MUST** auto-clean responded elicitations at T+30 days post-completion. Daily job UPDATE `response_payload_kms_blob=NULL WHERE status IN ('responded','expired','cancelled','validation_failed') AND COALESCE(responded_at, expires_at) < now() - interval '30 days'`. Metadata retained for forensic.

21. **MUST NOT** allow elicitation from sync tools at slice 3 per DEC-1147. Tool's elicit() API checks calling context; if not within a task (long_running tool execution), returns Err(`sync_elicit_forbidden_slice_3`).

22. **MUST NOT** allow cross-tenant elicitation access per DEC-1159. RLS + explicit check both enforce.

23. **MUST NOT** persist responses past 30 days post-completion per DEC-1157 + §1 #20. Pruning auto-runs daily.

24. **SHOULD** observe per-tool elicitation latency (request → response) via OTel histogram `mcp_elicitation_duration_seconds{tool_id, type}` for UX visibility.

25. **SHOULD** support NATS push transport in addition to polling per DEC-1145; clients with NATS access get sub-second wake-up; polling clients see 1-sec lag.

---

## §2 — Why this design (rationale for humans)

**Why server-initiated prompts (§1 #5, DEC-1140)?** Many tools need a clarification or confirmation that depends on partial work (e.g., "found 3 candidate documents — which one?"). Without elicitation, tool errors with missing-arg + caller retries from scratch; with elicitation, tool pauses + asks + resumes — much better UX.

**Why JSON Schema validation (§1 #7, DEC-1142)?** Free-form responses crash tool handlers (type errors, missing keys). JSON Schema declarative validation = tool implementer specifies the shape once; framework rejects malformed responses without tool code seeing them.

**Why per-type fixed schemas (§1 #9, DEC-1141 + DEC-1152 + DEC-1160)?** Common patterns (single_choice, confirmation) shouldn't require tool implementers to hand-craft JSON schemas every time. Fixed type-specific schemas = consistent UX across tools + less boilerplate.

**Why ban elicitation in sync tools at slice 3 (§1 #21, DEC-1147)?** Sync tools are 30s-bounded by gateway; blocking on a 5-min elicitation = guaranteed timeout. Long-running tasks (TASK-MCP-007) are the natural home for elicitation; banning sync usage prevents footgun. Slice 4 may revisit with shorter sync-elicit pattern.

**Why re-elicit retry cap of 3 (§1 #15, DEC-1150)?** Bug-defense — a tool that elicits with a too-strict schema could elicit-fail-elicit-fail forever if the caller can't satisfy it. 3 retries = generous (covers genuine typos); cap prevents infinite loops.

**Why polling required + NATS push optional (§1 #6, DEC-1145)?** Mirrors TASK-MCP-007 pattern: polling universal, push for high-volume clients. CHAT consumes elicitations via NATS push for sub-second UX; CLI tools poll.

**Why file_upload via presigned S3 (§1 #10, DEC-1153)?** Large file uploads through the MCP gateway bloat the request path + tie up memory. Presigned S3 = direct client-to-S3 upload; gateway only sees the reference. Standard pattern for file upload at scale.

**Why TASK-MCP-006 confirm-mode satisfied by confirmation elicitation (§1 #11, DEC-1151)?** Both patterns ask "user, please confirm X". Unifying them avoids the gating-layer reinventing prompt UX. The MCP-006 `elicit` mode (previously placeholder per TASK-MCP-006 DEC-1055) de-stubs by routing to this FR's confirmation elicitation.

**Why per-task elicitation scoping (§1 #3, schema task_id column)?** Elicitations belong to a tool invocation. Listing pending elicitations by task_id (or session_id) gives the UI a clean per-task action panel. Cross-task elicitations would require additional UX state.

**Why 30-min max timeout (§1 #12, DEC-1143)?** Beyond 30 min, the user has likely closed the tab / context-switched / forgotten. Timeout-and-resume-with-error is the right semantic; longer timeouts waste worker capacity (worker holding the task slot).

**Why audit at sev-3 for routine events (§1 #17)?** Elicitation traffic is high-volume in interactive scenarios. Sev-3 = informational, sampled at the OBS layer per TASK-OBS-006. Sev-2 reserved for validation_failed (forensic signal for bad client behaviour or tool bugs).

**Why presigned URL ttl matches elicitation timeout (§1 #10)?** Two timeouts (URL vs elicitation) creates failure-mode confusion. Same TTL = single failure boundary.

---

## §3 — API contract

### 3.1 Postgres schema

```sql
-- 0012_mcp_elicitations.sql
CREATE TYPE elicitation_type AS ENUM ('string_input','single_choice','multi_choice','confirmation','file_upload');
CREATE TYPE elicitation_status AS ENUM ('pending','responded','expired','cancelled','validation_failed');

CREATE TABLE mcp_elicitations (
  elicitation_id UUID PRIMARY KEY,
  tenant_id UUID NOT NULL,
  task_id UUID,                                       -- nullable for defensive logging (sync tools forbidden)
  caller_subject_id UUID NOT NULL,
  tool_id TEXT NOT NULL,
  elicitation_type elicitation_type NOT NULL,
  status elicitation_status NOT NULL DEFAULT 'pending',
  prompt JSONB NOT NULL,
  response_schema JSONB NOT NULL,
  response_payload_kms_blob BYTEA,
  response_payload_sha256 CHAR(64),
  validation_errors JSONB,
  retry_count INT NOT NULL DEFAULT 0 CHECK (retry_count BETWEEN 0 AND 3),
  timeout_seconds INT NOT NULL CHECK (timeout_seconds BETWEEN 1 AND 1800),
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  responded_at TIMESTAMPTZ,
  expires_at TIMESTAMPTZ NOT NULL,
  trace_id CHAR(32)
);
CREATE INDEX idx_elicit_caller_pending
  ON mcp_elicitations(caller_subject_id, status, created_at DESC)
  WHERE status = 'pending';
CREATE INDEX idx_elicit_task ON mcp_elicitations(task_id) WHERE task_id IS NOT NULL;
CREATE INDEX idx_elicit_expiry ON mcp_elicitations(expires_at) WHERE status = 'pending';
ALTER TABLE mcp_elicitations ENABLE ROW LEVEL SECURITY;
CREATE POLICY mcp_elicitations_rls ON mcp_elicitations
  USING (tenant_id = current_setting('auth.tenant_id')::uuid
         AND caller_subject_id = current_setting('auth.subject_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid
              AND caller_subject_id = current_setting('auth.subject_id')::uuid);
REVOKE DELETE ON mcp_elicitations FROM cyberos_app;
GRANT UPDATE (status, response_payload_kms_blob, response_payload_sha256, validation_errors, retry_count, responded_at)
  ON mcp_elicitations TO cyberos_app;
GRANT DELETE ON mcp_elicitations TO cyberos_pruner;
```

### 3.2 Rust types

```rust
// services/mcp/src/elicitation/mod.rs
#[derive(Copy, Clone, Eq, PartialEq, Debug, sqlx::Type, serde::Serialize)]
#[sqlx(type_name = "elicitation_type", rename_all = "snake_case")]
pub enum ElicitationType { StringInput, SingleChoice, MultiChoice, Confirmation, FileUpload }

#[derive(Copy, Clone, Eq, PartialEq, Debug, sqlx::Type, serde::Serialize)]
#[sqlx(type_name = "elicitation_status", rename_all = "snake_case")]
pub enum ElicitationStatus { Pending, Responded, Expired, Cancelled, ValidationFailed }

#[derive(Debug, serde::Serialize)]
pub struct ElicitationRequest {
    pub elicitation_id: Uuid,
    pub task_id: Option<Uuid>,
    pub tool_id: String,
    pub elicitation_type: ElicitationType,
    pub prompt: serde_json::Value,
    pub response_schema: serde_json::Value,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, serde::Deserialize)]
pub struct ElicitationRespondReq { pub response_payload: serde_json::Value }
```

### 3.3 REST endpoints

```text
GET    /v1/mcp/elicitations?task_id=...&status=pending    (caller-owned polling)
POST   /v1/mcp/elicitations/{id}/respond                   (caller submits response)
POST   /v1/mcp/elicitations/{id}/cancel                    (caller cancels)
```

### 3.4 Tool-side API

```rust
// available within TaskCtx (TASK-MCP-007 task execution context)
async fn elicit<R: DeserializeOwned>(
    &self,
    elicitation_type: ElicitationType,
    prompt: serde_json::Value,
    response_schema: serde_json::Value,
    timeout: Duration,
) -> Result<R, ElicitationError>;
```

---

## §4 — Acceptance criteria

1. **elicitation_type cardinality** — enum = exactly `{string_input, single_choice, multi_choice, confirmation, file_upload}`.
2. **elicitation_status cardinality** — enum = exactly `{pending, responded, expired, cancelled, validation_failed}`.
3. **Request + response happy path** — tool emits elicitation, caller polls, sees pending, POSTs response, tool resumes with parsed response.
4. **Timeout expires** — elicitation with `timeout=2s`; no response in 3s → status='expired'; tool receives `elicitation_timeout` error; audit row emitted.
5. **Cancellation** — caller cancels pending elicitation → status='cancelled'; tool receives `elicitation_cancelled`.
6. **Schema validation rejection** — response payload missing required field → 422 + validation_errors; retry_count++.
7. **Re-elicit cap 3 retries** — 4th attempt at re-submission for same `elicitation_id` → status='validation_failed' terminal.
8. **Confirmation type happy** — tool emits `confirmation` elicit with prompt "delete X?"; caller responds `{confirmed: true}`; tool proceeds.
9. **File-upload presigned URL** — tool emits `file_upload`; prompt response includes `upload_url` (presigned S3); caller PUTs file + POSTs response with s3_key.
10. **TASK-MCP-006 integration** — TASK-MCP-006 policy `mode=elicit` on destructive tool routes through this FR's confirmation elicit; previous 503 placeholder de-stubbed.
11. **Sync-tool elicit forbidden** — sync tool calling `elicit()` returns Err `sync_elicit_forbidden_slice_3`.
12. **Cross-caller access denied** — caller A polling caller B's elicitation_id → 403 + sev-1 audit.
13. **Cross-tenant access denied** — RLS prevents cross-tenant read; verified via two-tenant test.
14. **Rate limit 100/min/task** — 101st elicit() within 60s on same task → Err `elicitation_rate_limited`.
15. **Idempotent response submission** — re-POST same response → 200 with existing state.
16. **NATS push transport** — subscriber on `tenant.<slug>.mcp.elicitations.<id>.requested` receives push within 100ms of tool-side elicit().
17. **Trace_id preserved** — single trace_id across tool's elicit() span + caller poll response + audit rows + NATS payload.
18. **30-day retention pruning** — responded elicitation > 30d post-completion → `response_payload_kms_blob=NULL` after prune; metadata retained.
19. **PII scrub** — audit row carries `response_payload_sha256` only; raw payload in DB only.
20. **5 memory audit kinds emitted** — full lifecycle covers requested + responded + timeout + cancelled + validation_failed.

---

## §5 — Verification

### 5.1 `elicitation_request_response_test.rs`

```rust
#[tokio::test]
async fn happy_path_request_then_respond() {
    let ctx = TestContext::new().await;
    let task = ctx.spawn_long_running_tool_that_elicits().await;
    let pending = ctx.poll_pending_elicitations_until_nonempty(task).await;
    assert_eq!(pending.len(), 1);
    let eid = pending[0].elicitation_id;

    let r = ctx.respond_to_elicitation(eid, json!({"value": "answer"})).await;
    assert_eq!(r.status(), 200);

    let result = ctx.await_task_completion(task, Duration::from_secs(5)).await;
    assert_eq!(result["received_value"], "answer");
}
```

### 5.2 `elicitation_timeout_test.rs`

```rust
#[tokio::test]
async fn elicitation_expires_after_timeout() {
    let ctx = TestContext::new().await;
    let task = ctx.spawn_tool_that_elicits_with_timeout(2).await;
    tokio::time::sleep(Duration::from_secs(3)).await;
    ctx.run_elicitation_timeout_job().await;

    let row: (String,) = sqlx::query_as("SELECT status::text FROM mcp_elicitations WHERE task_id=$1")
        .bind(task).fetch_one(&ctx.pool).await.unwrap();
    assert_eq!(row.0, "expired");

    let task_result = ctx.poll_task(task).await;
    assert!(task_result.error.is_some());
    assert_eq!(task_result.error.unwrap().message, "elicitation_timeout");
}
```

### 5.3 `elicitation_schema_validation_test.rs`

```rust
#[tokio::test]
async fn missing_required_field_rejected_with_422() {
    let ctx = TestContext::new().await;
    let task = ctx.spawn_tool_that_elicits_string_input().await;
    let pending = ctx.poll_pending_elicitations_until_nonempty(task).await;
    let eid = pending[0].elicitation_id;

    let r = ctx.respond_to_elicitation(eid, json!({})).await;  // missing "value"
    assert_eq!(r.status(), 422);
    let body: serde_json::Value = r.json().await.unwrap();
    assert!(body["validation_errors"].as_array().unwrap().len() > 0);
}
```

### 5.4 `elicitation_re_elicit_after_invalid_test.rs`

```rust
#[tokio::test]
async fn three_retries_then_terminal() {
    let ctx = TestContext::new().await;
    let task = ctx.spawn_tool_that_elicits_string_input().await;
    let eid = ctx.poll_pending_elicitations_until_nonempty(task).await[0].elicitation_id;

    for _ in 0..3 {
        let r = ctx.respond_to_elicitation(eid, json!({})).await;
        assert_eq!(r.status(), 422);
    }
    let r4 = ctx.respond_to_elicitation(eid, json!({})).await;
    assert_eq!(r4.status(), 422);

    let row: (String,) = sqlx::query_as("SELECT status::text FROM mcp_elicitations WHERE elicitation_id=$1")
        .bind(eid).fetch_one(&ctx.pool).await.unwrap();
    assert_eq!(row.0, "validation_failed");
}
```

### 5.5 `elicitation_confirmation_integration_test.rs`

```rust
#[tokio::test]
async fn mcp006_elicit_mode_routes_through_elicitation() {
    let ctx = TestContext::with_policy_mode("elicit").await;
    ctx.register_tool("cyberos.docs.delete", destructive_long_running_annotations()).await;
    let r = ctx.invoke_tool("cyberos.docs.delete", json!({"doc_id": "x"})).await;
    let task_id: Uuid = r.json::<serde_json::Value>().await.unwrap()["task_id"].as_str().unwrap().parse().unwrap();

    let pending = ctx.poll_pending_elicitations_until_nonempty(task_id).await;
    assert_eq!(pending[0].elicitation_type, ElicitationType::Confirmation);

    ctx.respond_to_elicitation(pending[0].elicitation_id, json!({"confirmed": true})).await;
    let result = ctx.await_task_completion(task_id, Duration::from_secs(5)).await;
    assert_eq!(result["deleted"], true);
}
```

### 5.6 `elicitation_cancellation_test.rs`

```rust
#[tokio::test]
async fn cancel_transitions_and_unblocks_tool() {
    let ctx = TestContext::new().await;
    let task = ctx.spawn_tool_that_elicits_string_input().await;
    let eid = ctx.poll_pending_elicitations_until_nonempty(task).await[0].elicitation_id;
    let r = ctx.cancel_elicitation(eid).await;
    assert_eq!(r.status(), 200);
    let task_result = ctx.await_task_completion(task, Duration::from_secs(3)).await;
    assert!(task_result.error.is_some());
    assert_eq!(task_result.error.unwrap().message, "elicitation_cancelled");
}
```

### 5.7 `elicitation_type_enum_cardinality_test.rs`

```rust
#[tokio::test]
async fn elicitation_type_has_5_values() {
    let ctx = TestContext::new().await;
    let labels: Vec<String> = sqlx::query_scalar(
        "SELECT unnest(enum_range(NULL::elicitation_type))::text"
    ).fetch_all(&ctx.pool).await.unwrap();
    let mut labels = labels; labels.sort();
    assert_eq!(labels, vec!["confirmation","file_upload","multi_choice","single_choice","string_input"]);
}
```

### 5.8 `elicitation_cross_tenant_denied_test.rs`

```rust
#[tokio::test]
async fn caller_b_cannot_access_caller_a_elicitation() {
    let ctx = TestContext::with_two_callers().await;
    let task_a = ctx.as_caller("a").spawn_tool_that_elicits_string_input().await;
    let eid_a = ctx.as_caller("a").poll_pending_elicitations_until_nonempty(task_a).await[0].elicitation_id;

    let r = ctx.as_caller("b").respond_to_elicitation(eid_a, json!({"value": "x"})).await;
    assert_eq!(r.status(), 403);

    let audit = ctx.memory_rows().await;
    assert!(audit.iter().any(|r| r.kind == "mcp.elicitation_validation_failed"
        || r.kind == "mcp.cross_caller_access_denied"));  // either security audit
}
```

### 5.9 `elicitation_idempotency_test.rs`

```rust
#[tokio::test]
async fn duplicate_response_idempotent() {
    let ctx = TestContext::new().await;
    let task = ctx.spawn_tool_that_elicits_string_input().await;
    let eid = ctx.poll_pending_elicitations_until_nonempty(task).await[0].elicitation_id;

    let r1 = ctx.respond_to_elicitation(eid, json!({"value": "answer"})).await;
    assert_eq!(r1.status(), 200);
    let r2 = ctx.respond_to_elicitation(eid, json!({"value": "answer"})).await;
    assert_eq!(r2.status(), 200);  // idempotent

    let count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM mcp_elicitations WHERE elicitation_id=$1 AND status='responded'"
    ).bind(eid).fetch_one(&ctx.pool).await.unwrap();
    assert_eq!(count, 1);
}
```

### 5.10 `elicitation_file_upload_test.rs`

```rust
#[tokio::test]
async fn file_upload_via_presigned_url() {
    let ctx = TestContext::with_s3_mock().await;
    let task = ctx.spawn_tool_that_elicits_file_upload().await;
    let pending = ctx.poll_pending_elicitations_until_nonempty(task).await;
    let upload_url = pending[0].prompt["upload_url"].as_str().unwrap();

    let bytes = b"test file contents".to_vec();
    ctx.put_to_presigned(upload_url, &bytes).await;

    let sha256 = sha256_hex(&bytes);
    let s3_key = ctx.extract_s3_key_from_url(upload_url);
    let r = ctx.respond_to_elicitation(pending[0].elicitation_id, json!({
        "s3_key": s3_key, "sha256": sha256, "size_bytes": bytes.len()
    })).await;
    assert_eq!(r.status(), 200);
}
```

---

## §6 — Implementation skeleton

### 6.1 Tool-side elicit API

```rust
// services/mcp/src/elicitation/request.rs
impl TaskCtx {
    pub async fn elicit<R: DeserializeOwned>(
        &self,
        elicit_type: ElicitationType,
        prompt: serde_json::Value,
        schema: serde_json::Value,
        timeout: Duration,
    ) -> Result<R, ElicitationError> {
        // Check sync-tool ban
        if self.task_id.is_none() {
            return Err(ElicitationError::SyncElicitForbiddenSlice3);
        }
        // Rate limit
        if !self.elicit_rate_limit_check().await? { return Err(ElicitationError::RateLimited); }
        // Validate timeout
        if timeout > Duration::from_secs(1800) { return Err(ElicitationError::TimeoutTooLong); }

        let elicit_id = uuid7();
        let expires_at = Utc::now() + chrono::Duration::from_std(timeout)?;
        let trace_id = self.trace_id.clone();

        // INSERT pending row
        sqlx::query(/* ... */)
            .bind(elicit_id).bind(self.tenant_id).bind(self.task_id)
            .bind(self.caller_subject_id).bind(self.tool_id)
            .bind(elicit_type).bind(prompt).bind(schema)
            .bind(timeout.as_secs() as i32).bind(expires_at).bind(&trace_id)
            .execute(&self.pool).await?;

        // NATS push
        self.nats.publish(
            format!("tenant.{}.mcp.elicitations.{}.requested", self.tenant_slug, elicit_id),
            serde_json::to_vec(&ElicitationRequest {
                elicitation_id: elicit_id, task_id: self.task_id, tool_id: self.tool_id.clone(),
                elicitation_type: elicit_type, prompt: prompt.clone(), response_schema: schema.clone(),
                expires_at,
            })?
        ).await;

        emit_audit(&self.ctx, "mcp.elicitation_requested", json!({/*...*/})).await;

        // Await response via LISTEN/NOTIFY
        let mut listener = sqlx::postgres::PgListener::connect_with(&self.pool).await?;
        listener.listen(&format!("elicitation_responded:{elicit_id}")).await?;
        let signal = tokio::time::timeout(timeout, listener.recv()).await;

        match signal {
            Err(_) => {  // timeout
                let _ = sqlx::query("UPDATE mcp_elicitations SET status='expired' WHERE elicitation_id=$1 AND status='pending'")
                    .bind(elicit_id).execute(&self.pool).await;
                emit_audit(&self.ctx, "mcp.elicitation_timeout", json!({/*...*/})).await;
                Err(ElicitationError::Timeout)
            }
            Ok(Ok(_)) => {
                let row: (String, Option<Vec<u8>>) = sqlx::query_as(
                    "SELECT status::text, response_payload_kms_blob FROM mcp_elicitations WHERE elicitation_id=$1"
                ).bind(elicit_id).fetch_one(&self.pool).await?;
                match row.0.as_str() {
                    "responded" => {
                        let decrypted = kms_decrypt(&row.1.unwrap()).await?;
                        let parsed: R = serde_json::from_slice(&decrypted)?;
                        Ok(parsed)
                    }
                    "cancelled" => Err(ElicitationError::Cancelled),
                    "validation_failed" => Err(ElicitationError::ValidationFailed),
                    _ => Err(ElicitationError::UnexpectedState(row.0)),
                }
            }
            Ok(Err(e)) => Err(ElicitationError::ListenerError(e.to_string())),
        }
    }
}
```

### 6.2 Response handler

```rust
pub async fn respond(ctx: &AppCtx, elicit_id: Uuid, req: ElicitationRespondReq) -> Result<Response, RespondError> {
    let elicit_row = ctx.repo.elicitations.find_pending(elicit_id).await?
        .ok_or(RespondError::NotFound)?;

    // RLS already verified caller; explicit re-check
    if elicit_row.caller_subject_id != ctx.caller_subject_id {
        emit_audit(ctx, "mcp.cross_caller_access_denied", json!({/*...*/})).await;
        return Err(RespondError::Forbidden);
    }

    // Schema validation
    let validator = jsonschema::JSONSchema::compile(&elicit_row.response_schema)?;
    if let Err(errs) = validator.validate(&req.response_payload) {
        let errors_json: Vec<serde_json::Value> = errs.map(|e| json!({"path": e.instance_path.to_string(), "msg": e.to_string()})).collect();
        let next_retry = elicit_row.retry_count + 1;
        if next_retry > 3 {
            sqlx::query("UPDATE mcp_elicitations SET status='validation_failed', validation_errors=$1, retry_count=$2 WHERE elicitation_id=$3")
                .bind(serde_json::Value::Array(errors_json.clone())).bind(next_retry).bind(elicit_id).execute(&ctx.pool).await?;
            emit_audit(ctx, "mcp.elicitation_validation_failed", json!({/*...*/})).await;
            ctx.notify(&format!("elicitation_responded:{elicit_id}")).await;
        } else {
            sqlx::query("UPDATE mcp_elicitations SET validation_errors=$1, retry_count=$2 WHERE elicitation_id=$3")
                .bind(serde_json::Value::Array(errors_json.clone())).bind(next_retry).bind(elicit_id).execute(&ctx.pool).await?;
        }
        return Ok(Response::status(422).json(json!({"validation_errors": errors_json})));
    }

    // Persist response
    let bytes = serde_json::to_vec(&req.response_payload)?;
    let sha256 = sha256_hex(&bytes);
    let encrypted = kms_encrypt(&bytes).await?;
    sqlx::query("UPDATE mcp_elicitations SET status='responded', response_payload_kms_blob=$1,
                 response_payload_sha256=$2, responded_at=now() WHERE elicitation_id=$3 AND status='pending'")
        .bind(encrypted.blob).bind(&sha256).bind(elicit_id).execute(&ctx.pool).await?;
    ctx.notify(&format!("elicitation_responded:{elicit_id}")).await;
    emit_audit(ctx, "mcp.elicitation_responded", json!({/*...*/})).await;

    Ok(Response::status(200))
}
```

---

## §7 — Dependencies

**Upstream (depends_on):**
- **TASK-MCP-001** spec compliance — elicitation defined per MCP 2025-11-25 spec.
- **TASK-MCP-004** OAuth 2.1 PKCE — JWT bearer auth for poll/respond/cancel.

**Cross-module (related_tasks):**
- **TASK-MCP-006** Gating — confirmation elicitation satisfies MCP-006 `mode=elicit` policy; de-stubs MCP-006's 503 placeholder.
- **TASK-MCP-007** Tasks primitive — elicit() is exposed on TaskCtx; only callable from within a long-running task.
- **TASK-AUTH-004** JWT validate — caller_subject_id verification.
- **TASK-AI-003** memory audit-row bridge — 5 new kinds.
- **TASK-MEMORY-111** PII scrubbing — response payload SHA only.
- **TASK-CHAT-005** CHAT — surfaces elicitations inline in conversation; NATS push consumer.
- **TASK-OBS-005** Trace correlation — trace_id end-to-end.
- **TASK-OBS-007** Auto-runbook — sev-2 validation_failed alerts.

**Downstream (blocks):** None at this slice.

---

## §8 — Example payloads

### 8.1 Elicitation request (NATS payload + caller poll response)

```json
{
  "elicitation_id": "0190f7c0-8b3c-7a4f-aaaa-000000000001",
  "task_id": "0190f7c0-8b3c-7a4f-bbbb-000000000042",
  "tool_id": "cyberos.kb.bulk_delete",
  "elicitation_type": "confirmation",
  "prompt": {
    "title": "Confirm bulk delete",
    "description": "About to delete 247 documents matching 'archived'. This cannot be undone.",
    "action_summary": "Delete 247 archived documents"
  },
  "response_schema": {
    "type": "object",
    "properties": {
      "confirmed": { "type": "boolean" },
      "reason": { "type": "string", "maxLength": 512 }
    },
    "required": ["confirmed"]
  },
  "expires_at": "2026-05-17T09:19:32.847Z"
}
```

### 8.2 Caller response

```json
{
  "response_payload": {
    "confirmed": true,
    "reason": "Verified with team; archived docs cleanup approved"
  }
}
```

### 8.3 `mcp.elicitation_requested` memory row

```json
{
  "kind": "mcp.elicitation_requested",
  "severity": 3,
  "tenant_id": "8a2f...",
  "actor_id": "system.mcp.elicitation",
  "trace_id": "0af7651916cd43dd8448eb211c80319c",
  "occurred_at": "2026-05-17T09:14:32.847Z",
  "payload": {
    "elicitation_id": "0190f7c0-...",
    "task_id": "0190f7c0-...",
    "caller_subject_id": "7c4e...",
    "tool_id": "cyberos.kb.bulk_delete",
    "elicitation_type": "confirmation",
    "timeout_seconds": 300
  }
}
```

### 8.4 File-upload elicitation prompt

```json
{
  "title": "Upload signed contract",
  "description": "Please upload the signed PDF (max 100 MiB)",
  "upload_url": "https://s3.amazonaws.com/cyberos-elicit/elicitations/0190.../contract.pdf?X-Amz-Signature=...",
  "expires_at": "2026-05-17T09:19:32.847Z"
}
```

---

## §9 — Open questions

All resolved for slice 3. Deferred:

- **Deferred:** Sync-tool short-elicit pattern (< 10s timeout, blocking the sync response) — slice 4.
- **Deferred:** Elicitation chaining (response to one elicit triggers another) — slice 4.
- **Deferred:** Voice/audio elicitation type — slice 4.
- **Deferred:** Webhook-delivered elicitations (vs polling/push) — slice 4.
- **Deferred:** Multi-recipient elicitation (any-of-N teammates can answer) — slice 4.
- **Deferred:** Elicitation templates (named pre-configured prompts) — slice 4.
- **Deferred:** Per-tool elicitation rate limit override — slice 4.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Elicitation timeout | NOTIFY recv timeout | status='expired'; tool gets `Timeout` Err | Tool decides retry-or-fail; daily sweep ensures status update if NOTIFY missed |
| Response schema validation fails | jsonschema validate | 422 + validation_errors; retry_count++; capped at 3 | Caller fixes payload + re-submits |
| 4th retry on validation_failed | retry cap check | status='validation_failed' terminal; tool gets `ValidationFailed` Err | Tool decides retry-or-fail |
| Cancellation | caller POST cancel | status='cancelled'; tool gets `Cancelled` Err | Tool decides response |
| Cross-caller access | RLS + explicit caller_subject_id check | 403 + sev-1 audit | Investigate compromised JWT |
| Cross-tenant access | RLS rejects | 0 rows; appears as 404 | Inherent |
| Sync tool calls elicit() | task_id=None check | Err `sync_elicit_forbidden_slice_3` | Tool ported to long_running pattern |
| Rate limit hit (100/min/task) | counter | Err `elicitation_rate_limited` | Tool reduces elicit volume; possibly batch prompts |
| KMS unavailable for response decrypt | KMS timeout | Err `kms_unavailable`; tool gets error | AWS KMS recovery |
| Tool process crashes during NOTIFY wait | listener dies | Tool restarted; new instance picks up task; elicitation status still 'pending' or 'responded' | TASK-MCP-007 task resume covers; new tool instance polls elicitation status |
| Duplicate response submission | UPDATE WHERE status='pending' returns 0 rows on 2nd | 200 OK with existing state (idempotent) | Inherent |
| Presigned S3 URL expired before upload | S3 returns 403 on upload | Caller sees S3 error; elicitation_timeout eventually fires | Caller restarts upload; tool re-elicits |
| File upload exceeds 100 MiB | S3 PUT size check | S3 rejects; caller sees error | Caller compresses or splits |
| File SHA256 mismatch (corruption) | server compares | 422 + `sha256_mismatch` | Caller re-uploads |
| Tool elicits more than 3 retries same logical prompt | retry_count check | Returns `ValidationFailed` after 3rd terminal | Tool changes prompt or schema |
| Tenant_admin viewing elicitations across callers | separate handler with admin role | List view shows all; per-row audit on access | OK with audit visibility |
| Prompt JSONB > 1 MiB (huge schema) | size check | 400 + `prompt_too_large` | Tool slims prompt |
| Postgres LISTEN/NOTIFY drops (connection reset) | listener error | Tool restarts listener; checks DB for status | Inherent reconnect logic |
| Pending elicitation orphaned (task crashed without resolving) | daily expiry sweep | status='expired' at timeout | Inherent |
| Multiple workers respond to same NOTIFY | first writer wins UPDATE | Other workers see status≠'pending'; no-op | Inherent CAS |
| Elicitation_id collision (UUIDv7) | primary key constraint | INSERT fails; retry with new ID | Astronomical odds |
| Caller polls deleted task's elicitations | task_id check | Returns 404 | Inherent |

---

## §11 — Implementation notes

**§11.1** JSON Schema validator: `jsonschema` crate; compile schemas once + cache; instances reused per elicitation.

**§11.2** LISTEN/NOTIFY connection requires a dedicated pool connection (not from main pool — long-lived). The `tasks/worker_pool.rs` pre-allocates a notify-listener connection per worker.

**§11.3** NATS push subject pattern: `tenant.<slug>.mcp.elicitations.<id>.<event>` where event in {requested, responded, expired, cancelled}.

**§11.4** Presigned S3 URL TTL matches elicitation timeout exactly; TASK-DOC-001 provides the presigning helper.

**§11.5** Cross-caller audit uses a distinct kind (`mcp.cross_caller_access_denied`) outside the 5-kind core list per task-audit skill §8.1d (security signal needs distinct kind).

**§11.6** UUIDv7 generation: same `services/ten/src/residency/uuid_gen.rs` per TASK-TEN-103.

**§11.7** Rate-limit counter: Redis sliding-window per task_id; TASK-MCP-007 ctx exposes the limiter.

**§11.8** Prompt and response_schema stored as JSONB for flexibility; future indexed queries on prompt content (slice 4) trivial.

**§11.9** The 30-day pruning happens via daily `cyberos_pruner` job (same role as TASK-TEN-003 + TASK-MCP-007 patterns).

**§11.10** Elicitation status transitions are LINEAR: `pending → {responded | expired | cancelled | validation_failed}`. No reverse transitions. Tool's retry creates NEW elicitation row, not status reversal on existing.

**§11.11** Confirmation elicitation has a default 5-min timeout (matching TASK-MCP-006 confirm-token TTL for consistency); tools can override.

**§11.12** Per-type prompt JSONB schemas are described in implementer-docs (not normative spec); tools build prompts per type-specific UX guidelines.

**§11.13** The `task_id IS NULL` defensive column allows future sync-tool elicitation without schema migration; slice 3 enforces non-null via runtime check, not constraint.

**§11.14** Caller polling query uses `idx_elicit_caller_pending` index; sub-ms response.

**§11.15** NATS push payload is full ElicitationRequest struct; clients can react immediately without polling.

**§11.16** Cross-caller audit is sev-1 but uses a kind outside the 5-core list — operators see in security dashboards alongside TASK-TEN-103 cross-residency events.

**§11.17** Tool implementer responsibility: validate schema is reasonable (not too strict, not too permissive); poor schema = bad UX or security hole.

**§11.18** The elicit() API is awaitable inside async tool implementations; non-blocking from worker pool perspective (semaphore released? no — worker holds slot while waiting for human response, which is correct given task-bounded concurrency model).

**§11.19** Caller poll endpoint scope: `caller_subject_id = jwt.subject_id` (own elicitations only); tenant_admin has separate list endpoint at slice 4.

**§11.20** Idempotency on response submission is per `(elicitation_id, response_payload_sha256)` — different payload → different submission, not idempotent.

---

*End of TASK-MCP-008 spec.*
