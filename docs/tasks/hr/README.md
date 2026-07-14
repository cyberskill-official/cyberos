# HR module — task index

_Generated 2026-05-17 — 9 FRs, 52 engineering-hours total._

## FRs

| FR | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [TASK-HR-001](TASK-HR-001-member-schema/spec.md) | MUST | 1 | 6 | HR Member schema — profile + role + level + contract type + leave balance + sabbatical accrual + sta |
| [TASK-HR-002](TASK-HR-002-contract-types/spec.md) | MUST | 6 | 4 | HR 5 contract types — indefinite + fixed_term + probation + part_time + contractor with per-type lea |
| [TASK-HR-003](TASK-HR-003-cccd-kms/spec.md) | MUST | 6 | 5 | HR CCCD photo KMS — separate keyspace for VN citizen ID photos with sev-1 access audit + ROOT-CHRO-o |
| [TASK-HR-004](TASK-HR-004-leave-types/spec.md) | MUST | 6 | 5 | HR 8 leave types — annual/sick/maternity/paternity/sabbatical/unpaid/bereavement/public_holiday with |
| [TASK-HR-005](TASK-HR-005-working-hours-si-rates/spec.md) | MUST | 6 | 4 | HR Decree 145/2020 working-hour caps + Decree 152/2020 SI rates — version-pinned policy constants wi |
| [TASK-HR-006](TASK-HR-006-leave-accrual-cron/spec.md) | MUST | 6 | 4 | HR annual leave accrual nightly batch — Decree 145 formula (1d/month + 1d/5yr seniority bonus) with  |
| [TASK-HR-007](TASK-HR-007-onboarding-saga/spec.md) | MUST | 6 | 10 | HR onboarding saga — orchestrates AUTH + TIME + LEARN + KB + CHAT + REW provisioning on member.activ |
| [TASK-HR-008](TASK-HR-008-performance-signals/spec.md) | MUST | 7 | 6 | HR performance signal aggregator — read-only consumer of PROJ + TIME + LEARN signals for periodic pe |
| [TASK-HR-009](TASK-HR-009-termination-workflow/spec.md) | MUST | 7 | 8 | HR termination workflow — Good-Leaver / Bad-Leaver branch with CFO+CEO co-sign + ESOP forfeiture + a |

## Cross-module dependencies

**This module depends on:**

- **AUTH**: TASK-HR-001→TASK-AUTH-003, TASK-HR-001→TASK-AUTH-101
- **PROJ**: TASK-HR-008→TASK-PROJ-013
- **TIME**: TASK-HR-008→TASK-TIME-001

**This module is depended on by:**

- **ESOP**: TASK-ESOP-001→TASK-HR-001, TASK-ESOP-005→TASK-HR-009
- **LEARN**: TASK-LEARN-001→TASK-HR-001
- **RES**: TASK-RES-001→TASK-HR-001, TASK-RES-005→TASK-HR-005
- **REW**: TASK-REW-001→TASK-HR-001, TASK-REW-004→TASK-HR-005

---

_See `IMPLEMENTATION_ORDER.md` for the full topological build sequence._