---
fr_id: FR-AI-007
audited: 2026-05-16
auditor: manual (engineering-spec template)
verdict: PASS
score_pre_revision: 8.0/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_open: 0
issues_resolved: 6
issues_critical: 0
template: engineering-spec@1
revised_at: 2026-05-16
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 ISSes)
---

## §1 — Verdict summary

FR-AI-007 was expanded from 289 lines (compressed first-pass) to ~720 lines matching FR-AI-001 depth. The expansion added 6 §1 clauses (#10-#15: Decimal precision, is_embedding flag, OTel metrics, loaded_at accessor, no-DB-persistence), 4 §2 paragraphs (Decimal-vs-f64, debounce rationale, asymmetric reload failure, is_embedding origin), full schema types in §3 (CostTableHandle, FileFailure, RawCostTable), 6 additional §4 ACs (embedding flag, BGE rates = 0, aggregate failures, latency budget, concurrent reads, determinism, OBS metrics, last-updated lint), full Rust integration test bodies in §5 (8 test cases), expanded ~200-line §6 skeleton with metrics module + watcher with debounce loop, code/concept/operational deps in §7, 5 example payloads in §8, 12 failure modes in §10, 7 notes in §11.

Four residual issues prevent 10/10.

## §2 — Findings

### ISS-001 — AC #11 promises a proptest that §5 doesn't show
- **severity:** error
- **rule_id:** test-coverage
- **location:** §4 AC #11, §5 (verification)
- **status:** open

#### Description
§4 AC #11 says: *"Determinism property test — `proptest!` runs 500 random YAML fixtures (validity-mixed); each fixture's `init` MUST produce a stable `Result` across 10 reads."* But §5 only shows 8 named `#[tokio::test]` bodies and a benchmark. No proptest code. A code-gen agent reading the FR would skip the AC because there's no example to crib from.

#### Suggested fix
Add the proptest body to §5:

```rust
proptest! {
    #[test]
    fn determinism_across_reads(yaml in any_valid_cost_yaml()) {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("cost_rates.yaml");
        std::fs::write(&path, &yaml).unwrap();
        let _handle = futures::executor::block_on(init_cost_table(&path)).unwrap();
        let mut snapshots: Vec<_> = (0..10).map(|_| {
            cost_table::lookup(&ProviderKind::Bedrock, "anthropic.claude-3-5-sonnet-20241022-v2:0")
        }).collect();
        prop_assert!(snapshots.windows(2).all(|w| w[0] == w[1]));
    }
}
```

### ISS-002 — §1 #12 invariant (is_embedding ⇒ output=0) not enforced
- **severity:** error
- **rule_id:** correctness
- **location:** §1 #12, §6 skeleton (`validate_and_flatten`)
- **status:** open

#### Description
§1 #12 declares `is_embedding: true` semantically means `output_per_1k_usd == 0.0` ("embeddings have no output tokens by definition"). But the loader's `validate_and_flatten` doesn't enforce this — a malformed YAML with `is_embedding: true, output_per_1k_usd: 0.5` would load successfully. Downstream code (FR-AI-002 reconcile) treats `is_embedding` as informational; nothing catches the inconsistency.

#### Suggested fix
Add the cross-field check to `validate_and_flatten` in §6:

```rust
if rate.is_embedding && rate.output_per_1k_usd > dec!(0) {
    model_errors.push(format!(
        "is_embedding: true requires output_per_1k_usd == 0.0, got {}",
        rate.output_per_1k_usd
    ));
}
```

And add §4 AC #15: "**is_embedding consistency** — A YAML with `is_embedding: true, output_per_1k_usd: 0.5` MUST be rejected at init with a FileFailure naming the inconsistency."

### ISS-003 — Metric labels use `format!("{:?}", ProviderKind)` (fragile)
- **severity:** warning
- **rule_id:** observability-stability
- **location:** §6 skeleton, `metrics::LOOKUPS.with_label_values(&[&format!("{:?}", provider), ...])`
- **status:** open

#### Description
Using `format!("{:?}", provider)` for the label produces `"Bedrock"`, `"Anthropic"`, etc. — the Rust-Debug representation of the enum variant. This couples the OBS dashboard's label values to the Rust enum variant names. If a future refactor renames `ProviderKind::Bedrock` to `ProviderKind::AwsBedrock`, every Grafana dashboard breaks silently.

#### Suggested fix
Add an explicit string-conversion method on `ProviderKind` (in `policy/schema.rs`, but referenced here):

```rust
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
```

Then in the skeleton: `metrics::LOOKUPS.with_label_values(&[provider.as_metric_label(), outcome]).inc()`. This makes the label set explicit and rename-safe.

### ISS-004 — `spawn_watcher` parent() edge case for bare filenames
- **severity:** warning
- **rule_id:** robustness
- **location:** §6 skeleton, `watcher.watch(path.parent().unwrap_or(Path::new(".")), ...)`
- **status:** open

#### Description
If `init_cost_table` is called with a bare filename like `"cost_rates.yaml"` (no parent path), `path.parent()` returns `Some("")` (empty string, NOT None). The `unwrap_or` fallback is never triggered. Then `watcher.watch("")` likely fails with a path error. The fallback to `"."` doesn't work as intended.

#### Suggested fix
Replace with explicit logic:

```rust
let watch_dir = match path.parent() {
    Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
    _ => Path::new(".").to_path_buf(),
};
watcher.watch(&watch_dir, RecursiveMode::NonRecursive)?;
```

Also add §10 row: *"Watcher init on bare-filename path → watches CWD instead; functional but logs a warn if CWD wasn't the intended directory."*

### ISS-005 — feature-request-audit skill §3.4 rule 11 (money as BIGINT minor) — spec uses `rust_decimal::Decimal` not BIGINT minor
- **severity:** info (compliance check, exception justified)
- **rule_id:** authoring-md-§3.4 (rule 11)
- **location:** §1 (cost-table representation), §3 (CostRate schema)
- **status:** open

#### Description
feature-request-audit skill §3.4 rule 11 says "Money MUST be stored as `BIGINT minor` with currency-aware decimals" for **stored** monetary state — i.e., journal rows, ledger entries, billable amounts. The cost-table here uses `rust_decimal::Decimal` for `input_per_1k_usd` / `output_per_1k_usd`. This is intentional and correct: cost-table rates are **rate constants** (price per 1000 tokens, often 7-9 decimal places), not stored monetary amounts. The output of `cost_table::lookup × tokens` IS a money amount and DOES get stored as BIGINT minor in FR-AI-001/002's cost_ledger table. So the rule is observed at the right boundary — but the FR doesn't say so. A future reader could mistakenly "fix" this to BIGINT minor and break sub-cent rates.

#### Suggested fix
Add §11 note: "Rates use `rust_decimal::Decimal` not BIGINT minor — rates are constants (7-9 dp), not stored money. The product rate×tokens is stored as BIGINT minor by FR-AI-001 §3 `cost_ledger.estimated_cost_minor`. feature-request-audit skill §3.4 rule 11 applies to the storage tier, satisfied at the cost_ledger boundary." Also cross-link from §2.

### ISS-006 — feature-request-audit skill §3.9 rule 27 (determinism) only partially covered by AC #11 proptest
- **severity:** warning
- **rule_id:** authoring-md-§3.9 (rule 27)
- **location:** §4 AC #11, §5 verification, §6 (`validate_and_flatten`)
- **status:** open

#### Description
Determinism per feature-request-audit skill §3.9 is "two consecutive runs on the same input MUST produce byte-identical output." The proptest in §5 (added by ISS-001) covers `init` → `lookup` determinism for one provider/model. But the loader also produces a sorted internal HashMap that gets iterated for the OBS `cost_table_loaded_at_seconds` metric. HashMap iteration order is non-deterministic across runs, which CAN affect the order of warn-logs emitted during load (the file-failures sort). The `FileFailure` list's display order matters for operator diffing.

#### Suggested fix
Add explicit sort to `validate_and_flatten` output: `failures.sort_by(|a, b| a.path.cmp(&b.path).then(a.model.cmp(&b.model)))`. Add §4 AC #16: "**Failure list deterministic order** — Two `init` calls with the same malformed YAML MUST produce byte-identical `Vec<FileFailure>` (sorted by `(path, model)`)."

## §3 — Strengths preserved through expansion

- §3 schema includes `is_embedding` flag with rationale in §2 — a thoughtful addition over the original.
- §6 skeleton's metrics module preamble (added per FR-AI-006 ISS-003 pattern) is the right level of detail.
- §7 broken into code/concept/operational deps is more useful than a flat list.
- §10 failure-modes inventory covers 12 distinct paths including the NFS polling fallback row.

## §4 — Resolution

All 6 mechanical revisions applied:
- ISS-001 RESOLVED (2026-05-16): §5 now includes the `determinism_across_reads` proptest body matching AC #11.
- ISS-002 RESOLVED (2026-05-16): §6 `validate_and_flatten` now enforces is_embedding ⇒ output==0; §4 AC #15 added; §10 row added.
- ISS-003 RESOLVED (2026-05-16): §6 skeleton's `lookup` now calls `provider.as_metric_label()`; the impl is defined inline with explanatory comment.
- ISS-004 RESOLVED (2026-05-16): §6 `spawn_watcher` handles the bare-filename edge case with explicit match + warn log; §10 row added.
- ISS-005 RESOLVED (2026-05-16, feature-request-audit skill compliance pass): §11 note added explaining Decimal-vs-BIGINT boundary; rule 11 applies at FR-AI-001 storage tier, satisfied there; §2 cross-link added.
- ISS-006 RESOLVED (2026-05-16, feature-request-audit skill compliance pass): `failures.sort_by(...)` added in §6; §4 AC #16 added asserting byte-identical Vec<FileFailure> across runs.

**Score = 10/10.** Ship as-is. Ready to transition `draft → accepted`.

---

*End of FR-AI-007 audit (final). Status: PASS at 10/10.*
