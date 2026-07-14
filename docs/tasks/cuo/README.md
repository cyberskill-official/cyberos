# CUO module — task index

_Generated 2026-05-17 — 5 FRs, 37 engineering-hours total._

## FRs

| FR | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [TASK-CUO-101](TASK-CUO-101-langgraph-supervisor/spec.md) | MUST | 2 | 12 | CUO Phase 2 — LangGraph supervisor + LiteLLM cascade + confidence-band escalation + persona-aware ro |
| [TASK-CUO-102](TASK-CUO-102-langgraph-postgres-checkpointer/spec.md) | MUST | 6 | 5 | CUO Postgres checkpointer for LangGraph state — persists supervisor graph state per run with EU AI A |
| [TASK-CUO-103](TASK-CUO-103-trace-replay-rows/spec.md) | MUST | 6 | 4 | CUO Phase 2 trace rows include prompt + model + temperature + seed for deterministic replay |
| [TASK-CUO-104](TASK-CUO-104-topological-chain-walk/spec.md) | MUST | 6 | 10 | CUO topological walk of `depends_on` chain — orchestrates multi-step skill invocations with composit |
| [TASK-CUO-105](TASK-CUO-105-per-step-rollback/spec.md) | MUST | 6 | 6 | CUO per-step rollback on chain failure — execute compensating actions in reverse order with partial- |

## Cross-module dependencies

**This module depends on:**

- **AI**: TASK-CUO-101→TASK-AI-008

**This module is depended on by:**

- **CRM**: TASK-CRM-005→TASK-CUO-101, TASK-CRM-006→TASK-CUO-101, TASK-CRM-007→TASK-CUO-101
- **DOC**: TASK-DOC-009→TASK-CUO-101
- **EMAIL**: TASK-EMAIL-008→TASK-CUO-101
- **INV**: TASK-INV-010→TASK-CUO-101
- **KB**: TASK-KB-007→TASK-CUO-101
- **OKR**: TASK-OKR-006→TASK-CUO-101, TASK-OKR-007→TASK-CUO-101
- **PORTAL**: TASK-PORTAL-005→TASK-CUO-101
- **PROJ**: TASK-PROJ-011→TASK-CUO-101, TASK-PROJ-012→TASK-CUO-101
- **RES**: TASK-RES-004→TASK-CUO-101

---

_See `IMPLEMENTATION_ORDER.md` for the full topological build sequence._