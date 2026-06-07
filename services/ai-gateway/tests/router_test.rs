//! FR-AI-008 §5 — Integration tests for the multi-provider router.
//!
//! Tests use mock providers that return scripted responses to verify
//! retry, failover, deadline, and error handling behavior.

use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{HeaderMap, Response, StatusCode};
use axum::Router as AxumRouter;
use futures::StreamExt;

use cyberos_ai_gateway::alias::{LatencyClass, ResolvedModel};
use cyberos_ai_gateway::circuit_breaker::{self, clock::MockClock, CallOutcome};
use cyberos_ai_gateway::policy::{
    AiPolicy, EmergencyOverride, Provider as PolicyProvider, ProviderKind, Residency, TenantPolicy,
};
use cyberos_ai_gateway::router::anthropic::AnthropicProvider;
use cyberos_ai_gateway::router::bedrock::BedrockProvider;
use cyberos_ai_gateway::router::openai::OpenAIProvider;
use cyberos_ai_gateway::router::types::{Choice, FinishReason};
use cyberos_ai_gateway::router::{
    self, AttemptStatus, ChatCompleteRequest, EmbedRequest, EmbedResponse, Message, Provider,
    ProviderEndpoint, ProviderResponse, ProviderStreamResponse, ProviderUsage, RouterError,
};
use cyberos_ai_gateway::streaming::{ProviderStreamEvent, ProviderStreamUsage};

// ─── Mock infrastructure ─────────────────────────────────────────────────────

/// A scripted response sequence for a mock provider.
#[derive(Clone)]
enum ResponseScript {
    /// Always return this status.
    Always(u16),
    /// Return statuses in sequence (repeats last).
    Sequence(Vec<u16>),
    /// Return 200 after a delay.
    DelayedOk(Duration),
    /// Always return a retryable 429 with Retry-After.
    RetryAfter { status: u16, retry_after: u64 },
}

struct MockProvider {
    kind: ProviderKind,
    script: ResponseScript,
    call_count: Arc<std::sync::atomic::AtomicUsize>,
}

impl MockProvider {
    fn new(kind: ProviderKind, script: ResponseScript) -> Self {
        Self {
            kind,
            script,
            call_count: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }
}

#[async_trait]
impl Provider for MockProvider {
    fn kind(&self) -> ProviderKind {
        self.kind
    }

    async fn call_chat(
        &self,
        _req: &ChatCompleteRequest,
        _model: &str,
        _deadline: Instant,
    ) -> Result<ProviderResponse, RouterError> {
        let count = self
            .call_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let status = match &self.script {
            ResponseScript::Always(s) => *s,
            ResponseScript::Sequence(v) => {
                if count < v.len() {
                    v[count]
                } else {
                    *v.last().unwrap()
                }
            }
            ResponseScript::DelayedOk(d) => {
                tokio::time::sleep(*d).await;
                return Ok(make_ok_response(&format!(
                    "mock-{}",
                    self.kind.as_metric_label()
                )));
            }
            ResponseScript::RetryAfter {
                status,
                retry_after,
            } => {
                return Err(make_error_with_retry_after(
                    self.kind,
                    *status,
                    *retry_after,
                ));
            }
        };

        if status == 200 {
            Ok(make_ok_response(&format!(
                "mock-{}",
                self.kind.as_metric_label()
            )))
        } else {
            Err(make_error(self.kind, status))
        }
    }

    async fn call_embed(
        &self,
        _req: &EmbedRequest,
        _model: &str,
        _deadline: Instant,
    ) -> Result<EmbedResponse, RouterError> {
        Err(RouterError::InvalidResponse {
            reason: "mock embed not implemented".into(),
        })
    }

