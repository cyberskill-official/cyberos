//! FR-MEMORY-108 — `POST /v1/memory/search` with hybrid lexical + vector recall.
//!
//! Two retrievers fused via Reciprocal Rank Fusion (RRF):
//!   * **Lexical** — Postgres `to_tsvector('simple', body) @@ websearch_to_tsquery($q)`
//!     ranked by `ts_rank_cd`. Always available; no external dependency.
//!   * **Vector** — `embedding <=> $query_vec ORDER BY ... LIMIT k` against
//!     the pgvector column populated by FR-AI-019. Runs only if the
//!     embedding client is configured AND the query embeds successfully;
//!     a flaky sidecar degrades to lexical-only without erroring.
//!
//! Fusion: standard RRF with `k = 60` (the canonical paper default).
//! Final list is capped at the caller's `limit` (1..100).
//!
//! Tenant scope still comes from the `X-Tenant-Id` header in this slice;
//! the memory service will move to JWT-Extension scoping when its own
//! JWT-verify middleware lands (mirrors the AUTH service pattern).

use axum::{
    extract::{Json as JsonInput, State},
    http::{HeaderMap, StatusCode},
    response::Json,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::collections::HashMap;
use uuid::Uuid;

use crate::embeddings::{to_pgvector_literal, EmbeddingClient};
use crate::state::AppState;

const RRF_K: f64 = 60.0;
const LEX_FETCH_K: i64 = 50;
const VEC_FETCH_K: i64 = 50;

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    #[serde(default = "default_limit")]
    pub limit: i64,
    /// Optional override: `"lexical"` forces lexical-only; `"vector"` forces
    /// vector-only; default `"hybrid"` does RRF fusion.
    #[serde(default)]
    pub mode: Option<String>,
}
fn default_limit() -> i64 {
    20
}

#[derive(Debug, Serialize)]
pub struct SearchHit {
    pub seq: i64,
    pub path: String,
    pub snippet: String,
    pub score: f64,
    pub lexical_rank: Option<i64>,
    pub vector_rank: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub query: String,
    pub tenant_id: Uuid,
    pub mode: &'static str,
    pub total: usize,
    pub hits: Vec<SearchHit>,
}

pub async fn search(
    State(state): State<AppState>,
    headers: HeaderMap,
    JsonInput(req): JsonInput<SearchRequest>,
) -> Result<(StatusCode, Json<SearchResponse>), (StatusCode, Json<serde_json::Value>)> {
    let tenant_id = require_tenant(&headers)?;
    let limit = req.limit.clamp(1, 100);
    let mode_str = req.mode.as_deref().unwrap_or("hybrid").to_lowercase();
    let mode: &'static str = match mode_str.as_str() {
        "lexical" => "lexical",
        "vector" => "vector",
        _ => "hybrid",
    };

    // 1. Lexical retrieval.
    let lex_hits = if mode == "vector" {
        Vec::new()
    } else {
        lexical_search(&state, tenant_id, &req.query, LEX_FETCH_K).await?
    };

    // 2. Vector retrieval (best-effort).
    let vec_hits = if mode == "lexical" {
        Vec::new()
    } else {
        match EmbeddingClient::from_env() {
            Ok(client) => match client.embed_one(&req.query).await {
                Ok(vec) => vector_search(&state, tenant_id, &vec, VEC_FETCH_K).await?,
                Err(e) => {
                    tracing::info!(error = %e, "embedding for query failed — degrading to lexical");
                    Vec::new()
                }
            },
            Err(_) => Vec::new(), // sidecar not configured — degrade silently
        }
    };

    // 3. Fuse via RRF.
    let fused = match mode {
        "lexical" => lex_hits.into_iter().take(limit as usize).collect(),
        "vector" => vec_hits.into_iter().take(limit as usize).collect(),
        _ => rrf_fuse(lex_hits, vec_hits, limit as usize),
    };

    Ok((
        StatusCode::OK,
        Json(SearchResponse {
            query: req.query,
            tenant_id,
            mode,
            total: fused.len(),
            hits: fused,
        }),
    ))
}

async fn lexical_search(
    state: &AppState,
    tenant_id: Uuid,
    query: &str,
    k: i64,
) -> Result<Vec<SearchHit>, (StatusCode, Json<serde_json::Value>)> {
    let rows = sqlx::query(
        r#"
        SELECT
            seq,
            path,
            ts_headline('simple', body, websearch_to_tsquery('simple', $2),
                'StartSel=<<<,StopSel=>>>,MaxFragments=1,MaxWords=24,MinWords=8') AS snippet,
            ts_rank_cd(to_tsvector('simple', body),
                       websearch_to_tsquery('simple', $2)) AS score
        FROM l2_memory
        WHERE tenant_id = $1
          AND to_tsvector('simple', body) @@ websearch_to_tsquery('simple', $2)
        ORDER BY score DESC, seq DESC
        LIMIT $3
        "#,
    )
    .bind(tenant_id)
    .bind(query)
    .bind(k)
    .fetch_all(&state.pg)
    .await
    .map_err(internal)?;

    Ok(rows
        .into_iter()
        .enumerate()
        .map(|(i, r)| SearchHit {
            seq: r.try_get("seq").unwrap_or_default(),
            path: r.try_get::<String, _>("path").unwrap_or_default(),
            snippet: r.try_get::<String, _>("snippet").unwrap_or_default(),
            score: r.try_get::<f32, _>("score").unwrap_or(0.0) as f64,
            lexical_rank: Some((i + 1) as i64),
            vector_rank: None,
        })
        .collect())
}

