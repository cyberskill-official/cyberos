---
artefact: coverage-gate@1
task: TASK-IMP-091
phase: testing
tests_failed: 0
files_below_90pct: []
ecm_rows_uncovered: []
---
# TASK-IMP-091 coverage gate

Raw terminal (full suite):
```
$ bash scripts/tests/test_regen_backlog.sh
regen-backlog suite (TASK-IMP-091):
  ok   t01_live_corpus_parity
  ok   t02_totals_true
  ok   t03_every_status_emitted
regen-backlog: 3 passed, 0 failed
$ git status --short docs/tasks/BACKLOG.md
(empty - the live index is byte-untouched by the suite)
```

Touched files and their coverage:
| File | Covered by | Coverage |
|---|---|---|
| scripts/migrate_improvement_to_task.py (regen_backlog, status_line, halt) | t01, t02, t03 | both outcome branches (success summary, halt) + emission for 12 statuses |
| scripts/tests/test_regen_backlog.sh | is the coverage | n/a |

TRACE-004 closure:
- 1.1, 1.3 -> t01_live_corpus_parity: passed
- 1.2 -> t02_totals_true: passed
- 1.4 -> t03_every_status_emitted: passed
- 1.5 -> AC 4 verify (run_all lists `ok test_regen_backlog.sh`): satisfied

Edge-case matrix rows without a test: none. Row 7 (empty module folder) is pre-existing behavior this task does not change and carries no new assertion by design.

The repaired failure, measured: the regenerator that would have deleted 17 committed rows this morning now reproduces the committed improvement section byte-for-byte (t01 compares against `git show HEAD:docs/tasks/BACKLOG.md`).
