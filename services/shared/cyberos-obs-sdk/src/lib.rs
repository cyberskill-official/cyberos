//! cyberos-obs-sdk - shared RED (rate / errors / duration) metrics for every CyberOS service
//! (FR-OBS-003).
//!
//! Slice 1 (this crate) ships the metric core: `red::record_request` plus the cardinality guard, built
//! on the OTel 0.27 metrics API and the global meter. The service installs the OTel SDK provider and
//! OTLP exporter at boot (see ai-gateway's `otel/init.rs`); this crate only builds the instruments and
//! records to them. The `#[red_instrument]` proc macro, the per-service application, and the
//! completeness lint land in the next slice. See `docs/feature-requests/obs/FR-OBS-003-red-metrics.md`.

pub mod cardinality_guard;
pub mod exemplar;
pub mod layer;
pub mod red;
pub mod tracecontext;

pub use exemplar::record_with_exemplar;
pub use layer::{red_mw, RedState, TenantCtx};
pub use red::{
    init, record_request, record_tracecontext_extracted, status_class, HISTOGRAM_BUCKETS_MS,
};
pub use tracecontext::{
    extract_traceparent, format_traceparent, hash16, inject_traceparent, parse_w3c_traceparent,
    ExtractError, TraceContext,
};
