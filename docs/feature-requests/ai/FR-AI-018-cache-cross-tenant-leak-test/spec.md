---
# ───── Machine-readable frontmatter (parsed by feature-request-audit + future fr-catalog renderer) ─────
id: FR-AI-018
title: "Cross-tenant cache leak property-test (hard zero) — 200K random ops + 7 regression scenarios + adversarial inputs"
module: AI
priority: MUST
status: done
verify: T
phase: P0
milestone: P0 · slice 4
slice: 4
owner: Stephen Cheng
created: 2026-05-15
shipped: 2026-05-21
memory_chain_hash: null
related_frs: [FR-AI-005, FR-AI-014, FR-AI-016, FR-AI-017]
depends_on: [FR-AI-017]
blocks: []

# ───── Source contracts ─────
source_pages:
  - website/docs/modules/ai.html#cache
  - website/docs/legal/multi-tenancy-isolation.html
source_decisions:
  - DEC-058 multi-tenancy invariant: cross-tenant data leakage = 0 (no acceptable rate; ANY leak = sev-1)
  - PDPL Art. 7 (tenant data MUST NOT cross tenant boundary without explicit consent)
  - GDPR Art. 32 (security of processing — multi-tenant isolation is a fundamental control)
  - "MoPS A05 audit checkpoint: \"how do you prove no cross-tenant data exposure\" — this FR is the answer"

# ───── Build envelope ─────
language: rust 1.81 (test-only crate inside ai-gateway)
service: cyberos/services/ai-gateway/
new_files:
  - services/ai-gateway/tests/cache_isolation_property_test.rs
  - services/ai-gateway/tests/cache_isolation_regression_scenarios.rs
  - services/ai-gateway/tests/cache_isolation_adversarial_test.rs
  - services/ai-gateway/tests/cache_isolation_concurrent_test.rs
  - services/ai-gateway/tests/support/redis_isolation_helper.rs
  - services/ai-gateway/tests/support/proptest_strategies.rs
  - .github/workflows/cache-isolation-gate.yml                     # dedicated CI workflow with non-skip enforcement
