---
audit_template_version: "audit_rubric@2.0"
audited_file: "docs/tasks/improvement/TASK-IMP-102-audit-binding-normative-half/spec.md"
audited_file_sha256_prefix: "d48d4a2b814b6f23"
audited_body_sha256_prefix: "5c530084993c87d5"
rubric_version: "audit_rubric@2.0"
skill_id: "task-audit"
skill_version: "1.0.0"
last_audit_at: "2026-07-17T11:30:00Z"
overall_status: "pass"
iterations: 1
issue_counts: { total: 1, open: 0, needs_human: 0, fixed: 0, wontfix: 1 }
machine_floor: "task-lint.mjs run FIRST (clean, exit 0)"
trace_id: "cowork-cyberos-improvement-batch5-2026-07-17"
---

# TASK-IMP-102 spec audit - audit_rubric@2.0 (machine floor + judgment)

Machine floor: task-lint clean on first pass.

Judgment: this audit is the first to carry `audited_body_sha256_prefix` - the field the task itself defines, computed over the normative half of the spec it audits. That is deliberate: the convention's first witness is the task that introduces it, so the corpus contains a verifiable binding from the moment the rule exists. The whole-file prefix is retained as provenance of the exact bytes read, and will stop matching the moment ship-tasks flips `status` - which is precisely the fact this task documents rather than hides.

Alternatives are real and distinctly rejected (post-flip hashing still breaks on the second transition; relocating status inverts STATUS-REFERENCE §1's priority; corpus rewriting falsifies history). Metrics carry baseline (100 percent unverifiable) and a suite-asserted target. Legacy readability is a stated guardrail, not an afterthought.

ISSUE ISS-001 (QA-004, wontfix-info): AC 4 is a recorded-grep prose contract (accepted pattern, TASK-IMP-090 AC 1).

SUMMARY verdict: pass issues_open: 0 issues_human: 0 next_action: ship

## §gate-log

Populated during implementation (ship-tasks testing phase).
