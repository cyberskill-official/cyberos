---
id: FR-REW-004
title: "REW statutory deductions — BHXH 10.5% + BHYT 1.5% + BHTN 1% + PIT progressive per Decree 152/2020 with FR-HR-005 policy lookup"
module: REW
priority: MUST
status: draft
verify: T
phase: P2
milestone: P2 · slice 1
slice: 1
owner: Stephen Cheng (CFO)
created: 2026-05-17
shipped: null
brain_chain_hash: null
related_frs: [FR-HR-005, FR-REW-002, FR-REW-005, FR-BRAIN-111]
depends_on: [FR-HR-005]
blocks: []

source_pages:
  - website/docs/modules/rew.html#statutory
  - https://thuvienphapluat.vn/  # Decree 152/2020

source_decisions:
  - DEC-2180 2026-05-17 — Compute statutory deductions per Decree 152/2020 employee-side rates: BHXH 8% + BHYT 1.5% + BHTN 1% = 10.5% (NOT 17.5% which is employer-side) + PIT progressive
  - DEC-2181 2026-05-17 — Closed enum `deduction_kind` = {bhxh, bhyt, bhtn, pit, union_due, voluntary}; cardinality 6
  - DEC-2182 2026-05-17 — Rate lookups via FR-HR-005 + FR-REW-002 versioned policy; deterministic per period
  - DEC-2183 2026-05-17 — Contractor (per FR-HR-002 contract_type) exempt from BHXH/BHYT/BHTN; still subject to PIT
  - DEC-2184 2026-05-17 — BRAIN audit kinds: rew.deduction_computed, rew.deduction_skipped_contractor, rew.deduction_compute_failed

