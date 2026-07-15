//! pgvector projection — upsert l2_memory rows + optional embedding population
//! (TASK-MEMORY-101 + TASK-AI-019).
//!
//! Two write paths:
//!   * `upsert_memory(pool, row)` — basic upsert without an embedding; used
//!     by the ingest hot path so a flaky sidecar never blocks ingest.
//!   * `upsert_memory_with_embedding(pool, row, vec)` — same upsert + sets
//!     the `embedding` column. Used by the background re-embedder + by
//!     ingest when the embedding client is configured + healthy.

use crate::embeddings::{to_pgvector_literal, EmbedError, EmbeddingClient};
use crate::layer2::binlog_tail::L1Row;
use sqlx::PgPool;

/// Upsert a single L1 row's memory projection into `l2_memory`. Idempotent
/// on the (tenant_id, seq, path) primary key — re-running the ingest after
/// a crash is a no-op. Leaves the embedding column NULL — the re-embedder
/// or `upsert_memory_with_embedding` fills it in later.
pub async fn upsert_memory(pool: &PgPool, row: &L1Row) -> Result<(), sqlx::Error> {
    let body = row.body.as_deref().unwrap_or("");
    sqlx::query(
        "INSERT INTO l2_memory
                (tenant_id, seq, path, body, chain_anchor, frontmatter, ingested_at)
         VALUES ($1, $2, $3, $4, decode($5, 'hex'), '{}'::jsonb, NOW())
         ON CONFLICT (tenant_id, seq, path) DO NOTHING",
    )
    .bind(row.tenant_id)
    .bind(row.seq)
    .bind(&row.path)
    .bind(body)
    .bind(&row.chain_anchor_hex)
    .execute(pool)
    .await
    .map(|_| ())
}

/// TASK-AI-019 — Upsert + populate the `embedding` vector column. The caller
/// is responsible for obtaining the embedding from `EmbeddingClient::embed_one`
/// (or `embed_batch` for throughput). If the upsert is a no-op (PK conflict),
/// the existing embedding is preserved — we use a separate UPDATE for the
/// vector so we don't accidentally overwrite a newer one.
pub async fn upsert_memory_with_embedding(
    pool: &PgPool,
    row: &L1Row,
    embedding: &[f32],
) -> Result<(), sqlx::Error> {
    upsert_memory(pool, row).await?;
    let lit = to_pgvector_literal(embedding);
    sqlx::query(
        "UPDATE l2_memory
            SET embedding = $1::vector
          WHERE tenant_id = $2 AND seq = $3 AND path = $4
            AND embedding IS NULL",
    )
    .bind(&lit)
    .bind(row.tenant_id)
    .bind(row.seq)
    .bind(&row.path)
    .execute(pool)
    .await
    .map(|_| ())
}

/// TASK-AI-019 — One-shot helper used by the ingest hot path. Tries to embed
/// `row.body` via the configured sidecar, then writes both the row and the
/// embedding. If the sidecar is unconfigured or fails, falls back to a
/// bare `upsert_memory` and returns `Ok(false)` so the caller can record
/// that this row needs re-embedding later.
pub async fn try_embed_and_upsert(
    pool: &PgPool,
    embedder: Option<&EmbeddingClient>,
    row: &L1Row,
) -> Result<bool, sqlx::Error> {
    if let Some(client) = embedder {
        if let Some(body) = row.body.as_deref() {
            match client.embed_one(body).await {
                Ok(vec) => {
                    upsert_memory_with_embedding(pool, row, &vec).await?;
                    return Ok(true);
                }
                Err(e) => {
                    tracing::warn!(
                        seq = row.seq,
                        path = row.path,
                        error = %e,
                        "embedding failed — recording row without vector"
                    );
                }
            }
        }
    }
    upsert_memory(pool, row).await?;
    Ok(false)
}

/// TASK-AI-019 — Background re-embedder. Picks up to `batch_size` rows whose
/// `embedding IS NULL` and fills them in. Called by a periodic task in
/// `main.rs`. Returns the number of rows successfully embedded.
pub async fn reembed_missing(
    pool: &PgPool,
    embedder: &EmbeddingClient,
    tenant_id: uuid::Uuid,
    batch_size: i64,
) -> Result<usize, EmbedError> {
    let rows: Vec<(i64, String, String)> = sqlx::query_as(
        "SELECT seq, path, body
             FROM l2_memory
            WHERE tenant_id = $1 AND embedding IS NULL AND body <> ''
            ORDER BY seq ASC
            LIMIT $2",
    )
    .bind(tenant_id)
    .bind(batch_size)
    .fetch_all(pool)
    .await
    .map_err(|e| EmbedError::Network(e.to_string()))?;

    if rows.is_empty() {
        return Ok(0);
    }
    let bodies: Vec<&str> = rows.iter().map(|(_, _, b)| b.as_str()).collect();
    let vecs = embedder.embed_batch(&bodies).await?;

    let mut written = 0usize;
    for ((seq, path, _body), v) in rows.iter().zip(vecs.iter()) {
        let lit = to_pgvector_literal(v);
        let res = sqlx::query(
            "UPDATE l2_memory
                SET embedding = $1::vector
              WHERE tenant_id = $2 AND seq = $3 AND path = $4
                AND embedding IS NULL",
        )
        .bind(&lit)
        .bind(tenant_id)
        .bind(seq)
        .bind(path)
        .execute(pool)
        .await
        .map_err(|e| EmbedError::Network(e.to_string()))?;
        if res.rows_affected() > 0 {
            written += 1;
        }
    }
    Ok(written)
}

/// Insert an extracted entity into `l2_entity`. The entity table has no
/// unique constraint on `(tenant_id, kind, name)` in Wave 1 — duplicates are
/// expected and dedup'd in Phase 3 via embedding clustering. The ingest
/// runs do `SELECT 1 FROM l2_entity WHERE …` first to avoid trivial repeats
/// from the same source row.
pub async fn upsert_entity(
    pool: &PgPool,
    tenant_id: uuid::Uuid,
    kind: &str,
    name: &str,
    source_seq: i64,
    source_path: &str,
) -> Result<(), sqlx::Error> {
    let exists: Option<(uuid::Uuid,)> = sqlx::query_as(
        "SELECT entity_id FROM l2_entity
            WHERE tenant_id = $1 AND kind = $2 AND name = $3 AND source_seq = $4
            LIMIT 1",
    )
    .bind(tenant_id)
    .bind(kind)
    .bind(name)
    .bind(source_seq)
    .fetch_optional(pool)
    .await?;
    if exists.is_some() {
        return Ok(());
    }
    sqlx::query(
        "INSERT INTO l2_entity
                (tenant_id, kind, name, source_seq, source_path)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(tenant_id)
    .bind(kind)
    .bind(name)
    .bind(source_seq)
    .bind(source_path)
    .execute(pool)
    .await
    .map(|_| ())
}
