---
task_id: TASK-ESOP-002
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

ESOP monthly vesting batch with deterministic cliff-aware calc + immutable accrual rows + auto-fully_vested transition. 240 lines, 11 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 6 issues resolved (accrual_status enum cardinality 4, UNIQUE(tenant, grant, year_month) idempotency, deterministic pure-function calc, immutable via REVOKE, auto-transition to fully_vested, per-grant failure isolation). **Score = 10/10.**

*End of TASK-ESOP-002 audit.*