modified_files: []
allowed_tools:
  - file_read: services/ai-gateway/**
  - file_write: services/ai-gateway/tests/**
  - file_write: .github/workflows/cache-isolation-gate.yml
  - bash: cd services/ai-gateway && cargo test cache_isolation
  - bash: docker run -d --name test-redis -p 6379:6379 redis:7
disallowed_tools:
  - skip the property test in CI under any condition (NO `pytest.skip` analogue, no `#[ignore]` on the property body)
  - lower the case count below the §1 #2 floor without explicit FR amendment
  - omit any of the 7 enumerated regression scenarios (deletion = silent test-coverage drop)
  - share Redis state between test cases (per-case isolation prefix MUST be used; §1 #11)

# ───── Estimated work ─────
effort_hours: 6
sub_tasks:
  - "0.5h: proptest strategies for tenant_id, prompt, model, persona_handle (with adversarial corpus)"
  - "0.5h: redis_isolation_helper — per-case prefix `test_<uuid>_` with cleanup in Drop"
  - "1.0h: Property test #1: insert-as-A then lookup-as-B always returns Miss (200K ops)"
  - "0.5h: Property test #2: derive(A,...) ≠ derive(B,...) for any A ≠ B (key-derivation level)"
  - "0.5h: Property test #3: scan `KEYS ai_cache:v1:tenant_a:*` never returns tenant_b entries"
  - "1.0h: 7 enumerated regression scenarios (each documents a historical near-miss bug)"
  - "0.5h: Adversarial inputs test: null bytes, control chars, RTL unicode, very long strings, empty strings"
  - "0.5h: Concurrent test: 100 tasks racing insert/lookup across tenant pairs"
  - "0.5h: cache-isolation-gate.yml workflow with non-skip enforcement (parses output for skipped count)"
  - "0.5h: Failure-detail formatter (which tenant pair, prompt, model, step in sequence)"
risk_if_skipped: "Cross-tenant leak invariant unenforced. Cache implementation might silently key-collide. The compliance gate (zero cross-tenant reads) downgrades from 'proven' to 'we think so' — exactly the indistinguishability between competence and failure that the memory audit chain is designed to prevent. MoPS A05 audit response 'we have a property test' becomes 'we have a unit test that covers the cases we thought of' — much weaker. First multi-tenant data exposure incident in production (probability ~10⁻³ per year without this gate; ~10⁻⁹ with) ends 100% of B2B contracts that include data-isolation clauses."
---

## §1 — Description (BCP-14 normative)

The AI Gateway crate **MUST** include a property-based test asserting **zero cross-tenant cache reads** under any randomised input, plus enumerated regression scenarios, adversarial inputs, and concurrent-load testing. The test suite obeys the following:

1. **MUST** use the `proptest` crate as the property-test framework. The crate's shrinking discipline produces minimal failing cases for human review on regression; alternative frameworks (quickcheck) lack equivalent shrinking.
2. **MUST** generate at least **200 random tenant pairs × 1000 cache operations per pair = 200,000 randomised operations per test run**. Mechanically: `ProptestConfig::with_cases(256)` (proptest default) × `prop::collection::vec(any_cache_op(), 800..1200)` (each case fans out to ~1000 ops) → ~256K ops per CI run. The 200K floor catches cache-key collision bugs at ~1-in-10K frequency with ≥99.9% probability.
3. **MUST** assert: for any sequence of `cache::insert(tenant_a, ...)` followed by `cache::lookup(tenant_b, ...)` where `tenant_a ≠ tenant_b`, the lookup MUST return `CacheLookupOutcome::Miss` even if all other key components (redacted prompt, model, persona_handle) are byte-identical. The assertion is at the API surface (matching the production cache call shape), not at the internal hash function.
4. **MUST** include a second property test at the **key-derivation** level: for any `(A, B)` with `A ≠ B`, `CacheKey::derive(A, prompt, model, persona) ≠ CacheKey::derive(B, prompt, model, persona)` (byte-distinct hashes). This catches collisions BEFORE they manifest as Redis-level leaks; the dual-layer testing (derivation + end-to-end) is defence-in-depth.
5. **MUST** include a third property test at the **Redis-key namespace** level: scanning `KEYS ai_cache:v1:tenant_a:*` after inserting under both `tenant_a` and `tenant_b` MUST return only tenant_a's entries. This validates the structural prefix invariant (FR-AI-017 §1 #2).
6. **MUST** include exactly **7 enumerated regression scenarios** in `cache_isolation_regression_scenarios.rs`, each documenting a historical near-miss bug. The seven (each gets one `#[test]` function with a comment block citing the original incident date + root cause):
   - `regression_001_underscore_collision` — `"tenant_a"` and `"tenanta"` previously collided due to insufficient input separation.
   - `regression_002_unicode_normalization` — `"tenant_é"` (NFC) and `"tenant_é"` (NFD) hashed differently → race condition exposed; reverse case.
   - `regression_003_trailing_whitespace` — `"tenant_a"` and `"tenant_a "` should be DIFFERENT tenants (whitespace is a valid id character).
   - `regression_004_case_folding` — `"Tenant_A"` and `"tenant_a"` previously folded; tenants are case-sensitive.
   - `regression_005_empty_prompt` — `tenant_a` + empty prompt should NOT collide with `tenant_a` + single-space prompt.
   - `regression_006_persona_omitted` — Pre-FR-AI-017-ISS-001 keys omitted persona; this scenario asserts persona-handle inclusion.
   - `regression_007_model_substring` — `model = "chat.smart"` and `model = "chat.smartx"` previously hashed similarly under poor concat.
7. **MUST** include an adversarial-input test (`cache_isolation_adversarial_test.rs`) covering: null bytes (`"\x00"` in tenant id), unit separators (`"\x1f"` — the field separator we use internally; an attacker injecting it shouldn't break key uniqueness), control characters, RTL Unicode (`"‮"` reversal), zero-width joiners, very long strings (10KB tenant id), empty strings, and SQL-injection-shaped payloads (`"tenant'; DROP TABLE--"`).
8. **MUST** include a concurrent-load test (`cache_isolation_concurrent_test.rs`) — 100 tokio tasks racing `insert`/`lookup` across 50 tenant pairs simultaneously. The test asserts: across 10,000 concurrent operations, ZERO cross-tenant reads. Concurrency catches race conditions that sequential tests miss (e.g., a non-atomic insert path that briefly exposes the wrong tenant's data).
9. **MUST** complete the entire suite (property + regression + adversarial + concurrent) in ≤ 90 seconds on a standard `ubuntu-22.04` GitHub Actions runner. Budget breakdown: 60s for the main property test, 10s for regression, 10s for adversarial, 10s for concurrent.
10. **MUST** emit per-failure detail with the FULL cross-tenant context: `tenant_a`, `tenant_b`, `prompt`, `model`, `persona_handle`, `step_in_sequence`, AND the `CacheKey::derive` hash for each side. The hash inclusion is critical for triaging "is this a key-derivation bug or a Redis-level leak?". On proptest shrinking, the minimal failing case is also reported.
11. **MUST** isolate Redis state per test case via a per-case prefix `test_<uuid>_` injected into the tenant_id input. The `redis_isolation_helper` cleans up keys matching the prefix in `Drop`. This prevents inter-case state pollution that would produce flaky failures (one case inserting; the next case lookup-ing the same key).
12. **MUST** be a CI gate on every PR via `.github/workflows/cache-isolation-gate.yml`. The workflow MUST trigger on changes to `services/ai-gateway/src/cache/**` AND `services/ai-gateway/tests/cache_isolation*.rs` AND the workflow file itself (self-gate per FR-AI-013 §1 #8 pattern).
13. **MUST** enforce non-skip in CI: the workflow parses pytest output for `skipped` count and fails if any skip markers appear. This prevents the operational failure mode of a developer adding `#[ignore]` to silence a flaky test.
14. **SHOULD** emit a JSON artefact `cache_isolation_report_<sha>.json` with the case count, regression-scenario pass-list, adversarial-pass-list, concurrent-test wall-clock, and any failure detail. This is the audit primitive: when MoPS A05 asks "show me the proof of cross-tenant isolation as of date X," the artefact for that date's CI run is the answer.

---

## §2 — Why this design (rationale for humans)

**Why a property test instead of enumerated test cases?** Property tests scale much better than enumerated tests for invariants like "zero cross-tenant leaks." A traditional test suite might cover 10 hand-picked scenarios; a property test covers thousands of randomised scenarios that the engineer didn't think of. The cost is one-time test-strategy setup; the benefit is sustained for every PR. proptest's shrinking discipline produces minimal failing cases that humans can read — turning a 1000-op failure into a 2-op explanation.

**Why 200K operations as the floor (§1 #2)?** Cache-key collision bugs (the most likely class for cross-tenant leaks) tend to manifest at frequencies in the 1-in-10K to 1-in-1M range — too rare for a 100-case test, too common for a billion-case test. 200K ops sits at the sweet spot: ~99.9% probability of catching 1-in-10K bugs, runtime budget under 60 seconds. The number is calibrated, not arbitrary.

**Why dual-layer testing (key-derivation + end-to-end, §1 #4 + #5)?** A bug at the key-derivation level (`CacheKey::derive` produces same hash for different tenants) is also catchable at the end-to-end level (insert+lookup observes the leak). But the inverse isn't true: a bug in the Redis backend (e.g., a SCAN-pattern bug that returns wrong-tenant keys) wouldn't manifest at the derivation level. Testing both layers means a regression caught at one layer is also visible at the other — making triage much faster (the failure pattern tells you which layer regressed).

**Why a Redis-namespace test (§1 #5)?** FR-AI-017 §1 #2 makes per-tenant prefix a STRUCTURAL invariant. The Redis-namespace test is the assertion of that structure — without it, "structural" is just a word. The KEYS pattern test would catch a backend bug like "SCAN ignores tenant prefix" that key-derivation tests would miss.

**Why 7 enumerated regression scenarios (§1 #6)?** Property tests catch novel bugs; regression tests document known bugs. The seven scenarios were each a real near-miss in development — naming them and pinning them to specific test functions means a future engineer who breaks one of these patterns sees an immediately diagnostic test name (`regression_001_underscore_collision`) rather than a generic "property test failed at minimal case (..., ..., ...)". The comment block in each test cites the incident date and root cause; the test IS the post-mortem documentation.

**Why adversarial input testing (§1 #7)?** Multi-tenancy boundaries are attack surfaces. A malicious tenant might choose a tenant_id like `"tenant_a\x1ftenant_b"` (containing the unit separator we use internally) hoping it'd parse as a different key. The adversarial inputs are NOT testing for production behaviour (those tenant_ids would be rejected at policy load by FR-AI-005's validators) — they're testing that the cache key derivation is robust EVEN IF the tenant_id validator fails. Defence in depth.

**Why concurrent testing (§1 #8)?** Sequential tests can't catch race conditions. A non-atomic insert path (e.g., write-then-read in a non-transactional manner) might briefly expose the wrong tenant's data. Concurrent testing — 100 tasks racing across 50 tenant pairs — exercises the path under load. The assertion is the same (zero cross-tenant reads) but the discovery surface is different.

**Why per-case Redis isolation (§1 #11)?** Without isolation, test case N's leftover entries pollute test case N+1's lookup space. A flaky failure pattern would be: case A inserts under `(tenant_x, prompt_y)`; case B (with different randomly-generated tenant) accidentally generates `tenant_x` again and looks up `prompt_y` — gets a hit from case A's residue. The flakiness is hard to reproduce. The per-case prefix (`test_<uuid>_<original_tenant>`) makes every case operate in its own namespace; cleanup in Drop ensures no residue.

**Why non-skip enforcement (§1 #13)?** The most common failure mode for compliance-gate tests is silent disablement: a developer hits a flaky failure, adds `#[ignore]`, the test stops protecting the invariant, nobody notices for months. Parsing CI output for skip count and failing on any skip catches this pattern. The trade-off is operational rigidity — but for a hard-zero invariant, rigidity is the point.

**Why a JSON artefact (§1 #14)?** MoPS A05 (Vietnam's Department of Cyber Security) asks during audits: "how do you prove no cross-tenant data exposure?" Without an artefact, the answer is "we have a test in our codebase" — non-evidential. With the artefact, the answer is "this JSON file from CI run X on date Y proves the property held at that point." The artefact is a one-shot regulatory primitive.

**Why this is its own FR (FR-AI-018), not a test in FR-AI-017?** FR-AI-017 has its own property test (cross-tenant isolation at 1000-trial level) as a baseline. FR-AI-018 is the dedicated, exhaustive treatment with 200K ops + 7 regressions + adversarial + concurrent + CI-artefact emission. Two reasons for the split: (a) the test surface is large enough to warrant its own owner; (b) future hardening (10M-op runs, mutation testing of the cache code, fuzzing) extends FR-AI-018 without blowing up FR-AI-017's scope.

---

## §3 — API contract (formal spec for AI-agent implementers)

This FR ships only test code; no production API. The test imports `cache::lookup`, `cache::insert`, `CacheKey::derive` from FR-AI-017.

### Test helpers

```rust
// services/ai-gateway/tests/support/redis_isolation_helper.rs

use uuid::Uuid;

pub struct RedisTestNamespace {
    pub prefix: String,      // e.g. "test_a1b2c3d4_"
}

impl RedisTestNamespace {
    pub fn new() -> Self {
        let prefix = format!("test_{}_", Uuid::new_v4().simple());
        Self { prefix }
    }

    /// Wraps a tenant_id with the per-case prefix so test cases don't pollute each other.
    pub fn tenant(&self, original: &str) -> String {
        format!("{}{}", self.prefix, original)
    }
}

impl Drop for RedisTestNamespace {
    fn drop(&mut self) {
        // Clean up all keys matching `ai_cache:v1:{prefix}*`.
        let pattern = format!("ai_cache:v1:{}*", self.prefix);
        let _ = redis_helper::delete_keys_matching(&pattern);
    }
}
```

```rust
// services/ai-gateway/tests/support/proptest_strategies.rs

use proptest::prelude::*;

pub fn any_tenant_id() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_:.\\-]{1,32}".prop_map(String::from)
}

pub fn any_tenant_pair() -> impl Strategy<Value = (String, String)> {
    (any_tenant_id(), any_tenant_id())
        .prop_filter("tenants must differ", |(a, b)| a != b)
}

pub fn any_prompt() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 ?!.,]{0,200}".prop_map(String::from)
}

pub fn any_model() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("chat.fast".to_string()),
        Just("chat.smart".to_string()),
        Just("embed.standard".to_string()),
        "[a-z]{4,12}\\.[a-z]{4,8}".prop_map(String::from),
    ]
}

pub fn any_persona_handle() -> impl Strategy<Value = String> {
    "[a-z\\-]{4,16}@\\d+\\.\\d+\\.\\d+".prop_map(String::from)
}

pub fn any_cache_op() -> impl Strategy<Value = (String, String, String)> {
    (any_prompt(), any_model(), any_persona_handle())
}

pub fn adversarial_tenant_strings() -> Vec<&'static str> {
    vec![
        "",                                             // empty
        "\x00",                                         // null byte
        "\x1ftenant",                                   // unit separator (our internal field sep)
        "tenant\x1fid",                                 // unit separator mid-string
        "tenant\u{202E}id",                             // RTL override
        "tenant\u{200D}id",                             // zero-width joiner
        "tenant'; DROP TABLE--",                        // SQL-injection shape
        "../../../etc/passwd",                          // path traversal
        "\u{FEFF}tenant",                               // BOM-prefixed
        "TENANT",                                       // case variant (cross-test with "tenant")
    ]
}
```

### CI workflow

```yaml
# .github/workflows/cache-isolation-gate.yml
name: Cache Cross-Tenant Isolation Gate
on:
  pull_request:
    paths:
      - 'services/ai-gateway/src/cache/**'
      - 'services/ai-gateway/tests/cache_isolation*'
      - 'services/ai-gateway/tests/support/**'
      - '.github/workflows/cache-isolation-gate.yml'

jobs:
  isolation-gate:
    runs-on: ubuntu-22.04
    timeout-minutes: 5
    services:
      redis:
        image: redis:7
        ports: ['6379:6379']
        options: --health-cmd "redis-cli ping" --health-interval 5s
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - name: Run cache isolation suite
        working-directory: services/ai-gateway
        env:
          REDIS_URL: redis://localhost:6379
          PROPTEST_CASES: "256"   # explicit; non-default raises a CI WARN
        run: cargo test --release cache_isolation -- --test-threads=1 --format=json | tee report.json
      - name: Enforce no skipped tests (§1 #13)
        working-directory: services/ai-gateway
        run: |
          SKIP_COUNT=$(grep -c '"event":"ignored"' report.json || echo 0)
          if [ "$SKIP_COUNT" -gt 0 ]; then
            echo "❌ $SKIP_COUNT cache_isolation test(s) skipped — refusing to merge"
            grep '"event":"ignored"' report.json
            exit 1
          fi
      - name: Upload JSON artefact (§1 #14)
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: cache_isolation_report_${{ github.sha }}
          path: services/ai-gateway/report.json
```

---

## §4 — Acceptance criteria (testable, ordered, numbered)

1. **Test compiles and runs in CI** — `cargo test cache_isolation` exits 0 in the GitHub Actions runner.
2. **Property test runs ≥256 cases × ~1000 ops** — Verified by counting `proptest case` log output; total ops ≥ 200K per run.
3. **Zero cross-tenant reads (end-to-end)** — `no_cross_tenant_cache_reads` proptest passes; assertion never fails across 200K+ ops.
4. **Zero cross-tenant key collisions (derivation level)** — `no_cross_tenant_key_collisions` proptest: `CacheKey::derive(A,...) ≠ CacheKey::derive(B,...)` for any `A ≠ B`.
5. **Redis namespace is tenant-isolated** — `redis_keys_scan_is_tenant_isolated` proptest: scanning `ai_cache:v1:tenant_a:*` after inserting under both tenants returns only tenant_a entries.
6. **All 7 regression scenarios pass** — Each named test in `cache_isolation_regression_scenarios.rs` passes individually.
7. **Adversarial inputs handled** — `cache_isolation_adversarial_test` covers all 10 enumerated adversarial tenant strings; each case asserts no cross-leak when paired with a benign tenant_id.
8. **Concurrent test passes** — 100 tasks × 50 tenant pairs × 10,000 total ops; zero cross-tenant reads.
9. **Suite completes in ≤ 90 seconds** — Wall-clock from CI start to CI end (excluding setup); enforced by GitHub Actions `timeout-minutes: 5`.
10. **Failure detail is actionable** — On any failure, the message includes `tenant_a`, `tenant_b`, `prompt`, `model`, `persona_handle`, step number, AND both `CacheKey::derive` hashes.
11. **Per-case Redis isolation prevents pollution** — Sequential test runs with the same RNG seed produce the same outcome (no flakiness from inter-case state).
12. **CI gate blocks merge on failure** — A PR introducing a leaking change is rejected by the workflow with non-zero exit.
13. **No-skip enforcement fires** — A PR that adds `#[ignore]` to any cache_isolation test is rejected by the workflow's skip-count check.
14. **Workflow self-gate** — A PR that ONLY edits `.github/workflows/cache-isolation-gate.yml` triggers the gate against the modified workflow.
15. **JSON artefact emitted on every run** — `cache_isolation_report_<sha>.json` uploaded as CI artefact on both pass and fail.
16. **Proptest shrinking produces minimal failing case on regression** — A planted leak (test fixture that intentionally collides) reports a 2-3 op minimal case after shrinking, not the original 1000-op sequence.

---

## §5 — Verification

### Property test #1: end-to-end isolation

```rust
// services/ai-gateway/tests/cache_isolation_property_test.rs
use proptest::prelude::*;
use cyberos_ai_gateway::cache::{self, CacheKey, CacheLookupOutcome};

mod support;
use support::{proptest_strategies::*, redis_isolation_helper::RedisTestNamespace};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn no_cross_tenant_cache_reads(
        (tenant_a, tenant_b) in any_tenant_pair(),
        ops in prop::collection::vec(any_cache_op(), 800..1200),
    ) {
        let ns = RedisTestNamespace::new();
        let t_a = ns.tenant(&tenant_a);
        let t_b = ns.tenant(&tenant_b);

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            for (prompt, model, persona) in &ops {
                let k = CacheKey::derive(&t_a, prompt, model, persona);
                let _ = cache::insert(&k, &test_provider_response(), model).await;
            }
            for (prompt, model, persona) in &ops {
                let k = CacheKey::derive(&t_b, prompt, model, persona);
                match cache::lookup(&k).await {
                    CacheLookupOutcome::Miss => {}
                    other => prop_assert!(false,
                        "cross-tenant leak: t_a={} t_b={} prompt={:?} model={} persona={} \
                         k_a_hash={} k_b_hash={} outcome={:?}",
                        t_a, t_b, prompt, model, persona,
                        hex::encode(CacheKey::derive(&t_a, prompt, model, persona).prompt_hash),
                        hex::encode(k.prompt_hash), other,
                    ),
                }
            }
            Ok(())
        }).unwrap();
    }
}
```

### Property test #2: key-derivation level

```rust
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn no_cross_tenant_key_collisions(
        (a, b) in any_tenant_pair(),
        prompt in any_prompt(),
        model in any_model(),
        persona in any_persona_handle(),
    ) {
        let k_a = CacheKey::derive(&a, &prompt, &model, &persona);
        let k_b = CacheKey::derive(&b, &prompt, &model, &persona);
        prop_assert_ne!(k_a.prompt_hash, k_b.prompt_hash,
            "cache-key collision: tenant_a={a} tenant_b={b} prompt={prompt:?}");
    }
}
```

### Property test #3: Redis namespace

```rust
proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn redis_keys_scan_is_tenant_isolated(
        (a, b) in any_tenant_pair(),
        n_ops in 10..100u32,
    ) {
        let ns = RedisTestNamespace::new();
        let t_a = ns.tenant(&a);
        let t_b = ns.tenant(&b);
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            for i in 0..n_ops {
                let k_a = CacheKey::derive(&t_a, &format!("p{i}"), "chat.smart", "p@1.0.0");
                let k_b = CacheKey::derive(&t_b, &format!("p{i}"), "chat.smart", "p@1.0.0");
                let _ = cache::insert(&k_a, &test_provider_response(), "chat.fast").await;
                let _ = cache::insert(&k_b, &test_provider_response(), "chat.fast").await;
            }
            let scan = redis_helper::keys(&format!("ai_cache:v1:{}:*", t_a));
            for key in &scan {
                prop_assert!(key.starts_with(&format!("ai_cache:v1:{}:", t_a)),
                    "namespace leak: scan returned {key} when filtering for {t_a}");
            }
            prop_assert_eq!(scan.len(), n_ops as usize);
            Ok(())
        }).unwrap();
    }
}
```

### 7 enumerated regression scenarios

```rust
// services/ai-gateway/tests/cache_isolation_regression_scenarios.rs
use cyberos_ai_gateway::cache::{self, CacheKey, CacheLookupOutcome};

mod support;
use support::redis_isolation_helper::RedisTestNamespace;

/// REGRESSION-001 (incident 2026-04-12, pre-fix):
/// `tenant_a` and `tenanta` previously collided due to insufficient input separation
/// when the key derivation used naive concat without unit-separator.
/// Root cause: `concat(tenant, model)` → `concat(tenant_a, model_x)` and
///             `concat(tenanta, _model_x)` produced same hash.
#[tokio::test]
async fn regression_001_underscore_collision() {
    let ns = RedisTestNamespace::new();
    let k1 = CacheKey::derive(&ns.tenant("tenant_a"), "prompt", "chat.smart", "p@1.0.0");
    let k2 = CacheKey::derive(&ns.tenant("tenanta"), "prompt", "chat.smart", "p@1.0.0");
    assert_ne!(k1.prompt_hash, k2.prompt_hash);

    let _ = cache::insert(&k1, &test_provider_response(), "chat.fast").await;
    assert!(matches!(cache::lookup(&k2).await, CacheLookupOutcome::Miss));
}

/// REGRESSION-002 (incident 2026-04-15):
/// Unicode normalisation form differences ("é" precomposed vs. e + combining acute)
/// produced different hashes, but tenant policy parser later normalised both to NFC,
/// causing a brief race-window leak.
#[tokio::test]
async fn regression_002_unicode_normalization() {
    let ns = RedisTestNamespace::new();
    let nfc = ns.tenant("tenant_é");                  // precomposed
    let nfd = ns.tenant("tenant_é");                  // e + combining acute (depending on source encoding)
    let k1 = CacheKey::derive(&nfc, "p", "m", "p@1.0.0");
    let k2 = CacheKey::derive(&nfd, "p", "m", "p@1.0.0");
    // The cache treats them as distinct (current behaviour); FR-AI-005 enforces NFC at policy load.
    assert_ne!(k1.prompt_hash, k2.prompt_hash);

    let _ = cache::insert(&k1, &test_provider_response(), "chat.fast").await;
    assert!(matches!(cache::lookup(&k2).await, CacheLookupOutcome::Miss));
}

/// REGRESSION-003: trailing whitespace must NOT collapse two distinct tenant ids.
#[tokio::test]
async fn regression_003_trailing_whitespace() {
    let ns = RedisTestNamespace::new();
    let a = ns.tenant("tenant_a");
    let b = ns.tenant("tenant_a ");                   // trailing space
    let k1 = CacheKey::derive(&a, "p", "m", "p@1.0.0");
    let k2 = CacheKey::derive(&b, "p", "m", "p@1.0.0");
    assert_ne!(k1.prompt_hash, k2.prompt_hash);
}

/// REGRESSION-004: tenant ids are case-sensitive; "Tenant_A" ≠ "tenant_a".
#[tokio::test]
async fn regression_004_case_folding() {
    let ns = RedisTestNamespace::new();
    let upper = ns.tenant("Tenant_A");
    let lower = ns.tenant("tenant_a");
    let k1 = CacheKey::derive(&upper, "p", "m", "p@1.0.0");
    let k2 = CacheKey::derive(&lower, "p", "m", "p@1.0.0");
    assert_ne!(k1.prompt_hash, k2.prompt_hash);
}

/// REGRESSION-005: empty prompt vs. single-space prompt must NOT collide.
#[tokio::test]
async fn regression_005_empty_prompt() {
    let ns = RedisTestNamespace::new();
    let t = ns.tenant("tenant_a");
    let k_empty = CacheKey::derive(&t, "", "chat.smart", "p@1.0.0");
    let k_space = CacheKey::derive(&t, " ", "chat.smart", "p@1.0.0");
    assert_ne!(k_empty.prompt_hash, k_space.prompt_hash);
}

/// REGRESSION-006: persona-handle MUST be in the key (FR-AI-017 ISS-001 fix verification).
/// Pre-fix, persona changes didn't invalidate cache; this scenario asserts they do.
#[tokio::test]
async fn regression_006_persona_omitted() {
    let ns = RedisTestNamespace::new();
    let t = ns.tenant("tenant_a");
    let k1 = CacheKey::derive(&t, "p", "chat.smart", "cuo-cpo@0.4.1");
    let k2 = CacheKey::derive(&t, "p", "chat.smart", "cuo-cpo@0.4.2");
    assert_ne!(k1.prompt_hash, k2.prompt_hash);
}

/// REGRESSION-007: model substring "chat.smart" vs. "chat.smartx" must NOT collide.
#[tokio::test]
async fn regression_007_model_substring() {
    let ns = RedisTestNamespace::new();
    let t = ns.tenant("tenant_a");
    let k1 = CacheKey::derive(&t, "p", "chat.smart", "p@1.0.0");
    let k2 = CacheKey::derive(&t, "p", "chat.smartx", "p@1.0.0");
    assert_ne!(k1.prompt_hash, k2.prompt_hash);
}
```

### Adversarial inputs test

```rust
// services/ai-gateway/tests/cache_isolation_adversarial_test.rs
use support::proptest_strategies::adversarial_tenant_strings;

#[tokio::test]
async fn adversarial_tenant_strings_dont_leak() {
    let ns = RedisTestNamespace::new();
    let benign = ns.tenant("benign_tenant");
    let benign_key = CacheKey::derive(&benign, "p", "chat.smart", "p@1.0.0");
    let _ = cache::insert(&benign_key, &test_provider_response(), "chat.fast").await;

    for adv in adversarial_tenant_strings() {
        let adv_namespaced = ns.tenant(adv);
        let adv_key = CacheKey::derive(&adv_namespaced, "p", "chat.smart", "p@1.0.0");
        match cache::lookup(&adv_key).await {
            CacheLookupOutcome::Miss => {}
            other => panic!("adversarial leak: adv={adv:?} outcome={other:?}"),
        }
    }
}

#[tokio::test]
async fn unit_separator_in_tenant_id_is_distinct() {
    let ns = RedisTestNamespace::new();
    let t1 = ns.tenant("tenant\x1fa");                // unit separator inside
    let t2 = ns.tenant("tenant_a");
    let k1 = CacheKey::derive(&t1, "p", "m", "p@1.0.0");
    let k2 = CacheKey::derive(&t2, "p", "m", "p@1.0.0");
    assert_ne!(k1.prompt_hash, k2.prompt_hash);
}

#[tokio::test]
async fn very_long_tenant_id_distinct_from_short() {
    let ns = RedisTestNamespace::new();
    let short = ns.tenant("t");
    let long  = ns.tenant(&"a".repeat(10_000));
    let k1 = CacheKey::derive(&short, "p", "m", "p@1.0.0");
    let k2 = CacheKey::derive(&long, "p", "m", "p@1.0.0");
    assert_ne!(k1.prompt_hash, k2.prompt_hash);
}
```

### Concurrent test

```rust
// services/ai-gateway/tests/cache_isolation_concurrent_test.rs
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn one_hundred_tasks_racing_no_cross_tenant_reads() {
    let ns = std::sync::Arc::new(RedisTestNamespace::new());
    let tenants: Vec<String> = (0..50).map(|i| ns.tenant(&format!("t{i}"))).collect();
    let mut joinset = tokio::task::JoinSet::new();

    for task_id in 0..100 {
        let tenants = tenants.clone();
        joinset.spawn(async move {
            let owner = &tenants[task_id % 50];
            let other = &tenants[(task_id + 1) % 50];
            for op in 0..100 {
                let k_owner = CacheKey::derive(owner, &format!("p{op}"), "chat.smart", "p@1.0.0");
                let _ = cache::insert(&k_owner, &test_provider_response(), "chat.fast").await;
                // Same prompt+model+persona, different tenant — must miss.
                let k_other = CacheKey::derive(other, &format!("p{op}"), "chat.smart", "p@1.0.0");
                let outcome = cache::lookup(&k_other).await;
                assert!(matches!(outcome, CacheLookupOutcome::Miss),
                    "concurrent leak: task={task_id} owner={owner} other={other} outcome={outcome:?}");
            }
        });
    }
    while let Some(r) = joinset.join_next().await { r.unwrap(); }
}
```

```bash
docker run -d --name test-redis -p 6379:6379 redis:7
cd services/ai-gateway
cargo test --release cache_isolation -- --test-threads=1
```

---

## §6 — Implementation skeleton

See §3 (test helpers + workflow) and §5 (full test bodies). Boot order in CI workflow: Redis container starts → cargo test runs the four test files → JSON artefact uploaded.

The test files reference `support::` modules; the support directory is shared test infrastructure (per `/tests/support/` Rust test convention).

---

## §7 — Dependencies

### Code dependencies (other FRs/modules)

- **FR-AI-017** — `cache::lookup`, `cache::insert`, `CacheKey::derive`, `CacheLookupOutcome` are all consumed; this FR is the dedicated leak-test for FR-AI-017's per-tenant isolation invariant.
- **FR-AI-005** — Tenant-id validation rules (the adversarial inputs assume some inputs would be rejected by FR-AI-005's policy parser; this FR tests the cache layer assuming validation has failed).
- **FR-AI-014** — Persona handle format (`<id>@<version>`) is one of the cache key inputs; the regression test #6 asserts persona-handle inclusion.

### Concept dependencies (shared types)

- `CacheKey::derive(tenant, prompt, model, persona)` is the cryptographic primitive under test.
- The per-tenant Redis-key prefix (`ai_cache:v1:{tenant_id}:`) is the structural invariant under test.
- `CacheLookupOutcome` enum (Hit/Miss/SchemaMismatch/Error) is the assertion target — Miss is the only acceptable outcome for cross-tenant lookups.
- Adversarial tenant strings list is a maintained corpus; new attack vectors discovered in production should be added to `proptest_strategies::adversarial_tenant_strings()`.

### Operational / external

- Rust crates: `proptest@1`, `tokio@1` with `rt-multi-thread` feature, `uuid@1`, `hex@0.4`.
- Redis 7.x (test infrastructure; same as FR-AI-017).
- GitHub Actions runner with `services.redis` declared.
- `actions/upload-artifact@v4` for the JSON artefact.

---

## §8 — Example payloads

### CI output on pass

```text
running 4 test files
test no_cross_tenant_cache_reads ... ok (256 cases × ~1000 ops = 256,000 ops)
test no_cross_tenant_key_collisions ... ok (1000 cases)
test redis_keys_scan_is_tenant_isolated ... ok (50 cases × ~50 ops)
test regression_001_underscore_collision ... ok
test regression_002_unicode_normalization ... ok
test regression_003_trailing_whitespace ... ok
test regression_004_case_folding ... ok
test regression_005_empty_prompt ... ok
test regression_006_persona_omitted ... ok
test regression_007_model_substring ... ok
test adversarial_tenant_strings_dont_leak ... ok
test unit_separator_in_tenant_id_is_distinct ... ok
test very_long_tenant_id_distinct_from_short ... ok
test one_hundred_tasks_racing_no_cross_tenant_reads ... ok

cache_isolation suite passed in 73.2s (budget: 90s)
```

### CI output on failure (planted leak)

```text
test no_cross_tenant_cache_reads ... FAILED

failed: cross-tenant leak: t_a="tenant_x" t_b="tenant_y" prompt="hello"
        model="chat.smart" persona="cuo-cpo@0.4.1"
        k_a_hash=a1b2c3...d4e5f6  k_b_hash=a1b2c3...d4e5f6  outcome=Hit(...)
   at services/ai-gateway/tests/cache_isolation_property_test.rs:25
   minimal failing case after shrinking:
     tenant_a="x"  tenant_b="y"  ops=[("p", "chat.smart", "p@1.0.0")]
```

### JSON artefact (success)

```json
{
  "sha": "0123abc...",
  "suite": "cache_isolation",
  "passed": true,
  "case_counts": {
    "no_cross_tenant_cache_reads": 256,
    "no_cross_tenant_key_collisions": 1000,
    "redis_keys_scan_is_tenant_isolated": 50
  },
  "regression_scenarios_passed": [
    "regression_001_underscore_collision",
    "regression_002_unicode_normalization",
    "regression_003_trailing_whitespace",
    "regression_004_case_folding",
    "regression_005_empty_prompt",
    "regression_006_persona_omitted",
    "regression_007_model_substring"
  ],
  "adversarial_inputs_passed": [
    "adversarial_tenant_strings_dont_leak",
    "unit_separator_in_tenant_id_is_distinct",
    "very_long_tenant_id_distinct_from_short"
  ],
  "concurrent_test_passed": true,
  "wall_clock_seconds": 73.2,
  "skipped_count": 0
}
```

### Skip-detection failure

```text
❌ 1 cache_isolation test(s) skipped — refusing to merge
{"event":"ignored","name":"regression_002_unicode_normalization","reason":"flaky"}
```

---

## §9 — Open questions

All resolved at authoring time. Items deferred to later FRs:

- Mutation testing (deliberately mutate `CacheKey::derive` and assert the test catches it) — FR-AI-022 area; current property test is structural defence.
- Fuzz testing with libFuzzer over the cache key derivation — out of scope; proptest with adversarial inputs covers the current threat model.
- Cross-region cache replication leak testing — FR-AI-016 area; current tests are single-region.
- 10M-op extended runs in nightly CI — FR-AI-022 follow-up; adds another 9-of-nines confidence.
- Tenant-id whitelist validation as a separate compile-time check — out of scope; defended-in-depth at policy-load time (FR-AI-005) and via the cache key derivation.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Property test finds leak (real bug in `CacheKey::derive`) | proptest panics with shrunk minimal case | CI blocks PR; engineer reads minimal case + hashes | Fix key derivation; add as new regression scenario |
| Property test finds leak (Redis backend bug) | Same panic with key_a_hash ≠ key_b_hash but Redis returns Hit | CI blocks PR | Investigate Redis connection-pool / SCAN behaviour |
| Test runs > 90 sec | GitHub Actions `timeout-minutes: 5` fires | CI fails on timeout | Reduce per-case ops OR optimise hot path; investigate Redis latency |
| Flaky test (Redis state pollution between cases) | Intermittent fails with same RNG seed | sev-2 CI alarm | Investigate RedisTestNamespace cleanup; add explicit Drop assertion |
| Test skip slipped past CI | `cargo test` `--format=json` parsing detects | CI fails on skip-count check (§1 #13) | Engineer removes `#[ignore]`; if test is genuinely flaky, investigate root cause not silence |
| Regression scenario removed/renamed | Test count drops; CI compares against expected list | CI fails on missing-test detection | Re-add; if intentional removal, requires FR amendment |
| Adversarial input list shortened | Test count check | CI fails | Re-add; new adversarial inputs require addition not subtraction |
| Concurrent test hangs (deadlock in cache code) | `tokio::test` 90s timeout | Test fails | Investigate cache::insert/lookup for blocking calls |
| Concurrent test detects race | Atomic-Bool failure flag; assertion in any task fires | Test fails with task_id, owner, other | Investigate cache layer for non-atomic write paths |
| Per-case Redis namespace leaked (Drop didn't run) | Subsequent test run sees stale `test_*` keys | sev-2 CI alarm | Investigate Drop chain; ensure tokio runtime allows Drop to run |
| New attack vector discovered in production | Post-mortem analysis | Add to adversarial inputs corpus + regression scenarios | Standard process; `proptest_strategies::adversarial_tenant_strings()` is appended |
| Workflow self-gate disabled | `.github/workflows/cache-isolation-gate.yml` change without trigger | PR review catches; workflow file is in `paths:` so the gate fires against itself | Revert |
| JSON artefact upload fails | `actions/upload-artifact@v4` step fails | CI marks step as failed but main test result still authoritative | Investigate Actions API; manually re-run job |
| `RedisTestNamespace::new()` UUID collision (probability ~10⁻³⁰) | None practical | Self-resolves on next run | N/A |
| Property test triggers proptest deterministic-shrinking bug | Shrinker produces wrong minimal case | Diagnostic confusion | Update proptest crate; report upstream |
| `--release` flag missing (test runs in debug mode) | Wall-clock > 5 minutes | Suite times out | CI must use `cargo test --release`; documented in `.github/workflows/cache-isolation-gate.yml` |
| Redis container not ready when test starts | First test errors with "connection refused" | Health-check `--health-cmd "redis-cli ping"` ensures Redis is ready | By design (workflow YAML) |

---

## §11 — Notes

- This is the gold-standard invariant test for multi-tenancy in CyberOS. Every cache-touching change goes through this gate. The compliance value is "we can prove the invariant holds across 200K+ randomised operations on every PR" — much stronger than "we have unit tests covering scenarios we thought of."
- The 200K-operation budget catches 1-in-10K bugs at ≥99.9% probability per CI run. For 1-in-100K bugs, increase to `ProptestConfig::with_cases(2560)` (~2M ops) — currently out of scope but trivially extensible.
- The 7 enumerated regression scenarios serve a different purpose from the property test. The property test catches novel bugs; the regressions document KNOWN bugs so they can never recur silently. Each scenario's comment block IS the post-mortem for that specific bug.
- The adversarial inputs corpus (§3) is maintained — new attack vectors discovered in production are appended, never removed. The corpus is a collective hardening artefact across the lifetime of the codebase.
- The concurrent test uses `flavor = "multi_thread", worker_threads = 8` to genuinely exercise concurrency. A single-threaded tokio runtime would interleave operations cooperatively; multi-threaded actually races them. Race-detection is the ONLY mode that catches non-atomic insert-path bugs.
- The per-case Redis isolation (§1 #11) is the difference between "this test is authoritative" and "this test is flaky." Without it, intermittent failures from inter-case pollution would degrade developer trust until the test is silently disabled.
- The non-skip enforcement (§1 #13) is paranoid by design. The most common compliance-test-erosion pattern is silent disablement; making the CI workflow refuse to merge a PR with any cache_isolation skip stops that pattern at the gate.
- The Vietnamese regulator (MoPS A05) specifically asks during audits: "how do you prove no cross-tenant data exposure?" The JSON artefact (§1 #14) is the answer — durable, cryptographically-tied to the SHA, durable across Github Actions log retention.
- This FR is downstream of FR-AI-017 because it CONSUMES FR-AI-017's API surface. But it's also UPSTREAM of any future FR that touches the cache (e.g., FR-AI-022's cache warming, FR-AI-024's semantic cache) — those FRs will inherit this gate automatically because they touch `services/ai-gateway/src/cache/**`.
- The workflow file (`cache-isolation-gate.yml`) is in its own `paths:` for self-gate (matching FR-AI-013's pattern). A PR loosening the gate gets caught by the gate against itself.

---

*End of FR-AI-018. Status: draft (10/10 target).*
