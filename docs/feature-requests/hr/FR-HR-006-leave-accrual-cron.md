---
id: FR-HR-006
title: "HR annual leave accrual nightly batch — Decree 145 formula (1d/month + 1d/5yr seniority bonus) with immutable accrual ledger"
module: HR
priority: MUST
status: draft
verify: T
phase: P1
milestone: P1 · slice 6
slice: 6
owner: Stephen Cheng (CHRO)
created: 2026-05-17
shipped: null
brain_chain_hash: null
related_frs: [FR-HR-004, FR-HR-005, FR-MCP-007, FR-BRAIN-111]
depends_on: [FR-HR-004]
blocks: []

source_pages:
  - website/docs/modules/hr.html#leave-accrual
  - https://thuvienphapluat.vn/  # Decree 145/2020 Art. 65 (leave accrual formula)

source_decisions:
  - DEC-1850 2026-05-17 — Nightly batch at 02:00 tenant_tz computes monthly accrual + seniority bonus per Decree 145 Art. 65
  - DEC-1851 2026-05-17 — Formula: base = 1d/month; seniority bonus = floor(years_of_service / 5) days/year (Art. 66 for certain industries — currently 0 default)
  - DEC-1852 2026-05-17 — Closed enum `accrual_kind` = {monthly_base, seniority_bonus, correction, carryover}; cardinality 4
  - DEC-1853 2026-05-17 — Per-month accrual = IMMUTABLE row; corrections via new `correction` kind row (sign + reason)
  - DEC-1854 2026-05-17 — Idempotency: one accrual row per (member_id, year_month, kind); UNIQUE constraint
  - DEC-1855 2026-05-17 — BRAIN audit kinds: hr.accrual_batch_started, hr.accrual_added, hr.accrual_correction_added, hr.accrual_batch_failed

