mod support;

use support::{test_state, webhook, MockChat, MockCuo, MockPagerDuty, RecordingAudit};

#[tokio::test]
async fn dedup_within_5min_window_updates_existing_chat_counter() {
    let chat = MockChat::new();
    let state = test_state(
        MockCuo::confidence(0.90),
        chat.clone(),
        MockPagerDuty::new(),
        std::sync::Arc::new(RecordingAudit::default()),
    );

    let payload = serde_json::from_value(webhook("P3", "DuplicateAlert", "fp-dedup")).unwrap();
    state.process_webhook(payload).await.unwrap();

    let payload = serde_json::from_value(webhook("P3", "DuplicateAlert", "fp-dedup")).unwrap();
    let second = state.process_webhook(payload).await.unwrap();

    let payload = serde_json::from_value(webhook("P3", "DuplicateAlert", "fp-dedup")).unwrap();
    state.process_webhook(payload).await.unwrap();

    assert!(second[0].deduped);
    assert_eq!(chat.posts.lock().unwrap().len(), 1);
    assert_eq!(state.metrics.dedup_total(), 2);
    assert_eq!(
        chat.dedup_updates.lock().unwrap().as_slice(),
        &[("chat-1".to_string(), 2), ("chat-1".to_string(), 3)]
    );
}
