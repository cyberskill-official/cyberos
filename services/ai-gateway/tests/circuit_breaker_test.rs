//! FR-AI-009 §5 — Integration tests for the circuit breaker.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use cyberos_ai_gateway::circuit_breaker::{
    self, clock::MockClock, BreakerState, CallOutcome, Clock,
};
use cyberos_ai_gateway::policy::ProviderKind;
use once_cell::sync::Lazy;

// ─── Test infrastructure ──────────────────────────────────────────────────────

/// Serialise all tests that mutate global breaker state.
static TEST_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

/// First-call flag for init().
static INIT_ONCE: std::sync::Once = std::sync::Once::new();

/// Per-test setup: serialise, reset map, fresh clock.
/// Returns (clock, guard) — caller MUST hold the guard for the test's duration.
fn ensure_initialized() -> (Arc<MockClock>, std::sync::MutexGuard<'static, ()>) {
    let guard = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let clock = Arc::new(MockClock::new());
    INIT_ONCE.call_once(|| {
        circuit_breaker::init(Box::new(clock.clone()));
    });
    circuit_breaker::reset_for_tests();
    circuit_breaker::swap_clock(Box::new(clock.clone()));
    (clock, guard)
}

/// Read a Prometheus counter or gauge value by name + labels.
fn metric_value(name: &str, labels: &[(&str, &str)]) -> f64 {
    let metric_families = prometheus::default_registry().gather();
    for mf in metric_families {
        if mf.get_name() != name {
            continue;
        }
        for m in mf.get_metric() {
            let actual_labels: Vec<_> = m
                .get_label()
                .iter()
                .map(|p| (p.get_name(), p.get_value()))
                .collect();
            if labels
                .iter()
                .all(|(k, v)| actual_labels.iter().any(|(ak, av)| ak == k && av == v))
            {
                return m.get_counter().get_value();
            }
        }
    }
    0.0
}

// ─── Tests ────────────────────────────────────────────────────────────────────

/// AC #1: Closed → Open after 5 failures.
#[tokio::test]
async fn opens_after_5_failures() {
    let (_clock, _guard) = ensure_initialized();
    let model = "anthropic.claude-3-5-sonnet-20241022-v2:0";
    for _ in 0..5 {
        circuit_breaker::record_outcome(&ProviderKind::Bedrock, model, CallOutcome::Failure5xx);
    }
    assert!(circuit_breaker::is_open(&ProviderKind::Bedrock, model));
}

/// AC #2: Open blocks calls for 30s.
#[tokio::test]
async fn open_blocks_until_30s() {
    let (clock, _guard) = ensure_initialized();
    let model = "test-blocks-30s";
    for _ in 0..5 {
        circuit_breaker::record_outcome(&ProviderKind::Bedrock, model, CallOutcome::Failure5xx);
    }
    assert!(circuit_breaker::is_open(&ProviderKind::Bedrock, model));
    clock.advance(Duration::from_secs(29));
    assert!(circuit_breaker::is_open(&ProviderKind::Bedrock, model));
    clock.advance(Duration::from_millis(1100));
    assert!(!circuit_breaker::is_open(&ProviderKind::Bedrock, model));
}

/// AC #3: HalfOpen → Closed on probe success.
#[tokio::test]
async fn half_open_probe_success_closes() {
    let (clock, _guard) = ensure_initialized();
    let model = "test-probe-success";
    for _ in 0..5 {
        circuit_breaker::record_outcome(&ProviderKind::Bedrock, model, CallOutcome::Failure5xx);
    }
    clock.advance(Duration::from_secs(31));
    let _probe_slot = circuit_breaker::is_open(&ProviderKind::Bedrock, model);
    circuit_breaker::record_outcome(&ProviderKind::Bedrock, model, CallOutcome::Success);
    assert!(!circuit_breaker::is_open(&ProviderKind::Bedrock, model));
    let status = circuit_breaker::status_all()
        .into_iter()
        .find(|s| s.model == model)
        .unwrap();
    assert_eq!(status.state, BreakerState::Closed);
    assert_eq!(status.failure_count_window, 0);
}

/// AC #4 (partial): HalfOpen → Open on probe failure with timer reset.
#[tokio::test]
async fn half_open_probe_failure_reopens_with_reset_timer() {
    let (clock, _guard) = ensure_initialized();
    let model = "test-probe-fail-reopen";
    for _ in 0..5 {
        circuit_breaker::record_outcome(&ProviderKind::Bedrock, model, CallOutcome::Failure5xx);
    }
    clock.advance(Duration::from_secs(31));
    let _ = circuit_breaker::is_open(&ProviderKind::Bedrock, model);
    let _probe_open_at = clock.nanos_now();
    circuit_breaker::record_outcome(&ProviderKind::Bedrock, model, CallOutcome::Failure5xx);
    // Re-opened. Next HalfOpen at probe_open_at + 30s.
    clock.advance(Duration::from_secs(29));
    assert!(circuit_breaker::is_open(&ProviderKind::Bedrock, model));
    clock.advance(Duration::from_millis(1100));
    assert!(!circuit_breaker::is_open(&ProviderKind::Bedrock, model));
}

