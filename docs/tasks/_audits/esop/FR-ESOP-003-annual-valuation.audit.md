---
task_id: TASK-ESOP-003
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

ESOP annual valuation with CFO propose + Board ≥3-sign threshold + immutable + correction_of pattern. 240 lines, 9 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 6 issues resolved (valuation_status enum cardinality 4, board threshold (3 of 5 default, configurable), UNIQUE(tenant, year, correction_of) for immutability + correction support, CHECK constraints (price>=0, multiplier>0), auto-commit on threshold, board member-only sign). **Score = 10/10.**

*End of TASK-ESOP-003 audit.*
