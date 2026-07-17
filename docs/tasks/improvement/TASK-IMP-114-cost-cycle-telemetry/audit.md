---
task_id: TASK-IMP-114
audited: 2026-07-17
verdict: PASS (after revision)
score_pre_revision: 7/10
score_post_expansion: 9/10
score_post_revision: 10/10
issues_resolved: 7
template: task@1
audit_rubric_version: audit_rubric@2.0
audited_file_sha256: a9157c15b9ac350c
audited_body_sha256_prefix: 1c0adbf5b4bcfa0f
machine_floor: task-lint clean (exit 0) - run FIRST per TASK-IMP-084
---

## §1 - Verdict summary

Spec is 70 lines, 6 §1 clauses, 5 ACs, 6 edge cases. Makes the loop's economics visible from artefacts that already exist. Passes after 7 findings - one a real interaction with a shipped task.

## §2 - Findings (all resolved)

### ISS-001 - A varying rendered field would break TASK-IMP-082's byte-stability
The status page is byte-stable by design (the fp- corpus fingerprint). A token count reported by one harness and not another makes two renders of one corpus differ - re-introducing exactly the churn 082 removed, in the same file 082 fixed. Resolved: new §1 #1.6 requires every RENDERED field to derive deterministically from committed artefacts; a non-deterministic value lives in the artefact, never the row. AC 5 asserts an unchanged corpus re-renders byte-identical; §3 carries the edge case.

### ISS-002 - An incomplete batch has no wall time
Computing a duration to now for a cut batch invents a fact - this run was cut twice. Resolved: §3 requires marking it `incomplete` rather than fabricating a number.

### ISS-003 - Zeroed tokens would assert an unmeasured fact
A zero is a claim. Resolved: §1 #1.2 requires omission, not zeroing; AC 2 asserts the row degrades rather than lying.

### ISS-004 - Dollar estimates were tempting and expire
Prices change and harnesses differ. Resolved: Alternatives rejects them; Non-Goals forbids them.

### ISS-005 - A metric row is one step from a budget gate
A number on a page invites a threshold on the number. Resolved: §1 #1.5 forbids gating; AC 4 verifies the negative structurally.

### ISS-006 - A new writer on the phase path would widen the blast radius
Collecting economics could have meant instrumenting every step. Resolved: §1 #1.3 requires derivation from existing artefacts only.

### ISS-007 - Shared-file conflicts with 108 and four siblings were unrecorded
render-status-hub and ship-tasks are both contended. Resolved: Dependencies carries the §11a serialisation note.

## §3 - Resolution

All 7 concerns addressed. The machine floor (task-lint) ran FIRST and was clean before any judgment family was applied, per TASK-IMP-084. **Score = 10/10.**

---

*End of TASK-IMP-114 audit.*