async fn vector_search(
    state: &AppState,
    tenant_id: Uuid,
    query_vec: &[f32],
    k: i64,
) -> Result<Vec<SearchHit>, (StatusCode, Json<serde_json::Value>)> {
    let lit = to_pgvector_literal(query_vec);
    let rows = sqlx::query(
        r#"
        SELECT
            seq,
            path,
            LEFT(body, 240) AS snippet,
            (embedding <=> $2::vector) AS distance
        FROM l2_memory
        WHERE tenant_id = $1
          AND embedding IS NOT NULL
        ORDER BY embedding <=> $2::vector ASC
        LIMIT $3
        "#,
    )
    .bind(tenant_id)
    .bind(&lit)
    .bind(k)
    .fetch_all(&state.pg)
    .await
    .map_err(internal)?;

    Ok(rows
        .into_iter()
        .enumerate()
        .map(|(i, r)| {
            let dist = r.try_get::<f64, _>("distance").unwrap_or(2.0);
            SearchHit {
                seq: r.try_get("seq").unwrap_or_default(),
                path: r.try_get::<String, _>("path").unwrap_or_default(),
                snippet: r.try_get::<String, _>("snippet").unwrap_or_default(),
                // Score = 1 - cosine_distance, clipped to [0, 1].
                score: (1.0 - dist).clamp(0.0, 1.0),
                lexical_rank: None,
                vector_rank: Some((i + 1) as i64),
            }
        })
        .collect())
}

/// Reciprocal Rank Fusion. For each document, fused_score = Σ 1/(k + rank_i)
/// across every retriever that returned it. Documents from only one retriever
/// still contribute proportionally to their rank in that retriever.
fn rrf_fuse(
    lex: Vec<SearchHit>,
    vec: Vec<SearchHit>,
    limit: usize,
) -> Vec<SearchHit> {
    // Index by (seq, path) tuple. We keep the lexical hit object as the base
    // and merge the vector rank in (or vice-versa).
    let mut by_key: HashMap<(i64, String), SearchHit> = HashMap::new();

    for h in lex {
        by_key.insert((h.seq, h.path.clone()), h);
    }
    for h in vec {
        let key = (h.seq, h.path.clone());
        match by_key.get_mut(&key) {
            Some(existing) => {
                existing.vector_rank = h.vector_rank;
                // Preserve vector snippet only if lexical didn't supply one.
                if existing.snippet.is_empty() {
                    existing.snippet = h.snippet;
                }
            }
            None => {
                by_key.insert(key, h);
            }
        }
    }

    // Compute fused score.
    let mut hits: Vec<SearchHit> = by_key
        .into_values()
        .map(|mut h| {
            let lex_term = h
                .lexical_rank
                .map(|r| 1.0 / (RRF_K + r as f64))
                .unwrap_or(0.0);
            let vec_term = h
                .vector_rank
                .map(|r| 1.0 / (RRF_K + r as f64))
                .unwrap_or(0.0);
            h.score = lex_term + vec_term;
            h
        })
        .collect();
    hits.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    hits.truncate(limit);
    hits
}

fn require_tenant(
    headers: &HeaderMap,
) -> Result<Uuid, (StatusCode, Json<serde_json::Value>)> {
    headers
        .get("x-tenant-id")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "X-Tenant-Id header required (UUID)"})),
            )
        })
}

fn internal<E: std::fmt::Display>(e: E) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({"error": format!("search query failed: {e}")})),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hit(seq: i64, lex: Option<i64>, vec: Option<i64>) -> SearchHit {
        SearchHit {
            seq,
            path: format!("p/{seq}"),
            snippet: String::new(),
            score: 0.0,
            lexical_rank: lex,
            vector_rank: vec,
        }
    }

    #[test]
    fn rrf_merges_overlapping_docs() {
        let lex = vec![hit(1, Some(1), None), hit(2, Some(2), None)];
        let vec_ = vec![hit(1, None, Some(1)), hit(3, None, Some(2))];
        let out = rrf_fuse(lex, vec_, 10);
        // Doc 1 should rank first — present in both with rank 1.
        assert_eq!(out[0].seq, 1);
        assert!(out[0].score > out[1].score);
    }

    #[test]
    fn rrf_honors_limit() {
        let many: Vec<SearchHit> = (1..=20).map(|i| hit(i, Some(i), None)).collect();
        let out = rrf_fuse(many, vec![], 5);
        assert_eq!(out.len(), 5);
    }

    #[test]
    fn rrf_lexical_only_preserves_order() {
        let lex = vec![hit(5, Some(1), None), hit(6, Some(2), None), hit(7, Some(3), None)];
        let out = rrf_fuse(lex, vec![], 3);
        assert_eq!(out.iter().map(|h| h.seq).collect::<Vec<_>>(), vec![5, 6, 7]);
    }
}
