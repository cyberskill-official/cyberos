//! FR-MEMORY-123 — the BRAIN: the captured FR-MEMORY-121 interaction-event log becomes a fast, persistent,
//! citable brain. Ingestion + embedding (via the ai-gateway) + rolling summaries + hot/warm/cold tiering +
//! access-scoped, provenance-carrying recall.
//!
//! Design invariants (the FR's load-bearing decisions):
//!   * DEC-2721 — `l1_audit_log` (the hash chain) is the SYSTEM OF RECORD; `brain_event_embedding` +
//!     `brain_summary` are a DERIVED, rebuildable fast lens. Layer 1 wins on any conflict. The worker is
//!     READ-ONLY over the chain (it never writes, deletes, or mutates an audit row).
//!   * DEC-2722 — recall is ACCESS-SCOPED: tenant RLS at the DB PLUS the FR-EVAL-001 per-subject access
//!     predicate ([`access_scope`]), applied as an EXCLUDE (a closest neighbour the caller may not see never
//!     appears, it is not merely deranked) with deny-by-default on an unknown subject.
//!   * DEC-2723 — embeddings + summaries are generated ONLY through the ai-gateway ([`embed_client`]), which
//!     pins residency + ZDR + the tenant spend cap. Over-cap degrades to `pending_*` with backoff, never to a
//!     direct provider call.
//!   * DEC-2726 — every recall hit carries a [`Provenance`] pointer back to the exact `l1_audit_log` row(s)
//!     it was derived from, with a read-time `chain_anchor` verify ([`provenance`]) so FR-EVAL-003 can cite
//!     tamper-evident events.
//!
//! Layout:
//!   * [`mod@event_cursor`]  — per-tenant cursor over the FR-MEMORY-121 event stream; restart resume (§1 #1).
//!   * [`mod@embed_client`]  — the ONLY embedding path: the ai-gateway embeddings call (§1 #2, #13).
//!   * [`mod@ingest_worker`] — consume events -> embed -> idempotent UPSERT into pgvector (§1 #2, #12).
//!   * [`mod@summarize`]     — rolling per-subject / per-channel / per-window summaries (§1 #4).
//!   * [`mod@tiering`]       — age-based hot -> warm -> cold transitions (§1 #6).
//!   * [`mod@recall`]        — `POST /v1/memory/recall`: summaries-first, access-scoped (§1 #5, #7, #8, #9).
//!   * [`mod@access_scope`]  — the FR-EVAL-001 per-subject access predicate (§1 #8).
//!   * [`mod@provenance`]    — map every row to its Layer-1 source + read-time chain-anchor verify (§1 #10).
//!   * [`mod@backfill`]      — re-embed / re-summarise / index-rebuild from the chain (§1 #14).
//!   * [`mod@metrics`]       — OTel ingest lag, recall p50/p99, index size, spend, access-denied (§1 #15).

pub mod access_scope;
pub mod backfill;
pub mod embed_client;
pub mod event_cursor;
pub mod handler;
pub mod ingest_worker;
pub mod metrics;
pub mod provenance;
pub mod recall;
pub mod summarize;
pub mod tiering;

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

pub use embed_client::{EmbedClient, EmbedError};

/// The embedding dimension every brain vector carries (bge-m3 via the ai-gateway, FR-AI-019). Mirrors the
/// `VECTOR(1024)` column and the existing `crate::embeddings::DEFAULT_DIM`.
pub const EMBED_DIM: usize = 1024;

/// One interaction-event lifted out of the Layer-1 chain, ready to embed (§1 #2). `body` is the text the
/// worker embeds; `chain_anchor_hex` is the Layer-1 anchor carried for read-time tamper detection (§1 #10).
#[derive(Clone, Debug)]
pub struct BrainEvent {
    /// `l1_audit_log.seq` — the cursor key (§1 #1).
    pub source_seq: i64,
    /// Provenance pointer into `l1_audit_log` (§1 #9, #26) — `l1:<tenant>:<seq>`.
    pub audit_row_id: String,
    /// Whose interaction — the FR-EVAL-001 access subject (the Layer-1 row's `subject_id`).
    pub subject_id: Uuid,
    /// Where — chat channel / module surface; `None` for surfaces without one.
    pub channel_id: Option<Uuid>,
    /// The interaction kind from the FR-MEMORY-121 payload (`payload.event_type`, e.g. `chat.message_created`).
    pub kind: String,
    /// Occurred-at ns — the interaction's own `occurred_at_ns` from the Layer-1 payload (NOT the
    /// audit write time), so age-based tiering + recency reflect when the interaction happened.
    pub ts_ns: i64,
    /// The text embedded for semantic recall (the Layer-1 row `body`).
    pub body: String,
    /// `SHA-256(prev_hash_hex || body)` as lowercase hex, as Layer 1 advertises it (§1 #10).
    pub chain_anchor_hex: String,
}

