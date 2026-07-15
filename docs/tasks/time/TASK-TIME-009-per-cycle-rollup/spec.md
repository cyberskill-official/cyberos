---
id: TASK-TIME-009
title: "TIME per-cycle billable rollup → INV — per-Member × role × Engagement aggregation with rate-card application + idempotent emit"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-05-17T00:00:00+07:00
department: engineering
author: @stephencheng
template: task@1
module: TIME
priority: p0
status: draft
verify: T
phase: P1
milestone: P1 · slice 1
slice: 1
owner: Stephen Cheng (CFO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-TIME-001, TASK-TIME-005, TASK-TIME-006, TASK-INV-001, TASK-AUTH-101, TASK-AI-003, TASK-MEMORY-111]
depends_on: [TASK-TIME-001, TASK-TIME-005]
blocks: [TASK-INV-001]

source_pages:
  - website/docs/modules/time.html#rollup

source_decisions:
  - DEC-1430 2026-05-17 — Per-cycle rollup runs at end of billing cycle (engagement-configured: monthly default); aggregates approved billable entries per (Member, role-on-engagement, project, task); produces JSON payload consumed by TASK-INV-001 draft creation
  - DEC-1431 2026-05-17 — Idempotent on (engagement_id, cycle_end_date); duplicate rollup invocations return cached result
  - DEC-1432 2026-05-17 — Only `is_billable=true` AND timesheet status='locked' entries included; reject otherwise
  - DEC-1433 2026-05-17 — Rate-card applied at rollup time; snapshot stored in result for TASK-INV-001 to copy
  - DEC-1434 2026-05-17 — memory audit kinds: time.rollup_started, time.rollup_completed, time.rollup_failed, time.rollup_idempotent_hit

