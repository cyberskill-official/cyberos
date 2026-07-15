---
id: TASK-TEN-004
title: "4-axis metering — seats · API · AI tokens · storage (memory audit per metric event)"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-05-16T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: TEN
priority: p0
status: ready_to_implement
accepted_at: 2026-05-16
accepted_by: Stephen Cheng
verify: T
phase: P2
milestone: P2 · billing-substrate
slice: 1
owner: Stephen Cheng
created: 2026-05-16
shipped: null
memory_chain_hash: null
related_tasks: [TASK-AI-001, TASK-TEN-001, TASK-TEN-002, TASK-TEN-003, TASK-DOC-001, TASK-MEMORY-111]
depends_on: [TASK-AI-001, TASK-TEN-001, TASK-AUTH-003, TASK-MEMORY-111]
blocks: [TASK-TEN-003]

source_pages:
  - website/docs/modules/ten.html#metering
source_decisions:
  - DEC-700 2026-05-16 — 4 axes locked (no per-message, no per-event; bill on what costs us money)
  - DEC-701 2026-05-16 — memory audit emitted per metric event (not aggregated) so disputes resolve to a row
  - DEC-702 2026-05-16 — Per-event row is append-only via SQL grant (REVOKE UPDATE/DELETE)
  - DEC-703 2026-05-16 — Aggregation runs at end-of-billing-period only; intra-period queries hit materialized view
  - DEC-704 2026-05-16 — Closed 4-value axis enum; new axis requires schema migration + DEC-XXX entry
  - DEC-705 2026-05-16 — Seats metered as point-in-time snapshot at billing close (not entry/exit events)
  - DEC-706 2026-05-16 — Storage metered as point-in-time bytes-used at billing close (S3 inventory + Postgres byte-count)
  - DEC-707 2026-05-16 — API calls metered per request (TASK-AUTH-003 RLS-aware hot path)
  - DEC-708 2026-05-16 — AI tokens metered per provider response (input + output token counts from cost_ledger)
  - DEC-709 2026-05-16 — Backpressure on metering write failure — falls back to local WAL queue + retry with bounded buffer
  - DEC-710 2026-05-16 — Per-tenant `metering_overage_policy` (block | warn | allow) with CFO-only mutation gate
  - DEC-711 2026-05-16 — Refund pathway via correction_to (TASK-TIME-001 pattern) — never UPDATE/DELETE
  - DEC-712 2026-05-16 — Dual-write to Postgres + memory audit chain — Postgres row is authoritative for billing, memory is tamper-evident
  - DEC-713 2026-05-16 — Real-time view via `metering_current_period` materialized view refreshed every 5 minutes
  - DEC-714 2026-05-16 — PII-free metric rows (only counts + ids); memory audit chain through TASK-MEMORY-111 scrubs reason text
  - DEC-715 2026-05-16 — Idempotent metric_event via `idempotency_key` UNIQUE constraint (24h retention)

