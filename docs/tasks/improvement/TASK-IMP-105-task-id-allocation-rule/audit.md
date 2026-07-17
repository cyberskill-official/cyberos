---
task_id: TASK-IMP-105
audited: 2026-07-17
verdict: PASS (after revision)
score_pre_revision: 7/10
score_post_expansion: 9/10
score_post_revision: 10/10
issues_resolved: 6
template: task@1
audit_rubric_version: audit_rubric@2.0
audited_file_sha256: 1e2ef8de26de3d16
audited_body_sha256_prefix: 27f83b4bf7e3d722
machine_floor: task-lint clean (exit 0) - run FIRST per TASK-IMP-084
---

## §1 - Verdict summary

Spec is 79 lines, 6 §1 clauses, 5 ACs, 5 edge cases. Gap verified on main: task-author states no allocation rule; backlog-mutate.mjs:258 is the late net. Passes after 6 findings.

## §2 - Findings (all resolved)

### ISS-001 - Allocation could race between PLAN and write
An id chosen at PLAN time may be taken by the time files land. Resolved: §1 #1.1 requires a re-scan immediately before writing; AC 5 verifies the rule is stated.

### ISS-002 - Half-landed folder with no row would be skipped
Counting rows rather than folders re-issues the exact colliding id this task prevents. Resolved: §3 edge case makes the folder authoritative.

### ISS-003 - Gap reuse would make two tasks share a name in history
Taking the lowest free number recycles a retired id. Resolved: §1 #1.5 requires highest+1; AC 3 asserts it against a gapped corpus.

### ISS-004 - Could be read as replacing the uniqueness gate
It narrows the window; it does not close it. Resolved: §1 #1.6 keeps the gate authoritative, AC 4 asserts exit-7 still fires, and §3 names the residual race honestly.

### ISS-005 - Module argument is a path component - a traversal surface
`next-id ../../etc` would walk out of the corpus. Resolved: §3 security-class requires the `relUnderRoot` confinement the batch-5 review forced onto task-reconcile.

### ISS-006 - Empty module could be treated as an error
A module's first task must be allocatable. Resolved: §1 #1.4 makes empty legal; AC 2 covers it.

## §3 - Resolution

All 6 concerns addressed. The machine floor (task-lint) ran FIRST and was clean before any judgment family was applied, per TASK-IMP-084. **Score = 10/10.**

---

*End of TASK-IMP-105 audit.*
