---
id: TASK-PLUGIN-006
title: "memory audit emission — every plugin install/update/uninstall/invoke/auth event produces a plugin.* audit row with idempotent retry queue"
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
eu_ai_act_risk_class: not_ai
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-19T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: plugin
priority: p0
status: draft
verify: T
phase: P1
milestone: P1 · slice 2
slice: 2
owner: Stephen Cheng (CTO)
created: 2026-05-19
shipped: null
memory_chain_hash: null
related_tasks: [TASK-PLUGIN-001, TASK-PLUGIN-002, TASK-PLUGIN-005, TASK-PLUGIN-007, TASK-PLUGIN-008, TASK-MEMORY-101, TASK-MEMORY-104, TASK-OBS-001]
depends_on: [TASK-PLUGIN-002, TASK-MEMORY-101]
blocks: [TASK-PLUGIN-008]

source_pages:
  - modules/plugin/INTEROP.md universal constraint 4
  - modules/plugin/manifest.schema.json (audit section)
  - modules/memory/AGENTS.md §3-§4 (canonical audit row shape)

source_decisions:
  - DEC-2450 2026-05-19 — Plugin lifecycle emits 6 audit kinds: plugin.installed / plugin.updated / plugin.uninstalled / plugin.invoked / plugin.auth_refreshed / plugin.scope_denied
  - DEC-2451 2026-05-19 — Audit emission is REQUIRED — failure to emit MUST queue locally with idempotency key and retry; MUST NOT drop the row
  - DEC-2452 2026-05-19 — Retry queue is durable (Postgres-backed table plugin_host.audit_outbox) — survives process restart
  - DEC-2453 2026-05-19 — Idempotency key is SHA-256 of (kind || tenant_id || subject_id || trace_id || event_timestamp_ms) — same event produces same key, memory dedups on collision
  - DEC-2454 2026-05-19 — Retry policy is exponential backoff (250ms / 1s / 4s / 16s / 64s / 256s) up to 24h; after 24h, row marked failed and surfaces to TASK-OBS-007 alert
  - DEC-2455 2026-05-19 — Audit row body MUST include: plugin_id, plugin_version, tenant_id, subject_id, trace_id, outcome (success / error_class); body MUST NOT include user data from tool input/output
  - DEC-2456 2026-05-19 — Audit-emission failures themselves MUST emit an OTel span tagged plugin.audit_emission_failed for TASK-OBS-001 observability

language: rust 1.81
service: services/plugin-host/
new_files:
  - services/plugin-host/src/audit/mod.rs
  - services/plugin-host/src/audit/emitter.rs
  - services/plugin-host/src/audit/outbox.rs
  - services/plugin-host/src/audit/idempotency.rs
  - services/plugin-host/src/audit/retry.rs
  - services/plugin-host/migrations/0002_audit_outbox.sql
  - services/plugin-host/tests/audit_emission_happy_path_test.rs
  - services/plugin-host/tests/audit_emission_outbox_durability_test.rs
  - services/plugin-host/tests/audit_emission_idempotency_test.rs
  - services/plugin-host/tests/audit_emission_no_user_data_leak_test.rs
  - services/plugin-host/tests/audit_emission_retry_backoff_test.rs

modified_files:
  - services/plugin-host/src/handlers/tools_call.rs (emit plugin.invoked)
  - services/plugin-host/src/auth/refresh.rs (emit plugin.auth_refreshed)
  - services/plugin-host/src/auth/scope_check.rs (emit plugin.scope_denied)

