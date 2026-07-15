---
task_id: TASK-INV-010
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

CUO dunning drafts with bucket→tone mapping + CFO approval queue + TASK-EMAIL-009 send + idempotent daily scan. 270 lines, 12 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 6 issues resolved (NEVER auto-send invariant, UNIQUE(tenant_id, invoice_id, tone) idempotency, legal_warning red-banner CFO escalation, append-only via REVOKE UPDATE except status+review, PII scrubbed (draft_body SHA256), aging re-classification handled on rescan). **Score = 10/10.**

*End of TASK-INV-010 audit.*
