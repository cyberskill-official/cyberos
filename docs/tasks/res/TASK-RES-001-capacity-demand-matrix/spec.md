---
id: TASK-RES-001
title: "RES capacity-vs-demand matrix — nightly join across HR + PROJ + TIME + LEARN producing per-member-week capacity/demand grid"
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
module: res
priority: p0
status: draft
verify: T
phase: P1
milestone: P1 · slice 7
slice: 7
owner: Stephen Cheng (CHRO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-HR-001, TASK-PROJ-001, TASK-TIME-001, TASK-LEARN-001, TASK-MCP-007, TASK-MEMORY-111]
depends_on: [TASK-HR-001, TASK-PROJ-001, TASK-TIME-001]
blocks: [TASK-RES-002, TASK-RES-003]

source_pages:
  - website/docs/modules/res.html#capacity-matrix

source_decisions:
  - DEC-2030 2026-05-17 — Matrix is per (member_id, iso_week, project_id) with capacity_hours + demand_hours + allocated_hours; nightly batch refresh
  - DEC-2031 2026-05-17 — Capacity computed from member.hours_per_week (TASK-HR-002) minus task-LEARN training hours minus PTO; demand from TASK-PROJ-013 estimate
  - DEC-2032 2026-05-17 — Closed enum `matrix_run_status` = {running, completed, partial, failed}; cardinality 4
  - DEC-2033 2026-05-17 — Idempotent per (tenant_id, run_date); UNIQUE constraint
  - DEC-2034 2026-05-17 — memory audit kinds: res.matrix_run_started, res.matrix_member_computed, res.matrix_run_completed, res.matrix_run_failed

language: rust 1.81
service: cyberos/services/res/
new_files:
  - services/res/migrations/0001_capacity_demand_matrix.sql
  - services/res/src/matrix/mod.rs
  - services/res/src/matrix/computer.rs
  - services/res/src/matrix/batch_runner.rs
  - services/res/src/handlers/matrix_routes.rs
  - services/res/src/audit/matrix_events.rs
  - services/res/tests/matrix_computer_test.rs
  - services/res/tests/matrix_run_status_enum_cardinality_test.rs
  - services/res/tests/matrix_idempotent_test.rs
  - services/res/tests/matrix_capacity_calc_test.rs
  - services/res/tests/matrix_audit_emission_test.rs

modified_files:
  - services/res/src/lib.rs

