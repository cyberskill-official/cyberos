---
artefact: coverage-gate@1
task: TASK-IMP-090
phase: testing
tests_failed: 0
files_below_90pct: []
ecm_rows_uncovered: []
---
# TASK-IMP-090 coverage gate

Raw terminal:
```
$ bash tools/install/tests/test_install_hygiene.sh
  ok   t07_workflow_gitignore_patterns
install-hygiene: 17 passed, 0 failed
```

Touched files and their coverage:
| File | Covered by | Coverage |
|---|---|---|
| tools/install/install.sh (.workflow seed) | t07 (fresh + append-once paths) | 2/2 branches |
| modules/skill/task-author/SKILL.md | AC 1 recorded grep (prose contract) | n/a - not executable |
| docs/tasks/.workflow/.gitignore + index | AC 3 recorded ls-files | n/a - repo state |
| docs/tasks/_audits/IMPROVEMENT-BATCHES-2026-07-16.md | AC 4 recorded greps | n/a - single document |

TRACE-004 closure:
- 1.1 -> AC 1 verify (SKILL.md:184 grep recorded in gate-log E1): satisfied
- 1.2, 1.5 -> t07_workflow_gitignore_patterns: passed
- 1.3 -> AC 3 verify (`git ls-files docs/tasks/.workflow | grep -c manifest.json` = 0, on the committed tree via `git ls-tree -r HEAD`): satisfied
- 1.4 -> AC 4 verify (record present, 121 lines, 27 member-id mentions): satisfied

Ops-verified ACs carry the rationale recorded in audit.md ISS-001 and match the pattern accepted for TASK-IMP-086/087.
