---
artefact: coverage-gate@1
task: TASK-IMP-095
phase: testing
tests_failed: 0
files_below_90pct: []
ecm_rows_uncovered: []
---
# TASK-IMP-095 coverage gate

Raw terminal: hygiene `t08_gates_env_regen_notice` ok (three arms: edited -> notice naming a real .bak; unedited re-install -> silent; fresh -> silent) inside `install-hygiene: 19 passed, 0 failed`.

| File | Covered by | Coverage |
|---|---|---|
| tools/install/install.sh (step 3 notice) | t08 x3 arms | 3/3 branches |

TRACE-004: 1.1 -> t08 (edited arm) passed; 1.2 -> t08 (silent arms) passed; 1.3 -> AC 3 ops (suite summary counts t08; recorded). Live capture on a scratch repo: `cyberos install: gates.env regenerated (previous kept at .../.cyberos/gates.env.bak.1784237283); durable overrides belong in .cyberos/config.yaml`. ECM rows uncovered: none.
