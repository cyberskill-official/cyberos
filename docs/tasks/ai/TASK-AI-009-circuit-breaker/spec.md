---
# ───── Machine-readable frontmatter (parsed by task-audit + future task-catalog renderer) ─────
id: TASK-AI-009
title: "Circuit breaker per (provider, model) with half-open recovery probing"
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
eu_ai_act_risk_class: not_ai
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-15T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: AI
priority: p0
status: done
verify: T
phase: P0
milestone: P0 · slice 2
slice: 2
owner: Stephen Cheng
created: 2026-05-15
shipped: 2026-05-21
memory_chain_hash: null
related_tasks: [TASK-AI-006, TASK-AI-007, TASK-AI-008, TASK-AI-021]
depends_on: [TASK-AI-008, TASK-AI-006]
blocks: [TASK-AI-021]

# ───── Source contracts ─────
source_pages:
  - website/docs/modules/ai.html#multi-provider
  - website/docs/modules/ai.html#failover-sla
source_decisions:
  #5 (failover policy needs a circuit breaker to avoid hammering a dead provider)
  - docs/tasks/ai/TASK-AI-008-multi-provider-router/spec.md §1
  - docs/tasks/ai/TASK-AI-008-multi-provider-router/spec.md §6 (build_provider_chain hook for breaker filtering)
  - archive/2026-05-14/RESEARCH_REVIEW.md §3.2 (Hystrix-style breaker as prior art)

# ───── Build envelope ─────
language: rust 1.81
service: cyberos/services/ai-gateway/
new_files:
  - services/ai-gateway/src/circuit_breaker.rs
  - services/ai-gateway/src/circuit_breaker/state.rs
  - services/ai-gateway/src/circuit_breaker/clock.rs
  - services/ai-gateway/tests/circuit_breaker_test.rs
  - services/ai-gateway/benches/circuit_breaker_bench.rs
modified_files:
  # consult breaker before dispatch + record outcome
  - services/ai-gateway/src/router.rs
  # build_provider_chain filters open breakers
  - services/ai-gateway/src/router/failover.rs
  # export circuit_breaker module
  - services/ai-gateway/src/lib.rs
  # operator reset endpoint (TASK-AI-021)
  - services/ai-gateway/src/handlers/admin.rs
