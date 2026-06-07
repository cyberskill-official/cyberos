//! FR-AI-010 — first-token latency guard for the normalized stream path.

use std::time::{Duration, Instant};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use cyberos_ai_gateway::router::{ProviderStreamResponse, RouterError};
use cyberos_ai_gateway::streaming::ProviderStreamEvent;
use futures::StreamExt;

const STREAMS_PER_SAMPLE: usize = 1_000;
const FIRST_TOKEN_P95_GATE: Duration = Duration::from_millis(1_500);

fn first_token_p95(rt: &tokio::runtime::Runtime) -> Duration {
    let mut samples = rt.block_on(async {
        let mut samples = Vec::with_capacity(STREAMS_PER_SAMPLE);
        for index in 0..STREAMS_PER_SAMPLE {
            let started = Instant::now();
            let events: Vec<Result<ProviderStreamEvent, RouterError>> =
                vec![Ok(ProviderStreamEvent::Token {
                    text: format!("tok-{index}"),
                })];
            let response = ProviderStreamResponse::new(futures::stream::iter(events));
            let mut stream = response.into_events();
            let first = stream.next().await.expect("first event").expect("ok event");
            assert!(matches!(first, ProviderStreamEvent::Token { .. }));
            samples.push(started.elapsed());
        }
        samples
    });

    samples.sort_unstable();
    samples[((samples.len() as f64 * 0.95).ceil() as usize).saturating_sub(1)]
}

fn bench_first_token_p95(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");

    c.bench_function("streaming first_token_p95", |b| {
        b.iter(|| {
            let p95 = first_token_p95(&rt);
            assert!(
                p95 <= FIRST_TOKEN_P95_GATE,
                "first-token p95 {p95:?} exceeded {FIRST_TOKEN_P95_GATE:?}"
            );
            black_box(p95);
        });
    });
}

criterion_group!(benches, bench_first_token_p95);
criterion_main!(benches);
