---
# ───── Machine-readable frontmatter (parsed by feature-request-audit + future fr-catalog renderer) ─────
id: FR-AI-007
title: "Provider cost-table loader — YAML-backed, hot-reloadable rate table"
module: AI
priority: MUST
status: ready_to_test
verify: T
phase: P0
milestone: P0 · slice 2
slice: 2
owner: Stephen Cheng
created: 2026-05-15
shipped: null
memory_chain_hash: null
related_frs: [FR-AI-001, FR-AI-002, FR-AI-005, FR-AI-006, FR-AI-008]
depends_on: []
blocks: [FR-AI-001, FR-AI-002, FR-AI-006, FR-AI-008]

# ───── Source contracts ─────
source_pages:
  - website/docs/modules/ai.html#cost-gate
  - website/docs/modules/ai.html#multi-provider
source_decisions:
  - docs/feature-requests/ai/FR-AI-001-cost-ledger-precheck.md §1 #8 (YAML config promoted to normative)
  - docs/feature-requests/ai/FR-AI-005-tenant-policy-yaml-loader.md (loader pattern reuse)
  - docs/feature-requests/ai/FR-AI-006-model-alias-resolution.md §1 #5 (cost-entry validation)
  - archive/2026-05-14/RESEARCH_REVIEW.md §2.4 (cost-of-everything invariant)

# ───── Build envelope ─────
language: rust 1.81
service: cyberos/services/ai-gateway/
new_files:
  - services/ai-gateway/src/cost_table.rs
  - services/ai-gateway/src/cost_table/loader.rs
  - services/ai-gateway/src/cost_table/schema.rs
  - services/ai-gateway/config/cost_rates.yaml          # initial seed data
  - services/ai-gateway/tests/cost_table_test.rs
  - services/ai-gateway/tests/fixtures/cost_table/valid_rates.yaml
  - services/ai-gateway/tests/fixtures/cost_table/negative_rate.yaml
  - services/ai-gateway/tests/fixtures/cost_table/missing_field.yaml
modified_files:
  - services/ai-gateway/src/cost_ledger.rs   # use cost_table::lookup in precheck
  - services/ai-gateway/src/alias.rs         # FR-AI-006 cost-entry check uses this
  - services/ai-gateway/src/lib.rs            # export cost_table module
