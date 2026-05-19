---
fr_id: FR-AI-009
audited: 2026-05-16
auditor: manual (engineering-spec template)
verdict: PASS (after revision)
score_pre_revision: 7.5/10        # the first-pass compressed version (295 lines)
score_post_expansion: 9.0/10      # after expanding to FR-AI-001 depth (~770 lines)
score_post_revision: 10/10         # after 4 mechanical fixes
issues_open: 0
issues_resolved: 6
issues_critical: 0
template: engineering-spec@1
revised_at: 2026-05-16
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 ISSes)
---

## §1 — Verdict summary

FR-AI-009 was expanded from 295 lines (compressed first-pass) to ~770 lines matching FR-AI-001 depth. The expansion added 5 §1 clauses (#9 reset() API, #12 Clock abstraction, #13 4xx-ignored explicit, #14 429-as-failure-class, #15 key-bounding rationale), 6 additional §2 paragraphs (DashMap-vs-RwLock, nanos-AtomicU64-vs-Instant, no-jitter-rationale, probe-failure-resets-30s, closed-outcome-enum-rationale, no-tenant-scope-in-slice-2), full schema types in §3 (`BreakerState` with `#[repr(u8)]` and `from_u8`/`as_u8`/`as_metric_label` helpers, `CallOutcome` 6 variants, `BreakerStatus` with probe counters, `Clock` trait + `SystemClock`/`MockClock` impls), 4 additional §4 ACs (#6 concurrent CAS, #10 sliding window, #13 status_all non-disturbing, #14 reset force-close), full Rust integration test bodies in §5 (12 named tokio tests + Criterion benchmark), expanded ~250-line §6 skeleton with metrics module + `Breaker` struct + full state machine + `emit_transition` helper, code/concept/operational deps in §7, 5 example payloads in §8, 16 failure modes in §10, 9 implementation notes in §11.

Four residual issues prevent 10/10.

## §2 — Findings

### ISS-001 — ACs #1/#2/#4/#5 promise metric assertions but §5 doesn't test them
- **severity:** error
- **rule_id:** test-coverage
- **location:** §4 ACs #1, #2, #4, #5; §5 (verification)
- **status:** open

#### Description
Multiple ACs include metric-emission assertions:
- AC #1: *"MUST emit `ai_breaker_transitions_total{from='closed',to='open'}` once."*
- AC #2: *"Each blocked call MUST increment `ai_breaker_short_circuits_total{provider,model}` once."*
- AC #4: *"MUST emit `ai_breaker_probes_total{outcome='succeeded'}` once."*
- AC #5: *"MUST emit `ai_breaker_probes_total{outcome='failed'}` once."*

But §5's tests only assert state transitions and counter resets — not Prometheus metric increments. `opens_after_5_failures` checks `is_open` returns true; it doesn't verify the `TRANSITIONS` counter ticked. A code-gen agent reading this would skip the metric assertions because §5 has no template for them.

Same shape as FR-AI-007 ISS-001 (proptest body promised but not shown) and FR-AI-008 ISS-002 (placeholder test stubs).

#### Suggested fix
Add a metric-assertion helper to `tests/circuit_breaker_test.rs` and use it in the relevant tests:

```rust
fn metric_value(name: &str, labels: &[(&str, &str)]) -> f64 {
    use prometheus::core::Collector;
    let metric_families = prometheus::default_registry().gather();
    for mf in metric_families {
        if mf.get_name() != name { continue; }
        for m in mf.get_metric() {
            let actual_labels: Vec<_> = m.get_label().iter()
                .map(|p| (p.get_name(), p.get_value())).collect();
            if labels.iter().all(|(k, v)| actual_labels.iter().any(|(ak, av)| ak == k && av == v)) {
                return m.get_counter().get_value();
            }
        }
    }
    0.0
}

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
```

Add equivalent tests for short-circuits (AC #2), probe-success (AC #4), and probe-failure (AC #5).

### ISS-002 — `Closed → Open` transition is not CAS-guarded; concurrent failures double-count
- **severity:** error
- **rule_id:** correctness / concurrency
- **location:** §1 #1 (CAS rule), §6 skeleton (`record_outcome` open-on-threshold branch)
- **status:** open

#### Description
§1 #1 says: *"direct `if state == 1` comparisons are forbidden"* and §1 #3 specifies the `Open → HalfOpen` transition uses CAS. By symmetry the `Closed → Open` transition SHOULD also be CAS-protected, but §6's skeleton uses a plain read-then-store pattern:

```rust
let count = b.failure_count.fetch_add(1, Ordering::AcqRel) + 1;
if count >= FAILURE_THRESHOLD && prior_state == BreakerState::Closed {
    b.state.store(BreakerState::Open.as_u8(), Ordering::Release);
    b.open_until_nanos.store(now + OPEN_DURATION_NANOS, Ordering::Release);
    b.last_state_change.store(now, Ordering::Release);
    emit_transition(provider, model, BreakerState::Closed, BreakerState::Open);
}
```

If two concurrent callers both see `prior_state == Closed` AND both see `count >= 5` (because `fetch_add` makes them both observe ≥5 monotonically), both execute the `state.store(Open)` and BOTH call `emit_transition`. Result: `ai_breaker_transitions_total{from=closed,to=open}` increments by 2 instead of 1, even though logically only one transition happened. AC #1 ("MUST emit … once") then fails under concurrency.

#### Suggested fix
Use CAS to guard the transition emission, so only the actual transition winner emits:

```rust
let count = b.failure_count.fetch_add(1, Ordering::AcqRel) + 1;
if count >= FAILURE_THRESHOLD {
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
    // CAS losers: state was already Open or HalfOpen — no-op.
}
```

Drop the `prior_state == BreakerState::Closed` check above the threshold branch (the CAS subsumes it). Add §10 row: *"Concurrent failures racing past threshold → exactly one CAS winner emits the transition; AC #1's 'once' assertion holds under concurrency."*

### ISS-003 — Per-call `model.to_string()` allocation contradicts §1 #11's <100ns claim
- **severity:** warning
- **rule_id:** correctness / performance
- **location:** §1 #11, §6 skeleton (`is_open`, `record_outcome`, `reset`)
- **status:** open

#### Description
§1 #11 says: *"MUST complete `is_open` check in <100ns (single atomic read on the hot path)."*

But §6's `is_open` (and every other API) builds the lookup key with:

```rust
let key = (*provider, model.to_string());
```

The `model.to_string()` call allocates a heap `String` per call — typical heap-allocation latency on macOS is 50–200ns, immediately busting the 100ns budget. The benchmark bench_is_open_closed_state would surface this if measured carefully, but the AC text ("1M calls in <100ms total") allows up to 100ns per call which is exactly the borderline.

#### Suggested fix
Two-part fix:

1. **Borrow-key lookup** — use DashMap's `Borrow`-based `get` with a key type that doesn't require allocation. Define:

```rust
#[derive(Hash, PartialEq, Eq, Clone)]
struct BreakerKey(ProviderKind, String);

#[derive(Hash, PartialEq, Eq)]
struct BreakerKeyRef<'a>(ProviderKind, &'a str);

impl<'a> std::borrow::Borrow<BreakerKeyRef<'a>> for BreakerKey { /* … */ }
```

Then `is_open` calls `map.get(&BreakerKeyRef(*provider, model))` — no allocation.

2. **Update §1 #11 if the allocation is unavoidable** — if the borrow-key trick has implementation friction (DashMap's Borrow API is fiddly), tighten §1 #11 to: *"MUST complete `is_open` check in <500ns p99"* and document the per-call allocation in §10. Aim for the borrow-key version since the math says it can hit <100ns.

Add §10 row: *"Per-call `String` allocation in lookup → exceeds 100ns budget; mitigated by `Borrow`-based key type."*

### ISS-004 — `init()` silently no-ops on second call; breaks test isolation
- **severity:** warning
- **rule_id:** robustness
- **location:** §6 skeleton (`init` function), §10 (currently says "Idempotent by design")
- **status:** open

#### Description
`init()` uses `OnceCell::set(...).ok()` which silently swallows the `Err` returned on a second `set` attempt:

```rust
pub fn init(clock: Box<dyn Clock>) {
    BREAKERS.set(DashMap::new()).ok();
    CLOCK.set(clock).ok();
}
```

Consequences:
- A test that calls `setup_with_mock_clock()` twice (e.g., a parameterized test, or a test that re-initializes for isolation) gets the FIRST clock back, not the second. The `MockClock::advance` calls in the second test target a clock the breaker never uses.
- The DashMap is not cleared between tests, so state from prior tests leaks across.

§10 currently says *"Idempotent by design"* but that's misleading — the function isn't idempotent; it's first-write-wins-silently.

#### Suggested fix
Two options:

(a) **Make `init` panic on double-call** — surface the programmer error:
```rust
pub fn init(clock: Box<dyn Clock>) {
    BREAKERS.set(DashMap::new()).expect("circuit_breaker::init called twice");
    CLOCK.set(clock).expect("circuit_breaker::init called twice");
}
```

(b) **Add `reset_for_tests()` behind a `#[cfg(test)]` guard** that clears both cells AND the DashMap. Tests can call this between cases:
```rust
#[cfg(any(test, feature = "test-mock-clock"))]
pub fn reset_for_tests() {
    if let Some(map) = BREAKERS.get() { map.clear(); }
    // CLOCK is global; tests should re-init with a fresh MockClock per test file, not per test.
}
```

Recommended: ship both. Update §10's "Idempotent by design" row to: *"`init` panics on double-call to surface programmer error; `reset_for_tests()` is the correct way to clear state between cases."*

### ISS-005 — AUTHORING.md §3.8 rule 26 (pair-write history events) — breaker_opened present but breaker_closed not asserted
- **severity:** warning
- **rule_id:** authoring-md-§3.8 (rule 26)
- **location:** §1 (transition emit clauses), §6 (`emit_transition` callers)
- **status:** open

#### Description
AUTHORING.md §3.8 rule 26 says "Pair-write history events (e.g. `*_started` + `*_completed`) — operators tracing crashes need both bookends. Started without Completed = crash signal." The breaker has 4 transition events: Closed→Open (`breaker_opened`), Open→HalfOpen (`probe_started`), HalfOpen→Closed (`probe_succeeded` / `breaker_closed`), HalfOpen→Open (`probe_failed`). The spec emits all four via `emit_transition` but `probe_started` and `probe_succeeded` are not explicitly named as a pair in §1 — an operator tracing "breaker stuck open after Retry-After cooldown" can't tell whether a `probe_started` was followed by a `probe_succeeded` or `probe_failed` without scanning prior rows. The §1 normative text should explicitly name the pairs and assert that for every `probe_started` audit row, a `probe_succeeded` OR `probe_failed` row MUST follow within the probe timeout window (e.g., 30s).

#### Suggested fix
Add §1 #16: "**MUST** emit transitions as audit-row pairs per AUTHORING.md §3.8 rule 26: for every `probe_started` row, a `probe_succeeded` OR `probe_failed` row MUST appear within the probe-call deadline (default 30s, capped by the underlying provider call timeout). The pairing is enforced by an OBS lint: a Grafana alert fires if `count(probe_started) - count(probe_succeeded) - count(probe_failed) > 0` for any (provider, model) over a 5-minute window. A standalone `probe_started` is a crash signal."

### ISS-006 — AUTHORING.md §3.9 rule 27 (determinism) not asserted for breaker state transitions
- **severity:** warning
- **rule_id:** authoring-md-§3.9 (rule 27)
- **location:** §1 (state machine), §4 ACs, §5 verification
- **status:** open

#### Description
AUTHORING.md §3.9 rule 27 says outputs MUST be deterministic on the same input. The breaker's state transitions depend on (provider, model, sequence of outcomes, MockClock). Two test runs feeding the same outcome sequence to the same `MockClock` MUST produce byte-identical sequences of `emit_transition` audit rows. The DashMap iteration order is non-deterministic across runs, which COULD affect the order of audit-row emission if a single `record_outcome` call somehow triggered multiple transitions across multiple breakers (it doesn't today, but a future "global breaker-reset on policy reload" feature would). The spec has no §4 AC asserting determinism — relying on the MockClock alone is insufficient.

#### Suggested fix
Add §4 AC #16: "**Transition sequence deterministic across runs** — Two test runs feeding `[Failure, Failure, Failure, Failure, Failure, Success]` to the same (Bedrock, model-x) breaker with the same MockClock MUST produce byte-identical sequences of `MemoryRow` emissions (sorted by `ts_ns` then `extra.provider` then `extra.model`)." Add §5 test `test_transitions_deterministic_across_runs` that runs the sequence twice and asserts `assert_eq!` on the captured `MemoryRow` Vec.

## §3 — Strengths preserved through expansion

- §1 grew from 10 to 15 numbered MUST clauses, each individually testable in §4. The closed-set rationale (§1 #1's "direct `if state == 1` comparisons forbidden", §1 #14's 429-as-failure-class) anticipates the common refactor-error class.
- §3's `BreakerState::as_metric_label()` follows the FR-AI-007 ISS-003 pattern; OBS labels are rename-safe.
- §3's `Clock` trait + `MockClock` matches the testability bar set by FR-AI-005's policy-loader tests; explicit time-source abstraction is a clean way to avoid the `tokio::time::pause`-doesn't-help-with-`std::time::Instant` pitfall.
- §6's `emit_transition` helper centralizes the `STATE` gauge update — setting the new state to 1 and the others to 0 in one place, so a refactor changing the gauge semantics needs to touch exactly one function.
- §10 inventory covers 16 distinct failure paths including the `BreakerState::from_u8(99)` "atomic-state corruption" panic and the `init` race row — defence in depth surfaced as documentation.
- §11 notes flag the slice-5 cardinality concern explicitly: per-tenant breakers will multiply key-space and OBS labels; `tenant_id × provider × model × state` is a Prometheus-cardinality problem that needs an aggregation strategy.

## §4 — Resolution

All 6 mechanical revisions applied:
- ISS-001 RESOLVED (2026-05-16): §5 now includes the `metric_value` helper plus 4 metric-assertion tests; ACs #1/#2/#4/#5 now have direct test coverage.
- ISS-002 RESOLVED (2026-05-16): §6 `record_outcome` open-on-threshold branch rewritten to use `compare_exchange(Closed, Open)`; the CAS winner emits `emit_transition` exactly once; CAS losers are no-ops. §10 row added documenting the concurrent-failures-past-threshold case.
- ISS-003 RESOLVED (2026-05-16): §6 introduces `BreakerKey` and `BreakerKeyRef<'a>` types with `Borrow` impl; `is_open`/`record_outcome`/`reset` look up via `BreakerKeyRef(*provider, model)` — no per-call allocation. §10 row added.
- ISS-004 RESOLVED (2026-05-16): §6 `init` now uses `.expect("circuit_breaker::init called twice")` instead of `.ok()`; `reset_for_tests()` function added behind `#[cfg(any(test, feature = "test-mock-clock"))]`. §10 "Idempotent by design" row replaced with the panic-on-double-init row.
- ISS-005 RESOLVED (2026-05-16, AUTHORING.md compliance pass): §1 #16 added naming the probe-pair contract (`probe_started` + (`probe_succeeded` OR `probe_failed`) within probe deadline); OBS lint specified.
- ISS-006 RESOLVED (2026-05-16, AUTHORING.md compliance pass): §4 AC #16 added asserting transition-sequence determinism; §5 test `test_transitions_deterministic_across_runs` body added.

**Score = 10/10.** Ship as-is. Ready to transition `draft → accepted`.

---

*End of FR-AI-009 audit (final). Status: PASS at 10/10.*