    async fn call_chat_streaming(
        &self,
        _req: &ChatCompleteRequest,
        _model: &str,
        _deadline: Instant,
    ) -> Result<ProviderStreamResponse, RouterError> {
        let count = self
            .call_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let status = match &self.script {
            ResponseScript::Always(s) => *s,
            ResponseScript::Sequence(v) => {
                if count < v.len() {
                    v[count]
                } else {
                    *v.last().unwrap()
                }
            }
            ResponseScript::DelayedOk(d) => {
                tokio::time::sleep(*d).await;
                return Ok(mock_stream_response());
            }
            ResponseScript::RetryAfter {
                status,
                retry_after,
            } => {
                return Err(make_error_with_retry_after(
                    self.kind,
                    *status,
                    *retry_after,
                ));
            }
        };

        if status == 200 {
            Ok(mock_stream_response())
        } else {
            Err(make_error(self.kind, status))
        }
    }
}

fn make_ok_response(id: &str) -> ProviderResponse {
    ProviderResponse {
        id: id.to_string(),
        usage: ProviderUsage {
            prompt_tokens: 100,
            completion_tokens: 50,
            cached_input_tokens: 0,
        },
        choices: vec![Choice {
            index: 0,
            content: "Hello from mock".to_string(),
            tool_calls: vec![],
            finish_reason: FinishReason::Stop,
        }],
        finish_reason: FinishReason::Stop,
        latency_ms: 100,
        cache_state: router::CacheState::None,
        attempts: vec![],
    }
}

fn make_error(kind: ProviderKind, status: u16) -> RouterError {
    match status {
        401 | 403 => RouterError::AuthError {
            provider: kind,
            status,
        },
        _ => RouterError::TerminalProviderError {
            provider: kind,
            status,
            message: format!("mock error {}", status),
            retry_after_secs: None,
        },
    }
}

fn make_error_with_retry_after(kind: ProviderKind, status: u16, retry_after: u64) -> RouterError {
    RouterError::TerminalProviderError {
        provider: kind,
        status,
        message: format!("mock error {}", status),
        retry_after_secs: Some(retry_after),
    }
}

fn mock_stream_response() -> ProviderStreamResponse {
    ProviderStreamResponse::new(futures::stream::iter(vec![
        Ok(ProviderStreamEvent::Token {
            text: "mock-token".to_string(),
        }),
        Ok(ProviderStreamEvent::Usage(ProviderStreamUsage {
            prompt_tokens: 10,
            completion_tokens: 1,
            cached_input_tokens: 0,
        })),
        Ok(ProviderStreamEvent::Done(FinishReason::Stop)),
    ]))
}

static PROVIDER_ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
static BREAKER_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn provider_env_lock() -> MutexGuard<'static, ()> {
    PROVIDER_ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn reset_breakers_for_router_test() -> (Arc<MockClock>, MutexGuard<'static, ()>) {
    let guard = BREAKER_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let clock = Arc::new(MockClock::new());
    circuit_breaker::init(Box::new(clock.clone()));
    circuit_breaker::reset_for_tests();
    circuit_breaker::swap_clock(Box::new(clock.clone()));
    (clock, guard)
}

async fn spawn_json_server(
    status: StatusCode,
    body: &'static str,
    retry_after: Option<&'static str>,
) -> (String, Arc<Mutex<Vec<HeaderMap>>>) {
    let captured_headers = Arc::new(Mutex::new(Vec::new()));
    let route_headers = Arc::clone(&captured_headers);
    let body = body.to_string();
    let retry_after = retry_after.map(str::to_string);

    let app = AxumRouter::new().fallback(move |headers: HeaderMap| {
        let route_headers = Arc::clone(&route_headers);
        let body = body.clone();
        let retry_after = retry_after.clone();
        async move {
            route_headers.lock().expect("capture mutex").push(headers);
            let mut builder = Response::builder()
                .status(status)
                .header("content-type", "application/json");
            if let Some(retry_after) = retry_after {
                builder = builder.header("retry-after", retry_after);
            }
            builder.body(Body::from(body)).expect("response")
        }
    });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://{}", addr), captured_headers)
}

async fn spawn_sse_server(
    status: StatusCode,
    body: &'static str,
    retry_after: Option<&'static str>,
) -> (String, Arc<Mutex<Vec<HeaderMap>>>) {
    let captured_headers = Arc::new(Mutex::new(Vec::new()));
    let route_headers = Arc::clone(&captured_headers);
    let body = body.to_string();
    let retry_after = retry_after.map(str::to_string);

    let app = AxumRouter::new().fallback(move |headers: HeaderMap| {
        let route_headers = Arc::clone(&route_headers);
        let body = body.clone();
        let retry_after = retry_after.clone();
        async move {
            route_headers.lock().expect("capture mutex").push(headers);
            let mut builder = Response::builder()
                .status(status)
                .header("content-type", "text/event-stream");
            if let Some(retry_after) = retry_after {
                builder = builder.header("retry-after", retry_after);
            }
            builder.body(Body::from(body)).expect("response")
        }
    });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://{}", addr), captured_headers)
}

async fn collect_provider_events(resp: ProviderStreamResponse) -> Vec<ProviderStreamEvent> {
    resp.into_events()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
}

// ─── Test helpers ─────────────────────────────────────────────────────────────

fn default_req() -> ChatCompleteRequest {
    ChatCompleteRequest {
        alias: "chat.smart".into(),
        messages: vec![Message {
            role: "user".into(),
            content: "Hello".into(),
        }],
        max_tokens: Some(100),
        temperature: Some(0.7),
        traceparent: None,
        tracestate: None,
    }
}

fn resolved_with_kind(kind: ProviderKind, model: &str) -> ResolvedModel {
    ResolvedModel {
        provider_kind: kind,
        region: Some("ap-southeast-1".into()),
        model: model.to_string(),
        fallback_position: 0,
        is_zdr: true,
        latency_class: LatencyClass::Standard,
    }
}

fn policy_no_fallbacks() -> TenantPolicy {
    TenantPolicy {
        tenant_id: "test-tenant".into(),
        ai_policy: AiPolicy {
            monthly_cap_usd: "100.00".parse().unwrap(),
            warn_threshold: 0.80,
            hard_stop: true,
            primary_provider: PolicyProvider::Bedrock {
                region: "ap-southeast-1".into(),
                model_alias_map: std::collections::HashMap::new(),
            },
            fallback_chain: vec![],
            call_timeout_seconds: 60,
            residency: Residency::Sg1,
            zdr_required: false,
            emergency_override: EmergencyOverride::default(),
            allowed_personas: None,
            alias_overrides: None,
            residency_requires_regional_provider: None,
            pii_redaction_extra: None,
            pii_allowlist: None,
        },
    }
}

/// Wrap a mock provider for use as a `dyn Provider` in the chain.
/// The router's `build_provider_chain` creates providers from policy,
/// but for testing we inject mocks directly.
///
/// Since `call_provider` uses `build_provider_chain` internally,
/// we test through the mock provider trait directly by testing
/// the retry/failover logic at a lower level.
///
/// For integration-level tests, we test the individual components:
/// - jitter bounds (unit tests in jitter.rs)
/// - Provider trait behavior (mock tests here)
/// - Router error handling (direct calls with mock chain)

// ─── Tests ────────────────────────────────────────────────────────────────────

/// AC #11: Jitter bounds — proptest in router_proptest.rs

/// Test that RouterError variants are constructed correctly.
#[test]
fn router_error_display() {
    let e = RouterError::DeadlineExceeded;
    assert!(format!("{}", e).contains("deadline"));

    let e = RouterError::AuthError {
        provider: ProviderKind::Bedrock,
        status: 401,
    };
    assert!(format!("{}", e).contains("401"));

    let e = RouterError::TerminalProviderError {
        provider: ProviderKind::Openai,
        status: 400,
        message: "bad request".into(),
        retry_after_secs: None,
    };
    assert!(format!("{}", e).contains("400"));
}

/// AC #16 / FR-AI-010: streaming retries before a provider stream is accepted.
#[tokio::test]
async fn call_provider_streaming_retries_before_first_token_then_succeeds() {
    let provider = MockProvider::new(
        ProviderKind::Bedrock,
        ResponseScript::Sequence(vec![503, 200]),
    );
    let calls = Arc::clone(&provider.call_count);

    let response = router::call_provider_streaming_with_chain(
        &default_req(),
        Instant::now() + Duration::from_secs(5),
        vec![ProviderEndpoint::new(Box::new(provider), "test-model", 0)],
    )
    .await
    .unwrap();

    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 2);
    assert_eq!(response.attempts().len(), 2);
    assert_eq!(
        response.attempts()[0].status,
        AttemptStatus::RetriedAfter5xx
    );
    assert_eq!(response.attempts()[1].status, AttemptStatus::Succeeded);