allowed_tools:
  - file_read: services/ai-gateway/**
  - file_write: services/ai-gateway/{src,tests,config}/**
  - bash: cargo test -p cyberos-ai-gateway cost_table
  - bash: cargo bench --bench cost_table_lookup_bench
disallowed_tools:
  - hardcode cost values inline (anywhere outside cost_rates.yaml + its loader)
  - persist cost rates to Postgres or any other DB at slice 2 (YAML is the only source)
  - mix currencies in the cost table (USD only; FR-INV-002 handles FX)
  - call cost_table::lookup from a hot path that doesn't already pay the lookup cost

# ───── Estimated work ─────
effort_hours: 4
sub_tasks:
  - "0.5h: CostRate struct + RawCostTable deserialise shape + FileFailure error type"
  - "0.5h: cost_rates.yaml schema (provider → model → rate) with seed entries for slice-2 providers"
  - "0.5h: validate_and_flatten — aggregate failures, validate non-negative, validate provider-kind parse"
  - "1.0h: ArcSwap-backed in-memory cache + lookup() function"
  - "1.0h: notify-based hot reload with 100ms debounce + last-known-good fallback"
  - "0.5h: OTel metric registration + emit on lookup + reload"
  - "1.0h: integration tests (8 cases — happy / missing entry / invalid YAML / hot-reload / negative rate / aggregate failures / concurrent / determinism)"
risk_if_skipped: "FR-AI-001 (precheck) cannot compute estimated cost — every chat call fails before reaching the provider. FR-AI-002 (reconcile) cannot compute actual cost from provider usage. FR-AI-006 (alias resolve) cannot validate that the resolved model has a known price. Every consumer would either hardcode rates (creating 22 places to update on every provider price change) or fail. The whole cost-of-everything gate becomes non-functional — i.e., the entire P0 AI Gateway investment becomes worthless. This is the most load-bearing 4h FR in slice 2."
---

## §1 — Description (BCP-14 normative)

The AI Gateway service **MUST** load a YAML cost table at startup from `services/ai-gateway/config/cost_rates.yaml` and expose `cost_table::lookup(provider, model) -> Option<CostRate>` for synchronous in-memory lookup by every consumer that needs to estimate or reconcile cost. The loader:

1. **MUST** read the YAML at gateway startup; the loader runs **before** any chat request can be served (startup ordering: loader → cost-table init → policy init → router init → HTTP listener bound).
2. **MUST** validate every entry per the schema in §3: `input_per_1k_usd >= 0` and `output_per_1k_usd >= 0` (using `rust_decimal::Decimal` for precision), provider name MUST be one of the closed `ProviderKind` enum values (`bedrock`, `anthropic`, `openai`, `vertex`, `bge`), model name MUST be a non-empty string of length ≤ 256.
3. **MUST** aggregate ALL validation failures into a single `LoaderInitError::Schema { failures: Vec<FileFailure> }`. Reporting only the first failure forces multi-deploy iteration; aggregation lets the operator fix everything at once.
4. **MUST** reject the entire load on any aggregate failure — the gateway exits 1 at startup, no partial loading. Negative rates or unknown provider names are configuration errors that fail loud.
5. **MUST** support hot-reload via the `notify` crate with a 100ms debounce (avoid burst-edit thrash). On a successful reload, the new in-memory table replaces the old via `ArcSwap::store()` atomically.
6. **MUST** preserve the last-known-good table on hot-reload validation failure. If a malformed YAML is saved, the loader emits `tracing::error!` and `ai_cost_table_reload_failures_total{reason}` increments; the in-memory cache stays at the prior valid state. Hot-reload failure is non-fatal at runtime (unlike startup failure).
7. **MUST** expose `cost_table::lookup(provider, model) -> Option<CostRate>` as the only public read interface. The lookup is synchronous, returns `None` on miss (callers translate to their own error types), and completes in <1µs p95 on a 100-entry table.
8. **MUST** support concurrent reads from many tokio tasks without contention via the `ArcSwap<HashMap<(ProviderKind, String), CostRate>>` pattern. Read-path is lock-free; write-path (hot-reload) does a single atomic swap.
9. **MUST** keep cost rates in **USD only**. Multi-currency conversion happens at invoice time (FR-INV-002), not at cost-ledger time. Mixing currencies here would force every precheck/reconcile comparison to FX-convert, which is non-deterministic across timezones and breaks reproducibility.
10. **MUST** use `Decimal` with 6 decimal places of precision (`NUMERIC(12,6)` semantically). Cost rates like `gpt-4o-mini`'s `$0.00015/1k` need sub-cent precision; floating point would lose accuracy on aggregation.
11. **MUST** maintain a `last_updated` field at YAML top level (human-edited at every rate change). A CI lint MUST warn on entries older than 90 days; the warning is non-blocking but visible in PR reviews.
12. **MUST** support a per-model `is_embedding: true` flag (in `CostRate`) where `output_per_1k_usd == 0.0` — embeddings have no output tokens by definition. This is informational metadata that downstream callers (FR-AI-002 reconcile) can use to short-circuit output cost calculation.
13. **MUST** emit OTel metrics: `ai_cost_table_lookups_total{provider,outcome}` (counter; outcome ∈ hit/miss), `ai_cost_table_entries_total` (gauge; current entry count), `ai_cost_table_reload_failures_total{reason}` (counter; reason ∈ parse_error/validation_error/io_error), `ai_cost_table_loaded_at_ts` (gauge; UNIX timestamp of last successful load), `ai_cost_table_lookup_latency_ns` (histogram).
14. **MUST** expose `cost_table::loaded_at() -> Option<DateTime<Utc>>` for the operator CLI (FR-AI-021 `models pricing` subcommand) to show when the table was last loaded.
15. **MUST NOT** persist cost rates anywhere except in-memory (no Postgres write at slice 2). The TEN module's per-tenant override table (FR-TEN-006, P2) introduces DB-backed overrides; the YAML stays canonical and is read first.

---

## §2 — Why this design (rationale for humans)

**Why YAML and not a database?** Same reasoning as FR-AI-005's policy loader: cost rates change roughly weekly (a new model launches, Bedrock pricing updates, OpenAI deprecates `gpt-4-turbo`). Git-tracked YAML gives every rate change a PR + reviewer + diff. The audit trail of "who lowered the chat.smart rate on 2026-04-15?" is built into git blame, not in a separate audit log. The TEN module (FR-TEN-006, deferred to P2) eventually adds a Postgres-backed override for per-tenant negotiated rates (e.g., enterprise tenants who get 10% off), but the YAML stays as the canonical bootstrap source and the fallback when TEN's DB is unreachable.

**Why is `lookup()` synchronous (no async)?** This is on the precheck hot path of every chat call. Precheck's budget is 50ms p95. The cost-table lookup is called twice in the hot path (once in `alias::resolve` for validation, once in `cost_ledger::precheck` for the estimate). Adding async I/O would add 2 × ~5ms = 10ms — a 20% chunk of the precheck budget for what is fundamentally a HashMap lookup. Keeping it synchronous and in-memory makes the lookup "free" (sub-microsecond).

**Why aggregate validation failures?** Same operator-experience reason as FR-AI-005 §1 #11. An operator misconfiguring 3 entries (say, three new models added with copy-paste errors) gets all 3 failures in one error message. Without aggregation: fix one, redeploy, find the next, redeploy, find the third. Three deploys to find three errors. With aggregation: one deploy, all three visible.

**Why USD only?** Provider invoices arrive in USD. AI providers (Bedrock, Anthropic, OpenAI) all publish prices in USD. Tenant billing is in tenant currency (VND for Vietnamese tenants, EUR for EU, SGD for Singapore, etc.) — that conversion happens at invoice issue time in FR-INV-002, using SBV's daily FX rate published at 16:00 ICT. Mixing currencies at the cost-ledger layer would force every comparison ("is this call over the tenant's monthly cap?") to FX-convert, which is non-deterministic across timezones AND breaks reproducibility (the same audit row could yield different "in cap" verdicts on different days if FX moved).

**Why `Decimal` not `f64`?** Floating point is wrong for money. `gpt-4o-mini`'s rate is `$0.00015/1k`. A call that uses 1000 prompt tokens + 500 completion tokens costs `1.0 * 0.00015 + 0.5 * 0.0006 = 0.00045`. In `f64`, you get `0.00044999999999999996` due to representation. In `Decimal`, you get exactly `0.000450`. Aggregating thousands of these for a month, the `f64` error compounds; the `Decimal` is exact. This is a "decimal everywhere" project — cost ledger uses `Decimal`, FR-AI-001's holds use `Decimal`, FR-AI-002's reconciliation uses `Decimal`. Consistency.

**Why hot-reload at all?** Two reasons. (1) Provider price changes happen mid-month (Anthropic dropped `claude-3-haiku` prices in March 2026; we want that effective immediately, not at next gateway restart). (2) New model launches: Bedrock adds `claude-3.5-haiku-20251022` and the cost-rate YAML edit is the only blocker between "deploy and serve" — without hot-reload, we'd need a full gateway restart for every new model. With hot-reload, it's a `git push` and ~100ms later the model is bookable.

**Why 100ms debounce on the file-watch?** YAML edits via vim/vscode often generate multiple file-modification events (atomic write produces a rename + a write). Without debounce, the loader would try to parse three times in 30ms — twice on partial files (parse errors) and once on the final file. With 100ms debounce, the loader waits until the editor settles, then parses once on the final state.

**Why preserve last-known-good on hot-reload failure?** Failing hard on hot-reload failure would mean a single bad YAML edit (a stray colon, a typo on a model name) takes the gateway offline. That's an unacceptable trade — the prior cost table was working fine; running with it for a few minutes while the operator fixes the YAML is much safer than failing requests with `503 NO_COST_TABLE`. The trade-off is asymmetric: startup failure is loud (operator sees the deploy fail immediately and fixes); runtime failure is silent-but-recoverable (operator sees the OBS alarm and fixes within minutes).

**Why expose `is_embedding` metadata?** FR-AI-002's reconcile path computes `actual_usd = prompt_tokens × input_rate + completion_tokens × output_rate`. For embeddings, `completion_tokens` is always 0 by definition (embeddings produce vectors, not tokens). Without `is_embedding`, FR-AI-002 has no way to validate that a non-zero `completion_tokens` value on an embedding response is a bug. With it, FR-AI-002 can assert `is_embedding ⇒ completion_tokens == 0` and surface the bug at reconcile time.

---

## §3 — API contract (formal spec for AI-agent implementers)

### Public function signatures

```rust
// services/ai-gateway/src/cost_table.rs

/// Synchronous lookup against the in-memory cost table.
/// Returns None on miss; never panics; lock-free read via ArcSwap.
pub fn lookup(provider: &ProviderKind, model: &str) -> Option<CostRate>;

/// Returns the UNIX timestamp of the last successful load (or None if never loaded).
/// Used by FR-AI-021 operator CLI.
pub fn loaded_at() -> Option<DateTime<Utc>>;

/// Returns the current entry count (gauge value).
pub fn entry_count() -> usize;

/// Initialise the cost table at gateway startup.
/// MUST be called before the HTTP listener is bound.
pub async fn init_cost_table(config_path: &Path) -> Result<CostTableHandle, LoaderInitError>;
```

### Types

```rust
// services/ai-gateway/src/cost_table/schema.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CostRate {
    pub input_per_1k_usd:  Decimal,    // NUMERIC(12,6) precision — sub-cent matters
    pub output_per_1k_usd: Decimal,    // 0.0 for embeddings
    pub is_embedding: bool,            // §1 #12 — embeddings produce no output tokens
}

#[derive(Debug)]
pub enum LoaderInitError {
    /// One or more YAML files failed validation; ALL failures reported in one error.
    Schema { failures: Vec<FileFailure> },
    /// IO error reading the file (missing, permission denied, etc.).
    IoError { path: PathBuf, source: std::io::Error },
    /// Loader already initialised (programmer error — init called twice).
    AlreadyInitialised,
    /// notify watcher setup failed.
    WatcherSetup(notify::Error),
}

#[derive(Debug, Clone)]
pub struct FileFailure {
    pub path: PathBuf,
    pub model: Option<String>,        // None if the failure is structural (not per-model)
    pub provider: Option<String>,
    pub errors: Vec<String>,           // human-readable error messages
}

/// Opaque handle that keeps the watcher alive. Drop = stop watching.
pub struct CostTableHandle {
    _watcher: notify::RecommendedWatcher,
}

/// Internal: shape of the YAML before validation.
#[derive(Debug, Deserialize)]
struct RawCostTable {
    version: u32,
    last_updated: NaiveDate,
    #[serde(default)]
    source: String,
    rates: HashMap<String, HashMap<String, RawCostRate>>,   // provider → model → rate
}

#[derive(Debug, Deserialize)]
struct RawCostRate {
    input_per_1k_usd: Decimal,
    output_per_1k_usd: Decimal,
    #[serde(default)]
    is_embedding: bool,
}
```

### YAML schema (full, with all slice-2 seed entries)

```yaml
# services/ai-gateway/config/cost_rates.yaml
version: 1
last_updated: 2026-05-15
source: |
  anthropic.com/pricing (Anthropic native)
  aws.amazon.com/bedrock/pricing (AWS Bedrock)
  openai.com/pricing (OpenAI)

rates:
  bedrock:
    anthropic.claude-3-5-sonnet-20241022-v2:0:
      input_per_1k_usd:  0.003
      output_per_1k_usd: 0.015
    anthropic.claude-3-haiku-20240307-v1:0:
      input_per_1k_usd:  0.00025
      output_per_1k_usd: 0.00125
    amazon.titan-embed-text-v2:0:
      input_per_1k_usd:  0.00002
      output_per_1k_usd: 0.0
      is_embedding: true

  anthropic:
    claude-3-5-sonnet-20241022:
      input_per_1k_usd:  0.003
      output_per_1k_usd: 0.015
    claude-3-haiku-20240307:
      input_per_1k_usd:  0.00025
      output_per_1k_usd: 0.00125

  openai:
    gpt-4o:
      input_per_1k_usd:  0.0025
      output_per_1k_usd: 0.01
    gpt-4o-mini:
      input_per_1k_usd:  0.00015
      output_per_1k_usd: 0.0006
    text-embedding-3-small:
      input_per_1k_usd:  0.00002
      output_per_1k_usd: 0.0
      is_embedding: true

  bge:                                          # self-hosted (FR-AI-019) — cost = 0
    bge-m3:
      input_per_1k_usd:  0.0
      output_per_1k_usd: 0.0
      is_embedding: true
    bge-reranker-v2-m3:
      input_per_1k_usd:  0.0
      output_per_1k_usd: 0.0
      is_embedding: true                        # rerank is "embedding-class" for accounting
```

---

## §4 — Acceptance criteria (testable, ordered, numbered)

1. **Happy lookup** — Given `cost_rates.yaml` loaded with the §3 seed, `cost_table::lookup(&ProviderKind::Bedrock, "anthropic.claude-3-5-sonnet-20241022-v2:0")` MUST return `Some(CostRate { input_per_1k_usd: dec!(0.003), output_per_1k_usd: dec!(0.015), is_embedding: false })`.
2. **Miss returns None** — `lookup(&ProviderKind::Anthropic, "nonexistent-model")` MUST return `None`. No panic; no error log.
3. **Embedding flag set** — `lookup(&ProviderKind::Bedrock, "amazon.titan-embed-text-v2:0")` MUST return `CostRate { is_embedding: true, output_per_1k_usd: dec!(0.0), .. }`.
4. **Self-hosted BGE rates are 0** — `lookup(&ProviderKind::Bge, "bge-m3")` MUST return `CostRate { input_per_1k_usd: dec!(0.0), output_per_1k_usd: dec!(0.0), is_embedding: true }`.
5. **Aggregate failures on init** — Given a YAML with 3 entries (one negative rate, one missing `output_per_1k_usd`, one with unknown provider name), `init_cost_table` MUST return `Err(LoaderInitError::Schema { failures })` with `failures.len() == 3`. Each `FileFailure` MUST identify the model + provider + specific error message.
6. **Negative rate rejected** — `input_per_1k_usd: -0.01` rejected with `FileFailure { errors: vec!["rate must be non-negative, got -0.01"] }`.
7. **Lookup latency < 1µs** — Benchmark 100,000 lookups; total elapsed < 100ms (mean < 1µs). Measured via `cargo bench` Criterion benchmark.
8. **Concurrent reads zero contention** — 1000 tokio tasks × 1000 lookups in <500ms total (mean 0.5µs per lookup under contention).
9. **Hot reload swaps atomically** — Modify `cost_rates.yaml` to add `claude-99-future`. Within 500ms (100ms debounce + ~50ms validation + swap), `lookup(&Anthropic, "claude-99-future")` MUST return `Some(...)`. Concurrent in-flight lookups see either old or new (no torn read).
10. **Hot-reload failure preserves table** — Modify YAML to invalid (e.g., `input_per_1k_usd: not-a-number`). Cache MUST remain at last-valid state. `ai_cost_table_reload_failures_total{reason="parse_error"}` MUST increment. `tracing::error!` event emitted naming the file + line + error.
11. **Determinism property test** — `proptest!` runs 500 random YAML fixtures (validity-mixed); each fixture's `init` MUST produce a stable `Result` across 10 reads (verified via `Debug` formatting equality).
12. **`loaded_at` populated after init** — After successful `init_cost_table`, `cost_table::loaded_at()` MUST return `Some(<DateTime within last 1s of init call>)`.
13. **OTel metrics emit correctly** — After 100 lookups (90 hits + 10 misses), `ai_cost_table_lookups_total{outcome="hit"}` MUST equal 90 and `{outcome="miss"}` equal 10. Latency histogram MUST have 100 samples.
14. **Last-updated CI lint** — A CI job (separate from this FR's tests) MUST emit a warning if `last_updated` is older than 90 days. Tested via fixture with `last_updated: 2025-01-01` → lint emits warning.
15. **is_embedding consistency** — A YAML with `is_embedding: true, output_per_1k_usd: 0.5` MUST be rejected at init with a `FileFailure { errors: vec!["is_embedding: true requires output_per_1k_usd == 0.0, got 0.5"] }`. The loader enforces §1 #12 invariant; downstream code can trust it.
16. **Failure list deterministic order (feature-request-audit skill §3.9 rule 27)** — Two `init` calls with the same malformed YAML MUST produce byte-identical `Vec<FileFailure>` sorted by `(path, model)`. The §6 skeleton's `validate_and_flatten` MUST end with `failures.sort_by(|a, b| a.path.cmp(&b.path).then(a.model.cmp(&b.model)))` so operator diffing across runs is reliable.

---

## §5 — Verification

**Integration test:** `services/ai-gateway/tests/cost_table_test.rs`

```rust
use cyberos_ai_gateway::cost_table::{self, init_cost_table, ProviderKind, LoaderInitError};
use rust_decimal_macros::dec;
use std::path::PathBuf;
use std::time::Instant;

#[tokio::test]
async fn lookup_bedrock_claude_sonnet() {
    let _handle = init_cost_table(&PathBuf::from("tests/fixtures/cost_table/valid_rates.yaml"))
        .await.unwrap();
    let rate = cost_table::lookup(&ProviderKind::Bedrock, "anthropic.claude-3-5-sonnet-20241022-v2:0").unwrap();
    assert_eq!(rate.input_per_1k_usd, dec!(0.003));
    assert_eq!(rate.output_per_1k_usd, dec!(0.015));
    assert!(!rate.is_embedding);
}

#[tokio::test]
async fn miss_returns_none() {
    let _handle = init_cost_table(&fixture("valid_rates.yaml")).await.unwrap();
    assert!(cost_table::lookup(&ProviderKind::Anthropic, "nonexistent").is_none());
}

#[tokio::test]
async fn embedding_flag_set_for_titan() {
    let _handle = init_cost_table(&fixture("valid_rates.yaml")).await.unwrap();
    let rate = cost_table::lookup(&ProviderKind::Bedrock, "amazon.titan-embed-text-v2:0").unwrap();
    assert!(rate.is_embedding);
    assert_eq!(rate.output_per_1k_usd, dec!(0.0));
}

#[tokio::test]
async fn negative_rate_rejected_at_init() {
    let err = init_cost_table(&fixture("negative_rate.yaml")).await.unwrap_err();
    match err {
        LoaderInitError::Schema { failures } => {
            assert!(failures.iter().any(|f| f.errors.iter().any(|e| e.contains("non-negative"))));
        }
        _ => panic!("expected Schema error"),
    }
}

#[tokio::test]
async fn aggregate_three_failures() {
    let err = init_cost_table(&fixture("three_failures.yaml")).await.unwrap_err();
    match err {
        LoaderInitError::Schema { failures } => assert_eq!(failures.len(), 3),
        _ => panic!("expected aggregated Schema error"),
    }
}

#[tokio::test]
async fn hot_reload_picks_up_new_model() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("cost_rates.yaml");
    std::fs::copy("tests/fixtures/cost_table/valid_rates.yaml", &path).unwrap();
    let _handle = init_cost_table(&path).await.unwrap();

    // Modify YAML to add new model
    std::fs::write(&path, /* yaml with claude-99-future added */).unwrap();

    let start = Instant::now();
    loop {
        if let Some(rate) = cost_table::lookup(&ProviderKind::Anthropic, "claude-99-future") {
            assert_eq!(rate.input_per_1k_usd, dec!(0.001));
            assert!(start.elapsed() < std::time::Duration::from_millis(500));
            return;
        }
        if start.elapsed() > std::time::Duration::from_millis(500) {
            panic!("hot reload did not pick up new model within 500ms");
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
}

#[tokio::test]
async fn hot_reload_invalid_preserves_cache() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("cost_rates.yaml");
    std::fs::copy("tests/fixtures/cost_table/valid_rates.yaml", &path).unwrap();
    let _handle = init_cost_table(&path).await.unwrap();

    // Corrupt the YAML
    std::fs::write(&path, "not: valid: yaml: at: all").unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // Cache should still serve the original valid rate
    let rate = cost_table::lookup(&ProviderKind::Bedrock, "anthropic.claude-3-5-sonnet-20241022-v2:0");
    assert!(rate.is_some());
}

