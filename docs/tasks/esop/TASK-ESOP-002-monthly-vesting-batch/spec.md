---
id: TASK-ESOP-002
title: "ESOP monthly vesting accrual deterministic batch — runs EOM tenant_tz computing per-grant vested shares with cliff respect + immutable accrual rows"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-05-17T00:00:00+07:00
department: engineering
author: @stephencheng
template: task@1
module: ESOP
priority: p0
status: draft
verify: T
phase: P2
milestone: P2 · slice 1
slice: 1
owner: Stephen Cheng (CFO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-ESOP-001, TASK-MCP-007, TASK-MEMORY-111]
depends_on: [TASK-ESOP-001]
blocks: []

source_pages:
  - website/docs/modules/esop.html#vesting-accrual

source_decisions:
  - DEC-2260 2026-05-17 — Monthly EOM cron computes per-grant vested shares: 0 if pre-cliff; else (months_since_cliff / (vest_months - cliff_months)) * total_shares
  - DEC-2261 2026-05-17 — Closed enum `accrual_status` = {running, completed, partial, failed}; cardinality 4
  - DEC-2262 2026-05-17 — Deterministic per (grant, year_month); UNIQUE constraint enforces idempotency
  - DEC-2263 2026-05-17 — Status auto-advance: if vested_shares >= total_shares, mark TASK-ESOP-001 grant.status = fully_vested
  - DEC-2264 2026-05-17 — memory audit kinds: esop.accrual_batch_started, esop.accrual_row_created, esop.accrual_grant_fully_vested, esop.accrual_batch_completed, esop.accrual_batch_failed

