---
batch: batch/12c-gate-tooling
members:
  - TASK-IMP-011
  - TASK-IMP-012
  - TASK-IMP-008
  - TASK-IMP-026
started: 2026-07-25
ended: 2026-07-25
---

# Batch 12c — gate tooling (PR-C)

Payload/1.x-scoped gate tooling batch:

| Task | Title | Deliverable |
|------|-------|-------------|
| TASK-IMP-011 | Structured gate-failure taxonomy | `run-gates.sh` → `gate-failure@1` JSON + `GATE_FAILURE_JSON:` |
| TASK-IMP-012 | Coverage measurement and ratchet | `coverage-ratchet.mjs` + `coverage-baseline.json` (install-suite test-touch) |
| TASK-IMP-008 | Goldensets as first-class gate inputs | `tools/install/.awh/` + `run-goldenset.sh` + CI job |
| TASK-IMP-026 | Auto-revert on gate regression | `gate-auto-revert.sh` (opt-in / dry-run; never force-push/merge) |

## Honest scoping

- **Not** platform-wide `cargo llvm-cov` or full caf finding taxonomies.
- Coverage ratchet measures install-script test-touch %, fails only on regression vs committed baseline.
- Goldenset is install/payload offline tasks; awh optional with skip-clean.
- Auto-revert opens a revert PR when `CYBEROS_AUTO_REVERT=1`; never merges.

## Suites

- `tools/install/tests/test_gate_failure_taxonomy.sh`
- `tools/install/tests/test_coverage_ratchet.sh`
- `tools/install/tests/test_install_goldenset.sh`
- `tools/install/tests/test_gate_auto_revert.sh`

## HITL

Session operator continuum (Stephen Cheng): Gate-1 + Gate-2 evidence in
`docs/batches/batch-12c-gate1-acceptance.md` and `docs/batches/batch-12c-gate2-acceptance.md`.
