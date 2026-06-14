use cyberos_obs_sdk::logging::{request_span, ObsContextLayer};
use cyberos_obs_sdk::red::{self, DURATION_MS};
use cyberos_obs_sdk::tracecontext;
use tracing_subscriber::prelude::*;

#[test]
fn synthetic_call_correlates_log_metric_trace_and_ai_trace_ids() {
    red::reset_for_tests();
    let mut headers = http::HeaderMap::new();
    headers.insert(
        "traceparent",
        http::HeaderValue::from_static("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"),
    );
    let extracted = tracecontext::extract_or_generate(&headers);
    let trace_id = extracted.context.trace_id.clone();

    let subscriber = tracing_subscriber::registry().with(ObsContextLayer::new("ai-gateway"));
    tracing::subscriber::with_default(subscriber, || {
        let span = request_span(
            "ai-gateway",
            "/v1/chat",
            "tenant-a",
            &trace_id,
            &extracted.context.span_id,
        );
        let _entered = span.enter();
        tracing::info!("synthetic ai call completed");
    });

    red::record_request_with_trace(
        "ai-gateway",
        "/v1/chat",
        "tenant-a",
        200,
        37,
        &[],
        Some(&trace_id),
    );

    let mut outgoing = http::HeaderMap::new();
    tracecontext::inject_traceparent(&mut outgoing, &extracted.context);
    let downstream = tracecontext::parse_traceparent(
        outgoing
            .get("traceparent")
            .and_then(|value| value.to_str().ok())
            .expect("outgoing traceparent"),
    )
    .expect("outgoing traceparent parses");

    let langsmith_trace_id = trace_id.clone();
    let metric_exemplar = red::snapshot()
        .histogram_exemplars(
            DURATION_MS,
            &[
                ("service", "ai-gateway"),
                ("route", "/v1/chat"),
                ("tenant_id", "tenant-a"),
            ],
        )
        .pop()
        .expect("duration exemplar");

    assert_eq!(downstream.trace_id, trace_id);
    assert_eq!(metric_exemplar.trace_id, trace_id);
    assert_eq!(langsmith_trace_id, trace_id);
}
