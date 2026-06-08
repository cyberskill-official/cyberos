//! FR-AI-019 — BGE adaptive batching tests.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::body::Body;
use axum::extract::State;
use axum::http::{Response, StatusCode};
use axum::routing::post;
use axum::{Json, Router as AxumRouter};
use cyberos_ai_gateway::router::bge_provider::BgeProvider;
use cyberos_ai_gateway::router::{EmbedRequest, EmbedTask, Provider};
use serde_json::{json, Value};

#[derive(Clone)]
struct BatchSidecarState {
    requests: Arc<Mutex<Vec<Value>>>,
    request_count: Arc<AtomicUsize>,
}

#[derive(Clone)]
struct BatchSidecar {
    url: String,
    state: BatchSidecarState,
}

impl BatchSidecar {
    async fn start() -> Self {
        let state = BatchSidecarState {
            requests: Arc::new(Mutex::new(Vec::new())),
            request_count: Arc::new(AtomicUsize::new(0)),
        };
        let app = AxumRouter::new()
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

    fn requests(&self) -> Vec<Value> {
        self.state.requests.lock().unwrap().clone()
    }
}

async fn mock_embed(
    State(state): State<BatchSidecarState>,
    Json(body): Json<Value>,
) -> Response<Body> {
    state.request_count.fetch_add(1, Ordering::SeqCst);
    state.requests.lock().unwrap().push(body.clone());
    let texts = body["texts"].as_array().cloned().unwrap_or_default();
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "embeddings": vec![vec![0.0_f32; 1024]; texts.len()],
                "model_name": "bge-m3",
                "model_sha256": "0123456789abcdef",
                "sidecar_version": "1.0.0",
                "device": "cuda",
                "elapsed_ms": 9
            })
            .to_string(),
        ))
        .unwrap()
}

fn req(text: String, tenant_id: &str) -> EmbedRequest {
    EmbedRequest {
        texts: vec![text],
        tenant_id: tenant_id.to_string(),
        task: EmbedTask::Passage,
        region: "ap-southeast-1".to_string(),
    }
}

fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(5)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn thirty_two_concurrent_calls_dispatch_in_one_batch() {
    let sidecar = BatchSidecar::start().await;
    let provider = sidecar.provider();
    let mut joinset = tokio::task::JoinSet::new();

    for i in 0..32 {
        let provider = provider.clone();
        joinset.spawn(async move {
            provider
                .call_embed(
                    &req(format!("text-{i}"), &format!("tenant-{}", i % 4)),
                    "bge-m3",
                    deadline(),
                )
                .await
        });
    }

    while let Some(result) = joinset.join_next().await {
        result.unwrap().unwrap();
    }

    assert_eq!(sidecar.request_count(), 1);
    assert_eq!(sidecar.requests()[0]["texts"].as_array().unwrap().len(), 32);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn five_calls_wait_for_timeout_and_share_one_batch() {
    let sidecar = BatchSidecar::start().await;
    let provider = sidecar.provider();
    let started = Instant::now();
    let mut joinset = tokio::task::JoinSet::new();

    for i in 0..5 {
        let provider = provider.clone();
        joinset.spawn(async move {
            provider
                .call_embed(&req(format!("text-{i}"), "tenant-a"), "bge-m3", deadline())
                .await
        });
    }

    while let Some(result) = joinset.join_next().await {
        result.unwrap().unwrap();
    }

    assert_eq!(sidecar.request_count(), 1);
    assert!(started.elapsed() >= Duration::from_millis(45));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn per_tenant_fairness_no_starvation() {
    let sidecar = BatchSidecar::start().await;
    let provider = sidecar.provider();
    let mut joinset = tokio::task::JoinSet::new();

    for i in 0..32 {
        let provider = provider.clone();
        joinset.spawn(async move {
            provider
                .call_embed(&req(format!("a-{i}"), "tenant-a"), "bge-m3", deadline())
                .await
        });
    }
    tokio::time::sleep(Duration::from_millis(10)).await;
    let provider_b = provider.clone();
    let tenant_b = tokio::spawn(async move {
        provider_b
            .call_embed(&req("b".to_string(), "tenant-b"), "bge-m3", deadline())
            .await
    });

    tenant_b.await.unwrap().unwrap();
    while let Some(result) = joinset.join_next().await {
        result.unwrap().unwrap();
    }

    let first = &sidecar.requests()[0];
    let texts = first["texts"].as_array().unwrap();
    let tenants: HashSet<&str> = texts
        .iter()
        .filter_map(|value| value.as_str())
        .map(|text| {
            if text.starts_with("b") {
                "tenant-b"
            } else {
                "tenant-a"
            }
        })
        .collect();
    assert!(tenants.contains("tenant-a"));
    assert!(tenants.contains("tenant-b"));
    assert!(texts.len() <= 32);
}
