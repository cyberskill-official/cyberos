# TASK-IMP-096 observability injection

The deliverable is one conditional summary line in the installer - nothing executes at
consumer runtime afterwards, so there is no transition to log or IO to span. This task, like
TASK-IMP-095, is itself a piece of operator-facing observability: it surfaces a latent
precondition (ship-tasks needs git) at the cheapest possible moment.

- **Signal design**: prints only when `git rev-parse` fails against the install root - the
  exact predicate under which every later phase commit would fail. Git installs stay
  byte-identical (a no-op signal cannot desensitize anyone).
- **Actionability**: the line carries its own remedy, verbatim and copy-pastable
  (`git init -b main && git add -A && git commit -m init`) - observability that ends the
  incident it reports.
- **The suite is the monitor**: t09 asserts the line fires once on non-git, fires on a stale
  `.git` remnant (semantics guard), and never fires on git - every suite run.

Branch coverage: 2 of 2 branches of the new conditional asserted (fails -> line; succeeds ->
silence), plus the remnant sub-case of the failing branch - 100 % of the change's executable
surface.
