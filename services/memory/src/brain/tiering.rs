//! TASK-MEMORY-123 §1 #6 / DEC-2725 — age-based hot -> warm -> cold tiering of `brain_event_embedding`.
//!
//! Recency dominates relevance for evaluation, and indexing everything `hot` forever is the cost-and-latency
//! trap. The tier column bounds the partial hot HNSW index (and therefore every query's cost) to the hot
//! window:
//!   * **hot**  — recent raw events, fully HNSW-indexed, searched directly.
//!   * **warm** — older events: the embedding is RETAINED (still vector-searchable on `drill`) but the row
//!     drops out of the partial hot index; recall represents it through its summary.
//!   * **cold** — archived: the raw row stays in Layer 1 (the truth) and is retrievable on demand by
//!     `audit_row_id`; only the summary embedding is indexed for it. The embedding column is kept (the row
//!     is still in the table) but out of the hot index.
//!
//! Transitions are age-driven (`hot_max_age`, `warm_max_age`, default 30d / 180d) and IDEMPOTENT: re-running
//! the pass moves only rows whose age now crosses a boundary, so it never duplicates or loses rows. The
//! per-tenant `brain_tier_watermark` records the high-water occurred-at the pass reached.
//!
//! The partial hot HNSW index has `WHERE tier = 'hot'`, so demoting a row to warm/cold automatically removes
//! it from that index — no separate REINDEX needed on a transition (the index is maintained incrementally by
//! Postgres as the `tier` value changes).

use sqlx::PgPool;
use uuid::Uuid;

use super::{metrics, now_ns, BrainConfig};

/// The tier-row counts for a tenant (for the test assertions + the gauges).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TierCounts {
    pub hot: i64,
    pub warm: i64,
    pub cold: i64,
}

/// Run one tiering pass for `tenant_id` (§1 #6). Demotes rows whose age crosses `hot_max_age` (hot->warm) or
/// `warm_max_age` (warm->cold), advances the watermark, and emits the per-tier gauges. Idempotent: a second
/// run with no time passing is a no-op (the same WHERE clauses match nothing new).
pub async fn run_tier_pass(
    pool: &PgPool,
    tenant_id: Uuid,
    cfg: &BrainConfig,
) -> Result<TierCounts, sqlx::Error> {
    let now = now_ns();
    let hot_cutoff = now - cfg.hot_max_age_ns; // events older (ts_ns < cutoff) than this are no longer hot
    let warm_cutoff = now - cfg.warm_max_age_ns; // older than this are cold

    let mut tx = super::tenant_tx(pool, tenant_id).await?;

    // hot -> warm: rows currently hot whose ts_ns is older than the hot cutoff but newer than the warm cutoff.
    sqlx::query(
        "UPDATE brain_event_embedding
            SET tier = 'warm', updated_at = NOW()
          WHERE tenant_id = $1 AND tier = 'hot' AND ts_ns < $2 AND ts_ns >= $3",
    )
    .bind(tenant_id)
    .bind(hot_cutoff)
    .bind(warm_cutoff)
    .execute(&mut *tx)
    .await?;

    // (hot|warm) -> cold: anything older than the warm cutoff. Covers a row that aged straight past warm
    // since the last pass, so the transition is correct regardless of how long between runs (idempotent).
    sqlx::query(
        "UPDATE brain_event_embedding
            SET tier = 'cold', updated_at = NOW()
          WHERE tenant_id = $1 AND tier IN ('hot','warm') AND ts_ns < $2",
    )
    .bind(tenant_id)
    .bind(warm_cutoff)
    .execute(&mut *tx)
    .await?;

    // Advance the watermark to now (the pass has considered every event up to now).
    sqlx::query(
        "INSERT INTO brain_tier_watermark (tenant_id, last_tiered_ts_ns, last_tier_run_at)
         VALUES ($1, $2, NOW())
         ON CONFLICT (tenant_id)
         DO UPDATE SET last_tiered_ts_ns = GREATEST(brain_tier_watermark.last_tiered_ts_ns, EXCLUDED.last_tiered_ts_ns),
                       last_tier_run_at = NOW()",
    )
    .bind(tenant_id)
    .bind(now)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let counts = tier_counts(pool, tenant_id).await?;
    metrics::tier_rows(tenant_id, "hot", counts.hot);
    metrics::tier_rows(tenant_id, "warm", counts.warm);
    metrics::tier_rows(tenant_id, "cold", counts.cold);
    Ok(counts)
}

/// Read the current hot/warm/cold row counts for a tenant (RLS-scoped).
pub async fn tier_counts(pool: &PgPool, tenant_id: Uuid) -> Result<TierCounts, sqlx::Error> {
    let mut tx = super::tenant_tx(pool, tenant_id).await?;
    let rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT tier, COUNT(*) FROM brain_event_embedding WHERE tenant_id = $1 GROUP BY tier",
    )
    .bind(tenant_id)
    .fetch_all(&mut *tx)
    .await?;
    tx.commit().await?;

    let mut c = TierCounts::default();
    for (tier, n) in rows {
        match tier.as_str() {
            "hot" => c.hot = n,
            "warm" => c.warm = n,
            "cold" => c.cold = n,
            _ => {}
        }
    }
    Ok(c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_counts_default_is_zero() {
        let c = TierCounts::default();
        assert_eq!(
            c,
            TierCounts {
                hot: 0,
                warm: 0,
                cold: 0
            }
        );
    }

    #[test]
    fn cutoffs_order_hot_newer_than_warm() {
        // hot_cutoff (now - 30d) must be GREATER than warm_cutoff (now - 180d): a row between them is warm.
        let cfg = BrainConfig::default();
        let now = 1_000_000_000_000_000_000i64;
        let hot_cutoff = now - cfg.hot_max_age_ns;
        let warm_cutoff = now - cfg.warm_max_age_ns;
        assert!(hot_cutoff > warm_cutoff);
    }
}
