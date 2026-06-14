use cyberos_obs_sdk::red::{self, TRACECONTEXT_EXTRACTED_TOTAL};
use cyberos_obs_sdk::tracecontext::{self, ExtractOutcome};

#[test]
fn traceparent_parses_strictly_and_lowercases() {
    let parsed =
        tracecontext::parse_traceparent("00-4BF92F3577B34DA6A3CE929D0E0E4736-00F067AA0BA902B7-01")
            .unwrap();

    assert_eq!(parsed.trace_id, "4bf92f3577b34da6a3ce929d0e0e4736");
    assert_eq!(parsed.parent_span_id, "00f067aa0ba902b7");
    assert_eq!(parsed.trace_flags, "01");
}

#[test]
fn traceparent_rejects_bad_shape_and_zero_ids() {
    for value in [
        "totally-bogus",
        "00-00000000000000000000000000000000-00f067aa0ba902b7-01",
        "00-4bf92f3577b34da6a3ce929d0e0e4736-0000000000000000-01",
        "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-zz",
    ] {
        assert!(
            tracecontext::parse_traceparent(value).is_err(),
            "{value} should be rejected"
        );
    }
}

#[test]
fn extraction_records_outcome_metrics() {
    red::reset_for_tests();

    let mut headers = http::HeaderMap::new();
    let generated = tracecontext::extract_or_generate(&headers);
    assert_eq!(generated.outcome, ExtractOutcome::MissingGeneratedNew);
    assert!(tracecontext::is_valid_trace_id(&generated.context.trace_id));

    headers.insert(
        "traceparent",
        http::HeaderValue::from_static("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"),
    );
    let extracted = tracecontext::extract_or_generate(&headers);
    assert_eq!(extracted.outcome, ExtractOutcome::Extracted);
    assert_eq!(
        extracted.context.trace_id,
        "4bf92f3577b34da6a3ce929d0e0e4736"
    );
    assert_eq!(
        extracted.context.parent_span_id.as_deref(),
        Some("00f067aa0ba902b7")
    );

    headers.insert("traceparent", http::HeaderValue::from_static("bad"));
    let malformed = tracecontext::extract_or_generate(&headers);
    assert_eq!(malformed.outcome, ExtractOutcome::Malformed);
    assert!(malformed.malformed_hash16.is_some());

    let snapshot = red::snapshot();
    assert_eq!(
        snapshot.counter_value(
            TRACECONTEXT_EXTRACTED_TOTAL,
            &[("outcome", "missing_generated_new")]
        ),
        1
    );
    assert_eq!(
        snapshot.counter_value(TRACECONTEXT_EXTRACTED_TOTAL, &[("outcome", "extracted")]),
        1
    );
    assert_eq!(
        snapshot.counter_value(TRACECONTEXT_EXTRACTED_TOTAL, &[("outcome", "malformed")]),
        1
    );
}

#[test]
fn inject_traceparent_uses_local_span_id() {
    let context = tracecontext::TraceContext {
        trace_id: "4bf92f3577b34da6a3ce929d0e0e4736".into(),
        span_id: "1111111111111111".into(),
        parent_span_id: Some("00f067aa0ba902b7".into()),
        trace_flags: "01".into(),
    };
    let mut headers = http::HeaderMap::new();

    tracecontext::inject_traceparent(&mut headers, &context);

    assert_eq!(
        headers
            .get("traceparent")
            .and_then(|value| value.to_str().ok()),
        Some("00-4bf92f3577b34da6a3ce929d0e0e4736-1111111111111111-01")
    );
}
