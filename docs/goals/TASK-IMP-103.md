---
source: TASK-IMP-103
born: 2026-07-17
status: satisfied
last_pass: 2026-07-17
on_violation: report
retire_when: "the install lock is removed. Retirement is a human decision, logged."
predicates:
  - tools/install/tests/test_install_lock.sh
---
# TASK-IMP-103 - install concurrency lock

Graduated at the `done` flip (2026-07-17). The predicate is the task's §1 cited test - the same
suite TRACE-004 verified, so this goal claims nothing the acceptance did not already prove.

Every AC on this task cited a `test:`, so there are no `verify:`-only ACs to exclude.
