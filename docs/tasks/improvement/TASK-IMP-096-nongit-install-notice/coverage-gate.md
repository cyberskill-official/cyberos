---
artefact: coverage-gate@1
task: TASK-IMP-096
phase: testing
tests_failed: 0
files_below_90pct: []
ecm_rows_uncovered: []
---
# TASK-IMP-096 coverage gate

Raw terminal: hygiene `t09_nongit_summary_line` ok (non-git arm: line exactly once with the
verbatim remedy; stale-.git-remnant arm; git arm: zero) inside
`install-hygiene: 19 passed, 0 failed`.

| File | Covered by | Coverage |
|---|---|---|
| tools/install/install.sh (summary line, rev-parse gated) | t09 x3 arms | 3/3 branches |

TRACE-004: 1.1+1.3 -> t09 passed; 1.2 -> t09 git arm passed. Live capture (plain dir):
`cyberos install: this repo is not a git checkout - ship-tasks needs one; run: git init -b main
&& git add -A && git commit -m init` at summary line 43, exactly once. ECM rows uncovered: none.