    let events = collect_provider_events(response).await;
    assert_eq!(
        events,
        vec![
            ProviderStreamEvent::Token {
                text: "mock-token".into()
            },
            ProviderStreamEvent::Usage(ProviderStreamUsage {
                prompt_tokens: 10,
                completion_tokens: 1,
                cached_input_tokens: 0,
            }),
            ProviderStreamEvent::Done(FinishReason::Stop),
        ]
    );
}

#[tokio::test]
async fn call_provider_streaming_fails_over_after_primary_exhausts() {
    let primary = MockProvider::new(ProviderKind::Bedrock, ResponseScript::Always(503));
    let fallback = MockProvider::new(ProviderKind::Anthropic, ResponseScript::Always(200));
    let primary_calls = Arc::clone(&primary.call_count);
    let fallback_calls = Arc::clone(&fallback.call_count);

    let response = router::call_provider_streaming_with_chain(
        &default_req(),
        Instant::now() + Duration::from_secs(6),
        vec![
            ProviderEndpoint::new(Box::new(primary), "primary-model", 0),
            ProviderEndpoint::new(Box::new(fallback), "fallback-model", 1),
        ],
    )
    .await
    .unwrap();

    assert_eq!(primary_calls.load(std::sync::atomic::Ordering::SeqCst), 3);
    assert_eq!(fallback_calls.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert_eq!(response.attempts().len(), 4);
    assert_eq!(response.attempts()[2].status, AttemptStatus::FailedOver);
    assert_eq!(response.attempts()[3].status, AttemptStatus::Succeeded);
}

/// Test mock provider returns success.
#[tokio::test]
async fn mock_provider_success() {
    let mock = MockProvider::new(ProviderKind::Bedrock, ResponseScript::Always(200));
    let resp = mock
        .call_chat(
            &default_req(),
            "test-model",
            Instant::now() + Duration::from_secs(30),
        )
        .await
        .unwrap();
    assert_eq!(resp.id, "mock-bedrock");
    assert_eq!(resp.usage.prompt_tokens, 100);
}

/// Test mock provider returns error.
#[tokio::test]
async fn mock_provider_503() {
    let mock = MockProvider::new(ProviderKind::Bedrock, ResponseScript::Always(503));
    let err = mock
        .call_chat(
            &default_req(),
            "test-model",
            Instant::now() + Duration::from_secs(30),
        )
        .await
        .unwrap_err();
    match err {
        RouterError::TerminalProviderError { status, .. } => assert_eq!(status, 503),
        other => panic!("expected TerminalProviderError, got: {:?}", other),
    }
}

/// Test mock provider sequence.
#[tokio::test]
async fn mock_provider_sequence() {
    let mock = MockProvider::new(
        ProviderKind::Bedrock,
        ResponseScript::Sequence(vec![503, 200]),
    );
    // First call: 503
    let err = mock
        .call_chat(
            &default_req(),
            "test-model",
            Instant::now() + Duration::from_secs(30),
        )
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        RouterError::TerminalProviderError { status: 503, .. }
    ));
    // Second call: 200
    let resp = mock
        .call_chat(
            &default_req(),
            "test-model",
            Instant::now() + Duration::from_secs(30),
        )
        .await
        .unwrap();
    assert_eq!(resp.id, "mock-bedrock");
}

