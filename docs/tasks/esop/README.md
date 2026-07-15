# ESOP module — task index

_Generated 2026-05-17 — 7 tasks, 38 engineering-hours total._

## tasks

| Task | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [TASK-ESOP-001](TASK-ESOP-001-sp-grant-schema/spec.md) | MUST | 1 | 5 | ESOP SP grant schema — Stock Plan grant with 4-year vesting + 12-month cliff default + per-grant imm |
| [TASK-ESOP-002](TASK-ESOP-002-monthly-vesting-batch/spec.md) | MUST | 1 | 4 | ESOP monthly vesting accrual deterministic batch — runs EOM tenant_tz computing per-grant vested sha |
| [TASK-ESOP-003](TASK-ESOP-003-annual-valuation/spec.md) | MUST | 1 | 5 | ESOP annual valuation — CFO base + Board multiplier sign-off with immutable share-price snapshot per |
| [TASK-ESOP-004](TASK-ESOP-004-put-option-exec/spec.md) | MUST | 2 | 8 | ESOP put-option exec flow — Year 3+ eligibility + per-Member annual cap + CFO approve + bank wire vi |
| [TASK-ESOP-005](TASK-ESOP-005-gl-bl-branch/spec.md) | MUST | 2 | 5 | ESOP Good/Bad Leaver branch on HR offboarding — CFO+CEO co-sign to apply forfeiture/acceleration per |
| [TASK-ESOP-006](TASK-ESOP-006-ma-acceleration/spec.md) | SHOULD | 2 | 5 | ESOP M&A acceleration trigger — Board declares M&A event + 5-business-day Member notice + full vesti |
| [TASK-ESOP-007](TASK-ESOP-007-member-dashboard/spec.md) | SHOULD | 2 | 6 | ESOP Member dashboard — personal view only (own grants + vesting + estimated value); cross-Member ac |

## Cross-module dependencies

**This module depends on:**

- **HR**: TASK-ESOP-001→TASK-HR-001, TASK-ESOP-005→TASK-HR-009
- **INV**: TASK-ESOP-004→TASK-INV-005

**This module is depended on by:**

- **TEN**: TASK-TEN-201→TASK-ESOP-001

---

_See `IMPLEMENTATION_ORDER.md` for the full topological build sequence._