---
id: FR-KB-005
title: "KB BGE-M3 semantic search — BRAIN Layer 2 vector ingest + dense embedding query with chunk-level retrieval"
module: KB
priority: MUST
status: draft
verify: T
phase: P1
milestone: P1 · slice 5
slice: 5
owner: Stephen Cheng (CDO)
created: 2026-05-17
shipped: null
brain_chain_hash: null
related_frs: [FR-KB-002, FR-AI-019, FR-KB-006, FR-BRAIN-111]
depends_on: [FR-AI-019, FR-KB-001]
blocks: [FR-KB-006]

source_pages:
  - website/docs/modules/kb.html#semantic-search
  - https://huggingface.co/BAAI/bge-m3  # BGE-M3 multilingual embedding model

source_decisions:
  - DEC-1920 2026-05-17 — BGE-M3 multilingual dense embedding (1024-dim) — strong VN + English; ingested via FR-AI-019 BRAIN Layer 2
  - DEC-1921 2026-05-17 — Closed enum `chunk_kind` = {paragraph, section_heading, code_block, list_item, table_row}; cardinality 5
  - DEC-1922 2026-05-17 — Chunk size 256-512 tokens; overlap 64 tokens; semantic boundary detection avoids mid-sentence splits
  - DEC-1923 2026-05-17 — Embedding cache keyed by (doc_id, version_id, chunk_id); invalidate on new version
  - DEC-1924 2026-05-17 — Top-K=20 retrieval; results passed to FR-KB-006 reranker for final ordering
  - DEC-1925 2026-05-17 — BRAIN audit kinds: kb.semantic_ingest_started, kb.semantic_ingest_completed, kb.semantic_query_executed, kb.semantic_ingest_failed

