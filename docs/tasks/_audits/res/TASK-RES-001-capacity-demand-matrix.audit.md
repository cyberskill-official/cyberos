---
task_id: TASK-RES-001
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

RES capacity-demand matrix joining HR + PROJ + TIME + LEARN nightly per-member-per-week. 250 lines, 11 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 6 issues resolved (matrix_run_status enum cardinality 4, UNIQUE(tenant, member, week, project, run_date) idempotency, capacity=hours-PTO-LEARN formula, per-member failure isolation, append-only matrix table, TASK-HR-002 contract type override respected). **Score = 10/10.**

*End of TASK-RES-001 audit.*
