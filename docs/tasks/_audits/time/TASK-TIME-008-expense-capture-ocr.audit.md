---
task_id: TASK-TIME-008
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

Expense capture pipeline with async OCR + hóa đơn parser + Member confirm + policy validation + invoice integration. 580 lines, 16 §1 clauses, 20 ACs, 4 tests, 17 failure modes, 10 notes. 6 issues resolved (OCR retry on Textract quota, MST false-positive guard, duplicate detection, multi-language fallback, confirm-then-attach order, policy retro-application). **Score = 10/10.**

*End of TASK-TIME-008 audit.*