build_envelope:
  language: rust 1.81
  service: cyberos/services/time/
  new_files:
    - services/time/migrations/0007_rollup_cache.sql
    - services/time/src/rollup/mod.rs
    - services/time/src/rollup/aggregator.rs
    - services/time/src/rollup/rate_card_apply.rs
    - services/time/src/audit/rollup_events.rs
    - services/time/src/handlers/rollup_routes.rs
    - services/time/tests/rollup_aggregation_test.rs
    - services/time/tests/rollup_idempotent_test.rs
    - services/time/tests/rollup_billable_only_test.rs
    - services/time/tests/rollup_locked_only_test.rs
    - services/time/tests/rollup_rate_card_snapshot_test.rs
    - services/time/tests/rollup_audit_emission_test.rs

  modified_files:
    - services/time/src/lib.rs

  allowed_tools:
    - file_read: services/time/**
    - file_write: services/time/{src,tests,migrations}/**
    - bash: cd services/time && cargo test rollup

  disallowed_tools:
    - include non-billable entries (per DEC-1432)
    - include unlocked timesheet entries (per DEC-1432)
    - skip rate-card snapshot (per DEC-1433)

effort_hours: 6
subtasks:
  - "0.4h: 0007_rollup_cache.sql"
  - "0.3h: rollup/mod.rs"
  - "0.7h: rollup/aggregator.rs (SQL groupby per dimension)"
  - "0.4h: rollup/rate_card_apply.rs"
  - "0.3h: audit/rollup_events.rs"
  - "0.3h: handlers/rollup_routes.rs"
  - "1.5h: tests — 6 test files"
  - "0.6h: integration with TASK-INV-001 draft creation"

risk_if_skipped: "Without rollup, TASK-INV-001 has no way to convert TIME entries into invoice lines → entire billing pipeline broken. Without DEC-1431 idempotency, double-billing on retry. Without DEC-1432 status filter, draft entries leak to invoices. Without DEC-1433 rate snapshot, TASK-INV-001's rate-card snapshot DEC-1363 broken. The 6h effort completes the TIME→INV pipeline."
---

## §1 — Description (BCP-14 normative)

The TIME service **MUST** ship per-cycle billable rollup at `services/time/src/rollup/` producing per-Member × role × project × task aggregations consumed by TASK-INV-001, with idempotency cache, rate-card snapshot, status/billable filtering, and 4 memory audit kinds.

1. **MUST** expose `POST /v1/time/rollup` body `{ engagement_id, cycle_start_date, cycle_end_date }`. Caller has `cfo` OR `engagement_admin` role. Handler:
   - Check cache by `(engagement_id, cycle_end_date)`; hit → return cached + emit `time.rollup_idempotent_hit`.
   - Else: invoke aggregator; cache result; emit `time.rollup_started` + `time.rollup_completed`.

2. **MUST** aggregate via `aggregator.rs::aggregate(engagement_id, start, end)`:
   ```sql
   SELECT member_subject_id, role_on_engagement, project_id, task_id,
          SUM(duration_seconds) AS total_seconds,
          COUNT(*) AS entry_count
   FROM time_entries e
   JOIN timesheets t ON t.member_subject_id = e.member_subject_id
                     AND t.engagement_id = e.engagement_id
                     AND e.entry_date BETWEEN t.week_start_date AND t.week_end_date
   WHERE e.engagement_id = $1
     AND e.entry_date BETWEEN $2 AND $3
     AND e.is_billable = true
     AND t.status = 'locked'
   GROUP BY member_subject_id, role_on_engagement, project_id, task_id
   ```

3. **MUST** apply rate-card per DEC-1433 via `rate_card_apply.rs::apply(rollup_row, rate_card)`. For each aggregation row, look up rate by (role_on_engagement, project_kind) → produce `unit_price_minor` + computed `amount_minor = total_seconds / 3600 * unit_price_minor`.

4. **MUST** define rollup cache table at migration `0007`: `(engagement_id UUID NOT NULL, cycle_end_date DATE NOT NULL, result_jsonb JSONB NOT NULL, rate_card_snapshot JSONB NOT NULL, rolled_up_at TIMESTAMPTZ NOT NULL DEFAULT now(), rolled_up_by_subject_id UUID NOT NULL, trace_id CHAR(32), PRIMARY KEY (engagement_id, cycle_end_date))`. Append-only.

5. **MUST** enforce RLS scoped to tenant_id.

6. **MUST** return result shape consumable by TASK-INV-001:
   ```json
   {
     "engagement_id": "...",
     "cycle_start": "2026-04-17", "cycle_end": "2026-05-17",
     "rate_card_snapshot": { /* full card */ },
     "lines": [
       { "member_subject_id": "...", "role_on_engagement": "senior_consultant",
         "project_id": "...", "task_id": "...",
         "total_seconds": 144000, "unit_price_minor": 250_00,
         "amount_minor": 10_000_00 }
     ],
     "totals": { "total_seconds": ..., "amount_minor": ... }
   }
   ```

7. **MUST** emit 4 memory audit kinds per DEC-1434.

8. **MUST** thread trace_id end-to-end.

9. **MUST NOT** include non-billable or unlocked entries.

10. **MUST NOT** re-run rollup for cached (engagement, cycle_end) — return cached.

---

## §2 — Why this design (rationale)

**Why idempotent caching (§1 #1, DEC-1431)?** Rollup is computationally expensive; multiple INV-001 retry attempts would re-aggregate. Cache eliminates waste + ensures determinism (re-run produces same numbers).

**Why timesheets='locked' filter (§1 #2, DEC-1432)?** Pre-approval entries are draft; including would create invoices on un-reviewed data → trust + audit gap.

**Why rate-card snapshot in result (§1 #3, DEC-1433)?** TASK-INV-001 needs the snapshot to fulfill DEC-1363. This task provides it ready-to-copy.

---

## §3 — API contract

```sql
-- 0007_rollup_cache.sql
CREATE TABLE rollup_cache (
  engagement_id UUID NOT NULL,
  cycle_end_date DATE NOT NULL,
  tenant_id UUID NOT NULL,
  result_jsonb JSONB NOT NULL,
  rate_card_snapshot JSONB NOT NULL,
  rolled_up_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  rolled_up_by_subject_id UUID NOT NULL,
  trace_id CHAR(32),
  PRIMARY KEY (engagement_id, cycle_end_date)
);
ALTER TABLE rollup_cache ENABLE ROW LEVEL SECURITY;
CREATE POLICY rollup_cache_rls ON rollup_cache
  USING (tenant_id = current_setting('auth.tenant_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
REVOKE UPDATE, DELETE ON rollup_cache FROM cyberos_app;
```

Endpoint: `POST /v1/time/rollup`.

---

## §4 — Acceptance criteria

1. **Rollup aggregates per Member/role/project/task** — fixture entries produce expected groupby.
2. **Idempotent** — second call returns cached + audit.
3. **Excludes non-billable** — `is_billable=false` entries omitted.
4. **Excludes unlocked** — `timesheets.status≠'locked'` entries omitted.
5. **Rate-card snapshot returned** — `result.rate_card_snapshot` present.
6. **Amount = seconds/3600 × unit_price** — math correct.
7. **4 memory audit kinds emitted**.
8. **CFO or engagement_admin only** — other roles 403.
9. **Trace_id end-to-end**.
10. **RLS cross-tenant denied**.
11. **Empty rollup (no entries)** — returns 0-line result; cached.
12. **Cycle dates inclusive** — entries on cycle_end_date included.
13. **Multi-role same Member** — different role_on_engagement → separate lines.
14. **Long-running rollup → task** — > 30s engagement uses TASK-MCP-007 pattern (slice 2 enhancement).
15. **Cache PRIMARY KEY enforces idempotency** — concurrent calls race-safe.
16. **Total seconds sum matches** — Σ(lines) == result.totals.total_seconds.
17. **Rate card missing for role** — line emitted with 0 unit_price + warning.
18. **Member without engagement role** — skipped + sev-3 audit.
19. **Audit emit on idempotent hit** — `time.rollup_idempotent_hit` row added.
20. **JSON shape stable** — TASK-INV-001 consumer tested against fixture.

---

## §5 — Verification

```rust
#[tokio::test]
async fn rollup_aggregates_per_member_role_project() {
    let ctx = TestContext::with_engagement_and_locked_entries().await;
    let r = ctx.post_rollup(ctx.eng_id, ctx.cycle_start, ctx.cycle_end).await;
    let body: serde_json::Value = r.json().await.unwrap();
    assert!(body["lines"].as_array().unwrap().len() > 0);
    let total_seconds: i64 = body["totals"]["total_seconds"].as_i64().unwrap();
    let line_sum: i64 = body["lines"].as_array().unwrap().iter()
        .map(|l| l["total_seconds"].as_i64().unwrap()).sum();
    assert_eq!(total_seconds, line_sum);
}

#[tokio::test]
async fn rollup_idempotent() {
    let ctx = TestContext::with_engagement_and_locked_entries().await;
    let r1 = ctx.post_rollup(ctx.eng_id, ctx.cycle_start, ctx.cycle_end).await;
    let r2 = ctx.post_rollup(ctx.eng_id, ctx.cycle_start, ctx.cycle_end).await;
    let b1: serde_json::Value = r1.json().await.unwrap();
    let b2: serde_json::Value = r2.json().await.unwrap();
    assert_eq!(b1, b2);
    let audit = ctx.memory_rows().await;
    assert!(audit.iter().any(|r| r.kind == "time.rollup_idempotent_hit"));
}

#[tokio::test]
async fn rollup_excludes_non_billable_and_unlocked() {
    let ctx = TestContext::new().await;
    ctx.seed_entry(billable: true, status: "locked").await;
    ctx.seed_entry(billable: false, status: "locked").await;
    ctx.seed_entry(billable: true, status: "submitted").await;
    let r = ctx.post_rollup_minimal().await;
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["lines"].as_array().unwrap().len(), 1);
}

// 5.4..5.6: rate-card snapshot, role check, audit kinds
```

---

## §7 — Dependencies

**Upstream:** TASK-TIME-005 (billable flag); transitively TASK-TIME-001 + TASK-TIME-006 (locked status).
**Downstream:** TASK-INV-001 (consumes rollup result).
**Cross-module:** TASK-AUTH-101 (role gates), TASK-AI-003, TASK-MEMORY-111.

---

## §8 — Example payload

`time.rollup_completed`:
```json
{
  "kind": "time.rollup_completed",
  "severity": 2,
  "tenant_id": "8a2f...",
  "actor_id": "user.cfo.789",
  "trace_id": "...",
  "payload": {
    "engagement_id": "0190...",
    "cycle_start": "2026-04-17", "cycle_end": "2026-05-17",
    "line_count": 12,
    "total_seconds": 875000,
    "total_amount_minor": 21_875_00
  }
}
```

---

## §9 — Open questions

Deferred:
- **Deferred:** Multi-currency rollup (currently single-currency per engagement) — slice 2.
- **Deferred:** Per-task rate-card overrides — slice 2.
- **Deferred:** Async via TASK-MCP-007 Tasks for huge engagements — slice 2.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Engagement has no rate card | resolver miss | 412 + `no_rate_card` | Engagement admin configures |
| Cache PRIMARY KEY race | concurrent INSERT | Second sees ON CONFLICT; returns cached | Inherent |
| Rate-card mid-cycle change | resolver uses cycle_end version | Stable across reruns | Inherent |
| Member transferred mid-cycle | role_on_engagement snapshot at entry time | Each line reflects role-at-time | Inherent |
| Cycle includes future date | date validation | 400 | Inherent |
| Engagement archived during rollup | status check | Sev-2 audit; rollup proceeds (historical data still billable) | Inherent |
| Rate-card missing for specific role | line with 0 unit_price + warning audit | Engagement_admin completes card | Inherent |
| Cache JSONB > 100 KB | size limit | sev-2 alert | Investigate engagement size |
| Cross-tenant rollup | RLS | 0 rows; empty result | Inherent |
| Member without locked timesheet for week | entry filtered | Inherent | Member submits + AM approves |
| Concurrent locked status change | tx isolation | Cycle uses status-at-rollup-time | Inherent |
| Database query timeout | timeout 30s | 503 | Slice 2 async via Tasks |

---

## §11 — Implementation notes

**§11.1** Aggregation uses single SQL GROUP BY for performance.

**§11.2** Rate card lookup per (role, project_kind); falls back to engagement default.

**§11.3** Cache PRIMARY KEY on (engagement_id, cycle_end_date) enforces idempotency at schema level.

**§11.4** Result JSONB serialized canonically (sorted keys) for deterministic hash.

**§11.5** TASK-INV-001 invokes rollup synchronously then creates invoice with result.

**§11.6** Idempotent hit emits separate audit kind for forensic clarity.

**§11.7** Engagement timezone applied to cycle date interpretation.

**§11.8** Cache pruned at slice 3 (12-month retention).

**§11.9** Trace_id propagated from TASK-INV-001 caller through rollup to TASK-INV-001 invoice row.

**§11.10** PII: Member subject_id retained for traceability; description not in rollup payload (entries already filtered + aggregated).

---

*End of TASK-TIME-009 spec.*