/// AC #4: 400 is terminal.
#[tokio::test]
async fn mock_provider_400_terminal() {
    let mock = MockProvider::new(ProviderKind::Bedrock, ResponseScript::Always(400));
    let err = mock
        .call_chat(
            &default_req(),
            "test-model",
            Instant::now() + Duration::from_secs(30),
        )
        .await
        .unwrap_err();
    match err {
        RouterError::TerminalProviderError { status: 400, .. } => {}
        other => panic!("expected 400 terminal, got: {:?}", other),
    }
}

/// AC #5: 401 is auth error.
#[tokio::test]
async fn mock_provider_401_auth_error() {
    let mock = MockProvider::new(ProviderKind::Bedrock, ResponseScript::Always(401));
    let err = mock
        .call_chat(
            &default_req(),
            "test-model",
            Instant::now() + Duration::from_secs(30),
        )
        .await
        .unwrap_err();
    match err {
        RouterError::AuthError {
            provider: ProviderKind::Bedrock,
            status: 401,
        } => {}
        other => panic!("expected AuthError 401, got: {:?}", other),
    }
}

/// AC #6: 404 is terminal.
#[tokio::test]
async fn mock_provider_404_terminal() {
    let mock = MockProvider::new(ProviderKind::Bedrock, ResponseScript::Always(404));
    let err = mock
        .call_chat(
            &default_req(),
            "test-model",
            Instant::now() + Duration::from_secs(30),
        )
        .await
        .unwrap_err();
    match err {
        RouterError::TerminalProviderError { status: 404, .. } => {}
        other => panic!("expected 404 terminal, got: {:?}", other),
    }
}

/// Test delayed response works.
#[tokio::test]
async fn mock_provider_delayed_ok() {
    let mock = MockProvider::new(
        ProviderKind::Bedrock,
        ResponseScript::DelayedOk(Duration::from_millis(50)),
    );
    let resp = mock
        .call_chat(
            &default_req(),
            "test-model",
            Instant::now() + Duration::from_secs(30),
        )
        .await
        .unwrap();
    assert_eq!(resp.id, "mock-bedrock");
}

/// AC #10: Audit metadata — ProviderResponse carries attempts.
#[test]
fn provider_response_has_attempts_field() {
    let resp = make_ok_response("test");
    // Initially empty — the router fills this.
    assert!(resp.attempts.is_empty());
}

/// Test AttemptStatus variants.
#[test]
fn attempt_status_variants() {
    // Verify all variants exist and are debuggable.
    let _ = format!("{:?}", AttemptStatus::Succeeded);
    let _ = format!("{:?}", AttemptStatus::RetriedAfter5xx);
    let _ = format!("{:?}", AttemptStatus::RetriedAfter429);
    let _ = format!("{:?}", AttemptStatus::RetriedAfterTimeout);
    let _ = format!("{:?}", AttemptStatus::RetriedAfterConnReset);
    let _ = format!("{:?}", AttemptStatus::FailedOver);
    let _ = format!("{:?}", AttemptStatus::Terminal400);
    let _ = format!("{:?}", AttemptStatus::Terminal404);
    let _ = format!("{:?}", AttemptStatus::TerminalAuth);
    let _ = format!("{:?}", AttemptStatus::TimeoutBeforeFirstToken);
    let _ = format!("{:?}", AttemptStatus::DeadlineExceededMidCall);
}

/// Test FinishReason variants.
#[test]
fn finish_reason_variants() {
    let _ = format!("{:?}", FinishReason::Stop);
    let _ = format!("{:?}", FinishReason::Length);
    let _ = format!("{:?}", FinishReason::ToolCalls);
    let _ = format!("{:?}", FinishReason::ContentFilter);
    let _ = format!("{:?}", FinishReason::Other);
}