/// AC #5: Concurrent CAS — exactly one winner.
#[tokio::test]
async fn concurrent_cas_during_half_open_transition() {
    let (clock, _guard) = ensure_initialized();
    let model = "test-concurrent-cas";
    for _ in 0..5 {
        circuit_breaker::record_outcome(&ProviderKind::Bedrock, model, CallOutcome::Failure5xx);
    }
    clock.advance(Duration::from_secs(31));

    let handles: Vec<_> = (0..100)
        .map(|_| {
            tokio::spawn(async move { !circuit_breaker::is_open(&ProviderKind::Bedrock, model) })
        })
        .collect();
    let results = futures::future::join_all(handles).await;
    let probe_winners = results.into_iter().filter(|r| *r.as_ref().unwrap()).count();
    assert_eq!(
        probe_winners, 1,
        "exactly one caller MUST win the probe slot"
    );
}

/// AC #6: 4xx ignored.
#[tokio::test]
async fn four_xx_does_not_trip() {
    let (_clock, _guard) = ensure_initialized();
    let model = "test-4xx-ignored";
    for _ in 0..100 {
        circuit_breaker::record_outcome(&ProviderKind::Bedrock, model, CallOutcome::Failure4xx);
    }
    assert!(!circuit_breaker::is_open(&ProviderKind::Bedrock, model));
}

/// AC #7: 429 counts as failure.
#[tokio::test]
async fn four_two_nine_counts_as_failure() {
    let (_clock, _guard) = ensure_initialized();
    let model = "test-429-failure";
    for _ in 0..5 {
        circuit_breaker::record_outcome(&ProviderKind::Bedrock, model, CallOutcome::Failure429);
    }
    assert!(circuit_breaker::is_open(&ProviderKind::Bedrock, model));
}

/// AC #8: Per-(provider, model) isolation.
#[tokio::test]
async fn per_provider_model_isolation() {
    let (_clock, _guard) = ensure_initialized();
    for _ in 0..5 {
        circuit_breaker::record_outcome(
            &ProviderKind::Bedrock,
            "claude-3-5-sonnet",
            CallOutcome::Failure5xx,
        );
    }
    assert!(circuit_breaker::is_open(
        &ProviderKind::Bedrock,
        "claude-3-5-sonnet"
    ));
    assert!(!circuit_breaker::is_open(
        &ProviderKind::Bedrock,
        "claude-3-haiku"
    ));
}

/// AC #9: Sliding window — failures outside 60s don't count.
#[tokio::test]
async fn sliding_window_resets_after_60s() {
    let (clock, _guard) = ensure_initialized();
    let model = "test-sliding-window";
    for _ in 0..4 {
        circuit_breaker::record_outcome(&ProviderKind::Bedrock, model, CallOutcome::Failure5xx);
    }
    clock.advance(Duration::from_secs(70));
    circuit_breaker::record_outcome(&ProviderKind::Bedrock, model, CallOutcome::Failure5xx);
    assert!(!circuit_breaker::is_open(&ProviderKind::Bedrock, model));
    let status = circuit_breaker::status_all()
        .into_iter()
        .find(|s| s.model == model)
        .unwrap();
    assert_eq!(status.failure_count_window, 1);
}

/// AC #10: Concurrent record_outcome — no deadlock.
#[tokio::test]
async fn concurrent_1000_record_outcome_safe() {
    let (_clock, _guard) = ensure_initialized();
    let model = "test-1000-concurrent";
    let handles: Vec<_> = (0..1000)
        .map(|i| {
            tokio::spawn(async move {
                let outcome = if i % 3 == 0 {
                    CallOutcome::Failure5xx
                } else {
                    CallOutcome::Success
                };
                circuit_breaker::record_outcome(&ProviderKind::Bedrock, model, outcome);
            })
        })
        .collect();
    futures::future::join_all(handles).await;
    let _ = circuit_breaker::is_open(&ProviderKind::Bedrock, model);
}

/// AC #11: status_all doesn't disturb record_outcome.
#[tokio::test]
async fn status_all_does_not_disturb_record() {
    let (_clock, _guard) = ensure_initialized();
    let model = "test-status-non-disturb";
    let stop = Arc::new(AtomicBool::new(false));
    let stop_for_writer = stop.clone();
    let writer = tokio::spawn(async move {
        while !stop_for_writer.load(Ordering::Relaxed) {
            circuit_breaker::record_outcome(&ProviderKind::Bedrock, model, CallOutcome::Success);
        }
    });
    for _ in 0..100 {
        let _ = circuit_breaker::status_all();
    }
    stop.store(true, Ordering::Relaxed);
    writer.await.unwrap();
}

