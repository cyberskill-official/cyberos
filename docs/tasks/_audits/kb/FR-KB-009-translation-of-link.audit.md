---
task_id: TASK-KB-009
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

KB dual-language translation_of link (vi/en) with bidirectional invariant + parity check + locale-aware reader. 210 lines, 10 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 6 issues resolved (kb_locale enum cardinality 2, bidirectional link enforced via trigger, self-reference rejected, AI parity check timeout degrades to manual sev-2, one-to-one constraint (no triangulation), CDO-only link/parity write). **Score = 10/10.**

*End of TASK-KB-009 audit.*