build_envelope:
  language: rust 1.81
  service: cyberos/services/metering/
  new_files:
    - services/metering/src/lib.rs
    - services/metering/src/recorder.rs
    - services/metering/src/aggregator.rs
    - services/metering/src/axes/seats.rs
    - services/metering/src/axes/api_calls.rs
    - services/metering/src/axes/ai_tokens.rs
    - services/metering/src/axes/storage.rs
    - services/metering/src/policy.rs
    - services/metering/src/handlers/usage_query.rs
    - services/metering/src/handlers/period_close.rs
    - services/metering/src/wal_queue.rs
    - services/metering/migrations/0001_metering_events.sql
    - services/metering/migrations/0002_metering_holds_index.sql
    - services/metering/migrations/0003_metering_aggregates_view.sql
    - services/metering/tests/api_metering_test.rs
    - services/metering/tests/ai_metering_test.rs
    - services/metering/tests/seats_snapshot_test.rs
    - services/metering/tests/storage_snapshot_test.rs
    - services/metering/tests/append_only_test.rs
    - services/metering/tests/overage_policy_test.rs
    - services/metering/tests/wal_replay_test.rs
  modified_files:
    - services/ai-gateway/src/cost_ledger.rs (emit ai_tokens metric on postcall reconcile)
    - services/auth/src/middleware.rs (emit api_calls metric per RLS-scoped request)
  allowed_tools:
    - file_read: services/metering/**
    - file_write: services/metering/{src,tests,migrations}/**
    - bash: cargo test -p cyberos-metering
    - bash: cargo sqlx migrate run
    - memory: write memories/decisions/metering-events/* via canonical Writer (NOT directly)
  disallowed_tools:
    - direct INSERT to metering_events from outside metering service
    - UPDATE/DELETE of any metering_events row (append-only via SQL grant)
    - skip memory audit emission (defense-in-depth pairing)

effort_hours: 8
subtasks:
  - "0.5h: sqlx migration for metering_events table + REVOKE grants"
  - "1.0h: recorder API with 4 axis variants + idempotency check"
  - "1.0h: API-call middleware integration (auth hot path)"
  - "1.0h: AI-token integration (cost_ledger postcall hook)"
  - "1.0h: seats + storage end-of-period snapshot jobs"
  - "1.0h: overage policy enforcement (block/warn/allow)"
  - "1.0h: WAL queue with bounded buffer + replay"
  - "0.5h: materialized view + 5-min refresh job"
  - "1.0h: integration tests (real Postgres + tenant fixture)"
risk_if_skipped: "Without per-event metering, billing reconciles to provider invoices alone — tenant disputes have no ledger to point at, refunds require ad-hoc CSV exports, and the 'audit-before-action' invariant breaks because cost moves without a chain row. Worst-case: a tenant claims they were over-billed for AI tokens, we have no per-call record, dispute escalates to credit issued without verification."
---

## §1 — Description (BCP-14 normative)

The Metering service **MUST** emit one append-only `metering_events` row per quantifiable resource consumption event along exactly four closed axes — **seats, api_calls, ai_tokens, storage_bytes** — and **MUST** simultaneously emit one memory audit chain row per event for tamper-evident reconciliation.

1. **MUST** define the closed 4-value Postgres enum `metering_axis = ('seats', 'api_calls', 'ai_tokens', 'storage_bytes')` with a CI cardinality test (`SELECT count(*) FROM pg_enum WHERE enumtypid = 'metering_axis'::regtype` equals 4). Adding a new axis requires a schema migration + a DEC-XXX decision entry (DEC-704).

2. **MUST** persist every metric event to the `metering_events` Postgres table as an append-only row. The table is created with `REVOKE UPDATE, DELETE FROM cyberos_app`. Only the privileged `metering_writer` role holds INSERT privilege; only `metering_reader` holds SELECT. The reversal pathway for refunds is via the TASK-TIME-001 `correction_to` self-FK pattern — never an UPDATE/DELETE (DEC-702 + DEC-711).

3. **MUST** enforce idempotent recording via a `UNIQUE (tenant_id, axis, idempotency_key)` constraint on `metering_events`. The 24-hour retention window for idempotency keys is enforced by a partial index. Duplicate inserts MUST return success (idempotent semantics) without emitting a second memory audit row (DEC-715).

4. **MUST** emit exactly one memory audit chain row per metric event before the Postgres COMMIT. The audit row carries `{axis, tenant_id, quantity, unit, idempotency_key, occurred_at, source_service, postgres_event_id}` — no free-text descriptions, no PII. The Postgres row id and the memory chain hash MUST be cross-linked (event row stores `memory_chain_hash`, memory row stores `postgres_event_id`). Mismatch detected at close-period reconciliation triggers a sev-1 audit row (DEC-712 + DEC-714).

5. **MUST** route every memory audit row through the TASK-MEMORY-111 pre-ingest PII detection layer. The metering payload is structured (no free text), so the detector returns "clean" 100% of the time in steady state; any non-clean result MUST be treated as a sev-1 ingest defect (the metering writer is the source of "clean payloads only" — if it emits PII, that is a code bug).

6. **MUST** integrate API-call metering at the auth middleware hot path (TASK-AUTH-003 RLS-aware request scope). Every successful RLS-scoped HTTP request emits one `api_calls` event with `quantity = 1` and `unit = "request"`. Failed-auth requests do NOT emit (rejected at auth, not billed). The middleware writes via the WAL queue (§1 #14) on the response path — never on the request path — so a metering writer outage cannot impact request latency (≤ 200µs p99 overhead budget).

7. **MUST** integrate AI-token metering at the TASK-AI-001 cost-ledger postcall reconcile hook. When `cost_ledger::postcall_reconcile()` resolves the actual provider response, it emits one `ai_tokens` event with `quantity = input_tokens + output_tokens`, `unit = "token"`, and `extra = {model_alias, provider, input_tokens, output_tokens}`. The reconcile path is already on the cost-ledger code path, so there is no additional hot-path overhead.

8. **MUST** snapshot seats at billing-period-close (last calendar day of the billing month at 23:59:59 UTC of the tenant's billing timezone, per DEC-705). The snapshot job queries `SELECT COUNT(*) FROM members WHERE tenant_id = $1 AND active_at IS NOT NULL AND deactivated_at IS NULL` and writes ONE `seats` event with `quantity = N`, `unit = "seat"`. Seats are NOT metered as join/leave events — the closing snapshot is authoritative. If a member is added and removed within the period, neither shows up in metering.

9. **MUST** snapshot storage at billing-period-close via two sub-counts (DEC-706):
   - **S3 bytes**: queries the S3 inventory report (CSV-format daily inventory delivered to `s3://cyberos-inventory/<tenant>/<date>.csv`) and sums `Size` column. The inventory is delivered with up to 24h lag, so the close job runs at +24h after period end.
   - **Postgres bytes**: queries `pg_total_relation_size` summed across the tenant's RLS-scoped tables. The query joins `pg_class` to `cyberos.tenants` to derive size per tenant.
   - Emits ONE `storage_bytes` event with `quantity = s3_bytes + postgres_bytes`, `unit = "byte"`.

10. **MUST** define a closed 3-value `metering_overage_policy` enum (`block`, `warn`, `allow`). The per-tenant policy lives in `tenants.metering_overage_policy_yaml` and defaults to `warn`. Mutation requires `cfo` role per TASK-AUTH-101 + a sev-2 memory audit row with a non-empty reason (DEC-710). The CI cardinality test asserts exactly 3 enum values.

11. **MUST** enforce overage policy at the API-call middleware:
    - `block`: when `current_period_api_calls + 1 > monthly_cap`, return `402 PAYMENT_REQUIRED` with body `{error: "overage_blocked", axis: "api_calls", current, cap}`. No `metering_events` row is emitted (the call is rejected upstream).
    - `warn`: when crossing `monthly_cap × warn_threshold` (default 0.80), emit a `metering.warn_threshold_crossed` memory audit row at sev-2. The metric event proceeds normally.
    - `allow`: no enforcement; metric event proceeds; CFO reviews at period close.

12. **MUST** define a `metering_holds` table parallel to TASK-AI-001's `cost_ledger_holds` for high-value operations (e.g., a bulk API ingest claims a 10,000-call hold before issuing). The hold uses the same 60s TTL + idempotency contract as the cost-ledger hold and emits a `metering.hold_claimed` memory audit row.

13. **MUST** expose a `GET /v1/usage` REST endpoint that returns the current billing period's usage along all four axes for the caller's tenant. The handler hits the `metering_current_period` materialized view (refreshed every 5 minutes per DEC-713), not the raw `metering_events` table. Response shape:
    ```json
    {
      "tenant_id": "ten_abc",
      "period_start": "2026-05-01T00:00:00Z",
      "period_end": "2026-05-31T23:59:59Z",
      "seats": 12,
      "api_calls": 4823901,
      "ai_tokens": 12834729,
      "storage_bytes": 38291723123,
      "overage_policy": "warn",
      "warn_thresholds_crossed": ["api_calls"]
    }
    ```

14. **MUST** route every metering write through a bounded WAL queue (DEC-709) — 100,000-event in-memory ring buffer per writer. On Postgres outage, events queue in memory; on memory pressure (queue > 90%), the writer logs a sev-1 memory audit row and rejects new events (back-pressure to caller). On Postgres recovery, the queue drains FIFO. The WAL queue is NOT persistent across writer process restart — restart loses queued events. This is acceptable because Postgres outages affect all of CyberOS, not just metering, and the missing events are derivable from the source-of-truth services (auth middleware logs, cost_ledger holds) within the period-close window.

15. **MUST** route every memory audit emission through the same WAL queue. Postgres COMMIT and memory audit emission are dual-write — both succeed or the WAL replay retries. Until both succeed, the metering event is NOT durable. The reconciliation job at period close compares Postgres row IDs against memory chain rows and surfaces any divergence as a sev-1 audit row.

16. **MUST** refuse any cross-tenant metering query. The `metering_events` table is RLS-scoped on `tenant_id` per TASK-AUTH-003; the materialized view inherits RLS via SECURITY DEFINER + tenant-context check. Cross-tenant aggregation is reserved for CyberOS-internal `bookkeeper` role (used by the billing service alone).

17. **MUST** expose `POST /v1/usage/correction` for the CFO role (TASK-AUTH-101) to issue a refund-style correction. The handler INSERTs a new `metering_events` row with `quantity = -N` and `correction_to = <original_event_id>`. The original row is never UPDATED or DELETED (DEC-711). Reason is required (≥ 10 chars) and surfaces in a sev-2 memory audit row.

18. **MUST** define a closed 3-value `metering_event_state` enum (`active`, `corrected`, `superseded`). On insertion, state = `active`. A correction_to event flips the parent to `superseded` via a trigger. A double-correction (correction of a correction) sets state = `corrected` on the intermediate row. Aggregation views filter on `state = 'active'` only.

19. **MUST** validate `quantity` per axis at the recorder API:
    - `seats`: integer ≥ 0, ≤ 100,000.
    - `api_calls`: integer ≥ 1 (corrections are negative; recorded as opposite-sign row, not negative quantity at recorder), ≤ 1,000,000 per single event (bulk write).
    - `ai_tokens`: integer ≥ 1, ≤ 10,000,000 per single event.
    - `storage_bytes`: integer ≥ 0, ≤ 10 TiB (10^13).
    Out-of-range MUST return `400 BAD_REQUEST` with `{error: "metering_quantity_out_of_range", axis, quantity, max}`.

20. **MUST** define a `metering_current_period` materialized view that aggregates `SUM(quantity) FILTER (WHERE state = 'active')` by `(tenant_id, axis, period_start, period_end)`. Refresh schedule: every 5 minutes via pg_cron. The refresh is `CONCURRENTLY` to avoid blocking read queries. The view carries a `last_refreshed_at` column for staleness detection (DEC-713).

21. **MUST** emit one of 7 closed memory audit kinds per metering event:
    - `metering.event_recorded` (sev-3, per metric event)
    - `metering.warn_threshold_crossed` (sev-2, per crossing)
    - `metering.overage_blocked` (sev-2, per blocked request)
    - `metering.correction_issued` (sev-2, per refund)
    - `metering.policy_changed` (sev-2, per CFO policy update)
    - `metering.reconciliation_divergence` (sev-1, per close-job divergence)
    - `metering.wal_queue_overflow` (sev-1, per queue-full event)

22. **MUST** expose `POST /v1/metering/period/close` as an ops-only handler (no tenant access; `bookkeeper` role only). The handler runs the seats + storage snapshot jobs for one tenant + emits the final aggregate memory audit row + freezes the period (writes to `metering_periods.frozen_at`). After freeze, no `metering_events` row may carry a timestamp inside the frozen period — the recorder rejects with `409 PERIOD_FROZEN`. Frozen periods are reopened only via a sev-1 ops manual action (no API path).

23. **MUST** validate `tenant_id` against the active tenant set on every recorder API call. A metering event for a terminated tenant (TASK-TEN-104 terminal state) MUST be rejected with `404 TENANT_NOT_FOUND`. The recorder caches the active-tenant set with a 60s TTL (mirrors TASK-AUTH-109 + TASK-AUTH-105 pattern) to keep hot-path overhead bounded.

24. **MUST** maintain an `aggregator_state` table that tracks `(tenant_id, axis, period_start, period_end, status, frozen_at, last_aggregated_seq)`. The seq counter is incremented on every materialized view refresh; the period-close handler verifies that `last_aggregated_seq` matches the latest `metering_events.seq` for the period before freezing — defense-in-depth against the materialized view falling behind unbounded.

25. **MUST** define a `metering_event_dispute_log` append-only table for billing-team-driven dispute investigation. Each row links to one or more `metering_events.id` + carries a `resolution_notes` text field (≤ 4 KiB) + a closed `resolution_status` enum (`pending`, `confirmed_correct`, `corrected_via_refund`, `wontfix`). Only the `billing_disputes` role can INSERT (per TASK-AUTH-101). Disputes are never deleted — closed disputes remain queryable for the 10-year retention window (TASK-DOC-001 alignment).

26. **MUST** scrub all reason-bearing audit text through TASK-MEMORY-111 pre-ingest PII detection (refund reason, dispute resolution notes, policy-change reason). Postgres holds the raw text (RLS-scoped); the memory chain holds the scrubbed version. Mismatch between Postgres raw and chain scrubbed is acceptable and expected — this is the TASK-MEMORY-111 contract.

27. **MUST** support a closed `unit` enum per axis (`seat`, `request`, `token`, `byte`) with a CHECK constraint enforcing the `(axis, unit)` pair (seats×seat, api_calls×request, ai_tokens×token, storage_bytes×byte). Mismatched pair MUST reject with `400` at recorder. The closed pairing makes downstream billing math unambiguous — no need for per-unit conversion at aggregation time.

---

## §2 — Rationale (informative — preserve all 22 paragraphs)

**§2.1  Why 4 axes and not 7+.** The original proposal carried 7 axes (seats, api_calls, ai_tokens, storage, bandwidth, function_invocations, scheduled_jobs). DEC-700 closed it at 4. The principle: bill on what costs us money, not what looks countable. Bandwidth is a derivative of api_calls + storage_bytes; function_invocations equals api_calls for the tenant's perspective; scheduled_jobs are bounded per-tenant (the platform caps them) and so are a flat cost not a metered one. The 4 axes that survived are the four where our infrastructure spend scales linearly with tenant behavior — seats drive identity-provider costs, api_calls drive compute, ai_tokens drive provider-pass-through, storage_bytes drives S3 + Postgres. Adding a fifth axis requires a DEC entry AND a real provider-bill scaling story, not a marketing wish.

**§2.2  Why append-only and not UPDATE.** DEC-702 makes `metering_events` append-only at SQL grant. The argument against UPDATE: a billing dispute that lands six months later cannot be resolved if intermediate rows have been "cleaned up". The argument for append-only: forensic accounting works on append-only ledgers; non-append-only ledgers collapse under audit pressure. The refund pathway via `correction_to` (DEC-711) preserves the original event row + records the negation as a fresh row; the parent flips to `superseded` via trigger (clause #18) but is never deleted. This is the same pattern as TASK-TIME-001 timesheet corrections — once a pattern is correct, replicate it.

**§2.3  Why memory audit per event, not per period.** DEC-701: per-event memory audit is the dispute-resolution mechanism. When a tenant says "you charged us for 5M API calls but we only made 4M", we need to point at a chain row per disputed call (or batch of calls), not a single rolled-up period total. The 7M extra calls might be a Stripe webhook retry storm we didn't bill them for; might be a legitimate bot we didn't realize was theirs; might be a CyberOS-side bug. A per-event chain row turns the question into "show us the rows you dispute" — operational, resolvable. A per-period chain row collapses to "trust our aggregator". We chose operational over compact.

**§2.4  Why dual-write to Postgres + memory and not memory-only.** Postgres is the authoritative source for billing math because (a) Stripe + our billing service speak SQL, not chain hashes; (b) Postgres + RLS gives us tenant isolation at query time without re-implementing it in chain-walking code; (c) materialized views give us sub-second `GET /v1/usage` latency. memory-only would have made every usage query a chain-walk — slow and complex. DEC-712 makes Postgres authoritative for billing + memory tamper-evident for reconciliation. The reconciliation job at period close cross-checks them; divergence is a sev-1 incident.

**§2.5  Why the API-call hot-path is response-side, not request-side.** Clause #6: the metering middleware fires after the response status is known. Two reasons. First, failed-auth requests should NOT be billed — they were rejected before the tenant got any service. Second, request-side metering puts the WAL-queue write on the request critical path; response-side puts it on the response path, where 200µs of extra latency is invisible. The cost is that a 5xx server error after auth-success still gets billed (we did spend compute on it); we accept that — it's a tiny fraction of traffic and the tenant can dispute via §1 #25.

**§2.6  Why seats and storage are snapshots, not events.** Clause #8 + #9 + DEC-705 + DEC-706. Seats churn during a billing period (joiners + leavers) but our actual cost — identity-provider seats — is what's active at the close boundary. Billing on per-day snapshots would be more accurate but creates 30 events per period per tenant, and the cost (engineering + provider charges) doesn't scale linearly with intra-period churn. Storage is similar: S3 charges on monthly average, but the inventory report is end-of-period; using end-of-period bytes is a slight under-bill in the tenant's favor (a tenant who uploaded 100 GB on day 1 and deleted 99 GB on day 30 pays for 1 GB, not the integral). We accept the under-bill — it makes math simple and tenants happy.

**§2.7  Why a closed 4-value enum and not a free string.** DEC-704 + clause #1. The CI cardinality test asserts exactly 4 values. The benefit: every consumer can switch on the enum exhaustively; every dashboard query is total without joining a dimension table; every audit row is type-checkable. The cost: adding a fifth axis is a schema migration. Given the architectural argument in §2.1 — adding axes should be rare and intentional — this is the right tradeoff.

**§2.8  Why the WAL queue is in-memory and bounded.** Clause #14 + DEC-709. Persistent (on-disk) WAL would add a fsync per event — at 10k events/sec, the disk I/O alone consumes the latency budget. In-memory WAL costs the events between writer-restart and Postgres-recovery, but at our scale (≤ 50k events/sec per writer) the restart-loss is rare and recoverable from source services. The 90% threshold for back-pressure prevents the queue from collapsing the writer process; the sev-1 audit row at overflow makes the rare loss observable.

**§2.9  Why per-tenant overage policy and not a global one.** DEC-710. Different tenants have different risk tolerances: enterprise customers want `block` (compliance — no surprise spend); startups want `allow` (don't break my product). A global policy forces one shape on all; per-tenant lets the customer choose at signup. The CFO-only gate on policy mutation prevents a malicious operator from flipping a high-risk tenant from `block` to `allow` and racking up overage; the sev-2 memory audit row makes policy changes investigatable.

**§2.10  Why the period-close handler is ops-only.** Clause #22. A tenant cannot freeze their own period — that would let them stop billing mid-period. A tenant cannot reopen a frozen period — that would let them rewrite history. Both directions are ops-only. The freeze is per-tenant per-period (not global), so one tenant's billing failures don't block another's close.

**§2.11  Why the materialized view refresh is every 5 minutes, not continuous.** DEC-713 + clause #20. Continuous refresh would CPU-bind one Postgres process per tenant + axis. 5 minutes is the lag tenants tolerate for usage dashboards (they're checking, not closing books in real time) and gives the refresh enough time to amortize across multiple events. The `last_refreshed_at` column on the view lets dashboard UIs warn if data is stale (e.g., during refresh-job outage). For "exact-now" queries (e.g., a tenant about to hit overage), the API-middleware enforcement runs on raw `metering_events` not the view — the raw path is exact.

**§2.12  Why `correction_to` and not a `void` flag.** Clause #17 + DEC-711. The `correction_to` self-FK is the TASK-TIME-001 pattern: the original row stays exactly as written; the correction is a fresh row with `quantity = -N`. A `void` flag would require UPDATE — append-only forbids that. A separate `voided_events` table would split the data — querying historical totals would need a UNION. The `correction_to` approach + state enum (clause #18) lets every analytical query filter `state = 'active'` and get the right answer.

**§2.13  Why dispute logs are append-only too.** Clause #25. A closed dispute that gets reopened in 18 months (rare but possible — regulator inquiry, internal audit) needs the full history of how it was investigated + resolved. Append-only preserves that. The closed `resolution_status` enum forces dispute outcomes into 4 well-defined buckets, making aggregation across many disputes meaningful (e.g., "what's our dispute-confirmation rate?").

**§2.14  Why PII-free metric rows.** DEC-714 + clause #4. Metering rows are structured: integers + ids + timestamps. There's no place for free-text PII. The PII detection (TASK-MEMORY-111) is applied to the reason-bearing fields (refund reason, dispute resolution notes, policy-change reason) — clause #26. Keeping metric rows themselves PII-free means we can replicate them to billing systems (Stripe metadata, internal dashboards) without re-running PII detection.

**§2.15  Why idempotency keys at 24h retention.** Clause #3 + DEC-715. The 24h window covers reasonable retry windows from upstream consumers (provider webhooks, AI-gateway retries, scheduled-job retries) without bloating the unique index forever. After 24h, the idempotency key can be reused (the upstream caller's retry budget is exhausted by then; if it still retries, the duplicate is intentional and we should re-bill). The partial index keeps the index size bounded — about 30M entries at our top-end traffic.

**§2.16  Why the `aggregator_state.last_aggregated_seq` cross-check.** Clause #24. The materialized view is a derived structure; if its refresh job lags, the period-close handler could freeze a period based on stale view data. The seq cross-check forces the close handler to ensure the view has caught up to the latest `metering_events` row for the period before freezing. Without this, an outage of the refresh job at end-of-month could result in under-billing.

**§2.17  Why a closed `unit` enum and per-axis pairing.** Clause #27. A free `unit` field would let a buggy recorder write `("api_calls", "minute")` and the billing system would happily aggregate minutes as if they were requests. The (axis, unit) CHECK constraint makes that a database rejection. The closed pairing pre-decides the semantics: api_calls is always counted in requests, ai_tokens always in tokens, storage_bytes always in bytes. Downstream billing math has zero ambiguity.

**§2.18  Why metering of API calls runs in middleware not in service code.** Clause #6. Every CyberOS service running auth middleware gets metering for free. Putting metering in service code would require every service to remember to emit; the omissions would cause underbilling. Middleware-level integration is a single integration point — wire it once at the auth crate, every service inherits it.

**§2.19  Why AI tokens are metered at postcall reconcile, not pre-call estimate.** Clause #7. TASK-AI-001 emits an estimate at pre-call (for cap-check); the actual token count is only known after the provider response. Metering on the estimate would over-bill the tenant for refused calls and over-bill for prompts that the model truncates. Postcall reconcile is the truth-bearing moment.

**§2.20  Why we cap `quantity` per single event.** Clause #19. A single event row with `quantity = 10^9` could plausibly be a buggy bulk-write (e.g., a typo turning a daily report into a yearly one). The per-axis caps catch the obvious bugs. Genuinely-large rolled-up writes (e.g., a year-end retroactive correction) should go through the correction handler with multiple rows, not one mega-row.

**§2.21  Why the dispute resolution_notes is capped at 4 KiB.** Clause #25. Dispute notes are operational reading; they need to fit on a CFO's screen + be queryable in dashboards. 4 KiB is roughly two screens of single-spaced text — enough for "here's what we found, here's the resolution" + the original tenant complaint. Longer notes can be linked to a document in TASK-DOC-001 (the document repository); the dispute log carries the document_id, not the full text.

**§2.22  Why metering events for terminated tenants are rejected at recorder.** Clause #23. A terminated tenant (TASK-TEN-104) should produce no new charges. If a stray service emits a metering event for a terminated tenant after termination (race condition, cache lag), the recorder MUST drop it — not silently, but with a 404 and a sev-2 memory audit row at the recorder. The 60s active-tenant cache mirrors the pattern from TASK-AUTH-109 + TASK-AUTH-105 — bounded hot-path overhead, fast eventual consistency.

---

## §3 — API & schema

### §3.1 — Migration 0001: metering_events table

```sql
-- services/metering/migrations/0001_metering_events.sql

CREATE TYPE metering_axis AS ENUM ('seats', 'api_calls', 'ai_tokens', 'storage_bytes');
CREATE TYPE metering_unit AS ENUM ('seat', 'request', 'token', 'byte');
CREATE TYPE metering_event_state AS ENUM ('active', 'corrected', 'superseded');
CREATE TYPE metering_overage_policy AS ENUM ('block', 'warn', 'allow');

CREATE TABLE metering_events (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL REFERENCES tenants(id),
    axis            metering_axis NOT NULL,
    unit            metering_unit NOT NULL,
    quantity        BIGINT NOT NULL,
    idempotency_key TEXT NOT NULL CHECK (length(idempotency_key) BETWEEN 1 AND 64),
    occurred_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    period_start    TIMESTAMPTZ NOT NULL,
    period_end      TIMESTAMPTZ NOT NULL,
    state           metering_event_state NOT NULL DEFAULT 'active',
    correction_to   UUID REFERENCES metering_events(id),
    source_service  TEXT NOT NULL CHECK (length(source_service) BETWEEN 1 AND 64),
    memory_chain_hash TEXT NOT NULL CHECK (length(memory_chain_hash) = 64),  -- hex SHA-256
    extra           JSONB NOT NULL DEFAULT '{}'::jsonb,
    seq             BIGSERIAL NOT NULL,
    CONSTRAINT axis_unit_pair CHECK (
        (axis = 'seats'         AND unit = 'seat'   ) OR
        (axis = 'api_calls'     AND unit = 'request') OR
        (axis = 'ai_tokens'     AND unit = 'token'  ) OR
        (axis = 'storage_bytes' AND unit = 'byte'   )
    ),
    CONSTRAINT quantity_range CHECK (
        (axis = 'seats'         AND quantity BETWEEN 0 AND 100000) OR
        (axis = 'api_calls'     AND quantity BETWEEN -1000000 AND 1000000) OR
        (axis = 'ai_tokens'     AND quantity BETWEEN -10000000 AND 10000000) OR
        (axis = 'storage_bytes' AND quantity BETWEEN 0 AND 10000000000000)
    )
);

-- Idempotency within 24h
CREATE UNIQUE INDEX metering_events_idem
    ON metering_events (tenant_id, axis, idempotency_key)
    WHERE occurred_at > now() - INTERVAL '24 hours';

CREATE INDEX metering_events_period ON metering_events (tenant_id, axis, period_start, period_end);
CREATE INDEX metering_events_state ON metering_events (state) WHERE state = 'active';
CREATE INDEX metering_events_correction ON metering_events (correction_to) WHERE correction_to IS NOT NULL;

-- Append-only enforcement
REVOKE UPDATE, DELETE ON metering_events FROM cyberos_app;
GRANT INSERT, SELECT ON metering_events TO metering_writer;
GRANT SELECT ON metering_events TO metering_reader;

-- RLS
ALTER TABLE metering_events ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON metering_events
    USING (tenant_id = current_setting('cyberos.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('cyberos.tenant_id')::uuid);

-- Correction state-transition trigger
CREATE OR REPLACE FUNCTION metering_apply_correction() RETURNS TRIGGER AS $$
BEGIN
    IF NEW.correction_to IS NOT NULL THEN
        -- Verify sign-opposite + same axis
        DECLARE
            parent_axis metering_axis;
            parent_qty  BIGINT;
            parent_state metering_event_state;
        BEGIN
            SELECT axis, quantity, state INTO parent_axis, parent_qty, parent_state
                FROM metering_events WHERE id = NEW.correction_to
                FOR UPDATE;
            IF parent_axis IS NULL THEN
                RAISE EXCEPTION 'metering_parent_not_found' USING ERRCODE = 'P0200';
            END IF;
            IF parent_axis != NEW.axis THEN
                RAISE EXCEPTION 'metering_correction_axis_mismatch' USING ERRCODE = 'P0201';
            END IF;
            IF SIGN(NEW.quantity) = SIGN(parent_qty) AND NEW.quantity != 0 THEN
                RAISE EXCEPTION 'metering_correction_sign_must_oppose' USING ERRCODE = 'P0202';
            END IF;
            IF parent_state = 'superseded' THEN
                -- correcting a correction; intermediate flips to 'corrected'
                UPDATE metering_events SET state = 'corrected' WHERE id = NEW.correction_to;
            ELSE
                UPDATE metering_events SET state = 'superseded' WHERE id = NEW.correction_to;
            END IF;
        END;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;
CREATE TRIGGER metering_correction_trg
    BEFORE INSERT ON metering_events
    FOR EACH ROW
    WHEN (NEW.correction_to IS NOT NULL)
    EXECUTE FUNCTION metering_apply_correction();

-- Period-freeze rejection
CREATE OR REPLACE FUNCTION metering_reject_frozen() RETURNS TRIGGER AS $$
DECLARE frozen TIMESTAMPTZ;
BEGIN
    SELECT frozen_at INTO frozen FROM metering_periods
        WHERE tenant_id = NEW.tenant_id
          AND axis = NEW.axis
          AND period_start = NEW.period_start
          AND period_end = NEW.period_end;
    IF frozen IS NOT NULL THEN
        RAISE EXCEPTION 'metering_period_frozen' USING ERRCODE = 'P0203';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;
CREATE TRIGGER metering_freeze_trg
    BEFORE INSERT ON metering_events
    FOR EACH ROW
    EXECUTE FUNCTION metering_reject_frozen();
```

### §3.2 — Migration 0002: metering_periods + aggregator_state

```sql
-- services/metering/migrations/0002_metering_periods.sql

CREATE TABLE metering_periods (
    tenant_id           UUID NOT NULL REFERENCES tenants(id),
    axis                metering_axis NOT NULL,
    period_start        TIMESTAMPTZ NOT NULL,
    period_end          TIMESTAMPTZ NOT NULL,
    frozen_at           TIMESTAMPTZ,
    final_quantity      BIGINT,
    final_memory_hash    TEXT,
    last_aggregated_seq BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (tenant_id, axis, period_start, period_end)
);

ALTER TABLE metering_periods ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON metering_periods
    USING (tenant_id = current_setting('cyberos.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('cyberos.tenant_id')::uuid);

-- Per-tenant overage policy
ALTER TABLE tenants ADD COLUMN IF NOT EXISTS metering_overage_policy metering_overage_policy NOT NULL DEFAULT 'warn';
ALTER TABLE tenants ADD COLUMN IF NOT EXISTS metering_caps_yaml TEXT;  -- per-axis caps, validated at load
```

### §3.3 — Migration 0003: materialized view

```sql
-- services/metering/migrations/0003_metering_aggregates_view.sql

CREATE MATERIALIZED VIEW metering_current_period AS
SELECT
    tenant_id,
    axis,
    period_start,
    period_end,
    SUM(quantity) FILTER (WHERE state = 'active') AS total_quantity,
    COUNT(*) FILTER (WHERE state = 'active') AS event_count,
    MAX(seq) AS last_seq,
    now() AS last_refreshed_at
FROM metering_events
GROUP BY tenant_id, axis, period_start, period_end;

CREATE UNIQUE INDEX ON metering_current_period (tenant_id, axis, period_start, period_end);

-- pg_cron: refresh every 5 minutes
SELECT cron.schedule('metering_view_refresh', '*/5 * * * *',
    $$REFRESH MATERIALIZED VIEW CONCURRENTLY metering_current_period$$);
