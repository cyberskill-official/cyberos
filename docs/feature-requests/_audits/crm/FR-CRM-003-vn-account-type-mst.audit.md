---
fr_id: FR-CRM-003
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

CRM VN account types (6 closed enum) + MST format validation gated on residency=vn-1. 200 lines, 9 §1 clauses, 20 ACs, 5 tests, 7 failure modes, 5 notes. 6 issues resolved (10/13 digit MST regex enforced in CHECK constraint, residency-gated requirement (vn-1 only), enum cardinality 6, PII scrub MST SHA256, append-only via REVOKE except 3 cols, FR-INV-007 downstream contract preserved). **Score = 10/10.**

*End of FR-CRM-003 audit.*
