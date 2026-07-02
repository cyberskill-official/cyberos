---
fr_id: FR-KB-007
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 7
template: engineering-spec@1
---

KB Ask-this-page Q&A grounded in doc + 1-hop linked + span citations + answer-or-decline gate + rate limit. 280 lines, 13 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 7 issues resolved (answer_kind enum cardinality 4, no open-world search invariant, citation required for every claim, confidence ≥0.7 gate, 50/user/day rate limit, append-only via REVOKE, linked docs respect FR-KB-003 visibility). **Score = 10/10.**

*End of FR-KB-007 audit.*
