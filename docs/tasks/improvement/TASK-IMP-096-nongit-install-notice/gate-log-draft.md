# TASK-IMP-096 gate-log evidence (implementing -> ready_to_review)

E1 - hygiene suite (AC 1, 2) verbatim tail:
  ok   t07_workflow_gitignore_patterns
  ok   t08_gates_env_regen_notice
  ok   t09_nongit_summary_line
  install-hygiene: 19 passed, 0 failed

E2 - live non-git scratch install (AC 1) - summary tail, captured verbatim:
  $ mkdir plaindir && CYBEROS_NO_MIGRATE=1 bash <payload>/install.sh plaindir | tail -3
  .cyberos/AGENT-ENTRY.md, like CLAUDE.md). An agent working here records decisions, audits,
  and plans into the BRAIN per that protocol. Skip with CYBEROS_NO_MEMORY=1 on install.
  cyberos install: this repo is not a git checkout - ship-tasks needs one; run: git init -b main && git add -A && git commit -m init
  $ grep -c 'this repo is not a git checkout' run6.log
  1

E3 - live git scratch install (AC 2):
  $ grep -c 'this repo is not a git checkout' run1.log
  0
  (t05_no_hookspath_regression's fixed-string summary assertion also stayed green - the git
   path's output did not drift by a byte it pins)

E4 - semantics probe (spec edge 1): stale `.git` FILE remnant (`gitdir: /nonexistent`) ->
  rev-parse fails -> line prints (t09 stale-remnant arm ok). A valid worktree gitfile makes
  rev-parse succeed -> silent by the same predicate the installer's root resolution uses.
