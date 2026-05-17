---
fr_id: FR-KB-005
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 7
template: engineering-spec@1
---

KB BGE-M3 semantic search with chunked dense embedding (1024-dim) + pgvector ivfflat cosine + top-K=20 + version-keyed invalidation. 260 lines, 12 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 7 issues resolved (chunk_kind enum cardinality 5, semantic boundary detection no mid-sentence, version invalidation via DELETE, UNIQUE(doc, version, chunk_order), tier filter via RLS, embedding never in BRAIN chain (binary), bulk ingest via FR-MCP-007 async). **Score = 10/10.**

*End of FR-KB-005 audit.*
