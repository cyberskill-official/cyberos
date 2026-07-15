//! TASK-MEMORY-123 §1 #2,#12,#13 — the brain ingest worker: consume TASK-MEMORY-121 events -> embed via the
//! ai-gateway -> idempotent UPSERT into `brain_event_embedding` with provenance -> advance the cursor, all
//! per-tenant.
//!
//! Idempotency (§1 #12): the embedding INSERT is `ON CONFLICT (tenant_id, source_seq) DO NOTHING`, and the
//! cursor advance commits in the SAME transaction, so a restart-mid-batch re-processes from the saved cursor
//! and produces no duplicate rows.
//!
//! Spend + residency discipline (§1 #13, DEC-2723): the body is embedded ONLY through `EmbedClient` (the
//! ai-gateway). On `SpendCapExhausted` (gateway 402) or `GatewayDown`, the worker records the row as
//! `pending_embed_retry` (with a NULL embedding) and moves on — it NEVER calls a provider directly. A retry
//! pass (`retry_pending`) later re-embeds pending rows once the cap resets / the gateway recovers.

use sqlx::PgPool;
use uuid::Uuid;

use super::{metrics, now_ns, BrainEvent, EmbedClient, EmbedError};
use crate::embeddings::to_pgvector_literal;

/// How many events to pull per ingest batch for one tenant. Bounded so a busy tenant doesn't starve others.
const INGEST_BATCH: i64 = 256;

/// Summary of one tenant's ingest pass.
#[derive(Debug, Default, Clone, Copy)]
pub struct IngestStats {
    pub embedded: usize,
    pub pending: usize,
}

/// Run one ingest pass for `tenant_id`: read events after the cursor, embed + UPSERT each, advance the
/// cursor transactionally (§1 #2). Returns counts for the caller's logging. A per-event embed failure marks
/// that row pending and continues — one flaky event never stalls the batch.
pub async fn ingest_one_tenant(
    tenant_id: Uuid,
    pool: &PgPool,
    gw: &EmbedClient,
) -> Result<IngestStats, sqlx::Error> {
    let cursor = super::event_cursor::get(pool, tenant_id).await?;
    let events = super::event_cursor::read_after(pool, tenant_id, cursor, INGEST_BATCH).await?;

    let mut stats = IngestStats::default();
    for ev in events {
        match gw.embed(tenant_id, &ev.body).await {
            Ok(emb) => {
                insert_complete(pool, tenant_id, &ev, &emb.vector, &emb.model_version).await?;
                // Spend is charged only on a gateway 200 (a pending row is never charged — §10).
                metrics::embed_spend(tenant_id, 1.0);
                metrics::ingest_lag(tenant_id, now_ns() - ev.ts_ns);
                stats.embedded += 1;
            }
            Err(EmbedError::SpendCapExhausted) => {
                metrics::ingest_failure(tenant_id, "spend_cap_exhausted");
                insert_pending(pool, tenant_id, &ev).await?;
                stats.pending += 1;
                // Back off the whole tenant: the cap won't clear within this batch.
                break;
            }
            Err(EmbedError::GatewayDown(_)) => {
                metrics::ingest_failure(tenant_id, "embed_gateway_down");
                insert_pending(pool, tenant_id, &ev).await?;
                stats.pending += 1;
                // Gateway is down for everyone; stop this batch and retry next tick.
                break;
            }
            Err(EmbedError::Malformed(_)) => {
                // Wrong dim / shape from the gateway — record pending (no vector) and surface it; a
                // re-embed once the gateway is fixed will fill it.
                metrics::ingest_failure(tenant_id, "postgres_error");
                insert_pending(pool, tenant_id, &ev).await?;
                stats.pending += 1;
            }
        }

        // Re-summarise the windows this event touches (§1 #4). Best-effort: a summarise failure does not
        // roll back the committed embedding (the event is already durably ingested).
        if let Err(e) = super::summarize::touch_windows(pool, tenant_id, &ev, gw).await {
            tracing::warn!(target: "cyberos_memory::brain", error = %e, source_seq = ev.source_seq,
                "summarise after ingest failed (event is ingested; will re-summarise next pass)");
        }
    }
    Ok(stats)
}

