//! Prometheus exemplar capture types used by the RED SDK.

/// Label key used for Grafana trace exemplars.
pub const TRACE_ID_KEY: &str = "trace_id";

/// Deterministic in-process exemplar used by CI tests.
#[derive(Clone, Debug, PartialEq)]
pub struct HistogramExemplar {
    /// Trace id linked to the histogram observation.
    pub trace_id: String,
    /// Observed histogram value.
    pub value: f64,
}