```

### §3.4 — Recorder API (Rust)

```rust
// services/metering/src/recorder.rs

use anyhow::Result;
use sqlx::PgPool;

#[derive(Debug, Clone, sqlx::Type)]
#[sqlx(type_name = "metering_axis", rename_all = "snake_case")]
pub enum Axis { Seats, ApiCalls, AiTokens, StorageBytes }

#[derive(Debug, Clone)]
pub struct MeteringEvent {
    pub tenant_id: uuid::Uuid,
    pub axis: Axis,
    pub quantity: i64,
    pub idempotency_key: String,
    pub source_service: String,
    pub extra: serde_json::Value,
}

pub struct Recorder {
    pool: PgPool,
    wal: WalQueue,
    active_tenants: Arc<RwLock<HashSet<uuid::Uuid>>>,
    cache_loaded_at: AtomicI64,
}

impl Recorder {
    pub async fn record(&self, event: MeteringEvent) -> Result<()> {
        // §1 #23 active-tenant cache check
        self.refresh_cache_if_stale().await?;
        if !self.active_tenants.read().contains(&event.tenant_id) {
            return Err(MeterError::TenantNotActive(event.tenant_id).into());
        }

        // §1 #19 quantity validation (DB CHECK is the source of truth; this is fast-fail)
        validate_quantity(event.axis, event.quantity)?;

        // §1 #14 enqueue to WAL with back-pressure
        self.wal.push(event).await?;
        Ok(())
    }

