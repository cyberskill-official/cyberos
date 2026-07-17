# TASK-IMP-096 edge-case matrix

| # | Category | Trigger | Expected | Covered by |
|---|---|---|---|---|
| 1 | NON-GIT | plain directory install (CYBEROS_NO_MIGRATE path) | exactly one line: requirement + verbatim remedy `git init -b main && git add -A && git commit -m init` | t09_nongit_summary_line (non-git arm: count=1 + verbatim grep) |
| 2 | GIT | normal git checkout install | no such line; summary otherwise unchanged | t09_nongit_summary_line (git arm) + t05_no_hookspath_regression (pins an exact unrelated summary line - proves no drift) |
| 3 | .git IS A FILE (worktree/submodule) | valid gitfile: `git rev-parse` succeeds | counts as a checkout -> silent (rev-parse semantics, not `-d .git`) | install.sh:836 uses `git -C "$root" rev-parse`; stale-remnant arm proves the probe is rev-parse-shaped; reviewed |
| 4 | STALE .git REMNANT | `.git` file pointing nowhere: rev-parse fails | still "not a checkout" -> line prints | t09_nongit_summary_line (stale-remnant arm) |
| 5 | git BINARY ABSENT | no git on PATH | command failure = falsy -> line prints (correct: ship-tasks cannot run) | install.sh:836 `if ! git ...` shape; reviewed (mirrors root resolution's own fallback) |
| 6 | DOUBLE HINT | hook line already says "skipped (not a git checkout)" | both may appear; the new line's distinct phrase ("this repo is not a git checkout - ship-tasks needs one") is counted once | t09 greps the distinct phrase with count=1 |
| 7 | REMEDY VALIDITY | remedy must work verbatim on a fresh directory | `git init -b main && git add -A && git commit -m init` - standard commands, no repo state assumed | spec guardrail; asserted verbatim-present by t09; ops-verified class (recorded rationale in spec) |
| 8 | SECURITY | line could execute or leak | one echo; no commands executed on the consumer's behalf, no paths beyond none | reviewed - fixed string |