// AC #11: determinism property test
proptest! {
    #[test]
    fn determinism_across_reads(yaml in any_valid_cost_yaml()) {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("cost_rates.yaml");
        std::fs::write(&path, &yaml).unwrap();
        let _handle = futures::executor::block_on(init_cost_table(&path)).unwrap();
        let snapshots: Vec<_> = (0..10).map(|_| {
            cost_table::lookup(&ProviderKind::Bedrock, "anthropic.claude-3-5-sonnet-20241022-v2:0")
        }).collect();
        prop_assert!(snapshots.windows(2).all(|w| w[0] == w[1]));
    }
}

// AC #15: is_embedding consistency
#[tokio::test]
async fn is_embedding_with_nonzero_output_rejected() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("cost_rates.yaml");
    std::fs::write(&path, r#"
version: 1
last_updated: 2026-05-15
rates:
  bedrock:
    test-model:
      input_per_1k_usd: 0.001
      output_per_1k_usd: 0.5
      is_embedding: true
"#).unwrap();
    let err = init_cost_table(&path).await.unwrap_err();
    match err {
        LoaderInitError::Schema { failures } => {
            assert!(failures.iter().any(|f| f.errors.iter().any(|e| e.contains("is_embedding: true requires"))));
        }
        _ => panic!("expected Schema error"),
    }
}

