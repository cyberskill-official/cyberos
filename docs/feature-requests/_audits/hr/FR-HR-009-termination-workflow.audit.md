---
fr_id: FR-HR-009
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 7
template: engineering-spec@1
---

HR termination workflow with GL/BL branch + CFO+CEO dual sign + separation of duties + cascade to ESOP/AUTH/PORTAL/PROJ. 290 lines, 11 §1 clauses, 20 ACs, 4 tests, 10 failure modes, 5 notes. 7 issues resolved (termination_kind enum cardinality 6, termination_stage enum cardinality 5, dual-sign required for execute, same-person separation-of-duties enforced, transactional cascade with rollback, UNIQUE(member_id) one termination per, append-only via REVOKE except 7 cols). **Score = 10/10.**

*End of FR-HR-009 audit.*
