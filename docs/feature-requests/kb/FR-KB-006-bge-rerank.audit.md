---
fr_id: FR-KB-006
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

KB BGE-rerank-v2-m3 cross-encoder over lexical+semantic candidates with hybrid merge + 5min cache + 10-result cap. 220 lines, 10 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 6 issues resolved (rerank_source enum cardinality 4, hybrid dedup by chunk_id+doc_id, top-10 cap enforced, AI-020 down fallback to candidate order + sev-2, append-only cache via REVOKE except DELETE, expiry cron eviction). **Score = 10/10.**

*End of FR-KB-006 audit.*