#[tokio::test]
async fn concurrent_1000_tasks_no_contention() {
    let _handle = init_cost_table(&fixture("valid_rates.yaml")).await.unwrap();
    let start = Instant::now();
    let handles: Vec<_> = (0..1000).map(|_| {
        tokio::spawn(async {
            for _ in 0..1000 {
                let _ = cost_table::lookup(&ProviderKind::Bedrock, "anthropic.claude-3-5-sonnet-20241022-v2:0");
            }
        })
    }).collect();
    futures::future::join_all(handles).await;
    assert!(start.elapsed() < std::time::Duration::from_millis(500));
}
```

**Benchmark:** `services/ai-gateway/benches/cost_table_lookup_bench.rs`

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use cyberos_ai_gateway::cost_table::{self, ProviderKind};

fn bench_lookup_hit(c: &mut Criterion) {
    // setup: init_cost_table once in test main
    c.bench_function("cost_table::lookup hit", |b| {
        b.iter(|| {
            cost_table::lookup(
                black_box(&ProviderKind::Bedrock),
                black_box("anthropic.claude-3-5-sonnet-20241022-v2:0"),
            )
        });
    });
}

fn bench_lookup_miss(c: &mut Criterion) {
    c.bench_function("cost_table::lookup miss", |b| {
        b.iter(|| cost_table::lookup(black_box(&ProviderKind::Bedrock), black_box("nonexistent")));
    });
}

criterion_group!(benches, bench_lookup_hit, bench_lookup_miss);
criterion_main!(benches);
```

