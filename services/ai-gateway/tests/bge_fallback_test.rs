//! FR-AI-019 — BGE circuit-breaker/fallback integration tests.

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::body::Body;
use axum::http::{Response, StatusCode};
use axum::routing::post;
use axum::Router as AxumRouter;
use cyberos_ai_gateway::router::bge_provider::BgeProvider;
use cyberos_ai_gateway::router::{
    call_embed_provider_with_chain, EmbedRequest, EmbedTask, ProviderEndpoint, RouterError,
};

#[derive(Clone)]
struct FailingSidecar {
    url: String,
    request_count: Arc<AtomicUsize>,
}

impl FailingSidecar {
    async fn start() -> Self {
        let request_count = Arc::new(AtomicUsize::new(0));
        let route_count = request_count.clone();
        let app = AxumRouter::new().route(
            "/embed",
            post(move || {
                let route_count = route_count.clone();
                async move {
                    route_count.fetch_add(1, Ordering::SeqCst);
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .header("content-type", "application/json")
                        .body(Body::from(r#"{"error":"boom"}"#))
                        .unwrap()
                }
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        Self {
            url: format!("http://{addr}"),
            request_count,
        }
    }

    fn provider(&self) -> BgeProvider {
        let mut sidecars = HashMap::new();
        sidecars.insert("ap-southeast-1".to_string(), self.url.clone());
        BgeProvider::new(sidecars)
    }

    fn request_count(&self) -> usize {
        self.request_count.load(Ordering::SeqCst)
    }
}

fn req() -> EmbedRequest {
    EmbedRequest {
        texts: vec!["test".to_string()],
        tenant_id: "tenant-a".to_string(),
        task: EmbedTask::Passage,
        region: "ap-southeast-1".to_string(),
    }
}

fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(5)
}

#[tokio::test]
async fn repeated_sidecar_5xx_opens_breaker_and_subsequent_call_fails_fast() {
    cyberos_ai_gateway::circuit_breaker::reset_for_tests();
    let sidecar = FailingSidecar::start().await;
    let embed_req = req();

    for _ in 0..2 {
        let _ = call_embed_provider_with_chain(
            &embed_req,
            deadline(),
            vec![ProviderEndpoint::new(
                Box::new(sidecar.provider()),
                "bge-m3-breaker",
                0,
            )],
        )
        .await;
    }

    let before = sidecar.request_count();
    let err = call_embed_provider_with_chain(
        &embed_req,
        deadline(),
        vec![ProviderEndpoint::new(
            Box::new(sidecar.provider()),
            "bge-m3-breaker",
            0,
        )],
    )
    .await
    .unwrap_err();

    assert!(matches!(err, RouterError::AllProvidersFailed { .. }));
    assert_eq!(sidecar.request_count(), before);
}
