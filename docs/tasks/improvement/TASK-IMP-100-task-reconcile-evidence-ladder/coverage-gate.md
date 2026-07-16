---
artefact: coverage-gate@1
task: TASK-IMP-100
phase: testing
tests_failed: 0
files_below_90pct: []
ecm_rows_uncovered: []
---
# TASK-IMP-100 coverage gate

Raw terminal (full suite):
```
$ bash tools/install/tests/test_task_reconcile.sh
task-reconcile suite (TASK-IMP-100):
  ok   t01_clean_resume
  ok   t02_route_back
  ok   t03_adopt_candidate
  ok   t04_read_only_and_spec_drift
  ok   t05_payload_vendored
  ok   t06_body_binding_preferred
test_task_reconcile: pass=6 fail=0
```

| File | Covered by | Coverage |
|---|---|---|
| tools/install/docs-tools/task-reconcile.mjs | t01-t06 (every rung verdict + every recommendation branch + read-only guard) | all outcome branches |
| modules/skill/task-reconcile/SKILL.md | AC 6 recorded greps | n/a - prose contract |
| tools/install/build.sh (vendor line + VENDORED_SKILLS) | t05 + `chain OK: 25 referenced, 53 vendored` | 2/2 |

TRACE-004: 1.2/1.3 -> t01, t02, t03 passed; 1.1 -> t04 passed; 1.4 -> t03 (bundle arm) passed;
1.6 -> t05 passed; 1.5 -> AC 6 recorded greps. ECM rows uncovered: none (row 11's degradation
guards are rung-level notes, probed and recorded in the gate log).

Live-corpus evidence beyond the suite: the tool reads TASK-IMP-092 (shipped) as
resume_at_phase(confirm-done) and TASK-IMP-101/102 (mid-review) as resume_at_phase(17) - the
step they are actually at. The dogfood finding that drove R1's design is recorded at E4.
