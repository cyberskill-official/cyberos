//! FR-BRAIN-108 — `POST /v1/brain/search`.
//!
//! Hybrid recall over `l2_memory`:
//!   * **Lexical** — Postgres `to_tsvector('simple', body) @@ websearch_to_tsquery($q)`.
//!     Cheap, no extension needed beyond stdlib.
//!   * **Semantic** — pgvector `embedding <=> $query_vec ORDER BY ... LIMIT k`.
//!     Skipped automatically when `embedding IS NULL` (FR-AI-019 hasn't shipped
//!     the bge-m3 sidecar yet) — lexical results stand alone.
//!
//! Tenant scope is taken from the `X-Tenant-Id` header in this Wave-1 slice;
//! once the AUTH JWT middleware lands in services/brain too, switch to
//! `Extension<Claims>`.

use axum::{
    extract::{Json as JsonInput, State},
    http::{HeaderMap, StatusCode},
    response::Json,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    #[serde(default = "default_limit")]
    pub limit: i64,
}
fn default_limit() -> i64 { 20 }

#[derive(Debug, Serialize)]
pub struct SearchHit {
    pub seq: i64,
    pub path: String,
    pub snippet: String,
    /// Combined score: lexical rank only in Wave 1. With bge-m3 lands a
    /// reciprocal-rank fusion of lexical + vector cosine.
    pub score: f64,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub query: String,
    pub tenant_id: Uuid,
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
    .bind(&req.query)
    .bind(limit)
    .fetch_all(&state.pg)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({"error": format!("search query failed: {e}")})),
    ))?;

    let hits: Vec<SearchHit> = rows
        .into_iter()
        .map(|r| SearchHit {
            seq: r.try_get("seq").unwrap_or_default(),
            path: r.try_get::<String, _>("path").unwrap_or_default(),
            snippet: r.try_get::<String, _>("snippet").unwrap_or_default(),
            score: r.try_get::<f32, _>("score").unwrap_or(0.0) as f64,
        })
        .collect();

    Ok((
        StatusCode::OK,
        Json(SearchResponse {
            query: req.query,
            tenant_id,
            total: hits.len(),
            hits,
        }),
    ))
}

fn require_tenant(headers: &HeaderMap) -> Result<Uuid, (StatusCode, Json<serde_json::Value>)> {
    headers
        .get("x-tenant-id")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "X-Tenant-Id header required (UUID)"})),
        ))
}
