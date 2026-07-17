---
task_id: TASK-IMP-109
audited: 2026-07-17
verdict: PASS (after revision)
score_pre_revision: 6/10
score_post_expansion: 9/10
score_post_revision: 10/10
issues_resolved: 8
template: task@1
audit_rubric_version: audit_rubric@2.0
audited_file_sha256: bfa0f04651c6bf2b
audited_body_sha256_prefix: 764f86dbe41a6631
machine_floor: task-lint clean (exit 0) - run FIRST per TASK-IMP-084
---

## §1 - Verdict summary

Spec is 96 lines, 8 §1 clauses, 6 ACs, 6 edge cases. Converts `done` from a claim into a maintained invariant using predicates TRACE-004 already collects. Highest-risk task in the batch by construction - it executes commands read from files. Passes after 8 findings.

## §2 - Findings (all resolved)

### ISS-001 - Executing predicates read from files is the rung-5 defect, re-introduced
This task's core mechanism - run a command named in a repo file - is exactly what the batch-5 review caught in task-reconcile: a crafted file could name a command. Resolved: §3 security-class is marked HIGH and requires repo-root confinement plus a `git ls-tree HEAD` tracked check before execution, naming the precedent so the fix is not re-derived from scratch.

### ISS-002 - Auto-fix on violation would be the machine grading itself
An auto-fix on a violated acceptance is self-certification at the exact moment nobody is watching. Resolved: §1 #1.7 forbids status change, code change, and re-opening; AC 5 asserts detection-only.

### ISS-003 - verify:-only ACs cannot be predicates but could be faked as ones
A predicate that cannot be re-run is not a predicate. Resolved: §1 #1.3 excludes them and requires the goal to name the gap; §1 #1.4 covers the zero-predicate task with `predicate: none` rather than a fake pass.

### ISS-004 - A hanging predicate could be read as passing
A timeout returning nothing is not a success. Resolved: §1 #1.8 makes timeout a violation, named as such; AC 6 asserts it.

### ISS-005 - Re-opening the source task would destroy the acceptance record
`done` is terminal for a reason. Resolved: §1 #1.7 requires a new `type: bug` task through the normal loop.

### ISS-006 - 176 existing done tasks have no goals - the report could imply coverage
Claiming a guarantee covering a third of the corpus would be the 086 pattern. Resolved: §3 requires the report to state how many done tasks have no goal; backfill is an explicit Non-Goal.

### ISS-007 - A flaky predicate would poison the ledger and invite deletion
A goal deleted without a reason is the evidence loss this task exists to prevent. Resolved: §3 requires quarantine with a recorded reason, never silent deletion.

### ISS-008 - Scheduling would couple the payload to a host
Cron is a host decision; CyberOS is invoked. Resolved: explicit Non-Goal - the runner is a command, and when it runs is the operator's business.

## §3 - Resolution

All 8 concerns addressed. The machine floor (task-lint) ran FIRST and was clean before any judgment family was applied, per TASK-IMP-084. **Score = 10/10.**

---

*End of TASK-IMP-109 audit.*