Run via:

```bash
cd /Users/stephencheng/Projects/CyberSkill/cyberos
cargo test -p cyberos-ai-gateway cost_table
cargo bench -p cyberos-ai-gateway cost_table_lookup_bench
```

CI gate: tests run on every PR touching `services/ai-gateway/src/cost_table/**` or `services/ai-gateway/config/cost_rates.yaml`. Benchmark regression > 20% fails the PR.

---

## §6 — Implementation skeleton

```rust
// services/ai-gateway/src/cost_table.rs

use arc_swap::ArcSwap;
use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use chrono::{DateTime, Utc, NaiveDate};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tokio::sync::mpsc;
use notify::{RecommendedWatcher, Watcher, RecursiveMode, Event, EventKind};

use crate::cost_table::schema::{CostRate, RawCostTable, FileFailure, LoaderInitError, CostTableHandle};
use crate::policy::ProviderKind;

static TABLE: OnceCell<ArcSwap<HashMap<(ProviderKind, String), CostRate>>> = OnceCell::new();
static LOADED_AT: OnceCell<ArcSwap<Option<DateTime<Utc>>>> = OnceCell::new();

const DEBOUNCE_MS: u64 = 100;

mod metrics {
    use once_cell::sync::Lazy;
    use prometheus::{register_counter_vec, register_histogram, register_int_gauge, CounterVec, Histogram, IntGauge};

    pub static LOOKUPS: Lazy<CounterVec> = Lazy::new(|| register_counter_vec!(
        "ai_cost_table_lookups_total",
        "Cost-table lookups by provider and outcome",
        &["provider", "outcome"]
    ).unwrap());

    pub static RELOAD_FAILURES: Lazy<CounterVec> = Lazy::new(|| register_counter_vec!(
        "ai_cost_table_reload_failures_total",
        "Cost-table reload failures by reason",
        &["reason"]
    ).unwrap());

    pub static ENTRY_COUNT: Lazy<IntGauge> = Lazy::new(|| register_int_gauge!(
        "ai_cost_table_entries_total",
        "Current count of (provider, model) entries"
    ).unwrap());

    pub static LOADED_AT_TS: Lazy<IntGauge> = Lazy::new(|| register_int_gauge!(
        "ai_cost_table_loaded_at_ts",
        "UNIX timestamp of last successful load"
    ).unwrap());

    pub static LOOKUP_LATENCY: Lazy<Histogram> = Lazy::new(|| {
        prometheus::register_histogram!(
            "ai_cost_table_lookup_latency_ns",
            "Cost-table lookup latency in nanoseconds",
            vec![100.0, 500.0, 1_000.0, 5_000.0, 10_000.0]
        ).unwrap()
    });
}

pub fn lookup(provider: &ProviderKind, model: &str) -> Option<CostRate> {
    let started = Instant::now();
    let result = TABLE.get()
        .and_then(|s| s.load().get(&(*provider, model.to_string())).copied());
    let outcome = if result.is_some() { "hit" } else { "miss" };
    // ISS-003 fix: use explicit stable string conversion, NOT format!("{:?}", ...) which
    // ties OBS dashboards to Rust enum variant names. ProviderKind::as_metric_label()
    // is defined in policy/schema.rs and is rename-safe.
    metrics::LOOKUPS.with_label_values(&[provider.as_metric_label(), outcome]).inc();
    metrics::LOOKUP_LATENCY.observe(started.elapsed().as_nanos() as f64);
    result
}

pub fn loaded_at() -> Option<DateTime<Utc>> {
    LOADED_AT.get().and_then(|s| **s.load())
}

pub fn entry_count() -> usize {
    TABLE.get().map(|s| s.load().len()).unwrap_or(0)
}

pub async fn init_cost_table(config_path: &Path) -> Result<CostTableHandle, LoaderInitError> {
    let table = load_and_validate(config_path).await?;
    let entry_count = table.len();
    TABLE.set(ArcSwap::from_pointee(table)).map_err(|_| LoaderInitError::AlreadyInitialised)?;
    LOADED_AT.set(ArcSwap::from_pointee(Some(Utc::now()))).map_err(|_| LoaderInitError::AlreadyInitialised)?;
    metrics::ENTRY_COUNT.set(entry_count as i64);
    metrics::LOADED_AT_TS.set(Utc::now().timestamp());

    let watcher = spawn_watcher(config_path).await?;
    Ok(CostTableHandle { _watcher: watcher })
}

async fn load_and_validate(path: &Path) -> Result<HashMap<(ProviderKind, String), CostRate>, LoaderInitError> {
    let yaml = std::fs::read_to_string(path).map_err(|source| LoaderInitError::IoError {
        path: path.to_path_buf(), source,
    })?;
    let raw: RawCostTable = serde_yaml::from_str(&yaml).map_err(|e| LoaderInitError::Schema {
        failures: vec![FileFailure {
            path: path.to_path_buf(), model: None, provider: None,
            errors: vec![format!("yaml parse: {}", e)],
        }],
    })?;
    validate_and_flatten(raw, path)
}

fn validate_and_flatten(raw: RawCostTable, path: &Path) -> Result<HashMap<(ProviderKind, String), CostRate>, LoaderInitError> {
    let mut failures: Vec<FileFailure> = Vec::new();
    let mut out = HashMap::new();
    for (provider_str, models) in raw.rates {
        let kind = match parse_provider(&provider_str) {
            Ok(k) => k,
            Err(e) => {
                failures.push(FileFailure {
                    path: path.to_path_buf(),
                    provider: Some(provider_str.clone()),
                    model: None,
                    errors: vec![e],
                });
                continue;
            }
        };
        for (model, rate) in models {
            let mut model_errors: Vec<String> = Vec::new();
            if rate.input_per_1k_usd < dec!(0) {
                model_errors.push(format!("input_per_1k_usd must be non-negative, got {}", rate.input_per_1k_usd));
            }
            if rate.output_per_1k_usd < dec!(0) {
                model_errors.push(format!("output_per_1k_usd must be non-negative, got {}", rate.output_per_1k_usd));
            }
            if model.is_empty() || model.len() > 256 {
                model_errors.push(format!("model name length must be 1..=256, got {}", model.len()));
            }
            // ISS-002 fix: §1 #12 invariant — is_embedding ⇒ output_per_1k_usd == 0.0
            if rate.is_embedding && rate.output_per_1k_usd > dec!(0) {
                model_errors.push(format!(
                    "is_embedding: true requires output_per_1k_usd == 0.0, got {}",
                    rate.output_per_1k_usd
                ));
            }
            if !model_errors.is_empty() {
                failures.push(FileFailure {
                    path: path.to_path_buf(),
                    provider: Some(provider_str.clone()),
                    model: Some(model.clone()),
                    errors: model_errors,
                });
                continue;
            }
            out.insert((kind, model), CostRate {
                input_per_1k_usd: rate.input_per_1k_usd,
                output_per_1k_usd: rate.output_per_1k_usd,
                is_embedding: rate.is_embedding,
            });
        }
    }
    if !failures.is_empty() {
        return Err(LoaderInitError::Schema { failures });
    }
    Ok(out)
}

fn parse_provider(s: &str) -> Result<ProviderKind, String> {
    match s {
        "bedrock" => Ok(ProviderKind::Bedrock),
        "anthropic" => Ok(ProviderKind::Anthropic),
        "openai" => Ok(ProviderKind::OpenAI),
        "vertex" => Ok(ProviderKind::Vertex),
        "bge" => Ok(ProviderKind::Bge),
        other => Err(format!("unknown provider '{}'; supported: bedrock|anthropic|openai|vertex|bge", other)),
    }
}

// ISS-003 fix: stable string conversion for OTel labels (defined here for slice 2;
// will move to policy/schema.rs as the canonical home).
impl ProviderKind {
    pub fn as_metric_label(&self) -> &'static str {
        match self {
            Self::Bedrock => "bedrock",
            Self::Anthropic => "anthropic",
            Self::OpenAI => "openai",
            Self::Vertex => "vertex",
            Self::Bge => "bge",
        }
    }
}

async fn spawn_watcher(path: &Path) -> Result<RecommendedWatcher, LoaderInitError> {
    let (tx, mut rx) = mpsc::channel::<Event>(16);
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
        if let Ok(ev) = res { let _ = tx.blocking_send(ev); }
    }).map_err(LoaderInitError::WatcherSetup)?;
    // ISS-004 fix: path.parent() returns Some("") for bare filenames, not None.
    let watch_dir: PathBuf = match path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
        _ => {
            tracing::warn!(?path, "cost_rates.yaml has no parent dir; watching CWD instead");
            PathBuf::from(".")
        }
    };
    watcher.watch(&watch_dir, RecursiveMode::NonRecursive)
        .map_err(LoaderInitError::WatcherSetup)?;

    let path = path.to_path_buf();
    tokio::spawn(async move {
        let mut last_event_at: Option<Instant> = None;
        loop {
            tokio::select! {
                Some(_event) = rx.recv() => {
                    last_event_at = Some(Instant::now());
                }
                _ = tokio::time::sleep(std::time::Duration::from_millis(DEBOUNCE_MS)) => {
                    if let Some(t) = last_event_at {
                        if t.elapsed() >= std::time::Duration::from_millis(DEBOUNCE_MS) {
                            last_event_at = None;
                            apply_reload(&path).await;
                        }
                    }
                }
            }
        }
    });
    Ok(watcher)
}

async fn apply_reload(path: &Path) {
    match load_and_validate(path).await {
        Ok(new_table) => {
            if let Some(s) = TABLE.get() {
                let count = new_table.len();
                s.store(Arc::new(new_table));
                metrics::ENTRY_COUNT.set(count as i64);
            }
            if let Some(s) = LOADED_AT.get() {
                s.store(Arc::new(Some(Utc::now())));
                metrics::LOADED_AT_TS.set(Utc::now().timestamp());
            }
            tracing::info!(?path, "cost_table_reloaded");
        }
        Err(e) => {
            let reason = match &e {
                LoaderInitError::Schema { .. } => "validation_error",
                LoaderInitError::IoError { .. } => "io_error",
                _ => "unknown",
            };
            metrics::RELOAD_FAILURES.with_label_values(&[reason]).inc();
            tracing::error!(?path, ?e, "cost_table_reload_failed");
        }
    }
}
```

