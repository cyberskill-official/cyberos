---
fr_id: FR-REW-003
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 7
template: engineering-spec@1
---

REW P1 protection invariant with DB trigger + service guard + VN Labour Code Art. 35 demotion consent flow + dual-sign. 270 lines, 9 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 7 issues resolved (p1_change_kind enum cardinality 4, DB trigger as hard backstop, service validator for UX, demotion consent doc required (FR-DOC-001), CEO+CFO dual-sign with separation, CHECK new_p1 < old_p1 on consent, append-only consent table). **Score = 10/10.**

*End of FR-REW-003 audit.*
