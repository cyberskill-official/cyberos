# REW module — feature request index

_Generated 2026-05-17 — 10 FRs, 55 engineering-hours total._

## FRs

| FR | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [FR-REW-001](FR-REW-001-3p-income-schema/spec.md) | MUST | 1 | 6 | REW 3P income schema — P1 Base + P2 Allowance + P3 Performance with separate encrypted comp keyspace |
| [FR-REW-002](FR-REW-002-parameter-versioning/spec.md) | MUST | 1 | 6 | REW parameter versioning — immutable versioned formula parameters with 100% replay-equivalence on pr |
| [FR-REW-003](FR-REW-003-p1-protection-invariant/spec.md) | MUST | 1 | 4 | REW P1 protection invariant — DB CHECK constraint + service-layer guard forbidding any P1 cash reduc |
| [FR-REW-004](FR-REW-004-statutory-deductions/spec.md) | MUST | 1 | 6 | REW statutory deductions — BHXH 10.5% + BHYT 1.5% + BHTN 1% + PIT progressive per Decree 152/2020 wi |
| [FR-REW-005](FR-REW-005-monthly-payroll-compute/spec.md) | MUST | 2 | 8 | REW monthly payroll compute + CFO+CHRO co-sign commit gate — orchestrates 3P + deductions + net pay  |
| [FR-REW-006](FR-REW-006-payslip-pdf/spec.md) | MUST | 2 | 6 | REW byte-identical payslip PDF render — Tectonic + pinned fonts produces deterministic PDF bytes for |
| [FR-REW-007](FR-REW-007-bp-ledger/spec.md) | MUST | 2 | 5 | REW BP (Bonus Points) ledger with ACB-rate interest accrual nightly + per-Member balance + immutable |
| [FR-REW-008](FR-REW-008-p3-quarterly-distribution/spec.md) | MUST | 2 | 6 | REW quarterly P3 distribution from BP fund — CEO+CFO sign-off + LEARN-007 VP share splits + debit BP |
| [FR-REW-009](FR-REW-009-vietqr-payroll-batch/spec.md) | MUST | 2 | 5 | REW VietQR bank payroll batch send — bulk transfer file generation with CFO manual confirm at submis |
| [FR-REW-010](FR-REW-010-memory-exclusion-ci-gate/spec.md) | MUST | 1 | 3 | REW memory structural exclusion CI gate — no comp fields appear in memory-ingest paths; static analysi |

## Cross-module dependencies

**This module depends on:**

- **AUTH**: FR-REW-001→FR-AUTH-101
- **HR**: FR-REW-001→FR-HR-001, FR-REW-004→FR-HR-005
- **INV**: FR-REW-009→FR-INV-005

---

_See `IMPLEMENTATION_ORDER.md` for the full topological build sequence._