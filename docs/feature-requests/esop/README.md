# ESOP module — feature request index

_Generated 2026-05-17 — 7 FRs, 38 engineering-hours total._

## FRs

| FR | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [FR-ESOP-001](FR-ESOP-001-sp-grant-schema.md) | MUST | 1 | 5 | ESOP SP grant schema — Stock Plan grant with 4-year vesting + 12-month cliff default + per-grant imm |
| [FR-ESOP-002](FR-ESOP-002-monthly-vesting-batch.md) | MUST | 1 | 4 | ESOP monthly vesting accrual deterministic batch — runs EOM tenant_tz computing per-grant vested sha |
| [FR-ESOP-003](FR-ESOP-003-annual-valuation.md) | MUST | 1 | 5 | ESOP annual valuation — CFO base + Board multiplier sign-off with immutable share-price snapshot per |
| [FR-ESOP-004](FR-ESOP-004-put-option-exec.md) | MUST | 2 | 8 | ESOP put-option exec flow — Year 3+ eligibility + per-Member annual cap + CFO approve + bank wire vi |
| [FR-ESOP-005](FR-ESOP-005-gl-bl-branch.md) | MUST | 2 | 5 | ESOP Good/Bad Leaver branch on HR offboarding — CFO+CEO co-sign to apply forfeiture/acceleration per |
| [FR-ESOP-006](FR-ESOP-006-ma-acceleration.md) | SHOULD | 2 | 5 | ESOP M&A acceleration trigger — Board declares M&A event + 5-business-day Member notice + full vesti |
| [FR-ESOP-007](FR-ESOP-007-member-dashboard.md) | SHOULD | 2 | 6 | ESOP Member dashboard — personal view only (own grants + vesting + estimated value); cross-Member ac |

## Cross-module dependencies

**This module depends on:**

- **HR**: FR-ESOP-001→FR-HR-001, FR-ESOP-005→FR-HR-009
- **INV**: FR-ESOP-004→FR-INV-005

**This module is depended on by:**

- **TEN**: FR-TEN-201→FR-ESOP-001

---

_See `IMPLEMENTATION_ORDER.md` for the full topological build sequence._