//! cyberos-obs-collector — supervisor + tenant-isolation hooks around the upstream
//! `otelcol-contrib` binary.
//!
//! ## Architecture
//!
//! Slice-1 deployment runs the OpenTelemetry Collector (otelcol-contrib) as the data
//! plane; this Rust crate provides:
//!
//! - **Config validation** — `config::validate()` parses `otel-collector-config.yaml`
//!   and asserts the pipeline shape required by TASK-OBS-001 §3 (otlp→resource→
//!   attributes/pii_scrub→batch→loki/prometheusremotewrite/otlp/tempo).
//! - **Token-file management** — `auth::TokenFile` reads + reloads the bearer-token
//!   file the otelcol bearertokenauth extension consumes (TASK-OBS-001 §1 #2).
//! - **Self-metric types** — `metrics::SelfMetrics` defines the `obs_collector_*`
//!   metric family that the collector emits on `:8888` (TASK-OBS-001 §1 #14).
//!
//! The actual binary is `otelcol-contrib`, supervised by the
//! `cyberos-obs` Cargo bin in this crate (with health-check polling, log forwarding,
//! token-file reload). Helm chart + docker-compose live at `deploy/obs/`.

#![deny(missing_debug_implementations)]
#![warn(missing_docs)]

pub mod auth;
pub mod config;
pub mod metrics;

/// Banner emitted by the supervisor binary on startup.
pub const SERVICE_BANNER: &str = concat!(
    "cyberos-obs-collector v",
    env!("CARGO_PKG_VERSION"),
    " — observability supervisor (TASK-OBS-001..009)"
);
