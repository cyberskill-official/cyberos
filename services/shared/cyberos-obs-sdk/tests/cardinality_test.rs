use cyberos_obs_sdk::red::{self, REQUESTS_TOTAL, SDK_CARDINALITY_BLOCKED_TOTAL};

#[test]
fn cardinality_guard_blocks_1001st_combo() {
    red::reset_for_tests();

    for i in 0..1000 {
        let outcome = red::record_request(
            "auth-service",
            "/v1/admin/subjects",
            &format!("tenant-{i}"),
            200,
            3,
            &[],
        );
        assert!(outcome.recorded, "combo {i} should record");
    }

    let blocked = red::record_request(
        "auth-service",
        "/v1/admin/subjects",
        "tenant-1000",
        200,
        3,
        &[],
    );
    assert!(!blocked.recorded);
    assert_eq!(blocked.blocked_metric.as_deref(), Some(REQUESTS_TOTAL));
    assert_eq!(
        red::snapshot().counter_value(
            SDK_CARDINALITY_BLOCKED_TOTAL,
            &[("service", "auth-service"), ("metric", REQUESTS_TOTAL)],
        ),
        1
    );
}
