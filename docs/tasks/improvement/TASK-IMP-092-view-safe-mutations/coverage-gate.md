---
artefact: coverage-gate@1
task: TASK-IMP-092
phase: testing
tests_failed: 0
files_below_90pct: []
ecm_rows_uncovered: []
---
# TASK-IMP-092 coverage gate

Raw terminal (full suite):
```
$ bash tools/install/tests/test_workflow_helpers.sh
...
  ok   t10_retally_corrects_lying_header
  ok   t11_footprint_holds_with_retally
  ok   t12_doctrine_view_rules_vendored
test_workflow_helpers: pass=12 fail=0
```

Touched files and their coverage:
| File | Covered by | Coverage |
|---|---|---|
| tools/install/docs-tools/backlog-mutate.mjs (retally) | t10 (flip + insert arms), t11 (footprint), t01-t09 regressions | flip and insert paths; bare-header path via t06 |
| modules/cuo/chief-technology-officer/workflows/ship-tasks.md (2.6.3 doctrine) | t12 (source + scratch payload) | both passages + version |
| tools/install/tests/test_workflow_helpers.sh | is the coverage | n/a |

TRACE-004 closure:
- 1.1, 1.2 -> t10_retally_corrects_lying_header: passed
- 1.3 -> t11_footprint_holds_with_retally: passed
- 1.1 (regressions) -> t01-t09: passed (12/12)
- 1.4, 1.5 -> t12_doctrine_view_rules_vendored: passed

Edge-case matrix rows without a test: none.

Dogfood evidence (the tool maintained this batch's own rows through 20 mutations):
`## improvement  (67 draft, 5 ready_to_implement, 20 done)` -> implementing -> reviewing ->
ready_to_test -> `(67 draft, 5 testing, 20 done)`; every header printed "header retallied",
counts tracked the rows, no inherited baseline. Verified on committed objects per the rule
this task introduces.
