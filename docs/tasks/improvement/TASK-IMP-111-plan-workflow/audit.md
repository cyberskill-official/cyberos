---
task_id: TASK-IMP-111
audited: 2026-07-17
verdict: PASS (after revision)
score_pre_revision: 6/10
score_post_expansion: 9/10
score_post_revision: 10/10
issues_resolved: 8
template: task@1
audit_rubric_version: audit_rubric@2.0
audited_file_sha256: bb610bd77d1371d2
audited_body_sha256_prefix: 9ac661d0b6cd123e
machine_floor: task-lint clean (exit 0) - run FIRST per TASK-IMP-084
---

## §1 - Verdict summary

Spec is 108 lines, 9 §1 clauses, 7 ACs, 6 edge cases. Closes the verified gap that create-tasks cannot accept an idea, re-composing ~70% from existing skills rather than inventing a workflow. Passes after 8 findings.

## §2 - Findings (all resolved)

### ISS-001 - Plan could become a second writer to docs/tasks
A second writer to the corpus re-opens the 086 class - create-tasks owns the audited write path. Resolved: §1 #1.7 forbids writing tasks, rows, code, or statuses; AC 4 asserts both paths byte-identical after a run.

### ISS-002 - Extending repo-context-map risks moving the ship-tasks path
The scan is a live step-1 dependency; changing its default breaks the workflow that works. Resolved: §1 #1.3 requires `scope: task` to behave exactly as today; AC 2 asserts byte-identical output.

### ISS-003 - A second HITL gate would make both rubber stamps
create-tasks gates the same content minutes later. Resolved: one gate at the decision (§1 #1.5); the duplicate is an explicit Non-Goal with the reasoning recorded.

### ISS-004 - Ambiguous mode detection could guess
Guessing greenfield on a live repo plans against a codebase that exists. Resolved: §1 #1.1 requires a halt; AC 1 asserts the ambiguous fixture halts.

### ISS-005 - An unbounded scan cannot finish on a large repo
A scan exceeding the sandbox cap is a scan that gets skipped. Resolved: §3 requires bounding and reporting what was sampled rather than implying exhaustiveness.

### ISS-006 - Building IMP-21 triage separately would create two front doors
Both grade an idea before authoring. Resolved: Alternatives records the explicit rejection; triage's park/needs_info become plan outcomes, needs_spike is already a step.

### ISS-007 - The output contract could drift from what create-tasks accepts
A new input shape means touching task-author, widening the blast radius. Resolved: §1 #1.8 requires §6 consumable with no contract change; AC 5 asserts an idea-only plan feeds create-tasks unmodified.

### ISS-008 - An idea already covered by an existing task would be re-proposed
Planning a duplicate is worse than not planning. Resolved: §3 requires the brownfield scan to surface it and the option set to include 'this exists'.

## §3 - Resolution

All 8 concerns addressed. The machine floor (task-lint) ran FIRST and was clean before any judgment family was applied, per TASK-IMP-084. **Score = 10/10.**

---

*End of TASK-IMP-111 audit.*
