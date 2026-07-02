---
fr_id: FR-CRM-005
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

CRM crm.next-action@1 skill with top-3 AI ranking + FR-CRM-002 context + per-user rate limit + 7-day expiry. 250 lines, 11 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 6 issues resolved (top-3 cardinality enforced, 100/user/day rate limit via sliding window, next_action_kind enum cardinality 7, rationale required + SHA256 in audit, 7-day expiry cron, AI invalid JSON retry with sev-2 fallback). **Score = 10/10.**

*End of FR-CRM-005 audit.*
