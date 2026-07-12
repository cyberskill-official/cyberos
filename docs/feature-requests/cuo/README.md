# CUO module — feature request index

_Generated 2026-05-17 — 5 FRs, 37 engineering-hours total._

## FRs

| FR | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [FR-CUO-101](FR-CUO-101-langgraph-supervisor/spec.md) | MUST | 2 | 12 | CUO Phase 2 — LangGraph supervisor + LiteLLM cascade + confidence-band escalation + persona-aware ro |
| [FR-CUO-102](FR-CUO-102-langgraph-postgres-checkpointer/spec.md) | MUST | 6 | 5 | CUO Postgres checkpointer for LangGraph state — persists supervisor graph state per run with EU AI A |
| [FR-CUO-103](FR-CUO-103-trace-replay-rows/spec.md) | MUST | 6 | 4 | CUO Phase 2 trace rows include prompt + model + temperature + seed for deterministic replay |
| [FR-CUO-104](FR-CUO-104-topological-chain-walk/spec.md) | MUST | 6 | 10 | CUO topological walk of `depends_on` chain — orchestrates multi-step skill invocations with composit |
| [FR-CUO-105](FR-CUO-105-per-step-rollback/spec.md) | MUST | 6 | 6 | CUO per-step rollback on chain failure — execute compensating actions in reverse order with partial- |

## Cross-module dependencies

**This module depends on:**

- **AI**: FR-CUO-101→FR-AI-008

**This module is depended on by:**

- **CRM**: FR-CRM-005→FR-CUO-101, FR-CRM-006→FR-CUO-101, FR-CRM-007→FR-CUO-101
- **DOC**: FR-DOC-009→FR-CUO-101
- **EMAIL**: FR-EMAIL-008→FR-CUO-101
- **INV**: FR-INV-010→FR-CUO-101
- **KB**: FR-KB-007→FR-CUO-101
- **OKR**: FR-OKR-006→FR-CUO-101, FR-OKR-007→FR-CUO-101
- **PORTAL**: FR-PORTAL-005→FR-CUO-101
- **PROJ**: FR-PROJ-011→FR-CUO-101, FR-PROJ-012→FR-CUO-101
- **RES**: FR-RES-004→FR-CUO-101

---

_See `IMPLEMENTATION_ORDER.md` for the full topological build sequence._