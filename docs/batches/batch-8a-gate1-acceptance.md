---
batch: ship/batch-8a-core-locks
members:
  - TASK-CUO-302
  - TASK-CUO-303
  - TASK-CUO-304
started: 2026-07-23T15:40:00+07:00
route_backs: 0
gate_reasks: 0
tokens: unknown
---
# Batch 8A — core locks (ship/batch-8a-core-locks)

Sub-batch of `batch/8-audit-hardening`. One branch per batch: `ship/batch-8a-core-locks`.

## Gate-1 review acceptance (STATUS-REFERENCE §1.4)

**Verdict:** ACCEPT  
**Actor:** Stephen Cheng  
**Recorded:** 2026-07-23  
**Evidence source:** operator chat instruction for Batch A ship:

> Gate-1 ACCEPT for all Batch A tasks: TASK-CUO-302, TASK-CUO-303, TASK-CUO-304  
> ("all accept for A")

| Task | Title | Transition |
|------|-------|------------|
| TASK-CUO-302 | Fail-closed machine gates | reviewing → ready_to_test |
| TASK-CUO-303 | Mechanical HITL lock | reviewing → ready_to_test |
| TASK-CUO-304 | Unify route-back ceiling | reviewing → ready_to_test |

This file is the `--verdict-evidence` artefact for the three gated flips above.
It does **not** authorize `testing → done` (gate-2 / final acceptance).
It does **not** cover MEMORY-303 store repair, IMP-138, IMP-139, or other batch-8 members.

## Ship progress

| Task | Status after Batch A ship pass |
|------|--------------------------------|
| TASK-CUO-302 | testing (halted at gate-2) |
| TASK-CUO-303 | testing (halted at gate-2) |
| TASK-CUO-304 | testing (halted at gate-2) |

`ended` omitted until gate-2 closes the batch (incomplete ledger until then).
