---
fr_id: FR-HR-005
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

HR Decree 145/152 policy constants with version-pinning + immutability + tenant override for non-statutory only. 240 lines, 10 §1 clauses, 20 ACs, 4 tests, 10 failure modes, 5 notes. 6 issues resolved (policy_kind enum cardinality 6, REVOKE UPDATE/DELETE for immutability, CHECK constraint blocking statutory override (SI/PIT), version effective_at lookup determinism for FR-REW-004 replay, source law reference required, annual seed runbook documented). **Score = 10/10.**

*End of FR-HR-005 audit.*
