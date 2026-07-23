---
id: TASK-MCP-007
title: "MCP Tasks primitive — long-running tool calls with status polling + resume-on-reconnect + cancellation + per-task memory audit chain"
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
eu_ai_act_risk_class: not_ai
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-17T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: mcp
priority: p0
status: implementing
verify: T
phase: P0
milestone: P0 · slice 3
slice: 3
owner: Stephen Cheng (CTO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-MCP-001, TASK-MCP-002, TASK-MCP-004, TASK-MCP-005, TASK-MCP-006, TASK-MCP-008, TASK-AUTH-004, TASK-AI-003, TASK-MEMORY-111, TASK-OBS-005, TASK-OBS-007]
depends_on: [TASK-MCP-001, TASK-MCP-004]
blocks: []

source_pages:
  - website/docs/modules/mcp.html#tasks
  - https://modelcontextprotocol.io/specification/2025-11-25/server/tools#long-running
  # Prefer header (respond-async)
  - https://datatracker.ietf.org/doc/html/rfc7240

source_decisions:
  - DEC-1100 2026-05-17 — Tasks primitive per MCP 2025-11-25 spec: long-running tool calls return `task_id` immediately + status polling endpoint + final-result-on-completion; alternative to synchronous tool-call response for work > 5s wall-clock
  - DEC-1101 2026-05-17 — Closed enum `task_status` = {pending, running, completed, failed, cancelled, expired}; CI cardinality test asserts 6
  - DEC-1102 2026-05-17 — Per-tool registration declares `long_running: bool` annotation; tools with `long_running=true` route through Tasks primitive automatically; sync tools call directly per TASK-MCP-001
  - DEC-1103 2026-05-17 — Task TTL default 24h; per-tool override via `task_ttl_seconds` registration field; max 7 days enforced at gateway
  - DEC-1104 2026-05-17 — Task ID format: UUIDv7 (sortable + collision-free per TASK-TEN-103 DEC-927); prefixed `task_` for human readability
  - DEC-1105 2026-05-17 — Status polling endpoint `GET /v1/mcp/tasks/{task_id}`; rate-limit 60/min/task (1/sec sufficient for UX) per task-audit skill §8.2c absence-claim derivative — no busy-poll allowed
  - DEC-1106 2026-05-17 — Resume on reconnect: task_id is opaque + persistent in Postgres `mcp_tasks` table; client can poll from any session/device as long as JWT matches `caller_subject_id`
  - DEC-1107 2026-05-17 — Cancellation endpoint `POST /v1/mcp/tasks/{task_id}/cancel`; transitions task to `cancelling` (terminal-pending) then `cancelled`; per-tool registration declares whether cancellation is supported
  - DEC-1108 2026-05-17 — Tool execution runs in a dedicated Tokio worker pool with bounded concurrency per module (TASK-MCP-002 registration declares `max_concurrent_tasks`); over-limit tasks queue with `status=pending`
  - DEC-1109 2026-05-17 — Per-task memory audit chain: each task emits `mcp.task_started`, `mcp.task_progress` (informational sampled), `mcp.task_completed | mcp.task_failed | mcp.task_cancelled | mcp.task_expired`; all rows carry `task_id` for forensic reconstruction
  - DEC-1110 2026-05-17 — Progress reporting via NATS subject `tenant.<slug>.mcp.tasks.<task_id>.progress`; clients MAY subscribe for push updates instead of polling; PUSH is OPTIONAL, POLL is REQUIRED (clients without NATS access still work)
  - DEC-1111 2026-05-17 — TASK-MCP-006 gating integration: long-running tool with destructive_hint=true requires confirmation at task START; subsequent polls do NOT re-confirm; cancellation does not require confirmation
  - DEC-1112 2026-05-17 — Final result size cap 10 MiB; over-cap → status=failed + reason='result_too_large'; tools requiring larger outputs MUST stream via TASK-DOC-001 S3 + return reference URL
  - "DEC-1113 2026-05-17 — Task expiry semantics: `expires_at = created_at + ttl_seconds`; expired tasks transition to `status=expired` + result_payload pruned to save storage (only metadata retained)"
  - DEC-1114 2026-05-17 — Closed enum `task_progress_unit` = {percent, items, bytes, none}; CI cardinality test asserts 4
  - DEC-1115 2026-05-17 — Per-tenant rate limit on task creation: 100 tasks/min/tenant; over-limit returns 429 + Retry-After (tasks are expensive; abuse vector)
  - DEC-1116 2026-05-17 — Workers persist intermediate state to `mcp_task_checkpoints` table (optional); cancellation honours checkpoint boundary; restart-after-crash resumes from last checkpoint
  - "DEC-1117 2026-05-17 — Task result delivery: client polls until `status=completed`; final response body includes `result: <tool_specific_json>`; if oversized, `result_url: <s3_url>` is returned instead"
  - "DEC-1118 2026-05-17 — Per-task error: `error: { code, message, details? }` populated when `status=failed`; error.code matches MCP JSON-RPC error code conventions"
  - DEC-1119 2026-05-17 — Task list endpoint `GET /v1/mcp/tasks?status=running&tool_id=...` for tenant_admin view (caller subject + filter); paginated 50/page
  - DEC-1120 2026-05-17 — Worker pool isolation: per-module pools prevent one slow module from starving another; TASK-MCP-002 registration declares pool size
  - DEC-1121 2026-05-17 — Idempotency on task creation: client-supplied `idempotency_key` in `tools/call`; duplicate within 24h returns existing task_id instead of creating new
  - DEC-1122 2026-05-17 — Cancellation race: client may cancel a task between worker checkpoint + completion; worker MUST honour `is_cancelled()` check at every checkpoint boundary
  - DEC-1123 2026-05-17 — Per-task trace_id is the original tools/call trace_id; preserved across all status polls + audit rows
  - DEC-1124 2026-05-17 — memory audit kinds: mcp.task_started, mcp.task_progress, mcp.task_completed, mcp.task_failed, mcp.task_cancelled, mcp.task_expired, mcp.task_resumed_after_reconnect, mcp.task_checkpoint_persisted
  - DEC-1125 2026-05-17 — PII scrub via TASK-MEMORY-111: task input + result payload hashes only in chain; raw payloads in `mcp_tasks` table (RLS-scoped, 30-day retention post-completion)