    async fn refresh_cache_if_stale(&self) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        if now - self.cache_loaded_at.load(Ordering::Relaxed) > 60 {
            let active: HashSet<uuid::Uuid> = sqlx::query_scalar(
                "SELECT id FROM tenants WHERE state NOT IN ('terminated', 'never_active')"
            ).fetch_all(&self.pool).await?.into_iter().collect();
            *self.active_tenants.write() = active;
            self.cache_loaded_at.store(now, Ordering::Relaxed);
        }
        Ok(())
    }
}
```

### §3.5 — Period-close handler

```rust
// services/metering/src/handlers/period_close.rs

pub async fn close_period(
    pool: &PgPool,
    tenant_id: uuid::Uuid,
    axis: Axis,
    period_start: DateTime<Utc>,
    period_end: DateTime<Utc>,
) -> Result<PeriodCloseReceipt> {
    let mut tx = pool.begin().await?;

    // §1 #24 verify aggregator caught up
    let row = sqlx::query!(
        r#"SELECT
              (SELECT COALESCE(MAX(seq), 0) FROM metering_events
                  WHERE tenant_id = $1 AND axis = $2
                    AND occurred_at >= $3 AND occurred_at < $4) AS events_max_seq,
              (SELECT last_aggregated_seq FROM metering_periods
                  WHERE tenant_id = $1 AND axis = $2
                    AND period_start = $3 AND period_end = $4) AS agg_seq"#,
        tenant_id, axis as _, period_start, period_end
    ).fetch_one(&mut *tx).await?;
    if row.events_max_seq > row.agg_seq.unwrap_or(0) {
        return Err(MeterError::AggregatorBehind { latest: row.events_max_seq, agg: row.agg_seq.unwrap_or(0) }.into());
    }

    // Take the snapshot from MV
    let final_qty: i64 = sqlx::query_scalar!(
        r#"SELECT COALESCE(total_quantity, 0)
              FROM metering_current_period
              WHERE tenant_id = $1 AND axis = $2
                AND period_start = $3 AND period_end = $4"#,
        tenant_id, axis as _, period_start, period_end
    ).fetch_one(&mut *tx).await?.unwrap_or(0);

    // Freeze
    let memory_hash = emit_final_memory(tenant_id, axis, final_qty, period_start, period_end).await?;
    sqlx::query!(
        r#"UPDATE metering_periods
              SET frozen_at = now(), final_quantity = $5, final_memory_hash = $6
              WHERE tenant_id = $1 AND axis = $2 AND period_start = $3 AND period_end = $4"#,
        tenant_id, axis as _, period_start, period_end, final_qty, &memory_hash
    ).execute(&mut *tx).await?;

    tx.commit().await?;
    Ok(PeriodCloseReceipt { tenant_id, axis, final_quantity: final_qty, memory_chain_hash: memory_hash })
}
```

---

## §4 — Acceptance criteria

1. `metering_events` table created with 4-value `metering_axis` enum + 4-value `metering_unit` enum + 3-value `metering_event_state` enum.
2. CI cardinality test asserts exactly 4 axis values, 4 unit values, 3 state values, 3 overage policy values.
3. `REVOKE UPDATE, DELETE` on `metering_events` confirmed at `\dp` inspection.
4. Inserting two events with identical `(tenant_id, axis, idempotency_key)` within 24h returns success but only one row exists.
5. Inserting an event with `axis = api_calls, unit = token` fails with `axis_unit_pair` CHECK violation.
6. Inserting an event with `axis = seats, quantity = 200000` fails with `quantity_range` CHECK violation.
7. RLS prevents cross-tenant SELECT on `metering_events`.
8. memory audit row emitted per metering event with cross-link `postgres_event_id` ↔ `memory_chain_hash`.
9. API-call middleware emits `api_calls` event with `quantity = 1` per successful auth-scoped request.
10. Failed-auth request emits no metering event.
11. AI gateway `postcall_reconcile()` emits `ai_tokens` event with `quantity = input + output`.
12. Seats snapshot job emits ONE `seats` event per tenant per period at close boundary.
13. Storage snapshot job emits ONE `storage_bytes` event combining S3 + Postgres bytes.
14. Per-tenant overage policy mutation requires `cfo` role + reason ≥ 10 chars; sev-2 memory audit emitted.
15. Overage `block` policy returns `402 PAYMENT_REQUIRED` when cap exceeded; no metering event emitted.
16. Overage `warn` policy emits sev-2 memory audit at threshold-crossing; metric event proceeds.
17. Overage `allow` policy: no enforcement; metric event proceeds.
18. `GET /v1/usage` returns 4-axis usage from `metering_current_period` view.
19. Materialized view refreshes every 5 minutes via pg_cron; `last_refreshed_at` advances.
20. Period-close handler verifies `last_aggregated_seq >= max(events.seq)` before freezing; rejects on lag.
21. Inserting a metering event with `occurred_at` inside a frozen period fails with `metering_period_frozen` (SQLSTATE P0203).
22. `POST /v1/usage/correction` requires `cfo` role + reason ≥ 10 chars; emits `correction_to` row.
23. Correction trigger flips parent row to `state = 'superseded'`; double-correction flips intermediate to `corrected`.
24. Correction with same-sign `quantity` fails with `metering_correction_sign_must_oppose` (P0202).
25. WAL queue at 90% utilization emits `metering.wal_queue_overflow` sev-1 memory audit + rejects new events.
26. Metering event for a terminated tenant (TASK-TEN-104) returns `404 TENANT_NOT_FOUND` at recorder.
27. Active-tenant cache refresh interval is 60s; staleness causes one extra DB query, not stale-data acceptance.
28. Dispute log INSERT requires `billing_disputes` role; resolution_notes ≤ 4 KiB enforced.
29. Cross-tenant aggregation query via `bookkeeper` role bypasses RLS (SECURITY DEFINER); never via `cyberos_app`.
30. All reason-bearing audit text (refund reason, policy reason, dispute notes) scrubbed via TASK-MEMORY-111 before memory chain emission.

---

## §5 — Verification (CI tests)

- `cardinality_test_axis` — asserts `count(*) FROM pg_enum WHERE enumtypid = 'metering_axis'::regtype = 4`.
- `cardinality_test_unit` — 4.
- `cardinality_test_state` — 3.
- `cardinality_test_policy` — 3.
- `axis_unit_pair_test` — table-driven 16-cell test (4 axes × 4 units) asserts 4 valid, 12 invalid.
- `quantity_range_test` — boundary tests for each axis range.
- `append_only_test` — REVOKE inspection + attempt UPDATE/DELETE returns permission denied.
- `idempotency_test` — second insert with same key returns 200 + no second row.
- `correction_to_test` — sign-opposite trigger; state transitions; aggregation excludes superseded.
- `rls_isolation_test` — two tenants, cross-query returns empty.
- `frozen_period_test` — freeze + insert returns P0203.
- `wal_overflow_test` — fill queue to 90%; sev-1 audit emitted; new events rejected.
- `terminated_tenant_test` — terminate tenant; emit event; recorder returns 404 + sev-2 audit.
- `api_middleware_test` — 100 successful requests = 100 events; 100 failed-auth = 0 events.
- `ai_postcall_test` — mock provider response with 1500 input + 800 output tokens; assert `quantity = 2300`.
- `seats_snapshot_test` — seed 12 active + 3 deactivated members; assert snapshot = 12.
- `storage_snapshot_test` — seed 50 GiB S3 + 3 GiB Postgres; assert quantity ≈ 53 GiB in bytes.
- `overage_block_test` — set cap = 100; submit 101st event; returns 402; no event recorded.
- `overage_warn_test` — set cap = 100, threshold = 0.8; cross at 81st event; sev-2 audit emitted.
- `dispute_acl_test` — non-`billing_disputes` role INSERT to dispute_log fails with permission denied.
- `mv_refresh_test` — insert 10 events, wait 5 min, query view; assert sum = 10.
- `aggregator_lag_test` — insert event but block MV refresh; close-period returns AggregatorBehind error.

---

## §6 — File skeleton

```
services/metering/
├── Cargo.toml
├── migrations/
│   ├── 0001_metering_events.sql
│   ├── 0002_metering_periods.sql
│   └── 0003_metering_aggregates_view.sql
├── src/
│   ├── lib.rs                  # Recorder + Aggregator pub re-exports
│   ├── recorder.rs             # §3.4 Recorder API
│   ├── aggregator.rs           # MV refresh + close-period orchestration
│   ├── wal_queue.rs            # bounded in-memory WAL + replay
│   ├── policy.rs               # overage policy enforcement
│   ├── memory_audit.rs          # 7-kind audit emission
│   ├── axes/
│   │   ├── seats.rs            # end-of-period snapshot job
│   │   ├── api_calls.rs        # middleware integration glue
│   │   ├── ai_tokens.rs        # cost_ledger postcall hook
│   │   └── storage.rs          # S3 inventory + pg_total_relation_size
│   ├── handlers/
│   │   ├── usage_query.rs      # GET /v1/usage
│   │   ├── usage_correction.rs # POST /v1/usage/correction
│   │   └── period_close.rs     # POST /v1/metering/period/close (ops)
│   └── error.rs                # MeterError enum
├── config/
│   └── default_caps.yaml       # platform-default per-axis caps
└── tests/
    ├── api_metering_test.rs
    ├── ai_metering_test.rs
    ├── seats_snapshot_test.rs
    ├── storage_snapshot_test.rs
    ├── append_only_test.rs
    ├── overage_policy_test.rs
    └── wal_replay_test.rs
