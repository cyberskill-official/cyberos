---
artefact: coverage-gate@1
task: TASK-IMP-101
phase: testing
tests_failed: 0
files_below_90pct: []
ecm_rows_uncovered: []
---
# TASK-IMP-101 coverage gate

Raw terminal: `bash tools/install/tests/test_workflow_helpers.sh` -> `test_workflow_helpers: pass=14 fail=0`
(t14_reconcile_entry_and_deps_gate new; t09/t12 pins at 2.7.0; t01-t13 green).

| File | Covered by | Coverage |
|---|---|---|
| modules/cuo/chief-technology-officer/workflows/ship-tasks.md (two §§, step 0, version) | t14 (source AND scratch payload), t09/t12 (pins) | every gated passage |
| tools/install/tests/test_workflow_helpers.sh | is the coverage | n/a |

TRACE-004: 1.1/1.2/1.3/1.5 -> t14 passed; 1.4 -> t12 passed (pin at 2.7.0, moved with
t09_doctrine_wiring's identical pin - disclosed); AC 3 guardrail -> t01-t13 green.

Payload: `chain OK: 25 referenced, 53 vendored, 6 allowlisted` - naming task-reconcile in the
chain obliged the payload to carry it in both trees, and does. ECM rows uncovered: none.
