# TASK-IMP-095 code review

Reviewer: ship-tasks batch-4 install-trio agent (serial after TASK-IMP-094). Diff: `tools/install/install.sh` step 3 (capture `env_bak`, guarded one-line echo), `tools/install/tests/test_install_hygiene.sh` (t08_gates_env_regen_notice).

## Clause -> proof

| Clause | Requirement | Proof |
|---|---|---|
| 1.1 | prior file + differing regen -> one line naming the exact .bak and config.yaml | install.sh:189-191; live scratch: `cyberos install: gates.env regenerated (previous kept at <abs>/.cyberos/gates.env.bak.1784236345); durable overrides belong in .cyberos/config.yaml` - named .bak exists and carries the edit (gate log E2); t08 edited arm asserts count=1 + `-f` + content |
| 1.2 | identical regen and fresh install silent | `[ -n "$env_bak" ]` (fresh) and `cmp -s` (identical) guards; t08 silent arms + live run5 `notice-lines=0` (gate log E3) |
| 1.3 | hygiene scenario, both arms | t08_gates_env_regen_notice in the suite tail; install-hygiene: 19 passed, 0 failed (gate log E1) |

## Judgment

- **Correctness vs ticket**: the recorded IMP-09 gap ("the sachviet run only found the .bak by listing the directory") is closed at the moment of clobber, with zero behavior change to regeneration, backup naming, or the step-1 churn guard (`rm -f gates.env.bak.*` untouched).
- **Blast radius**: one variable + one guarded echo. The `[ -f ] && cp` -> if-block rewrite is behavior-identical (same cp, same name); under `set -euo pipefail` the `if` form is also the safer shape. Nothing else in step 3 moved - t06_* (config.yaml) and t05_no_hookspath's exact-summary assertions still pass, proving no output drift elsewhere.
- **Failure mode if wrong**: a false-positive notice on every install (killed by the cmp -s identical arm), a ghost .bak path (killed by t08's `-f "$bak"` + edit grep), or a missed notice (killed by the edited arm). All three arms are suite-asserted every run.
- **Message audit**: fixed format string + one absolute path; no content of the operator's file is echoed (SEC edge #7). Wording matches the spec byte-for-byte.
- **Scenario naming**: spec names `t08_gates_env_regen_notice`; hygiene had t01-t07, so t08 was free - landed under exactly that name, no remap.

Verdict: no open findings.

HALT: review acceptance (reviewing -> ready_to_test) is a human gate.
