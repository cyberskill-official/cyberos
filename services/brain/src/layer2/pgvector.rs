//! pgvector projection — upsert l2_memory rows with embeddings (FR-BRAIN-101 +
//! FR-AI-019). Wave-1 first-slice ships the `upsert_memory` half WITHOUT real
//! embeddings; the bge-m3 embedding sidecar lands in FR-AI-019 and we'll wire
//! the embed call here when that ships.

use crate::layer2::binlog_tail::L1Row;
use sqlx::PgPool;

/// Upsert a single L1 row's memory projection into `l2_memory`. Idempotent
/// on the (tenant_id, seq, path) primary key — re-running the ingest after
/// a crash is a no-op.
pub async fn upsert_memory(pool: &PgPool, row: &L1Row) -> Result<(), sqlx::Error> {
    // Delete/move ops drop the body; we record them with empty body so the
    // chain anchor still verifies on read.
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
