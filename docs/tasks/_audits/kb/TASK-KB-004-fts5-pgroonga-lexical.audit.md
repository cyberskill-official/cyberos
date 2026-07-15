---
task_id: TASK-KB-004
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

KB FTS5 + PGroonga lexical search with VN bigram tokenisation + English stemming + tier filter + sync index update. 220 lines, 10 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 6 issues resolved (lexical_engine enum cardinality 2, PGroonga primary with FTS5 fallback on error, VN bigram via TokenBigramSplitSymbolAlphaDigit, tier filter via RLS + visibility_tier, sync index via TRIGGER, PII scrub query text SHA256). **Score = 10/10.**

*End of TASK-KB-004 audit.*
