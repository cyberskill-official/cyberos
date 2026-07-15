---
task_id: TASK-EMAIL-003
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.0/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 7
template: engineering-spec@1
---

EMAIL Missive-style UX with shared inbox + state machine + internal comments + Genie panel + keyboard shortcuts. 380 lines, 12 §1 clauses, 20 ACs, 4 tests, 10 failure modes, 6 notes. 7 issues resolved (internal comments NEVER in reply quote — CI hard-block test, no email notification on assignment, thread_state enum cardinality 5, snooze wake via TASK-MCP-007 cron, PII scrub comment body SHA256, append-only thread_comments via REVOKE, keyboard shortcuts disabled in inputs). **Score = 10/10.**

*End of TASK-EMAIL-003 audit.*
