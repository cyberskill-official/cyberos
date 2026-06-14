use cyberos_obs_sdk::red::{
    self, DURATION_MS, ERRORS_TOTAL, RECORD_OVERHEAD_TARGET_NS, REQUESTS_TOTAL,
    SDK_RECORD_CALLS_TOTAL, STANDARD_BUCKETS_MS,
};
use std::sync::{Mutex, MutexGuard};
use std::thread;

static TEST_LOCK: Mutex<()> = Mutex::new(());

fn test_lock() -> MutexGuard<'static, ()> {
    TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[test]
fn record_emits_request_error_and_duration_metrics() {
    let _guard = test_lock();
    red::reset_for_tests();

    let outcome = red::record_request(
        "ai-gateway",
        "/v1/chat",
        "org:cyberskill",
        503,
        7,
        &[("model_alias", "chat.smart".to_string())],
    );

    assert!(outcome.recorded);
    assert_eq!(outcome.status_class, "5xx");
    assert_eq!(outcome.error_class, Some("server_error"));

    let snapshot = red::snapshot();
    assert_eq!(
        snapshot.counter_value(
            REQUESTS_TOTAL,
            &[
                ("service", "ai-gateway"),
                ("route", "/v1/chat"),
                ("tenant_id", "org:cyberskill"),
                ("status_class", "5xx"),
                ("model_alias", "chat.smart"),
            ],
        ),
        1
    );
    assert_eq!(
        snapshot.counter_value(
            ERRORS_TOTAL,
            &[
                ("service", "ai-gateway"),
                ("route", "/v1/chat"),
                ("tenant_id", "org:cyberskill"),
                ("error_class", "server_error"),
                ("model_alias", "chat.smart"),
            ],
        ),
        1
    );
    assert_eq!(
        snapshot.histogram_bucket_value(
            DURATION_MS,
            &[
                ("service", "ai-gateway"),
                ("route", "/v1/chat"),
                ("tenant_id", "org:cyberskill"),
                ("model_alias", "chat.smart"),
            ],
            10.0,
        ),
        1
    );
}

#[test]
fn status_class_is_bounded() {
    assert_eq!(red::status_class(200), "2xx");
    assert_eq!(red::status_class(201), "2xx");
    assert_eq!(red::status_class(204), "2xx");
    assert_eq!(red::status_class(301), "3xx");
    assert_eq!(red::status_class(404), "4xx");
    assert_eq!(red::status_class(500), "5xx");
    assert_eq!(red::status_class(999), "other");
}

#[test]
fn client_errors_use_client_error_class() {
    let _guard = test_lock();
    red::reset_for_tests();
    red::record_request("auth-service", "/v1/auth/token", "tenant-a", 401, 5, &[]);
    assert_eq!(
        red::snapshot().counter_value(
            ERRORS_TOTAL,
            &[
                ("service", "auth-service"),
                ("route", "/v1/auth/token"),
                ("tenant_id", "tenant-a"),
                ("error_class", "client_error"),
            ],
        ),
        1
    );
}

#[test]
fn buckets_and_self_metrics_are_standardised() {
    let _guard = test_lock();
    red::reset_for_tests();
    assert_eq!(
        STANDARD_BUCKETS_MS,
        &[1.0, 2.5, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0, 10000.0]
    );
    assert!(RECORD_OVERHEAD_TARGET_NS <= 100);

    red::record_request("memory", "/healthz", "tenant-a", 200, 1, &[]);
    assert_eq!(
        red::snapshot().counter_value(SDK_RECORD_CALLS_TOTAL, &[("service", "memory")]),
        1
    );
}

#[test]
fn concurrent_record_request_is_thread_safe() {
    let _guard = test_lock();
    red::reset_for_tests();
    let handles: Vec<_> = (0..8)
        .map(|_| {
            thread::spawn(|| {
                for _ in 0..1000 {
                    red::record_request("mcp-gateway", "/mcp", "tenant-a", 200, 2, &[]);
                }
            })
        })
        .collect();
    for handle in handles {
        handle.join().expect("thread joins");
    }
    assert_eq!(
        red::snapshot().counter_value(
            REQUESTS_TOTAL,
            &[
                ("service", "mcp-gateway"),
                ("route", "/mcp"),
                ("tenant_id", "tenant-a"),
                ("status_class", "2xx"),
            ],
        ),
        8000
    );
}

#[test]
fn init_can_run_without_otlp_exporter_for_tests() {
    let _guard = test_lock();
    std::env::set_var("CYBEROS_OBS_SDK_DISABLE_OTLP", "1");
    assert!(red::init("init-test", "0.0.0").is_ok());
}