build_envelope:
  language: rust 1.81
  service: cyberos/services/hr/
  new_files:
    - services/hr/migrations/0006_leave_accrual_ledger.sql
    - services/hr/src/accrual/mod.rs
    - services/hr/src/accrual/monthly_batch.rs
    - services/hr/src/accrual/correction_handler.rs
    - services/hr/src/handlers/accrual_routes.rs
    - services/hr/src/audit/accrual_events.rs
    - services/hr/tests/accrual_monthly_test.rs
    - services/hr/tests/accrual_seniority_bonus_test.rs
    - services/hr/tests/accrual_idempotency_test.rs
    - services/hr/tests/accrual_kind_enum_cardinality_test.rs
    - services/hr/tests/accrual_correction_test.rs
    - services/hr/tests/accrual_audit_emission_test.rs

  modified_files:
    - services/hr/src/lib.rs

  allowed_tools:
    - file_read: services/hr/**
    - file_write: services/hr/{src,tests,migrations}/**
    - bash: cd services/hr && cargo test accrual

  disallowed_tools:
    - mutate prior accrual row (per DEC-1853)
    - duplicate accrual per month (per DEC-1854)

effort_hours: 4
sub_tasks:
  - "0.3h: 0006_leave_accrual_ledger.sql"
  - "0.3h: accrual/mod.rs"
  - "0.6h: monthly_batch.rs"
  - "0.4h: correction_handler.rs"
  - "0.3h: handlers/accrual_routes.rs"
  - "0.3h: audit/accrual_events.rs"
  - "1.6h: tests — 6 test files"
  - "0.2h: cron registration"

risk_if_skipped: "Without nightly accrual, members never get monthly leave days → Labour Code violation. Without DEC-1853 immutability, prior accrual mutations corrupt audit. Without DEC-1854 idempotency, duplicate runs double-credit (over-payment risk)."
---

## §1 — Description (BCP-14 normative)

The HR service **MUST** ship leave accrual at `services/hr/src/accrual/` running nightly via FR-MCP-007 cron, computing base + seniority per Decree 145, immutable ledger, 4 BRAIN audit kinds.

1. **MUST** schedule batch at 02:00 tenant_tz per DEC-1850.

2. **MUST** validate `accrual_kind` against closed enum per DEC-1852.

3. **MUST** compute per-month accrual at `monthly_batch.rs::accrue(member, year_month)`:
   - base = 1.0d (Decree 145 Art. 65) × pro-rate (FR-HR-002 contract type override)
   - seniority = floor(years_of_service / 5) × 1.0d (Art. 66; default 0 unless industry special)
   - skip if member.is_active=false for that month

4. **MUST** be idempotent per DEC-1854 — UNIQUE on (member_id, year_month, kind); ON CONFLICT DO NOTHING.

5. **MUST** support correction at `correction_handler.rs::add_correction(member, year_month, days, reason)` per DEC-1853 — new row with kind='correction' + sign.

6. **MUST** define table at migration `0006`:
   ```sql
   CREATE TABLE hr_leave_accrual_ledger (
     ledger_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     member_id UUID NOT NULL,
     year_month CHAR(7) NOT NULL,  -- 'YYYY-MM'
     kind TEXT NOT NULL
       CHECK (kind IN ('monthly_base','seniority_bonus','correction','carryover')),
     days_added NUMERIC(5,2) NOT NULL,
     reason TEXT,
     applied_by UUID,  -- system or CHRO uuid
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     UNIQUE (tenant_id, member_id, year_month, kind)
   );
   CREATE INDEX accrual_member_year_idx ON hr_leave_accrual_ledger(tenant_id, member_id, year_month DESC);
   ALTER TABLE hr_leave_accrual_ledger ENABLE ROW LEVEL SECURITY;
   CREATE POLICY accrual_rls ON hr_leave_accrual_ledger
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON hr_leave_accrual_ledger FROM cyberos_app;
   -- Append-only per DEC-1853
   ```

7. **MUST** expose endpoints:
   ```text
   POST   /v1/hr/accrual/run-batch              (manual trigger, CHRO)
   POST   /v1/hr/accrual/corrections            (CHRO; new correction row)
   GET    /v1/hr/members/{id}/accrual-ledger    (history)
   ```

8. **MUST** emit 4 BRAIN audit kinds per DEC-1855. PII per FR-BRAIN-111: reason SHA-256 hashed.

9. **MUST** thread trace_id from cron → batch → ledger insert → audit.

10. **MUST NOT** mutate prior accrual per DEC-1853.

11. **MUST NOT** double-credit per DEC-1854 (UNIQUE).

---

## §2 — Why this design

**Why nightly cron (DEC-1850)?** Daily run catches month-end transitions; weekly delay = members short on leave for 7d.

**Why immutable ledger (DEC-1853)?** Audit lineage; corrections via new row preserve "who changed what when".

**Why idempotent (DEC-1854)?** Cron may retry on failure; double-run must not double-credit.

**Why seniority bonus (DEC-1851)?** Decree 145 Art. 66 mandates for hazardous/heavy industries; default 0 for office, configurable per tenant.

---

## §3 — API contract

Sample accrual ledger row:
```json
{
  "ledger_id": "uuid",
  "member_id": "uuid",
  "year_month": "2026-06",
  "kind": "monthly_base",
  "days_added": 1.00,
  "applied_by": "system",
  "created_at": "2026-07-01T02:00:00Z"
}
```

Sample correction:
```json
POST /v1/hr/accrual/corrections
{
  "member_id": "uuid",
  "year_month": "2026-05",
  "days_added": 0.5,
  "reason": "Late hire mid-May; pro-rate adjustment"
}
```

---

## §4 — Acceptance criteria
1. **Nightly batch 02:00 tenant_tz**. 2. **1d/month base accrual**. 3. **Seniority bonus configurable per tenant (default 0)**. 4. **Pro-rate respects FR-HR-002 contract type**. 5. **Inactive members skipped**. 6. **Idempotent via UNIQUE**. 7. **Correction kind allows manual adj**. 8. **kind enum cardinality 4**. 9. **4 BRAIN audit kinds emitted**. 10. **PII scrubbed (reason SHA256)**. 11. **RLS denies cross-tenant**. 12. **Trace_id preserved**. 13. **Append-only via REVOKE**. 14. **History query desc time**. 15. **CHRO-only manual trigger**. 16. **CHRO-only correction**. 17. **Year_month format 'YYYY-MM' enforced**. 18. **Cron skip if 0 active members**. 19. **Days_added precision 2 decimal (rust_decimal)**. 20. **Carryover kind for year-end roll**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn monthly_base_accrual() {
    let ctx = TestContext::with_active_indefinite_member().await;
    ctx.run_accrual_batch(ctx.tenant_id, "2026-06").await;
    let ledger = ctx.fetch_ledger(ctx.member_id).await;
    assert!(ledger.iter().any(|r| r.kind == "monthly_base" && r.year_month == "2026-06" && r.days_added == dec!(1.0)));
}

#[tokio::test]
async fn idempotent_double_run() {
    let ctx = TestContext::with_active_member().await;
    ctx.run_accrual_batch(ctx.tenant_id, "2026-06").await;
    ctx.run_accrual_batch(ctx.tenant_id, "2026-06").await;
    let ledger = ctx.fetch_ledger(ctx.member_id).await;
    let count = ledger.iter().filter(|r| r.kind == "monthly_base" && r.year_month == "2026-06").count();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn correction_creates_new_row() {
    let ctx = TestContext::with_accrual_for_member("2026-05").await;
    ctx.add_correction(ctx.member_id, "2026-05", dec!(0.5), "late hire").await;
    let ledger = ctx.fetch_ledger(ctx.member_id).await;
    assert!(ledger.iter().any(|r| r.kind == "correction" && r.days_added == dec!(0.5)));
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** FR-HR-004.
**Cross-module:** FR-HR-002 (contract pro-rate), FR-MCP-007 (cron), FR-AUTH-101 (CHRO), FR-BRAIN-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Batch fails mid-run | partial inserts | sev-2; resume from last successful | re-run safe |
| Duplicate cron instance | UNIQUE | second skipped | inherent |
| Member inactive mid-month | skip | inherent | data integrity |
| Year_month format wrong | validate | reject 400 | use YYYY-MM |
| Correction without reason | warn but allow | inherent | audit visible |
| Cross-tenant correction | RLS | 403 | inherent |
| Days_added negative | allow (correction reduces) | inherent | inherent |
| Pro-rate computation wrong | unit tests | inherent | code fix |
| Cron skipped (system down) | catch-up next boot | inherent | inherent |
| Seniority bonus misconfigured | default 0 | inherent | tenant config fix |

## §11 — Implementation notes
- §11.1 Cron via FR-MCP-007 `kind: 'hr.accrual_batch'`, daily 02:00 tenant_tz.
- §11.2 Year-end carryover: separate cron at Jan 1, kind='carryover', moves unused annual days per policy (max 5d typically).
- §11.3 BRAIN audit body: tenant_id, year_month, members_count; reason SHA256.
- §11.4 Manual trigger for backfill: CHRO specifies year_month range; iterates each month.
- §11.5 Days_added uses rust_decimal (2 decimal places); never float.

---

*End of FR-HR-006 spec.*
