//! Log enrichment: trace_id, span_id, and tenant_id on every log line (TASK-OBS-005 §1 #2).
//!
//! The correlation primitive is "show me everything that happened during this call" -
//! `loki: {trace_id="abc"}`. For that to work, every structured log line emitted while handling a request
//! must carry the request's trace_id. This module gives a service two pieces:
//!
//!   - [`request_span`] - the canonical `request` span carrying `trace_id` / `span_id` / `tenant_id`. A
//!     service instruments its request future with it (`.instrument(request_span(...))`), so every event
//!     logged inside the request inherits those fields through the span scope.
//!   - [`init_json_subscriber`] - a JSON `tracing` subscriber that renders the full span scope on every
//!     event. JSON is what Loki ingests, and emitting the span scope is what makes the inherited fields
//!     appear on each line. Without a span-rendering subscriber the fields exist on the span but never
//!     reach the log output, so the two are designed to be used together.
//!
//! This is the tracing-span route rather than the spec's custom OTel-context Layer: it needs no OTel
//! tracer provider, the field source is the span the request is already wrapped in, and the enrichment is
//! verifiable with an in-memory log capture (see the test). The id generation and the W3C parsing live in
//! `tracecontext`; this module only carries the resolved ids into the log context.

/// The canonical per-request span. Instrument the request future with it so every event logged while the
/// request is handled carries `trace_id`, `span_id`, and `tenant_id` (TASK-OBS-005 §1 #2, #6).
pub fn request_span(trace_id: &str, span_id: &str, tenant_id: &str) -> tracing::Span {
    tracing::info_span!(
        "request",
        trace_id = %trace_id,
        span_id = %span_id,
        tenant_id = %tenant_id,
    )
}

/// Install a global JSON `tracing` subscriber that renders the span scope on every event, so the
/// [`request_span`] fields appear on every log line. Honours `RUST_LOG` (defaults to `info`). Best-effort
/// and idempotent: if a global subscriber is already set, this is a no-op rather than a panic, so a
/// service that installs its own subscriber is not disrupted.
pub fn init_json_subscriber() {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    let fmt_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_current_span(true)
        .with_span_list(true);
    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .try_init();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::{Arc, Mutex};
    use tracing::subscriber::with_default;
    use tracing_subscriber::fmt::MakeWriter;
    use tracing_subscriber::layer::SubscriberExt;

    /// An in-memory writer so the test can read back what the subscriber emitted.
    #[derive(Clone, Default)]
    struct Buf(Arc<Mutex<Vec<u8>>>);

    impl Write for Buf {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(b);
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl<'a> MakeWriter<'a> for Buf {
        type Writer = Buf;
        fn make_writer(&'a self) -> Buf {
            self.clone()
        }
    }

    #[test]
    fn an_event_inside_the_request_span_carries_trace_id_and_tenant_id() {
        let buf = Buf::default();
        let layer = tracing_subscriber::fmt::layer()
            .json()
            .with_span_list(true)
            .with_writer(buf.clone());
        let subscriber = tracing_subscriber::registry().with(layer);

        with_default(subscriber, || {
            let span = request_span(
                "4bf92f3577b34da6a3ce929d0e0e4736",
                "00f067aa0ba902b7",
                "org:acme",
            );
            let _guard = span.enter();
            tracing::info!("handled a request");
        });

        let out = String::from_utf8(buf.0.lock().unwrap().clone()).unwrap();
        assert!(
            out.contains("4bf92f3577b34da6a3ce929d0e0e4736"),
            "every log line must carry the trace_id: {out}"
        );
        assert!(
            out.contains("org:acme"),
            "every log line must carry the tenant_id: {out}"
        );
    }
}