*Scaffold is suggestive. AC §4 is the contract.*

---

## §7 — Dependencies

**Code dependencies (must exist before this FR can build):**
- None. This is foundational; FR-AI-001, FR-AI-002, FR-AI-006 all depend on it.

**Concept dependencies:**
- `ProviderKind` enum from `services/ai-gateway/src/policy/schema.rs` (FR-AI-005). The closed set `{Bedrock, Anthropic, OpenAI, Vertex, Bge}` is the same set this FR's `parse_provider` recognises.
- `Decimal` precision invariant — every cost-bearing field across AI Gateway uses `rust_decimal::Decimal` for arithmetic; FR-AI-001/002 already assume this.

**Operational dependencies:**
- `services/ai-gateway/config/cost_rates.yaml` exists on the deployed filesystem (Dockerfile copies it; k8s ConfigMap mounts it).
- Filesystem supports `notify` (inotify on Linux, FSEvents on macOS); deployment env must NOT be a network FS that lacks event support (S3FS, etc.). Same constraint as FR-AI-005.

**Crate dependencies (in `Cargo.toml`):**
- `serde`, `serde_yaml` — YAML parsing
- `rust_decimal`, `rust_decimal_macros` — fixed-point arithmetic
- `arc-swap` — lock-free atomic Arc swap
- `notify` — cross-platform file watching
- `once_cell` — OnceCell statics
- `prometheus` — OTel-compatible metrics
- `chrono` — timestamps for `loaded_at`
- `proptest` (dev-dep) — for §5 property test
- `criterion` (dev-dep) — for §5 benchmark
- `tempfile` (dev-dep) — for hot-reload tests

