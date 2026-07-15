---
task_id: TASK-RES-003
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

RES over/under-allocation flags (110/60 thresholds) with weekly Friday digest to CHRO. 200 lines, 9 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 6 issues resolved (allocation_flag enum cardinality 3, threshold boundary precision, UNIQUE(tenant_id, iso_week) digest idempotency, 0-capacity → null + sev-3, append-only via REVOKE except sent_at, contractor excluded per TASK-HR-002). **Score = 10/10.**

*End of TASK-RES-003 audit.*
