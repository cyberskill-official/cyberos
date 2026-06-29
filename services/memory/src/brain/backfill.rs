//! FR-MEMORY-123 §1 #14 — backfill + rebuild: the derived lens is reproducible from the Layer-1 chain.
//!
//! DEC-2721 makes `l1_audit_log` the system of record and the embedding + summary tables a DERIVED lens. This
//! module is the proof + the recovery path:
//!   * `rebuild` — reset the cursor to 0 and re-ingest every interaction-event from the chain, re-deriving
//!     `brain_event_embedding` from `source_seq` 0. A model swap or an index bug is then recoverable, never
//!     destructive (AC #16: a rebuild must match a fresh ingest of the same range).
//!   * `reembed` — migrate to a new embedding model version, recording `embed_model_version` per row so a
//!     mixed-version migration is observable + recall still answers throughout (§1 #14, AC #17).
//!   * `resummarize` — force-rebuild summaries for a scope (or all current scopes) from the events.
//!   * `reindex_hot_hnsw` — `REINDEX` the partial hot HNSW index without dropping the table (§1 #14, §10 HNSW
//!     fragmentation).
//!   * `derived_fingerprint` — a stable digest of the derived state used to assert the derivability invariant
//!     in tests (rebuild == fresh ingest).

use sqlx::{PgPool, Row};
use uuid::Uuid;

use super::EmbedClient;

/// Rebuild the entire derived lens for `tenant_id` from Layer 1 (§1 #14). Resets the cursor to 0 then drains
/// the event stream through the normal ingest path, so the rebuilt state is byte-for-byte what a fresh ingest
/// of the same chain range produces. Returns the number of events re-ingested. The chain is never written.
pub async fn rebuild(
    tenant_id: Uuid,
    pool: &PgPool,
    gw: &EmbedClient,
) -> Result<usize, sqlx::Error> {
    // Reset the ingest cursor to 0 (admin path; nil-tenant RLS bypass via the tx).
    {
        let mut tx = super::tenant_tx(pool, tenant_id).await?;
        super::event_cursor::reset_in_tx(&mut tx, tenant_id).await?;
        tx.commit().await?;
    }
    // Drain the whole stream by repeatedly running the ingest pass until it makes no progress. The UPSERT is
    // idempotent, so re-deriving over existing rows refreshes them rather than duplicating (§1 #12).
    let mut total = 0usize;
    loop {
        let stats = super::ingest_worker::ingest_one_tenant(tenant_id, pool, gw).await?;
        let advanced = stats.embedded + stats.pending;
        total += stats.embedded;
        if advanced == 0 {
            break;
        }
    }
    Ok(total)
}

/// Re-embed every row for `tenant_id` with the current gateway model, stamping the new `embed_model_version`
/// per row (§1 #14, AC #17). Reads each row's body back from Layer 1 (the source of truth) by `source_seq`
/// and rewrites the vector. Recall keeps answering during the migration (rows are updated in place). Returns
/// the number of rows re-embedded. Bounded per call by `batch`; call until it returns 0.
pub async fn reembed(
    tenant_id: Uuid,
    pool: &PgPool,
    gw: &EmbedClient,
    target_model_alias: &str,
    batch: i64,
) -> Result<usize, sqlx::Error> {
    // Rows not already on the target model version.
    let seqs: Vec<(i64,)> = {
        let mut tx = super::tenant_tx(pool, tenant_id).await?;
        let rows = sqlx::query_as(
            "SELECT source_seq FROM brain_event_embedding
              WHERE tenant_id = $1 AND embed_model_version <> $2
              ORDER BY source_seq ASC LIMIT $3",
        )
        .bind(tenant_id)
        .bind(target_model_alias)
        .bind(batch)
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;
        rows
    };
    if seqs.is_empty() {
        return Ok(0);
    }

    let mut written = 0usize;
    for (seq,) in seqs {
        let evs = super::event_cursor::read_after(pool, tenant_id, seq - 1, 1).await?;
        let Some(ev) = evs.into_iter().find(|e| e.source_seq == seq) else {
            continue;
        };
        if let Ok(emb) = gw.embed(tenant_id, &ev.body).await {
            let lit = crate::embeddings::to_pgvector_literal(&emb.vector);
            let mut tx = super::tenant_tx(pool, tenant_id).await?;
            sqlx::query(
                "UPDATE brain_event_embedding
                    SET embedding = $1::vector, embed_model_version = $2, embed_state = 'complete',
                        stale = FALSE, updated_at = NOW()
                  WHERE tenant_id = $3 AND source_seq = $4",
            )
            .bind(&lit)
            .bind(&emb.model_version)
            .bind(tenant_id)
            .bind(seq)
            .execute(&mut *tx)
            .await?;
            tx.commit().await?;
            written += 1;
        }
    }
    Ok(written)
}

