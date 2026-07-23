---
id: TASK-MEMORY-108
title: "memory search — vector + graph + full-text in parallel + RRF fusion + BGE-rerank + RLS + ACL + chain_anchor verify + 250ms p95"
eu_ai_act_risk_class: not_ai
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-15T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: memory
priority: p0
status: done
verify: T
phase: P1
milestone: P1 · slice 2
slice: 2
owner: Stephen Cheng (CDO)
created: 2026-05-15
shipped: 2026-05-23
memory_chain_hash: null
related_tasks: [TASK-MEMORY-101, TASK-MEMORY-106, TASK-AI-019, TASK-AI-020, TASK-AUTH-003, TASK-AUTH-004]
depends_on: [TASK-MEMORY-101, TASK-MEMORY-107]
# placeholder — "Ask this page" KB Q&A task, not yet specified (downstream consumer)
blocks: [TASK-KB-007]

source_pages:
  - website/docs/modules/memory.html#search
source_decisions:
  - DEC-195 (3-way parallel: vector + graph + full-text; RRF fusion; rerank top-50)
  - DEC-196 (chain_anchor verify on every result; sev-1 on mismatch)
  - DEC-197 (graceful degrade — BGE down → full-text only; surface in explain)
  - DEC-198 (Vietnamese-aware tokenisation via PGroonga; default mecab + custom dictionary)

language: rust 1.81
service: cyberos/services/memory/
new_files:
  - services/memory/src/search/mod.rs
  - services/memory/src/search/vector.rs
  - services/memory/src/search/graph.rs
  - services/memory/src/search/fulltext.rs
  - services/memory/src/search/rrf.rs
  - services/memory/src/search/rerank.rs
  - services/memory/src/search/acl_filter.rs
  - services/memory/src/search/chain_anchor_verify.rs
  - services/memory/src/search/explain.rs
  - services/memory/migrations/0003_pgroonga.sql
  - services/memory/tests/ingest_test.rs
  - services/memory/tests/chain_anchor_test.rs
  - services/memory/tests/chain_anchor_test.rs
  - services/memory/tests/chain_anchor_test.rs
  - services/memory/tests/chain_anchor_test.rs
  - services/memory/tests/chain_anchor_test.rs
modified_files:
  # add search route
  - services/memory/src/handlers/mod.rs
