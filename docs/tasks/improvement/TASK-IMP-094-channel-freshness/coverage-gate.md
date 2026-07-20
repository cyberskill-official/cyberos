---
artefact: coverage-gate@1
task: TASK-IMP-094
phase: testing
tests_failed: 0
files_below_90pct: []
ecm_rows_uncovered: []
---
# TASK-IMP-094 coverage gate

Raw terminal: `bash tools/install/tests/test_channels.sh` -> `channels: pass=24 fail=0` (incl. t_shared_skills_and_devin_rules with the CYBEROS_AGENTS exclusion arm, t_channel_idempotence snapshot compare, t_shared_skills_resolve no-dangling); `bash tools/install/tests/test_install_hygiene.sh` -> `install-hygiene: 19 passed, 0 failed` (t01 extended: gitignore round-trip + uninstall arm).

| File | Covered by | Coverage |
|---|---|---|
| tools/install/install.sh (step 5b + gitignore block) | channels x3 + hygiene t01 | create, filter, fallback, idempotence, round-trip |
| tools/install/uninstall.sh (section 2b) | hygiene t01 uninstall arm | ours-only strip + dir prune |

TRACE-004: 1.1+1.2 -> t_shared_skills_and_devin_rules passed; 1.4 -> t_channel_idempotence passed; 1.3+1.5 -> hygiene t01 extended passed; AC 4 -> t_shared_skills_resolve passed. Live scratch install: symlinks resolve relative (`ship-tasks -> ../../.claude/skills/ship-tasks`), both pointers present, `.windsurfrules` kept. ECM rows uncovered: none. Disclosure carried from review: "three commands" mapped to the three payload skills; uninstall keeps the tracked rules pointers (t01 pins the behavior).
