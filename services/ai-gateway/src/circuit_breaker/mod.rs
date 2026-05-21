//! FR-AI-009 — Circuit breaker per (provider, model) with half-open recovery probing.

use std::sync::atomic::{AtomicU32, AtomicU64, AtomicU8, Ordering};
use std::sync::{Arc, Mutex};

use dashmap::DashMap;
use once_cell::sync::OnceCell;

use crate::policy::ProviderKind;

pub mod clock;
pub mod state;

pub use clock::Clock;
pub use state::{BreakerState, BreakerStatus, CallOutcome, SystemTimeUnix};

const FAILURE_THRESHOLD: u32 = 5;
const WINDOW_NANOS: u64 = 60 * 1_000_000_000;
const OPEN_DURATION_NANOS: u64 = 30 * 1_000_000_000;

// ─── Key types ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct BreakerKey {
    provider: ProviderKind,
    model: String,
}

#[derive(Debug, Hash, PartialEq, Eq)]
struct BreakerKeyRef<'a> {
    provider: ProviderKind,
    model: &'a str,
}

impl<'a> std::borrow::Borrow<BreakerKeyRef<'a>> for BreakerKey {
    fn borrow(&self) -> &BreakerKeyRef<'a> {
        // SAFETY: BreakerKey { provider, model: String } and
        // BreakerKeyRef { provider, model: &str } have identical layout.
        // ProviderKind is Copy; String and &str are both (ptr, len) fat pointers.
        // The Borrow impl only uses the returned reference for Hash + Eq,
        // which read the same fields at the same offsets.
        unsafe { std::mem::transmute::<&BreakerKey, &BreakerKeyRef<'a>>(self) }
    }
}

// ─── Global state ─────────────────────────────────────────────────────────────

static BREAKERS: OnceCell<DashMap<BreakerKey, Arc<Breaker>>> = OnceCell::new();
static CLOCK: OnceCell<Mutex<Box<dyn Clock>>> = OnceCell::new();

// ─── Metrics ──────────────────────────────────────────────────────────────────

mod metrics {
    use once_cell::sync::Lazy;
    use prometheus::{register_counter_vec, register_gauge_vec, CounterVec, GaugeVec};

    pub static STATE: Lazy<GaugeVec> = Lazy::new(|| {
        register_gauge_vec!(
            "ai_breaker_state",
            "Circuit breaker state (one row per state, value 0/1)",
            &["provider", "model", "state"]
        )
        .unwrap()
    });

    pub static TRANSITIONS: Lazy<CounterVec> = Lazy::new(|| {
        register_counter_vec!(
            "ai_breaker_transitions_total",
            "Breaker state transitions",
            &["provider", "model", "from", "to"]
        )
        .unwrap()
    });

    pub static SHORT_CIRCUITS: Lazy<CounterVec> = Lazy::new(|| {
        register_counter_vec!(
            "ai_breaker_short_circuits_total",
            "Calls blocked by an open breaker",
            &["provider", "model"]
        )
        .unwrap()
    });

    pub static PROBES: Lazy<CounterVec> = Lazy::new(|| {
        register_counter_vec!(
            "ai_breaker_probes_total",
            "HalfOpen probe outcomes",
            &["provider", "model", "outcome"]
        )
        .unwrap()
    });
}

// ─── Per-key breaker ──────────────────────────────────────────────────────────

struct Breaker {
    state: AtomicU8,
    failure_count: AtomicU32,
    window_start_nanos: AtomicU64,
    open_until_nanos: AtomicU64,
    short_circuits: AtomicU64,
    probes_succeeded: AtomicU64,
    probes_failed: AtomicU64,
    last_state_change: AtomicU64,
}