build_envelope:
  language: rust 1.81
  service: cyberos/services/esop/
  new_files:
    - services/esop/migrations/0002_vesting_accruals.sql
    - services/esop/src/vesting/mod.rs
    - services/esop/src/vesting/calculator.rs
    - services/esop/src/vesting/batch_runner.rs
    - services/esop/src/audit/vesting_events.rs
    - services/esop/tests/vesting_pre_cliff_zero_test.rs
    - services/esop/tests/vesting_post_cliff_linear_test.rs
    - services/esop/tests/accrual_status_enum_cardinality_test.rs
    - services/esop/tests/vesting_idempotent_test.rs
    - services/esop/tests/vesting_fully_vested_transition_test.rs
    - services/esop/tests/vesting_audit_emission_test.rs

  modified_files:
    - services/esop/src/lib.rs

  allowed_tools:
    - file_read: services/esop/**
    - file_write: services/esop/{src,tests,migrations}/**
    - bash: cd services/esop && cargo test vesting

  disallowed_tools:
    - mutate prior accrual (per DEC-2262)
    - non-deterministic calc (per DEC-2260)

effort_hours: 4
subtasks:
  - "0.3h: 0002_vesting_accruals.sql"
  - "0.3h: vesting/mod.rs"
  - "0.5h: calculator.rs"
  - "0.4h: batch_runner.rs"
  - "0.3h: audit/vesting_events.rs"
  - "1.8h: tests — 6 test files"
  - "0.4h: docs + cron"

risk_if_skipped: "Without monthly batch, vesting tracking manual → error-prone. Without DEC-2262 idempotency, double-run double-credits. Without DEC-2263 auto-transition, fully-vested grants stay 'active' indefinitely."
---

## §1 — Description (BCP-14 normative)

The ESOP service **MUST** ship monthly vesting batch at `services/esop/src/vesting/` running EOM cron, deterministic calc, immutable accrual rows, 5 memory audit kinds.

1. **MUST** validate `accrual_status` against closed enum per DEC-2261.

2. **MUST** compute at `calculator.rs::compute(grant, as_of_date)` per DEC-2260:
   - months_elapsed = months_between(grant.vest_start_date, as_of_date)
   - if months_elapsed < cliff_months: vested = 0
   - else if months_elapsed >= vest_months: vested = total_shares
   - else: vested = (months_elapsed / vest_months) * total_shares

3. **MUST** schedule EOM cron via TASK-MCP-007 at 04:30 tenant_tz.

4. **MUST** be idempotent per DEC-2262 — UNIQUE(grant_id, year_month).

5. **MUST** auto-transition to fully_vested per DEC-2263 when vested >= total.

6. **MUST** define table at migration `0002`:
   ```sql
   CREATE TABLE esop_vesting_accruals (
     accrual_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     grant_id UUID NOT NULL REFERENCES esop_sp_grants(grant_id),
     year_month CHAR(7) NOT NULL,
     vested_cumulative BIGINT NOT NULL,
     unvested_remaining BIGINT NOT NULL,
     monthly_vested BIGINT NOT NULL,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     UNIQUE (tenant_id, grant_id, year_month)
   );
   CREATE INDEX accruals_grant_idx ON esop_vesting_accruals(tenant_id, grant_id, year_month DESC);
   ALTER TABLE esop_vesting_accruals ENABLE ROW LEVEL SECURITY;
   CREATE POLICY accruals_rls ON esop_vesting_accruals
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON esop_vesting_accruals FROM cyberos_app;

   CREATE TABLE esop_vesting_batch_runs (
     run_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     year_month CHAR(7) NOT NULL,
     status TEXT NOT NULL DEFAULT 'running'
       CHECK (status IN ('running','completed','partial','failed')),
     grants_processed INT NOT NULL DEFAULT 0,
     grants_failed INT NOT NULL DEFAULT 0,
     started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     completed_at TIMESTAMPTZ,
     trace_id CHAR(32),
     UNIQUE (tenant_id, year_month)
   );
   ALTER TABLE esop_vesting_batch_runs ENABLE ROW LEVEL SECURITY;
   CREATE POLICY runs_rls ON esop_vesting_batch_runs
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON esop_vesting_batch_runs FROM cyberos_app;
   GRANT UPDATE (status, grants_processed, grants_failed, completed_at) ON esop_vesting_batch_runs TO cyberos_app;
   ```

7. **MUST** expose endpoints:
   ```text
   POST /v1/esop/vesting/run-batch   (CFO manual trigger)
   GET  /v1/esop/grants/{id}/accruals (history per grant)
   ```

8. **MUST** emit 5 memory audit kinds per DEC-2264. PII per TASK-MEMORY-111: vested_cumulative SHA256.

9. **MUST** thread trace_id from cron → calc → audit.

10. **MUST NOT** mutate prior accrual per DEC-2262 (REVOKE UPDATE/DELETE).

11. **MUST NOT** use non-deterministic inputs per DEC-2260 (no now()).

---

## §2 — Why this design

**Why deterministic (DEC-2260)?** Member challenges vesting → replay must yield same numbers.

**Why monthly cron (DEC-2260)?** Industry standard — monthly granularity matches grant agreements.

**Why auto-transition (DEC-2263)?** Without auto-flag, fully-vested grants stay "active" — TASK-ESOP-004 put-option needs flag.

---

## §3 — API contract

Sample accrual:
```json
{
  "accrual_id": "uuid",
  "grant_id": "uuid",
  "year_month": "2026-06",
  "vested_cumulative": 1666,  // 8/48 of 10000
  "unvested_remaining": 8334,
  "monthly_vested": 208
}
```

---

## §4 — Acceptance criteria
1. **accrual_status enum cardinality 4**. 2. **Pre-cliff → vested=0**. 3. **Post-cliff linear**. 4. **At vest_months → vested=total**. 5. **Idempotent UNIQUE(grant, month)**. 6. **EOM cron 04:30**. 7. **Auto-fully_vested transition**. 8. **5 memory audit kinds emitted**. 9. **PII scrubbed (counts SHA256)**. 10. **RLS denies cross-tenant**. 11. **CFO-only manual trigger**. 12. **Trace_id preserved**. 13. **Append-only via REVOKE**. 14. **Deterministic (no now)**. 15. **Per-grant failure isolated**. 16. **Bigint shares**. 17. **Cancelled grants skipped**. 18. **Accelerated grants handled separately (TASK-ESOP-005)**. 19. **History per grant queryable**. 20. **vest_start_date used (not grant_date)**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn pre_cliff_zero_vested() {
    let g = ctx.grant_active("2026-01-01", 48, 12, 10000).await;
    ctx.run_batch("2026-06").await;  // 5 months in, < 12 cliff
    let a = ctx.fetch_accrual(g.id, "2026-06").await;
    assert_eq!(a.vested_cumulative, 0);
}

#[tokio::test]
async fn post_cliff_linear() {
    let g = ctx.grant_active("2026-01-01", 48, 12, 10000).await;
    ctx.run_batch("2027-01").await;  // 12 months in
    let a = ctx.fetch_accrual(g.id, "2027-01").await;
    assert_eq!(a.vested_cumulative, 2500);  // 12/48 of 10000
}

#[tokio::test]
async fn idempotent_double_run() {
    let g = ctx.grant_active("2026-01-01", 48, 12, 10000).await;
    ctx.run_batch("2027-01").await;
    ctx.run_batch("2027-01").await;
    let accruals = ctx.fetch_accruals(g.id).await;
    assert_eq!(accruals.iter().filter(|a| a.year_month == "2027-01").count(), 1);
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-ESOP-001.
**Cross-module:** TASK-MCP-007 (cron), TASK-AUTH-101 (CFO), TASK-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Cron skipped | catch-up | inherent | inherent |
| Duplicate run | UNIQUE | skip | inherent |
| Cancelled grant in batch | filter | skip | inherent |
| Cross-tenant query | RLS | 0 rows | inherent |
| Decimal precision | bigint | inherent | inherent |
| Per-grant calc fail | isolate | partial | retry |
| Mid-batch crash | resume | partial | retry |
| vest_months = 0 | CHECK in TASK-ESOP-001 | prevented | inherent |
| Accelerated grant | exclude from monthly batch | handled by TASK-ESOP-005 | inherent |
| year_month format invalid | validate | 400 | YYYY-MM |

## §11 — Implementation notes
- §11.1 Calculator pure function: `(grant, as_of) → AccrualNumbers`.
- §11.2 Cron via TASK-MCP-007 `kind: 'esop.monthly_vesting'`, last day of month at 04:30.
- §11.3 Auto-transition: post-accrual, UPDATE TASK-ESOP-001 grants SET status='fully_vested' WHERE vested >= total.
- §11.4 memory audit body: grant_id, year_month, status; vested counts SHA256.
- §11.5 Backfill: CFO can trigger via year_month param to catch up missed months.

---

*End of TASK-ESOP-002 spec.*
