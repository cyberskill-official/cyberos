---
batch: batch/12f-imp-061
members:
  - TASK-IMP-061
recorded: 2026-07-25
actor: Stephen Cheng (session operator)
---

# batch/12f — IMP-061 BRAIN Phase 0 consent completion

## What landed

Layer-1 BRAIN Phase 0 consent scaffolding (not product EVAL activation):

- AGENTS.md **§19** Phase 0 consent (personnel-gated memories require resolvable `consent.consent_event`)
- `meta/consent/` store scaffold + `CONSENT.md` template + starter README
- Walker invariant `personnel-requires-consent` + `test_personnel_consent.py` (8/8)
- Fixture 21 expects `personnel-requires-consent`
- `build.sh` vendors `modules/memory/cyberos/data/AGENTS.md` (dense protocol) into payload `memory/AGENTS.md`
- `install.sh` creates `meta/consent/` on fresh BRAIN scaffold
- Install test `test_memory_agents_protocol.sh` pass=5

## Explicit non-goals (still open / separate)

- Counsel clearance of `docs/legal/data-monitoring-and-evaluation-notice.md`
- TASK-EVAL-001 notice publish / acknowledgment ledger / `CAPTURE_ENABLED`
- Track C activation (IMP-066 closed won't-do for 1.x)

## Tests

- `PYTHONPATH=cyberos:. pytest modules/memory/tests/test_personnel_consent.py -q` → 8 passed
- `bash tools/install/tests/test_memory_agents_protocol.sh` → pass=5 fail=0
- `node tools/install/docs-tools/task-lint.mjs …/TASK-IMP-061…/spec.md` → exit 0
