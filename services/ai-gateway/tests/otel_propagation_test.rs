//! FR-AI-022 §5 — W3C TraceContext propagation tests.

use cyberos_ai_gateway::otel::propagation;
use opentelemetry::trace::{SpanContext, SpanId, TraceContextExt, TraceFlags, TraceId, TraceState};
use opentelemetry::Context;

#[test]
fn extract_empty_headers_returns_default_context() {
    let headers = http::HeaderMap::new();
    let _ctx = propagation::extract_context_from_headers(&headers);
    // Should not panic on empty headers.
}

#[test]
fn inject_then_extract_roundtrip() {
    // Build a context with a known SpanContext directly.
    let span_ctx = SpanContext::new(
        TraceId::from_bytes(0x0102030405060708090a0b0c0d0e0f10u128.to_be_bytes()),
        SpanId::from_bytes(0x0102030405060708u64.to_be_bytes()),
        TraceFlags::SAMPLED,
        false,
        TraceState::default(),
    );
    let ctx = Context::current().with_remote_span_context(span_ctx);

    // Inject into headers.
    let mut headers = http::HeaderMap::new();
    propagation::inject_context_into_headers(&ctx, &mut headers);

    // Verify traceparent header was injected.
    let tp = headers.get("traceparent").and_then(|v| v.to_str().ok());
    assert!(
        tp.is_some(),
        "traceparent header should be injected; headers: {:?}",
        headers.keys().collect::<Vec<_>>()
    );
    let tp = tp.unwrap();
    assert!(
        tp.starts_with("00-"),
        "traceparent should start with version 00, got: {tp}"
    );
    assert!(
        tp.contains("0102030405060708090a0b0c0d0e0f10"),
        "traceparent should contain our trace_id"
    );

    // Extract back.
    let extracted = propagation::extract_context_from_headers(&headers);
    // The extraction should succeed without panicking.
    drop(extracted);
}

#[test]
fn extract_malformed_traceparent_is_safe() {
    let mut headers = http::HeaderMap::new();
    headers.insert(
        "traceparent",
        http::HeaderValue::from_static("totally-bogus"),
    );
    // Should not panic — malformed header is safely ignored.
    let _ctx = propagation::extract_context_from_headers(&headers);
}
