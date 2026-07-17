---
task_id: TASK-IMP-115
audited: 2026-07-17
verdict: PASS (after revision)
score_pre_revision: 8/10
score_post_expansion: 9/10
score_post_revision: 10/10
issues_resolved: 6
template: task@1
audit_rubric_version: audit_rubric@2.0
audited_file_sha256: ae8cd4465c182944
audited_body_sha256_prefix: 0c3ebeeece2a8544
machine_floor: task-lint clean (exit 0) - run FIRST per TASK-IMP-084
---

## §1 - Verdict summary

Spec is 66 lines, 5 §1 clauses, 5 ACs, 5 edge cases. Adopts BUILD 3's insight while rejecting its host-specific encoding. Passes after 6 findings.

## §2 - Findings (all resolved)

### ISS-001 - Clause 1.5 carried a MUST that no AC cited - caught by the machine floor
task-lint fired TRACE-001: the ambiguity rule (ambiguous steps are `medium`, not guessed `high`) was normative but untested. Resolved: AC 5 added with a justified `verify:` - no suite can decide whether a level was guessed, so the evidence is a recorded reviewer walk. The floor caught this before a human read the spec, which is exactly why TASK-IMP-084 runs it first.

### ISS-002 - Model strings would expire before 1.0.0 ships
A `claude-fable-5` literal in the payload is a rule with a shelf life. Resolved: §1 #1.4 forbids model strings, prices, and effort names; AC 3 asserts the negative across the payload.

### ISS-003 - An advisory field could be read as instruction
A field named judgment invites a reader to route on it. Resolved: §1 #1.3 documents it as advisory and forbids the payload reading it; AC 4 verifies.

### ISS-004 - Overstating a step's needs restores the expensive default
Marking everything high makes the field useless. Resolved: §1 #1.5 requires `medium` when genuinely ambiguous; AC 5 makes the assignment reviewable.

### ISS-005 - Inferring the level from the skill name was the cheap option
It is exactly the implicit rule this run keeps finding wrong. Resolved: Alternatives records it - if it matters, write it down.

### ISS-006 - mechanical could drift when a helper is replaced by a model
The field would then lie. Resolved: AC 2 asserts every mechanical step is helper-backed, which reds on drift; §3 names the case.

## §3 - Resolution

All 6 concerns addressed. The machine floor (task-lint) ran FIRST and was clean before any judgment family was applied, per TASK-IMP-084. **Score = 10/10.**

---

*End of TASK-IMP-115 audit.*
