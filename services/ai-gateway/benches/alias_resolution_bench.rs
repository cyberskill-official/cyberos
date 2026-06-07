//! FR-AI-006 §4 #12 — Latency benchmark for model-alias resolution.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Once;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use cyberos_ai_gateway::alias;
use cyberos_ai_gateway::cost_table::init_cost_table;
use cyberos_ai_gateway::policy::{AiPolicy, EmergencyOverride, Provider, Residency, TenantPolicy};

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

fn model_map(entries: &[(&str, &str)]) -> HashMap<String, String> {
    entries
        .iter()
        .map(|(alias, model)| ((*alias).to_string(), (*model).to_string()))
        .collect()
}

fn bench_policy() -> TenantPolicy {
    init_table();
    TenantPolicy {
        tenant_id: "org:bench".into(),
        ai_policy: AiPolicy {
            monthly_cap_usd: rust_decimal_macros::dec!(100.00),
            warn_threshold: 0.80,
            hard_stop: true,
            primary_provider: Provider::Bedrock {
                region: "ap-southeast-1".into(),
                model_alias_map: model_map(&[
                    ("chat.smart", "anthropic.claude-3-5-sonnet-20241022-v2:0"),
                    ("chat.fast", "anthropic.claude-3-haiku-20240307-v1:0"),
                ]),
            },
            fallback_chain: vec![Provider::Anthropic {
                model_alias_map: model_map(&[("chat.long", "claude-3-5-sonnet-20241022")]),
            }],
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

fn bench_primary_resolve(c: &mut Criterion) {
    let policy = bench_policy();
    c.bench_function("alias::resolve primary", |b| {
        b.iter(|| black_box(alias::resolve(black_box("chat.smart"), black_box(&policy))))
    });
}

fn bench_fallback_resolve(c: &mut Criterion) {
    let policy = bench_policy();
    c.bench_function("alias::resolve fallback", |b| {
        b.iter(|| black_box(alias::resolve(black_box("chat.long"), black_box(&policy))))
    });
}

criterion_group!(benches, bench_primary_resolve, bench_fallback_resolve);
criterion_main!(benches);
