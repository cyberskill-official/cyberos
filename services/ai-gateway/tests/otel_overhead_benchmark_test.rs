//! FR-AI-022 §1 #11 — lightweight span wrapper overhead guard.

use std::time::{Duration, Instant};

use cyberos_ai_gateway::otel::spans;

#[test]
fn otel_overhead_benchmark_p95_under_one_ms() {
    const N: usize = 1_000;
    let mut baseline = Vec::with_capacity(N);
    let mut enabled = Vec::with_capacity(N);

    for idx in 0..N {
        let started = Instant::now();
        std::hint::black_box(idx);
        baseline.push(started.elapsed());
    }

    spans::clear_finished_spans();
    for idx in 0..N {
        let started = Instant::now();
        let mut span =
            spans::start_rerank_root("tenant:test", "rerank.default", &format!("req-{idx}"));
        span.set_str(cyberos_ai_gateway::otel::attributes::OUTCOME, "allow");
        span.end_ok();
        enabled.push(started.elapsed());
    }

    baseline.sort_unstable();
    enabled.sort_unstable();
    let p95_baseline = baseline[(N as f64 * 0.95) as usize];
    let p95_enabled = enabled[(N as f64 * 0.95) as usize];
    let delta = p95_enabled.saturating_sub(p95_baseline);

    assert!(
        delta < Duration::from_millis(1),
        "p95 OTel wrapper overhead was {delta:?}; baseline={p95_baseline:?} enabled={p95_enabled:?}"
    );
}
