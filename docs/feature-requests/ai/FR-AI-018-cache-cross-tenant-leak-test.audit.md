---
fr_id: FR-AI-018
audited: 2026-05-16
auditor: manual (engineering-spec template)
verdict: PASS (after revision)
score_pre_revision: 7.0/10        # the first-pass compressed version (180 lines)
score_post_expansion: 9.0/10      # after expanding to FR-AI-014 / FR-AI-017 depth (~990 lines)
score_post_revision: 10/10        # after 6 mechanical fixes
issues_open: 0
issues_resolved: 6
issues_critical: 0
template: engineering-spec@1
revised_at: 2026-05-16
---

## §1 — Verdict summary

FR-AI-018 was expanded from 180 lines to ~990 lines matching FR-AI-014 / FR-AI-017 depth.

The expansion added 7 §1 normative clauses (#4 dual-layer testing at key-derivation level, #5 Redis-namespace isolation test, #7 adversarial-input corpus, #8 concurrent-load test, #11 per-case Redis isolation prefix, #13 non-skip CI enforcement, #14 JSON artefact emission), 7 substantive §2 rationale paragraphs (property-test-vs-enumerated argument, 200K-ops calibration math, dual-layer testing for triage-speed, Redis-namespace structural enforcement frame, regression-scenarios-as-post-mortem-docs, adversarial defence-in-depth, concurrent-vs-sequential discovery-surface argument, per-case isolation flaky-failure prevention, non-skip paranoid-gate principle, JSON-artefact regulatory primitive), full Rust test-helpers in §3 (RedisTestNamespace with Drop cleanup, proptest_strategies module with adversarial corpus, full GitHub Actions workflow with health-check + skip-detection + artefact upload), expanded §4 from 7 to 16 acceptance criteria, full Rust test bodies in §5 (3 property tests + 7 regression scenarios + 3 adversarial tests + concurrent test), §6 references §3 and §5, expanded §7 with code/concept/operational dep split, 4 example payloads in §8 (CI pass output, CI failure output with shrunk minimal case, JSON artefact schema, skip-detection failure), 17 failure modes in §10 (vs. 4 in first pass), 11 implementation notes in §11.

Six residual issues prevented 10/10 at the post-expansion checkpoint; all six are mechanical and all six are resolved in this revision.

## §2 — Findings

### ISS-001 — 200,000-ops claim contradicts proptest's 256-default case-budget

- **severity:** error
- **rule_id:** spec-vs-mechanism mismatch
- **location:** §1 #2 (claim), §5 (proptest body without explicit ProptestConfig)
- **status:** resolved

#### Description

First-pass §1 #2 said: *"MUST generate at least 200 random tenant pairs × 1000 cache operations per pair = 200,000 randomised operations per test run."* But proptest's default is 256 cases, and §5's `proptest!` block didn't specify `ProptestConfig::with_cases(...)`. Default would produce ~256 × ~500 = 128K ops — below the declared 200K floor.

The math also conflates two things: cases (proptest's iteration count) and ops-per-case (the fan-out within each case). The first-pass said "200 pairs × 1000 ops" — implying 200 cases — but proptest's cases are NOT the same as tenant pairs (each case generates its own pair AND its own op sequence).

#### Suggested fix

1. Make the math explicit: `ProptestConfig::with_cases(256)` × `prop::collection::vec(any_cache_op(), 800..1200)` → ~256K ops, comfortably above 200K floor.
2. Add the explicit `ProptestConfig` to every proptest body in §5.
3. Document the calibration in §2: 200K catches 1-in-10K bugs at ≥99.9%; 256 cases × ~1000 ops = 256K satisfies the floor.
4. Add CI step: `PROPTEST_CASES: "256"` env var to make the case count explicit and CI-visible.

### ISS-002 — `block_on(...)` inside proptest body creates a new tokio runtime per case

- **severity:** error
- **rule_id:** correctness / test fragility
- **location:** §5 stub body
- **status:** resolved

#### Description

First-pass §5 had:

```rust
for op in &operations {
    block_on(cache::insert(&key_for(tenant_a, op), &test_response(), Duration::from_secs(3600)));
}
```

`block_on` is presumed to mean `tokio::runtime::Runtime::new().unwrap().block_on(...)`. Creating a fresh tokio runtime per call (or per case) is:
- Slow (each runtime initialisation is ~10ms; 256K calls × 10ms = unworkable).
- Fragile (Redis connection pools across runtimes don't interoperate).
- Non-deterministic in CI under load (Redis connect timeouts vary).

The pattern is wrong; the test would either time out or be flaky.

#### Suggested fix

1. Construct ONE `tokio::runtime::Runtime` per case at the top, then `rt.block_on(async { ... })` for the entire case body.
2. Show the pattern explicitly in §5.
3. Document the design choice in §11.
4. Concurrent test uses `#[tokio::test(flavor = "multi_thread", worker_threads = 8)]` (separate concern; not block_on).

### ISS-003 — 5 regression scenarios mentioned in sub_tasks but not enumerated

- **severity:** error
- **rule_id:** spec-completeness / promise-vs-implementation
- **location:** sub_tasks (claim "5 hardcoded scenarios"), §1/§5/§6 (no scenario list)
- **status:** resolved

#### Description

First-pass had `sub_tasks: "0.5h: Regression-test data (5 hardcoded scenarios known to have leaked before)"` and §5 showed only ONE example (`regression_scenario_001_underscore_collision`). The other four were left as `// ... 4 more regression scenarios`.

A code-gen agent has no template for what the other four are. The seven-scenario list (after revision) was actually drawn from real cache-bug post-mortems; without enumeration, the agent fabricates scenarios that may overlap with the existing property test (no value) or miss real failure modes.

#### Suggested fix

1. Enumerate 7 scenarios (one more than the original "5" — added unicode-normalization and persona-omitted; both are FR-AI-017-related).
2. Each scenario gets:
   - A normative clause in §1 #6 (named with the scenario id).
   - A full Rust test body in §5 with comment block citing incident date + root cause.
3. Make the regression file (`cache_isolation_regression_scenarios.rs`) a separate test file in `new_files`.
4. Add §10 row: "Regression scenario removed/renamed → CI fails on missing-test detection" — preserves the corpus over time.

### ISS-004 — No adversarial input testing (null bytes, RTL unicode, very long strings, empty strings)

- **severity:** error
- **rule_id:** test-coverage / boundary inputs
- **location:** §1 (no clause), §5 (no adversarial test)
- **status:** resolved

#### Description

The first-pass property test used `any_tenant_pair()` strategy without specifying its alphabet. proptest's default for `String` strategies might produce reasonable strings but won't deliberately exercise pathological inputs (null bytes, control characters, RTL Unicode, zero-width joiners).

Multi-tenancy boundaries are attack surfaces. A malicious tenant could choose a tenant_id designed to exploit specific cache behaviours — these need explicit coverage. Random alphanumeric strings won't generate `"\x00"` or `"\x1f"` (the unit-separator we use internally as the field delimiter).

#### Suggested fix

1. Add §1 #7 normative requirement: adversarial-inputs test covering 10 enumerated pathological strings.
2. Add `proptest_strategies::adversarial_tenant_strings()` in §3 returning the corpus.
3. Add `cache_isolation_adversarial_test.rs` to `new_files` and §5.
4. Document the corpus-as-collective-hardening principle in §11 (add new vectors discovered in production; never remove).
5. Add §10 row: "New attack vector discovered in production → add to adversarial corpus."

### ISS-005 — No race-condition test; sequential tests miss non-atomic insert-path bugs

- **severity:** error
- **rule_id:** test-coverage / concurrency
- **location:** §1 (no clause), §5 (no concurrent test)
- **status:** resolved

#### Description

The first-pass tests are all sequential (insert, then lookup, both as single-threaded tokio tasks). A non-atomic insert path — for example, "write-then-read" without transactional semantics — could briefly expose the wrong tenant's data during the gap. Sequential tests can't catch this; they don't race.

The first-pass §10 had no row for race conditions. The cache-correctness floor depends on atomicity at the insert path; testing that requires actually racing.

#### Suggested fix

1. Add §1 #8 normative requirement: concurrent test with 100 tasks × 50 tenant pairs × 10K ops.
2. Use `#[tokio::test(flavor = "multi_thread", worker_threads = 8)]` to genuinely exercise concurrency.
3. Add `cache_isolation_concurrent_test.rs` to `new_files` and §5.
4. Add AC #8 asserting the concurrent test passes.
5. Add §10 row: "Concurrent test detects race → investigate cache layer for non-atomic write paths."

### ISS-006 — Per-case Redis cleanup not specified; tests will be flaky from inter-case pollution

- **severity:** warning
- **rule_id:** test isolation / robustness
- **location:** §1 (no clause), §5 (no cleanup)
- **status:** resolved

#### Description

The first-pass property test inserts under random tenant_ids generated by proptest. Two cases could (with low probability) generate the same tenant_id; the second case's lookup would see the first case's residue. The result is intermittent failures that don't reproduce on rerun (because RNG seeds vary) — exactly the flaky-test pattern that erodes developer trust.

The §5 stub had no `setUp`/`tearDown`. The §10 row "Flaky test (Redis state pollution) → Add per-test cleanup" acknowledges the problem but punts on the fix.

#### Suggested fix

1. Add §1 #11 normative requirement: per-case prefix `test_<uuid>_` with cleanup in Drop.
2. Add `RedisTestNamespace` in §3 with `Drop` impl that deletes keys matching its prefix.
3. Use `RedisTestNamespace::new()` at the top of every test body; wrap tenant_ids with `ns.tenant(&original)`.
4. Add AC #11 asserting per-case isolation prevents flakiness (re-runs with same RNG seed produce same outcome).
5. Add §10 row: "Per-case Redis namespace leaked (Drop didn't run) → sev-2 CI alarm."

## §3 — Strengths preserved through expansion

- §1 #4 + §1 #5 introduce dual-layer testing: derivation-level AND end-to-end. A regression caught at one layer is also visible at the other; the failure pattern tells you which layer regressed (faster triage).
- §1 #6 enumerates 7 named regression scenarios. Each scenario IS the post-mortem documentation for a real bug; future engineers see immediately diagnostic test names rather than generic "property test failed at minimal case (..., ..., ...)".
- §1 #7 introduces an adversarial corpus that grows monotonically — new attack vectors discovered in production are appended; never removed. Collective hardening artefact across the codebase lifetime.
- §1 #11 + the `RedisTestNamespace` helper makes per-case isolation a structural property, not a discipline. Drop cleanup runs even on test panic; intermittent failures from state pollution become impossible.
- §1 #13 + the workflow's skip-count parser enforce non-skippability at the CI gate. The most common compliance-test-erosion pattern (silent `#[ignore]`) is caught at merge time.
- §1 #14 introduces the JSON artefact as the durable audit primitive. MoPS A05's "prove no cross-tenant exposure" question gets a concrete file as the answer, durable beyond GitHub Actions log retention.
- §10 inventory grew from 4 rows to 17 — including the test-skip-slip path, the regression-removed path, the concurrent-deadlock path, the namespace-leaked path, and the proptest-shrinker-bug path. Each row has an unambiguous detection mechanism.
- §11 explicitly documents the design choices (`--release` flag requirement, multi-thread runtime requirement, non-skip paranoid principle) so future engineers don't accidentally weaken the gate.

## §4 — Resolution

All 6 mechanical revisions applied (2026-05-16) within the FR itself:

- **ISS-001 RESOLVED**: §1 #2 makes the math explicit (`ProptestConfig::with_cases(256)` × ~1000 ops/case = 256K ops); §5 every proptest body specifies `ProptestConfig`; CI workflow exposes `PROPTEST_CASES: "256"` env var; §2 has the calibration paragraph.

- **ISS-002 RESOLVED**: §5 uses ONE `tokio::runtime::Runtime::new().unwrap()` per proptest case wrapping the entire case body in `rt.block_on(async { ... })`; pattern shown explicitly; concurrent test uses `#[tokio::test(flavor = "multi_thread")]` (separate path).

- **ISS-003 RESOLVED**: 7 regression scenarios enumerated in §1 #6 (added two beyond the original 5: unicode-normalisation and persona-omitted); §5 has full Rust bodies for all 7 with comment-block post-mortem citations; `cache_isolation_regression_scenarios.rs` in `new_files`; §10 row for "Regression scenario removed/renamed → CI fails on missing-test detection."

- **ISS-004 RESOLVED**: §1 #7 adds adversarial-inputs requirement; `proptest_strategies::adversarial_tenant_strings()` in §3 returns 10-string corpus; `cache_isolation_adversarial_test.rs` in `new_files` and §5; §11 note on corpus-as-collective-hardening; §10 row for new-attack-vector discovery.

- **ISS-005 RESOLVED**: §1 #8 adds concurrent-test requirement; `#[tokio::test(flavor = "multi_thread", worker_threads = 8)]` in §5; 100 tasks × 50 pairs × 100 ops = 10K concurrent ops; AC #8 + §10 row for race-detection failure path.

- **ISS-006 RESOLVED**: §1 #11 adds per-case isolation requirement; `RedisTestNamespace` with `Drop` cleanup in §3; every test body in §5 starts with `let ns = RedisTestNamespace::new()`; AC #11 asserts non-flakiness; §10 row for namespace-leaked-from-Drop-failure path.

**Score = 10/10.** Ship as-is. Ready to transition `draft → accepted`.

---

*End of FR-AI-018 audit (final). Status: PASS at 10/10.*
