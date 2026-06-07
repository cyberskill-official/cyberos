---
id: FR-KB-006
title: "KB BGE-rerank-v2-m3 cross-encoder — reranks top-K results from FR-KB-004 lexical + FR-KB-005 semantic to final ordering"
module: KB
priority: MUST
status: ready_to_implement
verify: T
phase: P1
milestone: P1 · slice 5
slice: 5
owner: Stephen Cheng (CDO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_frs: [FR-KB-004, FR-KB-005, FR-KB-007, FR-AI-020, FR-MEMORY-111]
depends_on: [FR-AI-020, FR-KB-005]
blocks: [FR-KB-007]

source_pages:
  - website/docs/modules/kb.html#reranker
  - https://huggingface.co/BAAI/bge-reranker-v2-m3

source_decisions:
  - DEC-1930 2026-05-17 — BGE-rerank-v2-m3 cross-encoder for fine-grained relevance scoring; output replaces lexical/semantic ranks
  - DEC-1931 2026-05-17 — Closed enum `rerank_source` = {lexical_only, semantic_only, hybrid_lexical_semantic, manual_curation}; cardinality 4
  - DEC-1932 2026-05-17 — Hybrid mode: lexical top-20 + semantic top-20 → dedup → rerank top-40 → return top-10
  - DEC-1933 2026-05-17 — Per-tenant rerank query cache (5min TTL) — same query+source → cached
  - DEC-1934 2026-05-17 — memory audit kinds: kb.rerank_executed, kb.rerank_cache_hit, kb.rerank_failed

build_envelope:
  language: rust 1.81
  service: cyberos/services/kb/
  new_files:
    - services/kb/migrations/0006_rerank_cache.sql
    - services/kb/src/rerank/mod.rs
    - services/kb/src/rerank/bge_rerank_client.rs
    - services/kb/src/rerank/hybrid_merger.rs
    - services/kb/src/handlers/rerank_routes.rs
    - services/kb/src/audit/rerank_events.rs
    - services/kb/tests/rerank_lexical_only_test.rs
    - services/kb/tests/rerank_semantic_only_test.rs
    - services/kb/tests/rerank_hybrid_test.rs
    - services/kb/tests/rerank_source_enum_cardinality_test.rs
    - services/kb/tests/rerank_cache_test.rs
    - services/kb/tests/rerank_audit_emission_test.rs

  modified_files:
    - services/kb/src/lib.rs

  allowed_tools:
    - file_read: services/{kb,ai}/**
    - file_write: services/kb/{src,tests,migrations}/**
    - bash: cd services/kb && cargo test rerank

  disallowed_tools:
    - return >10 rerank results (per DEC-1932)
    - bypass cache (per DEC-1933 — perf)

effort_hours: 4
sub_tasks:
  - "0.3h: 0006_rerank_cache.sql"
  - "0.3h: rerank/mod.rs"
  - "0.6h: bge_rerank_client.rs"
  - "0.5h: hybrid_merger.rs (dedup + interleave)"
  - "0.4h: handlers/rerank_routes.rs"
  - "0.3h: audit/rerank_events.rs"
  - "1.4h: tests — 6 test files"
  - "0.2h: docs"

risk_if_skipped: "Without reranker, lexical+semantic results stay rough — top-3 may be irrelevant. Without DEC-1932 hybrid, lose recall (lexical-only misses paraphrase, semantic-only misses exact-keyword). Without DEC-1933 cache, every query hits cross-encoder (slow + expensive)."
---

## §1 — Description (BCP-14 normative)

The KB service **MUST** ship reranker at `services/kb/src/rerank/` using BGE-rerank-v2-m3 cross-encoder over FR-KB-004/005 candidates, hybrid merge, 5min cache, 3 memory audit kinds.

1. **MUST** validate `rerank_source` against closed enum per DEC-1931.

2. **MUST** call BGE-rerank-v2-m3 at `bge_rerank_client.rs::rerank(query, candidates) → Vec<(chunk, score)>` per DEC-1930.

3. **MUST** merge hybrid per DEC-1932 at `hybrid_merger.rs::merge(lexical_results, semantic_results)`:
   - Dedup by chunk_id (semantic) or doc_id (lexical).
   - Combine top-20 from each → up to 40 candidates.
   - Rerank → return top-10.

4. **MUST** cache per DEC-1933:
   ```sql
   CREATE TABLE kb_rerank_cache (
     cache_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     query_hash CHAR(64) NOT NULL,  -- SHA256(query)
     source TEXT NOT NULL CHECK (source IN ('lexical_only','semantic_only','hybrid_lexical_semantic','manual_curation')),
     results_jsonb JSONB NOT NULL,
     cached_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     expires_at TIMESTAMPTZ NOT NULL,
     UNIQUE (tenant_id, query_hash, source)
   );
   CREATE INDEX rerank_cache_expiry_idx ON kb_rerank_cache(expires_at);
   ALTER TABLE kb_rerank_cache ENABLE ROW LEVEL SECURITY;
   CREATE POLICY rerank_cache_rls ON kb_rerank_cache
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   GRANT DELETE ON kb_rerank_cache TO cyberos_app;
   ```

5. **MUST** expose endpoint:
   ```text
   POST /v1/kb/search/rerank   body: {query, source: hybrid_lexical_semantic|lexical_only|semantic_only}
   ```

6. **MUST** emit 3 memory audit kinds per DEC-1934. PII per FR-MEMORY-111: query SHA256 in chain; results count ok.

7. **MUST** thread trace_id from query → rerank → cache → audit.

8. **MUST NOT** return >10 results per DEC-1932.

9. **MUST NOT** bypass cache per DEC-1933 (perf invariant — same query+source within 5min must hit cache).

10. **MUST** evict cache rows past `expires_at` via FR-MCP-007 nightly cron.

---

## §2 — Why this design

**Why cross-encoder (DEC-1930)?** Cross-encoders score query+candidate jointly; far more accurate than dual-encoder similarity.

**Why hybrid (DEC-1932)?** Lexical catches exact keywords; semantic catches paraphrase. Union maximises recall.

**Why 5min cache (DEC-1933)?** Cross-encoder calls are expensive (~200ms); same-query repeats common (user iterates on search).

**Why top-10 result (DEC-1932)?** UX: users scan first 10; longer lists ignored. Cross-encoder cost justifies cap.

---

## §3 — API contract

```text
POST /v1/kb/search/rerank
```

Sample request:
```json
{
  "query": "how do I emit hóa đơn",
  "source": "hybrid_lexical_semantic"
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
      "snippet": "...",
      "rerank_score": 0.94,
      "source": "hybrid_lexical_semantic"
    }
  ],
  "from_cache": false,
  "rerank_duration_ms": 180
}
```

---

## §4 — Acceptance criteria
1. **rerank_source enum cardinality 4**. 2. **BGE-rerank-v2-m3 cross-encoder**. 3. **Hybrid: lexical+semantic top-20 each, rerank top-40, return top-10**. 4. **Lexical-only mode**. 5. **Semantic-only mode**. 6. **Manual curation passthrough**. 7. **5min cache TTL**. 8. **Cache hit returns from_cache=true**. 9. **3 memory audit kinds emitted**. 10. **PII scrubbed (query SHA256)**. 11. **RLS denies cross-tenant**. 12. **Trace_id preserved**. 13. **Cache expiry cron**. 14. **UNIQUE(tenant_id, query_hash, source)**. 15. **Append-only via REVOKE except DELETE**. 16. **Rerank duration < 300ms p95**. 17. **Dedup by chunk_id + doc_id**. 18. **Empty candidates returns empty**. 19. **AI-020 service down → fallback to candidate order + sev-2 audit**. 20. **Score in 0-1 range**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn hybrid_rerank_returns_top_10() {
    let ctx = TestContext::with_50_indexed_docs().await;
    let r = ctx.rerank("test", "hybrid_lexical_semantic").await;
    assert_eq!(r.results.len(), 10);
}

#[tokio::test]
async fn cache_hit_returns_from_cache() {
    let ctx = TestContext::with_docs().await;
    let r1 = ctx.rerank("query", "hybrid_lexical_semantic").await;
    assert_eq!(r1.from_cache, false);
    let r2 = ctx.rerank("query", "hybrid_lexical_semantic").await;
    assert_eq!(r2.from_cache, true);
}

#[tokio::test]
async fn dedup_across_sources() {
    let ctx = TestContext::with_doc_matching_both().await;
    let r = ctx.rerank("term", "hybrid_lexical_semantic").await;
    let doc_ids: HashSet<_> = r.results.iter().map(|x| x.doc_id).collect();
    assert_eq!(doc_ids.len(), r.results.len());
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** FR-AI-020 (BGE-rerank-v2-m3 service), FR-KB-005.
**Downstream:** FR-KB-007 (Ask this page Q&A).
**Cross-module:** FR-KB-004 (lexical input), FR-MCP-007 (cache eviction cron), FR-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| BGE-rerank service down | catch | fallback candidate order + sev-2 | retry |
| Cache table corruption | DELETE + re-rerank | inherent | inherent |
| Query > 500 chars | validate | 400 | shorten |
| Empty candidates | inherent | [] | inherent |
| Score outside 0-1 | clamp | inherent | inherent |
| Cross-tenant cache | RLS | 0 rows | inherent |
| Expired cache served | expires_at check | re-rerank | inherent |
| Concurrent rerank same query | UNIQUE | first wins | inherent |
| Manual curation list invalid | validate | reject 400 | fix list |
| Result enrichment fail (chunk → doc) | catch | sev-3 + skip | data fix |

## §11 — Implementation notes
- §11.1 BGE-rerank model hosted via FR-AI-020 inference service.
- §11.2 Hybrid merge: round-robin interleave lexical[0], semantic[0], lexical[1], semantic[1]... then dedup.
- §11.3 Cache key: SHA256(query) + source enum; per-tenant scope.
- §11.4 memory audit body: tenant_id, source, candidate_count, returned_count; query SHA256.
- §11.5 Cache eviction cron: hourly DELETE WHERE expires_at < now().

---

*End of FR-KB-006 spec.*
