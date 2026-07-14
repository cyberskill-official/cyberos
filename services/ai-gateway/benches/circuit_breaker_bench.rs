//! TASK-AI-009 §5 — Latency benchmark for `is_open` on the closed-state hot path.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use cyberos_ai_gateway::circuit_breaker::{self, clock::SystemClock, CallOutcome};
use cyberos_ai_gateway::policy::ProviderKind;

fn bench_is_open_closed_state(c: &mut Criterion) {
    circuit_breaker::init(Box::new(SystemClock::new()));
    // Pre-populate one breaker entry so the DashMap shard is warm.
    circuit_breaker::record_outcome(&ProviderKind::Bedrock, "bench-model", CallOutcome::Success);
    c.bench_function("circuit_breaker::is_open closed", |b| {
        b.iter(|| {
            circuit_breaker::is_open(black_box(&ProviderKind::Bedrock), black_box("bench-model"))
        });
    });
}

criterion_group!(benches, bench_is_open_closed_state);
criterion_main!(benches);