impl BrainEvent {
    /// The canonical provenance id for a Layer-1 row: `l1:<tenant>:<seq>`. Stable + unique per chain row, so
    /// a recall hit's `audit_row_id` resolves to exactly one Layer-1 row for verification + citation.
    pub fn make_audit_row_id(tenant_id: Uuid, source_seq: i64) -> String {
        format!("l1:{tenant_id}:{source_seq:08x}")
    }
}

/// The caller of a recall, resolved from the FR-AUTH-004 JWT: their tenant + their subject identity. The
/// access predicate ([`access_scope`]) resolves what subjects this caller may see (§1 #8).
#[derive(Clone, Copy, Debug)]
pub struct Caller {
    pub tenant_id: Uuid,
    /// The caller's own subject id (their `viewer_subject_id` in FR-EVAL-001's `access_grant`).
    pub viewer_subject_id: Uuid,
}

/// The recall request body (§1 #7). `limit` default 10, max 100; `drill` opts into raw hot-event search on
/// top of summaries-first; `explain` returns the diagnostic envelope (§8 example payload).
#[derive(Debug, Deserialize)]
pub struct RecallQuery {
    pub q: String,
    /// Optional narrowing to specific subjects; STILL re-checked against FR-EVAL-001 (a subject_scope value
    /// the caller may not see is excluded — narrowing can only ever reduce, never widen, the access set).
    #[serde(default)]
    pub subject_scope: Option<Vec<Uuid>>,
    #[serde(default)]
    pub channel_scope: Option<Vec<Uuid>>,
    #[serde(default)]
    pub ts_since: Option<i64>,
    #[serde(default)]
    pub ts_until: Option<i64>,
    #[serde(default = "default_limit")]
    pub limit: usize,
    /// Search raw hot events too, not just summaries (§1 #5).
    #[serde(default)]
    pub drill: bool,
    #[serde(default)]
    pub explain: bool,
}
fn default_limit() -> usize {
    10
}

/// The maximum `limit` a recall may request (§1 #7). Over this is a `400 limit_too_large`.
pub const MAX_RECALL_LIMIT: usize = 100;

/// Whether a hit came from a rolling summary or a raw event (§1 #9).
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HitSource {
    Event,
    Summary,
}

/// A provenance pointer back into `l1_audit_log` (§1 #9, DEC-2726). An event hit cites one `audit_row_id`;
/// a summary hit cites its `covered_seq_range` plus the top contributing rows. `chain_verified` records
/// whether the read-time anchor recompute matched Layer 1 (§1 #10).
#[derive(Debug, Clone, Serialize)]
pub struct Provenance {
    pub audit_row_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub covered_seq_range: Option<(i64, i64)>,
    pub chain_verified: bool,
}

/// One ranked recall result (§1 #9). `provenance` makes it citable; `source` says whether it came from a
/// summary or a raw event; `score` is the fused rank score (RRF over summary + event hits, §1 #5).
#[derive(Debug, Clone, Serialize)]
pub struct RecallHit {
    pub audit_row_id: String,
    pub subject_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<Uuid>,
    pub kind: String,
    pub ts_ns: i64,
    pub snippet: String,
    pub score: f32,
    pub source: HitSource,
    pub provenance: Provenance,
}

/// The recall response (§8 example). `degraded_backends` lists any retriever that fell back (e.g. the query
/// embed when the gateway is down, §1 #18); `explain` carries the diagnostic envelope when requested.
#[derive(Debug, Serialize)]
pub struct RecallResults {
    pub items: Vec<RecallHit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explain: Option<serde_json::Value>,
    pub degraded_backends: Vec<String>,
}

/// Brain runtime config, read from the environment at startup. Defaults match the FR (30d hot / 180d warm,
/// §1 #6). Kept in one place so the worker, the tiering pass, and the tests agree.
#[derive(Clone, Copy, Debug)]
pub struct BrainConfig {
    /// Events newer than this age stay `hot` (fully HNSW-indexed). Default 30 days.
    pub hot_max_age_ns: i64,
    /// Events between `hot_max_age` and this age are `warm`; older are `cold`. Default 180 days.
    pub warm_max_age_ns: i64,
    /// Re-summarise a window once it has at least this many new events since its current summary. Keeps a
    /// high-write channel from re-summarising on literally every event (§10 "summary window churns").
    pub summary_min_new_events: i64,
    /// Recall confidence floor: if the best summary score is below this, drill into hot events even when
    /// `drill=false` (§1 #5).
    pub recall_confidence_floor: f32,
}

impl Default for BrainConfig {
    fn default() -> Self {
        const DAY_NS: i64 = 86_400 * 1_000_000_000;
        Self {
            hot_max_age_ns: 30 * DAY_NS,
            warm_max_age_ns: 180 * DAY_NS,
            summary_min_new_events: 5,
            recall_confidence_floor: 0.30,
        }
    }
}