language: rust 1.81
service: cyberos/services/mcp/
new_files:
  # task registry
  - services/mcp/migrations/0009_mcp_tasks.sql
  # intermediate state
  - services/mcp/migrations/0010_mcp_task_checkpoints.sql
  # progress event log
  - services/mcp/migrations/0011_mcp_task_progress_events.sql
  # tasks orchestrator
  - services/mcp/src/tasks/mod.rs
  # task creation (from tools/call with long_running)
  - services/mcp/src/tasks/create.rs
  # status poll handler
  - services/mcp/src/tasks/status.rs
  # cancellation handler
  - services/mcp/src/tasks/cancel.rs
  # list handler
  - services/mcp/src/tasks/list.rs
  # per-module pool + bounded concurrency
  - services/mcp/src/tasks/worker_pool.rs
  # save/restore intermediate state
  - services/mcp/src/tasks/checkpoint.rs
  # daily TTL expiry sweep
  - services/mcp/src/tasks/expiry_job.rs
  # progress NATS publish
  - services/mcp/src/tasks/progress.rs
  # task-creation idempotency cache
  - services/mcp/src/tasks/idempotency.rs
  # 8 memory row builders
  - services/mcp/src/audit/task_events.rs
  - services/mcp/tests/task_create_async_test.rs
  - services/mcp/tests/task_status_poll_test.rs
  - services/mcp/tests/task_resume_after_reconnect_test.rs
  - services/mcp/tests/task_cancellation_test.rs
  - services/mcp/tests/task_cancellation_race_test.rs
  - services/mcp/tests/task_ttl_expiry_test.rs
  - services/mcp/tests/task_status_enum_cardinality_test.rs
  - services/mcp/tests/task_progress_unit_enum_cardinality_test.rs
  - services/mcp/tests/task_oversized_result_test.rs
  - services/mcp/tests/task_gating_integration_test.rs
  - services/mcp/tests/task_idempotency_test.rs
  - services/mcp/tests/task_worker_pool_isolation_test.rs
  - services/mcp/tests/task_checkpoint_resume_test.rs
  - services/mcp/tests/task_rate_limit_test.rs
  - services/mcp/tests/task_audit_emission_test.rs

modified_files:
  # branch sync vs task based on long_running annotation
  - services/mcp/src/handlers/tools_call.rs
  # add long_running, max_concurrent_tasks, task_ttl_seconds fields
  - services/mcp/src/server_registry.rs
  # mount task routes
  - services/mcp/src/lib.rs

