---
task_id: TASK-AI-006
audited: 2026-05-16
auditor: manual (engineering-spec template)
verdict: PASS
score_pre_revision: 8.5/10
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

TASK-AI-006 was expanded from 366 lines (first-pass compressed) to ~720 lines matching TASK-AI-001 depth. The expansion added 5 §1 clauses (#11-#15 on concurrency, determinism, TASK-AI-001 integration, full metrics), 3 §2 paragraphs (closed set rationale, lock-in math, LiteLLM contrast), full Provider trait surface in §3, 6 additional §4 ACs (override-paths-also-checked, determinism property test, concurrent safety), full benchmark in §5, ~80 more skeleton lines in §6 with helper functions, more failure-modes rows in §10, more informational notes in §11.

Four residual issues prevent 10/10. All mechanical fixes.

## §2 — Findings

### ISS-001 — `is_zdr` field source inconsistency in §6 skeleton
- **severity:** error
- **rule_id:** correctness
- **location:** §6 skeleton, `check_and_build_with_model` function
- **status:** open

#### Description
The §6 skeleton checks ZDR via `zdr::is_zdr(&kind, model)` (correct — TASK-AI-015 attestation table is the source of truth). But when constructing the returned `ResolvedModel`, the code uses `provider.is_zdr()` — the Provider trait method. The two can disagree: the policy YAML might claim a provider is ZDR (per TASK-AI-005's schema) but TASK-AI-015's attestation table says otherwise. The gate uses one source; the audit field uses another. They MUST be the same source.

#### Suggested fix
Change `is_zdr: provider.is_zdr()` to `is_zdr: zdr::is_zdr(&kind, model)` in the `Ok(ResolvedModel { ... })` constructor. This unifies the source: TASK-AI-015 is canonical for ZDR everywhere.

Also clarify in §3 — drop `is_zdr()` from the Provider trait surface since it's never the right answer. Replace with `is_zdr_attested(model)` if a per-model attestation per-provider is needed, OR delete the method entirely and have callers ask `zdr::is_zdr` directly. Prefer deletion: one source.

### ISS-002 — AC #5/#6/#7 don't cover override path; AC #9/#10 mention override but not residency
- **severity:** warning
- **rule_id:** test-coverage
- **location:** §4 ACs
- **status:** open

#### Description
ACs #5 (ZDR violation), #6 (residency violation), #7 (cost-table missing) describe the primary-path failures. AC #9 covers "override misses cost-table"; AC #10 covers "override misses ZDR". But there's no explicit AC for "override misses residency" — leaves a gap that "override fails when policy.residency forbids the override target's region" might be implemented inconsistently.

#### Suggested fix
Add AC #10a (or renumber): "**Override misses residency** — Given override target's region doesn't match `policy.residency`, `resolve` MUST return `Err(ResidencyViolation)` from the override path. Same terminal semantics as override-misses-cost / override-misses-ZDR — no fallthrough."

### ISS-003 — §6 skeleton references metrics objects without definition
- **severity:** warning
- **rule_id:** completeness
- **location:** §6 skeleton — uses `metrics::ALIAS_RESOLUTIONS`, `metrics::ALIAS_FAILURES`, `metrics::ALIAS_LATENCY_NS`
- **status:** open

#### Description
The skeleton references three metric handles without showing where they're defined. A code-gen agent will guess; different guesses produce subtly different label sets. Should declare them inline (Lazy<CounterVec> + Lazy<HistogramVec>).

#### Suggested fix
Add a metrics module preamble to §6:

```rust
mod metrics {
    use once_cell::sync::Lazy;
    use prometheus::{register_counter_vec, register_histogram, CounterVec, Histogram};

    pub static ALIAS_RESOLUTIONS: Lazy<CounterVec> = Lazy::new(|| register_counter_vec!(
        "ai_alias_resolutions_total",
        "Successful alias resolutions",
        &["alias", "resolved_provider", "fallback_position"]
    ).unwrap());

    pub static ALIAS_FAILURES: Lazy<CounterVec> = Lazy::new(|| register_counter_vec!(
        "ai_alias_resolution_failures_total",
        "Failed alias resolutions",
        &["alias", "reason"]
    ).unwrap());

    pub static ALIAS_LATENCY_NS: Lazy<Histogram> = Lazy::new(|| register_histogram!(
        "ai_alias_resolution_latency_ns",
        "Latency of alias::resolve calls",
        vec![100.0, 500.0, 1_000.0, 5_000.0, 10_000.0]
    ).unwrap());
}
```

### ISS-004 — §8 missing residency-violation example payload
- **severity:** info
- **rule_id:** documentation-completeness
- **location:** §8 example payloads
- **status:** open

#### Description
§8 shows happy-path, fallback, override, unknown-alias error, ZDR error, memory audit row, and OBS metric snapshot. Missing: residency-violation error payload. Operators triaging "why was my call refused?" benefit from seeing every error shape.

#### Suggested fix
Add to §8:

```rust
// Error — residency violation
Err(AliasError::ResidencyViolation {
    resolved_region: Some("us-east-1".to_string()),
    policy_residency: Residency::Sg1,
})
```

### ISS-005 — task-audit skill §3.7 trace propagation not asserted on alias::resolve callers
- **severity:** warning
- **rule_id:** authoring-md-§3.7 (rule 22 — outbound calls MUST carry W3C `traceparent`)
- **location:** §1 #13 (called from `cost_ledger::precheck`), §3 (Provider trait surface)
- **status:** open

#### Description
`alias::resolve` itself is in-process and trace-free (correct). But the §1 #13 contract says it's called from `cost_ledger::precheck` (TASK-AI-001), which IS a trust-boundary entry point. The spec does not explicitly require that the precheck propagate `traceparent` into the `ResolvedModel` flow so the downstream `Provider::complete()` call carries the inbound trace context. Without an explicit clause, two implementers can disagree about whether `alias::resolve` should take a `&SpanContext` parameter for instrumentation purposes (per task-audit skill §3.7 rule 22). The audit row contract (§1 #14 metrics) references `trace_id` via OBS dashboards but no §1 clause asserts the propagation contract.

#### Suggested fix
Add §1 #16: "**MUST** be callable without trace-context parameters (it has no I/O), AND its return value `ResolvedModel` MUST be carried through to `Provider::complete()` so that the inbound `traceparent` (read in `cost_ledger::precheck`) propagates downstream. The spec at TASK-AI-022 §1 #3 owns the trace-propagation contract; this task's responsibility is to not strip it." Also add cross-ref to TASK-AI-022 in §7.

### ISS-006 — §10 failure-modes inventory lists 12 paths but no Recovery column for ZdrViolation or ResidencyViolation
- **severity:** warning
- **rule_id:** authoring-md-§3.10 (rule 29 — at least one failure-mode row per architectural decision)
- **location:** §10 failure-modes inventory
- **status:** open

#### Description
The §10 table covers the failures (Failure | Detection | Outcome columns) but the Recovery column is sparse for the policy-violation paths. `ZdrViolation` and `ResidencyViolation` are terminal — they MUST NOT auto-fall through to the next provider (that would silently leak the violation). The current §10 row says "Outcome: error returned; call rejected" but doesn't explicitly say "Recovery: operator updates `policy.ai_policy.zdr_required` or `policy.ai_policy.residency` and reloads policy via TASK-AI-005 hot-path." Without the Recovery column populated, operators triaging a failure don't have a clear next step. Task-audit skill §3.10 rule 29 says every architectural decision needs a failure-mode row; the spirit extends to "every row needs a Recovery."

#### Suggested fix
Populate Recovery for every §10 row. Pattern: `ZdrViolation` → "Operator audits whether ZDR truly required; updates `policy.ai_policy.zdr_required` if false, OR adds a ZDR-attested provider to fallback chain. Reload via `cyberos-ai policy reload`." Same shape for ResidencyViolation, CostEntryMissing (point at cost-table reload), UnknownAlias (point at slice-3 alias closed-set extension RFC).

## §3 — Strengths preserved through expansion

- §2 rationale density is the right level for an experienced Rust engineer reviewing a new dep.
- §4 ACs include latency benchmark + property test for determinism — gold-standard test coverage.
- §6 skeleton's helper-function decomposition (`check_and_build` vs `check_and_build_with_model` vs `latency_class_for_alias`) makes the resolve function readable.
- §10 failure-modes inventory covers 12 distinct paths.

## §4 — Resolution

All 6 mechanical revisions applied:
- ISS-001 RESOLVED (2026-05-16): §6 skeleton now uses `zdr::is_zdr(&kind, model)` for both gate + audit field; §3 Provider trait surface drops `is_zdr` with explanatory comment.
- ISS-002 RESOLVED (2026-05-16): §4 AC #10a added covering override-misses-residency.
- ISS-003 RESOLVED (2026-05-16): §6 skeleton now has full `mod metrics { ... }` preamble with Lazy<CounterVec> + Lazy<Histogram> definitions.
- ISS-004 RESOLVED (2026-05-16): §8 now includes residency-violation + cost-table-missing example payloads.
- ISS-005 RESOLVED (2026-05-16, task-audit skill compliance pass): §1 #16 added asserting trace-propagation contract (callable without span, but ResolvedModel MUST be carried through so traceparent propagates per TASK-AI-022); §7 now references TASK-AI-022.
- ISS-006 RESOLVED (2026-05-16, task-audit skill compliance pass): §10 failure-modes table Recovery column populated for every row (ZdrViolation, ResidencyViolation, CostEntryMissing, UnknownAlias, etc.) with concrete operator next-steps.

**Score = 10/10.** Ship as-is. Ready to transition `draft → accepted`.

---

*End of TASK-AI-006 audit (final). Status: PASS at 10/10.*
