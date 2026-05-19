---
id: FR-REW-005
title: "REW monthly payroll compute + CFO+CHRO co-sign commit gate — orchestrates 3P + deductions + net pay with dual-sign before bank send"
module: REW
priority: MUST
status: draft
verify: T
phase: P2
milestone: P2 · slice 2
slice: 2
owner: Stephen Cheng (CFO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_frs: [FR-REW-001, FR-REW-002, FR-REW-004, FR-REW-006, FR-REW-009, FR-MEMORY-111]
depends_on: [FR-REW-001]
blocks: [FR-REW-006]

source_pages:
  - website/docs/modules/rew.html#payroll-compute

source_decisions:
  - DEC-2190 2026-05-17 — Monthly payroll compute orchestrates: P1+P2 + accrued P3 → gross → deductions (FR-REW-004) → net per member
  - DEC-2191 2026-05-17 — Closed enum `payroll_status` = {drafting, computed, cfo_signed, chro_signed, committed, paid, failed}; cardinality 7
  - DEC-2192 2026-05-17 — Commit (= ready-to-send to bank) requires CFO + CHRO dual-sign; same-person rejected
  - DEC-2193 2026-05-17 — Compute IMMUTABLE after committed; corrections via new payroll run (prior-period adjustment)
  - DEC-2194 2026-05-17 — memory audit kinds: rew.payroll_drafted, rew.payroll_computed, rew.payroll_signed, rew.payroll_committed, rew.payroll_paid, rew.payroll_failed

build_envelope:
  language: rust 1.81
  service: cyberos/services/rew/
  new_files:
    - services/rew/migrations/0005_payroll_runs.sql
    - services/rew/src/payroll/mod.rs
    - services/rew/src/payroll/compute.rs
    - services/rew/src/payroll/dual_sign_gate.rs
    - services/rew/src/handlers/payroll_routes.rs
    - services/rew/src/audit/payroll_events.rs
    - services/rew/tests/payroll_status_enum_cardinality_test.rs
    - services/rew/tests/payroll_compute_3p_test.rs
    - services/rew/tests/payroll_dual_sign_test.rs
    - services/rew/tests/payroll_same_person_rejected_test.rs
    - services/rew/tests/payroll_immutable_post_commit_test.rs
    - services/rew/tests/payroll_audit_emission_test.rs

  modified_files:
    - services/rew/src/lib.rs

  allowed_tools:
    - file_read: services/{rew,hr}/**
    - file_write: services/rew/{src,tests,migrations}/**
    - bash: cd services/rew && cargo test payroll

  disallowed_tools:
    - commit without dual-sign (per DEC-2192)
    - mutate committed payroll (per DEC-2193)

effort_hours: 8
sub_tasks:
  - "0.4h: 0005_payroll_runs.sql"
  - "0.4h: payroll/mod.rs"
  - "0.8h: compute.rs"
  - "0.5h: dual_sign_gate.rs"
  - "0.4h: handlers/payroll_routes.rs"
  - "0.4h: audit/payroll_events.rs"
  - "3.0h: tests — 6 test files"
  - "1.7h: CFO+CHRO UI for review + sign + docs"
  - "0.4h: cron registration"

risk_if_skipped: "Without orchestrator, payroll error-prone manual. Without DEC-2192 dual-sign, single-signer disaster. Without DEC-2193 immutability, post-commit edits break reconciliation."
---

## §1 — Description (BCP-14 normative)

The REW service **MUST** ship monthly payroll compute at `services/rew/src/payroll/` with 3P orchestration + dual-sign commit + immutable post-commit, 6 memory audit kinds.

1. **MUST** validate `payroll_status` against closed enum per DEC-2191.

2. **MUST** compute at `compute.rs::compute(tenant, period)` per DEC-2190:
   - For each active member: gross = P1 + P2 + P3 accruals
   - Call FR-REW-004 deductions
   - net = gross - total_deductions
   - Store per-member payslip data

3. **MUST** require CFO + CHRO dual-sign at `dual_sign_gate.rs::can_commit(payroll)` per DEC-2192:
   - Both signed
   - Same person rejected

4. **MUST** make immutable post-committed per DEC-2193 — REVOKE UPDATE on payroll_runs after status=committed (via trigger or app-layer check).

5. **MUST** define tables at migration `0005`:
   ```sql
   CREATE TABLE rew_payroll_runs (
     run_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     period_yyyymm CHAR(7) NOT NULL,
     status TEXT NOT NULL DEFAULT 'drafting'
       CHECK (status IN ('drafting','computed','cfo_signed','chro_signed','committed','paid','failed')),
     total_gross_vnd BIGINT,
     total_net_vnd BIGINT,
     members_count INT NOT NULL DEFAULT 0,
     cfo_signed_by UUID,
     cfo_signed_at TIMESTAMPTZ,
     chro_signed_by UUID,
     chro_signed_at TIMESTAMPTZ,
     committed_at TIMESTAMPTZ,
     paid_at TIMESTAMPTZ,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     UNIQUE (tenant_id, period_yyyymm)
   );
   ALTER TABLE rew_payroll_runs ENABLE ROW LEVEL SECURITY;
   CREATE POLICY runs_rls ON rew_payroll_runs
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON rew_payroll_runs FROM cyberos_app;
   GRANT UPDATE (status, total_gross_vnd, total_net_vnd, members_count, cfo_signed_by, cfo_signed_at, chro_signed_by, chro_signed_at, committed_at, paid_at) ON rew_payroll_runs TO cyberos_app;

   CREATE TABLE rew_payslip_rows (
     payslip_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     run_id UUID NOT NULL REFERENCES rew_payroll_runs(run_id),
     member_id UUID NOT NULL,
     gross_vnd BIGINT NOT NULL,
     deductions_total_vnd BIGINT NOT NULL,
     net_vnd BIGINT NOT NULL,
     deductions_jsonb JSONB NOT NULL,
     income_components_jsonb JSONB NOT NULL,
     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     UNIQUE (run_id, member_id)
   );
   ALTER TABLE rew_payslip_rows ENABLE ROW LEVEL SECURITY;
   CREATE POLICY payslip_rls ON rew_payslip_rows
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON rew_payslip_rows FROM cyberos_app;
   ```

6. **MUST** expose endpoints:
   ```text
   POST /v1/rew/payroll/runs                  (CFO draft new run)
   POST /v1/rew/payroll/runs/{id}/compute     (CFO trigger compute)
   POST /v1/rew/payroll/runs/{id}/cfo-sign
   POST /v1/rew/payroll/runs/{id}/chro-sign
   POST /v1/rew/payroll/runs/{id}/commit      (auto if both signed)
   GET  /v1/rew/payroll/runs/{id}             (status + summary)
   ```

7. **MUST** emit 6 memory audit kinds per DEC-2194. PII per FR-MEMORY-111: amounts SHA256.

8. **MUST** thread trace_id from draft → compute → sign → commit → audit.

9. **MUST NOT** commit without dual-sign per DEC-2192.

10. **MUST NOT** mutate committed payroll per DEC-2193.

11. **MUST** be deterministic — replay produces byte-identical totals per FR-REW-002.

---

## §2 — Why this design

**Why dual-sign (DEC-2192)?** Payroll = largest single financial event monthly; single-signer governance gap.

**Why immutable post-commit (DEC-2193)?** Bank send happens; mutating after creates reconciliation chaos.

**Why prior-period adjustment via new run (DEC-2193)?** Corrections create new run with negative entries; preserves history.

**Why 7-state lifecycle (DEC-2191)?** Captures real workflow: draft → compute → review → sign → commit → bank → paid.

---

## §3 — API contract

Sample payroll run:
```json
{
  "run_id": "uuid",
  "period_yyyymm": "2026-06",
  "status": "computed",
  "total_gross_vnd": 1200000000,
  "total_net_vnd": 925000000,
  "members_count": 30
}
```

---

## §4 — Acceptance criteria
1. **payroll_status enum cardinality 7**. 2. **Compute orchestrates 3P + deductions**. 3. **Net = gross - deductions**. 4. **CFO + CHRO dual-sign required for commit**. 5. **Same person rejected**. 6. **UNIQUE(tenant, period_yyyymm)**. 7. **6 memory audit kinds emitted**. 8. **PII scrubbed (amounts SHA256)**. 9. **RLS denies cross-tenant**. 10. **Trace_id preserved**. 11. **Append-only via REVOKE except status cols**. 12. **Immutable post-commit (status can advance only)**. 13. **Bigint VND**. 14. **Deterministic replay**. 15. **Per-member payslip row stored**. 16. **Deductions JSONB + income_components JSONB cached**. 17. **CFO-only draft + compute**. 18. **Status workflow enforced**. 19. **Failed compute → status=failed; reason logged**. 20. **Correction via new run (prior-period adjustment pattern)**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn dual_sign_required_for_commit() {
    let ctx = TestContext::with_computed_payroll().await;
    let r = ctx.try_commit_without_signs(ctx.run_id).await;
    assert!(r.is_err());
    ctx.cfo_sign(ctx.run_id).await;
    let r2 = ctx.try_commit(ctx.run_id).await;
    assert!(r2.is_err());  // CHRO missing
    ctx.chro_sign(ctx.run_id).await;
    let r3 = ctx.commit(ctx.run_id).await;
    assert!(r3.is_ok());
}

#[tokio::test]
async fn immutable_post_commit() {
    let ctx = TestContext::with_committed_payroll().await;
    let r = ctx.try_mutate_payslip_row(ctx.payslip_id).await;
    assert!(r.is_err());
}

#[tokio::test]
async fn deterministic_replay() {
    let ctx = TestContext::with_member_data().await;
    let r1 = ctx.compute_payroll("2026-06").await;
    let r2 = ctx.compute_payroll("2026-06").await;
    assert_eq!(r1.total_gross_vnd, r2.total_gross_vnd);
    assert_eq!(r1.total_net_vnd, r2.total_net_vnd);
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** FR-REW-001.
**Cross-module:** FR-REW-002 (versioning), FR-REW-004 (deductions), FR-REW-006 (PDF render), FR-REW-009 (bank send), FR-AUTH-101 (CFO/CHRO), FR-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Compute fails mid-run | rollback | status=failed | retry |
| One signer missing | gate | reject commit | wait |
| Same-person dual-sign | validate | 403 | different signer |
| Duplicate period | UNIQUE | 409 | use different period |
| Post-commit mutation | REVOKE | DB error | inherent |
| Cross-tenant query | RLS | 0 rows | inherent |
| Decimal precision | bigint VND | inherent | inherent |
| Status workflow violation | check | 409 | follow order |
| Member missing comp | sev-2 | exclude or fail | data fix |
| Decryption fail | cascade | status=failed | KMS check |

## §11 — Implementation notes
- §11.1 Compute reads encrypted comp via FR-REW-001 decrypt (CFO-only context); uses session-scoped key.
- §11.2 Net calc: gross - sum(deductions); validate sum matches.
- §11.3 memory audit body: run_id, period, status, members_count; amounts SHA256.
- §11.4 Prior-period adjustment: new run with kind='correction' + reference to prior; negative entries allowed.
- §11.5 FR-REW-006 PDF generates per payslip_row post-commit.

---

*End of FR-REW-005 spec.*