allowed_tools:
  - file_read: services/mcp/**
  - file_write: services/mcp/{src,tests,migrations}/**
  - bash: cd services/mcp && cargo test tasks

disallowed_tools:
  - block tools/call response on task completion (long_running tools MUST async-return)
  - allow cancellation without per-tool support flag (per DEC-1107)
  - return final result > 10 MiB inline (per DEC-1112)
  - reuse expired task_id (UUIDv7 collision-free)
  - allow worker pool to starve cross-module (per DEC-1108 + DEC-1120)
  - skip checkpoint cancellation check (per DEC-1122)

effort_hours: 10
subtasks:
  - "0.5h: 0009_mcp_tasks.sql + 0010_mcp_task_checkpoints.sql + 0011_mcp_task_progress_events.sql"
  - "0.5h: tasks/mod.rs + closed-enum + idempotency table"
  - "0.6h: tasks/create.rs — async task creation + worker dispatch + idempotency"
  - "0.5h: tasks/status.rs — status poll handler + result delivery"
  - "0.4h: tasks/cancel.rs — cancellation + checkpoint-aware"
  - "0.4h: tasks/list.rs — tenant_admin list view"
  - "0.7h: tasks/worker_pool.rs — per-module bounded-concurrency Tokio pool"
  - "0.5h: tasks/checkpoint.rs — save/restore intermediate state"
  - "0.4h: tasks/expiry_job.rs — daily TTL sweep + result pruning"
  - "0.4h: tasks/progress.rs — NATS publish"
  - "0.4h: tasks/idempotency.rs — 24h cache"
  - "0.4h: audit/task_events.rs — 8 builders"
  - "0.4h: tools_call.rs branch + server_registry.rs annotation fields"
  - "2.0h: tests — 15 test files covering create/poll/resume/cancel/expiry/idempotency/checkpoint/race/oversized/gating/audit"
  - "0.5h: integration smoke — long-running test tool through full lifecycle"

risk_if_skipped: "Without Tasks primitive, long-running MCP tools (data exports, batch processing, model fine-tuning, KB index rebuilds) timeout at the gateway's 30s sync limit. Clients see opaque 504 errors; results lost. TASK-MCP-006 destructive long-running tools cannot be gated cleanly (sync confirmation pattern doesn't apply). Without DEC-1106's resume-on-reconnect, a client crash mid-task = lost result. Without DEC-1107's cancellation, runaway tasks consume worker capacity indefinitely. Without DEC-1108's bounded concurrency, one module's task spike starves all other modules. Without DEC-1116's checkpoints, a worker crash forces task restart from zero. Without DEC-1112's size cap, a single large result OOMs the gateway. Without DEC-1115's per-tenant rate limit, task creation becomes an abuse vector (cheap to create, expensive to run). The 10h effort lands the async-tool primitive that unblocks every long-running operation across all CyberOS modules."
---

## §1 — Description (BCP-14 normative)

The MCP service **MUST** ship Tasks primitive at `services/mcp/src/tasks/` per MCP 2025-11-25 spec — long-running tool calls return task_id immediately, expose status polling + cancellation + list endpoints, run in per-module bounded-concurrency worker pools, persist checkpoints for crash recovery, emit 8 memory audit kinds.

1. **MUST** define the closed `task_status` Postgres enum at migration `0009`: `('pending','running','completed','failed','cancelled','expired')` per DEC-1101. CI cardinality test asserts 6.

2. **MUST** define the closed `task_progress_unit` enum at migration `0009`: `('percent','items','bytes','none')` per DEC-1114. CI cardinality test asserts 4.

3. **MUST** define `mcp_tasks` table at migration `0009`: `(task_id UUID PRIMARY KEY, tenant_id UUID NOT NULL, caller_subject_id UUID NOT NULL, tool_id TEXT NOT NULL, status task_status NOT NULL DEFAULT 'pending', input_payload_kms_blob BYTEA NOT NULL, input_payload_sha256 CHAR(64) NOT NULL, result_payload_kms_blob BYTEA, result_url TEXT, error_code TEXT, error_message TEXT, error_details JSONB, progress_value DOUBLE PRECISION, progress_unit task_progress_unit NOT NULL DEFAULT 'none', progress_total DOUBLE PRECISION, idempotency_key TEXT, created_at TIMESTAMPTZ NOT NULL DEFAULT now(), started_at TIMESTAMPTZ, completed_at TIMESTAMPTZ, expires_at TIMESTAMPTZ NOT NULL, trace_id CHAR(32))`. Partial unique index `(tenant_id, idempotency_key) WHERE idempotency_key IS NOT NULL AND created_at > now() - interval '24 hours'`.

4. **MUST** define `mcp_task_checkpoints` table at migration `0010`: `(id BIGSERIAL PRIMARY KEY, task_id UUID NOT NULL REFERENCES mcp_tasks(task_id), seq INT NOT NULL, checkpoint_data_kms_blob BYTEA NOT NULL, created_at TIMESTAMPTZ NOT NULL DEFAULT now(), UNIQUE(task_id, seq))`. Per-task ordered checkpoints; append-only via REVOKE per task-audit skill rule 12.

5. **MUST** define `mcp_task_progress_events` table at migration `0011`: `(id BIGSERIAL PRIMARY KEY, task_id UUID NOT NULL REFERENCES mcp_tasks(task_id), progress_value DOUBLE PRECISION NOT NULL, progress_unit task_progress_unit NOT NULL, message TEXT, emitted_at TIMESTAMPTZ NOT NULL DEFAULT now())`. Progress event log; high-volume + sampled at 1% to memory.

6. **MUST** enforce RLS with both USING and WITH CHECK on all 3 tables: `tenant_id = current_setting('auth.tenant_id')::uuid`.

7. **MUST** extend TASK-MCP-002 `server_registry` with 3 new annotation fields per DEC-1102 + DEC-1103 + DEC-1108 + DEC-1120:
- `long_running: bool` — defaults false (synchronous tool); when true, tools/call returns task_id instead of result.
- `task_ttl_seconds: u32` — defaults 86400 (24h); max 604800 (7d).
- `max_concurrent_tasks: u32` — per-module worker pool size; defaults 4.
- `supports_cancellation: bool` — defaults true; when false, cancellation endpoint returns 405.

8. **MUST** branch `tools/call` handler per DEC-1102. If invoked tool's `long_running=true`:
- Generate UUIDv7 task_id per DEC-1104.
- Check TASK-MCP-006 gating (destructive_hint requires confirmation per DEC-1111); confirmation token consumed at task START only.
- Check tenant rate-limit per DEC-1115 (100/min/tenant); excess → 429.
- Check idempotency_key per DEC-1121; duplicate within 24h returns existing task_id.
- INSERT `mcp_tasks` row with status='pending', KMS-encrypt input_payload, `expires_at = now() + ttl_seconds`.
- Enqueue to per-module worker pool per DEC-1108.
- Return `202 Accepted` + JSON-RPC body `{ task_id, status, poll_url, expires_at }`.

9. **MUST** expose `GET /v1/mcp/tasks/{task_id}` for status polling per DEC-1105. Handler:
- Validates caller JWT + verifies `caller_subject_id = jwt.subject_id` (RLS + explicit check).
- Rate-limit 60/min/task (1/sec sufficient; busier = bot).
- Returns current row state: `{ task_id, status, progress: {value, unit, total}, started_at, completed_at, result?, result_url?, error? }`.
- On `status=completed` with inline result: includes `result: <json>` (size ≤ 10 MiB per DEC-1112).
- On oversized result: returns `result_url: <s3_url>` (TASK-DOC-001 reference); inline result is NULL.

10. **MUST** expose `POST /v1/mcp/tasks/{task_id}/cancel` per DEC-1107. Handler:
- Validates caller JWT + verifies `caller_subject_id = jwt.subject_id`.
- Looks up task; if status='pending' or 'running': transition to status='cancelling' (intermediate transient state; not in closed-enum because instantaneous).
- Sends cancellation signal to worker via Tokio cancel-token.
- Worker honours `is_cancelled()` at next checkpoint boundary per DEC-1122; transitions status='cancelled'; emits `mcp.task_cancelled`.
- If tool registration `supports_cancellation=false`: returns 405 + `cancellation_not_supported`.
- If task already completed/failed/cancelled/expired: returns 409 + `task_terminal`.
- Returns 202 Accepted (cancellation initiated; final status confirmed via subsequent poll).

11. **MUST** expose `GET /v1/mcp/tasks?status=...&tool_id=...&caller_subject_id=...` per DEC-1119. Pagination via cursor (last task_id). Caller MUST be `tenant_admin` to filter by other subjects; non-admin sees only own tasks.

12. **MUST** run per-module bounded-concurrency worker pool per DEC-1108 + DEC-1120. The `tasks/worker_pool.rs::ModulePool`:
- Tokio semaphore sized at `max_concurrent_tasks` per module.
- Pending tasks enqueued in `mcp_tasks` table; pickup query `SELECT ... WHERE status='pending' AND module=$1 ORDER BY created_at FOR UPDATE SKIP LOCKED LIMIT 1`.
- Worker acquires semaphore, transitions status='running', invokes tool handler.
- On completion/failure: transitions status accordingly, emits audit row, releases semaphore.
- Crashed workers (process restart): in-flight tasks resume from last checkpoint per DEC-1116; if no checkpoint, retry from scratch (with idempotency_key dedup).

13. **MUST** persist checkpoints per DEC-1116 via `tasks/checkpoint.rs::save(task_id, seq, data)`:
- Worker invokes during long operations at sensible boundaries (per-item processed, per-batch, etc.).
- INSERT `mcp_task_checkpoints` row.
- Optional; tools choose when to checkpoint.
- On task restart: worker loads latest checkpoint via `SELECT ... ORDER BY seq DESC LIMIT 1`; resumes from that state.

14. **MUST** emit progress events via `tasks/progress.rs::publish(task_id, value, unit, total, msg)` per DEC-1110:
- INSERT `mcp_task_progress_events` row.
- UPDATE `mcp_tasks.progress_*` columns (latest state for fast polling).
- NATS publish `tenant.<slug>.mcp.tasks.<task_id>.progress` with payload.
- Sample 1% to memory as `mcp.task_progress` (high-volume).

15. **MUST** expire tasks via daily scheduled job per DEC-1103 + DEC-1113. The `tasks/expiry_job.rs::run_daily()`:
- SELECT tasks WHERE status IN ('pending','running','completed','failed','cancelled') AND expires_at < now().
- For pending/running: transition status='expired' + signal worker cancel.
- For terminal (completed/failed/cancelled/expired): prune result_payload_kms_blob (set to NULL) — retain metadata.
- Emit `mcp.task_expired` per task.

16. **MUST** apply idempotency on task creation per DEC-1121. The `tasks/idempotency.rs::find_or_create(idempotency_key, tenant_id)`:
- Lookup partial unique index `(tenant_id, idempotency_key) WHERE created_at > now() - interval '24 hours'`.
- Hit: return existing task_id (200 OK with existing task state, NOT 202 Accepted with new).
- Miss: INSERT new task row.
- Atomic via INSERT ... ON CONFLICT.

17. **MUST** integrate with TASK-MCP-006 gating per DEC-1111 + DEC-1122. Confirmation at task START (sync phase of create.rs); subsequent polls do NOT re-confirm; cancellation does NOT require confirmation. The confirmation token consumed in tasks/create.rs per TASK-MCP-006 §1 #12 atomic-consume pattern.

18. **MUST** enforce result size cap 10 MiB per DEC-1112. Worker computing result checks size before INSERT; over-cap → status='failed' + error.code='-32001' (custom) + error.message='result_too_large' + suggestion to use TASK-DOC-001 streaming.

19. **MUST** preserve trace_id end-to-end per DEC-1123 + task-audit skill rule 22-24. Original tools/call trace_id stored in `mcp_tasks.trace_id`; emitted on every audit row + every progress event + every NATS publish.

20. **MUST** emit 8 memory audit row kinds per DEC-1124:
- `mcp.task_started` (sev-2)
- `mcp.task_progress` (sev-3 — high-volume; sampled at 1% via TASK-OBS-006)
- `mcp.task_completed` (sev-2)
- `mcp.task_failed` (sev-2)
- `mcp.task_cancelled` (sev-2)
- `mcp.task_expired` (sev-3)
- `mcp.task_resumed_after_reconnect` (sev-3 — informational, helps debug worker restarts)
- `mcp.task_checkpoint_persisted` (sev-3 — sampled at 1%)

21. **MUST** PII-scrub audit rows per DEC-1125 + task-audit skill rule 18. `input_payload_sha256` and `result_payload_sha256` only in chain; raw payloads in `mcp_tasks` (RLS-scoped, 30-day retention post-completion).

22. **MUST** auto-clean completed task payloads at T+30 days post-completion per DEC-1125. Daily job UPDATE `mcp_tasks SET input_payload_kms_blob=NULL, result_payload_kms_blob=NULL WHERE status IN ('completed','failed','cancelled','expired') AND completed_at < now() - interval '30 days'`. Metadata (task_id, status, timestamps, error_code) retained for forensic.

23. **MUST NOT** return long-running tool results synchronously per DEC-1102. Gateway-side check: tool's `long_running=true` AND handler tries to return non-task response → developer error, returns 500 + `long_running_must_return_task`.

24. **MUST NOT** allow cross-tenant task access per RLS + explicit check. Caller's JWT tenant_id must match task's tenant_id (RLS enforces); attempt = 403 + `cross_tenant_task_access_denied` + sev-1 audit (security signal).

25. **SHOULD** observe per-tool task duration via OTel histogram `mcp_task_duration_seconds{tool_id}` for operator visibility on long-task performance.

---

## §2 — Why this design (rationale for humans)

**Why async Tasks primitive (§1 #8, DEC-1100)?** MCP gateway sync calls have a 30s soft limit (load balancer timeouts; client UX). Long-running operations (KB index rebuild, large data export, model fine-tuning) routinely exceed 30s. Without async, these operations either fail or require gateway-side patching of timeouts (operationally fragile). Tasks primitive is the standard async pattern from MCP 2025-11-25 spec; aligns with REST conventions (`202 Accepted + Location header`).

**Why per-module bounded concurrency (§1 #12, DEC-1108)?** One slow module (e.g., docs export with large datasets) running unbounded tasks would consume all worker capacity, starving other modules' calls. Per-module semaphore = bounded blast radius. Default 4 concurrent per module = conservative; tunable per registration.

**Why checkpoints (§1 #13, DEC-1116)?** Worker process restarts (deploys, OOM kills, host failures) are real. Without checkpoints, every restart = retry from zero. Long tasks (hours) would never complete on a deployment-heavy day. Optional checkpointing lets tools choose granularity (per-item vs per-batch).

**Why result size cap 10 MiB (§1 #18, DEC-1112)?** Inline result in JSON-RPC response means it traverses the gateway response path; > 10 MiB = memory pressure + slow client downloads. TASK-DOC-001 S3 streaming is the right pattern for larger outputs; result_url indirection keeps the gateway lean.

**Why idempotency key (§1 #16, DEC-1121)?** Network retries are common (client times out + retries; load balancer retries). Without idempotency, a single user click can spawn 5 identical KB-rebuild tasks. Idempotency key = client's commitment to "this is the same logical operation"; 24h cache window covers reasonable retry scenarios.

**Why 24h default TTL + 7d max (§1 #15, DEC-1103)?** Tasks beyond 24h are operationally suspect (likely orphaned; client gave up). 7d max gives migration jobs etc. enough runway. Storage growth bounded by TTL + 30-day post-completion retention.

**Why per-tool `supports_cancellation` flag (§1 #10, DEC-1107)?** Some operations are inherently uncancellable (signed transaction in flight; FFI call holding C-side state). Declaring `supports_cancellation=false` is honest about the contract; alternative (always-cancellable) lies to the client when cancellation doesn't actually work.

**Why progress events polling-primary + push-optional (§1 #14, DEC-1110)?** Polling is universally implementable (any HTTP client). NATS push is faster + more efficient at scale but requires WebSocket/SSE + NATS access. Polling-required + push-optional means every client works; high-volume clients get the optimisation.

**Why gating integration at start, not at poll (§1 #17, DEC-1111)?** The confirmation is "user intends this operation"; once started, polling is just observation. Re-confirming at every poll would be UX nightmare + serves no security purpose (the work is already in flight; re-deny doesn't undo).

**Why audit rows for resumed-after-reconnect (§1 #20, DEC-1124)?** Worker crashes happen; resumed-after-restart is operationally meaningful (DR exercise, deploy hot-fix). Forensic visibility on which tasks survived a crash + which had to restart helps debug deployment issues.

**Why UUIDv7 task IDs (§1 #1 frontmatter, DEC-1104)?** Sortable (chronological list works natively); collision-free across distributed workers (no central counter needed); compatible with TASK-TEN-103 residency-nibble encoding.

**Why per-tenant rate limit on creation (§1 #8, DEC-1115)?** Tasks are cheap to create (one INSERT), expensive to run (workers + storage + compute). Without rate limit, a malicious or buggy client can DOS the worker pool via task creation. 100/min/tenant is generous (operationally; org-wide bulk-export scenarios fit) but blocks runaway loops.

**Why 30-day post-completion retention (§1 #22, DEC-1125)?** Result payloads can be large + PII-laden. 30 days covers typical re-download scenarios (user comes back next sprint); pruning beyond retains metadata for forensic without storage cost.

---

## §3 — API contract

### 3.1 Postgres schema

```sql
-- 0009_mcp_tasks.sql
CREATE TYPE task_status AS ENUM ('pending','running','completed','failed','cancelled','expired');
CREATE TYPE task_progress_unit AS ENUM ('percent','items','bytes','none');

CREATE TABLE mcp_tasks (
  task_id UUID PRIMARY KEY,
  tenant_id UUID NOT NULL,
  caller_subject_id UUID NOT NULL,
  tool_id TEXT NOT NULL,
  status task_status NOT NULL DEFAULT 'pending',
  input_payload_kms_blob BYTEA NOT NULL,
  input_payload_sha256 CHAR(64) NOT NULL,
  result_payload_kms_blob BYTEA,
  result_url TEXT,
  error_code TEXT,
  error_message TEXT,
  error_details JSONB,
  progress_value DOUBLE PRECISION,
  progress_unit task_progress_unit NOT NULL DEFAULT 'none',
  progress_total DOUBLE PRECISION,
  idempotency_key TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  started_at TIMESTAMPTZ,
  completed_at TIMESTAMPTZ,
  expires_at TIMESTAMPTZ NOT NULL,
  trace_id CHAR(32)
);
-- Partial unique on idempotency_key — Postgres requires IMMUTABLE predicate, so no time bound here;
-- 24-hour enforcement is via the daily prune job (see §11.8) that deletes idempotency_key on rows > 24h old.
CREATE UNIQUE INDEX uniq_idempotency_active
  ON mcp_tasks(tenant_id, idempotency_key)
  WHERE idempotency_key IS NOT NULL;
CREATE INDEX idx_mcp_tasks_status_module
  ON mcp_tasks(status, tool_id, created_at)
  WHERE status IN ('pending','running');
CREATE INDEX idx_mcp_tasks_caller ON mcp_tasks(caller_subject_id, created_at DESC);
ALTER TABLE mcp_tasks ENABLE ROW LEVEL SECURITY;
CREATE POLICY mcp_tasks_rls ON mcp_tasks
  USING (tenant_id = current_setting('auth.tenant_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
REVOKE DELETE ON mcp_tasks FROM cyberos_app;
GRANT UPDATE (status, result_payload_kms_blob, result_url, error_code, error_message, error_details,
              progress_value, progress_unit, progress_total, started_at, completed_at) ON mcp_tasks TO cyberos_app;
GRANT DELETE ON mcp_tasks TO cyberos_pruner;  -- expired-row cleanup (metadata retained via re-INSERT pattern in slice 3)

-- 0010_mcp_task_checkpoints.sql
CREATE TABLE mcp_task_checkpoints (
  id BIGSERIAL PRIMARY KEY,
  task_id UUID NOT NULL REFERENCES mcp_tasks(task_id),
  seq INT NOT NULL,
  checkpoint_data_kms_blob BYTEA NOT NULL,
  kms_key_id TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (task_id, seq)
);
CREATE INDEX idx_checkpoints_task_seq ON mcp_task_checkpoints(task_id, seq DESC);
ALTER TABLE mcp_task_checkpoints ENABLE ROW LEVEL SECURITY;
CREATE POLICY mcp_task_checkpoints_rls ON mcp_task_checkpoints
  USING (task_id IN (SELECT task_id FROM mcp_tasks WHERE tenant_id = current_setting('auth.tenant_id')::uuid))
  WITH CHECK (task_id IN (SELECT task_id FROM mcp_tasks WHERE tenant_id = current_setting('auth.tenant_id')::uuid));
REVOKE UPDATE, DELETE ON mcp_task_checkpoints FROM cyberos_app;
GRANT DELETE ON mcp_task_checkpoints TO cyberos_pruner;

-- 0011_mcp_task_progress_events.sql
CREATE TABLE mcp_task_progress_events (
  id BIGSERIAL PRIMARY KEY,
  task_id UUID NOT NULL REFERENCES mcp_tasks(task_id),
  progress_value DOUBLE PRECISION NOT NULL,
  progress_unit task_progress_unit NOT NULL,
  message TEXT,
  emitted_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_progress_events_task ON mcp_task_progress_events(task_id, emitted_at DESC);
ALTER TABLE mcp_task_progress_events ENABLE ROW LEVEL SECURITY;
CREATE POLICY mcp_task_progress_events_rls ON mcp_task_progress_events
  USING (task_id IN (SELECT task_id FROM mcp_tasks WHERE tenant_id = current_setting('auth.tenant_id')::uuid))
  WITH CHECK (task_id IN (SELECT task_id FROM mcp_tasks WHERE tenant_id = current_setting('auth.tenant_id')::uuid));
REVOKE UPDATE, DELETE ON mcp_task_progress_events FROM cyberos_app;
GRANT DELETE ON mcp_task_progress_events TO cyberos_pruner;
```

### 3.2 Rust types

```rust
// services/mcp/src/tasks/mod.rs
#[derive(Copy, Clone, Eq, PartialEq, Debug, sqlx::Type, serde::Serialize)]
#[sqlx(type_name = "task_status", rename_all = "snake_case")]
pub enum TaskStatus { Pending, Running, Completed, Failed, Cancelled, Expired }

impl TaskStatus {
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled | Self::Expired)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, sqlx::Type, serde::Serialize)]
#[sqlx(type_name = "task_progress_unit", rename_all = "snake_case")]
pub enum TaskProgressUnit { Percent, Items, Bytes, None }

#[derive(Debug, serde::Serialize)]
pub struct TaskCreateResponse {
    pub task_id: Uuid,
    pub status: TaskStatus,
    pub poll_url: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, serde::Serialize)]
pub struct TaskStatusResponse {
    pub task_id: Uuid,
    pub status: TaskStatus,
    pub progress: Option<Progress>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result: Option<serde_json::Value>,
    pub result_url: Option<String>,
    pub error: Option<TaskError>,
}

#[derive(Debug, serde::Serialize)]
pub struct Progress { pub value: f64, pub unit: TaskProgressUnit, pub total: Option<f64> }

#[derive(Debug, serde::Serialize)]
pub struct TaskError { pub code: String, pub message: String, pub details: Option<serde_json::Value> }
```

### 3.3 REST endpoints

```text
POST   /v1/mcp/tools/{tool_id}/call                  (extended — branches sync vs task by tool annotation)
GET    /v1/mcp/tasks/{task_id}                       (status poll; caller-owned)
POST   /v1/mcp/tasks/{task_id}/cancel                (cancellation; caller-owned)
GET    /v1/mcp/tasks                                  (list; tenant_admin filter)
```

---

## §4 — Acceptance criteria

1. **Long-running tool returns task_id** — tool registered `long_running=true` invoked via tools/call → 202 Accepted + `{ task_id, status: "pending", poll_url, expires_at }`.
2. **Status polling returns running state** — within 1s of task creation, status poll returns `status=running` + `started_at` populated.
3. **Status poll terminal** — after worker completion, poll returns `status=completed` + `result: <json>` inline.
4. **Resume after reconnect** — client closes session, reconnects with new JWT (same subject), polls task_id → state intact; emits `mcp.task_resumed_after_reconnect`.
5. **Cancellation transitions to cancelled** — `POST /cancel` on running task → worker checkpoint-honour → status='cancelled'; subsequent poll returns cancelled.
6. **Cancellation race** — cancel issued between checkpoint + completion → next checkpoint detects cancellation + transitions cancelled (not completed).
7. **TTL expiry** — task with `expires_at = now() + 1s` after expiry job → status='expired' + result_payload pruned + audit emitted.
8. **task_status enum cardinality** — enum = exactly `{pending, running, completed, failed, cancelled, expired}`.
9. **task_progress_unit enum cardinality** — enum = exactly `{percent, items, bytes, none}`.
10. **Oversized result fails** — tool returning 11 MiB inline → status='failed' + error.code='-32001' + error.message='result_too_large'.
11. **Gating at start** — destructive long-running tool requires confirmation; first tools/call returns 403; post-confirm returns 202 task.
12. **Idempotency** — duplicate tools/call with same idempotency_key within 24h → returns existing task_id (200 OK), not new task (202 Accepted).
13. **Worker pool isolation** — module A with `max_concurrent_tasks=2` saturated; module B's tasks unaffected.
14. **Checkpoint resume** — worker checkpoints at item 50/100; simulated crash; restarted worker resumes from item 50; final result correct.
15. **Per-tenant rate limit** — 101st task creation in 60s → 429 + Retry-After.
16. **Cross-tenant access denied** — caller from tenant X polling tenant Y's task_id → 403 + `cross_tenant_task_access_denied` + sev-1 audit.
17. **Cancellation on uncancellable tool** — tool with `supports_cancellation=false` → 405 + `cancellation_not_supported`.
18. **Progress event sampling** — 100 progress events emitted → ~1 memory audit row + 100 progress_events table rows.
19. **Trace_id end-to-end** — single trace_id across tools/call + status poll + cancel + audit rows.
20. **8 memory audit kinds emitted** — happy + failure paths cover started + completed + failed + cancelled + expired + resumed + progress + checkpoint.

---

## §5 — Verification

### 5.1 `task_create_async_test.rs`

```rust
#[tokio::test]
async fn long_running_tool_returns_task_id() {
    let ctx = TestContext::new().await;
    ctx.register_tool("cyberos.kb.reindex", ToolAnnotations { long_running: true, ..default() }, /*max_concurrent*/ 2).await;

    let r = ctx.invoke_tool("cyberos.kb.reindex", json!({"corpus": "all"})).await;
    assert_eq!(r.status(), 202);
    let body: serde_json::Value = r.json().await.unwrap();
    let task_id: String = body["task_id"].as_str().unwrap().into();
    assert!(task_id.starts_with("task_") || uuid::Uuid::parse_str(&task_id).is_ok());
    assert_eq!(body["status"], "pending");
    assert!(body["poll_url"].as_str().unwrap().contains(&task_id));
}
```

### 5.2 `task_status_poll_test.rs`

```rust
#[tokio::test]
async fn poll_progresses_through_states() {
    let ctx = TestContext::new().await;
    ctx.register_test_tool_that_takes("3s").await;
    let task_id = ctx.create_task("cyberos.test.slow", json!({})).await;

    // poll until running
    let s1 = ctx.poll_until(task_id, |s| s != "pending", Duration::from_secs(2)).await;
    assert_eq!(s1, "running");

    // poll until completed
    let s2 = ctx.poll_until(task_id, |s| s == "completed", Duration::from_secs(10)).await;
    assert_eq!(s2, "completed");

    let final_resp: serde_json::Value = ctx.poll_task(task_id).await.json().await.unwrap();
    assert!(final_resp["completed_at"].is_string());
    assert!(final_resp["result"].is_object());
}
```

### 5.3 `task_cancellation_test.rs`

```rust
#[tokio::test]
async fn cancel_transitions_to_cancelled() {
    let ctx = TestContext::new().await;
    ctx.register_test_tool_that_takes("30s").await;
    let task_id = ctx.create_task("cyberos.test.long", json!({})).await;
    tokio::time::sleep(Duration::from_millis(500)).await;

    let r = ctx.cancel_task(task_id).await;
    assert_eq!(r.status(), 202);

    let s = ctx.poll_until(task_id, |s| s == "cancelled", Duration::from_secs(5)).await;
    assert_eq!(s, "cancelled");
}
```

### 5.4 `task_cancellation_race_test.rs`

```rust
#[tokio::test]
async fn cancellation_between_checkpoints_wins() {
    let ctx = TestContext::new().await;
    ctx.register_test_tool_with_checkpoints(100).await;
    let task_id = ctx.create_task("cyberos.test.checkpointed", json!({})).await;
    ctx.wait_for_checkpoint(task_id, 50).await;
    let cancel_r = ctx.cancel_task(task_id).await;
    assert_eq!(cancel_r.status(), 202);

    let s = ctx.poll_until(task_id, |s| s == "cancelled", Duration::from_secs(3)).await;
    assert_eq!(s, "cancelled");

    let final_state: serde_json::Value = ctx.poll_task(task_id).await.json().await.unwrap();
    assert!(final_state["progress"]["value"].as_f64().unwrap() < 100.0);
}
```

### 5.5 `task_ttl_expiry_test.rs`

```rust
#[tokio::test]
async fn expired_task_pruned() {
    let ctx = TestContext::new().await;
    let task_id = ctx.create_task_with_ttl("cyberos.test.slow", 1).await;
    tokio::time::sleep(Duration::from_secs(2)).await;
    ctx.run_expiry_job().await;

    let row: (String, Option<Vec<u8>>) = sqlx::query_as(
        "SELECT status::text, result_payload_kms_blob FROM mcp_tasks WHERE task_id=$1"
    ).bind(task_id).fetch_one(&ctx.pool).await.unwrap();
    assert_eq!(row.0, "expired");
    assert!(row.1.is_none());

    let audit = ctx.memory_rows().await;
    assert!(audit.iter().any(|r| r.kind == "mcp.task_expired"));
}
```

### 5.6 `task_status_enum_cardinality_test.rs`

```rust
#[tokio::test]
async fn task_status_has_6_values() {
    let ctx = TestContext::new().await;
    let labels: Vec<String> = sqlx::query_scalar(
        "SELECT unnest(enum_range(NULL::task_status))::text"
    ).fetch_all(&ctx.pool).await.unwrap();
    let mut labels = labels; labels.sort();
    assert_eq!(labels, vec!["cancelled","completed","expired","failed","pending","running"]);
}
```

### 5.7 `task_oversized_result_test.rs`

```rust
#[tokio::test]
async fn result_over_10mb_fails() {
    let ctx = TestContext::new().await;
    ctx.register_tool_that_returns_bytes(11 * 1024 * 1024).await;
    let task_id = ctx.create_task("cyberos.test.big_result", json!({})).await;
    let s = ctx.poll_until(task_id, |s| s == "failed", Duration::from_secs(5)).await;
    assert_eq!(s, "failed");
    let body: serde_json::Value = ctx.poll_task(task_id).await.json().await.unwrap();
    assert_eq!(body["error"]["message"], "result_too_large");
}
```

### 5.8 `task_idempotency_test.rs`

```rust
#[tokio::test]
async fn duplicate_idempotency_returns_existing_task() {
    let ctx = TestContext::new().await;
    let r1 = ctx.invoke_tool_with_idempotency("cyberos.test.long", json!({}), "key-abc").await;
    let r2 = ctx.invoke_tool_with_idempotency("cyberos.test.long", json!({}), "key-abc").await;

    let id1: String = r1.json::<serde_json::Value>().await.unwrap()["task_id"].as_str().unwrap().into();
    let id2: String = r2.json::<serde_json::Value>().await.unwrap()["task_id"].as_str().unwrap().into();
    assert_eq!(id1, id2);

    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM mcp_tasks WHERE idempotency_key='key-abc'")
        .fetch_one(&ctx.pool).await.unwrap();
    assert_eq!(count, 1);
}
```

### 5.9 `task_resume_after_reconnect_test.rs`

```rust
#[tokio::test]
async fn resume_after_simulated_worker_restart() {
    let ctx = TestContext::new().await;
    ctx.register_test_tool_with_checkpoints(100).await;
    let task_id = ctx.create_task("cyberos.test.checkpointed", json!({})).await;
    ctx.wait_for_checkpoint(task_id, 50).await;
    ctx.simulate_worker_crash().await;
    ctx.restart_worker_pool().await;

    let s = ctx.poll_until(task_id, |s| s == "completed", Duration::from_secs(10)).await;
    assert_eq!(s, "completed");
    let audit = ctx.memory_rows().await;
    assert!(audit.iter().any(|r| r.kind == "mcp.task_resumed_after_reconnect"));
}
```

### 5.10 `task_worker_pool_isolation_test.rs`

```rust
#[tokio::test]
async fn module_a_saturation_does_not_starve_module_b() {
    let ctx = TestContext::new().await;
    ctx.register_tool_in_module("cyberos.a.slow", "module-a", /*max_concurrent*/ 2, /*long_running*/ true).await;
    ctx.register_tool_in_module("cyberos.b.fast", "module-b", /*max_concurrent*/ 2, /*long_running*/ true).await;

    for _ in 0..10 {
        ctx.invoke_tool("cyberos.a.slow", json!({})).await;
    }

    let b_start = Instant::now();
    let b_id = ctx.invoke_tool("cyberos.b.fast", json!({})).await;
    let b_status = ctx.poll_until(b_id, |s| s == "completed", Duration::from_secs(3)).await;
    assert_eq!(b_status, "completed");
    assert!(b_start.elapsed() < Duration::from_secs(3), "B was starved by A");
}
```

---

## §6 — Implementation skeleton

### 6.1 Worker pool

```rust
// services/mcp/src/tasks/worker_pool.rs
pub struct ModulePool {
    module: String,
    semaphore: Arc<tokio::sync::Semaphore>,
    pool: PgPool,
    cancel_tokens: Arc<DashMap<Uuid, CancellationToken>>,
}

