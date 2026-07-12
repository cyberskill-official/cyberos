# HR module — feature request index

_Generated 2026-05-17 — 9 FRs, 52 engineering-hours total._

## FRs

| FR | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [FR-HR-001](FR-HR-001-member-schema/spec.md) | MUST | 1 | 6 | HR Member schema — profile + role + level + contract type + leave balance + sabbatical accrual + sta |
| [FR-HR-002](FR-HR-002-contract-types/spec.md) | MUST | 6 | 4 | HR 5 contract types — indefinite + fixed_term + probation + part_time + contractor with per-type lea |
| [FR-HR-003](FR-HR-003-cccd-kms/spec.md) | MUST | 6 | 5 | HR CCCD photo KMS — separate keyspace for VN citizen ID photos with sev-1 access audit + ROOT-CHRO-o |
| [FR-HR-004](FR-HR-004-leave-types/spec.md) | MUST | 6 | 5 | HR 8 leave types — annual/sick/maternity/paternity/sabbatical/unpaid/bereavement/public_holiday with |
| [FR-HR-005](FR-HR-005-working-hours-si-rates/spec.md) | MUST | 6 | 4 | HR Decree 145/2020 working-hour caps + Decree 152/2020 SI rates — version-pinned policy constants wi |
| [FR-HR-006](FR-HR-006-leave-accrual-cron/spec.md) | MUST | 6 | 4 | HR annual leave accrual nightly batch — Decree 145 formula (1d/month + 1d/5yr seniority bonus) with  |
| [FR-HR-007](FR-HR-007-onboarding-saga/spec.md) | MUST | 6 | 10 | HR onboarding saga — orchestrates AUTH + TIME + LEARN + KB + CHAT + REW provisioning on member.activ |
| [FR-HR-008](FR-HR-008-performance-signals/spec.md) | MUST | 7 | 6 | HR performance signal aggregator — read-only consumer of PROJ + TIME + LEARN signals for periodic pe |
| [FR-HR-009](FR-HR-009-termination-workflow/spec.md) | MUST | 7 | 8 | HR termination workflow — Good-Leaver / Bad-Leaver branch with CFO+CEO co-sign + ESOP forfeiture + a |

## Cross-module dependencies

**This module depends on:**

- **AUTH**: FR-HR-001→FR-AUTH-003, FR-HR-001→FR-AUTH-101
- **PROJ**: FR-HR-008→FR-PROJ-013
- **TIME**: FR-HR-008→FR-TIME-001

**This module is depended on by:**

- **ESOP**: FR-ESOP-001→FR-HR-001, FR-ESOP-005→FR-HR-009
- **LEARN**: FR-LEARN-001→FR-HR-001
- **RES**: FR-RES-001→FR-HR-001, FR-RES-005→FR-HR-005
- **REW**: FR-REW-001→FR-HR-001, FR-REW-004→FR-HR-005

---

_See `IMPLEMENTATION_ORDER.md` for the full topological build sequence._