// ─── Metric assertions (ISS-001) ─────────────────────────────────────────────

/// AC #1 metric: transition counter increments once on Closed → Open.
#[tokio::test]
async fn opens_after_5_failures_emits_transition_metric() {
    let (_clock, _guard) = ensure_initialized();
    let model = "test-transition-metric";
    let before = metric_value(
        "ai_breaker_transitions_total",
        &[
            ("provider", "bedrock"),
            ("model", model),
            ("from", "closed"),
            ("to", "open"),
        ],
    );
    for _ in 0..5 {
        circuit_breaker::record_outcome(&ProviderKind::Bedrock, model, CallOutcome::Failure5xx);
    }
    let after = metric_value(
        "ai_breaker_transitions_total",
        &[
            ("provider", "bedrock"),
            ("model", model),
            ("from", "closed"),
            ("to", "open"),
        ],
    );
    assert_eq!(
        after - before,
        1.0,
        "transition counter MUST increment exactly once"
    );
}

/// AC #2 metric: blocked call increments short_circuits.
#[tokio::test]
async fn open_blocked_call_emits_short_circuit() {
    let (_clock, _guard) = ensure_initialized();
    let model = "test-short-circuit-metric";
    for _ in 0..5 {
        circuit_breaker::record_outcome(&ProviderKind::Bedrock, model, CallOutcome::Failure5xx);
    }
    let before = metric_value(
        "ai_breaker_short_circuits_total",
        &[("provider", "bedrock"), ("model", model)],
    );
    let _ = circuit_breaker::is_open(&ProviderKind::Bedrock, model);
    let after = metric_value(
        "ai_breaker_short_circuits_total",
        &[("provider", "bedrock"), ("model", model)],
    );
    assert_eq!(
        after - before,
        1.0,
        "blocked call MUST increment short_circuits once"
    );
}

/// AC #4 metric: probe success emits probes_total{outcome=succeeded}.
#[tokio::test]
async fn half_open_success_emits_probe_succeeded() {
    let (clock, _guard) = ensure_initialized();
    let model = "test-probe-succeeded-metric";
    for _ in 0..5 {
        circuit_breaker::record_outcome(&ProviderKind::Bedrock, model, CallOutcome::Failure5xx);
    }
    clock.advance(Duration::from_secs(31));
    let _ = circuit_breaker::is_open(&ProviderKind::Bedrock, model);
    let before = metric_value(
        "ai_breaker_probes_total",
        &[
            ("provider", "bedrock"),
            ("model", model),
            ("outcome", "succeeded"),
        ],
    );
    circuit_breaker::record_outcome(&ProviderKind::Bedrock, model, CallOutcome::Success);
    let after = metric_value(
        "ai_breaker_probes_total",
        &[
            ("provider", "bedrock"),
            ("model", model),
            ("outcome", "succeeded"),
        ],
    );
    assert_eq!(
        after - before,
        1.0,
        "probe success MUST increment probes_total{{succeeded}} once"
    );
}

/// AC #5 metric: probe failure emits probes_total{outcome=failed}.
#[tokio::test]
async fn half_open_failure_emits_probe_failed() {
    let (clock, _guard) = ensure_initialized();
    let model = "test-probe-failed-metric";
    for _ in 0..5 {
        circuit_breaker::record_outcome(&ProviderKind::Bedrock, model, CallOutcome::Failure5xx);
    }
    clock.advance(Duration::from_secs(31));
    let _ = circuit_breaker::is_open(&ProviderKind::Bedrock, model);
    let before = metric_value(
        "ai_breaker_probes_total",
        &[
            ("provider", "bedrock"),
            ("model", model),
            ("outcome", "failed"),
        ],
    );
    circuit_breaker::record_outcome(&ProviderKind::Bedrock, model, CallOutcome::Failure5xx);
    let after = metric_value(
        "ai_breaker_probes_total",
        &[
            ("provider", "bedrock"),
            ("model", model),
            ("outcome", "failed"),
        ],
    );
    assert_eq!(
        after - before,
        1.0,
        "probe failure MUST increment probes_total{{failed}} once"
    );
}

// ─── Operator reset (AC #14) ─────────────────────────────────────────────────

#[tokio::test]
async fn reset_force_closes_open_breaker() {
    let (_clock, _guard) = ensure_initialized();
    let model = "test-reset";
    for _ in 0..5 {
        circuit_breaker::record_outcome(&ProviderKind::Bedrock, model, CallOutcome::Failure5xx);
    }
    assert!(circuit_breaker::is_open(&ProviderKind::Bedrock, model));
    let did_reset = circuit_breaker::reset(&ProviderKind::Bedrock, model);
    assert!(did_reset);
    assert!(!circuit_breaker::is_open(&ProviderKind::Bedrock, model));
    assert!(!circuit_breaker::reset(&ProviderKind::Bedrock, model)); // already closed
}
