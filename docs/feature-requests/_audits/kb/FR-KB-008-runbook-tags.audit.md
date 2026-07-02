---
fr_id: FR-KB-008
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

KB runbook applicability tags (provider/region/severity) with FR-OBS-007 incident match + multi-dim CHECK enforcement. 220 lines, 7 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 6 issues resolved (3 closed enums with CHECK array containment, GIN indexes filtered on is_runbook for perf, specificity ranking, global tag matches any region, append-only via REVOKE except 4 tag cols, CTO-only write). **Score = 10/10.**

*End of FR-KB-008 audit.*