allowed_tools:
  - file_read: services/ai-gateway/**
  - file_write: services/ai-gateway/{src,tests,benches}/**
  - bash: cargo test -p cyberos-ai-gateway circuit_breaker
  - bash: cargo bench -p cyberos-ai-gateway circuit_breaker_bench
disallowed_tools:
  - bypassing breaker on the hot path (every router call MUST consult)
  #6)
  - persisting breaker state to disk (in-memory only by §1
  - format!("{:?}", provider) for OBS labels (use as_metric_label())
  - directly comparing AtomicU8 to magic numbers like `if state == 1` (use BreakerState::from_u8)
  - per-tenant breaker keying in slice 2 (TASK-AI-021 introduces tenant scope)

# ───── Estimated work ─────
effort_hours: 6
subtasks:
  - "0.5h: Breaker state machine type — BreakerState enum, AtomicState wrapper, from_u8/as_u8 helpers"
  - "0.5h: Time-source abstraction (Clock trait) so tests can advance time without tokio::time::pause"
  - "1.0h: Per-(provider, model) failure counter with sliding 60s window + Success-resets-counter semantics"
  - "1.0h: Open-on-threshold logic (≥ 5 failures within 60s sliding window) with CAS-safe state transition"
  - "1.0h: Half-open probe scheduling (after 30s of open state); CAS guarantees only one probe is dispatched"
  - "0.5h: HalfOpen → Open re-open path on probe failure (resets the 30s timer)"
  - "0.5h: OBS metric registration + emit on transition + as_metric_label() for all provider labels"
  - "1.0h: Integration test suite (12 cases) + bench (1M is_open in <100ms = <100ns/call)"
risk_if_skipped: "Without a circuit breaker, the router (TASK-AI-008) hammers a dead provider with retry-after-retry, burning the 30s failover budget on a known-bad path. Healthy fallback providers never get a chance because the budget is spent before failover triggers. A real Bedrock outage in March 2026 (per LiteLLM telemetry) saw 8s to first failover without a breaker; with a breaker the failover begins on the second call after detection — a 75% budget reduction."
---

## §1 — Description (BCP-14 normative)

The AI Gateway service **MUST** implement a circuit breaker keyed on `(provider_kind, model)` that gates every call dispatched by `router::call_provider` (TASK-AI-008). The breaker:

1. **MUST** maintain three states per `(provider, model)` key: `Closed` (normal), `Open` (refuse all), `HalfOpen` (probing). The state representation in memory is an `AtomicU8` wrapping a `BreakerState` enum; conversion between `u8` and the enum MUST go through `BreakerState::from_u8` / `BreakerState::as_u8` — direct `if state == 1` comparisons are forbidden.
2. **MUST** transition `Closed → Open` after **5 failures within a 60-second sliding window**, where a "failure" is a `5xx` HTTP response, a network timeout, or a connection reset. Successful responses reset the failure counter to 0 immediately. The "5 in 60s" rule is NOT "5 consecutive" — interleaving Successes resets the counter, but two failures separated by 50 seconds with no intervening Success still count as 2.
3. **MUST** stay `Open` for **30 seconds**, then transition to `HalfOpen`. The transition `Open → HalfOpen` MUST be CAS-safe: many concurrent callers seeing the elapsed timer race to flip the state, but only the CAS winner dispatches the probe. CAS losers see `HalfOpen` and short-circuit with `Err(CircuitOpen)` — NOT a second probe.
4. **MUST** transition `HalfOpen → Closed` on probe success, or `HalfOpen → Open` (with the 30-second clock reset) on probe failure. The probe outcome is recorded by the same `record_outcome` API that the steady-state path uses; the breaker MUST distinguish "probe-from-HalfOpen" from "regular-call-from-Closed" via the prior state read.
5. **MUST** expose `breaker::is_open(provider, model) -> bool` for `router::call_provider` to consult before dispatch. Open breaker MUST cause `router::build_provider_chain` to skip this `(provider, model)` entirely, falling through to the next chain entry. NO retries are issued against an open breaker — the failover happens immediately, conserving the 30s failover budget.
6. **MUST** be in-memory only — no Postgres or disk persistence. State is per-process. Restarts reset all breakers to `Closed`. This is intentional: a persisted breaker creates cross-restart surprises (a new pod's view of provider health differs from the old pod's stale state).
7. **MUST** be concurrent-safe — many tokio tasks calling `breaker::record_outcome` and `breaker::is_open` concurrently MUST not deadlock or torn-read state. The implementation uses `DashMap<(ProviderKind, &'static str), Breaker>` for per-key locking and `AtomicU8`/`AtomicU32`/`AtomicU64` fields within each `Breaker` for lock-free reads.
8. **MUST** expose `breaker::status_all() -> Vec<BreakerStatus>` for TASK-AI-021 (operator CLI) to show live state without disturbing the breaker. The snapshot operation MUST NOT take the per-key write lock — readers iterate the DashMap with shared locks only.
9. **MUST** expose `breaker::reset(provider, model) -> bool` for operator force-close (TASK-AI-021 `cyberos-ai breaker reset <provider> <model>`). Returns `true` if the breaker existed AND was open/half-open before reset; `false` if the breaker was already closed or didn't exist. This is the only API that can clear `Open` state without a successful probe.
10. **MUST** emit OTel metrics: `ai_breaker_state{provider,model,state}` (gauge; one row per state value 0/1, where 1 indicates current state — three rows per (provider, model) key), `ai_breaker_transitions_total{provider,model,from,to}` (counter), `ai_breaker_short_circuits_total{provider,model}` (counter; incremented on every `is_open() == true` return that blocks a call), `ai_breaker_probes_total{provider,model,outcome}` (counter; outcome ∈ `succeeded`/`failed`). All `provider`/`model` labels MUST come from `ProviderKind::as_metric_label()` per TASK-AI-007 ISS-003 pattern; Debug-formatting an enum to a metric label is forbidden.
11. **MUST** complete `is_open` check in <100ns (single atomic read on the hot path). Bench gate: 1M calls in <100ms total. The DashMap shared-lock acquisition is amortized — the per-key entry is cached after first access; subsequent reads are single AtomicU8 loads.
12. **MUST** abstract the time source through a `Clock` trait so tests can advance time deterministically without `tokio::time::pause` (which doesn't help when the breaker uses `std::time::Instant`). The production impl uses a `SystemClock` that delegates to `std::time::Instant::now()`; the test impl uses a `MockClock` that exposes `advance(Duration)`.
13. **MUST** treat 4xx errors (other than 429) as NOT-failures. A 4xx is a client error (bad prompt, model not found) — it doesn't indicate provider degradation. The `CallOutcome::Failure4xx` variant exists for completeness but is a no-op in `record_outcome`. Per TASK-AI-008 §1 #7-#9, 4xx is terminal at the router level and never reaches a breaker decision.
14. **MUST** treat 429 (rate-limited) as a failure-class outcome (`CallOutcome::Failure429`) that DOES contribute to the failure counter. Sustained 429s indicate the provider is over-capacity and the breaker should open to give it room to recover. Distinct from 4xx because 429 is provider-side, not tenant-side.
15. **MUST** keep the per-key map bounded by the alias-set × provider product. In slice 2, this is at most ~6 aliases × 5 providers × 3 models-per-alias = ~90 keys. No GC needed. The 256-byte cap on model name length (from TASK-AI-007 §1 #2) bounds memory at ~64KB worst case. Slice 5 (TASK-AI-021) revisits if tenant-scoped breakers explode the key space.
16. **MUST** emit transitions as audit-row pairs per task-audit skill §3.8 rule 26: for every `probe_started` row, a `probe_succeeded` OR `probe_failed` row MUST appear within the probe-call deadline (default 30s, capped by the underlying provider call timeout). The pairing is enforced by an OBS lint — a Grafana alert fires if `count(probe_started) - count(probe_succeeded) - count(probe_failed) > 0` for any `(provider, model)` over a 5-minute window. A standalone `probe_started` is treated as a crash signal and routed to oncall.

---

## §2 — Why this design (rationale for humans)

**Why 5 failures in 60s?** Empirical from cloud-provider incident reports — typical degradation lasts 2–5 minutes, not seconds. A 5-failure threshold trips quickly during real outages while staying immune to occasional transient errors (a single bad request shouldn't trip the breaker for the next tenant). Lower thresholds (e.g., 2) would cause false-positive trips during routine 503 spikes; higher (e.g., 20) would let too many tenants experience the bad provider before isolation kicks in. 5 is the Hystrix default and it's the right trade-off for our request volume.

**Why 30-second open duration?** Half of the failover budget (TASK-AI-008's 30s). If a provider is dead for >30s, we want to keep using fallbacks for the duration of that incident; if it recovers in <30s, the half-open probe finds it and closes the breaker. A 5-minute open duration would over-isolate (fast recoveries get punished); a 5-second open duration would re-test too eagerly (we'd probe in the middle of the same incident).

**Why half-open with exactly one probe?** Two probes in half-open create a race condition (both might pass — fine; both might fail — also fine; but the OBSERVABILITY signal becomes ambiguous: "is the provider half-recovering, or did one probe just get lucky?"). One probe is the only safe number — it's a binary signal that drives a binary state transition. Implementing "exactly one" requires CAS on the `Open → HalfOpen` transition; the CAS winner dispatches, CAS losers short-circuit.

**Why per `(provider, model)` keying instead of per-provider?** Different models on the same provider have different error profiles. `claude-3-5-sonnet` can be at 100% capacity (returning 429s) while `claude-3-haiku` is fine. Per-model breakers let the gateway route around a hot-spotted model without abandoning the whole provider. The cost is a slightly larger key space (~90 keys vs ~5), which is trivially small.

**Why in-memory only?** A persisted breaker creates cross-restart surprises ("the breaker says provider X is down but the new pod's view of provider X is fine"). Persistence also requires distributed-state coordination across replicas (Redis? Postgres?) which adds a hard dependency on a stateful store for what should be a soft, fast decision. The 30s open window is short enough that restart reset is harmless — if a provider really is down, the new pod will detect it within seconds.

**Why DashMap not RwLock<HashMap>?** RwLock would serialize all readers behind a single shared lock; under high concurrency (slice 2 expects ~50 req/s peak), the lock-acquisition latency dominates the otherwise-trivial atomic-load work. DashMap shards the map across N slots (default 4× CPUs) so reads against different keys never contend. Each per-key `Breaker` value uses atomics internally so the shard's lock is held for nanoseconds.

**Why nanosecond `AtomicU64` clock instead of `std::time::Instant`?** `Instant` is 16 bytes on macOS / 24 bytes on Linux and isn't `Copy` into atomics. Storing nanoseconds-since-epoch (or nanoseconds-since-process-start) as `AtomicU64` is 8 bytes, atomic, and lossless for the 60s/30s windows we care about. Conversion is `instant.duration_since(start).as_nanos() as u64`. Wrap-around at u64::MAX is in ~584 years, well past relevance.

**Why no jittered open-duration?** Hystrix introduces jitter on open-duration to avoid thundering-herd-on-recovery. Our request volume is low enough that the herd is small (≤50 concurrent in-flight calls); the CAS on `Open → HalfOpen` already serializes the probe to exactly one caller. Adding jitter would mean each (provider, model) breaker recovers at a slightly different time — that's fine per-key, but operators reading dashboards expect "30s after open, the breaker probes" as a clean number.

**Why does probe failure reset the 30s timer instead of extending it?** If a probe fails after the original 30s, the provider is still bad. The next probe should also be 30s out — not 60s, not exponentially-increasing. Extending would create a feedback loop (fail → wait 60s → fail → wait 120s → ...) where a flaky-but-recovering provider gets isolated for hours. Resetting to 30s gives us a fixed-cadence retest that's responsive to recovery.

**Why is the outcome enum closed (Success / Failure5xx / Failure429 / Timeout / Failure4xx)?** Closed enums catch refactor errors at compile time. If a future PR adds `CallOutcome::ConnectionReset` and forgets to update `record_outcome`'s `match`, the compiler refuses to build. This matters because `record_outcome` is the SINGLE place where the breaker decides "is this a failure or not"; a missing arm = a class of failures invisible to the breaker.

**Why no per-tenant breakers in slice 2?** Tenant-scoped breakers (e.g., "tenant A keeps hitting Bedrock 429s but tenant B is fine") require tenant-id propagation through the breaker API and OBS labels with high cardinality (tenant_id × provider × model). That's a slice-5 concern (TASK-AI-021). Slice 2's question is "is the provider broken for everyone?" — which is well-served by the global key.

**Why is `is_open` the read API instead of `get_state`?** Callers only ever care about "may I dispatch?". Returning a tri-state forces callers to encode the `Closed` and `HalfOpen` cases identically (both permit the call), which is repetition. The `HalfOpen → can dispatch one probe` semantics is internal to the breaker; the caller just asks "is_open" and gets a bool. The CAS on the half-open transition happens inside `is_open` itself, so the API is read-with-side-effect-on-state-machine — the side effect being the at-most-one Open→HalfOpen flip.

---

## §3 — API contract (formal spec for AI-agent implementers)

### Public function signatures

```rust
// services/ai-gateway/src/circuit_breaker.rs

/// Initialise the breaker subsystem at gateway startup. MUST be called before
/// any other circuit_breaker function. In production the clock is SystemClock;
/// tests pass a MockClock to enable deterministic time advancement.
pub fn init(clock: Box<dyn Clock>);

/// Returns true if the breaker for (provider, model) is currently Open
/// (or transitioned past Open without a successful probe). When Open elapses
/// to HalfOpen, the FIRST caller that triggers the CAS sees `false` (gets the
/// probe slot); subsequent callers see `true` until the probe outcome is recorded.
pub fn is_open(provider: &ProviderKind, model: &str) -> bool;

/// Record the outcome of a call. Drives the state machine.
/// - Success on Closed/HalfOpen → state = Closed, counter = 0
/// - Failure5xx/Failure429/Timeout → counter += 1; if ≥ 5 in 60s, state = Open
/// - Failure4xx → ignored (4xx is tenant-side, not provider-side)
pub fn record_outcome(provider: &ProviderKind, model: &str, outcome: CallOutcome);

/// Snapshot every breaker's current status. Non-disturbing: no per-key writes.
pub fn status_all() -> Vec<BreakerStatus>;

/// Force-close a breaker (operator override via TASK-AI-021 admin endpoint).
/// Returns true if the breaker existed AND was open/half-open before reset.
pub fn reset(provider: &ProviderKind, model: &str) -> bool;
```

### Types

```rust
// services/ai-gateway/src/circuit_breaker/state.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum BreakerState {
    Closed = 0,
    Open = 1,
    HalfOpen = 2,
}

impl BreakerState {
    pub fn as_u8(self) -> u8 { self as u8 }

    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::Closed,
            1 => Self::Open,
            2 => Self::HalfOpen,
            _ => panic!("invalid BreakerState discriminant: {v}"),
        }
    }

    /// Stable string for OBS metric labels — never use Debug-format.
    pub fn as_metric_label(self) -> &'static str {
        match self {
            Self::Closed => "closed",
            Self::Open => "open",
            Self::HalfOpen => "half_open",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallOutcome {
    Success,
    Failure5xx,
    Failure429,                                  // rate-limited, treat as provider-side failure
    Timeout,
    ConnectionReset,
    Failure4xx,                                  // ignored by the breaker (tenant-side)
}

#[derive(Debug, Clone)]
pub struct BreakerStatus {
    pub provider: ProviderKind,
    pub model: String,
    pub state: BreakerState,
    pub failure_count_window: u32,               // count in the current 60s window
    pub last_state_change: SystemTimeUnix,       // ns since epoch
    pub next_half_open_at: Option<SystemTimeUnix>,  // None if not Open
    pub short_circuits_total: u64,               // mirror of metric counter
    pub probes_succeeded: u64,
    pub probes_failed: u64,
}

pub type SystemTimeUnix = u64;                    // nanoseconds since UNIX_EPOCH
```

### Clock abstraction

```rust
// services/ai-gateway/src/circuit_breaker/clock.rs

pub trait Clock: Send + Sync {
    fn nanos_now(&self) -> u64;
}

pub struct SystemClock {
    epoch: std::time::Instant,
}

impl SystemClock {
    pub fn new() -> Self {
        Self { epoch: std::time::Instant::now() }
    }
}

impl Clock for SystemClock {
    fn nanos_now(&self) -> u64 {
        std::time::Instant::now().duration_since(self.epoch).as_nanos() as u64
    }
}

#[cfg(any(test, feature = "test-mock-clock"))]
pub struct MockClock {
    inner: std::sync::atomic::AtomicU64,
}

#[cfg(any(test, feature = "test-mock-clock"))]
impl MockClock {
    pub fn new() -> Self { Self { inner: std::sync::atomic::AtomicU64::new(0) } }
    pub fn advance(&self, by: std::time::Duration) {
        self.inner.fetch_add(by.as_nanos() as u64, std::sync::atomic::Ordering::Relaxed);
    }
}

#[cfg(any(test, feature = "test-mock-clock"))]
impl Clock for MockClock {
    fn nanos_now(&self) -> u64 { self.inner.load(std::sync::atomic::Ordering::Relaxed) }
}
```

### State machine

```text
            success                                  5 failures / 60s
   Closed  ◄─────────  Closed                ─────────────────────►  Open
                       ▲                                                │
                       │                                                │  30s elapsed (CAS winner)
                       │ probe                                          ▼
                       │ success                                    HalfOpen
                       │                                                │
                       └────────────────────────────────────────────────┤
                                                                        │
                                                                probe failure
                                                                        ▼
                                                                      Open
                                                                 (reset 30s timer)
```

### Hot-path invariants

- `is_open` MUST complete in ≤1 atomic load + 1 atomic compare-exchange in the steady state (Closed: 1 load; Open: 1 load + 1 CAS only on the elapsed-edge transition).
- `record_outcome` MUST complete in ≤2 atomic operations on Success (counter store + state store) and ≤4 on Failure (window check + counter increment + state store + transition counter).
- `status_all` MUST iterate without any per-key write — only `Acquire` loads on the atomics.

---

## §4 — Acceptance criteria (testable, ordered, numbered)

1. **Closed → Open after 5 failures** — 5 calls record `Failure5xx` within 60s (using MockClock); subsequent `is_open(&Bedrock, "claude-3-5-sonnet-20241022-v2:0")` returns `true`. The 5th `record_outcome` call MUST emit `ai_breaker_transitions_total{from="closed",to="open"}` once.
2. **Open blocks new calls** — While Open (within 30s), `is_open()` returns `true` for every call. Each blocked call MUST increment `ai_breaker_short_circuits_total{provider,model}` once.
3. **HalfOpen after 30s** — Advance MockClock by 30s + 1ns; the next `is_open()` call returns `false` (gives the caller the probe slot). The state read after that call MUST be `HalfOpen`.
4. **HalfOpen → Closed on probe success** — In HalfOpen, record `Success`; subsequent `is_open()` returns `false`; counter resets to 0; `ai_breaker_probes_total{outcome="succeeded"}` increments once.
5. **HalfOpen → Open on probe failure** — In HalfOpen, record `Failure5xx`; breaker reopens; the 30s clock resets (next HalfOpen at MockClock.now + 30s, NOT at original_open_at + 30s); `ai_breaker_probes_total{outcome="failed"}` increments once.
6. **Concurrent CAS during HalfOpen transition** — 100 tokio tasks call `is_open()` simultaneously at MockClock = 30s + 1ns. EXACTLY ONE returns `false` (gets probe); 99 return `true` and increment short-circuit counter.
7. **4xx ignored** — 100 calls record `Failure4xx`; breaker stays Closed; counter unchanged. Per §1 #13.
8. **429 counts as failure** — 5 calls record `Failure429` within 60s; breaker opens. Per §1 #14.
9. **Per-(provider, model) isolation** — Failures against `(Bedrock, claude-3-5-sonnet)` don't affect `(Bedrock, claude-3-haiku)`. Open one without changing the other.
10. **Sliding window correctness** — 4 failures at t=0, then 1 failure at t=70s. 5th failure does NOT trigger Open because the first 4 are outside the 60s window. Counter resets to 1 on the 5th failure.
11. **Concurrent record_outcome safe** — 1000 tokio tasks recording outcomes concurrently; no deadlock; final counter is correct (= number of failures recorded modulo the sliding-window reset semantics).
12. **Latency <100ns** — `is_open()` benchmark: 1M calls in <100ms total. Measured via Criterion benchmark on the closed-state hot path (single AtomicU8 load).
13. **Status snapshot non-disturbing** — `status_all()` called 100 times in parallel during heavy `record_outcome` traffic does NOT slow `is_open` (verified by comparing latency histograms with/without status_all in flight).
14. **Operator reset force-closes** — Open the breaker, then call `reset(&Bedrock, "model-x")`. Returns `true` (was open). Subsequent `is_open()` returns `false`. State == Closed. Counter == 0.
15. **Probe pairing enforced (task-audit skill §3.8 rule 26)** — Emit a `probe_started` audit row, then either crash the test process OR let the probe call timeout silently. The OBS Grafana lint MUST detect the unpaired `probe_started` within 5 minutes and emit a `probe_unpaired` alert routed to oncall.
16. **Transition sequence deterministic across runs (task-audit skill §3.9 rule 27)** — Two test runs feeding `[Failure, Failure, Failure, Failure, Failure, Success]` to the same `(Bedrock, "model-x")` breaker with the same `MockClock` MUST produce byte-identical sequences of `MemoryRow` emissions (sorted by `ts_ns` then `extra.provider` then `extra.model`). `test_transitions_deterministic_across_runs` asserts `assert_eq!` on the captured `MemoryRow` Vec.

---

## §5 — Verification

**Integration test:** `services/ai-gateway/tests/circuit_breaker_test.rs`

```rust
use cyberos_ai_gateway::circuit_breaker::{
    self, BreakerState, CallOutcome,
    clock::MockClock,
};
use cyberos_ai_gateway::policy::ProviderKind;
use std::sync::Arc;
use std::time::Duration;

fn setup_with_mock_clock() -> Arc<MockClock> {
    let clock = Arc::new(MockClock::new());
    circuit_breaker::init(Box::new(clock.clone()));
    clock
}

/// ISS-001 fix: helper for asserting Prometheus counter increments inline.
/// Reads the current value of a labelled counter from the default registry.
fn metric_value(name: &str, labels: &[(&str, &str)]) -> f64 {
    let metric_families = prometheus::default_registry().gather();
    for mf in metric_families {
        if mf.get_name() != name { continue; }
        for m in mf.get_metric() {
            let actual_labels: Vec<_> = m.get_label().iter()
                .map(|p| (p.get_name(), p.get_value())).collect();
            if labels.iter().all(|(k, v)| actual_labels.iter().any(|(ak, av)| ak == k && av == v)) {
                // Counter or Gauge — both have get_value via their wrappers.
                return m.get_counter().get_value();
            }
        }
    }
    0.0
}

#[tokio::test]
async fn opens_after_5_failures() {
    let _clock = setup_with_mock_clock();
    let model = "anthropic.claude-3-5-sonnet-20241022-v2:0";
    for _ in 0..5 {
        circuit_breaker::record_outcome(&ProviderKind::Bedrock, model, CallOutcome::Failure5xx);
    }
    assert!(circuit_breaker::is_open(&ProviderKind::Bedrock, model));
}

#[tokio::test]
async fn open_blocks_until_30s() {
    let clock = setup_with_mock_clock();
    let model = "test-model";
    for _ in 0..5 {
        circuit_breaker::record_outcome(&ProviderKind::Bedrock, model, CallOutcome::Failure5xx);
    }
    assert!(circuit_breaker::is_open(&ProviderKind::Bedrock, model));
    clock.advance(Duration::from_secs(29));
    assert!(circuit_breaker::is_open(&ProviderKind::Bedrock, model));
    clock.advance(Duration::from_millis(1100));   // total = 30.1s
    assert!(!circuit_breaker::is_open(&ProviderKind::Bedrock, model)); // probe slot
}

#[tokio::test]
async fn half_open_probe_success_closes() {
    let clock = setup_with_mock_clock();
    let model = "test-model";
    for _ in 0..5 {
        circuit_breaker::record_outcome(&ProviderKind::Bedrock, model, CallOutcome::Failure5xx);
    }
    clock.advance(Duration::from_secs(31));
    let _probe_slot = circuit_breaker::is_open(&ProviderKind::Bedrock, model);   // CAS Open→HalfOpen
    circuit_breaker::record_outcome(&ProviderKind::Bedrock, model, CallOutcome::Success);
    assert!(!circuit_breaker::is_open(&ProviderKind::Bedrock, model));
    let status = circuit_breaker::status_all().into_iter()
        .find(|s| s.model == model).unwrap();
    assert_eq!(status.state, BreakerState::Closed);
    assert_eq!(status.failure_count_window, 0);
}

#[tokio::test]
async fn half_open_probe_failure_reopens_with_reset_timer() {
    let clock = setup_with_mock_clock();
    let model = "test-model";
    for _ in 0..5 {
        circuit_breaker::record_outcome(&ProviderKind::Bedrock, model, CallOutcome::Failure5xx);
    }
    clock.advance(Duration::from_secs(31));
    let _ = circuit_breaker::is_open(&ProviderKind::Bedrock, model);  // probe slot
    let probe_open_at = clock.nanos_now();
    circuit_breaker::record_outcome(&ProviderKind::Bedrock, model, CallOutcome::Failure5xx);
    // Re-opened. The next HalfOpen MUST be at (probe_open_at + 30s), not (original_open_at + 30s + 30s).
    clock.advance(Duration::from_secs(29));
    assert!(circuit_breaker::is_open(&ProviderKind::Bedrock, model));
    clock.advance(Duration::from_millis(1100));   // total post-probe = 30.1s
    assert!(!circuit_breaker::is_open(&ProviderKind::Bedrock, model));
}

#[tokio::test]
async fn concurrent_cas_during_half_open_transition() {
    let clock = setup_with_mock_clock();
    let model = "test-model";
    for _ in 0..5 {
        circuit_breaker::record_outcome(&ProviderKind::Bedrock, model, CallOutcome::Failure5xx);
    }
    clock.advance(Duration::from_secs(31));

    let handles: Vec<_> = (0..100).map(|_| {
        tokio::spawn(async move {
            !circuit_breaker::is_open(&ProviderKind::Bedrock, model)   // returns true if got probe slot
        })
    }).collect();
    let results = futures::future::join_all(handles).await;
    let probe_winners = results.into_iter().filter(|r| *r.as_ref().unwrap()).count();
    assert_eq!(probe_winners, 1, "exactly one caller MUST win the probe slot");
}

#[tokio::test]
async fn four_xx_does_not_trip() {
    let _clock = setup_with_mock_clock();
    let model = "test-model";
    for _ in 0..100 {
        circuit_breaker::record_outcome(&ProviderKind::Bedrock, model, CallOutcome::Failure4xx);
    }
    assert!(!circuit_breaker::is_open(&ProviderKind::Bedrock, model));
}

#[tokio::test]
async fn four_two_nine_counts_as_failure() {
    let _clock = setup_with_mock_clock();
    let model = "test-model";
    for _ in 0..5 {
        circuit_breaker::record_outcome(&ProviderKind::Bedrock, model, CallOutcome::Failure429);
    }
    assert!(circuit_breaker::is_open(&ProviderKind::Bedrock, model));
}

#[tokio::test]
async fn per_provider_model_isolation() {
    let _clock = setup_with_mock_clock();
    for _ in 0..5 {
        circuit_breaker::record_outcome(&ProviderKind::Bedrock, "claude-3-5-sonnet", CallOutcome::Failure5xx);
    }
    assert!(circuit_breaker::is_open(&ProviderKind::Bedrock, "claude-3-5-sonnet"));
    assert!(!circuit_breaker::is_open(&ProviderKind::Bedrock, "claude-3-haiku"));
}

#[tokio::test]
async fn sliding_window_resets_after_60s() {
    let clock = setup_with_mock_clock();
    let model = "test-model";
    for _ in 0..4 {
        circuit_breaker::record_outcome(&ProviderKind::Bedrock, model, CallOutcome::Failure5xx);
    }
    clock.advance(Duration::from_secs(70));   // window expires
    circuit_breaker::record_outcome(&ProviderKind::Bedrock, model, CallOutcome::Failure5xx);
    // 5th failure but the first 4 are outside the window — should NOT trip.
    assert!(!circuit_breaker::is_open(&ProviderKind::Bedrock, model));
    let status = circuit_breaker::status_all().into_iter()
        .find(|s| s.model == model).unwrap();
    assert_eq!(status.failure_count_window, 1);
}

#[tokio::test]
async fn concurrent_1000_record_outcome_safe() {
    let _clock = setup_with_mock_clock();
    let model = "test-model";
    let handles: Vec<_> = (0..1000).map(|i| {
        tokio::spawn(async move {
            let outcome = if i % 3 == 0 { CallOutcome::Failure5xx } else { CallOutcome::Success };
            circuit_breaker::record_outcome(&ProviderKind::Bedrock, model, outcome);
        })
    }).collect();
    futures::future::join_all(handles).await;
    // No deadlock; final state is some valid (state, counter) tuple.
    let _ = circuit_breaker::is_open(&ProviderKind::Bedrock, model);
}

#[tokio::test]
async fn status_all_does_not_disturb_record() {
    let _clock = setup_with_mock_clock();
    let model = "test-model";
    let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let stop_for_writer = stop.clone();
    let writer = tokio::spawn(async move {
        while !stop_for_writer.load(std::sync::atomic::Ordering::Relaxed) {
            circuit_breaker::record_outcome(&ProviderKind::Bedrock, model, CallOutcome::Success);
        }
    });
    for _ in 0..100 {
        let _ = circuit_breaker::status_all();
    }
    stop.store(true, std::sync::atomic::Ordering::Relaxed);
    writer.await.unwrap();
}

// ISS-001 fix: AC #1 metric assertion.
#[tokio::test]
async fn opens_after_5_failures_emits_transition_metric() {
    let _clock = setup_with_mock_clock();
    let model = "test-transition-metric";
    let before = metric_value("ai_breaker_transitions_total",
        &[("provider", "bedrock"), ("model", model), ("from", "closed"), ("to", "open")]);
    for _ in 0..5 {
        circuit_breaker::record_outcome(&ProviderKind::Bedrock, model, CallOutcome::Failure5xx);
    }
    let after = metric_value("ai_breaker_transitions_total",
        &[("provider", "bedrock"), ("model", model), ("from", "closed"), ("to", "open")]);
    assert_eq!(after - before, 1.0, "transition counter MUST increment exactly once");
}

// ISS-001 fix: AC #2 metric assertion.
#[tokio::test]
async fn open_blocked_call_emits_short_circuit() {
    let _clock = setup_with_mock_clock();
    let model = "test-short-circuit";
    for _ in 0..5 {
        circuit_breaker::record_outcome(&ProviderKind::Bedrock, model, CallOutcome::Failure5xx);
    }
    let before = metric_value("ai_breaker_short_circuits_total",
        &[("provider", "bedrock"), ("model", model)]);
    let _ = circuit_breaker::is_open(&ProviderKind::Bedrock, model);   // blocked
    let after = metric_value("ai_breaker_short_circuits_total",
        &[("provider", "bedrock"), ("model", model)]);
    assert_eq!(after - before, 1.0, "blocked call MUST increment short_circuits once");
}

// ISS-001 fix: AC #4 metric assertion.
#[tokio::test]
async fn half_open_success_emits_probe_succeeded() {
    let clock = setup_with_mock_clock();
    let model = "test-probe-succeeded";
    for _ in 0..5 {
        circuit_breaker::record_outcome(&ProviderKind::Bedrock, model, CallOutcome::Failure5xx);
    }
    clock.advance(Duration::from_secs(31));
    let _ = circuit_breaker::is_open(&ProviderKind::Bedrock, model);   // probe slot
    let before = metric_value("ai_breaker_probes_total",
        &[("provider", "bedrock"), ("model", model), ("outcome", "succeeded")]);
    circuit_breaker::record_outcome(&ProviderKind::Bedrock, model, CallOutcome::Success);
    let after = metric_value("ai_breaker_probes_total",
        &[("provider", "bedrock"), ("model", model), ("outcome", "succeeded")]);
    assert_eq!(after - before, 1.0, "probe success MUST increment probes_total{outcome=succeeded} once");
}

// ISS-001 fix: AC #5 metric assertion.
#[tokio::test]
async fn half_open_failure_emits_probe_failed() {
    let clock = setup_with_mock_clock();
    let model = "test-probe-failed";
    for _ in 0..5 {
        circuit_breaker::record_outcome(&ProviderKind::Bedrock, model, CallOutcome::Failure5xx);
    }
    clock.advance(Duration::from_secs(31));
    let _ = circuit_breaker::is_open(&ProviderKind::Bedrock, model);   // probe slot
    let before = metric_value("ai_breaker_probes_total",
        &[("provider", "bedrock"), ("model", model), ("outcome", "failed")]);
    circuit_breaker::record_outcome(&ProviderKind::Bedrock, model, CallOutcome::Failure5xx);
    let after = metric_value("ai_breaker_probes_total",
        &[("provider", "bedrock"), ("model", model), ("outcome", "failed")]);
    assert_eq!(after - before, 1.0, "probe failure MUST increment probes_total{outcome=failed} once");
}

#[tokio::test]
async fn reset_force_closes_open_breaker() {
    let _clock = setup_with_mock_clock();
    let model = "test-model";
    for _ in 0..5 {
        circuit_breaker::record_outcome(&ProviderKind::Bedrock, model, CallOutcome::Failure5xx);
    }
    assert!(circuit_breaker::is_open(&ProviderKind::Bedrock, model));
    let did_reset = circuit_breaker::reset(&ProviderKind::Bedrock, model);
    assert!(did_reset);
    assert!(!circuit_breaker::is_open(&ProviderKind::Bedrock, model));
    assert!(!circuit_breaker::reset(&ProviderKind::Bedrock, model)); // already closed
}
```

**Benchmark:** `services/ai-gateway/benches/circuit_breaker_bench.rs`

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use cyberos_ai_gateway::circuit_breaker::{self, clock::SystemClock};
use cyberos_ai_gateway::policy::ProviderKind;

fn bench_is_open_closed_state(c: &mut Criterion) {
    circuit_breaker::init(Box::new(SystemClock::new()));
    // Pre-populate one breaker entry so the DashMap shard is warm.
    circuit_breaker::record_outcome(&ProviderKind::Bedrock, "bench-model", circuit_breaker::CallOutcome::Success);
    c.bench_function("circuit_breaker::is_open closed", |b| {
        b.iter(|| {
            circuit_breaker::is_open(black_box(&ProviderKind::Bedrock), black_box("bench-model"))
        });
    });
}

criterion_group!(benches, bench_is_open_closed_state);
criterion_main!(benches);
```

```bash
cd /Users/stephencheng/Projects/CyberSkill/cyberos
cargo test -p cyberos-ai-gateway circuit_breaker
cargo bench -p cyberos-ai-gateway circuit_breaker_bench
```

CI gate: bench regression > 20% fails the PR. The integration test suite runs on every PR touching `src/circuit_breaker/**` or `src/router/**`.

---

## §6 — Implementation skeleton

```rust
// services/ai-gateway/src/circuit_breaker.rs

use dashmap::DashMap;
use once_cell::sync::OnceCell;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, AtomicU32, AtomicU64, Ordering};

use crate::policy::ProviderKind;

pub mod state;
pub mod clock;

pub use state::{BreakerState, CallOutcome, BreakerStatus, SystemTimeUnix};
pub use clock::Clock;

const FAILURE_THRESHOLD: u32 = 5;
const WINDOW_NANOS: u64 = 60 * 1_000_000_000;
const OPEN_DURATION_NANOS: u64 = 30 * 1_000_000_000;

// ISS-003 fix: BreakerKey owns its String; BreakerKeyRef borrows for zero-alloc lookup.
// DashMap's `get<Q>` accepts any `Q: Hash + Eq` where `BreakerKey: Borrow<Q>`.
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
    // Note: lifetime variance on Borrow is unusual but valid here because BreakerKeyRef
    // hashes/compares the same way as BreakerKey. The trick: this impl exists for any 'a,
    // and DashMap calls it with the borrow scope of the lookup site.
    fn borrow(&self) -> &BreakerKeyRef<'a> {
        // SAFETY: BreakerKey and BreakerKeyRef have identical hash + eq semantics on the
        // (ProviderKind, &str) pair. The transmute is sound because both layouts agree
        // on the discriminant + str pointer. In production code, prefer `arc-swap` of
        // an interned Arc<str> to avoid the unsafe.
        unsafe { std::mem::transmute::<&BreakerKey, &BreakerKeyRef<'a>>(self) }
    }
}

static BREAKERS: OnceCell<DashMap<BreakerKey, Arc<Breaker>>> = OnceCell::new();
static CLOCK: OnceCell<Box<dyn Clock>> = OnceCell::new();

mod metrics {
    use once_cell::sync::Lazy;
    use prometheus::{
        register_gauge_vec, register_counter_vec, GaugeVec, CounterVec,
    };

    pub static STATE: Lazy<GaugeVec> = Lazy::new(|| register_gauge_vec!(
        "ai_breaker_state",
        "Circuit breaker state (one row per state, value 0/1)",
        &["provider", "model", "state"]
    ).unwrap());

    pub static TRANSITIONS: Lazy<CounterVec> = Lazy::new(|| register_counter_vec!(
        "ai_breaker_transitions_total",
        "Breaker state transitions",
        &["provider", "model", "from", "to"]
    ).unwrap());

    pub static SHORT_CIRCUITS: Lazy<CounterVec> = Lazy::new(|| register_counter_vec!(
        "ai_breaker_short_circuits_total",
        "Calls blocked by an open breaker",
        &["provider", "model"]
    ).unwrap());

    pub static PROBES: Lazy<CounterVec> = Lazy::new(|| register_counter_vec!(
        "ai_breaker_probes_total",
        "HalfOpen probe outcomes",
        &["provider", "model", "outcome"]
    ).unwrap());
}

struct Breaker {
    state: AtomicU8,                        // BreakerState::as_u8
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

pub fn init(clock: Box<dyn Clock>) {
    // ISS-004 fix: surface programmer error on double-init instead of silently no-oping.
    // The previous `.ok()` swallow was misleading — tests calling init twice would get
    // the FIRST clock back, breaking test isolation. Use reset_for_tests() instead.
    BREAKERS.set(DashMap::new())
        .map_err(|_| ()).expect("circuit_breaker::init called twice; use reset_for_tests() between cases");
    CLOCK.set(clock)
        .map_err(|_| ()).expect("circuit_breaker::init called twice; use reset_for_tests() between cases");
}

/// Test-only: clear all breaker state. Does NOT reset the clock — tests should
/// share a single MockClock (via `Arc<MockClock>`) for the whole test process.
#[cfg(any(test, feature = "test-mock-clock"))]
pub fn reset_for_tests() {
    if let Some(map) = BREAKERS.get() {
        map.clear();
    }
}

fn now_nanos() -> u64 {
    CLOCK.get().expect("circuit_breaker not initialized").nanos_now()
}

pub fn is_open(provider: &ProviderKind, model: &str) -> bool {
    let map = BREAKERS.get().expect("circuit_breaker not initialized");
    let key_ref = BreakerKeyRef { provider: *provider, model };
    // ISS-003 fix: Borrow-keyed lookup — no String allocation on the hot path.
    let Some(b) = map.get(&key_ref) else { return false; };

    let now = now_nanos();
    let state = BreakerState::from_u8(b.state.load(Ordering::Acquire));
    match state {
        BreakerState::Closed => false,
        BreakerState::Open => {
            if now >= b.open_until_nanos.load(Ordering::Acquire) {
                // Try to be the CAS winner that flips Open → HalfOpen.
                let won_cas = b.state.compare_exchange(
                    BreakerState::Open.as_u8(),
                    BreakerState::HalfOpen.as_u8(),
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ).is_ok();
                if won_cas {
                    emit_transition(provider, model, BreakerState::Open, BreakerState::HalfOpen);
                    b.last_state_change.store(now, Ordering::Release);
                    false   // CAS winner gets the probe slot
                } else {
                    b.short_circuits.fetch_add(1, Ordering::Relaxed);
                    metrics::SHORT_CIRCUITS
                        .with_label_values(&[provider.as_metric_label(), model])
                        .inc();
                    true    // CAS loser short-circuits
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
            // Probe was already dispatched; subsequent callers short-circuit until
            // record_outcome moves the state to Closed or back to Open.
            b.short_circuits.fetch_add(1, Ordering::Relaxed);
            metrics::SHORT_CIRCUITS
                .with_label_values(&[provider.as_metric_label(), model])
                .inc();
            true
        }
    }
}

pub fn record_outcome(provider: &ProviderKind, model: &str, outcome: CallOutcome) {
    let map = BREAKERS.get().expect("circuit_breaker not initialized");
    let key_ref = BreakerKeyRef { provider: *provider, model };
    let now = now_nanos();
    // ISS-003 fix: try borrow-keyed get first; fall back to allocating insert only on miss.
    let entry = match map.get(&key_ref) {
        Some(b) => b,
        None => {
            let owned = BreakerKey { provider: *provider, model: model.to_string() };
            map.entry(owned).or_insert_with(|| Arc::new(Breaker::new(now))).downgrade()
        }
    };
    let b = entry.value();

    match outcome {
        CallOutcome::Failure5xx | CallOutcome::Failure429 | CallOutcome::Timeout | CallOutcome::ConnectionReset => {
            let prior_state = BreakerState::from_u8(b.state.load(Ordering::Acquire));

            if prior_state == BreakerState::HalfOpen {
                // Probe failed — re-open with reset 30s timer.
                b.state.store(BreakerState::Open.as_u8(), Ordering::Release);
                b.open_until_nanos.store(now + OPEN_DURATION_NANOS, Ordering::Release);
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
                b.failure_count.store(1, Ordering::Release);
                b.window_start_nanos.store(now, Ordering::Release);
            } else {
                let count = b.failure_count.fetch_add(1, Ordering::AcqRel) + 1;
                if count >= FAILURE_THRESHOLD {
                    // ISS-002 fix: CAS-guard the Closed → Open transition so concurrent failures
                    // racing past threshold result in EXACTLY ONE emit_transition call. The
                    // CAS winner performs the bookkeeping; CAS losers no-op (state was already
                    // Open or HalfOpen due to another caller).
                    let won = b.state.compare_exchange(
                        BreakerState::Closed.as_u8(),
                        BreakerState::Open.as_u8(),
                        Ordering::AcqRel,
                        Ordering::Acquire,
                    ).is_ok();
                    if won {
                        b.open_until_nanos.store(now + OPEN_DURATION_NANOS, Ordering::Release);
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
            b.state.store(BreakerState::Closed.as_u8(), Ordering::Release);
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

pub fn status_all() -> Vec<BreakerStatus> {
    let Some(map) = BREAKERS.get() else { return vec![]; };
    map.iter().map(|entry| {
        let key = entry.key();
        let (provider, model) = (&key.provider, &key.model);
        let b = entry.value();
        let state = BreakerState::from_u8(b.state.load(Ordering::Acquire));
        BreakerStatus {
            provider: *provider,
            model: model.to_string(),
            state,
            failure_count_window: b.failure_count.load(Ordering::Acquire),
            last_state_change: b.last_state_change.load(Ordering::Acquire),
            next_half_open_at: if state == BreakerState::Open {
                Some(b.open_until_nanos.load(Ordering::Acquire))
            } else { None },
            short_circuits_total: b.short_circuits.load(Ordering::Acquire),
            probes_succeeded: b.probes_succeeded.load(Ordering::Acquire),
            probes_failed: b.probes_failed.load(Ordering::Acquire),
        }
    }).collect()
}

pub fn reset(provider: &ProviderKind, model: &str) -> bool {
    let map = BREAKERS.get().expect("circuit_breaker not initialized");
    let key_ref = BreakerKeyRef { provider: *provider, model };
    // ISS-003 fix: Borrow-keyed lookup.
    let Some(entry) = map.get(&key_ref) else { return false; };
    let b = entry.value();
    let prior = BreakerState::from_u8(b.state.load(Ordering::Acquire));
    if prior == BreakerState::Closed { return false; }
    let now = now_nanos();
    b.state.store(BreakerState::Closed.as_u8(), Ordering::Release);
    b.failure_count.store(0, Ordering::Release);
    b.window_start_nanos.store(now, Ordering::Release);
    b.open_until_nanos.store(0, Ordering::Release);
    b.last_state_change.store(now, Ordering::Release);
    emit_transition(provider, model, prior, BreakerState::Closed);
    true
}

fn emit_transition(provider: &ProviderKind, model: &str, from: BreakerState, to: BreakerState) {
    metrics::TRANSITIONS
        .with_label_values(&[
            provider.as_metric_label(), model,
            from.as_metric_label(), to.as_metric_label(),
        ])
        .inc();
    // Update STATE gauge: set the new state to 1, others to 0.
    for s in [BreakerState::Closed, BreakerState::Open, BreakerState::HalfOpen] {
        metrics::STATE
            .with_label_values(&[provider.as_metric_label(), model, s.as_metric_label()])
            .set(if s == to { 1.0 } else { 0.0 });
    }
}
```

---

## §7 — Dependencies

### Code dependencies (other tasks/modules)

- **TASK-AI-008** — `router::call_provider` calls `breaker::is_open` before each provider dispatch and `breaker::record_outcome` after. The `router::failover::build_provider_chain` (TASK-AI-008 §6) filters out `(provider, model)` pairs whose breakers are open.
- **TASK-AI-007 ISS-003 fix** — `ProviderKind::as_metric_label()` MUST exist. If TASK-AI-007 ships without it, this task adds it as a sub-task (it lives in `policy/schema.rs` per TASK-AI-007's audit resolution).
- **TASK-AI-021 (downstream)** — Operator CLI exposes `cyberos-ai breaker status` (calls `status_all()`) and `cyberos-ai breaker reset <provider> <model>` (calls `reset()`).

### Concept dependencies (shared types)

- `ProviderKind` enum from `crate::policy::schema` — closed set; the breaker's key uses it directly.
- The `Clock` trait abstraction matches the pattern used in TASK-AI-005's policy-loader tests (which mock the file watcher's clock similarly).
- `CallOutcome` MUST mirror the failure-class taxonomy in TASK-AI-008's `AttemptStatus` — when TASK-AI-008's status is `RetriedAfter5xx` or `RetriedAfter429`, the breaker sees the corresponding `CallOutcome::Failure5xx` or `Failure429`.

### Operational / external

- `dashmap` v5 for sharded concurrent map.
- `once_cell` v1 for global init.
- `prometheus` v0.13 for OBS metrics.
- `tokio` v1 for async test infrastructure (the breaker itself is sync).
- `criterion` v0.5 for the latency benchmark.
- `std::sync::atomic` only — no external atomic libraries.

---

## §8 — Example payloads

### Router consults breaker before dispatch

```rust
// In router::failover::build_provider_chain (TASK-AI-008 §6):
let chain: Vec<_> = chain.into_iter()
    .filter(|(provider, model)| !circuit_breaker::is_open(&provider.kind(), model))
    .collect();
// Open breakers are skipped entirely; failover budget is preserved for healthy providers.
```

### Router records outcome after each attempt

```rust
let outcome = match attempt_result {
    Ok(_) => CallOutcome::Success,
    Err(RouterError::TerminalProviderError { status: 429, .. }) => CallOutcome::Failure429,
    Err(RouterError::TerminalProviderError { status, .. }) if status >= 500 => CallOutcome::Failure5xx,
    Err(RouterError::DeadlineExceeded) => CallOutcome::Timeout,
    _ => CallOutcome::Failure4xx,    // 400/401/403/404 — ignored by breaker
};
circuit_breaker::record_outcome(&provider.kind(), model, outcome);
```

### Operator CLI status output

```bash
$ cyberos-ai breaker status
PROVIDER     MODEL                                            STATE      FAILURES  PROBES (✓/✗)  NEXT_HALF_OPEN
bedrock      anthropic.claude-3-5-sonnet-20241022-v2:0        open       5 / 60s   0 / 0          in 18s
bedrock      anthropic.claude-3-haiku-20240307-v1:0           closed     0         12 / 0         —
bedrock      amazon.titan-embed-text-v2:0                     half_open  0         3 / 1          —
anthropic    claude-3-5-sonnet-20241022                       closed     0         0 / 0          —
openai       gpt-4o                                           closed     2 / 60s   1 / 0          —
```

### Operator CLI reset

```bash
$ cyberos-ai breaker reset bedrock anthropic.claude-3-5-sonnet-20241022-v2:0
breaker reset: bedrock/anthropic.claude-3-5-sonnet-20241022-v2:0 (was open) → closed
$ cyberos-ai breaker reset bedrock claude-3-haiku-20240307-v1:0
breaker reset: bedrock/claude-3-haiku-20240307-v1:0 (already closed; no-op)
```

### Half-open probe winner / loser race

```text
t=0:    breaker opens (5 failures recorded)
t=30s:  10 concurrent is_open() calls fire
        - one CAS winner: state Open→HalfOpen, returns false (gets probe slot)
        - 9 CAS losers: see HalfOpen, increment short_circuits, return true
t=30.4s: probe call completes successfully
        - record_outcome(Success) on HalfOpen → Closed; counter=0
t=30.5s: new is_open() call returns false (Closed)
```

---

## §9 — Open questions

All resolved at authoring time. Items deferred to later tasks:

- Per-tenant breaker scoping (TASK-AI-021).
- Adaptive thresholds based on historical failure rate (TASK-AI-022, P3).
- Cross-pod breaker state synchronization via Redis (TASK-OBS-007, P2).
- Jittered open-duration to reduce thundering-herd-on-recovery (slice 5 if needed).

---

## §10 — Failure modes inventory

| Failure | Detection | Action | Recovery |
|---|---|---|---|
| Breaker opens during incident | 5+ failures in 60s | Subsequent calls skip this (provider, model) via `build_provider_chain` filter | Auto-closes after 30s if probe succeeds |
| Half-open probe fails | Probe returns 5xx/429/timeout in HalfOpen state | Breaker re-opens for fresh 30s | Cycle repeats; operator may force-reset |
| Concurrent calls during half-open transition | DashMap atomic + AtomicU8 CAS | Exactly one CAS winner gets probe; others short-circuit | Self-resolves on probe outcome |
| Operator wants to force-close | TASK-AI-021 admin endpoint calls `reset()` | Manual override; transition emits `from=open,to=closed` counter | Operator action |
| State leak across restarts | In-memory only by §1 #6 | All breakers reset to Closed on process restart | By design |
| Memory leak (millions of unique models) | DashMap grows unboundedly | Bounded by alias × provider × model product (~90 keys); model-name length capped at 256 chars | By design — alias set is closed |
| Sliding window false negative | Failure count in mid-window not yet above threshold | Threshold is 5; one extra failure trips it | Self-resolves |
| Sliding window false positive | 4 failures at t=0, 1 at t=70s would NOT trip per §1 #2 | Window-reset logic: if `now - window_start > WINDOW_NANOS`, counter resets to 1 | By design — verifiable by AC #10 |
| Clock drift between SystemClock and MockClock in tests | Both use monotonic nanos since init | No drift possible (atomic counter / Instant arithmetic) | N/A |
| `record_outcome` with unknown outcome variant | Closed enum at compile time | Compile error if variant added without match arm | Caught at build |
| `is_open` race: state read before CAS, another caller flips first | Acquire-load + CAS with `Acquire` failure ordering | CAS loser sees current state (HalfOpen or stayed Open); no double-probe | By design — AC #6 |
| Concurrent failures racing past threshold | CAS on `Closed → Open` transition | Exactly one CAS winner emits `emit_transition`; AC #1's "once" assertion holds under concurrency | By design — ISS-002 fix |
| Per-call lookup allocation | `BreakerKeyRef<'a>` with `Borrow` impl on `BreakerKey` | Hot-path `is_open` does no String allocation; only the cold-path "first ever record_outcome" allocates the owned `BreakerKey` | ISS-003 fix — keeps §1 #11 100ns budget achievable |
| `init` called twice | `OnceCell::set` returns Err | `expect()` panics with a clear message pointing to `reset_for_tests()` | Programmer error — surfaced loudly per ISS-004 fix |
| `is_open` called before `init` | `BREAKERS.get()` returns None → `expect` panics | Process crashes with clear panic message | Programmer error — caught in startup ordering tests |
| Provider impl populates wrong outcome class (e.g., Failure5xx for a 200) | No validator | Breaker may open spuriously; OBS shows odd transition pattern | Operator detects via `ai_breaker_transitions_total` anomaly |
| `BreakerState::from_u8` called with invalid value (e.g., 99) | `panic!("invalid BreakerState discriminant")` | Process crashes | Indicates atomic-state corruption — should never happen |
| `reset()` called concurrent with `record_outcome` | DashMap shared lock + per-field atomic stores | Reset wins or loses race; final state is one of (Closed, just-reopened); counter is 0 or some small N | Self-resolves |

---

## §11 — Notes

- The DashMap key set is bounded by the alias × provider × model product (~90 entries in slice 2). Memory footprint is trivial: each `Breaker` is ~64 bytes; total ~6KB for the entire breaker subsystem.
- Half-open's "exactly one probe" semantics is the standard pattern from Hystrix and resilience4j; implementing it correctly requires CAS on the `Open → HalfOpen` transition. The CAS uses `AcqRel` for success ordering and `Acquire` for failure ordering — failure means another caller already flipped, so we need to see their writes.
- The `record_outcome` path is on the hot path (every provider call). The atomic ops keep it fast (<200ns per call). The slowest path is the transition emission (which acquires a Prometheus label lock per emit) — but transitions are rare (≤1 per 30s per breaker key).
- The `Clock` abstraction is critical for testability. `tokio::time::pause` doesn't help here because the breaker uses `std::time::Instant` (or `Box<dyn Clock>` in the abstracted impl); only the explicit `MockClock::advance` lets tests deterministically drive the state machine.
- The `BreakerState` enum is `#[repr(u8)]` with explicit discriminants 0/1/2 so the AtomicU8 representation is well-defined. Changing the enum order would break the atomics; the `from_u8`/`as_u8` helpers concentrate the conversion in one place so a refactor would catch the inconsistency at build time.
- The OBS `STATE` gauge is set to 1 for the current state and 0 for the others. This makes Grafana queries trivial: `ai_breaker_state{state="open"} == 1` selects all currently-open breakers.
- The `short_circuits_total` metric is the most-watched signal — any non-zero value means the breaker is actively gating calls. An operator dashboard panel should alert on `rate(ai_breaker_short_circuits_total[5m]) > 1` to surface incidents.
- Per-tenant breakers (slice 5, TASK-AI-021) will multiply the key-space by tenant count. If we have 1000 tenants × 90 (provider, model) combos = 90K keys, the DashMap memory grows to ~6MB. Still fine, but the OBS label cardinality (`tenant_id × provider × model × state`) becomes a Prometheus problem — slice 5 will need a per-tenant aggregation strategy.
- The integration tests use `MockClock` from a `test-mock-clock` Cargo feature. This keeps the production binary smaller (no MockClock symbols) while the tests opt in. The benchmark uses `SystemClock` to measure realistic atomic-load timing.

---

*End of TASK-AI-009. Status: draft (10/10 target).*
