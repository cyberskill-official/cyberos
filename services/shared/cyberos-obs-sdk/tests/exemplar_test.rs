use cyberos_obs_sdk::red::{self, DURATION_MS, EXEMPLAR_EMISSION_TOTAL, REQUESTS_TOTAL};

#[test]
fn duration_histogram_captures_trace_exemplar_without_trace_label() {
    red::reset_for_tests();

    red::record_request_with_trace(
        "ai-gateway",
        "/v1/chat",
        "tenant-a",
        200,
        42,
        &[("model_alias", "chat.smart".to_string())],
        Some("4bf92f3577b34da6a3ce929d0e0e4736"),
    );

    let snapshot = red::snapshot();
    assert_eq!(
        snapshot.counter_value(
            REQUESTS_TOTAL,
            &[
                ("service", "ai-gateway"),
                ("route", "/v1/chat"),
                ("tenant_id", "tenant-a"),
                ("status_class", "2xx"),
                ("model_alias", "chat.smart"),
            ],
        ),
        1
    );

    let exemplars = snapshot.histogram_exemplars(
        DURATION_MS,
        &[
            ("service", "ai-gateway"),
            ("route", "/v1/chat"),
            ("tenant_id", "tenant-a"),
            ("model_alias", "chat.smart"),
        ],
    );
    assert_eq!(exemplars.len(), 1);
    assert_eq!(exemplars[0].trace_id, "4bf92f3577b34da6a3ce929d0e0e4736");
    assert_eq!(exemplars[0].value, 42.0);
    assert_eq!(
        snapshot.counter_value(EXEMPLAR_EMISSION_TOTAL, &[("service", "ai-gateway")]),
        1
    );
}

#[test]
fn invalid_trace_id_does_not_emit_exemplar() {
    red::reset_for_tests();

    red::record_request_with_trace(
        "ai-gateway",
        "/v1/chat",
        "tenant-a",
        200,
        42,
        &[],
        Some("not-a-trace"),
    );

    assert!(red::snapshot()
        .histogram_exemplars(
            DURATION_MS,
            &[
                ("service", "ai-gateway"),
                ("route", "/v1/chat"),
                ("tenant_id", "tenant-a"),
            ],
        )
        .is_empty());
}
