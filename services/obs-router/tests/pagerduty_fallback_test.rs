mod support;

use cyberos_obs_router::severity::Route;
use support::{test_state, webhook, MockChat, MockCuo, MockPagerDuty, RecordingAudit};

#[tokio::test]
async fn low_confidence_routes_to_pagerduty() {
    let chat = MockChat::new();
    let pd = MockPagerDuty::new();
    let state = test_state(
        MockCuo::confidence(0.40),
        chat.clone(),
        pd.clone(),
        std::sync::Arc::new(RecordingAudit::default()),
    );

    let reports = state
        .process_webhook(serde_json::from_value(webhook("P3", "CacheMissBurst", "fp-pd")).unwrap())
        .await
        .unwrap();

    assert_eq!(reports[0].actual_route, Some(Route::PagerDuty));
    assert_eq!(pd.incidents.lock().unwrap().len(), 1);
    assert_eq!(chat.posts.lock().unwrap().len(), 0);
}

#[tokio::test]
async fn chat_failure_falls_back_to_pagerduty() {
    let chat = MockChat::failing_post();
    let pd = MockPagerDuty::new();
    let state = test_state(
        MockCuo::confidence(0.95),
        chat.clone(),
        pd.clone(),
        std::sync::Arc::new(RecordingAudit::default()),
    );

    let reports = state
        .process_webhook(
            serde_json::from_value(webhook("P2", "ChatFallback", "fp-chat-fail")).unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(reports[0].actual_route, Some(Route::PagerDuty));
    assert_eq!(reports[0].outcome, "chat_failed");
    assert_eq!(pd.incidents.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn pagerduty_failure_uses_emergency_chat() {
    let chat = MockChat::new();
    let pd = MockPagerDuty::failing_trigger();
    let state = test_state(
        MockCuo::confidence(0.20),
        chat.clone(),
        pd,
        std::sync::Arc::new(RecordingAudit::default()),
    );

    let reports = state
        .process_webhook(
            serde_json::from_value(webhook("P2", "PagerFailure", "fp-pd-fail")).unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(reports[0].actual_route, Some(Route::Chat));
    assert_eq!(reports[0].outcome, "pagerduty_failed");
    assert_eq!(chat.emergency.lock().unwrap().len(), 1);
}