build_envelope:
  language: rust 1.81
  service: cyberos/services/kb/
  new_files:
    - services/kb/migrations/0005_semantic_chunks.sql
    - services/kb/src/semantic/mod.rs
    - services/kb/src/semantic/chunker.rs
    - services/kb/src/semantic/bge_m3_client.rs
    - services/kb/src/semantic/vector_query.rs
    - services/kb/src/handlers/semantic_routes.rs
    - services/kb/src/audit/semantic_events.rs
    - services/kb/tests/semantic_chunker_test.rs
    - services/kb/tests/semantic_bge_m3_embed_test.rs
    - services/kb/tests/semantic_query_top_k_test.rs
    - services/kb/tests/semantic_invalidation_on_version_test.rs
    - services/kb/tests/chunk_kind_enum_cardinality_test.rs
    - services/kb/tests/semantic_audit_emission_test.rs

  modified_files:
    - services/kb/src/lib.rs

  allowed_tools:
    - file_read: services/{kb,ai}/**
    - file_write: services/kb/{src,tests,migrations}/**
    - bash: cd services/kb && cargo test semantic

  disallowed_tools:
    - skip BRAIN Layer 2 ingest (per DEC-1920)
    - mid-sentence chunk splits (per DEC-1922)

effort_hours: 6
sub_tasks:
  - "0.4h: 0005_semantic_chunks.sql"
  - "0.3h: semantic/mod.rs"
  - "0.7h: chunker.rs"
  - "0.7h: bge_m3_client.rs"
  - "0.6h: vector_query.rs (pgvector cosine)"
  - "0.4h: handlers/semantic_routes.rs"
  - "0.3h: audit/semantic_events.rs"
  - "2.0h: tests — 6 test files"
  - "0.6h: docs"

risk_if_skipped: "Without semantic search, KB only finds exact-keyword matches → 'how do I emit hóa đơn' misses doc about 'invoice generation'. Without DEC-1922 boundary detection, chunks split mid-sentence (poor embedding). Without DEC-1924 top-K, full corpus rerank too slow."
---

## §1 — Description (BCP-14 normative)

The KB service **MUST** ship BGE-M3 semantic search at `services/kb/src/semantic/` ingesting via FR-AI-019 Layer 2, chunked dense embedding, top-K retrieval, 4 BRAIN audit kinds.

1. **MUST** validate `chunk_kind` against closed enum per DEC-1921.

2. **MUST** chunk via `chunker.rs::chunk(doc_plaintext)` per DEC-1922:
   - Detect semantic boundaries (paragraphs, headings, code blocks).
   - Each chunk 256-512 tokens; overlap 64.
   - Tag with kind enum.

3. **MUST** embed via `bge_m3_client.rs::embed(text) → Vec<f32; 1024>` per DEC-1920.

4. **MUST** ingest via FR-AI-019 BRAIN Layer 2 — call AI-019 ingest API with doc context + chunk embeddings.

5. **MUST** define table at migration `0005`:
   ```sql
   CREATE EXTENSION IF NOT EXISTS vector;
   CREATE TABLE kb_semantic_chunks (
     chunk_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     doc_id UUID NOT NULL,
     version_id UUID NOT NULL,
     chunk_kind TEXT NOT NULL CHECK (chunk_kind IN ('paragraph','section_heading','code_block','list_item','table_row')),
     chunk_text TEXT NOT NULL,
     embedding VECTOR(1024) NOT NULL,
     chunk_order INT NOT NULL,
     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     UNIQUE (tenant_id, doc_id, version_id, chunk_order)
   );
   CREATE INDEX chunks_doc_idx ON kb_semantic_chunks(tenant_id, doc_id, version_id);
   CREATE INDEX chunks_embedding_idx ON kb_semantic_chunks USING ivfflat (embedding vector_cosine_ops);
   ALTER TABLE kb_semantic_chunks ENABLE ROW LEVEL SECURITY;
   CREATE POLICY chunks_rls ON kb_semantic_chunks
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON kb_semantic_chunks FROM cyberos_app;
   GRANT DELETE ON kb_semantic_chunks TO cyberos_app;  -- invalidation
   ```

6. **MUST** invalidate on new version per DEC-1923 — DELETE chunks for old version on new version commit.

7. **MUST** query at `vector_query.rs::search(tenant, query, top_k=20)` per DEC-1924:
   - Embed query via BGE-M3.
   - Cosine similarity search against chunks_embedding_idx.
   - Return top-K with chunk + doc context.
   - Apply RLS tier filter.

8. **MUST** expose endpoint:
   ```text
   POST /v1/kb/search/semantic   body: {query, top_k?: 20, filters?}
   ```

9. **MUST** emit 4 BRAIN audit kinds per DEC-1925. PII per FR-BRAIN-111: query + chunk text SHA-256 hashed; embedding never in chain (binary).

10. **MUST** thread trace_id from query → embed → search → audit.

11. **MUST NOT** skip BRAIN Layer 2 ingest per DEC-1920.

12. **MUST NOT** split chunks mid-sentence per DEC-1922.

---

## §2 — Why this design

**Why BGE-M3 (DEC-1920)?** Multi-lingual (VN+EN+more), 1024-dim balance accuracy/speed, open weights (no API lock-in).

**Why chunking 256-512 (DEC-1922)?** BGE-M3 context 8192 but embedding quality degrades at scale; smaller chunks = better recall.

**Why top-K=20 (DEC-1924)?** Reranker (FR-KB-006) needs candidates; 20 balances recall + rerank cost.

**Why version-keyed invalidation (DEC-1923)?** Doc updates change semantics; stale embeddings produce wrong results.

---

## §3 — API contract

```text
POST /v1/kb/search/semantic
```

Sample request:
```json
{
  "query": "how do I issue an invoice",
  "top_k": 20,
  "filters": {"category": "finance"}
}
```

Sample response:
```json
{
  "results": [
    {
      "chunk_id": "uuid",
      "doc_id": "uuid",
      "doc_title": "Quy trình xuất hóa đơn",
      "chunk_text": "Để xuất hóa đơn cho khách hàng VN, bạn cần...",
      "chunk_kind": "paragraph",
      "similarity": 0.87
    }
  ],
  "total": 20
}
```

---

## §4 — Acceptance criteria
1. **chunk_kind enum cardinality 5**. 2. **BGE-M3 embedding 1024-dim**. 3. **Chunks 256-512 tokens**. 4. **Semantic boundary detection (no mid-sentence)**. 5. **Ingest via FR-AI-019**. 6. **pgvector ivfflat index**. 7. **Cosine similarity query**. 8. **Top-K=20 default**. 9. **Tier filter applied (RLS)**. 10. **Invalidation on new version (DELETE)**. 11. **4 BRAIN audit kinds emitted**. 12. **PII scrubbed (query+chunk SHA256; embedding never in chain)**. 13. **RLS denies cross-tenant**. 14. **Trace_id preserved**. 15. **UNIQUE(doc, version, chunk_order)**. 16. **Append-only via REVOKE except DELETE**. 17. **Empty index returns empty array**. 18. **Query embedding cached 5min**. 19. **Bulk ingest async via FR-MCP-007 task**. 20. **Multilingual query (VN+EN mixed) supported**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn semantic_finds_paraphrase() {
    let ctx = TestContext::with_doc("invoice issuance process").await;
    let r = ctx.semantic_search("how do I bill a customer").await;
    assert!(r.results.iter().any(|c| c.doc_id == ctx.doc_id));
}

#[tokio::test]
async fn invalidation_on_new_version() {
    let ctx = TestContext::with_indexed_doc().await;
    let original_chunks = ctx.fetch_chunks(ctx.doc_id).await;
    ctx.create_new_version(ctx.doc_id, "new content").await;
    ctx.run_ingest().await;
    let chunks = ctx.fetch_chunks(ctx.doc_id).await;
    let old_ones = chunks.iter().filter(|c| c.version_id == ctx.original_version).count();
    assert_eq!(old_ones, 0);
}

#[tokio::test]
async fn top_k_returned() {
    let ctx = TestContext::with_50_indexed_docs().await;
    let r = ctx.semantic_search("test").await;
    assert_eq!(r.results.len(), 20);
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** FR-AI-019 (BRAIN Layer 2), FR-KB-001.
**Downstream:** FR-KB-006 (rerank).
**Cross-module:** FR-KB-002 (plaintext source), FR-MCP-007 (async ingest), FR-BRAIN-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| BGE-M3 service down | retry | sev-2; fall back to lexical only | retry |
| Embedding dimension mismatch | validate | reject; sev-1 | bug fix |
| pgvector extension missing | catch | sev-1 | install |
| Ingest mid-doc fail | rollback | sev-2 | re-ingest |
| Cross-tenant query | RLS | 0 rows | inherent |
| Large doc (>100k tokens) | chunking handles | inherent | inherent |
| ivfflat index build slow | async | inherent | tune lists param |
| Query embedding cache miss | re-embed | inherent | inherent |
| Multilingual mixed query | BGE-M3 native | inherent | inherent |
| Index drift (stale chunks) | invalidation cron | sev-3 | re-ingest |

## §11 — Implementation notes
- §11.1 BGE-M3 model hosted via FR-AI-019 inference endpoint; service-side caching of embeddings.
- §11.2 Chunker uses tree-sitter for code-aware splits; falls back to paragraph for prose.
- §11.3 ivfflat: lists=floor(sqrt(row_count)); rebuilt monthly via maintenance cron.
- §11.4 BRAIN audit body: doc_id, version_id, chunk_count; query+text SHA256.
- §11.5 Bulk ingest: large doc → chunked → batched embed calls (10 chunks per batch).

---

*End of FR-KB-005 spec.*
