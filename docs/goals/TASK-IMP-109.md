---
source: TASK-IMP-109
born: 2026-07-17
status: satisfied
last_pass: 2026-07-17
on_violation: report
retire_when: "standing goals are removed from the workflow. Retirement is a human decision, logged."
predicates:
  - tools/install/tests/test_verify_goals.sh
---
# TASK-IMP-109 - standing goals

Enrolled at its own `done` flip, by its own §11c rule. The predicate is the task's §1 cited test.

The guard arms (t07/t08/t09) are inside this predicate, so if anyone ever loosens the confinement,
the tracked-check, or the refuse-don't-skip rule, this goal violates and says so.
