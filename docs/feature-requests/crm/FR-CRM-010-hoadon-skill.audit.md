---
fr_id: FR-CRM-010
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

CRM vn-vat-invoice@1 skill orchestrating FR-INV-001 invoice + FR-INV-007 hóa đơn emit on deal.stage=won. 250 lines, 11 §1 clauses, 20 ACs, 3 tests, 11 failure modes, 5 notes. 6 issues resolved (delegation to FR-INV-007 (no duplication), UNIQUE(deal_id) idempotency across re-won, silent skip non-VN tenant, trigger enum cardinality 3, no auto-cancel on stage revert (CFO must use FR-INV-008), PII scrub deal_value SHA256). **Score = 10/10.**

*End of FR-CRM-010 audit.*
