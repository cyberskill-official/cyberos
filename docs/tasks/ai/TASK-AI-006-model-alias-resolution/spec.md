---
# ───── Machine-readable frontmatter (parsed by task-audit + future fr-catalog renderer) ─────
id: TASK-AI-006
title: "Model-alias resolution (chat.smart → bedrock:claude-3.5-sonnet) with per-tenant override"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-05-15T00:00:00+07:00
department: engineering
author: @stephencheng
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
shipped: null
memory_chain_hash: null
related_tasks: [TASK-AI-005, TASK-AI-007, TASK-AI-008, TASK-AI-015, TASK-AI-016, TASK-AI-022]
depends_on: [TASK-AI-005, TASK-AI-007, TASK-AUTH-004]
blocks: [TASK-AI-008, TASK-AI-009, TASK-AI-015, TASK-AI-016]

# ───── Source contracts (where the spec authority lives) ─────
source_pages:
  - website/docs/modules/ai.html#multi-provider
  - website/docs/modules/ai.html#cost-gate
source_decisions:
  - docs/tasks/ai/TASK-AI-005-tenant-policy-yaml-loader/spec.md §3 (Provider enum + model_alias_map)
  - docs/tasks/ai/TASK-AI-007-provider-cost-table-loader/spec.md §1 (cost entry must exist)
  - archive/2026-05-14/AUDIT_AND_PLAN.md §3.3 (P0 · slice 2 build placement)

# ───── Build envelope (read by AI agent before code-gen) ─────
language: rust 1.81
service: cyberos/services/ai-gateway/
new_files:
  - services/ai-gateway/src/alias.rs
  - services/ai-gateway/src/alias/registry.rs
  - services/ai-gateway/src/alias/types.rs
  - services/ai-gateway/tests/alias_resolution_test.rs
  - services/ai-gateway/tests/fixtures/alias/policy_bedrock_primary.yaml
  - services/ai-gateway/tests/fixtures/alias/policy_with_override.yaml
modified_files:
  - services/ai-gateway/src/cost_ledger.rs   # use alias::resolve() instead of inline logic
  - services/ai-gateway/src/lib.rs           # export alias module
  - services/ai-gateway/src/policy/schema.rs # add AliasOverrides field if not in TASK-AI-005
