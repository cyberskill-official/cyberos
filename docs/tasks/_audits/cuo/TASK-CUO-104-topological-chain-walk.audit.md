---
task_id: TASK-CUO-104
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

CUO topological chain walker with cycle detection + composite + sub-row audit + step failure cascade. 260 lines, 10 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 6 issues resolved (chain_status enum cardinality 5, cycle detection via Kahn's in-degree, UNIQUE(chain_id, step_order), append-only via REVOKE except status cols, step failure → skipped cascade for subsequent, TASK-SKILL-001 skill_id validation). **Score = 10/10.**

*End of TASK-CUO-104 audit.*
