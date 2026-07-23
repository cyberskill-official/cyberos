---
batch: ship/batch-8c-memory
members:
  - TASK-MEMORY-303
started: 2026-07-23T16:52:00+07:00
route_backs: 0
gate_reasks: 0
tokens: unknown
---
# Batch 8C — memory store repair (ship/batch-8c-memory)

Sub-batch of `batch/8-audit-hardening`. Branched from `ship/batch-8a-core-locks` after Batch A gate-2 close (shared BACKLOG already carries CUO-302/303/304 `done`).

## Scope this branch

1. Execute TASK-MEMORY-303 live-store layout repair (`store-repair-plan.md`) via canonical `cyberos move`.
2. Human MMR rebuild after Writer cold-start mismatch (see `store-repair-evidence.md`).
3. Refresh this repo's installed `.cyberos/` so fail-closed + doctor gates activate.
4. Record TASK-IMP-138 Branch A decision (no implementation).

## Status after this pass

| Task | Status | Note |
|------|--------|------|
| TASK-MEMORY-303 | ready_to_review | Repair executed; doctor READY; awaiting review HITL |
| TASK-IMP-138 | ready_to_implement | Branch A recorded; implement in Batch D |

`ended` omitted until MEMORY-303 gate-2 (or operator closes the sub-batch).
