---
task_id: TASK-IMP-110
audited: 2026-07-17
verdict: PASS (after revision)
score_pre_revision: 6/10
score_post_expansion: 9/10
score_post_revision: 10/10
issues_resolved: 7
template: task@1
audit_rubric_version: audit_rubric@2.0
audited_file_sha256: 5de39998862d43c9
audited_body_sha256_prefix: 112f4500a8a2a78d
machine_floor: task-lint clean (exit 0) - run FIRST per TASK-IMP-084
---

## §1 - Verdict summary

Spec is 92 lines, 7 §1 clauses, 7 ACs, 5 edge cases. Authors the loop TASK-IMP-028 reserved on 2026-07-08 and never specified. Passes after 7 findings.

## §2 - Findings (all resolved)

### ISS-001 - Gate logs are model-written prose - untrusted input
An improver that quotes evidence into a command is an injection path through our own artefacts. Resolved: §3 security-class requires verbatim reproduction with ids, no interpolation, and `relUnderRoot` confinement - the TASK-IMP-100 rung-5 rule.

### ISS-002 - Applying amendments directly would delete the human-accepts premise
The article's outer agent opens a PR against its own skill; our doctrine says a human accepts every change, and a skill edit is a doctrine change. Resolved: §1 #1.4 forbids writing modules/**; §1 #1.5 lands proposals as draft tasks; AC 4 asserts modules/** byte-identical after a run.

### ISS-003 - An unbounded proposal count produces a review nobody does
An unreviewed amendment to a skill is doctrine nobody agreed to. Resolved: §1 #1.2 caps at 3; AC 3 asserts 8 patterns yield exactly 3, highest-evidence first.

### ISS-004 - A single occurrence is an anecdote, not a pattern
Proposing from one event manufactures doctrine from noise. Resolved: §1 #1.3 requires >=2 independent evidence rows; AC 2 asserts the single-occurrence case yields nothing.

### ISS-005 - A clean window could be padded to the cap
An improver that must always find three findings will invent the third. Resolved: §1 #1.6 requires reporting no proposal and emitting nothing; AC 5 asserts silence.

### ISS-006 - TASK-IMP-028 would become a silent duplicate
Two tasks for one idea, the older left as a stub nobody closes. Resolved: §1 #1.7 requires flipping 028 to `duplicate` with a resolving `duplicate_of` (FM-113); AC 7 covers it.

### ISS-007 - Confidence-thresholded auto-apply was tempting and is wrong
Confidence is the model's opinion of itself - precisely what the two-gate design exists not to trust. Resolved: recorded in Alternatives as rejected.

## §3 - Resolution

All 7 concerns addressed. The machine floor (task-lint) ran FIRST and was clean before any judgment family was applied, per TASK-IMP-084. **Score = 10/10.**

---

*End of TASK-IMP-110 audit.*