/// Test that ProviderUsage is Copy.
#[test]
fn provider_usage_is_copy() {
    let u = ProviderUsage {
        prompt_tokens: 10,
        completion_tokens: 20,
        cached_input_tokens: 0,
    };
    let u2 = u;
    assert_eq!(u, u2);
}

/// AC #3: Retry transient 5xx failures before success.
#[tokio::test]
async fn call_provider_retries_transient_5xx_then_succeeds() {
    let provider = MockProvider::new(
        ProviderKind::Bedrock,
        ResponseScript::Sequence(vec![503, 200]),
    );
    let calls = Arc::clone(&provider.call_count);
    let response = router::call_provider_with_chain(
        &default_req(),
        Instant::now() + Duration::from_secs(5),
        vec![ProviderEndpoint::new(
            Box::new(provider),
            "anthropic.claude-3-5-sonnet-20241022-v2:0",
            0,
        )],
    )
    .await
    .unwrap();

    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 2);
    assert_eq!(response.attempts.len(), 2);
    assert_eq!(response.attempts[0].status, AttemptStatus::RetriedAfter5xx);
    assert_eq!(response.attempts[1].status, AttemptStatus::Succeeded);
}

/// AC #5/#12: Exhausted primary fails over and preserves attempt metadata.
#[tokio::test]
async fn call_provider_fails_over_after_primary_exhausts_retries() {
    let primary = MockProvider::new(ProviderKind::Bedrock, ResponseScript::Always(503));
    let fallback = MockProvider::new(ProviderKind::Anthropic, ResponseScript::Always(200));
    let primary_calls = Arc::clone(&primary.call_count);
    let fallback_calls = Arc::clone(&fallback.call_count);

    let response = router::call_provider_with_chain(
        &default_req(),
        Instant::now() + Duration::from_secs(6),
        vec![
            ProviderEndpoint::new(
                Box::new(primary),
                "anthropic.claude-3-5-sonnet-20241022-v2:0",
                0,
            ),
            ProviderEndpoint::new(Box::new(fallback), "claude-3-5-sonnet-20241022", 1),
        ],
    )
    .await
    .unwrap();

    assert_eq!(primary_calls.load(std::sync::atomic::Ordering::SeqCst), 3);
    assert_eq!(fallback_calls.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert_eq!(response.attempts.len(), 4);
    assert_eq!(response.attempts[2].status, AttemptStatus::FailedOver);
    assert_eq!(response.attempts[3].fallback_position, 1);
    assert_eq!(response.attempts[3].status, AttemptStatus::Succeeded);
}

/// AC #7/#8/#9: Terminal provider errors do not retry or fail over.
#[tokio::test]
async fn call_provider_terminal_400_does_not_retry_or_failover() {
    let primary = MockProvider::new(ProviderKind::Bedrock, ResponseScript::Always(400));
    let fallback = MockProvider::new(ProviderKind::Anthropic, ResponseScript::Always(200));
    let primary_calls = Arc::clone(&primary.call_count);
    let fallback_calls = Arc::clone(&fallback.call_count);

    let err = router::call_provider_with_chain(
        &default_req(),
        Instant::now() + Duration::from_secs(5),
        vec![
            ProviderEndpoint::new(Box::new(primary), "bad-model", 0),
            ProviderEndpoint::new(Box::new(fallback), "fallback-model", 1),
        ],
    )
    .await
    .unwrap_err();

    assert!(matches!(
        err,
        RouterError::TerminalProviderError { status: 400, .. }
    ));
    assert_eq!(primary_calls.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert_eq!(fallback_calls.load(std::sync::atomic::Ordering::SeqCst), 0);
}

#[tokio::test]
async fn call_provider_auth_error_does_not_retry_or_failover() {
    let primary = MockProvider::new(ProviderKind::Openai, ResponseScript::Always(401));
    let fallback = MockProvider::new(ProviderKind::Anthropic, ResponseScript::Always(200));
    let primary_calls = Arc::clone(&primary.call_count);
    let fallback_calls = Arc::clone(&fallback.call_count);

    let err = router::call_provider_with_chain(
        &default_req(),
        Instant::now() + Duration::from_secs(5),
        vec![
            ProviderEndpoint::new(Box::new(primary), "gpt-4o", 0),
            ProviderEndpoint::new(Box::new(fallback), "claude-3-5-sonnet-20241022", 1),
        ],
    )
    .await
    .unwrap_err();

    assert!(matches!(
        err,
        RouterError::AuthError {
            provider: ProviderKind::Openai,
            status: 401
        }
    ));
    assert_eq!(primary_calls.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert_eq!(fallback_calls.load(std::sync::atomic::Ordering::SeqCst), 0);
}

/// AC #10: Retry-After beyond budget fails over immediately.
#[tokio::test]
async fn retry_after_past_budget_fails_over_without_sleeping() {
    let primary = MockProvider::new(
        ProviderKind::Bedrock,
        ResponseScript::RetryAfter {
            status: 429,
            retry_after: 60,
        },
    );
    let fallback = MockProvider::new(ProviderKind::Anthropic, ResponseScript::Always(200));
    let primary_calls = Arc::clone(&primary.call_count);
    let fallback_calls = Arc::clone(&fallback.call_count);

    let response = router::call_provider_with_chain(
        &default_req(),
        Instant::now() + Duration::from_secs(5),
        vec![
            ProviderEndpoint::new(
                Box::new(primary),
                "anthropic.claude-3-5-sonnet-20241022-v2:0",
                0,
            ),
            ProviderEndpoint::new(Box::new(fallback), "claude-3-5-sonnet-20241022", 1),
        ],
    )
    .await
    .unwrap();

    assert_eq!(primary_calls.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert_eq!(fallback_calls.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert_eq!(response.attempts[0].status, AttemptStatus::FailedOver);
    assert_eq!(response.attempts[1].status, AttemptStatus::Succeeded);
}

/// AC #15: Caller deadline is enforced during provider calls.
#[tokio::test]
async fn deadline_elapsed_mid_call_returns_deadline_exceeded() {
    let provider = MockProvider::new(
        ProviderKind::Bedrock,
        ResponseScript::DelayedOk(Duration::from_millis(100)),
    );
    let err = router::call_provider_with_chain(
        &default_req(),
        Instant::now() + Duration::from_millis(10),
        vec![ProviderEndpoint::new(Box::new(provider), "slow-model", 0)],
    )
    .await
    .unwrap_err();

    assert!(matches!(err, RouterError::DeadlineExceeded));
}

/// AC #12: A resolved fallback position is not collapsed back to 0.
#[test]
fn build_provider_chain_preserves_resolved_fallback_position() {
    let (_clock, _guard) = reset_breakers_for_router_test();
    let mut policy = policy_no_fallbacks();
    let mut fallback_map = std::collections::HashMap::new();
    fallback_map.insert(
        "chat.long".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
    );
    policy.ai_policy.fallback_chain = vec![PolicyProvider::Anthropic {
        model_alias_map: fallback_map,
    }];
    let mut resolved = resolved_with_kind(ProviderKind::Anthropic, "claude-3-5-sonnet-20241022");
    resolved.fallback_position = 1;

    let chain = router::failover::build_provider_chain(&resolved, &policy, "chat.long");
    assert_eq!(chain.len(), 1);
    assert_eq!(chain[0].provider_kind(), ProviderKind::Anthropic);
    assert_eq!(chain[0].fallback_position(), 1);
}

#[test]
fn build_provider_chain_skips_open_breaker() {
    let (_clock, _guard) = reset_breakers_for_router_test();
    let model = "anthropic.claude-3-5-sonnet-20241022-v2:0";
    for _ in 0..5 {
        circuit_breaker::record_outcome(&ProviderKind::Bedrock, model, CallOutcome::Failure5xx);
    }

    let mut policy = policy_no_fallbacks();
    let mut fallback_map = std::collections::HashMap::new();
    fallback_map.insert(
        "chat.smart".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
    );
    policy.ai_policy.fallback_chain = vec![PolicyProvider::Anthropic {
        model_alias_map: fallback_map,
    }];
    let resolved = resolved_with_kind(ProviderKind::Bedrock, model);

    let chain = router::failover::build_provider_chain(&resolved, &policy, "chat.smart");
    assert_eq!(chain.len(), 1);
    assert_eq!(chain[0].provider_kind(), ProviderKind::Anthropic);
    assert_eq!(chain[0].model(), "claude-3-5-sonnet-20241022");
    assert_eq!(chain[0].fallback_position(), 1);
}

#[tokio::test]
async fn call_provider_records_retryable_failures_to_breaker() {
    let (_clock, _guard) = reset_breakers_for_router_test();
    let model = "breaker-record-model";

    for _ in 0..2 {
        let provider = MockProvider::new(ProviderKind::Bedrock, ResponseScript::Always(503));
        let err = router::call_provider_with_chain(
            &default_req(),
            Instant::now() + Duration::from_secs(4),
            vec![ProviderEndpoint::new(Box::new(provider), model, 0)],
        )
        .await
        .unwrap_err();
        assert!(matches!(err, RouterError::AllProvidersFailed { .. }));
    }

    assert!(circuit_breaker::is_open(&ProviderKind::Bedrock, model));
}

/// AC #11/#17: OpenAI response normalization and trace propagation.
#[tokio::test]
async fn openai_provider_normalizes_response_and_propagates_trace_headers() {
    let _guard = provider_env_lock();
    let body = r#"{
        "id": "chatcmpl-test",
        "usage": { "prompt_tokens": 11, "completion_tokens": 7 },
        "choices": [
          { "index": 0, "message": { "content": "hello" }, "finish_reason": "stop" }
        ]
    }"#;
    let (base_url, captured) = spawn_json_server(StatusCode::OK, body, None).await;
    std::env::set_var("CYBEROS_AI_GATEWAY_OPENAI_BASE_URL", base_url);
    std::env::set_var("CYBEROS_AI_GATEWAY_OPENAI_API_KEY", "test-key");

    let mut req = default_req();
    req.traceparent = Some("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01".into());
    req.tracestate = Some("vendor=value".into());

    let response = OpenAIProvider
        .call_chat(&req, "gpt-4o", Instant::now() + Duration::from_secs(5))
        .await
        .unwrap();

    assert_eq!(response.id, "chatcmpl-test");
    assert_eq!(response.usage.prompt_tokens, 11);
    assert_eq!(response.usage.completion_tokens, 7);
    assert_eq!(response.choices[0].content, "hello");

    let headers = captured.lock().unwrap();
    let first = headers.first().expect("captured request");
    assert_eq!(
        first
            .get("traceparent")
            .and_then(|value| value.to_str().ok()),
        req.traceparent.as_deref()
    );
    assert_eq!(
        first
            .get("tracestate")
            .and_then(|value| value.to_str().ok()),
        req.tracestate.as_deref()
    );
}

/// AC #10: Provider impls parse Retry-After from headers, not response text.
#[tokio::test]
async fn openai_provider_maps_retry_after_header() {
    let _guard = provider_env_lock();
    let (base_url, _captured) = spawn_json_server(
        StatusCode::TOO_MANY_REQUESTS,
        r#"{"error":"slow down"}"#,
        Some("2"),
    )
    .await;
    std::env::set_var("CYBEROS_AI_GATEWAY_OPENAI_BASE_URL", base_url);
    std::env::set_var("CYBEROS_AI_GATEWAY_OPENAI_API_KEY", "test-key");

    let err = OpenAIProvider
        .call_chat(
            &default_req(),
            "gpt-4o",
            Instant::now() + Duration::from_secs(5),
        )
        .await
        .unwrap_err();

    match err {
        RouterError::TerminalProviderError {
            status: 429,
            retry_after_secs: Some(2),
            ..
        } => {}
        other => panic!("expected 429 with retry-after, got {other:?}"),
    }
}

#[tokio::test]
async fn anthropic_provider_normalizes_message_response() {
    let _guard = provider_env_lock();
    let body = r#"{
        "id": "msg-test",
        "content": [{ "type": "text", "text": "anthropic hello" }],
        "usage": { "input_tokens": 13, "output_tokens": 8 },
        "stop_reason": "end_turn"
    }"#;
    let (base_url, _captured) = spawn_json_server(StatusCode::OK, body, None).await;
    std::env::set_var("CYBEROS_AI_GATEWAY_ANTHROPIC_BASE_URL", base_url);
    std::env::set_var("CYBEROS_AI_GATEWAY_ANTHROPIC_API_KEY", "test-key");

    let response = AnthropicProvider
        .call_chat(
            &default_req(),
            "claude-3-5-sonnet-20241022",
            Instant::now() + Duration::from_secs(5),
        )
        .await
        .unwrap();

    assert_eq!(response.id, "msg-test");
    assert_eq!(response.usage.prompt_tokens, 13);
    assert_eq!(response.usage.completion_tokens, 8);
    assert_eq!(response.choices[0].content, "anthropic hello");
    assert_eq!(response.finish_reason, FinishReason::Stop);
}

#[tokio::test]
async fn bedrock_provider_normalizes_message_response() {
    let _guard = provider_env_lock();
    let body = r#"{
        "id": "bedrock-test",
        "content": [{ "type": "text", "text": "bedrock hello" }],
        "usage": { "input_tokens": 17, "output_tokens": 9 },
        "stop_reason": "end_turn"
    }"#;
    let (base_url, _captured) = spawn_json_server(StatusCode::OK, body, None).await;
    std::env::set_var("CYBEROS_AI_GATEWAY_BEDROCK_BASE_URL", base_url);
    std::env::set_var("CYBEROS_AI_GATEWAY_BEDROCK_API_KEY", "test-key");

    let response = BedrockProvider
        .call_chat(
            &default_req(),
            "anthropic.claude-3-5-sonnet-20241022-v2:0",
            Instant::now() + Duration::from_secs(5),
        )
        .await
        .unwrap();

    assert_eq!(response.id, "bedrock-test");
    assert_eq!(response.usage.prompt_tokens, 17);
    assert_eq!(response.usage.completion_tokens, 9);
    assert_eq!(response.choices[0].content, "bedrock hello");
    assert_eq!(response.finish_reason, FinishReason::Stop);
}

#[tokio::test]
async fn openai_provider_streaming_normalizes_sse_and_propagates_trace_headers() {
    let _guard = provider_env_lock();
    let body = r#"data: {"choices":[{"delta":{"content":"Hel"}}]}

data: {"choices":[{"delta":{"content":"lo"}}]}

data: {"usage":{"prompt_tokens":4,"completion_tokens":2,"prompt_tokens_details":{"cached_tokens":1}},"choices":[]}

data: [DONE]

"#;
    let (base_url, captured) = spawn_sse_server(StatusCode::OK, body, None).await;
    std::env::set_var("CYBEROS_AI_GATEWAY_OPENAI_BASE_URL", base_url);
    std::env::set_var("CYBEROS_AI_GATEWAY_OPENAI_API_KEY", "test-key");

    let mut req = default_req();
    req.traceparent = Some("00-11111111111111111111111111111111-2222222222222222-01".into());

    let response = OpenAIProvider
        .call_chat_streaming(&req, "gpt-4o", Instant::now() + Duration::from_secs(5))
        .await
        .unwrap();
    let events = collect_provider_events(response).await;

    assert_eq!(
        events,
        vec![
            ProviderStreamEvent::Token { text: "Hel".into() },
            ProviderStreamEvent::Token { text: "lo".into() },
            ProviderStreamEvent::Usage(ProviderStreamUsage {
                prompt_tokens: 4,
                completion_tokens: 2,
                cached_input_tokens: 1,
            }),
            ProviderStreamEvent::Done(FinishReason::Stop),
        ]
    );

    let headers = captured.lock().unwrap();
    let first = headers.first().expect("captured request");
    assert_eq!(
        first
            .get("traceparent")
            .and_then(|value| value.to_str().ok()),
        req.traceparent.as_deref()
    );
}

#[tokio::test]
async fn anthropic_provider_streaming_normalizes_sse() {
    let _guard = provider_env_lock();
    let body = r#"event: message_start
data: {"type":"message_start","message":{"usage":{"input_tokens":5,"cache_read_input_tokens":1}}}

event: content_block_delta
data: {"type":"content_block_delta","delta":{"type":"text_delta","text":"hi"}}

event: message_delta
data: {"type":"message_delta","delta":{"stop_reason":"end_turn"},"usage":{"output_tokens":1}}

event: message_stop
data: {"type":"message_stop"}

"#;
    let (base_url, _captured) = spawn_sse_server(StatusCode::OK, body, None).await;
    std::env::set_var("CYBEROS_AI_GATEWAY_ANTHROPIC_BASE_URL", base_url);
    std::env::set_var("CYBEROS_AI_GATEWAY_ANTHROPIC_API_KEY", "test-key");

    let response = AnthropicProvider
        .call_chat_streaming(
            &default_req(),
            "claude-3-5-sonnet-20241022",
            Instant::now() + Duration::from_secs(5),
        )
        .await
        .unwrap();
    let events = collect_provider_events(response).await;

    assert_eq!(
        events,
        vec![
            ProviderStreamEvent::Token { text: "hi".into() },
            ProviderStreamEvent::Usage(ProviderStreamUsage {
                prompt_tokens: 5,
                completion_tokens: 1,
                cached_input_tokens: 1,
            }),
            ProviderStreamEvent::Done(FinishReason::Stop),
        ]
    );
}

#[tokio::test]
async fn bedrock_provider_streaming_normalizes_anthropic_family_sse() {
    let _guard = provider_env_lock();
    let body = r#"event: message_start
data: {"type":"message_start","message":{"usage":{"input_tokens":7}}}

event: content_block_delta
data: {"type":"content_block_delta","delta":{"type":"text_delta","text":"bed"}}

event: content_block_delta
data: {"type":"content_block_delta","delta":{"type":"text_delta","text":"rock"}}

event: message_delta
data: {"type":"message_delta","delta":{"stop_reason":"end_turn"},"usage":{"output_tokens":2}}

event: message_stop
data: {"type":"message_stop"}

"#;
    let (base_url, _captured) = spawn_sse_server(StatusCode::OK, body, None).await;
    std::env::set_var("CYBEROS_AI_GATEWAY_BEDROCK_BASE_URL", base_url);
    std::env::set_var("CYBEROS_AI_GATEWAY_BEDROCK_API_KEY", "test-key");

    let response = BedrockProvider
        .call_chat_streaming(
            &default_req(),
            "anthropic.claude-3-5-sonnet-20241022-v2:0",
            Instant::now() + Duration::from_secs(5),
        )
        .await
        .unwrap();
    let events = collect_provider_events(response).await;

    assert_eq!(
        events,
        vec![
            ProviderStreamEvent::Token { text: "bed".into() },
            ProviderStreamEvent::Token {
                text: "rock".into()
            },
            ProviderStreamEvent::Usage(ProviderStreamUsage {
                prompt_tokens: 7,
                completion_tokens: 2,
                cached_input_tokens: 0,
            }),
            ProviderStreamEvent::Done(FinishReason::Stop),
        ]
    );
}
