---
artefact: coverage-gate@1
task: TASK-IMP-099
phase: testing
tests_failed: 0
files_below_90pct: []
ecm_rows_uncovered: []
---
# TASK-IMP-099 coverage gate

Raw terminal: `bash tools/install/tests/test_workflow_helpers.sh` -> `test_workflow_helpers: pass=13 fail=0` (t13_queue_rule_p0_p3 new; t12 pin at 2.6.4; t01-t11 untouched and green).

| File | Covered by | Coverage |
|---|---|---|
| modules/cuo/chief-technology-officer/workflows/ship-tasks.md (line 312 rule + version) | t13 (source + payload, rule shape negative-grep) + t12 (pin) | rule, parenthetical, version |
| tools/install/tests/test_workflow_helpers.sh (t12/t09 pins, t13) | is the coverage | n/a |

TRACE-004: 1.1+1.3 -> t13 passed; 1.2 -> t12 passed (pin moved, disclosed - t09_doctrine_wiring's identical pin moved with it, mechanically forced, disclosed in review); AC 3 guardrail -> t01-t11 green. Payload: cuo/ship-tasks.md and the plugin copy both 2.6.4 with the p0-p3 rule; `grep -ci moscow` = 1 (the FM-105 parenthetical only). ECM rows uncovered: none.
