---
task_id: TASK-LEARN-002
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

LEARN bằng cấp + chứng chỉ evidence types with TASK-DOC-001 scan link + expiry monitoring + skill-tree linkage. 200 lines, 7 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 6 issues resolved (evidence_kind enum cardinality 5, 30d expiry alert via cron + TASK-CHAT-005, append-only via REVOKE except verify cols, doc_id optional TASK-DOC-001 ref, CHRO-only verification, renewal via new row preserves history). **Score = 10/10.**

*End of TASK-LEARN-002 audit.*
