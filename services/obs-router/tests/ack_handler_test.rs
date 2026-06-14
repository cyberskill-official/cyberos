mod support;

use cyberos_obs_router::ack_handler::{handle_ack, handle_escalate};
use support::{test_state, webhook, MockChat, MockCuo, MockPagerDuty, RecordingAudit};

#[tokio::test]
async fn ack_button_closes_pagerduty_for_sev1() {
    let chat = MockChat::new();
    let pd = MockPagerDuty::new();
    let audit = std::sync::Arc::new(RecordingAudit::default());
    let state = test_state(
        MockCuo::confidence(0.80),
        chat.clone(),
        pd.clone(),
        audit.clone(),
    );

    state
        .process_webhook(serde_json::from_value(webhook("P1", "AckMe", "fp-ack")).unwrap())
        .await
        .unwrap();

    handle_ack(&state, "fp-ack", "alice@cyberos.world")
        .await
        .unwrap();

    assert_eq!(chat.acks.lock().unwrap()[0].1, "alice@cyberos.world");
    assert_eq!(pd.resolves.lock().unwrap()[0], "fp-ack");
    assert!(audit
        .rows
        .lock()
        .unwrap()
        .iter()
        .any(|row| row.kind == "obs.alert_acked"));
}

#[tokio::test]
async fn escalate_button_triggers_pagerduty_post_hoc() {
    let chat = MockChat::new();
    let pd = MockPagerDuty::new();
    let audit = std::sync::Arc::new(RecordingAudit::default());
    let state = test_state(MockCuo::confidence(0.95), chat, pd.clone(), audit.clone());

    state
        .process_webhook(
            serde_json::from_value(webhook("P2", "EscalateMe", "fp-escalate")).unwrap(),
        )
        .await
        .unwrap();

    handle_escalate(&state, "fp-escalate", "bob@cyberos.world")
        .await
        .unwrap();

    assert_eq!(pd.incidents.lock().unwrap().len(), 1);
    let rows = audit.rows.lock().unwrap();
    let row = rows
        .iter()
        .find(|row| row.kind == "obs.alert_escalated")
        .unwrap();
    assert_eq!(row.payload["escalated_from_chat"], true);
}