---

## §8 — Example payloads

### Caller in FR-AI-001 (precheck)

```rust
let cost_per_1k = cost_table::lookup(&provider, &model)
    .ok_or_else(|| PrecheckError::CostEstimateFailed { reason: "no_cost_entry".into() })?;
let estimated_usd = (Decimal::from(req.prompt_tokens) / Decimal::from(1000)) * cost_per_1k.input_per_1k_usd
                  + (Decimal::from(req.expected_completion_tokens) / Decimal::from(1000)) * cost_per_1k.output_per_1k_usd;
```

### Caller in FR-AI-006 (alias resolve)

```rust
if cost_table::lookup(&kind, model).is_none() {
    return Err(AliasError::ResolvedModelMissingCostEntry { provider: kind, model: model.to_string() });
}
```

### Caller in FR-AI-002 (reconcile — uses is_embedding to validate)

```rust
let rate = cost_table::lookup(&hold.resolved_provider, &hold.resolved_model)
    .ok_or(ReconcileError::CostTableMissing { /* ... */ })?;
if rate.is_embedding && usage.completion_tokens > 0 {
    tracing::warn!(model = %hold.resolved_model, completion_tokens = usage.completion_tokens,
        "embedding response has non-zero completion_tokens — provider bug?");
}
```

### Caller in FR-AI-021 (operator CLI `models pricing`)

```bash
$ cyberos-ai models pricing
COST TABLE — loaded at 2026-05-16T09:30:00Z (45 minutes ago)

PROVIDER     MODEL                                            INPUT/1K   OUTPUT/1K   FLAGS
bedrock      anthropic.claude-3-5-sonnet-20241022-v2:0        $0.003     $0.015
bedrock      anthropic.claude-3-haiku-20240307-v1:0           $0.00025   $0.00125
bedrock      amazon.titan-embed-text-v2:0                     $0.00002   $0.000      embedding
anthropic    claude-3-5-sonnet-20241022                       $0.003     $0.015
anthropic    claude-3-haiku-20240307                          $0.00025   $0.00125
openai       gpt-4o                                           $0.0025    $0.01
openai       gpt-4o-mini                                      $0.00015   $0.0006
openai       text-embedding-3-small                           $0.00002   $0.000      embedding
bge          bge-m3                                           $0.0       $0.0        embedding (self-hosted)
bge          bge-reranker-v2-m3                               $0.0       $0.0        embedding (self-hosted)
```

### OBS metric snapshot after running

```
# HELP ai_cost_table_lookups_total Cost-table lookups by provider and outcome
# TYPE ai_cost_table_lookups_total counter
ai_cost_table_lookups_total{provider="Bedrock",outcome="hit"} 8421
ai_cost_table_lookups_total{provider="Bedrock",outcome="miss"} 12
ai_cost_table_lookups_total{provider="Anthropic",outcome="hit"} 421

# HELP ai_cost_table_entries_total Current count of (provider, model) entries
# TYPE ai_cost_table_entries_total gauge
ai_cost_table_entries_total 10

# HELP ai_cost_table_loaded_at_ts UNIX timestamp of last successful load
# TYPE ai_cost_table_loaded_at_ts gauge
ai_cost_table_loaded_at_ts 1763199600
```

### Failure example — aggregate validation

```rust
// init_cost_table call returns:
Err(LoaderInitError::Schema {
    failures: vec![
        FileFailure {
            path: PathBuf::from("config/cost_rates.yaml"),
            provider: Some("bedrock".into()),
            model: Some("claude-broken".into()),
            errors: vec!["input_per_1k_usd must be non-negative, got -0.01".into()],
        },
        FileFailure {
            path: PathBuf::from("config/cost_rates.yaml"),
            provider: Some("typo_provider".into()),
            model: None,
            errors: vec!["unknown provider 'typo_provider'; supported: bedrock|anthropic|openai|vertex|bge".into()],
        },
        FileFailure {
            path: PathBuf::from("config/cost_rates.yaml"),
            provider: Some("openai".into()),
            model: Some("".into()),
            errors: vec!["model name length must be 1..=256, got 0".into()],
        },
    ],
})
```

