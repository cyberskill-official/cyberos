---
fr_id: FR-ESOP-001
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

ESOP SP grant schema with 4y vesting + 12mo cliff defaults + CEO+member dual-sign + immutable. 220 lines, 8 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 6 issues resolved (grant_kind enum cardinality 5, grant_status enum cardinality 5, CHECK constraints (total>0, cliff<=vest, strike>=0), immutable via REVOKE UPDATE/DELETE except 4 status cols, dual-sign required for active, cancel restricted to pre-cliff). **Score = 10/10.**

*End of FR-ESOP-001 audit.*
