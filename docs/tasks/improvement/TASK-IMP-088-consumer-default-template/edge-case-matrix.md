---
artefact: edge-case-matrix@1
task_id: TASK-IMP-088
total_rows: 7
created: 2026-07-16
verdict: pass (edge-case-matrix-audit: every category >=1 row, covered-by names real test functions or recorded evidence, SECURITY rows point at code+test, DEGRADATION rows carry detection+recovery)
---
# Edge-case matrix - TASK-IMP-088

Test functions live in tools/install/tests/test_install_hygiene.sh.

| # | category | trigger | expected behavior | covered by |
|---|---|---|---|---|
| 1 | null/empty | bare consumer repo, no .cyberos/config.yaml | scaffold created once with the LIVE `task_template: task@1` line; `# coverage_threshold: 90` and `# profile: full` (the neighbours) still present exactly as today; no `engineering-spec@1` text anywhere | t06_consumer_template_default |
| 2 | null/empty | platform marker exists but is EMPTY (0 bytes) | `is_platform_repo()` tests `-f`, not size - treated as platform; commented default kept | t06_platform_keeps_comment (the fixture marker is created with `: >`, i.e. 0 bytes) |
| 3 | bounds/order | step 3b (old line ~172) calls a function formerly defined at old line ~362 | without the hoist: `command not found` (127) inside a non-final `&&` under `set -e` = NO abort, consumer line silently scaffolded on the platform repo. With the hoist the call is defined-before-use | t06_platform_keeps_comment (fails exactly if the definition is not visible at step 3b) |
| 4 | malformed | consumer repo that HAPPENS to carry modules/memory/memory.schema.json | treated as platform - documented (spec §3); the guard is the pre-existing detector, unchanged, and the marker is exercised directly | t06_platform_keeps_comment |
| 5 | concurrency/order | re-install over an existing config.yaml (operator-edited; fixture even carries a live `task_template: engineering-spec@1` line) | byte-identical file after install - create-once regression; the operator's contrary choice sticks forever | t06_existing_config_untouched (cmp against a pre-install copy) |
| 6 | SECURITY | scaffold content lands in a gitignored local config | `cfg_tmpl_line` is one of two FIXED literals chosen by a `-f` test - no operator input flows into the heredoc line, no execution surface (spec §3: security-class none) | inspection (install.sh step 3b, `cfg_tmpl_line=` branch) + t06_consumer_template_default (asserts the exact line with `grep -qx`) |
| 7 | DEGRADATION | operator wants engineering-spec@1 in a consumer repo after the new default lands | edits the live line in place - the line is inspectable and overridable (the recorded IMP-06 rationale for a config line over an invisible chain default). detection: `cat .cyberos/config.yaml`; recovery: one-line edit, create-once preserves it on every future install | t06_existing_config_untouched (proves an operator-authored file survives re-install byte-for-byte) |

Documented-by-design: migrating existing consumer repos is out of scope (their config.yaml is theirs - spec Non-Goals); the resolution chain default when config.yaml is silent stays engineering-spec@1 (chain untouched by decision).
