---
fr_id: FR-ESOP-006
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

ESOP M&A acceleration with board threshold + acceleration cron + 5-business-day member notice tracking. 240 lines, 10 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 6 issues resolved (ma_event_status enum cardinality 5, board ≥3 sign threshold reuses FR-ESOP-003 logic, all-grants acceleration via cron, per-member notice with UNIQUE constraint, 5-business-day deadline tracked via cron + sev-1 alert, append-only via REVOKE). **Score = 10/10.**

*End of FR-ESOP-006 audit.*