build_envelope:
  language: rust 1.81
  service: cyberos/services/rew/
  new_files:
    - services/rew/migrations/0004_deductions.sql
    - services/rew/src/deductions/mod.rs
    - services/rew/src/deductions/computer.rs
    - services/rew/src/deductions/pit_progressive.rs
    - services/rew/src/audit/deductions_events.rs
    - services/rew/tests/deduction_kind_enum_cardinality_test.rs
    - services/rew/tests/bhxh_8pct_test.rs
    - services/rew/tests/pit_progressive_brackets_test.rs
    - services/rew/tests/contractor_si_exempt_test.rs
    - services/rew/tests/deduction_deterministic_test.rs
    - services/rew/tests/deduction_audit_emission_test.rs

  modified_files:
    - services/rew/src/lib.rs

  allowed_tools:
    - file_read: services/{rew,hr}/**
    - file_write: services/rew/{src,tests,migrations}/**
    - bash: cd services/rew && cargo test deductions

  disallowed_tools:
    - deduct SI from contractors (per DEC-2183)
    - use unversioned rates (per DEC-2182)

effort_hours: 6
sub_tasks:
  - "0.3h: 0004_deductions.sql"
  - "0.3h: deductions/mod.rs"
  - "0.6h: computer.rs"
  - "0.6h: pit_progressive.rs"
  - "0.3h: audit/deductions_events.rs"
  - "2.5h: tests — 6 test files"
  - "1.0h: docs"
  - "0.4h: integration test with FR-REW-005"

risk_if_skipped: "Without statutory deductions, payroll non-compliant → tax authority fines. Without DEC-2182 versioning, replay drift. Without DEC-2183 contractor exemption, over-deducting illegal."
---

## §1 — Description (BCP-14 normative)

The REW service **MUST** ship statutory deductions at `services/rew/src/deductions/` computing BHXH + BHYT + BHTN + PIT per Decree 152/2020 with versioned rates, 3 BRAIN audit kinds.

1. **MUST** validate `deduction_kind` against closed enum per DEC-2181.

2. **MUST** compute at `computer.rs::compute(member, gross_taxable, period)` per DEC-2180:
   - Read rates from FR-REW-002 / FR-HR-005 policy at period start
   - Default rates per Decree 152: BHXH 8%, BHYT 1.5%, BHTN 1%
   - PIT via `pit_progressive.rs::compute(taxable_income, brackets)`

3. **MUST** check contractor exemption per DEC-2183 — if `member.contract_type='contractor'`, skip BHXH/BHYT/BHTN; still compute PIT.

4. **MUST** be deterministic per DEC-2182 — pure function; same input + same policy version → same output.

5. **MUST** define table at migration `0004`:
   ```sql
   CREATE TABLE rew_deductions (
     deduction_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     payroll_run_id UUID NOT NULL,
     member_id UUID NOT NULL,
     kind TEXT NOT NULL
       CHECK (kind IN ('bhxh','bhyt','bhtn','pit','union_due','voluntary')),
     rate NUMERIC(7,6),
     base_amount_vnd BIGINT NOT NULL,
     deducted_amount_vnd BIGINT NOT NULL,
     policy_version_id UUID NOT NULL,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   CREATE INDEX deductions_payroll_member_idx ON rew_deductions(tenant_id, payroll_run_id, member_id);
   ALTER TABLE rew_deductions ENABLE ROW LEVEL SECURITY;
   CREATE POLICY deductions_rls ON rew_deductions
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON rew_deductions FROM cyberos_app;
   ```

6. **MUST** emit 3 BRAIN audit kinds per DEC-2184. PII per FR-BRAIN-111: amounts SHA-256 hashed.

7. **MUST** thread trace_id from FR-REW-005 compute → deductions → audit.

8. **MUST NOT** deduct SI from contractors per DEC-2183.

9. **MUST NOT** use unversioned rates per DEC-2182.

---

## §2 — Why this design

**Why employee-side rates (DEC-2180)?** Decree 152 distinguishes employer (17.5+4.5+2=24%) from employee (8+1.5+1=10.5%). Common bug: using wrong side.

**Why versioned (DEC-2182)?** Rates change annually; FR-REW-002 replay requires deterministic lookup.

**Why contractor exempt (DEC-2183)?** Decree 152 explicitly exempts contractors from SI participation.

---

## §3 — API contract

Sample deduction list (in payroll context):
```json
{
  "member_id": "uuid",
  "deductions": [
    {"kind": "bhxh", "rate": 0.08, "base": 30000000, "deducted": 2400000},
    {"kind": "bhyt", "rate": 0.015, "base": 30000000, "deducted": 450000},
    {"kind": "bhtn", "rate": 0.01, "base": 30000000, "deducted": 300000},
    {"kind": "pit", "rate": null, "base": 26850000, "deducted": 1842500}
  ],
  "total_deducted": 4992500
}
```

---

## §4 — Acceptance criteria
1. **deduction_kind enum cardinality 6**. 2. **BHXH 8% (employee-side, not 17.5%)**. 3. **BHYT 1.5%**. 4. **BHTN 1%**. 5. **PIT progressive brackets**. 6. **Contractor SI-exempt**. 7. **Contractor still PIT'd**. 8. **Versioned policy lookup**. 9. **Deterministic**. 10. **3 BRAIN audit kinds emitted**. 11. **PII scrubbed (amounts SHA256)**. 12. **RLS denies cross-tenant**. 13. **Trace_id preserved**. 14. **Append-only via REVOKE**. 15. **Bigint VND (no float)**. 16. **Rate precision (7,6)**. 17. **Bracket edge handling per Decree 152 Art. 7**. 18. **Policy version_id stored per deduction**. 19. **Multiple deductions per member per run**. 20. **Total = sum(deducted_amount_vnd) matches**. 

---

## §5 — Verification

```rust
#[tokio::test]
async fn bhxh_8pct_not_17() {
    let ctx = TestContext::with_gross_30m_vnd().await;
    let deductions = compute(ctx.member, dec!(30_000_000), period).await;
    let bhxh = deductions.iter().find(|d| d.kind == "bhxh").unwrap();
    assert_eq!(bhxh.deducted_amount_vnd, 2_400_000);
}

#[tokio::test]
async fn contractor_si_exempt() {
    let ctx = TestContext::with_contractor_gross_30m().await;
    let deductions = compute(ctx.member, dec!(30_000_000), period).await;
    assert!(!deductions.iter().any(|d| d.kind == "bhxh"));
    assert!(deductions.iter().any(|d| d.kind == "pit"));
}

#[tokio::test]
async fn pit_progressive_brackets() {
    let taxable = dec!(20_000_000);
    let pit = pit_progressive::compute(taxable, default_brackets()).await;
    // Per Decree 152 Art. 7:
    // 5M @ 5% = 250k
    // 5M @ 10% = 500k
    // 10M @ 15% = 1500k
    // Total: 2.25M
    assert_eq!(pit, dec!(2_250_000));
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** FR-HR-005.
**Downstream:** FR-REW-005 (payroll compute uses this).
**Cross-module:** FR-REW-002 (versioning), FR-HR-002 (contract type), FR-BRAIN-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Policy lookup fail | catch | sev-1; reject compute | retry |
| Bracket misconfigured | validator | reject | fix policy |
| Decimal precision | bigint | inherent | inherent |
| Contractor type missing | default exempt? no — reject | sev-2 | data fix |
| Cross-tenant query | RLS | 0 rows | inherent |
| Rate change mid-period | use period start version | inherent | inherent |
| Negative gross | reject | 400 | data fix |
| Zero deductions edge | inherent | inherent | inherent |
| Mid-compute crash | rollback | sev-2 | retry |
| Compute non-deterministic | code review | inherent | bug fix |

## §11 — Implementation notes
- §11.1 Brackets stored as JSONB in FR-REW-002 param_versions; computer reads + applies.
- §11.2 PIT computer iterates brackets; per-bracket: taxable_in_bracket × rate.
- §11.3 Rate (7,6) = up to 99.9999%; comfortably handles all SI/PIT scenarios.
- §11.4 BRAIN audit body: payroll_run_id, member_id, kind; amounts SHA256.
- §11.5 union_due + voluntary = optional kinds for future use.

---

*End of FR-REW-004 spec.*
