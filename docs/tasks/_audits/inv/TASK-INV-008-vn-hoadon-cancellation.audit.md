---
task_id: TASK-INV-008
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

VN hóa đơn cancellation/replace flow with Decree 123 Art. 19 + form 04/SS-HDDT + customer agreement requirement. 250 lines, 12 §1 clauses, 20 ACs, 3 tests, 11 failure modes, 5 notes. 6 issues resolved (replacement-before-cancel ordering, append-only via no DELETE grant + replacement_hoadon_id pointer, accepted-status requires agreement enforced, form xml signed via tenant KMS, 24h window audit, pending/rejected may cancel without agreement). **Score = 10/10.**

*End of TASK-INV-008 audit.*