impl ModulePool {
    pub async fn run(&self) -> ! {
        loop {
            let task = self.pick_next_pending().await;
            let permit = self.semaphore.clone().acquire_owned().await.unwrap();
            let cancel_token = CancellationToken::new();
            self.cancel_tokens.insert(task.task_id, cancel_token.clone());
            let pool = self.pool.clone();
            let cancel_tokens = self.cancel_tokens.clone();
            tokio::spawn(async move {
                let _permit = permit;
                let result = run_task_with_cancellation(&pool, task.clone(), cancel_token).await;
                cancel_tokens.remove(&task.task_id);
                persist_task_result(&pool, task.task_id, result).await;
            });
        }
    }

    pub fn cancel_task(&self, task_id: Uuid) -> bool {
        if let Some((_, token)) = self.cancel_tokens.remove(&task_id) {
            token.cancel();
            true
        } else {
            false
        }
    }
}
```

### 6.2 Checkpoint API for tool implementers

```rust
// services/mcp/src/tasks/checkpoint.rs
pub struct TaskCtx {
    pub task_id: Uuid,
    pub cancel_token: CancellationToken,
    seq: AtomicU32,
    pool: PgPool,
}

impl TaskCtx {
    pub async fn checkpoint<T: Serialize>(&self, state: &T) -> Result<(), CheckpointError> {
        if self.cancel_token.is_cancelled() {
            return Err(CheckpointError::Cancelled);
        }
        let seq = self.seq.fetch_add(1, Ordering::SeqCst);
        let bytes = serde_json::to_vec(state)?;
        let encrypted = kms_encrypt(&bytes).await?;
        sqlx::query("INSERT INTO mcp_task_checkpoints (task_id, seq, checkpoint_data_kms_blob, kms_key_id) VALUES ($1, $2, $3, $4)")
            .bind(self.task_id).bind(seq as i32).bind(encrypted.blob).bind(encrypted.key_id)
            .execute(&self.pool).await?;
        Ok(())
    }