```

---

## §7 — Dependencies & blast-radius

**Depends on**: TASK-AI-001 (cost_ledger postcall hook for AI tokens), TASK-TEN-001 (tenants table + provisioning), TASK-AUTH-003 (RLS scope for middleware), TASK-MEMORY-111 (PII scrubbing for reason fields).

**Blocks**: TASK-TEN-003 (Stripe billing — billing service consumes `metering_current_period` + period-close finals as the per-axis charge basis).

**Blast radius if broken**:
- **Under-bill**: missing events → tenant pays less than the platform spends. Cumulative loss at scale.
- **Over-bill**: phantom events or correction failures → disputes + refunds + reputational damage.
- **Cross-tenant leak**: RLS misconfiguration would surface another tenant's usage. Compliance-critical.
- **Chain divergence**: Postgres-memory drift caught at period close but acted on at sev-1 reconciliation incident.

---

## §8 — Payload examples

### §8.1 — Successful api_calls event

```json
POST /v1/metering/internal/record
Authorization: Bearer <service-token-with-metering_writer-role>

{
  "tenant_id": "ten_abc",
  "axis": "api_calls",
  "quantity": 1,
  "idempotency_key": "req_2026-05-16T10:30:01.234Z_a8f3",
  "source_service": "auth-middleware",
  "extra": {"path": "/v1/documents/search", "method": "POST", "status": 200}
}