impl BrainConfig {
    /// Read overrides from the environment, falling back to [`Default`]. All ages are configured in DAYS for
    /// operator ergonomics (`BRAIN_HOT_MAX_AGE_DAYS`, `BRAIN_WARM_MAX_AGE_DAYS`).
    pub fn from_env() -> Self {
        const DAY_NS: i64 = 86_400 * 1_000_000_000;
        let mut c = Self::default();
        if let Some(d) = env_i64("BRAIN_HOT_MAX_AGE_DAYS") {
            c.hot_max_age_ns = d.saturating_mul(DAY_NS);
        }
        if let Some(d) = env_i64("BRAIN_WARM_MAX_AGE_DAYS") {
            c.warm_max_age_ns = d.saturating_mul(DAY_NS);
        }
        if let Some(n) = env_i64("BRAIN_SUMMARY_MIN_NEW_EVENTS") {
            c.summary_min_new_events = n.max(1);
        }
        if let Ok(f) = std::env::var("BRAIN_RECALL_CONFIDENCE_FLOOR") {
            if let Ok(v) = f.parse::<f32>() {
                c.recall_confidence_floor = v;
            }
        }
        c
    }
}

fn env_i64(key: &str) -> Option<i64> {
    std::env::var(key).ok().and_then(|v| v.parse::<i64>().ok())
}

/// Current wall-clock in ns since the Unix epoch. Shared by ingest lag + tiering age math.
pub fn now_ns() -> i64 {
    chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
}

/// Set the per-transaction tenant GUC (`app.tenant_id`) the brain-table RLS policies read (§1 #16). Mirrors
/// the eval module's `tenant_tx`: every brain query runs inside a transaction whose `app.tenant_id` is the
/// caller's tenant, so RLS confines reads + writes to that tenant. `set_config(..., true)` is transaction-
/// local (reset on commit/rollback) so a pooled connection never leaks one tenant's GUC into the next.
///
/// MEM-002 (R74): the brain-table policies are FAIL-CLOSED (migration 0009) — there is no NULL/unset arm and
/// no nil-uuid bypass, so a query that forgets this wrapper matches `tenant_id = NULL` and reads ZERO rows
/// rather than every tenant's. Every brain-table access MUST go through here (or [`tenant_tx`]).
pub async fn rls_set_tenant(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT set_config('app.tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut **tx)
        .await?;
    Ok(())
}

/// Begin a tenant-scoped transaction with `app.tenant_id` set, so the brain-table RLS policies fire. The
/// caller commits/rolls back. Convenience over [`rls_set_tenant`] for the common "open a scoped tx" case.
pub async fn tenant_tx(
    pool: &PgPool,
    tenant_id: Uuid,
) -> Result<sqlx::Transaction<'_, sqlx::Postgres>, sqlx::Error> {
    let mut tx = pool.begin().await?;
    rls_set_tenant(&mut tx, tenant_id).await?;
    Ok(tx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_row_id_is_stable_and_unique_per_seq() {
        let t = Uuid::nil();
        let a = BrainEvent::make_audit_row_id(t, 0x1f3a2);
        assert_eq!(a, "l1:00000000-0000-0000-0000-000000000000:0001f3a2");
        // Different seq -> different id.
        assert_ne!(a, BrainEvent::make_audit_row_id(t, 0x1f3a3));
    }

    #[test]
    fn recall_query_defaults() {
        let q: RecallQuery = serde_json::from_str(r#"{"q":"hello"}"#).unwrap();
        assert_eq!(q.limit, 10);
        assert!(!q.drill);
        assert!(!q.explain);
        assert!(q.subject_scope.is_none());
    }

    #[test]
    fn config_defaults_are_30d_hot_180d_warm() {
        let c = BrainConfig::default();
        const DAY_NS: i64 = 86_400 * 1_000_000_000;
        assert_eq!(c.hot_max_age_ns, 30 * DAY_NS);
        assert_eq!(c.warm_max_age_ns, 180 * DAY_NS);
        assert!(c.warm_max_age_ns > c.hot_max_age_ns);
    }

    #[test]
    fn hit_source_serialises_snake_case() {
        assert_eq!(
            serde_json::to_string(&HitSource::Summary).unwrap(),
            "\"summary\""
        );
        assert_eq!(
            serde_json::to_string(&HitSource::Event).unwrap(),
            "\"event\""
        );
    }

    #[test]
    fn provenance_omits_covered_range_for_event_hits() {
        let p = Provenance {
            audit_row_ids: vec!["l1:t:0001".into()],
            covered_seq_range: None,
            chain_verified: true,
        };
        let v = serde_json::to_value(&p).unwrap();
        assert!(v.get("covered_seq_range").is_none());
        assert_eq!(v["chain_verified"], true);
    }
}
