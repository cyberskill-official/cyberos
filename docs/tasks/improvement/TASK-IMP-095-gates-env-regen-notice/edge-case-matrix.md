# TASK-IMP-095 edge-case matrix

| # | Category | Trigger | Expected | Covered by |
|---|---|---|---|---|
| 1 | FRESH | no prior gates.env (first install) | silent - `env_bak` stays empty, guard short-circuits | t08_gates_env_regen_notice (fresh arm) |
| 2 | IDENTICAL | unedited re-install: regeneration reproduces the prior bytes | silent - `cmp -s` equal | t08_gates_env_regen_notice (unedited arm) |
| 3 | EDITED | operator-edited prior file, re-install regenerates different bytes | exactly ONE line naming the real .bak path and `.cyberos/config.yaml` | t08_gates_env_regen_notice (edited arm: count=1, `-f "$bak"`, edit greps in the .bak) |
| 4 | REPRODUCED EDIT | operator edit that regeneration happens to reproduce byte-identically | silent by design (1.2) - nothing was lost | same cmp -s branch as #2; reviewed (spec edge) |
| 5 | SAME-SECOND | two installs within one `date +%s` tick | message names whichever .bak step 3 just wrote (cp overwrites the same name); step-1 churn guard unchanged | install.sh:157-160 + 72; reviewed (spec edge) |
| 6 | DETECTION DRIFT | gate autodetect changes between installs (new Makefile target etc.) | differs from prior -> notice fires; correct: the operator's effective config DID change | same code path as #3 (asserted by construction: the test's "edit" is any byte delta) |
| 7 | SECURITY | notice could leak file content | one echo of two local paths; no content of the edited file is printed | reviewed - line is a fixed format string + `$env_bak` |
