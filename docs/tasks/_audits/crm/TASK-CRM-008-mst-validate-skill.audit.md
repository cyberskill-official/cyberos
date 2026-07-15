---
task_id: TASK-CRM-008
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

CRM vietnam-mst-validate@1 skill with GDT TIN lookup + 30d cache + Levenshtein name match + non-blocking. 220 lines, 12 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 6 issues resolved (non-blocking on GDT down — account save succeeds, 30d cache namespaced per-tenant, Levenshtein ≤3 tolerance with normalization, result enum cardinality 5, append-only audit, PII scrub MST+names SHA256). **Score = 10/10.**

*End of TASK-CRM-008 audit.*
