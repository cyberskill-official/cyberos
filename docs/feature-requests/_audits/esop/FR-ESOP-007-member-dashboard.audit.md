---
fr_id: FR-ESOP-007
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

ESOP Member dashboard with self-only default + CFO/CEO audited cross-member view + estimated value computation. 240 lines, 9 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 6 issues resolved (dashboard_access_kind enum cardinality 4, cross-member denied without audit_reason, sev-2 memory audit on denied, audit log append-only, IP hashed in log, no-valuation-for-year → estimated=null + sev-3). **Score = 10/10.**

*End of FR-ESOP-007 audit.*
