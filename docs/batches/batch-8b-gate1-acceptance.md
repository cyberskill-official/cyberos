---
batch: ship/batch-8b-install-ci-skills
members:
  - TASK-IMP-136
  - TASK-IMP-137
  - TASK-SKILL-202
started: 2026-07-23T17:32:00+07:00
route_backs: 0
gate_reasks: 0
tokens: unknown
---
# Batch 8B — install / CI / skills (ship/batch-8b-install-ci-skills)

Sub-batch of `batch/8-audit-hardening`. One branch per batch: `ship/batch-8b-install-ci-skills`.

**Base:** branched from `ship/batch-8c-memory` tip `e7d3eb06` (Batch A gate-2 done on the hardening line + MEMORY-303 store repair + IMP-138 Branch A decision). Cleanest tip with Batch A complete and a useful BACKLOG; does not re-merge 8a separately (8c already contains 8a).

## Gate-1 review acceptance (STATUS-REFERENCE §1.4)

**Verdict:** ACCEPT (all-accept for Batch B)  
**Actor:** Stephen Cheng  
**Recorded:** 2026-07-23  
**Evidence source:** operator chat instruction:

> Batch B gate-1: all-accept for TASK-IMP-136, TASK-IMP-137, TASK-SKILL-202

| Task | Title | Transition |
|------|-------|------------|
| TASK-IMP-136 | CI truth - CAF evals in root CI, hook-claim honesty, stub workflow sweep | reviewing → ready_to_test |
| TASK-IMP-137 | Install portability - MCP loopback+token, shasum fallback, atomic vendor | reviewing → ready_to_test |
| TASK-SKILL-202 | Skill quality floor - NFR stubs, untrusted-content backport, pair parity | reviewing → ready_to_test |

This file is the `--verdict-evidence` artefact for the three gated flips above.
It does **not** authorize `testing → done` (gate-2 / final acceptance).
It does **not** cover MEMORY-303, IMP-138, or other batch-8 members.

## Ship progress

| Task | Status after Batch B ship pass |
|------|--------------------------------|
| TASK-IMP-136 | testing (halted at gate-2) |
| TASK-IMP-137 | testing (halted at gate-2) |
| TASK-SKILL-202 | testing (halted at gate-2) |

`ended` omitted until Batch B gate-2.
