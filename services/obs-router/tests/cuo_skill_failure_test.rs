mod support;

use cyberos_obs_router::severity::Route;
use support::{test_state, webhook, MockChat, MockCuo, MockPagerDuty, RecordingAudit};

#[tokio::test]
async fn cuo_timeout_falls_back_to_pagerduty() {
    let pd = MockPagerDuty::new();
    let state = test_state(
        MockCuo::timing_out(),
        MockChat::new(),
        pd.clone(),
        std::sync::Arc::new(RecordingAudit::default()),
    );

    let reports = state
        .process_webhook(serde_json::from_value(webhook("P2", "CuoSlow", "fp-timeout")).unwrap())
        .await
        .unwrap();

    assert_eq!(reports[0].actual_route, Some(Route::PagerDuty));
    assert!(reports[0].cuo_fallback);
    assert_eq!(state.metrics.cuo_timeouts_total(), 1);
    assert_eq!(pd.incidents.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn cuo_failure_falls_back_to_pagerduty() {
    let pd = MockPagerDuty::new();
    let state = test_state(
        MockCuo::failing(),
        MockChat::new(),
        pd.clone(),
        std::sync::Arc::new(RecordingAudit::default()),
    );

    let reports = state
        .process_webhook(serde_json::from_value(webhook("P2", "CuoError", "fp-cuo-fail")).unwrap())
        .await
        .unwrap();

    assert_eq!(reports[0].actual_route, Some(Route::PagerDuty));
    assert!(reports[0].cuo_fallback);
    assert_eq!(pd.incidents.lock().unwrap().len(), 1);
}
