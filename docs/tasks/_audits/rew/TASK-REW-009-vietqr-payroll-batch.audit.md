---
task_id: TASK-REW-009
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

REW VietQR payroll batch with CFO manual confirm + TASK-INV-005 reconciliation + per-member ack tracking. 240 lines, 10 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 6 issues resolved (batch_status enum cardinality 6, NEVER auto-submit to bank, UNIQUE(payroll_run_id) idempotency, per-member ack rows for partial failure tracking, append-only via REVOKE except status cols, memo template per TASK-CRM-009 for reconciliation). **Score = 10/10.**

*End of TASK-REW-009 audit.*