allowed_tools:
  - file_read: services/memory/**
  - file_write: services/memory/{src,tests,migrations}/**
  - bash: cd services/memory && cargo test search
disallowed_tools:
  #4)
  - return search results without RLS scoping (per §1
  #5)
  - skip ACL filter (per §1
  #11)
  - skip chain_anchor verification (per §1
  #8 — graceful degrade)
  - hard-fail on BGE down (per §1

effort_hours: 12
subtasks:
  - "0.5h: search/mod.rs API + types"
  - "1.0h: vector.rs — BGE-M3 embed + HNSW cosine search"
  - "1.0h: graph.rs — Cypher query on AGE for related entities"
  - "1.0h: fulltext.rs — PGroonga search with VN tokenisation"
  - "0.5h: 0003_pgroonga.sql migration (PGroonga extension + index)"
  - "0.5h: rrf.rs — Reciprocal Rank Fusion implementation"
  - "0.5h: rerank.rs — BGE-rerank-v2-m3 invocation"
  - "0.5h: acl_filter.rs — ACL respected at result-time"
  - "0.5h: chain_anchor_verify.rs — Layer 1 verification per result"
  - "0.5h: explain.rs — per-backend score breakdown"
  - "0.5h: Graceful BGE-down fallback to fulltext only"
  - "0.5h: AGE query timeout → skip graph (continue with vec + fulltext)"
  - "0.5h: handlers/search.rs — HTTP route + parameter parsing"
  - "0.5h: OTel metrics + spans"
  - "1.5h: Tests — happy + RLS + ACL + p95 + fallback + chain_anchor + VN-query + explain"
  - "1.0h: Tests — graph contributions + RRF stability + rerank improves precision"
  - "0.5h: Test fixture (1M chunks for p95 benchmark)"
risk_if_skipped: "memory search is the read API for everything: KB retrieval (TASK-KB-007), CUO ambient context, OBS auto-triage. Without it, all those modules fail. Without RLS+ACL, search leaks across tenants/actors. Without chain_anchor verification, Layer 1 tampering goes undetected at query time. Without graceful degrade, BGE outage takes down search entirely."
---

## §1 — Description (BCP-14 normative)

The memory service **MUST** expose `GET /v1/memory/search` returning ranked memories. Each request:

1. **MUST** accept `?q=<query>&kind=<filter>&ts_since=<ns>&ts_until=<ns>&limit=<int>&explain=<bool>`. Limit default 10, max 100.
2. **MUST** dispatch to 3 backends in parallel via `tokio::try_join!`:
- **Vector**: BGE-M3 embed query (TASK-AI-019) → cosine similarity search in `layer2_memories` HNSW index.
- **Graph**: Cypher query on Apache AGE finding semantically-related entities (within 2 hops).
- **Full-text**: PGroonga search with Vietnamese-aware tokenisation (mecab + custom dictionary).
3. **MUST** combine via Reciprocal Rank Fusion (RRF) with k=60 (standard parameter); take top 50 candidates; rerank via TASK-AI-020 BGE-reranker-v2-m3 → final top-N.
4. **MUST** apply RLS at the DB layer — caller's tenant_id (from JWT) scopes `layer2_memories` queries via `current_setting('app.tenant_id')`. RLS USING clause filters; cross-tenant queries return 0 rows.
5. **MUST** consult `meta.acl[]` per memory at result-filtering time — if non-empty AND caller's actor_id NOT in list, exclude from results. ACL applied AFTER RLS (defense-in-depth: RLS at DB, ACL at API).
6. **MUST** return `[{id, kind, path, ts_ns, snippet, score, related_count}]`. Snippet is a 200-char excerpt around the match (full-text) OR a synthesized summary (vector); `related_count` = entities linked via graph.
7. **MUST** complete p95 ≤ 250ms on 1M-chunk tenant fixture (NFR-PERF-01). Above 250ms, OBS sev-3 alarm.
8. **MUST** fall back gracefully on backend failures:
- BGE sidecar down → vector backend skipped; results from graph + fulltext only.
- AGE query timeout (5s) → graph skipped.
- PGroonga error → fulltext skipped. When all 3 fail, return `503 SERVICE_UNAVAILABLE`. When at least one succeeds, return results + surface degradation in `?explain=true` response.
9. **SHOULD** support `?explain=true` returning per-backend scores + RRF computation + rerank input/output for debugging.
10. **MUST** verify `chain_anchor` for each returned result against Layer 1 (per TASK-MEMORY-101 §1 #4). Mismatch → drop the result + emit sev-1 OBS event `memory_search_chain_anchor_mismatch{tenant_id, seq}`.
11. **MUST** authenticate via TASK-AUTH-004 JWT; extract tenant_id + actor_id from claims.
12. **MUST** support empty results — return `[]` with HTTP 200; NOT 404 (search semantics: "no matches" is normal, not an error).
13. **MUST** support multi-language (Vietnamese + English) queries. PGroonga's VN tokenisation handles Vietnamese; BGE-M3 multilingual embedding handles cross-language.
14. **SHOULD** emit OTel metrics:
- `memory_search_requests_total{tenant_id, outcome}` (counter; outcome ∈ ok | partial_degrade | full_failure | empty).
- `memory_search_latency_ms{tenant_id}` (histogram; SLO p95 < 250ms).
- `memory_search_backend_latency_ms{backend}` (histogram per vector/graph/fulltext).
- `memory_search_chain_anchor_mismatch_total{tenant_id}` (counter; sev-1 alarm).
- `memory_search_acl_filtered_total{tenant_id}` (counter; how many results filtered).
- `memory_search_rerank_improvement` (histogram; rerank-position-delta tracking).

---

## §2 — Why this design (rationale for humans)

**Why 3-way parallel (DEC-195)?** Each backend has different recall/precision profiles:
- Vector: semantically-similar but maybe topically-different ("legal advice" finds "regulatory guidance").
- Graph: explicit relationships ("Alice's projects" finds projects she owns).
- Full-text: lexically-similar ("Decree 13" finds documents containing those words).

Combining via RRF preserves diversity; reranking polishes precision.

**Why RRF + rerank (DEC-195)?** RRF is a robust ensemble method (no parameter tuning needed); BGE-reranker is the precision-polish on the top-50 candidates. Two-stage: ensemble for recall, rerank for precision.

**Why chain_anchor verify (DEC-196)?** Layer 2 is derived; if Layer 1 is tampered AND Layer 2 hasn't re-ingested, Layer 2 contains stale (pre-tamper) data. The chain_anchor stored in Layer 2 is `SHA-256(Layer 1 row at ingest time)`. At read, recompute from current Layer 1; mismatch → tampering detected.

**Why graceful BGE degrade (DEC-197)?** BGE outage shouldn't take down search. Falling back to fulltext (less semantic, still useful) preserves availability. The `explain` field surfaces the degradation so users know.

**Why Vietnamese tokenisation (DEC-198)?** Default PGroonga uses BLI tokenisation (good for English); Vietnamese has different word-segmentation rules. PGroonga's mecab support + custom Vietnamese dictionary gives proper VN segmentation.

**Why 250ms p95 budget (§1 #7)?** Search is interactive (user types → results appear). 250ms is at the edge of human perception; longer feels sluggish. Vector + graph + fulltext in parallel + rerank fits in 250ms on 1M-chunk Postgres + GPU rerank.

**Why ACL at result-time, not query-time (§1 #5)?** ACL is per-memory metadata; computing "give me only memories where I'm in ACL" at SQL-time would require GIN index on `acl` array. At search-time scale (top-50 candidates), in-memory filter is fast (~1ms). Trade-off: more rows fetched then filtered; manageable at scale.

**Why empty results = 200 not 404 (§1 #12)?** Search semantics: "no matches" is a valid answer, not an error. 404 implies "this URL doesn't exist"; 200 with `[]` says "search ran; no matches." Standard search-API convention.

**Why explain support (§1 #9)?** Search relevance is hard to debug. `?explain=true` returns per-backend scores + RRF computation + rerank input/output — operators can answer "why is X ranked higher than Y?"

**Why chain_anchor mismatch is sev-1 (§1 #10)?** Layer 1 tampering is the highest-stakes failure: someone modified the chain. Even a single mismatch warrants immediate investigation.

---

## §3 — API contract

```rust
// services/memory/src/search/mod.rs
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(serde::Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub kind: Option<String>,
    pub ts_since: Option<i64>,
    pub ts_until: Option<i64>,
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default)]
    pub explain: bool,
}
fn default_limit() -> usize { 10 }

#[derive(serde::Serialize)]
pub struct SearchResults {
    pub items: Vec<SearchResult>,
    pub explain: Option<ExplainPayload>,
    pub degraded_backends: Vec<String>,
}

#[derive(serde::Serialize)]
pub struct SearchResult {
    pub id: Uuid,
    pub kind: String,
    pub path: String,
    pub ts_ns: i64,
    pub snippet: String,
    pub score: f32,
    pub related_count: u32,
}

#[derive(serde::Serialize)]
pub struct ExplainPayload {
    pub backend_scores: HashMap<String, Vec<(Uuid, f32)>>,
    pub rrf_input: Vec<Uuid>,
    pub rerank_pre: Vec<Uuid>,
    pub rerank_post: Vec<Uuid>,
    pub timings_ms: HashMap<String, u32>,
}

#[derive(thiserror::Error, Debug)]
pub enum SearchError {
    #[error("auth failed")]
    AuthFailed,
    #[error("all backends failed")]
    AllBackendsFailed,
    #[error("limit exceeds max (100)")]
    LimitTooLarge,
    #[error("db: {0}")]
    Db(#[from] sqlx::Error),
}

pub async fn search(query: SearchQuery, tenant_id: Uuid, actor_id: &str, pool: &PgPool, bge_url: &str, rerank_url: &str)
    -> Result<SearchResults, SearchError>
{
    if query.limit > 100 { return Err(SearchError::LimitTooLarge); }

    let mut explain = if query.explain { Some(ExplainPayload::default()) } else { None };
    let mut degraded = vec![];

    // §1 #2: 3 backends in parallel; collect Result<Vec<_>, _> so failures don't poison the join
    let (vec_res, graph_res, fts_res) = tokio::join!(
        vector::search(&query, tenant_id, pool, bge_url),
        graph::search(&query, tenant_id, pool),
        fulltext::search(&query, tenant_id, pool),
    );

    let mut backends: Vec<Vec<(Uuid, f32)>> = vec![];
    match vec_res {
        Ok(v) => backends.push(v),
        Err(e) => { tracing::warn!(error = %e, "vector search degraded"); degraded.push("vector".into()); }
    }
    match graph_res {
        Ok(v) => backends.push(v),
        Err(e) => { tracing::warn!(error = %e, "graph search degraded"); degraded.push("graph".into()); }
    }
    match fts_res {
        Ok(v) => backends.push(v),
        Err(e) => { tracing::warn!(error = %e, "fulltext search degraded"); degraded.push("fulltext".into()); }
    }

    if backends.is_empty() { return Err(SearchError::AllBackendsFailed); }

    let fused = rrf::reciprocal_rank_fusion(&backends, 60);
    let candidates: Vec<Uuid> = fused.into_iter().map(|(id, _)| id).take(50).collect();
    let reranked = rerank::with_bge(&query.q, &candidates, rerank_url, pool, tenant_id).await
        .unwrap_or_else(|_| {
            degraded.push("rerank".into());
            candidates.iter().enumerate().map(|(i, id)| (*id, 1.0 / (i + 1) as f32)).collect()
        });

    let mut items = vec![];
    for (id, score) in reranked.iter().take(query.limit * 2) {   // overfetch for ACL filtering
        let row = layer2_memories::fetch(pool, tenant_id, *id).await?;
        if !chain_anchor_verify::matches_layer1(&row, pool).await? {
            metrics::chain_anchor_mismatch(tenant_id, row.seq);
            tracing::error!(seq = row.seq, "chain_anchor mismatch; sev-1");
            continue;
        }
        if !acl_filter::passes(&row, actor_id) {
            metrics::acl_filtered(tenant_id);
            continue;
        }
        items.push(SearchResult {
            id: row.id, kind: row.kind, path: row.path,
            ts_ns: row.ts_ns, snippet: build_snippet(&row, &query.q),
            score: *score, related_count: graph::related_count(&row, pool).await.unwrap_or(0),
        });
        if items.len() >= query.limit { break; }
    }

    Ok(SearchResults { items, explain, degraded_backends: degraded })
}
```

```rust
// services/memory/src/search/rrf.rs
pub fn reciprocal_rank_fusion(backend_results: &[Vec<(Uuid, f32)>], k: usize) -> Vec<(Uuid, f32)> {
    let mut scores: HashMap<Uuid, f32> = HashMap::new();
    for backend in backend_results {
        for (rank, (id, _)) in backend.iter().enumerate() {
            *scores.entry(*id).or_insert(0.0) += 1.0 / (k as f32 + rank as f32 + 1.0);
        }
    }
    let mut sorted: Vec<_> = scores.into_iter().collect();
    sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    sorted
}
```

```rust
// services/memory/src/search/chain_anchor_verify.rs
pub async fn matches_layer1(row: &Layer2Row, pool: &PgPool) -> Result<bool, SearchError> {
    let layer1_canonical = layer1::fetch_canonical_bytes(pool, row.tenant_id, row.seq).await?;
    let recomputed = sha256(&layer1_canonical);
    Ok(recomputed == row.chain_anchor[..])
}
```

```sql
-- services/memory/migrations/0003_pgroonga.sql
CREATE EXTENSION IF NOT EXISTS pgroonga;

ALTER TABLE layer2_memories ADD COLUMN body_text TEXT;   -- denormalised for full-text
CREATE INDEX layer2_memories_pgroonga ON layer2_memories
    USING pgroonga (body_text)
    WITH (tokenizer = 'TokenMecab');   -- VN-aware tokenisation
```

---

## §4 — Acceptance criteria

1. Vector + graph + full-text all contribute to results.
2. RRF fusion produces stable ordering — same query twice returns same ordering.
3. Rerank improves precision — known-relevance fixture has relevant docs in top-3 (cosine ≥ 0.5).
4. p95 < 250ms on 1M chunks.
5. Cross-tenant: caller in tenant A never sees B's results (RLS enforces).
6. ACL filter: memories with `acl: [@alice]` invisible to @bob; metric `memory_search_acl_filtered_total` increments.
7. BGE sidecar down → fallback to graph + fulltext; surfaced in `explain.degraded_backends`.
8. AGE timeout → graph skipped; still returns from vec + fulltext.
9. PGroonga error → fulltext skipped.
10. All 3 backends fail → 503 with `all_backends_failed`.
11. Vietnamese query "Nghị định 13" returns Vietnamese-tagged docs (mecab tokenisation).
12. Empty results → 200 with `[]`.
13. Limit > 100 → 400 with `limit_too_large`.
14. `?explain=true` returns per-backend scores + RRF + rerank input/output.
15. Chain anchor mismatch → drop result + sev-1 metric.
16. JWT missing → 401.
17. Snippet returned per result (200 chars around match).
18. `related_count` reflects graph linkage.

---

## §5 — Verification

```rust
#[tokio::test]
async fn three_backends_contribute_to_results() {
    let (pool, tenant) = test_setup().await;
    test_helper::insert_memories(&pool, tenant, vec![
        memory("decisions/decree-13.md", "Decree 13 PDPL"),
        memory("decisions/related.md", "regulatory guidance Vietnam"),
    ]).await;

    let result = search(SearchQuery { q: "Decree 13".into(), explain: true, ..default_query() },
                        tenant, "@alice", &pool, &bge_url(), &rerank_url()).await.unwrap();
    let explain = result.explain.unwrap();
    assert!(!explain.backend_scores["vector"].is_empty());
    assert!(!explain.backend_scores["fulltext"].is_empty());
}

#[tokio::test]
async fn cross_tenant_rls_blocks() {
    let (pool, t_a) = test_setup().await;
    let t_b = test_helper::create_tenant().await;
    test_helper::insert_memory(&pool, t_a, "secret", "secret data").await;

    let result = search(default_query("secret"), t_b, "@bob", &pool, &bge_url(), &rerank_url()).await.unwrap();
    assert!(result.items.is_empty(), "tenant B saw tenant A's data");
}

#[tokio::test]
async fn acl_excludes_unauthorised_actor() {
    let (pool, tenant) = test_setup().await;
    test_helper::insert_memory_with_acl(&pool, tenant, "private-x", "secret", &["@alice"]).await;
    let result_alice = search(default_query("private-x"), tenant, "@alice", &pool, &bge_url(), &rerank_url()).await.unwrap();
    assert_eq!(result_alice.items.len(), 1);
    let result_bob = search(default_query("private-x"), tenant, "@bob", &pool, &bge_url(), &rerank_url()).await.unwrap();
    assert_eq!(result_bob.items.len(), 0);
    let metric: u64 = otel_test_helper::counter_value("memory_search_acl_filtered_total", &[("tenant_id", &tenant.to_string())]);
    assert!(metric > 0);
}

#[tokio::test]
async fn bge_down_falls_back_to_fulltext() {
    let (pool, tenant) = test_setup().await;
    test_helper::insert_memory(&pool, tenant, "test", "Decree 13").await;
    bge_test_helper::stop();
    let result = search(SearchQuery { q: "Decree 13".into(), explain: true, ..default_query() },
                        tenant, "@a", &pool, &bge_url(), &rerank_url()).await.unwrap();
    assert!(!result.items.is_empty());
    assert!(result.degraded_backends.contains(&"vector".to_string()));
}

#[tokio::test]
async fn all_backends_fail_503() {
    bge_test_helper::stop();
    age_test_helper::stop();
    pgroonga_test_helper::stop();
    let err = search(default_query("anything"), test_tenant().await, "@a", &test_pool().await, &bge_url(), &rerank_url()).await.expect_err("expected AllBackendsFailed");
    assert!(matches!(err, SearchError::AllBackendsFailed));
}

#[tokio::test]
async fn vietnamese_query_returns_vn_docs() {
    let (pool, tenant) = test_setup().await;
    test_helper::insert_memory(&pool, tenant, "nd13", "Nghị định 13/2023 về bảo vệ dữ liệu cá nhân").await;
    let result = search(default_query("Nghị định 13"), tenant, "@a", &pool, &bge_url(), &rerank_url()).await.unwrap();
    assert_eq!(result.items.len(), 1);
}

#[tokio::test]
async fn chain_anchor_mismatch_drops_result_and_sev1() {
    let (pool, tenant) = test_setup().await;
    let id = test_helper::insert_memory(&pool, tenant, "x", "body").await;
    test_helper::corrupt_layer1(&pool, tenant, id).await;

    let result = search(default_query("body"), tenant, "@a", &pool, &bge_url(), &rerank_url()).await.unwrap();
    assert_eq!(result.items.len(), 0);
    let metric: u64 = otel_test_helper::counter_value("memory_search_chain_anchor_mismatch_total", &[("tenant_id", &tenant.to_string())]);
    assert!(metric > 0);
}

#[tokio::test]
#[ignore = "long-running benchmark"]
async fn p95_under_250ms_on_1m_chunks() {
    let (pool, tenant) = setup_with_1m_chunks().await;
    let mut samples = vec![];
    for _ in 0..200 {
        let t0 = std::time::Instant::now();
        let _ = search(default_query("test query"), tenant, "@a", &pool, &bge_url(), &rerank_url()).await.unwrap();
        samples.push(t0.elapsed().as_millis() as u64);
    }
    samples.sort();
    let p95 = samples[(samples.len() as f64 * 0.95) as usize];
    assert!(p95 < 250, "p95 {p95}ms exceeds 250ms");
}

#[tokio::test]
async fn explain_returns_full_breakdown() {
    let result = search(SearchQuery { q: "test".into(), explain: true, ..default_query() }, test_tenant().await, "@a", &test_pool().await, &bge_url(), &rerank_url()).await.unwrap();
    assert!(result.explain.is_some());
    let e = result.explain.unwrap();
    assert!(!e.backend_scores.is_empty());
    assert!(!e.timings_ms.is_empty());
}

#[tokio::test]
async fn limit_over_100_returns_400() {
    let err = search(SearchQuery { limit: 101, ..default_query() }, test_tenant().await, "@a", &test_pool().await, &bge_url(), &rerank_url()).await.expect_err("expected LimitTooLarge");
    assert!(matches!(err, SearchError::LimitTooLarge));
}
```

---

## §6 — Implementation skeleton

See §3.

---

## §7 — Dependencies

- **TASK-MEMORY-101** — Layer 2 + chain_anchor.
- **TASK-AI-019** — BGE-M3 sidecar.
- **TASK-AI-020** — BGE-rerank-v2-m3.
- **TASK-AUTH-003** — RLS enforcement.
- **TASK-AUTH-004** — JWT.
- **TASK-MEMORY-106** — sync_class informs ACL filter (related).
- Crates: `axum`, `sqlx`, `tokio`, `serde`, `reqwest`, `proptest@1` (test).
- Postgres 16 + pgvector + AGE + PGroonga (with mecab for Vietnamese).

---

## §8 — Example payloads

### Search request

```http
GET /v1/memory/search?q=Decree%2013&kind=decisions&limit=10&explain=true HTTP/1.1
Authorization: Bearer <jwt>
```

### Response

```json
{
  "items": [
    {
      "id": "550e...", "kind": "decisions",
      "path": "memories/decisions/decree-13-pdpl.md",
      "ts_ns": 1747526400000000000,
      "snippet": "...Decree 13/2023 establishes Vietnam's personal data protection framework...",
      "score": 0.94,
      "related_count": 5
    }
  ],
  "explain": {
    "backend_scores": {
      "vector": [["550e...", 0.85], ["..."]],
      "graph":  [["550e...", 1.0]],
      "fulltext": [["550e...", 0.9]]
    },
    "rrf_input": ["550e...", "..."],
    "rerank_pre": ["550e...", "..."],
    "rerank_post": ["550e...", "..."],
    "timings_ms": { "vector": 45, "graph": 30, "fulltext": 60, "rrf": 1, "rerank": 80, "acl_filter": 5, "chain_anchor_verify": 3 }
  },
  "degraded_backends": []
}
```

### Degraded response (BGE down)

```json
{
  "items": [...],
  "degraded_backends": ["vector"]
}
```

### Chain-anchor mismatch sev-1

```text
ERROR seq=12345 stored_anchor=abc... recomputed=def...
      memory_search_chain_anchor_mismatch; result dropped
sev-1 memory_search_chain_anchor_mismatch_total{tenant_id=...} incremented
```

---

## §9 — Open questions

All resolved. Deferred:
- Hybrid sparse+dense retrieval (BM25 + vector) — slice 4+.
- Per-user search-history personalisation — slice 5+.
- Query suggestions (autocomplete) — slice 4+.
- Multi-modal search (images + audio) — slice 5+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| BGE down | reqwest error | Vector backend degraded; graph + fulltext continue | Self-heals when BGE up |
| AGE query timeout (5s) | tokio timeout | Skip graph; use vec + fulltext | Self-heals |
| PGroonga error | sqlx error | Skip fulltext; use vec + graph | Self-heals |
| All 3 backends fail | empty backends list | 503 | Operator investigates |
| Cross-tenant attempt | RLS blocks at DB | 0 rows | By design |
| Empty results | normal | 200 with `[]` | By design |
| Chain anchor mismatch | per-result verify | Drop result + sev-1 | Investigate Layer 1 corruption |
| ACL filter excludes all results | result-time filter | 200 with `[]` | By design |
| Vietnamese query no mecab match | fallback to BLI tokenisation | Lower precision but works | Update mecab dict |
| Limit > 100 | param check | 400 | Caller fixes limit |
| JWT missing | auth check | 401 | Caller obtains JWT |
| Slow query (> 250ms p95) | OTel histogram | sev-3 | Investigate hot path |
| Rerank fails | catch | Use unranked top-50 | Self-heals |
| Snippet build fails | catch | Empty snippet | Self-heals |
| Concurrent search same query | tokio handles | Each independent | By design |
| Limit 0 | param check | 400 | Caller fixes |
| ts_since > ts_until | param check | 400 | Caller fixes |
| Empty query string | accept; returns broad results | 200 | By design |
| Backend latency outliers | histogram | Sev-3 alarm if backend p99 > budget | Operator investigates |
| Index missing (pgvector or pgroonga) | startup check | Refuse to start | Operator runs migration |

---

## §11 — Notes

- 3-way parallel search exploits backend complementarity — vector/graph/fulltext catch different relevance signals.
- RRF (k=60 standard) is robust to backend score-scale differences; doesn't need per-backend tuning.
- BGE-reranker-v2-m3 polishes top-50 with cross-encoder precision (TASK-AI-020).
- Chain_anchor verify per result is the read-time tamper detection — slow but security-load-bearing.
- ACL applied at result-time (in-memory) avoids GIN index complexity; performant for top-50.
- Graceful degrade preserves availability during backend outages; surfaced in `explain.degraded_backends`.
- Vietnamese tokenisation via PGroonga + mecab + custom dictionary handles VN word segmentation correctly.
- 250ms p95 budget covers vector (~50ms) + graph (~50ms) + fulltext (~50ms parallel) + rerank (~80ms) + chain_anchor verify (~20ms) + ACL filter (~5ms).
- Empty query returns broad results (latest memories within filters); useful as "browse" mode.
- Sev-1 chain_anchor mismatch is the highest-value security signal — Layer 1 tampering caught at query time.

---

*End of TASK-MEMORY-108. Status: done (implemented 2026-05-23).*

## As built (2026-07-02)

Shipped as services/memory/src/search.rs (single module), not the search/{vector,graph,...} tree; AGE-era graph search is the relational l2_edge path.