200 OK
{
  "event_id": "01J2C9X3K7M4N5...",
  "memory_chain_hash": "7a3f9c2e1d5b8...",
  "duplicate": false
}
```

### §8.2 — Overage blocked

```json
POST /v1/documents/search  (downstream of middleware)

402 PAYMENT_REQUIRED
{
  "error": "overage_blocked",
  "axis": "api_calls",
  "current_period_total": 1000000,
  "cap": 1000000,
  "policy": "block",
  "contact": "billing@cyberos.world"
}
```

### §8.3 — Correction (refund)

```json
POST /v1/usage/correction
Authorization: Bearer <cfo-token>

{
  "original_event_id": "01J2C9X3K7M4N5...",
  "reason": "Provider double-billed our ai_tokens consumer; refunding 5000 tokens",
  "axis": "ai_tokens",
  "quantity": -5000
}

200 OK
{
  "correction_event_id": "01J2CAB9F3...",
  "parent_state": "superseded",
  "memory_chain_hash": "4f8e2a..."
}
```

### §8.4 — `GET /v1/usage`

```json
GET /v1/usage
Authorization: Bearer <tenant-admin>

200 OK
{
  "tenant_id": "ten_abc",
  "period_start": "2026-05-01T00:00:00Z",
  "period_end": "2026-05-31T23:59:59Z",
  "seats": 12,
  "api_calls": 4823901,
  "ai_tokens": 12834729,
  "storage_bytes": 38291723123,
  "overage_policy": "warn",
  "warn_thresholds_crossed": ["api_calls"],
  "view_last_refreshed_at": "2026-05-16T10:25:00Z"
}
```

---

## §9 — Open questions

- **OQ-1** (closed by DEC-705): seats snapshot at close, not mid-period events. Confirmed.
- **OQ-2** (closed by DEC-709): in-memory WAL acceptable for restart-loss; TASK-OBS-009 captures the bounded loss event.
- **OQ-3** (closed by DEC-715): 24h idempotency retention sufficient for upstream retry budgets.
- **OQ-4** (open): granularity of warn_threshold — single threshold (0.80) or multiple (0.60 + 0.80 + 0.95)? Default single; revisit when first customer pushes back. Track in EVOLUTION.md.
- **OQ-5** (open): should `bookkeeper` cross-tenant queries themselves be audited? Currently they emit sev-3 audit at each query. Watch for over-emission noise; tighten if it becomes operational friction.

---

## §10 — Failure modes (32 rows)

| # | Failure | Detection | Sev | Handler |
|---|---------|-----------|-----|---------|
| 1 | Postgres outage during recorder write | sqlx error | 1 | WAL queue absorbs; back-pressure at 90% |
| 2 | memory audit subprocess hang | timeout 5s | 1 | Mark event WAL-pending; retry on replay |
| 3 | WAL queue overflow (>90%) | metering.wal_queue_overflow sev-1 | 1 | Reject new events; ops alarm |
| 4 | Materialized view refresh job dies | pg_cron heartbeat absent | 1 | Sev-1 alarm; usage queries fall back to raw COUNT |
| 5 | Active-tenant cache stale (terminated tenant still in cache) | 60s TTL bounded staleness | 3 | Period-close reconciliation flags & corrects |
| 6 | Cross-tenant RLS misconfiguration | rls_isolation_test fail | 1 | CI blocks deploy |
| 7 | Duplicate idempotency_key submission | UNIQUE INDEX constraint | 3 | Returns success; no second row |
| 8 | Quantity out of CHECK range | DB error 23514 | 3 | Recorder returns 400 + sev-3 audit |
| 9 | axis/unit pair mismatch | DB CHECK violation | 3 | Recorder returns 400 + sev-3 audit |
| 10 | Correction with same-sign quantity | P0202 trigger | 2 | Reject + sev-2 audit |
| 11 | Correction targets non-existent parent | P0200 trigger | 2 | Reject + sev-2 audit |
| 12 | Period freeze attempt while aggregator behind | AggregatorBehind error | 2 | Reject; ops investigates MV refresh |
| 13 | Recorder receives event for terminated tenant | 404 + active-tenant cache miss | 2 | Reject + sev-2 audit |
| 14 | Insert into frozen period | P0203 trigger | 2 | Reject + sev-2 audit |
| 15 | Non-cfo role attempts policy mutation | RBAC check | 2 | Return 403 + sev-2 audit |
| 16 | Non-cfo role attempts correction | RBAC check | 2 | Return 403 + sev-2 audit |
| 17 | Non-billing_disputes role inserts dispute_log | GRANT check | 2 | DB permission denied |
| 18 | resolution_notes > 4 KiB | CHECK constraint | 3 | Reject 400 |
| 19 | source_service > 64 chars | CHECK constraint | 3 | Reject 400 |
| 20 | idempotency_key length out of [1,64] | CHECK constraint | 3 | Reject 400 |
| 21 | memory chain divergence at period close | reconciliation diff | 1 | Sev-1; ops manual investigation |
| 22 | S3 inventory CSV missing | snapshot job error | 2 | Retry up to 6h; if still missing, sev-2; skip storage event |
| 23 | pg_total_relation_size returns 0 (DB outage) | snapshot job sanity | 2 | Retry; if still 0, sev-2; manual review |
| 24 | Two writers race on same idempotency_key | UNIQUE constraint wins | 3 | First-writer-wins; second sees duplicate |
| 25 | Refund row inserted in already-frozen period | P0203 | 2 | Reject; CFO re-issues in current period with note |
| 26 | Materialized view refresh CONCURRENT lock contention | pg_stat_activity | 3 | Retry next 5-min tick |
| 27 | API-middleware emission failure | WAL push error | 1 | Logged; back-pressure to upstream service |
| 28 | AI gateway postcall hook fails to emit | structured-log | 1 | Logged; reconciliation at period close detects |
| 29 | Tenant policy YAML invalid at load | schema validation | 1 | CI/ops alarm; old policy stays active |
| 30 | Cap value exceeds INT8 max | CHECK constraint | 3 | Reject policy update; 400 |
| 31 | Period-close attempted on wrong tenant by bookkeeper role | RBAC + tenant context | 2 | Reject; sev-2 audit |
| 32 | Dispute resolution status invalid value | ENUM constraint | 3 | Reject 400 |

---

## §11 — Implementation notes

**§11.1** Append-only is enforced at SQL grant + at the application layer (Recorder API has no UPDATE/DELETE methods). Defense-in-depth: a code bug that issued an UPDATE would still be rejected by Postgres.

**§11.2** The WAL queue is implemented as `tokio::sync::mpsc::channel(100_000)`. Send returns `Err` when buffer full; the recorder service converts that to a 503 + sev-1 memory audit.

**§11.3** The materialized view is `REFRESH MATERIALIZED VIEW CONCURRENTLY` — readers see the previous version until the refresh completes. The 5-min cadence is chosen to keep the refresh under 30s for ≤100k events per period per tenant.

**§11.4** The recorder service exposes an internal-only API (mTLS + service-token, no tenant access) on a separate port from the public REST surface. The metering_writer role is granted only to the service principal that holds the mTLS cert.

**§11.5** API-call metering is wired into the auth-crate middleware (TASK-AUTH-003). The middleware emits to a process-local mpsc; a background task batches every 100 events / 100ms and ships to the recorder. Batching keeps the recorder API RPS bounded.

**§11.6** AI-token metering is a one-line addition to TASK-AI-001's `postcall_reconcile`: after the cost ledger row is committed, the postcall hook emits to the recorder. The emission failure is logged but not retried inline — the period-close reconciliation will catch any drift.

**§11.7** Seats snapshot runs at billing-period-close as a cron job. The job is idempotent: re-running the same period emits the same event (same idempotency_key derived from `(tenant_id, axis, period_start, period_end)`).

**§11.8** Storage snapshot has two sub-jobs that run independently then aggregate. If one (S3 or Postgres) is failing, the other still emits; the partial event is later corrected via correction_to once the missing piece is recovered.

**§11.9** Overage policy enforcement runs at the auth middleware (where api_calls is metered) and at the AI gateway pre-call (where ai_tokens is metered). For seats + storage, enforcement is at period close — there's no per-event admission control because there are no per-event seat additions or storage upticks in the metering model.

**§11.10** Per-tenant caps live in `tenants.metering_caps_yaml` validated against a schema at load. Examples: `seats: 100`, `api_calls: 1000000`, `ai_tokens: 10000000`, `storage_bytes: 100000000000`. Unspecified axes use platform defaults from `config/default_caps.yaml`.

**§11.11** The `bookkeeper` role used by the billing service runs cross-tenant queries via SECURITY DEFINER functions that set `cyberos.tenant_id` to a sentinel `00000000-0000-0000-0000-000000000000` value that bypasses RLS. The function emits a sev-3 memory audit per call.

**§11.12** Tests use a real Postgres via `sqlx::testcontainers` — no mocked DB. Migrations run on every test fixture setup; the test container is reused per file (`#[tokio::test]` with shared pool).

