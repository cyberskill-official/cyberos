---
task_id: TASK-ESOP-005
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

ESOP GL/BL branch with CFO+CEO dual-sign + forfeiture executor + UNIQUE(termination_id, grant_id) idempotency. 240 lines, 11 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 6 issues resolved (leaver_outcome enum cardinality 5, dual-sign with separation, math invariant check (vested+forfeit+retained ≤ total), append-only via REVOKE except status cols, immutable post-commit, auto-draft from TASK-HR-009 cascade). **Score = 10/10.**

*End of TASK-ESOP-005 audit.*
