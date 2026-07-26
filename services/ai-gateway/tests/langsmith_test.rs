//! TASK-OBS-004 — LangSmith export integration tests (mock HTTP server).

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::{Json, Router};
use cyberos_ai_gateway::langsmith::{
    asserts_self_hosted, build_payload, export, post_with_retry, url_for_residency, ExportOutcome,
    LangSmithError, LangSmithMetadata, RedactedPrompt, RedactedResponse,
};
use cyberos_ai_gateway::policy::Residency;
use serde_json::Value;
use tokio::sync::oneshot;

#[derive(Clone, Default)]
struct MockState {
    status: Arc<Mutex<u16>>,
    received: Arc<Mutex<Vec<(HeaderMap, Value)>>>,
}

async fn capture(
    State(st): State<MockState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> StatusCode {
    st.received.lock().unwrap().push((headers, body));
    StatusCode::from_u16(*st.status.lock().unwrap()).unwrap_or(StatusCode::OK)
}

async fn start_mock(status: u16) -> (SocketAddr, MockState, oneshot::Sender<()>) {
    let st = MockState {
        status: Arc::new(Mutex::new(status)),
        received: Arc::new(Mutex::new(Vec::new())),
    };
    let app = Router::new()
        .route("/api/v1/traces", post(capture))
        .with_state(st.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (tx, rx) = oneshot::channel::<()>();
    tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                let _ = rx.await;
            })
            .await
            .ok();
    });
    (addr, st, tx)
}

fn meta(trace_id: &str) -> LangSmithMetadata {
    LangSmithMetadata {
        model_alias: "chat.smart".into(),
        resolved_model: "claude-3-5-sonnet".into(),
        provider: "anthropic".into(),
        temperature: Some(0.2),
        max_tokens: Some(1024),
        latency_ms: 42,
        cost_usd: 0.0012,
        persona_handle: "default".into(),
        tenant_id: "org:cyberskill".into(),
        trace_id: trace_id.into(),
        tool_calls: vec![],
    }
}

#[tokio::test]
async fn opt_in_tenant_produces_langsmith_trace() {
    let (addr, st, shutdown) = start_mock(200).await;
    let base = format!("http://{addr}");
    std::env::set_var("LANGSMITH_URL", &base);
    std::env::set_var("LANGSMITH_API_TOKEN", "test-token");

    let trace_id = "4bf92f3577b34da6a3ce929d0e0e4736";
    let outcome = export(
        true,
        Residency::Sg1,
        RedactedPrompt("hello".into()),
        RedactedResponse("hi".into()),
        meta(trace_id),
    );
    assert_eq!(outcome, ExportOutcome::Dispatched);
    tokio::time::sleep(Duration::from_millis(200)).await;

    let got = st.received.lock().unwrap().clone();
    assert_eq!(got.len(), 1);
    let (headers, body) = &got[0];
    assert_eq!(
        headers.get("idempotency-key").and_then(|v| v.to_str().ok()),
        Some(trace_id)
    );
    assert_eq!(body["trace_id"], trace_id);
    assert_eq!(body["prompt"], "hello");

    let _ = shutdown.send(());
    std::env::remove_var("LANGSMITH_URL");
}

#[tokio::test]
async fn opt_out_tenant_no_export() {
    let (addr, st, shutdown) = start_mock(200).await;
    std::env::set_var("LANGSMITH_URL", format!("http://{addr}"));

    let outcome = export(
        false,
        Residency::Sg1,
        RedactedPrompt("p".into()),
        RedactedResponse("r".into()),
        meta("t-opt-out"),
    );
    assert_eq!(outcome, ExportOutcome::DroppedOptOut);
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(st.received.lock().unwrap().len(), 0);

    let _ = shutdown.send(());
    std::env::remove_var("LANGSMITH_URL");
}

#[tokio::test]
async fn unreachable_langsmith_does_not_block_gateway() {
    std::env::remove_var("LANGSMITH_URL");
    // Dead port — export must return before the 2s client timeout because spawn is fire-and-forget.
    std::env::set_var("LANGSMITH_URL", "http://127.0.0.1:9");
    let t0 = std::time::Instant::now();
    let outcome = export(
        true,
        Residency::Sg1,
        RedactedPrompt("p".into()),
        RedactedResponse("r".into()),
        meta("t-dead"),
    );
    let elapsed = t0.elapsed();
    assert_eq!(outcome, ExportOutcome::Dispatched);
    assert!(
        elapsed < Duration::from_millis(50),
        "gateway blocked for {elapsed:?}"
    );
    std::env::remove_var("LANGSMITH_URL");
}

#[tokio::test]
async fn retries_3_times_on_500() {
    let (addr, st, shutdown) = start_mock(500).await;
    let base = format!("http://{addr}");
    std::env::set_var("LANGSMITH_API_TOKEN", "t");
    let payload = build_payload(
        RedactedPrompt("p".into()),
        RedactedResponse("r".into()),
        meta("retry-tid"),
    );
    let err = post_with_retry(&base, &payload)
        .await
        .expect_err("expected drop");
    assert!(matches!(
        err,
        LangSmithError::ServerError(500) | LangSmithError::DroppedAfterRetries
    ));
    assert_eq!(st.received.lock().unwrap().len(), 3);
    let _ = shutdown.send(());
}

#[tokio::test]
async fn auth_failed_drops_immediately_no_retry() {
    let (addr, st, shutdown) = start_mock(401).await;
    let base = format!("http://{addr}");
    let payload = build_payload(
        RedactedPrompt("p".into()),
        RedactedResponse("r".into()),
        meta("auth-tid"),
    );
    let err = post_with_retry(&base, &payload)
        .await
        .expect_err("expected AuthFailed");
    assert!(matches!(err, LangSmithError::AuthFailed));
    assert_eq!(st.received.lock().unwrap().len(), 1);
    let _ = shutdown.send(());
}

#[test]
fn region_map_and_self_hosted_assertion() {
    std::env::remove_var("LANGSMITH_URL");
    assert!(asserts_self_hosted(
        &url_for_residency(Residency::Sg1).unwrap()
    ));
    assert!(url_for_residency(Residency::Vn1).is_none());
    assert!(!asserts_self_hosted("https://api.smith.langchain.com"));
}

#[test]
fn truncation_at_100kb() {
    let big = RedactedPrompt("x".repeat(200_000));
    let payload = build_payload(big, RedactedResponse("r".into()), meta("trunc"));
    assert!(payload.prompt.len() <= 100 * 1024 + 50);
    assert!(payload.prompt.ends_with("...[truncated by TASK-OBS-004]"));
}
