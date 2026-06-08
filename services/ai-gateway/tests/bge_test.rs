//! FR-AI-019 — BGE provider adapter contract tests.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU16, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::body::Body;
use axum::extract::State;
use axum::http::{Response, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router as AxumRouter};
use cyberos_ai_gateway::router::bge_provider::{load_sidecar_urls, BgeProvider};
use cyberos_ai_gateway::router::{EmbedRequest, EmbedTask, Provider, RouterError};
use serde_json::{json, Value};

#[derive(Clone)]
struct MockSidecarState {
    requests: Arc<Mutex<Vec<Value>>>,
    status: Arc<AtomicU16>,
    health_status: Arc<AtomicU16>,
    request_count: Arc<AtomicUsize>,
    devices: Arc<Mutex<Vec<String>>>,
}

#[derive(Clone)]
struct MockSidecar {
    url: String,
    state: MockSidecarState,
}

impl MockSidecar {
    async fn start() -> Self {
        let state = MockSidecarState {
            requests: Arc::new(Mutex::new(Vec::new())),
            status: Arc::new(AtomicU16::new(200)),
            health_status: Arc::new(AtomicU16::new(200)),
            request_count: Arc::new(AtomicUsize::new(0)),
            devices: Arc::new(Mutex::new(vec!["cuda".to_string()])),
        };
        let app = AxumRouter::new()
            .route("/health", get(mock_health))
            .route("/embed", post(mock_embed))
            .with_state(state.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        Self {
            url: format!("http://{addr}"),
            state,
        }
    }

    fn provider(&self) -> BgeProvider {
        let mut sidecars = HashMap::new();
        sidecars.insert("ap-southeast-1".to_string(), self.url.clone());
        BgeProvider::new(sidecars)
    }

    fn request_count(&self) -> usize {
        self.state.request_count.load(Ordering::SeqCst)
    }

    fn set_status(&self, status: StatusCode) {
        self.state.status.store(status.as_u16(), Ordering::SeqCst);
    }

    fn set_health_status(&self, status: StatusCode) {
        self.state
            .health_status
            .store(status.as_u16(), Ordering::SeqCst);
    }

    fn set_devices(&self, devices: Vec<&str>) {
        *self.state.devices.lock().unwrap() = devices.into_iter().map(str::to_string).collect();
    }
}

async fn mock_health(State(state): State<MockSidecarState>) -> Response<Body> {
    let status = StatusCode::from_u16(state.health_status.load(Ordering::SeqCst)).unwrap();
    let body = if status.is_success() {
        json!({
            "status": "ok",
            "device": "cuda",
            "sidecar_version": "1.0.0",
            "model_sha256": "0123456789abcdef"
        })
    } else {
        json!({"status": "warming"})
    };
    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

async fn mock_embed(
    State(state): State<MockSidecarState>,
    Json(body): Json<Value>,
) -> Response<Body> {
    state.request_count.fetch_add(1, Ordering::SeqCst);
    state.requests.lock().unwrap().push(body.clone());
    let status = StatusCode::from_u16(state.status.load(Ordering::SeqCst)).unwrap();
    if !status.is_success() {
        return Response::builder()
            .status(status)
            .header("content-type", "application/json")
            .body(Body::from(
                json!({"error":"input_too_long","max_tokens":8192,"actual_tokens":9000,"text_index":0})
                    .to_string(),
            ))
            .unwrap();
    }

    let texts = body["texts"].as_array().cloned().unwrap_or_default();
    let device = {
        let mut devices = state.devices.lock().unwrap();
        if devices.len() > 1 {
            devices.remove(0)
        } else {
            devices
                .first()
                .cloned()
                .unwrap_or_else(|| "cuda".to_string())
        }
    };
    let embeddings = vec![vec![0.0_f32; 1024]; texts.len()];
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "embeddings": embeddings,
                "model_name": "bge-m3",
                "model_sha256": "0123456789abcdef",
                "sidecar_version": "1.0.0",
                "device": device,
                "elapsed_ms": 7
            })
            .to_string(),
        ))
        .unwrap()
}

fn req(texts: Vec<String>, region: &str) -> EmbedRequest {
    EmbedRequest {
        texts,
        tenant_id: "tenant-a".to_string(),
        task: EmbedTask::Passage,
        region: region.to_string(),
    }
}

fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(5)
}

