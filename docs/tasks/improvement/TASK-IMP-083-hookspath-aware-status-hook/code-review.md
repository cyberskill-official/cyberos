# TASK-IMP-083 — code review packet

Files under review: `tools/install/install.sh` (step 6b), `tools/install/uninstall.sh`
(root resolution + hook section), `tools/install/tests/test_install_hygiene.sh`
(t05_hookspath_* block). Suite state at review: 13 passed, 0 failed (~9 s), including all
pre-existing t01–t06.

## §1 clause → proof

| Clause | Requirement | Proof |
|---|---|---|
| 1.1 | resolve effective hooks dir: `core.hooksPath` when set/non-empty (relative → repo root, absolute → as is), else `.git/hooks` | resolver in step 6b (`[ -z ]` + `case /*)`); test `t05_hookspath_standalone` (relative, anchored at root — repo created under mktemp, test cwd elsewhere); absolute arm is the same one-line `case`, code-reviewed (spec §3 accepts this split); empty-string → same `[ -z ]` branch |
| 1.2 | UNCHANGED state machine against `<hooks_dir>/pre-commit`; `mkdir -p` the dir | state machine diff-clean (only `hk`/`mkdir` inputs and `HOOK_SET` suffixes changed); tests `t05_hookspath_standalone` (absent → standalone v2, dir created from nothing), `t05_hookspath_foreign_append` (foreign → marked append, exactly once, idempotent on re-install); v1-upgrade / v2-keep arms byte-untouched and still covered at the default dir by t06 |
| 1.3 | summary names the path actually written | `hook_at=" at <hooksPath>/pre-commit"` appended inside the five HOOK_SET strings; test `t05_summary_names_path` (`" at .githooks/pre-commit"` in install output) |
| 1.4 | hooksPath unset → every written byte and summary word identical to today | ops proof: install from pre-change committed payload (`dist/cyberos`, zero `hooksPath` matches) vs new payload on no-hooksPath repos — hook bytes `cmp`-identical, auto-sync line `diff`-identical; test `t05_no_hookspath_regression` pins the exact wording with `grep -qF` + negative hooksPath greps |
| 1.5 | uninstall resolves the same way, removes/strips at `<hooks_dir>/pre-commit`, never touches `.git/hooks/pre-commit` when hooksPath points elsewhere; ownership test becomes the exact line-2 `_cyberos_owns_hook` | resolver copied; `_cyberos_owns_hook` copied verbatim with a comment crediting the install-side fix and naming the head-5 bug; tests `t05_hookspath_uninstall` (removed from `.githooks/`, `.git/hooks` untouched) and `t05_short_foreign_uninstall_preserved` (strip-only, foreign bytes preserved). Ops proof the old heuristic was destructive: `git show HEAD:tools/install/uninstall.sh` (root line patched only) on a 3-line foreign hook + real appended block → "removed managed pre-commit hook", file deleted whole |
| 1.6 | non-git targets keep the skip behavior | skip branch untouched (resolver sits inside the `else`); test `t05_non_git_skip` ("skipped (not a git checkout)") |
| 1.7 | hygiene coverage as a new scenario block in the existing suite | seven `t05_hookspath_*` functions, foot-called with shared counters; standalone-fires-on-backlog-commit asserted node-gated (commit tree contains `docs/status/`); AC 6's `run_all.sh` pass is the batch parent's recorded ops check |

## Acceptance criteria

AC 1 `t05_hookspath_standalone` ok · AC 2 `t05_hookspath_foreign_append` ok · AC 3
`t05_no_hookspath_regression` ok · AC 4 `t05_hookspath_uninstall` ok · AC 5
`t05_summary_names_path` ok · AC 6 suite-integrated (run_all.sh = parent's gate log) ·
AC 7 `t05_non_git_skip` ok · AC 8 `t05_short_foreign_uninstall_preserved` ok.

## Implementer notes / issues for the reviewer

- ISS-1 (necessary enabling fix, same owned file): uninstall.sh's `root=` was mis-grouped
  — `((cd && rev-parse) || cd) && pwd` — so `$root` captured two newline-joined paths and
  every uninstall on a git repo exited "nothing to do"; the hook section this task
  modifies was unreachable. Fixed with explicit grouping + comment. Pre-existing, exposed
  by the first test that ever exercised uninstall; without it ACs 4/8 cannot execute.
- ISS-2: `t05_summary_names_path` asserts on `t05_hookspath_standalone`'s captured output
  (same scenario, one fewer install; introduces an intra-block ordering dependency,
  commented at the capture site).
- ISS-3: legacy inert hook at `.git/hooks/pre-commit` on hooksPath repos is not migrated
  (spec §3 known leftover); `t05_hookspath_uninstall` pins that we do not touch it.
- Protected invariants re-checked: no foreign hook clobbered (exact ownership both sides);
  append block still POSIX sh, foreign exit code preserved (exit-7 asserts); `dist/`
  untouched here — rebuild, version-sync and full suite before commit are the batch
  parent's step per payload-sync doctrine.

## Verdict

| Area | Verdict |
|---|---|
| §1 conformance (1.1–1.7) | pass |
| ACs 1–8 | pass (AC 6 pending parent's run_all.sh gate log) |
| Regression contract (bytes + words, no hooksPath) | pass (suite + dist-control diff) |
| Data-loss class (head-5) | fixed; regression-tested by construction |
| Invariants (§5) | intact |

HALT: review acceptance (reviewing -> ready_to_test) is a human gate.
