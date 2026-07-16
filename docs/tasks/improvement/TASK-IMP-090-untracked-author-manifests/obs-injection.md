# TASK-IMP-090 observability injection

The deliverables are a prose default, a gitignore seed, an index operation, and a tracked
record - none of them execute at runtime, so there is no state transition to log, no external
IO to span, and no error branch to count. Recording that honestly is the correct output here
rather than inventing telemetry for a `printf`.

What stands in for observability:
- **install.sh emits its own trace**: the seed block is inside the install log the operator
  reads; a re-vendor that appends the pattern is visible in `git diff` on the consumer side.
- **The index operation is self-evidencing**: `git ls-files docs/tasks/.workflow` is the check,
  recorded in the gate log (AC 3).
- **The suite is the monitor**: t07 fails loudly if either seed path regresses; it runs on
  every suite invocation, which is the only signal this change class can produce.

Branch coverage: 2 of 2 seed paths (fresh, append-once) asserted; 100 % of the change's
executable surface.