allowed_tools:
  - file_read: services/plugin-host/**
  - file_write: services/plugin-host/{src,tests,migrations}/**
  - bash: cd services && cargo test -p cyberos-plugin-host audit

disallowed_tools:
  - drop audit row on emission failure (per DEC-2451)
  - leak tool input/output into audit body (per DEC-2455)
  - skip OTel emission on failure (per DEC-2456)

effort_hours: 6
subtasks:
  - "0.4h: audit/mod.rs trait + types + 6 audit kinds"
  - "0.8h: audit/emitter.rs (memory HTTP POST + status mapping)"
  - "1.0h: audit/outbox.rs (Postgres durable queue + RLS)"
  - "0.5h: audit/idempotency.rs (SHA-256 key composer)"
  - "0.8h: audit/retry.rs (exponential backoff worker + 24h timeout)"
  - "0.3h: 0002_audit_outbox.sql"
  - "2.2h: 5 test files"

risk_if_skipped: "Without audit emission, plugin operations are invisible to the memory — Strategy §2 'audit-chained' positioning collapses. Without DEC-2451 retry-not-drop, transient memory outages silently lose rows, breaking compliance reporting (SOC 2, GDPR DSAR). Without DEC-2452 durable outbox, process restart loses queued rows. Without DEC-2453 idempotency, retries produce duplicate audit entries. Without DEC-2455 body-scrubbing, user data leaks into the audit chain (privacy violation). Without DEC-2456 OTel emission on failure, audit-emission outages are invisible to ops."
---

## §1 — Description (BCP-14 normative)

The PLUGIN module **MUST** emit memory audit rows for every plugin lifecycle event. The emitter at `services/plugin-host/src/audit/` writes to the memory HTTP endpoint, queues failures to a durable Postgres outbox, and retries with exponential backoff for up to 24 hours.

1. **MUST** emit exactly 6 audit kinds per DEC-2450:
- `plugin.installed` — on first successful OAuth-PKCE token exchange (per TASK-PLUGIN-005 clause 9)
- `plugin.updated` — on version bump install replacing a prior version of the same plugin_id
- `plugin.uninstalled` — on revoke + grant deletion
- `plugin.invoked` — on every successful tool call (one row per `tools/call`)
- `plugin.auth_refreshed` — on every refresh-token rotation
- `plugin.scope_denied` — on every scope check failure

2. **MUST** treat audit emission as REQUIRED per DEC-2451 — a tool call response MUST NOT be returned to the host until either (a) the audit row is acknowledged by memory, OR (b) the row is durably queued in the outbox. Failures in both paths MUST surface as `upstream_unavailable` error class.

3. **MUST** persist the outbox in Postgres per DEC-2452 at `plugin_host.audit_outbox` (schema in §3). Outbox survives process restart. A dedicated worker drains the outbox in FIFO order per (tenant_id, plugin_id) shard.

4. **MUST** compose an idempotency key per DEC-2453: `key = SHA256_HEX(kind || "\x1f" || tenant_id || "\x1f" || subject_id || "\x1f" || trace_id || "\x1f" || event_timestamp_ms)`. The key is sent to memory as the `Idempotency-Key` HTTP header. memory dedups: same key + same body → 200 with previously-issued seq; same key + different body → 409 (must not happen if implementation correct, but defends against bugs).

5. **MUST** implement retry per DEC-2454: exponential backoff `[250ms, 1s, 4s, 16s, 64s, 256s, 1024s, 4096s, ...]`; up to 24 hours total. After 24h, mark row `failed` and emit OTel alert (TASK-OBS-007 picks up).

6. **MUST** scrub user data from audit bodies per DEC-2455. The audit row body MUST include exactly these fields (and nothing else):
- `plugin_id`, `plugin_version` — from manifest
- `tenant_id`, `subject_id` — from JWT
- `trace_id` — request trace identifier
- `outcome` — "success" | one of 4 error classes from TASK-PLUGIN-002 clause 7
- For `plugin.invoked` only: `tool_name` (SEP-986 name; not the tool's input/output)
- For `plugin.scope_denied` only: `missing_scopes` (array of scope strings)
- For `plugin.installed` only: `granted_scopes` (array)
- `duration_ms` for `plugin.invoked` The body MUST NOT contain: tool input arguments, tool output content, workflow inputs, audit row contents read by the plugin, file paths or filenames the plugin touched.

7. **MUST** emit an OTel span per DEC-2456 for every audit emission, both success + failure. Span attributes: `cyberos.audit_kind`, `cyberos.idempotency_key`, `cyberos.outcome` (success / retry_queued / failed), `cyberos.tenant_id`, `cyberos.trace_id`, `cyberos.duration_ms`. Failures additionally tag `error.kind` (network / 4xx / 5xx).

8. **MUST** acquire memory's `actor_id` from the AUTH JWT's `sub` claim. Audit rows MUST be attributed to the authenticated user, not to the plugin itself. (The plugin appears in the body via `plugin_id`.)

9. **MUST** order audit rows so that `plugin.installed` is emitted BEFORE the first `plugin.invoked` for that grant. The emitter blocks the first invocation until install audit is acknowledged or queued.

10. **MUST** preserve audit emission across plugin version upgrades. When a manifest with same `id` but higher `version` installs, emit `plugin.updated` with old + new version in body; do NOT emit `plugin.uninstalled` + `plugin.installed`.

11. **MUST** retry transient failures (HTTP 5xx, connection refused, timeout) but NOT permanent failures (4xx other than 429). 4xx non-429 are programming errors and MUST be surfaced via OTel for human investigation.

12. **MUST NOT** drop audit rows on any failure mode per DEC-2451. The only sanctioned drop is after 24h retry exhaustion, and that requires an OTel alert + a failed-row in the outbox for forensics.

13. **MUST NOT** include user data in audit body per clause 6 + DEC-2455.

14. **MUST NOT** retry on 4xx non-429 — those are programming errors. Retrying a malformed request indefinitely is waste.

---

## §2 — Why this design

**Why 6 audit kinds, locked (DEC-2450)?** Each represents a distinct lifecycle moment that compliance frameworks (SOC 2 CC6.1, ISO 27001 A.9.2) require visibility into. Locking the set means downstream memory consumers (dashboards, alerting rules, exports) can rely on the schema. Future additions need successor task.

**Why audit-then-respond, not audit-after-respond (DEC-2451, clause 2)?** Audit-after means a process crash between response and audit emission loses the row. Audit-then-respond means a crash loses the response, but the audit is queued — client retries safely (idempotent). For an audit-chained product, missing audits are worse than missing responses.

**Why Postgres-backed outbox (DEC-2452, clause 3)?** In-memory queue loses on process restart. Filesystem queue requires its own atomicity model. Postgres is already in the bridge's stack (TASK-PLUGIN-002 task table); reusing it is free.

**Why hashed idempotency key (DEC-2453, clause 4)?** memory-side dedup requires a stable key for the same event. Composing from (kind, tenant, subject, trace, ts_ms) makes the key globally unique while remaining deterministic — same event produces same hash whether emitted on first attempt or 100th retry.

**Why exponential backoff up to 24h (DEC-2454, clause 5)?** Faster gives up too early — extended memory outage (4h+ during a maintenance window) would discard real rows. Slower wastes outbox space. 24h is the SOC 2 audit-completeness window most operators target.

**Why aggressive body scrubbing (DEC-2455, clause 6)?** Audit chain is read by tenant admins, compliance reviewers, and (post-export) by GDPR DSAR fulfillers. Including tool input/output would leak end-user content into the audit chain. Strategy §2 "audit-chained" requires audit be SAFE to read, not just present.

**Why required OTel emission on emission failure (DEC-2456, clause 7)?** Audit outages are the most dangerous failure — they're silent. OTel emission ensures TASK-OBS-007 + the dashboards know a tenant's audit chain is degraded. Without it, memory-write failures hide for hours.

**Why subject_id, not plugin_id, as actor (clause 8)?** Audit rows are attributed to the human/service that took the action. The plugin is the conduit, not the actor. This matches Strategy §2 model: every action traceable to a human; plugin context is metadata.

**Why install audit before first invocation (clause 9)?** Without ordering, an admin auditing a grant sees `plugin.invoked` before `plugin.installed` — confusing. Forcing the order makes the audit chain narratively coherent.

**Why plugin.updated instead of uninstall+install (clause 10)?** Upgrades are common and benign. Treating them as uninstall+install would: (a) churn the audit chain; (b) lose the version history in a single row; (c) double the rows. plugin.updated captures the same information cleanly.

**Why don't retry 4xx non-429 (clause 11, 14)?** 4xx (other than rate-limit 429) means the request is wrong. Retrying produces identical wrongness. The right response is to surface the failure to a human via OTel and fix the bug.

**Why duration_ms only on invoked (clause 6)?** Other kinds are instantaneous events. Invocation is the only kind that has measurable duration. Keep body minimal for the rest.

---

## §3 — API contract

### Postgres outbox schema

```sql
-- migrations/0002_audit_outbox.sql
CREATE TABLE plugin_host.audit_outbox (
  outbox_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  plugin_id TEXT NOT NULL,
  kind TEXT NOT NULL CHECK (kind IN (
    'plugin.installed','plugin.updated','plugin.uninstalled',
    'plugin.invoked','plugin.auth_refreshed','plugin.scope_denied'
  )),
  idempotency_key TEXT NOT NULL UNIQUE,
  body JSONB NOT NULL,
  status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','acked','failed')),
  next_retry_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  retry_count INT NOT NULL DEFAULT 0,
  last_error TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  acked_at TIMESTAMPTZ,
  failed_at TIMESTAMPTZ,
  trace_id CHAR(32) NOT NULL
);
ALTER TABLE plugin_host.audit_outbox ENABLE ROW LEVEL SECURITY;
CREATE POLICY outbox_rls ON plugin_host.audit_outbox
  USING (tenant_id = current_setting('auth.tenant_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
CREATE INDEX ON plugin_host.audit_outbox (status, next_retry_at) WHERE status = 'pending';
CREATE INDEX ON plugin_host.audit_outbox (tenant_id, plugin_id, created_at DESC);
```

### Rust emitter trait

```rust
// services/plugin-host/src/audit/mod.rs
#[async_trait]
pub trait AuditEmitter: Send + Sync {
    async fn emit(&self, event: AuditEvent) -> Result<AuditAck, AuditError>;
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditEvent {
    pub kind: AuditKind,
    pub tenant_id: Uuid,
    pub subject_id: Uuid,
    pub plugin_id: String,
    pub plugin_version: String,
    pub trace_id: String,
    pub timestamp_ms: i64,
    pub body_specific: AuditBodySpecific,  // enum per kind
}

#[derive(Debug, Clone, Serialize)]
pub enum AuditKind {
    Installed, Updated, Uninstalled, Invoked, AuthRefreshed, ScopeDenied,
}
```

### Idempotency key composer

```rust
// services/plugin-host/src/audit/idempotency.rs
pub fn compose_key(e: &AuditEvent) -> String {
    let mut h = Sha256::new();
    h.update(e.kind.as_str().as_bytes());
    h.update(b"\x1f");
    h.update(e.tenant_id.as_bytes());
    h.update(b"\x1f");
    h.update(e.subject_id.as_bytes());
    h.update(b"\x1f");
    h.update(e.trace_id.as_bytes());
    h.update(b"\x1f");
    h.update(&e.timestamp_ms.to_be_bytes());
    hex::encode(h.finalize())
}
```

### Retry backoff schedule

```rust
// services/plugin-host/src/audit/retry.rs
pub const BACKOFF_SECONDS: &[u64] = &[
    0,        // first attempt: immediate
    1,        // retry 1: +1s after first
    4,        // retry 2: +4s
    16,       // ...
    64,
    256,
    1024,
    4096,
    16384,    // retry 8: ~4.5h
    65536,    // retry 9: ~18h
    86400,    // retry 10: 24h cap
];
// After 24h total → status='failed'; alert.
```

---

## §4 — Acceptance criteria

1. **plugin.invoked row emitted on tool call success** — call tool, then query memory for kind='plugin.invoked' → row exists.
2. **plugin.installed row emitted before first invoked** — DB query ordered by seq: installed.seq < invoked.seq.
3. **plugin.updated emitted on version bump** — install v1.0.0 then v1.1.0; check single plugin.updated row, no extra installed/uninstalled.
4. **plugin.uninstalled emitted on revoke** — revoke + remove grant; check row.
5. **plugin.auth_refreshed emitted on refresh** — refresh token; check row.
6. **plugin.scope_denied emitted on denial** — call without scope; check row.
7. **Idempotency key dedups duplicate emit** — emit same event twice; memory returns same seq both times; one outbox row.
8. **Outbox persists across restart** — kill bridge, restart, queued rows still in outbox table; worker drains.
9. **Retry backoff respects schedule** — fail memory for 5 minutes; observe retries at 0/1/4/16/64/256s offsets.
10. **24h failure marks status='failed'** — fail memory persistently; after 24h simulated, row status='failed'.
11. **Body scrubbing — no tool input** — invoke tool with sensitive input; body MUST NOT contain input.
12. **Body scrubbing — no tool output** — same; body MUST NOT contain output.
13. **Body has all 6 required fields for invoked** — plugin_id, plugin_version, tenant_id, subject_id, trace_id, outcome, tool_name, duration_ms.
14. **OTel span emitted on success** — find span with cyberos.audit_kind=plugin.invoked + outcome=success.
15. **OTel span emitted on retry-queued** — fail memory; find span outcome=retry_queued.
16. **OTel span emitted on 24h failure** — find span outcome=failed + error.kind set.
17. **4xx non-429 not retried** — emit with mocked 400; retry_count stays 0; status='failed' immediately.
18. **429 IS retried** — emit with mocked 429; retry_count grows.
19. **RLS prevents cross-tenant outbox read** — tenant A cannot SELECT tenant B's rows.
20. **actor_id is subject_id from JWT** — audit row's actor_id matches JWT.sub.
21. **Audit-then-respond ordering** — tool call response not returned until audit acked or queued.
22. **Concurrent emit of same idempotency key** — two simultaneous emits with same key collapse to one outbox row.

---

## §5 — Verification

```rust
// services/plugin-host/tests/audit_emission_happy_path_test.rs
#[tokio::test]
async fn invoked_row_emitted_on_tool_call() {
    let ctx = TestContext::with_memory_mock().await;
    ctx.bridge.tools_call("cyberos.cuo.list_personas", json!({})).await;
    let rows = ctx.memory.fetch_rows(kind="plugin.invoked").await;
    assert_eq!(rows.len(), 1);
    let body = &rows[0]["body"];
    assert_eq!(body["plugin_id"], "cyberos");
    assert_eq!(body["tool_name"], "cyberos.cuo.list_personas");
    assert_eq!(body["outcome"], "success");
    assert!(body["duration_ms"].is_number());
    assert!(body["trace_id"].is_string());
}
```

```rust
// services/plugin-host/tests/audit_emission_outbox_durability_test.rs
#[tokio::test]
async fn outbox_survives_restart() {
    let ctx = TestContext::with_memory_unavailable().await;
    ctx.bridge.tools_call("cyberos.cuo.list_personas", json!({})).await;
    let outbox = ctx.db.fetch_all("SELECT * FROM plugin_host.audit_outbox WHERE status='pending'").await;
    assert_eq!(outbox.len(), 1);
    ctx.bridge.shutdown().await;
    let ctx2 = TestContext::resume(&ctx).await;
    ctx2.memory.recover();
    ctx2.wait_for_drain(Duration::from_secs(5)).await;
    let pending = ctx2.db.fetch_all("SELECT * FROM plugin_host.audit_outbox WHERE status='pending'").await;
    assert_eq!(pending.len(), 0);
}
```

```rust
// services/plugin-host/tests/audit_emission_idempotency_test.rs
#[tokio::test]
async fn duplicate_emit_collapses_to_one_row() {
    let ctx = TestContext::with_memory_mock().await;
    let event = make_event_at("2026-05-19T10:00:00Z");
    ctx.bridge.audit.emit(event.clone()).await.unwrap();
    ctx.bridge.audit.emit(event.clone()).await.unwrap();
    let count = ctx.db.fetch_one("SELECT count(*) FROM plugin_host.audit_outbox WHERE idempotency_key=$1", &[&key_of(&event)]).await;
    assert_eq!(count, 1);
}
```

```rust
// services/plugin-host/tests/audit_emission_no_user_data_leak_test.rs
#[tokio::test]
async fn body_does_not_contain_tool_input() {
    let ctx = TestContext::with_memory_mock().await;
    ctx.bridge.tools_call("cyberos.cuo.execute_workflow", json!({
        "persona": "chief-technology-officer",
        "workflow": "adr-quick-capture",
        "inputs": {"title": "SECRET-PROJECT-X", "context": "extremely confidential"}
    })).await;
    let rows = ctx.memory.fetch_rows(kind="plugin.invoked").await;
    let body_json = serde_json::to_string(&rows[0]["body"]).unwrap();
    assert!(!body_json.contains("SECRET-PROJECT-X"), "tool input leaked into audit body");
    assert!(!body_json.contains("extremely confidential"), "tool input leaked");
}
```

```rust
// services/plugin-host/tests/audit_emission_retry_backoff_test.rs
#[tokio::test]
async fn retry_follows_exponential_backoff() {
    let ctx = TestContext::with_failing_memory().await;
    ctx.bridge.tools_call("cyberos.cuo.list_personas", json!({})).await;
    let attempt_times: Vec<i64> = ctx.memory.recorded_attempts();
    let intervals: Vec<i64> = attempt_times.windows(2).map(|w| w[1] - w[0]).collect();
    // First 5 intervals should approximate [1s, 4s, 16s, 64s, 256s] (within ±300ms jitter)
    assert!(approx_eq(intervals[0], 1000, 300));
    assert!(approx_eq(intervals[1], 4000, 300));
    assert!(approx_eq(intervals[2], 16000, 600));
}
```

---

## §6 — Implementation skeleton

(Schema + traits + helpers in §3 are the skeleton. Audit module is ~500 Rust lines.)

---

## §7 — Dependencies

- **Upstream:** TASK-PLUGIN-002 (bridge calls emitter on each tool call); TASK-MEMORY-101 (Layer-2 ingest pipeline, shipped).
- **Downstream:** TASK-PLUGIN-008 (marketplace audit chain — publish events use the same emitter for plugin.published).
- **Cross-module:** TASK-MEMORY-104 (memory HTTP REST, shipped); TASK-OBS-001 (OTel collector for clause 7 spans); TASK-OBS-007 (alertmanager picks up 24h-failure rows per DEC-2454).

---

## §8 — Example payloads

(See §3 for schema. Sample bodies in TASK-PLUGIN-005 §8.)

### `plugin.invoked` body

```json
{
  "plugin_id": "cyberos",
  "plugin_version": "1.0.0",
  "tool_name": "cyberos.cuo.execute_workflow",
  "outcome": "success",
  "duration_ms": 47230,
  "trace_id": "01HX...",
  "tenant_id": "11111111-...",
  "subject_id": "22222222-..."
}
```

### `plugin.updated` body

```json
{
  "plugin_id": "cyberos",
  "plugin_version": "1.0.1",
  "previous_version": "1.0.0",
  "outcome": "success",
  "trace_id": "01HX...",
  "tenant_id": "...",
  "subject_id": "..."
}
```

### Outbox row in retry-queued state

```json
{
  "outbox_id": "...",
  "kind": "plugin.invoked",
  "idempotency_key": "a1b2c3...",
  "status": "pending",
  "retry_count": 3,
  "next_retry_at": "2026-05-19T10:01:04Z",
  "last_error": "connection refused: memory.cyberskill.world:443",
  "body": { ... as above ... }
}
```

---

## §9 — Open questions

All resolved.

- ~~Should outbox have a max size?~~ → No hard cap; rely on the 24h retry timeout to bound queue size. Hard cap would force dropping rows, violating DEC-2451.
- ~~Should we attempt to emit one final row on graceful shutdown?~~ → No; on shutdown, in-flight emissions complete or queue to outbox via existing path. Special-cased "shutdown emit" complicates correctness.
- ~~Should rate-limit 429 reset the retry counter?~~ → No; counts toward 24h cap. 429 storms are operational issues that need investigation, not silent re-queuing.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| memory HTTP 5xx | client response code | queue to outbox | Worker retries with backoff |
| memory HTTP 4xx (non-429) | client response code | queue + mark failed immediately | Operator inspects via OTel; fix bug |
| memory HTTP 429 (rate limit) | client response code | queue with backoff | Inherent — workers retry |
| Network partition to memory | reqwest connect error | queue to outbox | Worker retries on reconnect |
| Outbox DB unavailable | sqlx error | tool call fails (upstream_unavailable) | Bridge alarms via OTel; admin recovers DB |
| Idempotency key collision (different bodies) | memory 409 | outbox row marked failed; alert | Programming bug — investigate |
| Process restart with in-flight emit | startup detects pending outbox rows | drain queue on boot | Inherent |
| 24h retry exhaustion | retry_count reaches schedule end | mark failed; alert via OTel | Operator investigates memory outage |
| User-data leak in body | unit test fails | CI blocks merge | Author scrubs |
| OTel exporter down | exporter buffer fills | spans dropped, emit continues | OTel recovers; metric gap acceptable |
| Cross-tenant outbox read | RLS check fails | 0 rows returned | Inherent (RLS) |
| Concurrent retry of same row | row UPDATE WHERE status='pending' AND outbox_id=X | second updater gets 0 rows | Inherent — first wins |
| Plugin manifest declares wrong audit endpoint | manifest validation at install | install fails | Author fixes manifest URL |
| Bridge unable to compose idempotency key (missing field) | composer assertion | tool call fails internal_error | Programming bug — investigate |
| Clock skew between bridge + memory | timestamp mismatch | dedup may fail | NTP; memory tolerates ±30s |
| Audit row body size > memory limit | response 413 | mark failed | Body is bounded by spec; not expected; investigate if hit |

---

## §11 — Implementation notes

- §11.1 **Audit-then-respond implementation.** `tools/call.rs` calls `emit_audit_blocking(event).await?` BEFORE sending the response frame. `emit_audit_blocking` returns Ok once either memory acks (200) or outbox enqueue succeeds (Ok(QueuedForRetry)).

- §11.2 **Worker drain pattern.** A dedicated tokio task `audit::retry::worker_loop` polls `SELECT * FROM audit_outbox WHERE status='pending' AND next_retry_at <= now() ORDER BY next_retry_at LIMIT 100` every second. For each row, attempt memory emit; on success update status='acked', acked_at=now(); on failure compute next_retry_at, increment retry_count.

- §11.3 **Backoff jitter.** Add ±20% random jitter to each next_retry_at delta to avoid thundering herd if many emits failed at the same wall-clock minute.

- §11.4 **Why idempotency key uses timestamp_ms.** Without timestamp, retrying the SAME logical event would produce the same key — fine. But two distinct events with identical (kind, tenant, subject, trace) would collide. Adding timestamp_ms (millisecond resolution) makes the key unique while still deterministic.

- §11.5 **RLS on outbox.** Worker process operates with `cyberos_outbox_worker` role that has elevated SELECT/UPDATE on the outbox table but no other tables. Tenant-scoped reads use the normal RLS-respecting role for /admin endpoints.

- §11.6 **OTel attribute redaction.** Idempotency key is hash of identity-sensitive fields. OK to log directly — it's not a secret. Trace_id is also OK. BUT body content is NOT logged in OTel — only kind + outcome + retry_count.

- §11.7 **Worker concurrency control.** Single worker per bridge process. With multiple bridge instances behind a load balancer, each works on its own outbox slice (modulo by tenant_id hash). Postgres FOR UPDATE SKIP LOCKED prevents two workers grabbing the same row.

- §11.8 **Graceful shutdown.** On SIGTERM: stop accepting new emit calls; wait up to 30s for in-flight emits to complete; flush outbox writes; exit. Remaining in-flight emits become outbox rows.

- §11.9 **Schema migration.** `0002_audit_outbox.sql` runs ahead of bridge startup. Bridge startup verifies the schema exists; failure logs and exits non-zero (Fargate restarts; not a self-heal loop).

- §11.10 **Manual replay tool.** `cyberos-plugin-host audit-replay --outbox-id <uuid>` reruns a specific row. Used by operators for one-off recovery after memory bug fixes.

---

*End of TASK-PLUGIN-006 spec.*
