# RES module — task index

_Generated 2026-05-17 — 5 FRs, 38 engineering-hours total._

## FRs

| FR | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [TASK-RES-001](TASK-RES-001-capacity-demand-matrix/spec.md) | MUST | 7 | 10 | RES capacity-vs-demand matrix — nightly join across HR + PROJ + TIME + LEARN producing per-member-we |
| [TASK-RES-002](TASK-RES-002-allocation-gantt-ui/spec.md) | MUST | 8 | 12 | RES allocation Gantt UI — drag-rebalance interface over capacity matrix with optimistic concurrency  |
| [TASK-RES-003](TASK-RES-003-over-under-flags/spec.md) | MUST | 8 | 4 | RES over/under-allocation flags — 110% warning / 60% under-utilization threshold with weekly digest  |
| [TASK-RES-004](TASK-RES-004-hiring-memo-cuo/spec.md) | MUST | 8 | 8 | RES hiring memo CUO draft — skill-gap × CRM pipeline trigger → CEO+CFO review queue with cost-benefi |
| [TASK-RES-005](TASK-RES-005-vn-ot-cap-hard-block/spec.md) | MUST | 8 | 4 | RES VN Labour Code Art. 107 OT cap hard-block — propose-time validation gate preventing weekly + ann |

## Cross-module dependencies

**This module depends on:**

- **CRM**: TASK-RES-004→TASK-CRM-001
- **CUO**: TASK-RES-004→TASK-CUO-101
- **HR**: TASK-RES-001→TASK-HR-001, TASK-RES-005→TASK-HR-005
- **PROJ**: TASK-RES-001→TASK-PROJ-001
- **TIME**: TASK-RES-001→TASK-TIME-001

---

_See `IMPLEMENTATION_ORDER.md` for the full topological build sequence._