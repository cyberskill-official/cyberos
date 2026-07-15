---
task_id: TASK-INV-007
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 7
template: engineering-spec@1
---

VN hóa đơn auto-emit on AM-send with Decree 123 XML + KMS signing + GDT async polling + idempotency. 280 lines, 11 §1 clauses, 22 ACs, 3 tests, 13 failure modes, 6 notes. 7 issues resolved (KMS errors don't transmit unsigned, UNIQUE(invoice_id) idempotency, non-VN silent skip, poll 24h timeout → pending+sev-1, PII scrub amounts → SHA-256, append-only via REVOKE UPDATE on key cols, resubmit endpoint CFO-only). **Score = 10/10.**

*End of TASK-INV-007 audit.*