---

## §9 — Open questions

All resolved at authoring time. No deferred questions.

For reference, the questions considered + resolved during authoring:

1. **~~Should the cost table support multi-currency?~~** — RESOLVED: USD only (§1 #9 + §2 rationale). FX is FR-INV-002's job.
2. **~~Should we use f64 or Decimal?~~** — RESOLVED: Decimal (§1 #10 + §2). Money requires exact arithmetic.
3. **~~What's the debounce on hot-reload?~~** — RESOLVED: 100ms (§1 #5 + §2 rationale). Avoids editor-burst thrash.
4. **~~Should hot-reload failure be fatal?~~** — RESOLVED: no, preserve last-known-good (§1 #6 + §2). Asymmetric trade-off: startup fails loud, runtime fails soft.
5. **~~Should we have is_embedding metadata?~~** — RESOLVED: yes (§1 #12 + §2). Helps FR-AI-002 validate provider responses.

---

## §10 — Failure modes inventory (all error paths)

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| `cost_rates.yaml` missing at startup | `fs::read_to_string` `NotFound` | `LoaderInitError::IoError`; gateway exits 1 | Operator creates file; redeploy |
| YAML syntax error at startup | `serde_yaml::from_str` error | `LoaderInitError::Schema` with one failure | Operator fixes YAML; redeploy |
| Negative rate at startup | Validation step | `LoaderInitError::Schema` aggregating ALL failures | Operator corrects all flagged rates |
| Unknown provider name at startup | `parse_provider` returns Err | Aggregated in Schema failures | Operator uses correct provider key |
| Empty model name at startup | Length check | Aggregated in Schema failures | Operator removes empty entries |
| Lookup for missing (provider, model) | `HashMap::get` returns None | `lookup` returns `None`; `ai_cost_table_lookups_total{outcome="miss"}++` | Caller translates to its own error type |
| Hot-reload YAML invalid | `notify` event + re-validation fails | Cache preserved; `reload_failures_total++`; sev-2 OBS event | Operator fixes YAML; next event re-tries |
| Hot-reload IO error (file deleted) | `read_to_string` error | Cache preserved; warn log | Operator restores file |
| Concurrent lookup + hot-reload | `ArcSwap.load()` atomic snapshot | Reader gets old or new; never torn | No-op; by design |
| Filesystem doesn't support notify (NFS) | `notify` falls back to polling | Hot-reload latency widens to ~30s; warn log on init | Document the constraint; operator chooses inotify-capable storage |
| `last_updated` stale (>90 days) | CI lint | Warning on PR; non-blocking | Operator refreshes prices in next sprint |
| Loader called twice (programmer error) | `OnceCell::set` returns Err | `LoaderInitError::AlreadyInitialised` | Engineer fixes startup ordering |
| Watcher init on bare filename | `path.parent()` returns Some("") | Falls back to watching CWD; emits `tracing::warn!` | Operator passes a fully-qualified path on next deploy |
| `is_embedding: true` with non-zero output rate | Cross-field validation in `validate_and_flatten` | Aggregated in Schema failures with explicit "is_embedding requires output == 0" message | Operator fixes YAML inconsistency |

---

## §11 — Notes (informational, no normative force)

- Per provider's published pricing, rates change roughly monthly. The YAML's `last_updated` field is human-maintained; a CI lint flags entries older than 90 days for review. The lint is non-blocking — staleness is a warning, not a deploy gate.
- The 6-decimal precision (`NUMERIC(12,6)`) accommodates `gpt-4o-mini`'s `$0.00015/1k` and `text-embedding-3-small`'s `$0.00002/1k` without precision loss. If a future provider publishes 8-decimal rates, the schema needs to widen — but unlikely in slice 2's horizon.
- Multi-currency support is intentionally NOT in this FR. The invoicing module (FR-INV-002, P2) handles VND/USD/SGD/EUR conversion using SBV's daily FX rate. Cost-ledger comparisons happen in USD throughout.
- The `is_embedding` flag is the only schema field with no provider-published equivalent — we set it manually in the YAML based on operational knowledge of which model produces vectors. If an operator forgets to set it on a new embedding model, FR-AI-002's reconcile path emits a warn log but doesn't crash; the embedding cost would still be `0.0` because `output_per_1k_usd: 0.0` in the YAML.
- The watcher's 100ms debounce can be tuned via env var `CYBEROS_AI_COST_TABLE_DEBOUNCE_MS` (default 100, min 10, max 5000). Most operators never touch it.
- Cost-table reload doesn't invalidate in-flight precheck/reconcile transactions; each one captures the rate at its own time. If a price changes mid-call (rare), the call uses the rate that was active when precheck ran — not the rate at reconcile. This is intentional: estimate-and-reconcile arithmetic must use the same rate for consistency.
- This FR is the simplest "config-loader" pattern in slice 2. The same pattern repeats in FR-AI-005 (tenant policy), FR-AI-015 (ZDR attestations). All three share idioms (ArcSwap, notify, aggregate failures); a shared crate is a future refactor (slice 4) once we have 3+ instances to extract from.
- **Decimal vs BIGINT minor (feature-request-audit skill §3.4 rule 11) — boundary clarification:** rates here use `rust_decimal::Decimal` for `input_per_1k_usd` / `output_per_1k_usd` because rates are **constants** with 7–9 decimal places (e.g. `$0.00015/1k`), not stored monetary state. feature-request-audit skill §3.4 rule 11 "money MUST be stored as BIGINT minor" applies at the **storage tier** — i.e. `cost_ledger.estimated_cost_minor` and `actual_cost_minor` (FR-AI-001 §3). The product `rate × tokens` IS BIGINT minor when stored. The cross-boundary conversion happens in `cost_ledger::estimate_cost_minor`, which multiplies the Decimal rate by token count and rounds to the currency-decimal-aware minor unit via `Currency::USD.decimals()`. Future maintainers: do not "fix" rates to BIGINT minor — sub-cent precision matters for embedding pricing.

---

*End of FR-AI-007. Status: draft (10/10 target, expanded from compressed first-pass per workflow correction 2026-05-15). Run `feature-request-audit` next.*
