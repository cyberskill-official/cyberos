---
artefact: coverage-gate@1
task: TASK-IMP-089
phase: testing
tests_failed: 0
files_below_90pct: []
ecm_rows_uncovered: []
---
# TASK-IMP-089 coverage gate

Raw terminal (full suite):
```
$ bash scripts/tests/test_template_schema.sh
...
  ok   t08_single_out_of_scope_home
  ok   t08_duplicate_reintroduction_fails
  ok   t08_payload_carries_shape
pass=10 fail=0
```

Touched files and their coverage:
| File | Covered by | Coverage |
|---|---|---|
| tools/install/templates/TASK-TEMPLATE.md | t08 x3 (shape oracle, canary, payload parity) | shape fully asserted |
| scripts/tests/test_template_schema.sh | is the coverage | n/a |

TRACE-004 closure:
- 1.1, 1.2 -> t08_single_out_of_scope_home: passed
- 1.3 -> t08_duplicate_reintroduction_fails: passed (oracle canary: a re-added section 4 is caught)
- 1.4 -> t08_payload_carries_shape: passed (scratch build + byte parity with source)

Edge-case matrix rows without a test: none. Historical specs keeping the old shape are
out of scope by design (the rubric never required section 4); the suite targets the
template only.

Committed-object evidence: `git show HEAD:tools/install/templates/TASK-TEMPLATE.md`
-> 0 matches for "## 4. Out of scope"; invariants present at `## 4.`.
