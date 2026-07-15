---
task_id: TASK-REW-004
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

REW statutory deductions BHXH+BHYT+BHTN+PIT per Decree 152/2020 with versioned rates + contractor exemption. 220 lines, 9 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 6 issues resolved (deduction_kind enum cardinality 6, employee-side rates (8/1.5/1% not employer 17.5/4.5/2%), contractor SI-exempt per Decree 152, versioned policy lookup via TASK-REW-002, bigint VND precision, PIT progressive bracket math). **Score = 10/10.**

*End of TASK-REW-004 audit.*
