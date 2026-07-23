---
batch: ship/batch-8a-core-locks
members:
  - TASK-CUO-302
  - TASK-CUO-303
  - TASK-CUO-304
started: 2026-07-23T15:40:00+07:00
ended: 2026-07-23T16:50:00+07:00
route_backs: 0
gate_reasks: 0
tokens: unknown
---
# Batch 8A — gate-2 final acceptance (ship/batch-8a-core-locks)

Sub-batch of `batch/8-audit-hardening`. One branch per batch: `ship/batch-8a-core-locks`.

## Gate-2 final acceptance (STATUS-REFERENCE §1.4)

**Verdict:** ACCEPT (all-accept)  
**Actor:** Stephen Cheng  
**Recorded:** 2026-07-23  
**Evidence source:** operator chat instruction for Batch A close:

> Gate-2 all-accept for Batch A: TASK-CUO-302, TASK-CUO-303, TASK-CUO-304 → flip each `testing → done`

| Task | Title | Transition |
|------|-------|------------|
| TASK-CUO-302 | Fail-closed machine gates | testing → done |
| TASK-CUO-303 | Mechanical HITL lock | testing → done |
| TASK-CUO-304 | Unify route-back ceiling | testing → done |

This file is the `--verdict-evidence` artefact for the three gated flips above.

## Ship close

| Task | Status after gate-2 |
|------|---------------------|
| TASK-CUO-302 | done |
| TASK-CUO-303 | done |
| TASK-CUO-304 | done |

Batch A core locks are closed. MEMORY-303 store repair and IMP-138 are out of Batch A scope (separate operator items).
