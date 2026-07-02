---
fr_id: FR-CRM-007
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

CRM win/loss CUO draft auto-created at deal close, CDO-reviewed, memory-persisted as searchable lesson memories. 250 lines, 10 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 6 issues resolved (NEVER auto-persist to memory, wl_outcome enum cardinality 6, UNIQUE(deal_id) idempotency, append-only via REVOKE except review/path cols, AI failure → status=failed + retry, FR-MEMORY-108 searchable kind=lessons). **Score = 10/10.**

*End of FR-CRM-007 audit.*