#[tokio::test]
async fn single_embed_returns_1024_dim_and_identity() {
    let sidecar = MockSidecar::start().await;
    let resp = sidecar
        .provider()
        .call_embed(
            &req(vec!["test".to_string()], "ap-southeast-1"),
            "bge-m3",
            deadline(),
        )
        .await
        .unwrap();

    assert_eq!(resp.embeddings.len(), 1);
    assert_eq!(resp.embeddings[0].len(), 1024);
    assert_eq!(resp.model_name, "bge-m3");
    assert_eq!(resp.model_sha256, "0123456789abcdef");
    assert_eq!(resp.sidecar_version, "1.0.0");
    assert_eq!(resp.device, "cuda");
    assert_eq!(resp.usage.completion_tokens, 0);
}

#[tokio::test]
async fn batch_of_32_returns_32_embeddings() {
    let sidecar = MockSidecar::start().await;
    let texts = (0..32).map(|i| format!("text {i}")).collect();
    let resp = sidecar
        .provider()
        .call_embed(&req(texts, "ap-southeast-1"), "bge-m3", deadline())
        .await
        .unwrap();

    assert_eq!(resp.embeddings.len(), 32);
    assert!(resp
        .embeddings
        .iter()
        .all(|embedding| embedding.len() == 1024));
}

#[tokio::test]
async fn cost_for_bge_is_zero() {
    let sidecar = MockSidecar::start().await;
    let provider = sidecar.provider();

    assert_eq!(provider.cost_for("bge-m3", 1), 0.0);
    assert_eq!(provider.cost_for("bge-m3", 1_000_000), 0.0);
}

#[tokio::test]
async fn input_too_long_maps_to_413() {
    let sidecar = MockSidecar::start().await;
    sidecar.set_status(StatusCode::PAYLOAD_TOO_LARGE);
    let err = sidecar
        .provider()
        .call_embed(
            &req(vec!["x ".repeat(9_000)], "ap-southeast-1"),
            "bge-m3",
            deadline(),
        )
        .await
        .unwrap_err();

    assert!(matches!(
        err,
        RouterError::TerminalProviderError {
            provider: cyberos_ai_gateway::policy::ProviderKind::Bge,
            status: 413,
            ..
        }
    ));
}

#[tokio::test]
async fn no_sidecar_in_region_errors_before_http() {
    let sidecar = MockSidecar::start().await;
    let err = sidecar
        .provider()
        .call_embed(&req(vec!["test".to_string()], "vn-1"), "bge-m3", deadline())
        .await
        .unwrap_err();

    assert!(matches!(err, RouterError::NoSidecarForRegion { region } if region == "vn-1"));
    assert_eq!(sidecar.request_count(), 0);
}

#[tokio::test]
async fn health_check_all_sidecars_observes_ready_and_warmup() {
    let sidecar = MockSidecar::start().await;
    let provider = sidecar.provider();
    provider.health_check_all_sidecars().await.unwrap();

    sidecar.set_health_status(StatusCode::SERVICE_UNAVAILABLE);
    let err = provider.health_check_all_sidecars().await.unwrap_err();
    assert!(matches!(
        err,
        RouterError::TerminalProviderError { status: 503, .. }
    ));
}

#[tokio::test]
async fn per_region_sidecar_selection_uses_requested_region() {
    let sg = MockSidecar::start().await;
    let eu = MockSidecar::start().await;
    let mut sidecars = HashMap::new();
    sidecars.insert("ap-southeast-1".to_string(), sg.url.clone());
    sidecars.insert("eu-central-1".to_string(), eu.url.clone());
    let provider = BgeProvider::new(sidecars);

    provider
        .call_embed(
            &req(vec!["sg".to_string()], "ap-southeast-1"),
            "bge-m3",
            deadline(),
        )
        .await
        .unwrap();
    provider
        .call_embed(
            &req(vec!["eu".to_string()], "eu-central-1"),
            "bge-m3",
            deadline(),
        )
        .await
        .unwrap();

    assert_eq!(sg.request_count(), 1);
    assert_eq!(eu.request_count(), 1);
}

#[tokio::test]
async fn load_sidecar_urls_parses_embeddings_yaml() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("config/embeddings.yaml");
    let urls = load_sidecar_urls(&path).unwrap();

    assert_eq!(
        urls.get("ap-southeast-1").map(String::as_str),
        Some("http://bge-sidecar-sg-1:5060")
    );
}

#[tokio::test]
async fn device_flip_from_cuda_to_cpu_is_observed_without_failing_call() {
    let sidecar = MockSidecar::start().await;
    sidecar.set_devices(vec!["cuda", "cpu"]);
    let provider = sidecar.provider();

    provider
        .call_embed(
            &req(vec!["a".to_string()], "ap-southeast-1"),
            "bge-m3",
            deadline(),
        )
        .await
        .unwrap();
    let resp = provider
        .call_embed(
            &req(vec!["b".to_string()], "ap-southeast-1"),
            "bge-m3",
            deadline(),
        )
        .await
        .unwrap();

    assert_eq!(resp.device, "cpu");
}