/// Force a re-summarise of one scope, or of every current scope when `scope` is `None` (§1 #14). For the
/// all-scopes case it re-summarises each distinct (scope_kind, scope_id) that currently has a summary.
pub async fn resummarize(
    tenant_id: Uuid,
    pool: &PgPool,
    gw: &EmbedClient,
    scope: Option<(&str, &str)>,
) -> Result<usize, sqlx::Error> {
    if let Some((kind, id)) = scope {
        let subject = if kind == "subject" {
            Uuid::parse_str(id).ok()
        } else {
            None
        };
        let v = super::summarize::resummarize_now(pool, tenant_id, kind, id, subject, gw).await?;
        return Ok(v.is_some() as usize);
    }

    // All current scopes.
    let scopes: Vec<(String, String)> = {
        let mut tx = super::tenant_tx(pool, tenant_id).await?;
        let rows = sqlx::query_as(
            "SELECT scope_kind, scope_id FROM brain_summary
              WHERE tenant_id = $1 AND superseded_by IS NULL",
        )
        .bind(tenant_id)
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;
        rows
    };
    let mut n = 0usize;
    for (kind, id) in scopes {
        let subject = if kind == "subject" {
            Uuid::parse_str(&id).ok()
        } else {
            None
        };
        if super::summarize::resummarize_now(pool, tenant_id, &kind, &id, subject, gw)
            .await?
            .is_some()
        {
            n += 1;
        }
    }
    Ok(n)
}

/// `REINDEX` the partial hot HNSW index (§1 #14, §10 HNSW fragmentation). The table stays online; only the
/// index is rebuilt. Runs outside a transaction (REINDEX cannot run inside one for some forms; CONCURRENTLY
/// keeps reads available). Best-effort: a failure is surfaced to the operator.
pub async fn reindex_hot_hnsw(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("REINDEX INDEX CONCURRENTLY brain_event_embedding_hot_hnsw")
        .execute(pool)
        .await?;
    Ok(())
}

/// A stable fingerprint of the derived lens for a tenant, for the derivability-invariant test (AC #16). It
/// digests the embedding rows' identity + provenance + tier (NOT the float vectors, which a deterministic
/// stub makes reproducible but which a real model may render with tiny float noise) plus the summary
/// coverage. A rebuild from Layer 1 must yield the same fingerprint as a fresh ingest of the same range.
pub async fn derived_fingerprint(pool: &PgPool, tenant_id: Uuid) -> Result<String, sqlx::Error> {
    let mut tx = super::tenant_tx(pool, tenant_id).await?;
    let ev_rows = sqlx::query(
        "SELECT source_seq, audit_row_id, subject_id, kind, tier
           FROM brain_event_embedding WHERE tenant_id = $1 ORDER BY source_seq ASC",
    )
    .bind(tenant_id)
    .fetch_all(&mut *tx)
    .await?;
    let sum_rows = sqlx::query(
        "SELECT scope_kind, scope_id, covered_seq_lo, covered_seq_hi
           FROM brain_summary WHERE tenant_id = $1 AND superseded_by IS NULL
          ORDER BY scope_kind, scope_id",
    )
    .bind(tenant_id)
    .fetch_all(&mut *tx)
    .await?;
    tx.commit().await?;

    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    for r in &ev_rows {
        let seq: i64 = r.try_get("source_seq").unwrap_or_default();
        let arid: String = r.try_get("audit_row_id").unwrap_or_default();
        let subj: Uuid = r.try_get("subject_id").unwrap_or_else(|_| Uuid::nil());
        let kind: String = r.try_get("kind").unwrap_or_default();
        let tier: String = r.try_get("tier").unwrap_or_default();
        h.update(format!("E|{seq}|{arid}|{subj}|{kind}|{tier}\n").as_bytes());
    }
    for r in &sum_rows {
        let sk: String = r.try_get("scope_kind").unwrap_or_default();
        let sid: String = r.try_get("scope_id").unwrap_or_default();
        let lo: i64 = r.try_get("covered_seq_lo").unwrap_or_default();
        let hi: i64 = r.try_get("covered_seq_hi").unwrap_or_default();
        h.update(format!("S|{sk}|{sid}|{lo}|{hi}\n").as_bytes());
    }
    Ok(hex_lower(&h.finalize()))
}

/// Truncate the derived lens for a tenant (embeddings + summaries + cursor), keeping Layer 1 intact — the
/// setup step for the rebuild test (AC #16) and an operator "re-derive from scratch" action.
pub async fn truncate_derived(pool: &PgPool, tenant_id: Uuid) -> Result<(), sqlx::Error> {
    let mut tx = super::tenant_tx(pool, tenant_id).await?;
    sqlx::query("DELETE FROM brain_event_embedding WHERE tenant_id = $1")
        .bind(tenant_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM brain_summary WHERE tenant_id = $1")
        .bind(tenant_id)
        .execute(&mut *tx)
        .await?;
    super::event_cursor::reset_in_tx(&mut tx, tenant_id).await?;
    tx.commit().await?;
    Ok(())
}

fn hex_lower(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_lower_is_64_chars_for_sha256() {
        use sha2::{Digest, Sha256};
        let d = Sha256::digest(b"x");
        assert_eq!(hex_lower(&d).len(), 64);
    }
}
