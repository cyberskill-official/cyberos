---
task_id: TASK-IMP-106
audited: 2026-07-17
verdict: PASS (after revision)
score_pre_revision: 8/10
score_post_expansion: 9/10
score_post_revision: 10/10
issues_resolved: 6
template: task@1
audit_rubric_version: audit_rubric@2.0
audited_file_sha256: 38af283622f1b2ab
audited_body_sha256_prefix: 50bd38390e75cddf
machine_floor: task-lint clean (exit 0) - run FIRST per TASK-IMP-084
---

## §1 - Verdict summary

Spec is 72 lines, 5 §1 clauses, 3 ACs, 5 edge cases. Same failure shape as TASK-IMP-095/096 - a correct default undocumented at the moment it surprises someone - and both proved worth fixing. Passes after 6 findings.

## §2 - Findings (all resolved)

### ISS-001 - depends_on was empty despite sharing uninstall.sh with 103
103 adds the lock-removal branch this summary reports; concurrent edits to one file violate §11a. Resolved: `depends_on: [TASK-IMP-103]` with reciprocal `blocks`, evidence gate named.

### ISS-002 - A hard-coded kept list would drift from behavior
Printing four fixed paths claims a path that may not exist. Resolved: §1 #1.4 requires derivation; AC 2 asserts a missing status dir is not claimed.

### ISS-003 - Scope creep toward a purge flag
The defect is silence, not a missing capability. Resolved: explicit Non-Goal, keeping the task at one hour.

### ISS-004 - Summary could imply uninstall changed behavior
A new kept list might read as a new policy. Resolved: §1 #1.5 forbids behavior change; AC 3 asserts the post-uninstall file set is byte-identical.

### ISS-005 - Never-installed repo would print a misleading kept list
Nothing was kept because nothing was removed. Resolved: §3 edge case suppresses the block entirely.

### ISS-006 - Printed removal command is a shell string
A command in output the script might later execute is an injection surface. Resolved: §3 security-class fixes it as documentation, never executed, no interpolation.

## §3 - Resolution

All 6 concerns addressed. The machine floor (task-lint) ran FIRST and was clean before any judgment family was applied, per TASK-IMP-084. **Score = 10/10.**

---

*End of TASK-IMP-106 audit.*