allowed_tools:
  - file_read: services/{res,hr,proj,time,learn}/**
  - file_write: services/res/{src,tests,migrations}/**
  - bash: cd services/res && cargo test matrix

disallowed_tools:
  - mutate prior matrix run (per DEC-2030)
  - duplicate run same date (per DEC-2033)

effort_hours: 10
subtasks:
  - "0.4h: 0001_capacity_demand_matrix.sql"
  - "0.4h: matrix/mod.rs"
  - "1.2h: computer.rs (4-module join)"
  - "0.6h: batch_runner.rs"
  - "0.5h: handlers/matrix_routes.rs"
  - "0.4h: audit/matrix_events.rs"
  - "0.3h: members.rs hook"
  - "3.0h: tests — 5 test files"
  - "2.2h: docs + dashboard UI"
  - "1.0h: cron registration"

risk_if_skipped: "Without capacity matrix, allocation flies blind → over-allocation + burnout. Without DEC-2031 LEARN+PTO deduct, training time double-counted. Without DEC-2033 idempotency, cron retry doubles capacity."
---

## §1 — Description (BCP-14 normative)

The RES service **MUST** ship capacity matrix at `services/res/src/matrix/` joining HR + PROJ + TIME + LEARN nightly, per-member-week grid, 4 memory audit kinds.

1. **MUST** validate `matrix_run_status` against closed enum per DEC-2032.

2. **MUST** compute at `computer.rs::compute(member, iso_week)` per DEC-2031:
- capacity_hours = member.hours_per_week (TASK-HR-002) - approved PTO (TASK-HR-004) - LEARN training (TASK-LEARN-001)
- demand_hours = sum TASK-PROJ-013 estimates for member's assigned issues in week
- allocated_hours = sum task-TIME entries (actual + planned)

3. **MUST** run batch at `batch_runner.rs::run(tenant, run_date)` per DEC-2030:
- For each active member, compute next 12 weeks of capacity+demand+allocated
- Insert/update rows in matrix table
- Catch + isolate per-member failures

4. **MUST** define table at migration `0001`:
   ```sql
   CREATE TABLE res_capacity_matrix (
     matrix_row_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     member_id UUID NOT NULL,
     iso_week CHAR(8) NOT NULL,
     project_id UUID,  -- NULL for member-total row
     capacity_hours NUMERIC(5,2) NOT NULL,
     demand_hours NUMERIC(5,2) NOT NULL,
     allocated_hours NUMERIC(5,2) NOT NULL,
     run_date DATE NOT NULL,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     UNIQUE (tenant_id, member_id, iso_week, project_id, run_date)
   );
   CREATE INDEX matrix_member_week_idx ON res_capacity_matrix(tenant_id, member_id, iso_week);
   ALTER TABLE res_capacity_matrix ENABLE ROW LEVEL SECURITY;
   CREATE POLICY matrix_rls ON res_capacity_matrix
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON res_capacity_matrix FROM cyberos_app;

   CREATE TABLE res_matrix_runs (
     run_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     run_date DATE NOT NULL,
     status TEXT NOT NULL DEFAULT 'running'
       CHECK (status IN ('running','completed','partial','failed')),
     members_total INT NOT NULL DEFAULT 0,
     members_succeeded INT NOT NULL DEFAULT 0,
     started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     completed_at TIMESTAMPTZ,
     trace_id CHAR(32),
     UNIQUE (tenant_id, run_date)
   );
   ALTER TABLE res_matrix_runs ENABLE ROW LEVEL SECURITY;
   CREATE POLICY matrix_runs_rls ON res_matrix_runs
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON res_matrix_runs FROM cyberos_app;
   GRANT UPDATE (status, members_total, members_succeeded, completed_at) ON res_matrix_runs TO cyberos_app;
   ```

5. **MUST** schedule cron 04:00 tenant_tz via TASK-MCP-007.

6. **MUST** be idempotent per DEC-2033 (UNIQUE constraint).

7. **MUST** expose endpoints:
   ```text
   POST /v1/res/matrix/trigger             (CHRO manual trigger)
   GET  /v1/res/matrix/runs/{id}           (status)
   GET  /v1/res/members/{id}/capacity      (member capacity view)
   ```

8. **MUST** emit 4 memory audit kinds per DEC-2034. PII per TASK-MEMORY-111: hours SHA-256 hashed.

9. **MUST** thread trace_id from cron → batch → per-member compute → audit.

10. **MUST NOT** mutate prior matrix row per DEC-2030 (append-only).

11. **MUST NOT** double-count training hours (LEARN deducts from capacity, not demand).

---

## §2 — Why this design

**Why join 4 modules (DEC-2030)?** True capacity needs all sources — HR provides headcount, PROJ demand, TIME actual, LEARN time-off-for-training.

**Why per-week (DEC-2030)?** Weekly granularity matches sprint cadence; daily too noisy, monthly too coarse.

**Why nightly (DEC-2030)?** Allocation decisions need fresh data Monday; nightly batch guarantees ≤24h lag.

**Why idempotent (DEC-2033)?** Cron retry must not double-count; UNIQUE enforces.

---

## §3 — API contract

Sample capacity view:
```json
{
  "member_id": "uuid",
  "weeks": [
    {
      "iso_week": "2026-W20",
      "capacity_hours": 40,
      "demand_hours": 35,
      "allocated_hours": 33,
      "utilization_pct": 82.5,
      "per_project": [
        {"project_id": "uuid-a", "allocated_hours": 20},
        {"project_id": "uuid-b", "allocated_hours": 13}
      ]
    }
  ]
}
```

---

## §4 — Acceptance criteria
1. **matrix_run_status enum cardinality 4**. 2. **Capacity = hours_per_week - PTO - LEARN**. 3. **Demand = sum PROJ-013 estimates**. 4. **Allocated = sum TIME entries**. 5. **Per-member 12-week forecast**. 6. **Per-project breakdown row + total row**. 7. **Nightly cron 04:00**. 8. **Idempotent via UNIQUE**. 9. **Per-member failure isolated**. 10. **4 memory audit kinds emitted**. 11. **PII scrubbed (hours SHA256)**. 12. **RLS denies cross-tenant**. 13. **CHRO-only manual trigger**. 14. **Trace_id preserved**. 15. **Append-only matrix table**. 16. **rust_decimal precision**. 17. **Inactive member skipped**. 18. **Contract type override respected (TASK-HR-002)**. 19. **Run status transitions correct**. 20. **Empty tenant skipped**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn capacity_minus_pto_and_learn() {
    let ctx = TestContext::member_40h_with_pto_4h_learn_2h().await;
    ctx.run_matrix_batch(today()).await;
    let row = ctx.fetch_matrix_row(ctx.member_id, this_week()).await;
    assert_eq!(row.capacity_hours, dec!(34.0));  // 40 - 4 - 2
}

#[tokio::test]
async fn idempotent_double_run() {
    let ctx = TestContext::with_active_member().await;
    ctx.run_matrix_batch(today()).await;
    ctx.run_matrix_batch(today()).await;
    let rows = ctx.fetch_matrix_rows(ctx.member_id, this_week()).await;
    let count = rows.iter().filter(|r| r.run_date == today()).count();
    assert!(count <= 12);  // 12 weeks per member; not doubled
}

#[tokio::test]
async fn per_member_failure_isolated() {
    let ctx = TestContext::with_5_members_one_will_fail().await;
    ctx.run_matrix_batch(today()).await;
    let run = ctx.fetch_latest_run().await;
    assert_eq!(run.members_succeeded, 4);
    assert_eq!(run.status, "partial");
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-HR-001, TASK-PROJ-001, TASK-TIME-001. **Downstream:** TASK-RES-002 (Gantt UI), TASK-RES-003 (over/under flags), TASK-RES-005 (OT cap check). **Cross-module:** TASK-HR-002 (contract type), TASK-HR-004 (PTO), TASK-LEARN-001 (training), TASK-PROJ-013 (estimates), TASK-MCP-007 (cron), TASK-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Source module unavailable | catch | sev-2; null demand | retry |
| Cron skipped | catch-up next | inherent | inherent |
| Duplicate run | UNIQUE | skip | inherent |
| Decimal precision drift | rust_decimal | inherent | inherent |
| Member just hired | no history | use current contract | inherent |
| 0 active members | skip | inherent | inherent |
| Cross-tenant query | RLS | 0 rows | inherent |
| Mid-batch crash | resume from last | partial | retry |
| Future week with no projects | inherent | demand=0 | inherent |
| Negative capacity (over-PTO) | clamp at 0 | sev-3 | data fix |

## §11 — Implementation notes
- §11.1 12-week forecast horizon balances visibility + churn (closer weeks more reliable).
- §11.2 Run via TASK-MCP-007 `kind: 'res.matrix_batch'`, daily 04:00.
- §11.3 memory audit body: tenant_id, run_id, member_id, week; hours SHA256.
- §11.4 Computer is pure function: `(member_data, week, hr_pto, learn_hours, proj_estimates, time_entries) → MatrixRow`.
- §11.5 Per-project rows enable Gantt UI per TASK-RES-002.

---

*End of TASK-RES-001 spec.*
