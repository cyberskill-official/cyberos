---
audited_file: docs/tasks/improvement/TASK-IMP-118-tests-must-test-their-clause/spec.md
audited_file_sha256: b8cea7c1934531ef
audited_body_sha256_prefix: 85fadd8a036537b9
rubric: audit_rubric@2.0
audited_at: 2026-07-17T15:10:00+07:00
auditor: claude-fable-5
verdict: pass
score: 10/10
machine_floor: task-lint 0 errors, 1 info (TRACE-001)
---

# Audit - TASK-IMP-118

Machine floor first per TASK-IMP-084: 0 errors. TRACE-001 info is the `## 1. Clauses` heading shape, as with 117; traceability is discharged below.

## Findings

ISS-001 (info, accepted): TRACE-001 heading shape. 1.1-1.5 each cite a test; AC1-AC6 each cite a clause. Traceability holds.

ISS-002 (accepted, and the point of the task): this spec is written by the agent whose defect it generalizes. That is disclosed in the AI Authorship Disclosure rather than left implicit. The mitigation is AC6, which does not ask whether the rule sounds right - it requires the rule to FAIL 108 §1.7's original test and PASS its replacement. A rule that cannot fail the case that motivated it is decoration, and this spec says so in its own success metrics.

ISS-003 (accepted): 1.5 forbids adding TRACE-006 to task-lint. This looks like refusing to automate, and is the opposite: a structural check shaped to look like TRACE-006 would pass 108 §1.7's original assertion (the string IS in the file) and restore exactly the false assurance the task exists to remove. The rule is unmechanizable by construction; saying so in the clause is what stops a later author from "improving" it into uselessness.

ISS-004 (accepted): §3 row 11 - the rule is REPORTED, never auto-failed, against tasks audited before it existed. Retroactively breaking 180 done tasks on a rule they could not have known is the machine making a scope decision that belongs to the operator. Sizing that sweep is deferred to the handoff, matching how 117 handled its own corpus question.

## Rubric families

- FM: clean (machine floor).
- SEC: seven required sections present.
- COND: three-bullet disclosure, naming the agent's authorship of the original defect.
- QA: §3 carries 11 rows across all six categories, 2 SECURITY and 2 DEGRADATION. Above the 8-row floor for MUST-priority work. Rows 7 and 8 are the sharpest: "MUST refuse" discharged by a log line, and "MUST NOT execute" discharged by a happy path, are the two nearest neighbours of the 108 §1.7 mistake.
- SAFE: adds no executable surface. Changes documents and one skill.
- TRACE: 1.1-1.5 cite tests; AC1-AC6 cite clauses. AC6 is the load-bearing one.

## Re-binding, 2026-07-17

The spec's §Scope cited a `contracts/task/RUBRIC.md` path under `modules/skill/`. No such file exists - the rubric is at `modules/skill/task-audit/RUBRIC.md`. I pattern-matched off `modules/skill/contracts/task/STATUS-REFERENCE.md`, which IS real, and assumed RUBRIC.md sat beside it.

(The dead path is described here rather than quoted: check_doc_anchors reads audit bodies too, so writing it out verbatim re-commits the same error inside the note explaining it. Found by the checker on the first re-run, which is the check doing its job twice in five minutes.)

CI caught it (`check_doc_anchors.sh`, TASK-SKILL-119, exit 10). The check exists and runs locally; I ran build, version-sync, and six suites before committing, and not this one. The mechanism was not missing - I did not run it. That is a worse failure than the ones this task is about, and it is recorded rather than quietly amended.

This was a BODY edit, so `audited_body_sha256_prefix` genuinely moved (85257517bec2baca -> 85fadd8a036537b9) and both hashes are re-bound above. The verdict is unchanged: correcting a citation does not touch the argument the audit assessed. Had the change altered a clause, this would be a re-audit, not a re-binding.

## Cone declared, 2026-07-17

The spec reached `ready_to_implement` with NO `new_files`, `modified_files` or `service`. Its §Scope described what it touches in prose, where no tool reads it. batch-select computes conflicts from those three fields, so an undeclared cone is the EMPTY SET - which intersects nothing, so the task was provably independent of everything and joined every batch. I authored this spec today and did not declare its cone; the batch it then joined was wrong because of it.

Cone now declared from this spec's own §Scope. Frontmatter-only edit, so `audited_body_sha256_prefix` HELD (85fadd8a036537b9) and only `audited_file_sha256` is re-bound - TASK-IMP-102 built that split so lifecycle and metadata edits cannot break an audit's binding to the argument it assessed. Verdict unchanged: declaring what the spec already said touches nothing the audit weighed.

## Verdict

pass - 10/10. The task adds the check that no existing gate performs: comparing what a clause promised against what its test asserts. It was found by a reviewer asking a question the rubric never asks.
