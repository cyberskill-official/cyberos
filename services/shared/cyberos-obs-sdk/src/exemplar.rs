//! Histogram exemplar emission (FR-OBS-005 §1 #3).
//!
//! An exemplar links a histogram bucket to the trace that produced the sample, so a Grafana operator can
//! click a latency-spike bucket and jump straight to the offending trace in Tempo. With the OTel ->
//! Prometheus exporter, the exemplar's trace_id is taken from the current OTel context automatically -
//! there is no per-call trace_id argument here; the request's `trace_ctx` boundary has already put the
//! trace in context. This helper records the value and counts the emission (`obs_exemplar_emission_total`,
//! §1 #12) so the emission rate is itself observable.

use opentelemetry::metrics::Histogram;
use opentelemetry::KeyValue;

/// Record `value` on `histogram` with the standard labels and count the exemplar emission. The trace_id
/// rides via the current OTel context, so a sample taken inside a `trace_ctx`-wrapped request carries its
/// trace as an exemplar. A safe no-op before `init` (the counter is simply not yet built).
pub fn record_with_exemplar(histogram: &Histogram<f64>, value: f64, labels: &[KeyValue]) {
    histogram.record(value, labels);
    crate::red::note_exemplar_emission();
}
