---
source: TASK-IMP-110
born: 2026-07-17
status: satisfied
last_pass: 2026-07-17
on_violation: report
retire_when: "the outer loop is removed, or skill amendments stop being proposed from run evidence. Retirement is a human decision, logged."
predicates:
  - tools/install/tests/test_workflow_improver.sh
---
# TASK-IMP-110 - the outer loop proposes, never edits

Enrolled at its own `done` flip, by §11c. The predicate is the task's §1 cited tests: t04 pins that
a run leaves `modules/**` byte-identical (§1.4 - it PROPOSES, it never edits), t06 that proposals
land at `draft` and are never self-audited into `ready_to_implement` (§1.5), t02 that a single
occurrence is an anecdote and yields nothing (§1.3), t05 that a clean window emits nothing rather
than padding to the cap (§1.6), t07 that the payload actually carries it.

If anyone lets the outer loop write a skill directly, self-audit its own proposals, promote an
anecdote, or pad a clean window, this goal violates and says so.

## What this goal does NOT prove, and why it matters more than usual

The predicate is a FIXTURE suite. It proves the tool behaves correctly on constructed windows. It
cannot prove the tool finds anything real.

Against the live corpus at the `done` flip, workflow-improve reads the right 20 shipped tasks and
finds ZERO evidence rows: every windowed task carries `routed_back_count: 0`, the 17 gate logs
contain no structured `reason:` rows, and no reconcile reports exist. It correctly reports "no
amendment proposed". The spec's premise - "every ingredient exists" - is half true: the FIELD
exists; the reasons it wants clustered were never written. This week's operator corrections live in
prose addenda, which §3 explicitly calls untrusted input.

So this goal will report `satisfied` forever while the tool finds nothing, and both facts are
correct simultaneously. A standing goal inherits the quality of its predicate; it makes `done` a
maintained claim, not a true one. That distinction was learned the hard way on TASK-IMP-108 §1.7
and is restated here rather than rediscovered.

The real gate on this tool's value is TASK-IMP-112 (structured review findings). Until it lands,
110 is a correct engine with an empty tank, and this file says so.
