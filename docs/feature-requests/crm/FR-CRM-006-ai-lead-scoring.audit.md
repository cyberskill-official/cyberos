---
fr_id: FR-CRM-006
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

CRM AI lead scoring 0-100 + 4-tier with FR-CRM-002 signal context + immutable snapshots + per-tenant weights + nightly refresh. 260 lines, 11 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 6 issues resolved (snapshots immutable via no UPDATE grant, score CHECK constraint 0-100, tier enum cardinality 4 with threshold validation, weights versioned + per-tenant override, signal missing → null + sev-2 (no lie), PII scrub signals JSON SHA256). **Score = 10/10.**

*End of FR-CRM-006 audit.*
