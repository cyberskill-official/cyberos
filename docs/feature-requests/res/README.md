# RES module — feature request index

_Generated 2026-05-17 — 5 FRs, 38 engineering-hours total._

## FRs

| FR | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [FR-RES-001](FR-RES-001-capacity-demand-matrix.md) | MUST | 7 | 10 | RES capacity-vs-demand matrix — nightly join across HR + PROJ + TIME + LEARN producing per-member-we |
| [FR-RES-002](FR-RES-002-allocation-gantt-ui.md) | MUST | 8 | 12 | RES allocation Gantt UI — drag-rebalance interface over capacity matrix with optimistic concurrency  |
| [FR-RES-003](FR-RES-003-over-under-flags.md) | MUST | 8 | 4 | RES over/under-allocation flags — 110% warning / 60% under-utilization threshold with weekly digest  |
| [FR-RES-004](FR-RES-004-hiring-memo-cuo.md) | MUST | 8 | 8 | RES hiring memo CUO draft — skill-gap × CRM pipeline trigger → CEO+CFO review queue with cost-benefi |
| [FR-RES-005](FR-RES-005-vn-ot-cap-hard-block.md) | MUST | 8 | 4 | RES VN Labour Code Art. 107 OT cap hard-block — propose-time validation gate preventing weekly + ann |

## Cross-module dependencies

**This module depends on:**

- **CRM**: FR-RES-004→FR-CRM-001
- **CUO**: FR-RES-004→FR-CUO-101
- **HR**: FR-RES-001→FR-HR-001, FR-RES-005→FR-HR-005
- **PROJ**: FR-RES-001→FR-PROJ-001
- **TIME**: FR-RES-001→FR-TIME-001

---

_See `IMPLEMENTATION_ORDER.md` for the full topological build sequence._