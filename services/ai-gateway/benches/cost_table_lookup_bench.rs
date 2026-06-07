//! FR-AI-007 §4 #7 — Latency benchmark for cost-table lookups.

use std::path::PathBuf;
use std::sync::Once;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use cyberos_ai_gateway::cost_table::{self, init_cost_table};
use cyberos_ai_gateway::policy::ProviderKind;

static INIT: Once = Once::new();

fn init_table() {
    INIT.call_once(|| {
        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        let handle = runtime
            .block_on(init_cost_table(&PathBuf::from(
                "tests/fixtures/cost_table/valid_rates.yaml",
            )))
            .expect("cost table fixture should load");
        let _ = Box::leak(Box::new(handle));
    });
}

fn bench_lookup_hit(c: &mut Criterion) {
    init_table();
    c.bench_function("cost_table::lookup hit", |b| {
        b.iter(|| {
            black_box(cost_table::lookup(
                black_box(&ProviderKind::Bedrock),
                black_box("anthropic.claude-3-5-sonnet-20241022-v2:0"),
            ))
        });
    });
}

fn bench_lookup_miss(c: &mut Criterion) {
    init_table();
    c.bench_function("cost_table::lookup miss", |b| {
        b.iter(|| {
            black_box(cost_table::lookup(
                black_box(&ProviderKind::Bedrock),
                black_box("nonexistent-model"),
            ))
        });
    });
}

criterion_group!(benches, bench_lookup_hit, bench_lookup_miss);
criterion_main!(benches);
