---
task_id: TASK-DOC-008
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

DOC expiry alert cascade with 90/30/7-day thresholds + UNIQUE-dedup + snooze + email+chat dispatch. 220 lines, 11 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 6 issues resolved (UNIQUE(doc_id, threshold) dedup, snooze suppression respected, expired/terminated excluded from scan, alert_threshold enum cardinality 3, append-only alerts table via REVOKE, boundary day exact match (90d on scan day)). **Score = 10/10.**

*End of TASK-DOC-008 audit.*