/// UPSERT a fully-embedded event row + advance the cursor in ONE transaction (§1 #2, #12). The embedding is
/// bound as a pgvector literal (`$N::vector`), matching the existing `layer2::pgvector` write idiom.
async fn insert_complete(
    pool: &PgPool,
    tenant_id: Uuid,
    ev: &BrainEvent,
    embedding: &[f32],
    model_version: &str,
) -> Result<(), sqlx::Error> {
    let lit = to_pgvector_literal(embedding);
    let mut tx = super::tenant_tx(pool, tenant_id).await?;
    sqlx::query(
        "INSERT INTO brain_event_embedding
            (tenant_id, source_seq, audit_row_id, subject_id, channel_id, kind, ts_ns,
             embedding, embed_model_version, chain_anchor, tier, embed_state, updated_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8::vector,$9,decode($10,'hex'),'hot','complete',NOW())
         ON CONFLICT (tenant_id, source_seq) DO UPDATE
           SET embedding = EXCLUDED.embedding,
               embed_model_version = EXCLUDED.embed_model_version,
               embed_state = 'complete',
               stale = FALSE,
               updated_at = NOW()
           WHERE brain_event_embedding.embed_state = 'pending_embed_retry'
              OR brain_event_embedding.stale = TRUE",
    )
    .bind(tenant_id)
    .bind(ev.source_seq)
    .bind(&ev.audit_row_id)
    .bind(ev.subject_id)
    .bind(ev.channel_id)
    .bind(&ev.kind)
    .bind(ev.ts_ns)
    .bind(&lit)
    .bind(model_version)
    .bind(&ev.chain_anchor_hex)
    .execute(&mut *tx)
    .await?;
    super::event_cursor::advance_in_tx(&mut tx, tenant_id, ev.source_seq).await?;
    tx.commit().await?;
    Ok(())
}

/// UPSERT an event row with NO embedding, marked `pending_embed_retry` (§1 #13). Still advances the cursor:
/// the row exists and is on the retry list, so the worker makes forward progress rather than re-pulling the
/// same event forever. `retry_pending` fills the vector later.
async fn insert_pending(
    pool: &PgPool,
    tenant_id: Uuid,
    ev: &BrainEvent,
) -> Result<(), sqlx::Error> {
    let mut tx = super::tenant_tx(pool, tenant_id).await?;
    sqlx::query(
        "INSERT INTO brain_event_embedding
            (tenant_id, source_seq, audit_row_id, subject_id, channel_id, kind, ts_ns,
             embedding, embed_model_version, chain_anchor, tier, embed_state, updated_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7,NULL,'pending',decode($8,'hex'),'hot','pending_embed_retry',NOW())
         ON CONFLICT (tenant_id, source_seq) DO NOTHING",
    )
    .bind(tenant_id)
    .bind(ev.source_seq)
    .bind(&ev.audit_row_id)
    .bind(ev.subject_id)
    .bind(ev.channel_id)
    .bind(&ev.kind)
    .bind(ev.ts_ns)
    .bind(&ev.chain_anchor_hex)
    .execute(&mut *tx)
    .await?;
    super::event_cursor::advance_in_tx(&mut tx, tenant_id, ev.source_seq).await?;
    tx.commit().await?;
    Ok(())
}

/// Re-embed rows previously marked `pending_embed_retry` for `tenant_id` (§1 #13, §10 self-heal). Reads the
/// pending rows' bodies back from Layer 1 (the system of record) by `source_seq`, embeds via the gateway, and
/// fills the vector. Stops early on a fresh `SpendCapExhausted` / `GatewayDown` (still pending; try again).
/// Returns the number of rows successfully re-embedded.
pub async fn retry_pending(
    tenant_id: Uuid,
    pool: &PgPool,
    gw: &EmbedClient,
    limit: i64,
) -> Result<usize, sqlx::Error> {
    // Find pending source_seqs (RLS-scoped).
    let pending: Vec<(i64,)> = {
        let mut tx = super::tenant_tx(pool, tenant_id).await?;
        let rows = sqlx::query_as(
            "SELECT source_seq FROM brain_event_embedding
              WHERE tenant_id = $1 AND embed_state = 'pending_embed_retry'
              ORDER BY source_seq ASC LIMIT $2",
        )
        .bind(tenant_id)
        .bind(limit)
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;
        rows
    };
    if pending.is_empty() {
        return Ok(0);
    }

    let mut written = 0usize;
    for (seq,) in pending {
        // Re-read the body from Layer 1 by seq (not from the brain row — Layer 1 is the source of truth).
        let evs = super::event_cursor::read_after(pool, tenant_id, seq - 1, 1).await?;
        let Some(ev) = evs.into_iter().find(|e| e.source_seq == seq) else {
            continue;
        };
        match gw.embed(tenant_id, &ev.body).await {
            Ok(emb) => {
                insert_complete(pool, tenant_id, &ev, &emb.vector, &emb.model_version).await?;
                metrics::embed_spend(tenant_id, 1.0);
                written += 1;
            }
            Err(EmbedError::SpendCapExhausted) | Err(EmbedError::GatewayDown(_)) => break,
            Err(EmbedError::Malformed(_)) => continue,
        }
    }
    Ok(written)
}
