//! FR-AI-011 — loopback redaction latency guard for <=8 KiB prompts.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use axum::body::Body;
use axum::http::{Response, StatusCode};
use axum::Router as AxumRouter;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use cyberos_ai_gateway::policy::{AiPolicy, EmergencyOverride, Provider, Residency, TenantPolicy};
use cyberos_ai_gateway::redact;
use serde_json::json;

const CALLS_PER_SAMPLE: usize = 64;
const REDACTION_P95_GATE: Duration = Duration::from_millis(30);

fn minimal_policy() -> TenantPolicy {
    TenantPolicy {
        tenant_id: "bench-tenant".into(),
        ai_policy: AiPolicy {
            monthly_cap_usd: "100.00".parse().unwrap(),
            warn_threshold: 0.80,
            hard_stop: true,
            primary_provider: Provider::Anthropic {
                model_alias_map: HashMap::new(),
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
        },
    }
}

fn prompt_8kb() -> String {
    let mut prompt = String::new();
    while prompt.len() < 8 * 1024 {
        prompt.push_str("Contact secret@example.com or 4111-1111-1111-1111. ");
    }
    prompt.truncate(8 * 1024);
    prompt
}

fn spawn_sidecar(rt: &tokio::runtime::Runtime) -> String {
    rt.block_on(async {
        let body = json!({
            "redacted_text": "Contact <EMAIL_ADDRESS_1> or <CREDIT_CARD_1>.",
            "items": [
                {
                    "entity": "EMAIL_ADDRESS",
                    "start": 8,
                    "end": 26,
                    "original": "secret@example.com"
                },
                {
                    "entity": "CREDIT_CARD",
                    "start": 30,
                    "end": 49,
                    "original": "4111-1111-1111-1111"
                }
            ]
        })
        .to_string();

        let app = AxumRouter::new().fallback(move |_request_body: String| {
            let body = body.clone();
            async move {
                Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .expect("response")
            }
        });

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind loopback");
        let addr = listener.local_addr().expect("local addr");
        tokio::spawn(async move {
            axum::serve(listener, app).await.expect("sidecar server");
        });
        format!("http://{addr}/redact")
    })
}

fn redaction_p95(rt: &tokio::runtime::Runtime, policy: &TenantPolicy, prompt: &str) -> Duration {
    let mut samples = rt.block_on(async {
        let mut samples = Vec::with_capacity(CALLS_PER_SAMPLE);
        for _ in 0..CALLS_PER_SAMPLE {
            let started = Instant::now();
            let result = redact::redact(prompt, policy).await.expect("redaction ok");
            assert!(!result.redacted_text.contains("secret@example.com"));
            samples.push(started.elapsed());
        }
        samples
    });
    samples.sort_unstable();
    samples[((samples.len() as f64 * 0.95).ceil() as usize).saturating_sub(1)]
}

fn bench_redaction_p95(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("tokio runtime");
    let sidecar_url = spawn_sidecar(&rt);
    std::env::set_var("CYBEROS_AI_GATEWAY_PRESIDIO_URL", sidecar_url);
    std::env::set_var("CYBEROS_AI_GATEWAY_PRESIDIO_TIMEOUT_MS", "2000");

    let policy = minimal_policy();
    let prompt = prompt_8kb();

    c.bench_function("redact p95 <=8kb loopback", |b| {
        b.iter(|| {
            let p95 = redaction_p95(&rt, &policy, &prompt);
            assert!(
                p95 <= REDACTION_P95_GATE,
                "redaction p95 {p95:?} exceeded {REDACTION_P95_GATE:?}"
            );
            black_box(p95);
        });
    });
}

criterion_group!(benches, bench_redaction_p95);
criterion_main!(benches);
