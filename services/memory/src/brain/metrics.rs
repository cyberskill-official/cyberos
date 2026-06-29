//! FR-MEMORY-123 §1 #15 — OTel metrics for the brain, emitted as structured `tracing` events (this crate's
//! metrics path is OTel via `cyberos-obs-sdk`, exactly like `interaction::emit`). Each function emits one
//! event carrying the metric name + its label set as structured fields; the obs pipeline derives the
//! histograms / gauges / counters. Promoting these to native meters later does not change these call sites.
//!
//! The metric names match the FR verbatim so the obs dashboards + SLO alerts bind without translation.

use uuid::Uuid;

/// `memory_brain_ingest_lag_seconds{tenant_id}` (histogram): event-append -> embedding-visible (§1 #15).
/// `lag_ns` is `now_ns() - event.ts_ns` at the moment the embedding row commits.
pub fn ingest_lag(tenant_id: Uuid, lag_ns: i64) {
    let lag_s = (lag_ns as f64) / 1_000_000_000.0;
    tracing::info!(
        target: "cyberos_memory::brain",
        metric = "memory_brain_ingest_lag_seconds",
        tenant_id = %tenant_id,
        lag_seconds = lag_s,
        "brain ingest lag observed"
    );
}

/// `memory_brain_recall_latency_ms{tenant_id, path}` (histogram; path = summary | drill): the recall path's
/// wall time (§1 #15). The SLO is p50 + p99; the obs pipeline computes the quantiles from these samples.
pub fn recall_latency(tenant_id: Uuid, path: &str, latency_ms: f64) {
    tracing::info!(
        target: "cyberos_memory::brain",
        metric = "memory_brain_recall_latency_ms",
        tenant_id = %tenant_id,
        path = path,
        latency_ms = latency_ms,
        "brain recall latency observed"
    );
}

/// `memory_brain_tier_rows_total{tenant_id, tier}` (gauge) — also serves as `memory_brain_index_size_rows`
/// for `tier=hot` (the hot index size, §1 #15). Emitted by the tiering pass after it settles a tenant.
pub fn tier_rows(tenant_id: Uuid, tier: &str, rows: i64) {
    tracing::info!(
        target: "cyberos_memory::brain",
        metric = "memory_brain_tier_rows_total",
        tenant_id = %tenant_id,
        tier = tier,
        rows = rows,
        "brain tier row count"
    );
}

/// `memory_brain_summary_count{tenant_id, scope_kind}` (gauge): current (non-superseded) summaries per scope
/// kind (§1 #15). Emitted by the summarise pass.
pub fn summary_count(tenant_id: Uuid, scope_kind: &str, count: i64) {
    tracing::info!(
        target: "cyberos_memory::brain",
        metric = "memory_brain_summary_count",
        tenant_id = %tenant_id,
        scope_kind = scope_kind,
        count = count,
        "brain summary count"
    );
}

/// `memory_brain_embed_spend_units{tenant_id}` (counter): embedding spend charged via the gateway (§1 #15).
/// Charged ONLY on a gateway 200 (a pending row is not charged twice — §10 "spend metric over-counts").
pub fn embed_spend(tenant_id: Uuid, units: f64) {
    tracing::info!(
        target: "cyberos_memory::brain",
        metric = "memory_brain_embed_spend_units",
        tenant_id = %tenant_id,
        units = units,
        "brain embed spend charged"
    );
}

/// `memory_brain_recall_access_denied_total{tenant_id, reason}` (counter): a hit excluded by access scope
/// (§1 #8, #15). `reason` ∈ `tenant_rls | subject_scope | unknown_subject`.
pub fn access_denied(tenant_id: Uuid, reason: &str) {
    tracing::info!(
        target: "cyberos_memory::brain",
        metric = "memory_brain_recall_access_denied_total",
        tenant_id = %tenant_id,
        reason = reason,
        "brain recall access denied"
    );
}

/// `memory_brain_ingest_failures_total{tenant_id, reason}` (counter): an ingest failure (§1 #15). `reason` ∈
/// `embed_gateway_down | spend_cap_exhausted | postgres_error | chain_anchor_mismatch`.
pub fn ingest_failure(tenant_id: Uuid, reason: &str) {
    tracing::warn!(
        target: "cyberos_memory::brain",
        metric = "memory_brain_ingest_failures_total",
        tenant_id = %tenant_id,
        reason = reason,
        "brain ingest failure"
    );
}

/// Sev-1 `memory_brain_chain_anchor_mismatch{tenant_id, source_seq}` (§1 #10): a returned hit's anchor did
/// not match Layer 1 at read time. The hit is dropped and this fires — the derived index is never trusted
/// over a tampered chain.
pub fn chain_anchor_mismatch(tenant_id: Uuid, source_seq: i64) {
    tracing::error!(
        target: "cyberos_memory::brain",
        metric = "memory_brain_chain_anchor_mismatch",
        sev = 1,
        tenant_id = %tenant_id,
        source_seq = source_seq,
        "brain recall dropped a hit: chain_anchor mismatch vs Layer 1 (possible tamper)"
    );
}
