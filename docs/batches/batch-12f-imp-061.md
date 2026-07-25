---
batch: batch/12f-imp-061-brain-consent
members:
  - TASK-IMP-061
recorded: 2026-07-25
actor: Stephen Cheng (session operator)
---

# batch/12f — BRAIN Phase 0 consent (IMP-061)

## Deliverable

| Item | Path |
|---|---|
| Protocol §19 | `modules/memory/cyberos/data/AGENTS.md` |
| Invariant | `personnel-requires-consent` in both invariants.yaml + `cyberos.core.invariants` |
| Template | `modules/memory/runtime/starter/templates/CONSENT.md` |
| Starter scaffold | `…/store/meta/consent/README.md` |
| Install | `install.sh` creates `meta/consent/` + README |
| Build | `build.sh` vendors dense `modules/memory/cyberos/data/AGENTS.md` |
| Tests | `modules/memory/tests/test_personnel_consent.py` (8/8) |
| Fixture 21 | expects `personnel-requires-consent` |

## Non-goals (explicit)

- Counsel clearance of employment monitoring notice
- EVAL-001 / `CAPTURE_ENABLED` / product acknowledgment ledger
- Track C rollout (IMP-066 closed won't-do for 1.x)

## Merge order

**6 of A→F** — after #151–#154 (expect BACKLOG/status conflicts).
