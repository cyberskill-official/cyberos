---
fr_id: FR-HR-006
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

HR leave accrual nightly batch per Decree 145 Art. 65 with immutable ledger + idempotent + correction support. 230 lines, 11 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 6 issues resolved (UNIQUE(member_id, year_month, kind) idempotency, accrual_kind enum cardinality 4, immutable ledger via REVOKE UPDATE/DELETE, correction via new row pattern, pro-rate via FR-HR-002 contract override, rust_decimal precision). **Score = 10/10.**

*End of FR-HR-006 audit.*