    pub fn is_cancelled(&self) -> bool { self.cancel_token.is_cancelled() }

    pub async fn restore_latest<T: DeserializeOwned>(&self) -> Result<Option<T>, CheckpointError> {
        let row: Option<(Vec<u8>,)> = sqlx::query_as(
            "SELECT checkpoint_data_kms_blob FROM mcp_task_checkpoints WHERE task_id=$1 ORDER BY seq DESC LIMIT 1"
        ).bind(self.task_id).fetch_optional(&self.pool).await?;
        match row {
            Some((blob,)) => {
                let bytes = kms_decrypt(&blob).await?;
                Ok(Some(serde_json::from_slice(&bytes)?))
            }
            None => Ok(None),
        }
    }
}
```

---

## §7 — Dependencies

**Upstream (depends_on):**
- **TASK-MCP-001** spec compliance — tools/call handler branches sync vs task here.

**Cross-module (related_tasks):**
- **TASK-MCP-002** Per-module registration — extended with long_running + ttl + max_concurrent fields.
- **TASK-MCP-004** OAuth 2.1 PKCE — JWT bearer auth for poll/cancel.
- **TASK-MCP-005** PRM — long_running tools advertised via per-module PRM (slice-3 add).
- **TASK-MCP-006** Gating — destructive long-running tools confirm at start; TASK-MCP-006 §1 #22 cross-task contract honoured here.
- **TASK-MCP-008** Elicitation — long-running tools may emit elicitation requests mid-run (slice 3+).
- **TASK-AUTH-004** JWT validate — caller_subject_id verification on poll/cancel.
- **TASK-AI-003** memory audit-row bridge — 8 new kinds.
- **TASK-MEMORY-111** PII scrubbing — payload SHA only in chain.
- **TASK-OBS-005** Trace correlation — trace_id end-to-end.
- **TASK-OBS-007** Auto-runbook — sev-1 cross-tenant access + expiry-job failure alerts.

**Downstream (blocks):** None at this slice.

---

## §8 — Example payloads

### 8.1 Task creation response (202)

```json
{
  "jsonrpc": "2.0",
  "result": {
    "task_id": "0190f7c0-8b3c-7a4f-aaaa-000000000001",
    "status": "pending",
    "poll_url": "https://api.cyberos.world/v1/mcp/tasks/0190f7c0-...",
    "expires_at": "2026-05-18T09:14:32.847Z"
  },
  "id": 42
}
```

### 8.2 Status poll response (running)

```json
{
  "task_id": "0190f7c0-...",
  "status": "running",
  "progress": { "value": 47.3, "unit": "percent", "total": 100.0 },
  "started_at": "2026-05-17T09:14:33.221Z",
  "completed_at": null,
  "result": null,
  "result_url": null,
  "error": null
}
```

### 8.3 Status poll response (completed)

```json
{
  "task_id": "0190f7c0-...",
  "status": "completed",
  "progress": { "value": 100.0, "unit": "percent", "total": 100.0 },
  "started_at": "2026-05-17T09:14:33.221Z",
  "completed_at": "2026-05-17T09:18:42.118Z",
  "result": { "indexed_documents": 12847, "elapsed_seconds": 248 },
  "result_url": null,
  "error": null
}
```

### 8.4 `mcp.task_started` memory row

```json
{
  "kind": "mcp.task_started",
  "severity": 2,
  "tenant_id": "8a2f...",
  "actor_id": "user.subject.456",
  "trace_id": "0af7651916cd43dd8448eb211c80319c",
  "occurred_at": "2026-05-17T09:14:33.221Z",
  "payload": {
    "task_id": "0190f7c0-...",
    "tool_id": "cyberos.kb.reindex",
    "input_payload_sha256": "9c4e7a8b6d2f1e3a...",
    "ttl_seconds": 86400,
    "max_concurrent_tasks_remaining": 1
  }
}
```

---

## §9 — Open questions

All resolved for slice 3. Deferred:

- **Deferred:** Streaming progress via SSE (vs polling + NATS) — slice 4.
- **Deferred:** Task priority queue (high/normal/low) — slice 4.
- **Deferred:** Task dependency graph (task B starts after task A completes) — slice 4.
- **Deferred:** Per-tool retry policy (auto-retry on failure with backoff) — slice 4.
- **Deferred:** Worker affinity (sticky-worker for stateful tools) — slice 4.
- **Deferred:** Task cost attribution (per-tenant compute budget) — slice 4, TASK-TEN-004 derivative.
- **Deferred:** Task templates (named pre-configured task definitions) — slice 4.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Worker crashes mid-task | process supervisor restart | Crashed task remains status='running' briefly; expiry detector flags > 60s no-progress; resumes from checkpoint if available | Inherent — checkpoint resume; if no checkpoint, retry from scratch (idempotency_key prevents double-side-effect) |
| Tool handler panics | tokio::spawn catches | status='failed' + error.code='-32000' + error.message=panic_message | Inherent — caller sees error + retries with fixed input |
| Cancellation arrives after completion | terminal-state check | Returns 409 + `task_terminal` | Inherent — caller knows task already done |
| Idempotency cache hit returns task in failed state | partial unique lookup | Returns existing failed task (same task_id); client decides retry | Caller checks status before assuming reuse |
| Result oversized (> 10 MiB) | size check pre-INSERT | status='failed' + error.code='-32001' + `result_too_large` | Tool implementer adds TASK-DOC-001 streaming |
| Worker pool saturated | semaphore blocks | Task remains status='pending'; picked up when slot frees | Inherent backpressure |
| Cross-tenant poll attempt | RLS + explicit check | 403 + sev-1 audit + caller's tenant audit row | Investigate compromised JWT or misconfigured client |
| TTL hit while task running | expiry job | status='expired' + cancel signal sent to worker | Caller sees expired status on next poll |
| Checkpoint write fails | sqlx error | Checkpoint skipped; warning logged; worker continues (next checkpoint may succeed) | Inherent best-effort |
| Cancellation token consumed but worker no-op | next-checkpoint check missed | status remains 'running' until completion or next expiry tick | §11 documents tool implementer responsibility to check is_cancelled at every checkpoint boundary |
| KMS unavailable for input decrypt | KMS timeout | status='failed' + error.code='-32002' + `kms_unavailable` | AWS KMS recovers; manual restart |
| Idempotency-key collision across tenants | partial unique scoped to (tenant_id, idempotency_key) | Inherent — different tenants OK | None needed |
| Progress event NATS publish fails | NATS error | Logged; polling-path still works | Inherent — push optional |
| Status poll on long-completed task | < 30d post-completion | Returns row with result_payload still present | Caller copies result before 30d |
| Status poll on > 30d completed task | result pruned | Returns metadata-only (no result); client must re-trigger | Caller saved result earlier |
| Per-tenant rate limit hit | counter | 429 + Retry-After | Caller backs off |
| Reserved tool annotation `long_running=true` declared but tool returns sync | gateway check | 500 + `long_running_must_return_task` | Tool implementer fixes registration or handler |
| Worker holds task forever (deadlocked tool) | watchdog: no progress event > 10x TTL | status='failed' + error.code='-32003' + `worker_deadlock_detected` | Manual operator review; tool fix |
| Cancellation against `supports_cancellation=false` | annotation check | 405 + `cancellation_not_supported` | Caller waits for natural completion or TTL |
| `mcp_task_progress_events` table grows unbounded | partition by month + prune job | Old partitions dropped at 90 days | Inherent retention policy |
| Cancellation cascade for parent-task with sub-tasks | not supported at slice 3 | Sub-tasks NOT cancelled when parent cancelled | Slice 4 dependency graph |
| Caller poll-rate-limit hit | per-task counter | 429; caller backs off | Inherent |
| UUIDv7 generation collision | partial unique constraint | INSERT fails; retry with new ID | Astronomical odds |

---

## §11 — Implementation notes

**§11.1** Worker pool initial design uses Tokio semaphore for in-process concurrency control; multi-process pool coordination via Postgres `FOR UPDATE SKIP LOCKED` pickup query.

**§11.2** Cancellation tokens use `tokio-util::sync::CancellationToken` — supports both checking + awaiting cancellation.

**§11.3** Checkpoint serialization is tool-defined (caller's struct); the framework just persists bytes. Tool implementer documents the schema.

**§11.4** Worker watchdog (§10 deadlock-detected row) is a slice-3 enhancement — implementation gated on operator-observed need; slice-2 relies on TTL only.

**§11.5** The `is_cancelled()` check at every checkpoint boundary is the tool implementer's responsibility — framework provides the API, tool must call it.

**§11.6** Progress events partitioned monthly by `emitted_at` (Postgres declarative partitioning); old partitions dropped at 90 days via scheduled job (slice 3 enhancement; slice 2 keeps single-table + bounded rate-limit).

**§11.7** UUIDv7 generation: same `services/ten/src/residency/uuid_gen.rs` from TASK-TEN-103 (cross-residency collision-free encoding).

**§11.8** Task creation idempotency_key cleared by daily prune (24h window enforced by query, not by deletion; old keys stay in table but not matched).

**§11.9** The 10 MiB result-size cap is checked BEFORE KMS encryption (saves crypto cost on doomed results).

**§11.10** Per-tenant rate limit uses Redis sliding-window (matches TASK-TEN-101 pattern); fallback to Postgres counter if Redis down (degraded throughput acceptable).

**§11.11** Worker process restart triggers re-pickup of tasks with status='running' AND `started_at < now() - interval '60s'` (heuristic for crashed workers); checkpoint resume re-attempted.

**§11.12** Cross-tenant poll attempts log to a dedicated `mcp_security_events` table (slice 4) for SOC monitoring; slice 3 audit row + OBS alarm sufficient.

**§11.13** OTel histogram `mcp_task_duration_seconds` labels by `(tool_id, status)` — high cardinality on tool_id but acceptable at 100s of tools.

**§11.14** The 8-kind core list (§1 #20) emits at every state transition; ensure no double-emit on retry paths (idempotency-key dedup at audit-row level).

**§11.15** Progress NATS publish uses subject pattern `tenant.<slug>.mcp.tasks.<task_id>.progress`; clients subscribe with task_id for targeted push.

**§11.16** Worker pool sizing tunable per-module via registration; default 4 = balance between throughput + resource isolation.

**§11.17** TTL expiry job runs daily at 02:00 UTC; one query covers all tenants (system-tenant scope); does NOT need RLS (operates on raw table).

**§11.18** Checkpoint encryption key is per-task (not per-tenant) to allow future per-task key rotation (slice 4).

**§11.19** Tool implementer can opt out of checkpointing by never calling `ctx.checkpoint()`; framework treats as no-checkpoint task (restart-from-scratch on worker crash).

**§11.20** Progress polling caches latest state in Redis with 1s TTL — reduces DB load at 1/sec poll rate.

---

*End of TASK-MCP-007 spec.*