impl Breaker {
    fn new(now: u64) -> Self {
        Self {
            state: AtomicU8::new(BreakerState::Closed.as_u8()),
            failure_count: AtomicU32::new(0),
            window_start_nanos: AtomicU64::new(now),
            open_until_nanos: AtomicU64::new(0),
            short_circuits: AtomicU64::new(0),
            probes_succeeded: AtomicU64::new(0),
            probes_failed: AtomicU64::new(0),
            last_state_change: AtomicU64::new(now),
        }
    }
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// Initialise the breaker subsystem. MUST be called before any other function.
/// Production passes `SystemClock`; tests pass `MockClock`.
/// Idempotent: subsequent calls update the clock but reuse the map.
pub fn init(clock: Box<dyn Clock>) {
    BREAKERS.get_or_init(DashMap::new);
    CLOCK.get_or_init(|| Mutex::new(clock));
}

/// Swap the clock (for tests that need deterministic time). Panics if not initialized.
#[allow(dead_code)]
pub fn swap_clock(clock: Box<dyn Clock>) {
    let mutex = CLOCK.get().expect("circuit_breaker not initialized");
    *mutex.lock().unwrap() = clock;
}

/// Test-only: clear all breaker state. Clock persists across resets.
#[allow(dead_code)]
pub fn reset_for_tests() {
    if let Some(map) = BREAKERS.get() {
        map.clear();
    }
}

fn now_nanos() -> u64 {
    CLOCK
        .get()
        .expect("circuit_breaker not initialized")
        .lock()
        .unwrap()
        .nanos_now()
}

/// Returns `true` if the breaker for `(provider, model)` is Open.
///
/// When Open elapses to HalfOpen, the FIRST caller that wins the CAS
/// gets `false` (probe slot); subsequent callers see `true`.
pub fn is_open(provider: &ProviderKind, model: &str) -> bool {
    let map = BREAKERS.get().expect("circuit_breaker not initialized");
    let key = BreakerKey {
        provider: *provider,
        model: model.to_string(),
    };

    let Some(b) = map.get(&key) else {
        return false;
    };

    let now = now_nanos();
    let state = BreakerState::from_u8(b.state.load(Ordering::Acquire));

    match state {
        BreakerState::Closed => false,
        BreakerState::Open => {
            if now >= b.open_until_nanos.load(Ordering::Acquire) {
                // Try CAS Open → HalfOpen.
                let won_cas = b
                    .state
                    .compare_exchange(
                        BreakerState::Open.as_u8(),
                        BreakerState::HalfOpen.as_u8(),
                        Ordering::AcqRel,
                        Ordering::Acquire,
                    )
                    .is_ok();
                if won_cas {
                    emit_transition(provider, model, BreakerState::Open, BreakerState::HalfOpen);
                    b.last_state_change.store(now, Ordering::Release);
                    false // CAS winner gets the probe slot
                } else {
                    b.short_circuits.fetch_add(1, Ordering::Relaxed);
                    metrics::SHORT_CIRCUITS
                        .with_label_values(&[provider.as_metric_label(), model])
                        .inc();
                    true // CAS loser short-circuits
                }
            } else {
                b.short_circuits.fetch_add(1, Ordering::Relaxed);
                metrics::SHORT_CIRCUITS
                    .with_label_values(&[provider.as_metric_label(), model])
                    .inc();
                true
            }
        }
        BreakerState::HalfOpen => {
            // Probe dispatched; subsequent callers short-circuit.
            b.short_circuits.fetch_add(1, Ordering::Relaxed);
            metrics::SHORT_CIRCUITS
                .with_label_values(&[provider.as_metric_label(), model])
                .inc();
            true
        }
    }
}

/// Record the outcome of a call. Drives the state machine.
pub fn record_outcome(provider: &ProviderKind, model: &str, outcome: CallOutcome) {
    let map = BREAKERS.get().expect("circuit_breaker not initialized");
    let now = now_nanos();

    let key = BreakerKey {
        provider: *provider,
        model: model.to_string(),
    };
    let entry = map.entry(key).or_insert_with(|| Arc::new(Breaker::new(now)));
    let b = entry.value();

    match outcome {
        CallOutcome::Failure5xx
        | CallOutcome::Failure429
        | CallOutcome::Timeout
        | CallOutcome::ConnectionReset => {
            let prior_state = BreakerState::from_u8(b.state.load(Ordering::Acquire));

            if prior_state == BreakerState::HalfOpen {
                // Probe failed — re-open with reset 30s timer.
                b.state
                    .store(BreakerState::Open.as_u8(), Ordering::Release);
                b.open_until_nanos
                    .store(now + OPEN_DURATION_NANOS, Ordering::Release);
                b.last_state_change.store(now, Ordering::Release);
                b.probes_failed.fetch_add(1, Ordering::Relaxed);
                metrics::PROBES
                    .with_label_values(&[provider.as_metric_label(), model, "failed"])
                    .inc();
                emit_transition(provider, model, BreakerState::HalfOpen, BreakerState::Open);
                return;
            }

            // Sliding window check.
            let window_start = b.window_start_nanos.load(Ordering::Acquire);
            if now.saturating_sub(window_start) > WINDOW_NANOS {
                // Window expired — reset counter.
                b.failure_count.store(1, Ordering::Release);
                b.window_start_nanos.store(now, Ordering::Release);
            } else {
                let count = b.failure_count.fetch_add(1, Ordering::AcqRel) + 1;
                if count >= FAILURE_THRESHOLD {
                    // CAS-guard Closed → Open so concurrent failures emit exactly once.
                    let won = b
                        .state
                        .compare_exchange(
                            BreakerState::Closed.as_u8(),
                            BreakerState::Open.as_u8(),
                            Ordering::AcqRel,
                            Ordering::Acquire,
                        )
                        .is_ok();
                    if won {
                        b.open_until_nanos
                            .store(now + OPEN_DURATION_NANOS, Ordering::Release);
                        b.last_state_change.store(now, Ordering::Release);
                        emit_transition(provider, model, BreakerState::Closed, BreakerState::Open);
                    }
                }
            }
        }
        CallOutcome::Success => {
            let prior_state = BreakerState::from_u8(b.state.load(Ordering::Acquire));
            b.failure_count.store(0, Ordering::Release);
            b.window_start_nanos.store(now, Ordering::Release);
            b.state
                .store(BreakerState::Closed.as_u8(), Ordering::Release);
            b.last_state_change.store(now, Ordering::Release);
            if prior_state == BreakerState::HalfOpen {
                b.probes_succeeded.fetch_add(1, Ordering::Relaxed);
                metrics::PROBES
                    .with_label_values(&[provider.as_metric_label(), model, "succeeded"])
                    .inc();
                emit_transition(provider, model, BreakerState::HalfOpen, BreakerState::Closed);
            } else if prior_state != BreakerState::Closed {
                emit_transition(provider, model, prior_state, BreakerState::Closed);
            }
        }
        CallOutcome::Failure4xx => { /* §1 #13: 4xx is tenant-side; ignored */ }
    }
}

/// Snapshot every breaker's current status. Non-disturbing: no per-key writes.
pub fn status_all() -> Vec<BreakerStatus> {
    let Some(map) = BREAKERS.get() else {
        return vec![];
    };
    map.iter()
        .map(|entry| {
            let key = entry.key();
            let b = entry.value();
            let state = BreakerState::from_u8(b.state.load(Ordering::Acquire));
            BreakerStatus {
                provider: key.provider,
                model: key.model.clone(),
                state,
                failure_count_window: b.failure_count.load(Ordering::Acquire),
                last_state_change: b.last_state_change.load(Ordering::Acquire),
                next_half_open_at: if state == BreakerState::Open {
                    Some(b.open_until_nanos.load(Ordering::Acquire))
                } else {
                    None
                },
                short_circuits_total: b.short_circuits.load(Ordering::Acquire),
                probes_succeeded: b.probes_succeeded.load(Ordering::Acquire),
                probes_failed: b.probes_failed.load(Ordering::Acquire),
            }
        })
        .collect()
}

/// Force-close a breaker (operator override). Returns `true` if the breaker
/// existed AND was open/half-open before reset.
pub fn reset(provider: &ProviderKind, model: &str) -> bool {
    let map = BREAKERS.get().expect("circuit_breaker not initialized");
    let key = BreakerKey {
        provider: *provider,
        model: model.to_string(),
    };
    let Some(entry) = map.get(&key) else {
        return false;
    };
    let b = entry.value();
    let prior = BreakerState::from_u8(b.state.load(Ordering::Acquire));
    if prior == BreakerState::Closed {
        return false;
    }
    let now = now_nanos();
    b.state
        .store(BreakerState::Closed.as_u8(), Ordering::Release);
    b.failure_count.store(0, Ordering::Release);
    b.window_start_nanos.store(now, Ordering::Release);
    b.open_until_nanos.store(0, Ordering::Release);
    b.last_state_change.store(now, Ordering::Release);
    emit_transition(provider, model, prior, BreakerState::Closed);
    true
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn emit_transition(provider: &ProviderKind, model: &str, from: BreakerState, to: BreakerState) {
    metrics::TRANSITIONS
        .with_label_values(&[
            provider.as_metric_label(),
            model,
            from.as_metric_label(),
            to.as_metric_label(),
        ])
        .inc();
    // Update STATE gauge: set the new state to 1, others to 0.
    for s in [BreakerState::Closed, BreakerState::Open, BreakerState::HalfOpen] {
        metrics::STATE
            .with_label_values(&[provider.as_metric_label(), model, s.as_metric_label()])
            .set(if s == to { 1.0 } else { 0.0 });
    }
}
