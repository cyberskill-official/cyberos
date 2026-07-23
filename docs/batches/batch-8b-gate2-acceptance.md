---
batch: ship/batch-8b-install-ci-skills
members:
  - TASK-IMP-136
  - TASK-IMP-137
  - TASK-SKILL-202
started: 2026-07-23T17:00:00+07:00
ended: 2026-07-23T22:51:00+07:00
route_backs: 0
gate_reasks: 0
tokens: unknown
---
# Batch 8B — gate-2 final acceptance (ship/batch-8b-install-ci-skills)

Sub-batch of `batch/8-audit-hardening`. One branch per batch: `ship/batch-8b-install-ci-skills`.

## Gate-2 final acceptance (STATUS-REFERENCE §1.4)

**Verdict:** ACCEPT (all-accept)  
**Actor:** Stephen Cheng  
**Recorded:** 2026-07-23  
**Evidence source:** operator chat instruction:

> all-accept

(covers Batch B gate-2 for TASK-IMP-136, TASK-IMP-137, TASK-SKILL-202, and MEMORY-303 on batch-8c — MEMORY-303 is closed on `ship/batch-8c-memory` with its own gate-2 note.)

| Task | Title | Transition |
|------|-------|------------|
| TASK-IMP-136 | CI CAF evals + stub truth | testing → done |
| TASK-IMP-137 | Install portability hardening | testing → done |
| TASK-SKILL-202 | Skill quality floor | testing → done |

This file is the `--verdict-evidence` artefact for the three gated flips above.

## Known residuals accepted with this verdict

1. SKILL-202 AC names `test_skill_floor.sh`; live suite is `scripts/tests/test_skill_stub_lint.sh` (7/7 green at gate-1→testing).
2. Gated `status_overridden` appends may desync MMR `peaks.bin` until a human/doctor rebuild — store was READY after rebuild during ship.

## Ship close

| Task | Status after gate-2 |
|------|---------------------|
| TASK-IMP-136 | done |
| TASK-IMP-137 | done |
| TASK-SKILL-202 | done |
