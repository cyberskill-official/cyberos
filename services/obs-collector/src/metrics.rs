//! TASK-OBS-001 §1 #14 — Self-metric family emitted by the supervisor on `:8888`.
//!
//! Slice-1 defines the metric names + label sets as constants so call sites in
//! follow-on FRs (TASK-OBS-002 / TASK-OBS-007 dashboard rules) reference one canonical
//! source. The actual exposition is delegated to the upstream `otelcol-contrib`
//! self-metrics endpoint; the supervisor proxies + decorates with `cyberos_` prefix.

/// Self-metric names emitted on `:8888/metrics`.
pub mod names {
    /// Counter — receiver-side span count, labelled by `service`.
    pub const RECEIVED_SPANS_TOTAL: &str = "obs_collector_received_spans_total";
    /// Counter — receiver-side log count, labelled by `service`.
    pub const RECEIVED_LOGS_TOTAL: &str = "obs_collector_received_logs_total";
    /// Counter — receiver-side metric count, labelled by `service`.
    pub const RECEIVED_METRICS_TOTAL: &str = "obs_collector_received_metrics_total";
    /// Counter — dropped count by reason (`auth` | `pii_scrub` | `backend_error` | `buffer_full`).
    pub const DROPPED_TOTAL: &str = "obs_collector_dropped_total";
    /// Gauge — file-storage buffer bytes in use.
    pub const BUFFER_BYTES: &str = "obs_collector_buffer_bytes";
    /// Histogram — exporter-side latency by backend (`loki`/`prometheusremotewrite`/`otlp/tempo`).
    pub const EXPORT_LATENCY_MS: &str = "obs_collector_export_latency_ms";
}

/// Slice-1 self-metric label values for `dropped_total{reason=…}`.
pub mod drop_reasons {
    /// Bearer-token auth failure.
    pub const AUTH: &str = "auth";
    /// PII-scrub processor matched.
    pub const PII_SCRUB: &str = "pii_scrub";
    /// Downstream backend exporter errored.
    pub const BACKEND_ERROR: &str = "backend_error";
    /// file_storage buffer at cap.
    pub const BUFFER_FULL: &str = "buffer_full";
}

/// Slice-1 self-metric label values for `export_latency_ms{backend=…}`.
pub mod backends {
    /// Loki (logs).
    pub const LOKI: &str = "loki";
    /// Prometheus remote-write (metrics).
    pub const PROMETHEUS: &str = "prometheusremotewrite";
    /// Tempo (traces).
    pub const TEMPO: &str = "otlp/tempo";
}
