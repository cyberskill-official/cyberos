mod support;

use cyberos_obs_router::severity::Route;
use support::{test_state, webhook, MockChat, MockCuo, MockPagerDuty, RecordingAudit};

#[tokio::test]
async fn sev1_routes_to_both_regardless_of_confidence() {
    let chat = MockChat::new();
    let pd = MockPagerDuty::new();
    let state = test_state(
        MockCuo::confidence(0.99),
        chat.clone(),
        pd.clone(),
        std::sync::Arc::new(RecordingAudit::default()),
    );

    let reports = state
        .process_webhook(serde_json::from_value(webhook("P1", "DatabaseDown", "fp-sev1")).unwrap())
        .await
        .unwrap();

    assert_eq!(reports[0].decision_route, Some(Route::Both));
    assert_eq!(reports[0].actual_route, Some(Route::Both));
    assert_eq!(chat.posts.lock().unwrap().len(), 1);
    assert_eq!(pd.incidents.lock().unwrap().len(), 1);
}
