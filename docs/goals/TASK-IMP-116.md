---
source: TASK-IMP-116
born: 2026-07-17
status: satisfied
last_pass: 2026-07-17
on_violation: report
retire_when: "BACKLOG.md stops carrying a Totals line, or the index stops being derived from task frontmatter. Retirement is a human decision, logged."
predicates:
  - tools/install/tests/test_workflow_helpers.sh
---
# TASK-IMP-116 - every mutation emits the whole file's truth

Enrolled at its own `done` flip, by §11c. The predicate is the task's §1 cited test: t06 pins the retally (counting independently, with its own awk tally, rather than asserting a literal), t07 and t11 pin the 3-line footprint ceiling in all three shapes - counted header 3/3, bare header 2/2, insert 2/3 - and t11 additionally pins that a bare header is never rewritten and a file with no Totals line is never given one.

If anyone widens the footprint past three lines, drops the retally back to a section-local count, or starts inventing a Totals line where the corpus never had one, this goal violates and says so.

## What this goal does NOT cover, and why that matters

It pins the INDEX write only. It cannot see the record of truth.

116's own coverage gate caught the gap that proves the point: the BACKLOG row said `testing` while the task's own spec.md frontmatter still said `reviewing`, because two flips moved the index and nothing moved the truth. Per STATUS-REFERENCE §1 the frontmatter IS the record of truth; the BACKLOG is its index. `backlog-mutate` executes the index write and nothing binds the frontmatter write to it, so they can silently diverge - and did, inside the task built to stop the index from lying.

Totals agreeing with the rows therefore proves the file is INTERNALLY consistent. It does not prove the file is TRUE. A corpus-wide frontmatter-vs-index reconciliation is the missing predicate; until one exists, this goal is narrower than its title sounds, and that is stated here rather than discovered later.