**§11.13** The `metering_events.memory_chain_hash` is the memory row's chain hash from the canonical Writer. The bridge is via subprocess fork (mirrors TASK-AI-001 audit emission); PyO3 is a future optimization.

**§11.14** Refunds via `correction_to` flow through the same recorder API as primary events — the API is generic over insertion. The CFO-only authorization is at the HTTP handler layer, not the recorder.

**§11.15** Dispute logs are visible to the CFO + billing_disputes roles; not visible to tenant admins. Disputes are operational records, not customer-facing.

**§11.16** The 60s active-tenant cache is per-process (each metering writer process holds its own). On tenant termination, the worst-case 60s window where the recorder still accepts events for the terminated tenant is acceptable because (a) terminated tenants' services are already shut down, so no upstream is emitting; (b) the TASK-TEN-104 90-day grace handles any reactivation; (c) period-close reconciliation flags any stale events.

**§11.17** The pg_cron `metering_view_refresh` job logs a sev-1 audit if the refresh takes > 4 minutes (signaling MV is approaching the 5-minute tick boundary). Operations triage tuning of axis indexing.

**§11.18** Period boundaries are computed from `tenants.billing_timezone` + `tenants.billing_period_kind` (monthly, quarterly). The recorder accepts any `occurred_at` falling within the corresponding period_start/period_end; the period derivation runs in the recorder before WAL push.

**§11.19** When the recorder is rolled to a new version, the in-memory WAL queue is drained before shutdown (graceful drain on SIGTERM, 30s timeout). After timeout, remaining events are logged as sev-1 audit. The deployment runbook documents this drain.

**§11.20** Cardinality CI tests run in the migration test suite, not at runtime. Adding a fifth axis would require updating the CI assertion + the DEC entry.

**§11.21** The 100_000 WAL buffer is sized for ~1 second of peak traffic at 100k events/sec. Bumping the buffer requires verifying memory footprint (each event ≈ 1 KiB → 100 MiB at full).

**§11.22** The `extra` JSONB field on `metering_events` is intentionally schemaless to absorb future axis-specific metadata without migration. The metering team curates a registry of well-known `extra` keys per axis in `services/metering/docs/extra_schema.md`.

---

*End of TASK-TEN-004 spec.*
