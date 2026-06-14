mod support;

use axum::http::StatusCode;
use cyberos_obs_router::router::json_request;
use cyberos_obs_router::severity::Route;
use cyberos_obs_router::{app, RouterState};
use support::{test_state, webhook, MockChat, MockCuo, MockPagerDuty, RecordingAudit};
use tower::ServiceExt;

#[tokio::test]
async fn high_confidence_p2_routes_to_chat_only() {
    let cuo = MockCuo::confidence(0.85);
    let chat = MockChat::new();
    let pd = MockPagerDuty::new();
    let audit = std::sync::Arc::new(RecordingAudit::default());
    let state = test_state(cuo, chat.clone(), pd.clone(), audit.clone());

    let reports = state
        .process_webhook(
            serde_json::from_value(webhook("P2", "MemorySearchLatencyHigh", "fp-chat")).unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(reports[0].actual_route, Some(Route::Chat));
    assert_eq!(chat.posts.lock().unwrap().len(), 1);
    assert_eq!(pd.incidents.lock().unwrap().len(), 0);
    assert_eq!(audit.rows.lock().unwrap()[0].kind, "obs.alert_triaged");

    let post = &chat.posts.lock().unwrap()[0];
    assert!(post.title.contains("MemorySearchLatencyHigh"));
    assert!(post.title.contains("sev-2"));
    assert!(post.text.contains("Recent surge"));
    assert!(post.text.contains("index rebalance"));
    assert!(post.text.contains("Pause ingest"));
    assert!(post.text.contains("0af7651916cd43dd8448eb211c80319c"));
    assert!(post.buttons.iter().any(|b| b.action_id == "ack"));
    assert!(post.buttons.iter().any(|b| b.action_id == "escalate"));
}

#[tokio::test]
async fn webhook_secret_enforced() {
    let state: RouterState = test_state(
        MockCuo::confidence(0.80),
        MockChat::new(),
        MockPagerDuty::new(),
        std::sync::Arc::new(RecordingAudit::default()),
    );
    let app = app(state);

    let response = app
        .clone()
        .oneshot(json_request(
            "/alert",
            None,
            webhook("P2", "MemorySearchLatencyHigh", "fp-auth"),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let response = app
        .oneshot(json_request(
            "/alert",
            Some("secret"),
            webhook("P2", "MemorySearchLatencyHigh", "fp-auth"),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