allowed_tools:
  - file_read: services/ai-gateway/**
  - file_write: services/ai-gateway/{src,tests}/**
  - bash: cargo test -p cyberos-ai-gateway alias
  - bash: cargo bench --bench alias_resolution_bench
disallowed_tools:
  - hardcode any provider name outside the registry (no `if provider == "bedrock"` checks anywhere)
  - bypass policy.ai_policy.primary_provider when resolving an alias
  - call cost_table::lookup or zdr::is_zdr from anywhere except alias::resolve in the hot path
  - mutate policy from inside resolve() (read-only contract)

# ───── Estimated work (for human triage + scheduling) ─────
effort_hours: 6
subtasks:
  - "0.5h: ModelAlias enum + SUPPORTED_ALIASES const (chat.smart, chat.fast, chat.long, embed.standard, embed.code, rerank.fast)"
  - "1.0h: resolve() function — consume TenantPolicy.primary_provider + fallback_chain"
  - "1.0h: per-tenant override path (TenantPolicy.alias_overrides)"
  - "0.5h: AliasError enum + variants per failure mode"
  - "1.0h: ResolvedModel struct + LatencyClass enum + helper functions"
  - "1.0h: integration with cost_ledger::precheck (replace inline alias resolution)"
  - "1.5h: integration tests (12 cases — primary, fallback, override, missing alias, missing provider, ZDR, residency, cost-table miss, etc.)"
risk_if_skipped: "Every consumer module (CUO, KB, CHAT, OBS auto-triage) hard-codes (provider, model) tuples. Switching providers becomes a multi-PR change across 22 modules instead of a one-line YAML edit per tenant. TASK-AI-008 router cannot work because there's no abstraction layer to redirect to fallback. The cost-of-everything gate becomes per-(provider+model) instead of per-tenant. ZDR + residency enforcement (TASK-AI-015 / TASK-AI-016) cannot land cleanly because each consumer would need to re-implement the checks. Vendor lock-in returns by the back door."
---

## §1 — Description (BCP-14 normative)

The AI Gateway service **MUST** expose `alias::resolve(alias, policy) -> Result<ResolvedModel, AliasError>` that maps a closed-set logical alias (`chat.smart`, `chat.fast`, etc.) to a concrete `(provider, model)` tuple via the tenant's `TenantPolicy.ai_policy`. Given a string alias and a reference to the active tenant policy, the function:

1. **MUST** accept exactly the 6 aliases in the slice-2 closed set: `chat.smart`, `chat.fast`, `chat.long`, `embed.standard`, `embed.code`, `rerank.fast`. Unknown aliases MUST return `Err(AliasError::UnknownAlias { alias, supported })` where `supported` is the full enumeration. The closed-set check is the first operation in `resolve` (cheap reject path).
2. **MUST** consult `policy.ai_policy.alias_overrides` FIRST, before primary/fallback. An override pins an alias to a specific `(provider, model)` regardless of the primary provider — useful for "for this tenant, `chat.long` MUST go to Anthropic native, not Bedrock." The override path bypasses fallback chain entirely.
3. **MUST** prefer the policy's `primary_provider.model_alias_map` lookup if no override exists. If `model_alias_map[alias]` is present, return that model under the primary provider with `fallback_position: 0`.
4. **MUST** fall through to `fallback_chain` in order if the primary doesn't carry the alias. Each fallback's `model_alias_map` is consulted; the first hit wins. The fallback index (1, 2, …) is returned as `fallback_position` so callers can attribute failover.
5. **MUST** validate that the resolved `(provider, model)` exists in the cost table (TASK-AI-007). If `cost_table::lookup(&provider_kind, &model)` returns `None`, the function MUST return `Err(AliasError::ResolvedModelMissingCostEntry { provider, model })`. The cost-entry check prevents resolution-time success that would fail seconds later at the precheck cost-estimate step.
6. **MUST** validate that the resolved provider satisfies `policy.ai_policy.zdr_required`. If `zdr_required: true` and the resolved provider's `is_zdr` attestation (TASK-AI-015 source of truth) is `false`, return `Err(AliasError::ZdrViolation)`. The ZDR check runs after cost-entry validation so that a missing-cost-entry error takes precedence (more actionable).
7. **MUST** validate that the resolved provider's `region` matches `policy.ai_policy.residency`. If `residency: sg-1` but the resolved provider is `Bedrock` configured for `us-east-1`, return `Err(AliasError::ResidencyViolation)`. Providers without regional pinning (e.g., Anthropic native) pass the residency check by default unless `policy.residency_requires_regional_provider: true` (slice-2 optional flag).
8. **MUST** return `ResolvedModel { provider_kind, region, model, fallback_position, is_zdr, latency_class }` on success. Each field carries information needed by downstream code: `provider_kind` drives the `Provider` trait dispatch in TASK-AI-008; `region` is logged in audit rows; `model` is the literal string passed to the provider SDK; `fallback_position` flags "this call used a fallback" in OBS dashboards; `is_zdr` propagates to the audit row's `extra.is_zdr` field; `latency_class` informs TASK-AI-008's per-call timeout.
9. **MUST** complete synchronously in under **1ms p95** for any input. All state is in-memory (policy is `Arc<TenantPolicy>` from TASK-AI-005's cache; cost table is `ArcSwap` from TASK-AI-007's cache; ZDR table is `ArcSwap` from TASK-AI-015's cache). No I/O, no Postgres, no network.
10. **MUST NOT** mutate the policy under any code path. The contract is read-only; mutating would surprise concurrent readers (ArcSwap guarantees consistent snapshots only if no in-place writes happen).
11. **MUST** be safe under concurrent invocation from many tokio tasks. The `ArcSwap::load()` semantics guarantee each call sees a consistent policy snapshot; if TASK-AI-005 hot-reloads policy mid-call, the in-flight call uses the snapshot it loaded, and the next call sees the new policy.
12. **MUST** be deterministic — same `(alias, policy_snapshot)` always returns the same `ResolvedModel` or the same `AliasError`. Determinism is a CI gate: a property test (§4 AC #11) asserts 10,000 random runs over 100 random fixtures produce stable outputs.
13. **MUST** be called from `cost_ledger::precheck` (TASK-AI-001 §6) BEFORE the cost-estimate step. TASK-AI-001's skeleton is updated to call `alias::resolve` and pass the `ResolvedModel` to the rest of the precheck.
14. **MUST** emit OTel metrics on every call: `ai_alias_resolutions_total{alias,resolved_provider,fallback_position}` (counter), `ai_alias_resolution_failures_total{alias,reason}` (counter; reason ∈ `unknown_alias` / `cost_missing` / `zdr` / `residency` / `no_provider_has_alias`), `ai_alias_resolution_latency_ns` (histogram with buckets at 100ns / 500ns / 1µs / 5µs / 10µs).
15. **SHOULD** expose `alias::supported_aliases() -> &'static [&'static str]` so operator CLI (TASK-AI-021 `models list`) and the operator UI can enumerate the closed set without re-reading the const.
16. **MUST** be callable without trace-context parameters — it has no I/O and emits no audit row of its own — AND its return value `ResolvedModel` MUST be carried through to `Provider::complete()` so the inbound `traceparent` (read at the trust boundary in `cost_ledger::precheck`) propagates to the downstream HTTP call per task-audit skill §3.7 rule 22. TASK-AI-022 §1 #3 owns the propagation contract; this task's responsibility is to NOT strip the span context from the caller's flow. A spec violation: passing `ResolvedModel` by value to a function that then opens a new span without inheriting from the caller.

This task provides the abstraction layer that every consumer module (CUO Phase 2, KB ingest, CHAT, OBS auto-triage, PROJ Genie inline) uses to ask for an LLM call without naming a provider. The provider naming is a tenant-policy concern, not an application concern. Once TASK-AI-006 ships, the gateway has a single chokepoint for cross-cutting checks (cost-table, ZDR, residency); TASK-AI-008 routes the resolved tuple, TASK-AI-015/016 enforce policy at the point of resolution, and adding a new provider (Vertex, on-prem Llama, etc.) is a one-file additive change.

---

## §2 — Why this design (rationale for humans)

**Why a closed alias set of exactly 6?** Six aliases is the right grain at slice 2. Too few (`fast`, `smart`) loses precision — a caller writing "chat with this 100k-token document" can't tell the gateway it needs long-context behaviour. Too many (one per provider's marketing name like `claude-3-5-sonnet-20241022-v2:0`) reproduces the vendor lock-in we're trying to escape — the caller now hard-codes Anthropic naming. The 6-tier set (smart/fast/long for chat, standard/code for embed, fast for rerank) covers every reasonable LLM workload we identified across the 22 module pages. Adding a 7th requires a const edit, a task, and a schema update — that's the right level of friction.

**Why pure synchronous, no I/O?** This is on the precheck hot path of every chat call. Precheck's budget is 50ms p95 (TASK-AI-001 §1 #7). Adding a Postgres roundtrip would consume ~5ms; adding an HTTP roundtrip would consume ~20ms. Neither is acceptable when the entire alias resolution can be done in <1µs of HashMap lookups against in-memory caches. Sub-microsecond latency makes the alias layer "free" — every consumer can call it without measuring impact.

**Why override beats primary?** A tenant in `sg-1` residency might still want a specific call (one persona, one workflow) to use Anthropic native instead of Bedrock for regulatory or quality reasons — e.g., "for client-facing legal drafts, the persona MUST use Anthropic's longer-context claude-3-5-sonnet, not Bedrock's claude-3-haiku, because legal quality matters more than residency." The override path lets policy thread that needle per-alias without restructuring the whole `primary_provider` config (which would affect every other call). Without override, the only escape hatch would be a per-call API parameter — which defeats the abstraction and re-introduces the lock-in.

**Why does this task enforce ZDR + residency?** Both are tenant-policy invariants. Violating either is a contract breach with regulatory consequences (PDPL Art. 6, Decree 53/2022, GDPR Art. 5(1)(f)). Catching the violation at alias resolution (before the provider call) is much cheaper than catching it after the bytes are in flight — the precheck is the hot path; the provider call is 100-1000x more expensive. TASK-AI-015 and TASK-AI-016 add the *attestation surface* (which providers/models are ZDR, which regions count as sg-1); this task is the *enforcement point* that consumes those attestations.

**Why expose `fallback_position` in the result?** Three reasons. (1) memory audit row includes it (DEC-072 — every AI call's provenance is auditable; "this call used fallback #2 because primary was open" is part of the chain). (2) OBS dashboards filter by "calls that used a fallback" to detect primary-provider degradation before the SLA alarm fires. (3) TASK-AI-008's circuit breaker uses `fallback_position` to skip the open breakers — if `fallback_position: 0` is open, resolve picks 1.

**Why expose `latency_class` and not raw latency budgets?** Different providers have different latency profiles for the same logical alias. `chat.smart` on Bedrock-Claude-3.5-Sonnet is ~2-4s p95; on Anthropic-native-Claude-3.5-Sonnet it's ~1.5-3s; on a future fast-path provider it might be ~800ms. The downstream caller (TASK-AI-008 router, TASK-AI-010 streaming) needs to know what latency budget to set for the call, but doesn't need the exact number — `Standard | Fast | Slow` is enough resolution. The mapping from latency class to actual budget is config (`policy.ai_policy.call_timeout_seconds` from TASK-AI-005), not hard-coded here.

**Why not just use LiteLLM's model registry?** LiteLLM is Python and stateful; we're Rust and the alias registry is part of a deterministic policy snapshot. LiteLLM's model registry is also unversioned — a `pip install --upgrade litellm` could silently change which model `gpt-4` resolves to. Our closed set with explicit version pins (e.g., `anthropic.claude-3-5-sonnet-20241022-v2:0`) makes the contract explicit and version-locked. We've borrowed the *idea* of LiteLLM's alias→model mapping; we've not borrowed its implementation.

**Why is the alias-resolution decision auditable?** Every `ai.precheck` memory row (TASK-AI-001 §1 #6) includes the resolved provider + model. A future auditor asking "what model did the gateway use for tenant X's call at 14:32 yesterday?" gets the answer from the chain. Without aliasing, the answer is "whatever the caller wrote in the request payload" — which a malicious caller could spoof. With aliasing, the gateway is the authority.

---

## §3 — API contract (formal spec for AI-agent implementers)

### Public function signatures

```rust
// services/ai-gateway/src/alias.rs

/// Resolve a closed-set alias to a concrete (provider, model) tuple via tenant policy.
///
/// This is the only public entry point. Callers MUST NOT bypass this function
/// (e.g., reading `policy.ai_policy.primary_provider.model_alias_map` directly).
pub fn resolve(
    alias: &str,
    policy: &TenantPolicy,
) -> Result<ResolvedModel, AliasError>;

/// Return the closed set of supported aliases (for operator CLI / UI enumeration).
pub fn supported_aliases() -> &'static [&'static str];

/// Static const declaring the closed set.
pub const SUPPORTED_ALIASES: &[&str] = &[
    "chat.smart",
    "chat.fast",
    "chat.long",
    "embed.standard",
    "embed.code",
    "rerank.fast",
];
```

### Return + error types

```rust
// services/ai-gateway/src/alias/types.rs

#[derive(Debug, Clone)]
pub struct ResolvedModel {
    pub provider_kind: ProviderKind,           // bedrock | anthropic | openai | vertex | bge
    pub region: Option<String>,                // None for providers without regional pinning
    pub model: String,                         // e.g. "anthropic.claude-3-5-sonnet-20241022-v2:0"
    pub fallback_position: u8,                 // 0 = primary, 1 = first fallback, 2 = second, ...
    pub is_zdr: bool,                          // whether the resolved provider is ZDR-attested
    pub latency_class: LatencyClass,           // fast | standard | slow (drives TASK-AI-008 timeout)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AliasError {
    /// The alias isn't in SUPPORTED_ALIASES.
    UnknownAlias {
        alias: String,
        supported: Vec<String>,
    },
    /// The resolved (provider, model) pair has no cost-table entry — TASK-AI-007 source of truth.
    ResolvedModelMissingCostEntry {
        provider: ProviderKind,
        model: String,
    },
    /// Policy requires ZDR but the resolved provider isn't attested.
    ZdrViolation {
        resolved_provider: ProviderKind,
        resolved_model: String,
    },
    /// Policy's residency pin doesn't match the resolved provider's region.
    ResidencyViolation {
        resolved_region: Option<String>,
        policy_residency: Residency,
    },
    /// Walked primary + all fallbacks; no provider has this alias in its model_alias_map.
    NoProviderHasAlias {
        alias: String,
        providers_tried: u8,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LatencyClass {
    Fast,       // typically <2s p95 (haiku, gpt-4o-mini, embeds, rerank)
    Standard,   // typically <5s p95 (sonnet, gpt-4o)
    Slow,       // typically <30s p95 (opus, long-context chat)
}

impl LatencyClass {
    /// Map a class to the timeout budget config field — TASK-AI-008 reads this.
    pub fn timeout_budget_seconds(&self, policy: &TenantPolicy) -> u32 {
        match self {
            Self::Fast => policy.ai_policy.call_timeout_seconds.min(15),
            Self::Standard => policy.ai_policy.call_timeout_seconds,
            Self::Slow => policy.ai_policy.call_timeout_seconds.max(60),
        }
    }
}
```

### Alias registry (closed set with intent annotations)

| Alias | Use case | Typical latency class | Typical token budget |
|---|---|---|---:|
| `chat.smart` | Default chat / Genie / orchestrator (CUO Phase 2 LLM) | Standard | 8k in + 4k out |
| `chat.fast` | Ambient digests, low-stakes summaries, OBS triage | Fast | 4k in + 1k out |
| `chat.long` | Document Q&A, 100k+ context, legal/contract review | Slow | 100k in + 8k out |
| `embed.standard` | KB ingest, memory Layer 2 vectors, persona-keyword updates | Fast | 8k tokens per batch |
| `embed.code` | KB-code embeddings, code-search corpus | Fast | 4k tokens per batch |
| `rerank.fast` | BGE-rerank cross-encoder; KB top-K refinement | Fast | (query, candidate[≤50]) |

### TenantPolicy schema additions (extending TASK-AI-005's `AiPolicy`)

```yaml
# config/tenants/<tenant_id>.yaml — slice-2 additions
ai_policy:
  # … existing fields from TASK-AI-005 …

  alias_overrides:                           # OPTIONAL — per-alias override beats primary + fallback
    chat.long:                                # alias → override target
      provider:
        kind: anthropic
        model_alias_map:
          chat.long: claude-3-5-sonnet-20241022   # 200k context, native (not Bedrock)
    embed.code:
      provider:
        kind: openai
        model_alias_map:
          embed.code: text-embedding-3-large   # better at code than BGE-M3 for some tenants

  residency_requires_regional_provider: false   # OPTIONAL — defaults false (anthropic native passes residency)
                                                 # set true to force regional pinning (rejects no-region providers)
```

### Provider trait surface required by `resolve()`

```rust
// In TASK-AI-005's policy schema, every Provider variant must implement:
pub trait Provider {
    fn kind(&self) -> ProviderKind;
    fn region(&self) -> Option<String>;
    fn model_for_alias(&self, alias: &str) -> Option<&str>;
    // NOTE: is_zdr is intentionally NOT on the Provider trait. The single source
    // of truth for ZDR attestation is TASK-AI-015's zdr::is_zdr(&kind, model)
    // function. Allowing per-provider claims in policy YAML would create two
    // potentially-disagreeing sources; we prevent that at the type level by
    // removing the method.
}
```

---

## §4 — Acceptance criteria (testable, ordered, numbered)

1. **Happy path — primary** — Given `policy.primary_provider = Bedrock { region: "ap-southeast-1", model_alias_map: { "chat.smart": "anthropic.claude-3-5-sonnet-20241022-v2:0" }, ... }`, `resolve("chat.smart", &policy)` MUST return `Ok(ResolvedModel { provider_kind: Bedrock, region: Some("ap-southeast-1"), model: "anthropic.claude-3-5-sonnet-20241022-v2:0", fallback_position: 0, is_zdr: true, latency_class: Standard })`.
2. **Fallback resolution** — Given primary doesn't have `chat.long` in its `model_alias_map`, and `fallback_chain[0] = Anthropic { model_alias_map: { "chat.long": "claude-3-5-sonnet-20241022" }, ... }`, `resolve("chat.long", &policy)` MUST return `Ok(ResolvedModel { provider_kind: Anthropic, fallback_position: 1, latency_class: Slow, ... })`.
3. **Override beats primary** — Given `policy.alias_overrides["chat.long"] = anthropic.claude-3-5-sonnet-20241022` AND primary's `model_alias_map["chat.long"] = bedrock:claude-3-haiku`, `resolve("chat.long", &policy)` MUST return the override's `(Anthropic, claude-3-5-sonnet-20241022)`, not the primary's Bedrock entry. `fallback_position: 0` (override always reports as position 0).
4. **Unknown alias** — `resolve("chat.tiny", &policy)` MUST return `Err(AliasError::UnknownAlias { alias: "chat.tiny", supported: vec!["chat.smart", "chat.fast", "chat.long", "embed.standard", "embed.code", "rerank.fast"] })`.
5. **ZDR violation** — Given `policy.ai_policy.zdr_required: true` AND resolved provider is `OpenAI { gpt-4o, is_zdr: false }`, `resolve("chat.smart", &policy)` MUST return `Err(AliasError::ZdrViolation { resolved_provider: OpenAI, resolved_model: "gpt-4o" })`. No `ResolvedModel` is constructed.
6. **Residency violation** — Given `policy.residency: Sg1` AND resolved provider is `Bedrock { region: "us-east-1", ... }`, `resolve("chat.smart", &policy)` MUST return `Err(AliasError::ResidencyViolation { resolved_region: Some("us-east-1"), policy_residency: Sg1 })`.
7. **Cost-table validation** — Given `cost_table::lookup(&Bedrock, "claude-99-fake-model")` returns `None` (a model name not in cost_rates.yaml), `resolve` of an alias pointing to that model MUST return `Err(AliasError::ResolvedModelMissingCostEntry { provider: Bedrock, model: "claude-99-fake-model" })`.
8. **No provider has alias** — Given `chat.smart` is missing from primary's map AND every fallback's map, `resolve("chat.smart", &policy)` MUST return `Err(AliasError::NoProviderHasAlias { alias: "chat.smart", providers_tried: <primary_count + fallback_count> })`.
9. **Override misses cost-table** — Given override points to a model with no cost-table entry, return `Err(ResolvedModelMissingCostEntry)`. The override path does NOT fall through to primary on cost-table miss; failure is terminal.
10. **Override misses ZDR** — Given override target is non-ZDR and `zdr_required: true`, return `Err(ZdrViolation)`. Override doesn't bypass policy invariants.
10a. **Override misses residency** — Given override target's region (e.g., `Bedrock us-east-1`) doesn't match `policy.residency: Sg1`, `resolve` MUST return `Err(ResidencyViolation { resolved_region: Some("us-east-1"), policy_residency: Sg1 })` from the override path. Same terminal semantics as #9 and #10 — no fallthrough to primary.
11. **Determinism property test** — `proptest!` runs 10,000 random `(alias, policy)` pairs; for each pair, calling `resolve` 100 times produces byte-identical `Result` (compared via `Debug` formatting). 0 flaky outcomes.
12. **Latency budget** — Benchmark 100,000 sequential `resolve()` calls on a warm policy snapshot. Total elapsed MUST be < 100ms (< 1µs per call). Recorded via `cargo bench` Criterion benchmark.
13. **Concurrent safety** — 16 tokio tasks each call `resolve()` 10,000 times concurrently against the same `Arc<TenantPolicy>`. Zero panics; results match the single-threaded baseline.
14. **OBS metrics emitted** — After 100 resolutions composed of 90 primary + 10 first-fallback + 0 errors, `ai_alias_resolutions_total{fallback_position="0"}` increments by 90, `{fallback_position="1"}` by 10. Failure counter never increments on success path.
15. **Empty fallback chain** — Given `policy.fallback_chain: []` AND primary doesn't have the alias, return `Err(NoProviderHasAlias { providers_tried: 1 })` (primary counted, but no fallbacks tried).
16. **Override without provider field** — A malformed override YAML missing the inner `provider` field MUST be rejected at TASK-AI-005 load time (not at `resolve` runtime). `resolve` assumes well-formed input.

---

## §5 — Verification method

**Integration test:** `services/ai-gateway/tests/alias_resolution_test.rs`

```rust
use cyberos_ai_gateway::alias;
use cyberos_ai_gateway::alias::{AliasError, ResolvedModel, LatencyClass};
use cyberos_ai_gateway::policy::*;
use rust_decimal_macros::dec;

#[test]
fn resolves_chat_smart_to_bedrock_via_primary() {
    let policy = test_policy_with_bedrock_primary();
    let r = alias::resolve("chat.smart", &policy).unwrap();
    assert_eq!(r.provider_kind, ProviderKind::Bedrock);
    assert_eq!(r.model, "anthropic.claude-3-5-sonnet-20241022-v2:0");
    assert_eq!(r.region, Some("ap-southeast-1".to_string()));
    assert_eq!(r.fallback_position, 0);
    assert!(r.is_zdr);
    assert_eq!(r.latency_class, LatencyClass::Standard);
}

#[test]
fn falls_through_to_anthropic_for_chat_long() {
    let mut policy = test_policy_with_bedrock_primary();
    // Primary doesn't have chat.long; fallback (Anthropic native) does
    policy.ai_policy.primary_provider.model_alias_map.remove("chat.long");
    policy.ai_policy.fallback_chain.push(Provider::Anthropic {
        model_alias_map: hashmap! {
            "chat.long".into() => "claude-3-5-sonnet-20241022".into(),
        },
    });
    let r = alias::resolve("chat.long", &policy).unwrap();
    assert_eq!(r.provider_kind, ProviderKind::Anthropic);
    assert_eq!(r.fallback_position, 1);
}

#[test]
fn override_beats_primary() {
    let mut policy = test_policy_with_bedrock_primary();
    policy.ai_policy.alias_overrides = Some(hashmap! {
        "chat.long".into() => OverrideTarget {
            provider: Provider::Anthropic {
                model_alias_map: hashmap! { "chat.long".into() => "claude-3-5-sonnet-20241022".into() },
            },
        },
    });
    let r = alias::resolve("chat.long", &policy).unwrap();
    assert_eq!(r.provider_kind, ProviderKind::Anthropic);
    assert_eq!(r.fallback_position, 0);  // override always reports as position 0
}

#[test]
fn unknown_alias_errors_with_supported_list() {
    let policy = test_policy_with_bedrock_primary();
    let err = alias::resolve("chat.tiny", &policy).unwrap_err();
    match err {
        AliasError::UnknownAlias { alias, supported } => {
            assert_eq!(alias, "chat.tiny");
            assert_eq!(supported.len(), 6);
            assert!(supported.contains(&"chat.smart".to_string()));
        }
        _ => panic!("expected UnknownAlias, got {:?}", err),
    }
}

#[test]
fn zdr_required_but_provider_not_zdr_errors() {
    let mut policy = test_policy_with_bedrock_primary();
    policy.ai_policy.zdr_required = true;
    policy.ai_policy.primary_provider = Provider::OpenAI {  // OpenAI default = not ZDR
        model_alias_map: hashmap! { "chat.smart".into() => "gpt-4o".into() },
    };
    let err = alias::resolve("chat.smart", &policy).unwrap_err();
    assert!(matches!(err, AliasError::ZdrViolation { resolved_provider: ProviderKind::OpenAI, .. }));
}

#[test]
fn residency_pin_mismatch_errors() {
    let mut policy = test_policy_with_bedrock_primary();
    policy.ai_policy.residency = Residency::Eu1;   // Frankfurt
    // primary is Bedrock in ap-southeast-1 — mismatch
    let err = alias::resolve("chat.smart", &policy).unwrap_err();
    assert!(matches!(err, AliasError::ResidencyViolation { policy_residency: Residency::Eu1, .. }));
}

#[test]
fn cost_table_missing_errors() {
    let mut policy = test_policy_with_bedrock_primary();
    policy.ai_policy.primary_provider.model_alias_map.insert(
        "chat.smart".into(), "fake-model-not-in-cost-table".into(),
    );
    let err = alias::resolve("chat.smart", &policy).unwrap_err();
    assert!(matches!(err, AliasError::ResolvedModelMissingCostEntry { .. }));
}

#[test]
fn empty_fallback_returns_no_provider() {
    let mut policy = test_policy_with_bedrock_primary();
    policy.ai_policy.primary_provider.model_alias_map.remove("chat.long");
    policy.ai_policy.fallback_chain.clear();
    let err = alias::resolve("chat.long", &policy).unwrap_err();
    assert!(matches!(err, AliasError::NoProviderHasAlias { providers_tried: 1, .. }));
}

// Property test for determinism
proptest! {
    #[test]
    fn determinism_property(alias in alias_strategy(), policy in policy_strategy()) {
        let r1 = alias::resolve(&alias, &policy);
        let r2 = alias::resolve(&alias, &policy);
        let r3 = alias::resolve(&alias, &policy);
        prop_assert_eq!(format!("{:?}", r1), format!("{:?}", r2));
        prop_assert_eq!(format!("{:?}", r2), format!("{:?}", r3));
    }
}
```

**Benchmark:** `services/ai-gateway/benches/alias_resolution_bench.rs`

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use cyberos_ai_gateway::alias;

fn bench_resolve(c: &mut Criterion) {
    let policy = test_policy_with_bedrock_primary();
    c.bench_function("alias::resolve chat.smart", |b| {
        b.iter(|| alias::resolve(black_box("chat.smart"), black_box(&policy)));
    });
}

criterion_group!(benches, bench_resolve);
criterion_main!(benches);
```

Run via:

```bash
cd /Users/stephencheng/Projects/CyberSkill/cyberos
cargo test -p cyberos-ai-gateway alias
cargo bench -p cyberos-ai-gateway alias_resolution_bench
```

**CI gate:** the integration test file is required on every PR touching `services/ai-gateway/src/alias/**`. Benchmark regression > 20% fails the PR.

---

## §6 — Implementation skeleton (suggested scaffold for AI-agent code-gen)

```rust
// services/ai-gateway/src/alias.rs

use crate::alias::types::{AliasError, LatencyClass, ResolvedModel};
use crate::cost_table;
use crate::policy::{Provider, ProviderKind, Residency, TenantPolicy};
use crate::zdr;
use std::time::Instant;

mod metrics {
    use once_cell::sync::Lazy;
    use prometheus::{register_counter_vec, register_histogram, CounterVec, Histogram};

    pub static ALIAS_RESOLUTIONS: Lazy<CounterVec> = Lazy::new(|| {
        register_counter_vec!(
            "ai_alias_resolutions_total",
            "Successful alias resolutions by alias, provider, and fallback position",
            &["alias", "resolved_provider", "fallback_position"]
        ).unwrap()
    });

    pub static ALIAS_FAILURES: Lazy<CounterVec> = Lazy::new(|| {
        register_counter_vec!(
            "ai_alias_resolution_failures_total",
            "Failed alias resolutions by alias and failure reason",
            &["alias", "reason"]
        ).unwrap()
    });

    pub static ALIAS_LATENCY_NS: Lazy<Histogram> = Lazy::new(|| {
        register_histogram!(
            "ai_alias_resolution_latency_ns",
            "Latency of alias::resolve calls in nanoseconds",
            vec![100.0, 500.0, 1_000.0, 5_000.0, 10_000.0]
        ).unwrap()
    });
}

pub const SUPPORTED_ALIASES: &[&str] = &[
    "chat.smart", "chat.fast", "chat.long",
    "embed.standard", "embed.code", "rerank.fast",
];

pub fn supported_aliases() -> &'static [&'static str] {
    SUPPORTED_ALIASES
}

pub fn resolve(alias: &str, policy: &TenantPolicy) -> Result<ResolvedModel, AliasError> {
    let started = Instant::now();

    // 1. Closed-set check — cheapest reject path
    if !SUPPORTED_ALIASES.contains(&alias) {
        metrics::ALIAS_FAILURES.with_label_values(&[alias, "unknown_alias"]).inc();
        return Err(AliasError::UnknownAlias {
            alias: alias.to_string(),
            supported: SUPPORTED_ALIASES.iter().map(|s| s.to_string()).collect(),
        });
    }

    // 2. Override path — beats primary + fallback
    if let Some(overrides) = &policy.ai_policy.alias_overrides {
        if let Some(override_target) = overrides.get(alias) {
            let r = check_and_build(alias, &override_target.provider, 0, policy)?;
            record_success(alias, &r, started);
            return Ok(r);
        }
    }

    // 3. Primary path
    let primary = &policy.ai_policy.primary_provider;
    if let Some(model) = primary.model_for_alias(alias) {
        let r = check_and_build_with_model(alias, primary, model, 0, policy)?;
        record_success(alias, &r, started);
        return Ok(r);
    }

    // 4. Fallback chain (in order)
    let mut providers_tried: u8 = 1;  // primary counted
    for (idx, fb) in policy.ai_policy.fallback_chain.iter().enumerate() {
        providers_tried = providers_tried.saturating_add(1);
        if let Some(model) = fb.model_for_alias(alias) {
            let r = check_and_build_with_model(alias, fb, model, (idx + 1) as u8, policy)?;
            record_success(alias, &r, started);
            return Ok(r);
        }
    }

    metrics::ALIAS_FAILURES.with_label_values(&[alias, "no_provider_has_alias"]).inc();
    Err(AliasError::NoProviderHasAlias {
        alias: alias.to_string(),
        providers_tried,
    })
}

fn check_and_build(
    alias: &str,
    provider: &Provider,
    fallback_position: u8,
    policy: &TenantPolicy,
) -> Result<ResolvedModel, AliasError> {
    let model = provider
        .model_for_alias(alias)
        .ok_or_else(|| AliasError::NoProviderHasAlias {
            alias: alias.to_string(),
            providers_tried: 1,
        })?;
    check_and_build_with_model(alias, provider, model, fallback_position, policy)
}

fn check_and_build_with_model(
    alias: &str,
    provider: &Provider,
    model: &str,
    fallback_position: u8,
    policy: &TenantPolicy,
) -> Result<ResolvedModel, AliasError> {
    let kind = provider.kind();
    let region = provider.region();

    // Cost-table check (TASK-AI-007) — first because most common config error
    if cost_table::lookup(&kind, model).is_none() {
        metrics::ALIAS_FAILURES.with_label_values(&[alias, "cost_missing"]).inc();
        return Err(AliasError::ResolvedModelMissingCostEntry {
            provider: kind,
            model: model.to_string(),
        });
    }

    // ZDR check (TASK-AI-015 attestation table — single source of truth).
    // Provider's own `is_zdr` field (if present in policy schema) is NOT consulted here.
    if policy.ai_policy.zdr_required && !zdr::is_zdr(&kind, model) {
        metrics::ALIAS_FAILURES.with_label_values(&[alias, "zdr"]).inc();
        return Err(AliasError::ZdrViolation {
            resolved_provider: kind,
            resolved_model: model.to_string(),
        });
    }

    // Residency check (TASK-AI-016 matcher)
    if let Some(region_str) = &region {
        if !crate::residency::matches(policy.ai_policy.residency, region_str) {
            metrics::ALIAS_FAILURES.with_label_values(&[alias, "residency"]).inc();
            return Err(AliasError::ResidencyViolation {
                resolved_region: Some(region_str.clone()),
                policy_residency: policy.ai_policy.residency,
            });
        }
    } else if policy.ai_policy.residency_requires_regional_provider.unwrap_or(false) {
        // Provider has no region (e.g., Anthropic native), but policy requires regional
        return Err(AliasError::ResidencyViolation {
            resolved_region: None,
            policy_residency: policy.ai_policy.residency,
        });
    }

    Ok(ResolvedModel {
        provider_kind: kind,
        region,
        model: model.to_string(),
        fallback_position,
        // ISS-001 fix: single source of truth for is_zdr — TASK-AI-015's attestation table.
        // Provider's own `is_zdr` field (from policy YAML) is NOT consulted here.
        is_zdr: zdr::is_zdr(&kind, model),
        latency_class: latency_class_for_alias(alias),
    })
}

fn latency_class_for_alias(alias: &str) -> LatencyClass {
    match alias {
        "chat.long" => LatencyClass::Slow,
        "chat.smart" => LatencyClass::Standard,
        "chat.fast" | "embed.standard" | "embed.code" | "rerank.fast" => LatencyClass::Fast,
        _ => unreachable!("alias already validated against SUPPORTED_ALIASES"),
    }
}

fn record_success(alias: &str, r: &ResolvedModel, started: Instant) {
    let elapsed_ns = started.elapsed().as_nanos() as f64;
    metrics::ALIAS_RESOLUTIONS
        .with_label_values(&[alias, &format!("{:?}", r.provider_kind), &r.fallback_position.to_string()])
        .inc();
    metrics::ALIAS_LATENCY_NS.observe(elapsed_ns);
}
```

*Scaffold above is suggestive, not normative. AC §4 is the contract.*

---

## §7 — Dependencies

**Code dependencies (must exist before this task can build):**
- **TASK-AI-005** — `TenantPolicy` struct + the `alias_overrides`, `residency_requires_regional_provider` field additions. The schema lives in `services/ai-gateway/src/policy/schema.rs`. This task adds 2 fields if not already present.
- **TASK-AI-007** — `cost_table::lookup` function. Used at step 5 of every resolution.
- **TASK-AI-015** — `zdr::is_zdr` function (attestation source of truth). Used at step 6.
- **TASK-AI-016** — `residency::matches` function. Used at step 7.
- **TASK-AI-022** — W3C trace-propagation contract. This task's §1 #16 asserts `alias::resolve` does not strip span context from its caller; TASK-AI-022 owns the propagation contract for the outbound `Provider::complete()` call that follows.

**Concept dependencies (must be agreed before this task can be audited):**
- The closed alias set is fixed at 6 for slice 2. Adding `chat.toolcall`, `embed.multimodal`, or `chat.fast-long` is a separate task.
- `latency_class_for_alias` mapping is opinionated; the latency-class → actual-budget translation is a policy field, not hardcoded here.
- `fallback_position` semantics: override → 0, primary → 0, fallback[N] → N+1. The "override reports as 0" choice matches TASK-AI-008's circuit-breaker logic.

**Operational dependencies:**
- The policy cache (TASK-AI-005's `ArcSwap<HashMap<String, Arc<TenantPolicy>>>`) must be initialised before any `resolve()` call. Startup order: cost-table → ZDR table → policy loader → router.
- The cost table and ZDR table are independent ArcSwaps — both must be loaded.

**Crate dependencies (in `Cargo.toml`):**
- `prometheus` (for OTel metric helpers)
- `proptest` (dev-dep, for §5 property test)
- `criterion` (dev-dep, for §5 benchmark)

---

## §8 — Example payloads

### Caller in TASK-AI-001 (precheck)

```rust
let resolved = alias::resolve(&req.model_alias, &policy)
    .map_err(|e| match e {
        AliasError::UnknownAlias { .. } => PrecheckError::CostEstimateFailed { reason: "unknown_alias".into() },
        AliasError::ZdrViolation { .. } => PrecheckError::CostEstimateFailed { reason: "zdr".into() },
        // ... map other variants
        _ => PrecheckError::CostEstimateFailed { reason: "alias_failed".into() },
    })?;

// Continue with the resolved (provider_kind, model) for cost estimation:
let cost_per_1k = cost_table::lookup(&resolved.provider_kind, &resolved.model)
    .ok_or_else(|| PrecheckError::CostEstimateFailed { reason: "no_cost_entry".into() })?;
```

### Output (happy path — primary)

```rust
ResolvedModel {
    provider_kind: ProviderKind::Bedrock,
    region: Some("ap-southeast-1".to_string()),
    model: "anthropic.claude-3-5-sonnet-20241022-v2:0".to_string(),
    fallback_position: 0,
    is_zdr: true,
    latency_class: LatencyClass::Standard,
}
```

### Output (fallback — primary missed)

```rust
ResolvedModel {
    provider_kind: ProviderKind::Anthropic,
    region: None,                  // Anthropic native has no regional pinning
    model: "claude-3-5-sonnet-20241022".to_string(),
    fallback_position: 1,          // bedrock primary didn't have chat.long
    is_zdr: true,
    latency_class: LatencyClass::Slow,
}
```

### Output (override beats primary)

```rust
ResolvedModel {
    provider_kind: ProviderKind::Anthropic,
    region: None,
    model: "claude-3-5-sonnet-20241022".to_string(),
    fallback_position: 0,          // override always reports as position 0
    is_zdr: true,
    latency_class: LatencyClass::Slow,
}
```

### Error — unknown alias

```rust
Err(AliasError::UnknownAlias {
    alias: "chat.tiny".to_string(),
    supported: vec![
        "chat.smart".to_string(), "chat.fast".to_string(), "chat.long".to_string(),
        "embed.standard".to_string(), "embed.code".to_string(), "rerank.fast".to_string(),
    ],
})
```

### Error — ZDR violation

```rust
Err(AliasError::ZdrViolation {
    resolved_provider: ProviderKind::OpenAI,
    resolved_model: "gpt-4o".to_string(),
})
```

### Error — residency violation

```rust
Err(AliasError::ResidencyViolation {
    resolved_region: Some("us-east-1".to_string()),
    policy_residency: Residency::Sg1,
})
```

### Error — cost-table missing

```rust
Err(AliasError::ResolvedModelMissingCostEntry {
    provider: ProviderKind::Bedrock,
    model: "claude-99-fake-model".to_string(),
})
```

### memory audit row carrying resolved metadata (via TASK-AI-001 precheck row)

```json
{
  "seq": 18421,
  "ts_ns": 1763112131000000000,
  "op": "put",
  "path": "memories/decisions/ai-invocations/1763112131000_org-cyberskill_01HZK9R7A2B4C8D6.md",
  "extra": {
    "kind": "ai.precheck",
    "tenant_id": "org:cyberskill",
    "agent_persona": "cuo-cpo@0.4.1",
    "model_alias": "chat.smart",
    "resolved_provider": "bedrock",
    "resolved_model": "anthropic.claude-3-5-sonnet-20241022-v2:0",
    "resolved_region": "ap-southeast-1",
    "fallback_position": 0,
    "is_zdr": true,
    "latency_class": "standard",
    "estimated_usd": 0.0085,
    "current_spent_usd": 47.23
  },
  "prev_chain": "...",
  "chain": "..."
}
```

### OBS metric snapshot after 100 resolutions

```
# HELP ai_alias_resolutions_total Successful alias resolutions
# TYPE ai_alias_resolutions_total counter
ai_alias_resolutions_total{alias="chat.smart",resolved_provider="Bedrock",fallback_position="0"} 78
ai_alias_resolutions_total{alias="chat.smart",resolved_provider="Anthropic",fallback_position="1"} 7
ai_alias_resolutions_total{alias="chat.fast",resolved_provider="Bedrock",fallback_position="0"} 12
ai_alias_resolutions_total{alias="chat.long",resolved_provider="Anthropic",fallback_position="0"} 3

# HELP ai_alias_resolution_latency_ns Latency of alias::resolve calls
# TYPE ai_alias_resolution_latency_ns histogram
ai_alias_resolution_latency_ns_bucket{le="100"} 87
ai_alias_resolution_latency_ns_bucket{le="500"} 99
ai_alias_resolution_latency_ns_bucket{le="1000"} 100
```

---

## §9 — Open questions

All resolved at authoring time. No deferred questions.

For reference, the questions considered + resolved during authoring:

1. **~~Should latency_class be exposed in ResolvedModel or computed by caller?~~** — RESOLVED: exposed in struct. Callers shouldn't re-derive; mapping is TASK-AI-006's concern.
2. **~~Should override path consult ZDR / residency / cost-table checks?~~** — RESOLVED: yes (§1 #2 + ACs #9, #10). Override is a policy decision; policy invariants still apply.
3. **~~What's the override fallback semantics?~~** — RESOLVED: override is terminal. If override fails ZDR / residency / cost, error returns immediately; we do NOT fall through to primary. Rationale: the override exists because the operator explicitly wanted that target; falling through would silently subvert their intent.
4. **~~Should we cache resolved results?~~** — RESOLVED: no. The resolution is sub-microsecond. Caching adds invalidation complexity for zero perceptible gain.
5. **~~How does `residency_requires_regional_provider` interact with Anthropic native?~~** — RESOLVED: default false (Anthropic native passes residency by being region-less). Setting true rejects region-less providers — useful for the most-strict residency tenants. Documented in §1 #7 + §3 schema.

---

## §10 — Failure modes inventory (all error paths)

| Failure | Detection | Return | HTTP map | Recovery |
|---|---|---|---|---|
| Unknown alias | Closed-set check at step 1 | `Err(UnknownAlias)` with supported list | Caller maps to `400 BAD_REQUEST` | Caller fixes; static check possible at compile time via const |
| ZDR violation (primary) | `zdr::is_zdr` returns false + `zdr_required: true` | `Err(ZdrViolation { resolved_provider, resolved_model })` | `403 FORBIDDEN` (`zdr_violation`) | Operator switches provider or relaxes `zdr_required`; TASK-AI-015 attestation refresh |
| ZDR violation (override) | Same check on override target | `Err(ZdrViolation)` (no fallthrough) | `403` | Operator removes override or chooses ZDR target |
| Residency violation (regional mismatch) | `residency::matches` returns false | `Err(ResidencyViolation { resolved_region, policy_residency })` | `403 FORBIDDEN` | Operator pins provider to the right region |
| Residency violation (no region on provider) | `residency_requires_regional_provider: true` + provider has no region | `Err(ResidencyViolation { resolved_region: None, .. })` | `403` | Operator chooses regional provider or relaxes the flag |
| Cost-table missing entry | `cost_table::lookup` returns None | `Err(ResolvedModelMissingCostEntry { provider, model })` | `503 SERVICE_UNAVAILABLE` (`no_cost_table_for_model`) | Operator adds entry to `cost_rates.yaml`; gateway hot-reloads |
| No provider has alias | Walked primary + all fallbacks; none match | `Err(NoProviderHasAlias { alias, providers_tried })` | `503` (`no_provider_for_alias`) | Operator updates `model_alias_map` in policy YAML |
| Empty fallback chain + primary miss | Special case of above; `providers_tried: 1` | Same as above | `503` | Operator adds at least one fallback or fixes primary |
| Concurrent policy hot-reload | `ArcSwap::load()` snapshot semantics | In-flight call uses pre-reload snapshot; next call sees new | (no error) | By design — no torn read possible |
| Policy not initialised | `OnceCell::get()` returns None (TASK-AI-005 init pending) | This case is impossible at runtime — TASK-AI-005 init blocks gateway startup | (gateway exits 1) | Operator fixes config; redeploys |
| Override YAML malformed | Rejected at TASK-AI-005 load time | (never reaches resolve) | (gateway exits 1) | Operator fixes YAML; redeploys |
| `model_for_alias` returns empty string | Defensive: treated as miss → `NoProviderHasAlias` | Same as above | `503` | Operator fixes policy YAML (empty model string is a config error) |

---

## §11 — Notes (informational, no normative force)

- This task is intentionally small in code (6h effort) because the complexity lives in the data model (TASK-AI-005's policy schema) and the cross-cutting attestation tables (TASK-AI-007 cost, TASK-AI-015 ZDR, TASK-AI-016 residency). The function itself is ~150 lines of Rust.
- After this task ships, every other AI Gateway task uses `alias::resolve()` instead of inline alias logic. A refactor PR will touch TASK-AI-001's §6 skeleton to call `resolve()` rather than the inline `resolve_model_alias` placeholder. The refactor is mechanical.
- The closed alias set is intentional friction — adding a new alias (`chat.toolcall`, `embed.multimodal`, `chat.fast-long`) requires updating the const, the schema, the latency-class mapping, and the test suite. This is the right amount of pushback against ad-hoc alias proliferation.
- The "override reports as `fallback_position: 0`" choice is debatable. The alternative — reporting some marker like `255` — would make "this call used an override" visible in OBS. The current choice prioritises simplicity (no special case). If operators report that override visibility is needed, we add a `was_override: bool` field in slice 4 (task-AI-006a).
- Latency budget on this function is so tight that the OBS histogram has nano-second buckets (100ns / 500ns / 1µs / 5µs / 10µs). Any call landing in the >10µs bucket triggers an investigation — likely a CPU pegged elsewhere.
- The latency-class → timeout-budget mapping (`LatencyClass::timeout_budget_seconds`) is a convenience helper for TASK-AI-008. The policy field `call_timeout_seconds` is still the canonical config; this helper just translates it.
- TASK-AI-006 + TASK-AI-007 + TASK-AI-015 + TASK-AI-016 together form the "policy enforcement layer" that sits between consumers and the provider layer. Treat the four as a unit — none ships alone.

---

*End of TASK-AI-006. Status: draft (10/10 target, expanded from compressed first-pass per workflow correction 2026-05-15). Run `task-audit` next: `cargo run -p cyberos-skill-cli -- run task-audit --input '{"fr_path": "docs/tasks/ai/TASK-AI-006-model-alias-resolution/spec.md"}'`*
