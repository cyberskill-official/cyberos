# TASK-IMP-096 code review

Reviewer: ship-tasks batch-4 install-trio agent (serial after TASK-IMP-094/095). Diff:
`tools/install/install.sh` step 7 (one guarded echo after the summary heredoc),
`tools/install/tests/test_install_hygiene.sh` (t09_nongit_summary_line).

## Clause -> proof

| Clause | Requirement | Proof |
|---|---|---|
| 1.1 | non-git target -> summary includes the line with the verbatim remedy | install.sh:836-838; live non-git scratch prints, as the summary's last line: `cyberos install: this repo is not a git checkout - ship-tasks needs one; run: git init -b main && git add -A && git commit -m init` (gate log E2); t09 non-git arm: count=1 + verbatim-remedy grep |
| 1.2 | git checkout -> no line | t09 git arm (zero matches); live git-repo run: 0 matches (gate log E3); t05_no_hookspath_regression's exact-summary pin still green = no other output drift |
| 1.3 | hygiene scenario, non-git (NO_MIGRATE ok) + git arms | t09_nongit_summary_line in the suite tail; install-hygiene: 19 passed, 0 failed (gate log E1) |

## Judgment

- **Correctness vs ticket**: the requirement moves from "discovered at the first phase commit"
  to "stated at install", with the remedy inline. IMP-12's observation O3 is closed.
- **Detection parity**: the probe is `git -C "$root" rev-parse --show-toplevel` - the same
  resolver the installer's own `root=` line uses (install.sh:30), per spec edge 1. A worktree
  or submodule (`.git` FILE, valid) is silent; a stale `.git` remnant still gets the line -
  the remnant arm in t09 pins exactly this distinction, so a future rewrite to `-d .git`
  fails the suite. Note the deliberate contrast left untouched: step 6b probes `-d .git` for
  hook placement; that is a different question (where hooks go) and out of this task's scope.
- **Blast radius**: one `if !` + one echo at the summary tail; the git path adds zero bytes of
  output. Non-git installs already print the hook aside "skipped (not a git checkout)" - the
  new line's phrase is distinct ("this repo is not a git checkout - ship-tasks needs one"), so
  the count=1 assertion cannot be satisfied or confused by the aside.
- **Failure mode if wrong**: line on git installs (killed by the git arm + the exact-summary
  pin), missing line on plain dirs (non-git arm), wrong semantics on gitfile shapes (remnant
  arm). git-binary-absent behaves as non-git - correct, ship-tasks cannot run there either.
- **Security**: fixed-string echo; the remedy is named, never executed (spec Non-Goal held).
- **Scenario naming**: spec names `t09_nongit_summary_line`; hygiene had t01-t08 after this
  batch's t08, so t09 was free - landed under exactly that name, no remap.

Verdict: no open findings.

HALT: review acceptance (reviewing -> ready_to_test) is a human gate.
