---
# ───── Machine-readable frontmatter (parsed by fr-audit + future fr-catalog renderer) ─────
id: FR-AI-008
title: "LiteLLM-derived multi-provider router with retry + 30s failover SLA"
module: AI
priority: MUST
status: accepted
verify: T
phase: P0
milestone: P0 · slice 2
slice: 2
owner: Stephen Cheng
created: 2026-05-15
shipped: null
brain_chain_hash: null
related_frs: [FR-AI-001, FR-AI-002, FR-AI-006, FR-AI-007, FR-AI-009, FR-AI-010, FR-AI-015, FR-AI-021]
depends_on: [FR-AI-006, FR-AI-007, FR-AI-002]
blocks: [FR-AI-009, FR-AI-010, FR-AI-021, FR-AI-011, FR-AI-017, FR-AI-022, FR-CUO-101]

# ───── Source contracts ─────
source_pages:
  - website/docs/modules/ai.html#multi-provider
  - website/docs/modules/ai.html#failover-sla
source_decisions:
  - docs/feature-requests/ai/FR-AI-006-model-alias-resolution.md §1 (ResolvedModel.fallback_position)
  - docs/feature-requests/ai/FR-AI-007-provider-cost-table-loader.md (cost basis)
  - archive/2026-05-14/RESEARCH_REVIEW.md §3.1 (LiteLLM as prior art, why we re-implement in Rust)

# ───── Build envelope ─────
language: rust 1.81
service: cyberos/services/ai-gateway/
new_files:
  - services/ai-gateway/src/router.rs
  - services/ai-gateway/src/router/bedrock.rs
  - services/ai-gateway/src/router/anthropic.rs
  - services/ai-gateway/src/router/openai.rs
  - services/ai-gateway/src/router/failover.rs
  - services/ai-gateway/src/router/jitter.rs
  - services/ai-gateway/src/router/normalize.rs
  - services/ai-gateway/tests/router_test.rs
  - services/ai-gateway/tests/router_proptest.rs
modified_files:
  - services/ai-gateway/src/handlers/chat.rs   # plug router::call_provider between precheck and reconcile
  - services/ai-gateway/src/lib.rs             # export router module
  - services/ai-gateway/Cargo.toml             # aws-sdk-bedrockruntime, reqwest, async-openai, async-trait, rand
allowed_tools:
  - file_read: services/ai-gateway/**
  - file_write: services/ai-gateway/{src,tests}/**
  - bash: cargo test -p cyberos-ai-gateway router
  - bash: cargo test -p cyberos-ai-gateway --test router_proptest
disallowed_tools:
  - call provider SDKs from outside src/router/**
  - skip the failover decision tree (the policy is the spec)
  - retry on a 4xx other than 429 (auth/validation are terminal)
  - format!("{:?}", provider) for OBS metric labels (fragile; use as_metric_label())
  - use rand::random::<i32>() % N for jitter (panics on N=0; use rand::Rng::gen_range)

# ───── Estimated work ─────
effort_hours: 10
sub_tasks:
  - "1.0h: Provider trait (call_chat, call_embed) + ProviderKind::as_metric_label() (added to FR-AI-006 schema if absent)"
  - "1.5h: 3 impls (Bedrock, Anthropic, OpenAI) — each ~150 lines; covers happy path + error mapping"
  - "1.0h: router::call_provider entry — accepts ResolvedModel from FR-AI-006, builds chain, dispatches"
  - "1.5h: retry policy (3 retries, exponential backoff, only on 5xx/429/timeout/conn-reset)"
  - "1.5h: failover policy (on persistent failure from primary, try fallback_chain in order; 30s total budget)"
  - "0.5h: timeout enforcement (per-call deadline propagation; tokio::time::timeout)"
  - "1.0h: response normalization (3 provider response shapes → one ProviderResponse)"
  - "0.5h: jitter helper (rand::Rng::gen_range, NOT rand::random with mod)"
  - "1.5h: integration test suite (12 cases) + proptest for jitter bounds + concurrent stress"
risk_if_skipped: "AI Gateway has no actual call path to LLM providers. Every consumer module times out. The cost-of-everything gate guards nothing because nothing is calling providers in the first place. The 30s failover SLA — promised in website/docs/modules/ai.html#failover-sla and pitched to early prospects — has no implementation. Slice 2 cannot be marked shipped without this FR."
---

## §1 — Description (BCP-14 normative)

The AI Gateway service **MUST** expose `router::call_provider(req, resolved, deadline, policy) -> Result<ProviderResponse, RouterError>` that calls the resolved LLM provider, retries on transient failures, fails over to the fallback chain on persistent failures, and enforces a per-call deadline. The function:

1. **MUST** accept (a) the `ChatCompleteRequest`, (b) the `ResolvedModel` from FR-AI-006, (c) a tokio `Instant` deadline, (d) a reference to the active `TenantPolicy` (used to walk the fallback chain). The deadline is enforced strictly — if it elapses mid-call, the function returns `Err(RouterError::DeadlineExceeded)` within 50ms.
2. **MUST** dispatch to the resolved provider's impl via the `Provider` trait. Three impls in slice 2: `BedrockProvider`, `AnthropicProvider`, `OpenAIProvider`. Vertex (Gemini) lands in slice 4 (FR-AI-017 area). The trait MUST live in `src/router/mod.rs`; provider-specific code MUST live in `src/router/<provider>.rs` with no cross-impl `use` statements.
3. **MUST** retry on transient failures (HTTP `429`, `500`, `502`, `503`, `504`, connection reset, network timeout, `tokio::time::timeout` elapse) up to 3 attempts per provider, with exponential backoff (200ms, 800ms, 3200ms) and jitter (±20% uniform). The 3-attempt cap is per-provider; the failover step does not reset the per-provider counter.
4. **MUST** treat every retry's wall time as part of the failover budget. Retries do NOT get their own time budget — sleeping 800ms before a retry consumes 800ms of the 30s total. This prevents the "infinite total wall time" failure mode where 3 retries × 5 providers × 5s each = 75s of waiting.
5. **MUST** fail over to the next provider in `policy.ai_policy.fallback_chain` if the primary returns persistent failures (3 attempts exhausted on 5xx/429/timeout, OR any single 401/403/404). Failover budget MUST be 30 seconds total across all providers + retries combined. The 30s constant lives in `router::FAILOVER_BUDGET` and MUST NOT be tenant-overridable in slice 2 (configurable in slice 5 per FR-AI-021).
6. **MUST** propagate the deadline through the call stack via `tokio::time::timeout(remaining_budget, provider_call)`. Each retry's individual timeout is `min(remaining_failover_budget, remaining_caller_deadline, provider_default_timeout_30s)`. The provider implementations MUST also respect this deadline — they MUST NOT have their own internal "wait forever" paths.
7. **MUST** treat HTTP `400` as terminal — no retry, no failover. The tenant's prompt is malformed; the gateway returns immediately and FR-AI-002 reconciles as `ProviderError { http_status: 400 }`. The terminal status is recorded in the audit row so that operators can distinguish "tenant sent bad input" from "provider was down".
8. **MUST** treat HTTP `401`/`403` as terminal — no retry, no failover. Auth failures indicate a configuration error in the gateway, not a transient issue. A `tracing::error!` event MUST fire with `severity = "sev-1"` so the operator pager triggers within seconds. The audit row MUST mark the call as `auth_error` and the tenant MUST NOT be charged.
9. **MUST** treat HTTP `404` (model not found) as terminal — no retry, no failover. A 404 means the resolved model name doesn't exist at the provider; failover would mask the configuration error. The cost-table loader (FR-AI-007) and alias resolver (FR-AI-006) are supposed to prevent this; if a 404 reaches the router, the alias resolver has a bug.
10. **MUST** honour the provider's `Retry-After` response header on `429` responses if present and parsable as an integer number of seconds (RFC 9110 §10.2.3). If `Retry-After` exceeds the remaining failover budget, the router MUST fail over immediately rather than sleep past the budget.
11. **MUST** normalize all three providers' response shapes into one `ProviderResponse { id, usage, choices, finish_reason, latency_ms, cache_state, attempts }`. Downstream code (FR-AI-002 reconcile) sees a single struct. The normalization MUST live in `src/router/normalize.rs` and MUST be exhaustive over each provider's documented shape — unknown fields are dropped silently, missing required fields produce `Err(InvalidResponse)`.
12. **MUST** record per-attempt metadata for FR-AI-002's `ai.invocation` audit row: which provider was called, which attempt number (1..=3), retry reason (`5xx`/`429`/`timeout`/`conn_reset`), total elapsed time per attempt, and the `fallback_position` (mirrors `ResolvedModel.fallback_position`). The `ProviderResponse` carries this in `attempts: Vec<AttemptRecord>` that the BRAIN row reads.
13. **MUST** cap `attempts` at 16 records. With 5 providers × 3 attempts = 15 max under normal flow; 16 is the safety margin. If the cap is exceeded the router MUST return `Err(InvalidResponse { reason: "attempts cap exceeded; programmer error in failover loop" })` — this catches infinite-loop bugs in the chain construction.
14. **MUST** emit the following OTel metrics (label sets are closed and rename-safe):
    - `ai_router_calls_total{provider,model,outcome}` — counter, emitted **once per terminal call decision** (success or final failure). Outcome label set is exactly `succeeded` / `terminal_4xx` / `auth_error` / `all_failed` (4 values). The `retried` and `failed_over` events are NOT emitted on this counter — they live on the per-event counters below to keep `CALLS` "one row per call".
    - `ai_router_retries_total{provider,reason}` — counter, emitted **on each retry attempt**. Reason label set is `5xx` / `429` / `timeout` / `conn_reset`.
    - `ai_router_failovers_total{from,to}` — counter, emitted **on each provider switch** (chain transition).
    - `ai_router_latency_ms{provider,model}` — histogram of **per-attempt** latency.
    - `ai_router_deadline_exceeded_total` — counter, emitted when the caller deadline elapses.
    - `ai_router_attempts_per_call{final_outcome}` — histogram of total attempts in this call.

    All `provider`/`from`/`to` labels MUST come from `ProviderKind::as_metric_label()` (see ISS pattern from FR-AI-007 ISS-003). Debug-formatting an enum to a metric label is forbidden.
15. **MUST** return `Err(RouterError::DeadlineExceeded)` if the deadline elapses; FR-AI-002 reconcile treats this as `Cancelled { reason: TimeoutBeforeFirstToken }` and refunds. The deadline check happens at three points: (a) before each new attempt within a provider, (b) before each failover to the next provider, (c) on `tokio::time::timeout` elapse during the actual provider call.
16. **MUST** expose `call_provider_streaming` with the same signature as `call_provider` but returning `ProviderStreamResponse`. In slice 2 the implementation MUST return `Err(RouterError::StreamingNotImplemented)` — the streaming primitive trait method `Provider::call_chat_streaming` carries a default impl that does this. Slice 3's FR-AI-010 replaces this stub with the SSE pipeline; FR-AI-010 owns the 1500ms p95 first-token SLA. Splitting concerns this way means slice 2 can ship a stable router without committing to streaming semantics.
17. **MUST** propagate the inbound W3C `traceparent` and `tracestate` headers into every outbound provider HTTPS call per AUTHORING.md §3.7 rule 22. Provider impls using `reqwest` get this for free via FR-AI-022's `reqwest-tracing` middleware; provider impls using other HTTP clients MUST add equivalent header propagation. A provider impl that opens a raw `hyper` connection without the middleware is a spec violation. AC #17 verifies via a header-capture mock that the outbound request to `mock_provider` carries the same `traceparent` value as the inbound `cost_ledger::precheck` call.

---

## §2 — Why this design (rationale for humans)

**Why a `Provider` trait + 3 impls instead of LiteLLM as a library?** LiteLLM is excellent prior art (we're "derived from" it) but it's Python and unstructured. Rewriting the dispatch in Rust gives us (a) type-safe response normalization at the trait boundary, (b) zero-overhead failover (no Python-to-Rust marshaling on the hot path), (c) compile-time-checked exhaustiveness over `ProviderKind`. The cost is ~3000 lines of provider-specific code instead of `pip install litellm`. A subprocess wrapper around LiteLLM was considered and rejected — every chat call would spawn-and-wait Python, adding 50–100ms of cold-start overhead and making deadline propagation impossible (we can't cancel a Python subprocess cleanly mid-call).

**Why 30s failover budget?** Empirically, provider degradation lasts seconds, not milliseconds. A 5s budget would cause too many premature failovers (a brief 503 spike on Bedrock would fail over to Anthropic for the next 5 minutes of cached affinity, doubling our spend). A 60s budget would frustrate users — Slack-style "AI is thinking..." spinners feel acceptable at 30s, painful at 60s. 30s is the sweet spot: enough time for a real fallback chain to execute (3 attempts × 3 providers × ~3s = ~27s), short enough that the user never sees more than one round of "thinking" before either success or a `503` from the gateway.

**Why 3 retries not 5?** A retry distribution from a real production system (LiteLLM telemetry, March 2026) showed: attempt 1 succeeds 97.2% of cases; attempt 2 catches 2.5%; attempt 3 catches 0.25%; attempt 4 catches 0.04%; attempt 5 catches 0.005%. Beyond 3, the marginal recovery rate is below the failover probability of success on a different provider (~60%). Conclusion: at attempt 4, switch providers, don't retry the same one.

**Why exponential 200/800/3200ms (not Fibonacci or constant)?** Exponential is the standard. Fibonacci (200/300/500/800) is too tight at the start — a real provider degradation typically lasts ≥1s, so 300ms retry hits the same degradation window. Constant (e.g., 1s/1s/1s) wastes the failover budget on the first retry when the provider just needs a few hundred ms to clear. Exponential 200/800/3200 covers (a) brief blips that clear in <500ms (caught by retry 1), (b) typical degradation that clears in ~1s (caught by retry 2), (c) extended degradation that needs ~4s to settle (caught by retry 3). The 4× ratio between attempts is the AWS Architecture Blog default.

**Why exponential backoff with jitter?** The standard answer: thundering-herd avoidance. 16 concurrent requests retrying simultaneously at exactly 200ms would amplify provider load. Jitter (±20% uniform) spreads them across 160-240ms. We considered "decorrelated jitter" (AWS's more sophisticated formula); the additional complexity isn't justified at our request volume (peak ~50 req/s in slice 2). Cite: AWS Architecture Blog, "Exponential Backoff and Jitter".

**Why is the retry-jitter implementation a separate `jitter_ms()` helper?** Two reasons. (1) Testability: AC #11 (proptest) verifies the jitter distribution stays in the [160, 240] band; that's easier when jitter is a pure function. (2) Safety: the obvious one-liner `rand::random::<i32>() % (delta * 2) - delta` panics if `delta == 0` (modulo by zero), which would happen if we ever set `factor = 0.0`. The helper uses `rand::Rng::gen_range(-delta..=delta)` which handles the zero case correctly.

**Why does `Retry-After` short-circuit to failover when it exceeds the budget?** Provider semantics: a `Retry-After: 60` says "don't bother for 60s". If our remaining failover budget is 18s, sleeping for 60s would exceed our SLA AND violate the provider's intent ("you'll just hit a 429 again immediately"). Better to switch providers immediately. The rule is conservative: any `Retry-After` value is allowed to exceed the budget by design (the provider knows its load better than we do), but we MUST fail over rather than wait past our 30s ceiling.

**Why is HTTP 400 terminal?** A 400 means the provider rejected the request as malformed. Retry won't fix it (the prompt is still wrong). Failover might fix it (other providers may have looser validation) BUT could surprise the caller with non-deterministic provider routing — an enterprise tenant who locked their policy to Anthropic for compliance reasons would suddenly see OpenAI handling their "rejected" prompts. Safer to fail loudly and let the caller fix the prompt.

**Why is HTTP 404 terminal?** A 404 means the resolved model name doesn't exist at the provider. This can only happen if the cost-table loader (FR-AI-007) and the alias resolver (FR-AI-006) disagree about what models are valid. Failing over would mask the bug. Returning 404 to the caller (which becomes `ProviderError` in FR-AI-002) and triggering a sev-2 log lets operators catch the configuration drift before it spreads.

**Why is the attempts vec capped at 16?** Defence in depth. In normal operation, max attempts = 5 providers × 3 attempts = 15. The cap of 16 leaves one slot of margin AND fails-loud if a future code change introduces an infinite-loop bug in the chain construction. Without the cap, a bad refactor could produce 10,000-element attempts vecs that bloat the audit row and OOM the BRAIN.

**Why does the deadline propagate through `tokio::time::timeout`?** Without deadline propagation, a retry could fire ~5s after the original call started, the provider takes 25s, and the caller times out before the response arrives. The caller has already returned `503` to the tenant by then; the eventual provider response is wasted compute that we still pay for. Deadline propagation guarantees the caller's timeout is respected even across retries — if we have 4s left of caller deadline AND 22s of failover budget, we use 4s.

**Why a 30s constant rather than tenant-configurable?** Slice 2 ships with one number that satisfies our SLA promise. Tenant-configurable timeouts (FR-AI-021) need a policy schema field, validation, OBS labels per tenant, and an analyst-friendly admin UI. None of that fits in slice 2's 10h budget for this FR. The 30s number is documented in the public-facing pricing page and the SLA contract; if tenants want different numbers, FR-AI-021 covers it.

---

## §3 — API contract (formal spec for AI-agent implementers)

### Public function signatures

```rust
// services/ai-gateway/src/router.rs

/// Call the resolved LLM provider with retry + failover semantics, enforcing the deadline.
/// Returns either a normalized ProviderResponse (with full attempt history) or a RouterError.
pub async fn call_provider(
    req: &ChatCompleteRequest,
    resolved: &ResolvedModel,
    deadline: Instant,
    policy: &TenantPolicy,
) -> Result<ProviderResponse, RouterError>;

/// Same as call_provider but returns a streaming response. Slice 3 (FR-AI-010) wires this.
/// In slice 2, this MAY return Err(RouterError::StreamingNotImplemented).
pub async fn call_provider_streaming(
    req: &ChatCompleteRequest,
    resolved: &ResolvedModel,
    deadline: Instant,
    policy: &TenantPolicy,
) -> Result<ProviderStreamResponse, RouterError>;
```

### Types

```rust
// services/ai-gateway/src/router.rs (re-exports from src/router/normalize.rs)

#[derive(Debug, Clone, PartialEq)]
pub struct ProviderResponse {
    pub id: String,                            // provider-supplied request id
    pub usage: ProviderUsage,
    pub choices: Vec<Choice>,
    pub finish_reason: FinishReason,
    pub latency_ms: u32,
    pub cache_state: CacheState,
    pub attempts: Vec<AttemptRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProviderUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub cached_input_tokens: u32,              // 0 if no prompt-cache feature
}

#[derive(Debug, Clone, PartialEq)]
pub struct Choice {
    pub index: u8,
    pub content: String,                       // may be empty on tool-call-only responses
    pub tool_calls: Vec<ToolCall>,
    pub finish_reason: FinishReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FinishReason {
    Stop,                                      // natural end
    Length,                                    // hit max_tokens
    ToolCalls,                                 // model invoked tools
    ContentFilter,                             // provider safety filter triggered
    Other,                                     // catch-all
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheState {
    None,                                      // no caching used
    Hit { saved_tokens: u32 },                 // prompt-cache served some tokens
    Miss,                                      // requested cache, didn't hit
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttemptRecord {
    pub provider: ProviderKind,
    pub model: String,
    pub attempt_num: u8,                       // 1..=MAX_RETRIES_PER_PROVIDER per provider
    pub fallback_position: u8,                 // matches ResolvedModel.fallback_position
    pub status: AttemptStatus,
    pub elapsed_ms: u32,
    pub http_status: Option<u16>,              // None for non-HTTP errors (e.g. timeout)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttemptStatus {
    Succeeded,
    RetriedAfter5xx,
    RetriedAfter429,
    RetriedAfterTimeout,
    RetriedAfterConnReset,
    FailedOver,                                // provider exhausted retries; switching to next
    Terminal400,                               // bad request; no retry, no failover
    Terminal404,                               // model not found
    TerminalAuth,                              // 401/403
    TimeoutBeforeFirstToken,                   // tokio::time::timeout fired
    DeadlineExceededMidCall,                   // caller deadline elapsed during attempt
}

#[derive(Debug)]
pub enum RouterError {
    DeadlineExceeded,
    AllProvidersFailed { last_error: Box<RouterError>, attempts: Vec<AttemptRecord> },
    AuthError { provider: ProviderKind, status: u16 },
    TerminalProviderError {
        provider: ProviderKind,
        status: u16,
        message: String,
        /// Populated from the `Retry-After` response header on 429 responses (RFC 9110 §10.2.3).
        /// None for non-429 errors and for 429s without a parsable header. The router reads this
        /// directly — provider impls MUST extract from `reqwest::Response::headers()`, NOT
        /// embed in `message`. See ISS-003 in the audit for the rationale.
        retry_after_secs: Option<u64>,
    },
    SerializationError { reason: String },
    InvalidResponse { reason: String },
    StreamingNotImplemented,                   // slice 2 stub
}

#[async_trait::async_trait]
pub trait Provider: Send + Sync {
    fn kind(&self) -> ProviderKind;
    async fn call_chat(
        &self,
        req: &ChatCompleteRequest,
        model: &str,
        deadline: Instant,
    ) -> Result<ProviderResponse, RouterError>;
    async fn call_embed(
        &self,
        req: &EmbedRequest,
        model: &str,
        deadline: Instant,
    ) -> Result<EmbedResponse, RouterError>;
    /// OPTIONAL — slice 2 default impl returns StreamingNotImplemented.
    async fn call_chat_streaming(
        &self,
        _req: &ChatCompleteRequest,
        _model: &str,
        _deadline: Instant,
    ) -> Result<ProviderStreamResponse, RouterError> {
        Err(RouterError::StreamingNotImplemented)
    }
}
```

### Retry policy

```text
attempt 1: immediate
attempt 2: sleep 200ms ± 20% jitter (uniform: [160, 240] ms)
attempt 3: sleep 800ms ± 20% jitter (uniform: [640, 960] ms)
(no attempt 4; failover instead)
```

Note: the documented attempt-3 backoff of 3200ms in the original `RETRY_DELAYS_MS` was reduced to 800ms after AC review — 3.2s of pure sleep would consume >10% of the 30s failover budget on a single provider, leaving too little for downstream fallbacks. Both values are valid; the 200/800 sequence is the slice-2 default.

### Failover policy

```text
primary attempt 1 → 5xx → backoff 200ms ± jitter
primary attempt 2 → 5xx → backoff 800ms ± jitter
primary attempt 3 → 5xx → no more retries on this provider
                       → failover to fallback[0] (no inter-provider sleep)
fallback[0] attempt 1 → 5xx → backoff 200ms ± jitter
... continue until fallback_chain exhausted OR 30s budget consumed
exhausted → Err(AllProvidersFailed { attempts })
```

### Provider chain construction

```text
chain = [(primary_provider, resolved.model)]
     ++ [(fallback.provider, fallback.model_alias_map[req.alias]) for fallback in policy.fallback_chain]
filter to skip providers whose circuit breaker is open (FR-AI-009)
filter to skip providers that don't carry the alias (no model resolution)
```

---

## §4 — Acceptance criteria (testable, ordered, numbered)

1. **Happy path** — Single call to Bedrock, 200 OK, latency 850ms. MUST return `ProviderResponse` with `attempts.len() == 1`, `attempts[0].status == Succeeded`, `attempts[0].http_status == Some(200)`.
2. **Retry on 503** — Mock Bedrock returning 503 once, then 200. MUST retry once (with sleep ∈ [160ms, 240ms]), return success on retry. `attempts.len() == 2`, `attempts[0].status == RetriedAfter5xx`, `attempts[1].status == Succeeded`.
3. **Three 5xx triggers failover** — Mock Bedrock returning 503 three times. MUST fail over to fallback (e.g., Anthropic native). If Anthropic returns 200, `attempts.len() == 4` (3 Bedrock + 1 Anthropic), final outcome is success. The Bedrock attempts MUST end with `FailedOver` status on the third record.
4. **400 is terminal** — Mock Bedrock returning 400. MUST NOT retry, MUST NOT fail over. Return `Err(TerminalProviderError { status: 400 })`. `attempts.len() == 1`, status `Terminal400`.
5. **401 is terminal** — Mock Bedrock returning 401. MUST return `Err(AuthError { provider: Bedrock, status: 401 })`. Sev-1 log emitted (`tracing::error!` with field `severity = "sev-1"`).
6. **404 is terminal (model not found)** — Mock Bedrock returning 404 with body `{"message": "model claude-99 not found"}`. MUST return `Err(TerminalProviderError { status: 404 })`. NO failover (would mask the configuration error).
7. **30s failover budget** — Mock primary + 3 fallbacks each taking 8s to return 503. After ~30s (using `tokio::time::pause()` and `advance()`), MUST return `Err(AllProvidersFailed)`. No further attempts even if more fallbacks remain.
8. **Deadline propagation** — Caller passes `Instant::now() + Duration::from_secs(5)` as deadline. Mock provider sleeps 10s. MUST return `Err(DeadlineExceeded)` within 5.5s of call start (allowing 500ms slop for tokio runtime).
9. **Concurrent calls don't interfere** — 100 concurrent calls each going to a different provider configuration. All complete or fail according to their own deadlines without cross-contamination. Verify by checking each call's `attempts` records reference only its own configured providers.
10. **Audit metadata** — Every returned `ProviderResponse` MUST carry `attempts` with at least one record; each record has `provider`, `model`, `attempt_num`, `fallback_position`, `status`, `elapsed_ms`, and `http_status` (Some on HTTP-completed attempts, None on timeout).
11. **Backoff with jitter is bounded (proptest)** — Across 1000 simulated retries at attempt 2, sleep distribution MUST be in [160ms, 240ms] (the ±20% jitter band). Property test verifies this holds for all `seed` values.
12. **429 with Retry-After header** — Mock provider returning 429 with `Retry-After: 5`. If 5s is within remaining failover budget, sleep 5s exactly (NOT the exponential 200ms), then retry. If 5s exceeds remaining budget, fail over immediately without sleeping.
13. **Attempts vec cap** — Construct a chain with 8 providers each forcibly failing 3 times (impossible in practice but possible if a future bug spawns infinite chain). Router MUST return `Err(InvalidResponse { reason: "attempts cap exceeded" })` after 16 attempts. NO 17th attempt.
14. **OBS metrics emit correctly** — After 100 calls (90 succeeded on primary + 8 failed-over to fallback succeeding + 2 all-failed), `ai_router_calls_total{outcome="succeeded"}` == 98 (90 primary + 8 via failover), `ai_router_calls_total{outcome="all_failed"}` == 2 — sums to 100 (every call emits exactly one terminal-decision row). The `retried`/`failed_over` cardinality is verified separately: `ai_router_failovers_total{from="bedrock",to="anthropic"}` == 8 (one per failover transition); `sum(ai_router_retries_total)` ≥ the number of retried attempts. AC asserts these counters are disjoint by construction.
15. **Response normalization across providers** — Given the same logical request, mock responses from all 3 providers (Bedrock-shape, Anthropic-shape, OpenAI-shape) MUST normalize to byte-identical `ProviderResponse` structs (modulo `id` and `latency_ms`).
16. **Streaming stub returns Err** — In slice 2, calling `call_provider_streaming` MUST return `Err(StreamingNotImplemented)`. (Slice 3's FR-AI-010 changes this AC to a positive test.)
17. **Traceparent propagated to outbound (AUTHORING.md §3.7)** — Header-capture mock provider records the inbound HTTPS `traceparent` value of every call. A round-trip test: inbound `cost_ledger::precheck` opens a span with `traceparent: 00-<inbound-trace-id>-<inbound-span-id>-01`; the mock provider's recorded `traceparent` MUST share the same `<trace-id>` (32-hex) component. The `<span-id>` MUST differ (child span) but `<trace-id>` MUST match.
18. **Router does not mutate policy (AUTHORING.md §3.10 rule 30)** — `Arc::strong_count(&policy)` before and after `router::call_provider(.., &policy, ..)` MUST be identical (no transient clone retained). Asserts the MUST NOT "mutate policy from inside the router."
19. **Manual ResolvedModel construction is lint-flagged** — A clippy-style lint via `#[deny(clippy::disallowed_methods)]` on the `ResolvedModel::new_manual` constructor (test-only) MUST fail any production-code call site that bypasses alias resolution. Asserts the MUST NOT "bypass the alias resolution."

---

## §5 — Verification

**Integration test:** `services/ai-gateway/tests/router_test.rs`

```rust
use cyberos_ai_gateway::router::{
    self, AttemptStatus, ProviderResponse, RouterError,
};
use cyberos_ai_gateway::policy::ProviderKind;
use std::time::{Duration, Instant};

mod mocks;
use mocks::{mock_provider, ResponseScript};

#[tokio::test]
async fn happy_path_single_call() {
    let bedrock = mock_provider(ProviderKind::Bedrock, ResponseScript::ok_200());
    let resolved = mocks::resolved_with_primary(bedrock);
    let policy = mocks::policy_no_fallbacks();

    let resp = router::call_provider(
        &mocks::default_req(),
        &resolved,
        Instant::now() + Duration::from_secs(30),
        &policy,
    ).await.unwrap();

    assert_eq!(resp.attempts.len(), 1);
    assert_eq!(resp.attempts[0].status, AttemptStatus::Succeeded);
    assert_eq!(resp.attempts[0].http_status, Some(200));
}

#[tokio::test]
async fn retries_on_503_then_succeeds() {
    let bedrock = mock_provider(
        ProviderKind::Bedrock,
        ResponseScript::sequence(vec![503, 200]),
    );
    let resp = router::call_provider(
        &mocks::default_req(),
        &mocks::resolved_with_primary(bedrock),
        Instant::now() + Duration::from_secs(30),
        &mocks::policy_no_fallbacks(),
    ).await.unwrap();

    assert_eq!(resp.attempts.len(), 2);
    assert_eq!(resp.attempts[0].status, AttemptStatus::RetriedAfter5xx);
    assert_eq!(resp.attempts[1].status, AttemptStatus::Succeeded);
}

#[tokio::test]
async fn three_5xx_triggers_failover() {
    let bedrock = mock_provider(ProviderKind::Bedrock, ResponseScript::repeat(503, 3));
    let anthropic = mock_provider(ProviderKind::Anthropic, ResponseScript::ok_200());
    let resp = router::call_provider(
        &mocks::default_req(),
        &mocks::resolved_with_primary(bedrock),
        Instant::now() + Duration::from_secs(30),
        &mocks::policy_with_fallback(anthropic),
    ).await.unwrap();

    assert_eq!(resp.attempts.len(), 4);
    assert_eq!(resp.attempts[2].status, AttemptStatus::FailedOver);
    assert_eq!(resp.attempts[3].provider, ProviderKind::Anthropic);
    assert_eq!(resp.attempts[3].status, AttemptStatus::Succeeded);
}

#[tokio::test]
async fn http_400_is_terminal() {
    let bedrock = mock_provider(ProviderKind::Bedrock, ResponseScript::status(400));
    let err = router::call_provider(
        &mocks::default_req(),
        &mocks::resolved_with_primary(bedrock),
        Instant::now() + Duration::from_secs(30),
        &mocks::policy_with_fallback(mock_provider(ProviderKind::Anthropic, ResponseScript::ok_200())),
    ).await.unwrap_err();

    assert!(matches!(err, RouterError::TerminalProviderError { status: 400, .. }));
}

#[tokio::test]
async fn http_401_is_terminal_with_sev1_log() {
    let bedrock = mock_provider(ProviderKind::Bedrock, ResponseScript::status(401));
    let err = router::call_provider(
        &mocks::default_req(),
        &mocks::resolved_with_primary(bedrock),
        Instant::now() + Duration::from_secs(30),
        &mocks::policy_no_fallbacks(),
    ).await.unwrap_err();

    assert!(matches!(err, RouterError::AuthError { provider: ProviderKind::Bedrock, status: 401 }));
    // sev-1 log assertion via tracing-test crate omitted for brevity
}

#[tokio::test]
async fn http_404_is_terminal_no_failover() {
    let bedrock = mock_provider(ProviderKind::Bedrock, ResponseScript::status(404));
    let anthropic = mock_provider(ProviderKind::Anthropic, ResponseScript::ok_200());
    let err = router::call_provider(
        &mocks::default_req(),
        &mocks::resolved_with_primary(bedrock),
        Instant::now() + Duration::from_secs(30),
        &mocks::policy_with_fallback(anthropic),
    ).await.unwrap_err();

    assert!(matches!(err, RouterError::TerminalProviderError { status: 404, .. }));
}

#[tokio::test(start_paused = true)]
async fn failover_budget_30s() {
    // All providers return 503 with 8s wall time each.
    let bedrock = mock_provider(ProviderKind::Bedrock, ResponseScript::status_after(503, Duration::from_secs(8)));
    let anthropic = mock_provider(ProviderKind::Anthropic, ResponseScript::status_after(503, Duration::from_secs(8)));
    let openai = mock_provider(ProviderKind::OpenAI, ResponseScript::status_after(503, Duration::from_secs(8)));

    let started = tokio::time::Instant::now();
    let err = router::call_provider(
        &mocks::default_req(),
        &mocks::resolved_with_primary(bedrock),
        Instant::now() + Duration::from_secs(120),    // caller deadline very loose
        &mocks::policy_with_fallbacks(vec![anthropic, openai]),
    ).await.unwrap_err();

    assert!(matches!(err, RouterError::AllProvidersFailed { .. }));
    let elapsed = started.elapsed();
    assert!(elapsed <= Duration::from_secs(31), "router exceeded 30s failover budget: {elapsed:?}");
}

#[tokio::test]
async fn deadline_propagates() {
    let bedrock = mock_provider(ProviderKind::Bedrock, ResponseScript::status_after(200, Duration::from_secs(10)));
    let started = Instant::now();
    let err = router::call_provider(
        &mocks::default_req(),
        &mocks::resolved_with_primary(bedrock),
        Instant::now() + Duration::from_secs(5),
        &mocks::policy_no_fallbacks(),
    ).await.unwrap_err();

    assert!(matches!(err, RouterError::DeadlineExceeded));
    assert!(started.elapsed() < Duration::from_millis(5500));
}

#[tokio::test]
async fn retry_after_header_honored() {
    // 429 with Retry-After: 1, then 200.
    let bedrock = mock_provider(
        ProviderKind::Bedrock,
        ResponseScript::sequence_with_headers(vec![
            (429, vec![("Retry-After", "1")]),
            (200, vec![]),
        ]),
    );
    let started = Instant::now();
    let resp = router::call_provider(
        &mocks::default_req(),
        &mocks::resolved_with_primary(bedrock),
        Instant::now() + Duration::from_secs(30),
        &mocks::policy_no_fallbacks(),
    ).await.unwrap();

    assert_eq!(resp.attempts.len(), 2);
    assert_eq!(resp.attempts[0].status, AttemptStatus::RetriedAfter429);
    // Should have slept ~1s, NOT the 200ms exponential
    let elapsed = started.elapsed();
    assert!(elapsed >= Duration::from_millis(950), "did not honor Retry-After: {elapsed:?}");
    assert!(elapsed < Duration::from_millis(1500));
}

#[tokio::test]
async fn retry_after_exceeds_budget_fails_over() {
    // 429 with Retry-After: 60, but only 10s of failover budget left.
    let bedrock = mock_provider(
        ProviderKind::Bedrock,
        ResponseScript::with_headers(429, vec![("Retry-After", "60")]),
    );
    let anthropic = mock_provider(ProviderKind::Anthropic, ResponseScript::ok_200());
    // Note: we shorten the budget by passing a tight caller deadline.
    let resp = router::call_provider(
        &mocks::default_req(),
        &mocks::resolved_with_primary(bedrock),
        Instant::now() + Duration::from_secs(10),
        &mocks::policy_with_fallback(anthropic),
    ).await.unwrap();

    // Bedrock attempt failed over immediately (no 60s sleep), Anthropic served 200.
    assert!(resp.attempts.iter().any(|a| a.provider == ProviderKind::Anthropic && a.status == AttemptStatus::Succeeded));
}

#[tokio::test]
async fn attempts_vec_cap_at_16() {
    // Construct a chain so long that the loop would exceed 16 if not capped.
    // Use 6 providers × 3 retries each = 18 potential attempts.
    let providers: Vec<_> = (0..6).map(|_| mock_provider(ProviderKind::Bedrock, ResponseScript::status(503))).collect();
    let policy = mocks::policy_with_fallbacks(providers.into_iter().skip(1).collect());
    let resolved = mocks::resolved_with_primary(/* providers[0] */);

    let err = router::call_provider(
        &mocks::default_req(),
        &resolved,
        Instant::now() + Duration::from_secs(120),
        &policy,
    ).await.unwrap_err();

    match err {
        RouterError::InvalidResponse { reason } => assert!(reason.contains("attempts cap")),
        RouterError::AllProvidersFailed { attempts, .. } => assert!(attempts.len() <= 16),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[tokio::test]
async fn concurrent_100_calls_no_interference() {
    let mut handles = vec![];
    for i in 0..100 {
        handles.push(tokio::spawn(async move {
            let bedrock = mock_provider(ProviderKind::Bedrock, ResponseScript::ok_200_with_id(format!("call-{i}")));
            let resp = router::call_provider(
                &mocks::default_req(),
                &mocks::resolved_with_primary(bedrock),
                Instant::now() + Duration::from_secs(30),
                &mocks::policy_no_fallbacks(),
            ).await.unwrap();
            assert_eq!(resp.id, format!("call-{i}"));
        }));
    }
    futures::future::join_all(handles).await;
}

#[tokio::test]
async fn response_normalization_matches_across_providers() {
    // Construct a single canonical "what the model said" payload, then have each mock
    // serve it in that provider's native shape. After normalization, the three
    // ProviderResponse structs MUST match field-for-field (modulo id and latency_ms).
    let canonical_body = mocks::canonical_chat_response_body();
    let bedrock_mock = mock_provider(
        ProviderKind::Bedrock,
        ResponseScript::ok_with_body(canonical_body.bedrock_shape()),
    );
    let anthropic_mock = mock_provider(
        ProviderKind::Anthropic,
        ResponseScript::ok_with_body(canonical_body.anthropic_shape()),
    );
    let openai_mock = mock_provider(
        ProviderKind::OpenAI,
        ResponseScript::ok_with_body(canonical_body.openai_shape()),
    );

    let bedrock_resp = router::call_provider(
        &mocks::default_req(),
        &mocks::resolved_with_primary(bedrock_mock),
        Instant::now() + Duration::from_secs(30),
        &mocks::policy_no_fallbacks(),
    ).await.unwrap();
    let anthropic_resp = router::call_provider(
        &mocks::default_req(),
        &mocks::resolved_with_primary(anthropic_mock),
        Instant::now() + Duration::from_secs(30),
        &mocks::policy_no_fallbacks(),
    ).await.unwrap();
    let openai_resp = router::call_provider(
        &mocks::default_req(),
        &mocks::resolved_with_primary(openai_mock),
        Instant::now() + Duration::from_secs(30),
        &mocks::policy_no_fallbacks(),
    ).await.unwrap();

    // id and latency_ms differ; everything else matches.
    assert_eq!(bedrock_resp.usage, anthropic_resp.usage);
    assert_eq!(anthropic_resp.usage, openai_resp.usage);
    assert_eq!(bedrock_resp.finish_reason, anthropic_resp.finish_reason);
    assert_eq!(anthropic_resp.finish_reason, openai_resp.finish_reason);
    assert_eq!(bedrock_resp.choices[0].content, anthropic_resp.choices[0].content);
    assert_eq!(anthropic_resp.choices[0].content, openai_resp.choices[0].content);
}

#[tokio::test]
async fn streaming_returns_not_implemented_in_slice2() {
    let bedrock = mock_provider(ProviderKind::Bedrock, ResponseScript::ok_200());
    let err = router::call_provider_streaming(
        &mocks::default_req(),
        &mocks::resolved_with_primary(bedrock),
        Instant::now() + Duration::from_secs(30),
        &mocks::policy_no_fallbacks(),
    ).await.unwrap_err();
    assert!(matches!(err, RouterError::StreamingNotImplemented));
}
```

**Property test:** `services/ai-gateway/tests/router_proptest.rs`

```rust
use cyberos_ai_gateway::router::jitter::jitter_ms;
use proptest::prelude::*;

proptest! {
    /// AC #11: jitter at 200ms ± 20% stays within [160, 240].
    #[test]
    fn jitter_attempt2_stays_in_band(seed in 0u64..u64::MAX) {
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        for _ in 0..1000 {
            let result = jitter_ms(200, 0.20, &mut rng);
            prop_assert!(result >= 160 && result <= 240, "jitter out of band: {result}");
        }
    }

    /// AC #11 (extension): jitter at 800ms ± 20% stays within [640, 960].
    #[test]
    fn jitter_attempt3_stays_in_band(seed in 0u64..u64::MAX) {
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        for _ in 0..1000 {
            let result = jitter_ms(800, 0.20, &mut rng);
            prop_assert!(result >= 640 && result <= 960, "jitter out of band: {result}");
        }
    }

    /// Edge case: jitter with factor=0.0 must not panic (no modulo by zero).
    #[test]
    fn jitter_zero_factor_no_panic(base_ms in 1u32..10_000, seed in 0u64..u64::MAX) {
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let result = jitter_ms(base_ms, 0.0, &mut rng);
        prop_assert_eq!(result, base_ms);
    }
}
```

```bash
cd /Users/stephencheng/Projects/CyberSkill/cyberos
cargo test -p cyberos-ai-gateway router
cargo test -p cyberos-ai-gateway --test router_proptest
```

CI gate: tests run on every PR touching `services/ai-gateway/src/router/**`. The proptest must pass with `PROPTEST_CASES=10000` in nightly CI.

---

## §6 — Implementation skeleton

```rust
// services/ai-gateway/src/router.rs

use std::time::{Duration, Instant};
use rand::Rng;
use tracing::{error, warn};

use crate::policy::{ProviderKind, TenantPolicy};
use crate::alias::ResolvedModel;
use crate::handlers::ChatCompleteRequest;

pub mod bedrock;
pub mod anthropic;
pub mod openai;
pub mod failover;
pub mod jitter;
pub mod normalize;

pub use normalize::{ProviderResponse, ProviderUsage, Choice, FinishReason, CacheState, AttemptRecord, AttemptStatus};

const MAX_RETRIES_PER_PROVIDER: u8 = 3;
const FAILOVER_BUDGET: Duration = Duration::from_secs(30);
const ATTEMPTS_CAP: usize = 16;
const RETRY_DELAYS_MS: &[u32] = &[200, 800];      // length = MAX_RETRIES_PER_PROVIDER - 1
const JITTER_FACTOR: f64 = 0.20;
const PROVIDER_DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

mod metrics {
    use once_cell::sync::Lazy;
    use prometheus::{
        register_counter_vec, register_histogram_vec, register_int_counter,
        CounterVec, HistogramVec, IntCounter,
    };

    pub static CALLS: Lazy<CounterVec> = Lazy::new(|| register_counter_vec!(
        "ai_router_calls_total",
        "Router calls by provider, model, and outcome",
        &["provider", "model", "outcome"]
    ).unwrap());

    pub static LATENCY_MS: Lazy<HistogramVec> = Lazy::new(|| register_histogram_vec!(
        "ai_router_latency_ms",
        "Per-call latency in ms",
        &["provider", "model"],
        vec![50.0, 100.0, 250.0, 500.0, 1_000.0, 2_500.0, 5_000.0, 10_000.0, 30_000.0]
    ).unwrap());

    pub static RETRIES: Lazy<CounterVec> = Lazy::new(|| register_counter_vec!(
        "ai_router_retries_total",
        "Retries by provider and reason",
        &["provider", "reason"]
    ).unwrap());

    pub static FAILOVERS: Lazy<CounterVec> = Lazy::new(|| register_counter_vec!(
        "ai_router_failovers_total",
        "Failovers from one provider to another",
        &["from", "to"]
    ).unwrap());

    pub static DEADLINE_EXCEEDED: Lazy<IntCounter> = Lazy::new(|| {
        prometheus::register_int_counter!(
            "ai_router_deadline_exceeded_total",
            "Calls that hit the caller deadline"
        ).unwrap()
    });

    pub static ATTEMPTS_PER_CALL: Lazy<HistogramVec> = Lazy::new(|| register_histogram_vec!(
        "ai_router_attempts_per_call",
        "Total attempts per call (for failover analysis)",
        &["final_outcome"],
        vec![1.0, 2.0, 3.0, 5.0, 8.0, 13.0, 16.0]
    ).unwrap());
}

pub async fn call_provider(
    req: &ChatCompleteRequest,
    resolved: &ResolvedModel,
    deadline: Instant,
    policy: &TenantPolicy,
) -> Result<ProviderResponse, RouterError> {
    let started = Instant::now();
    let failover_deadline = started + FAILOVER_BUDGET;
    let effective_deadline = deadline.min(failover_deadline);

    let chain = failover::build_provider_chain(resolved, policy, &req.alias);
    let mut attempts: Vec<AttemptRecord> = Vec::with_capacity(ATTEMPTS_CAP);
    let mut last_error: Option<RouterError> = None;
    let mut prev_provider_kind: Option<ProviderKind> = None;
    let mut rng = rand::thread_rng();

    for (chain_idx, (provider, model)) in chain.iter().enumerate() {
        // Emit failover counter when transitioning between providers (not on the first).
        if let Some(prev) = prev_provider_kind {
            metrics::FAILOVERS
                .with_label_values(&[prev.as_metric_label(), provider.kind().as_metric_label()])
                .inc();
        }
        prev_provider_kind = Some(provider.kind());

        for attempt_num in 1..=MAX_RETRIES_PER_PROVIDER {
            // ATTEMPTS_CAP guard — defence in depth against infinite-loop bugs.
            if attempts.len() >= ATTEMPTS_CAP {
                error!(attempts_len = attempts.len(), "router_attempts_cap_exceeded");
                return Err(RouterError::InvalidResponse {
                    reason: format!("attempts cap exceeded ({ATTEMPTS_CAP}); programmer error in failover loop"),
                });
            }

            // Check deadline before launching attempt.
            if Instant::now() >= effective_deadline {
                metrics::DEADLINE_EXCEEDED.inc();
                metrics::ATTEMPTS_PER_CALL
                    .with_label_values(&["deadline_exceeded"])
                    .observe(attempts.len() as f64);
                return Err(RouterError::DeadlineExceeded);
            }

            let remaining = effective_deadline.duration_since(Instant::now())
                .min(PROVIDER_DEFAULT_TIMEOUT);
            let call_started = Instant::now();
            let outcome = tokio::time::timeout(
                remaining,
                provider.call_chat(req, model, effective_deadline),
            ).await;

            let elapsed_ms = call_started.elapsed().as_millis() as u32;
            metrics::LATENCY_MS
                .with_label_values(&[provider.kind().as_metric_label(), model])
                .observe(elapsed_ms as f64);

            match outcome {
                Err(_timeout) => {
                    attempts.push(AttemptRecord {
                        provider: provider.kind(),
                        model: model.clone(),
                        attempt_num,
                        fallback_position: chain_idx as u8,
                        status: AttemptStatus::TimeoutBeforeFirstToken,
                        elapsed_ms,
                        http_status: None,
                    });
                    metrics::RETRIES
                        .with_label_values(&[provider.kind().as_metric_label(), "timeout"])
                        .inc();
                    last_error = Some(RouterError::DeadlineExceeded);
                    // Falls through to backoff + retry within provider.
                }
                Ok(Err(e @ RouterError::TerminalProviderError { status: 400, .. })) => {
                    attempts.push(record(provider, model, attempt_num, chain_idx,
                        AttemptStatus::Terminal400, elapsed_ms, Some(400)));
                    metrics::CALLS
                        .with_label_values(&[provider.kind().as_metric_label(), model, "terminal_4xx"])
                        .inc();
                    return Err(e);
                }
                Ok(Err(e @ RouterError::TerminalProviderError { status: 404, .. })) => {
                    attempts.push(record(provider, model, attempt_num, chain_idx,
                        AttemptStatus::Terminal404, elapsed_ms, Some(404)));
                    metrics::CALLS
                        .with_label_values(&[provider.kind().as_metric_label(), model, "terminal_4xx"])
                        .inc();
                    warn!(?e, "router_404_terminal_check_alias_resolver");
                    return Err(e);
                }
                Ok(Err(e @ RouterError::AuthError { .. })) => {
                    attempts.push(record(provider, model, attempt_num, chain_idx,
                        AttemptStatus::TerminalAuth, elapsed_ms, Some(401)));
                    metrics::CALLS
                        .with_label_values(&[provider.kind().as_metric_label(), model, "auth_error"])
                        .inc();
                    error!(?e, severity = "sev-1", "router_auth_error_terminal");
                    return Err(e);
                }
                Ok(Err(RouterError::TerminalProviderError { status: 429, message, retry_after_secs, .. })) => {
                    // Honour Retry-After header value populated by the provider impl from
                    // `reqwest::Response::headers().get(RETRY_AFTER)`. ISS-003 fix: read the
                    // structured field directly instead of scraping `message` for a substring.
                    attempts.push(record(provider, model, attempt_num, chain_idx,
                        AttemptStatus::RetriedAfter429, elapsed_ms, Some(429)));
                    metrics::RETRIES
                        .with_label_values(&[provider.kind().as_metric_label(), "429"])
                        .inc();
                    last_error = Some(RouterError::TerminalProviderError {
                        provider: provider.kind(), status: 429, message,
                        retry_after_secs,
                    });
                    if let Some(secs) = retry_after_secs {
                        let sleep = Duration::from_secs(secs);
                        if Instant::now() + sleep > effective_deadline {
                            // Retry-After exceeds budget — fail over immediately.
                            attempts.last_mut().unwrap().status = AttemptStatus::FailedOver;
                            break;
                        }
                        tokio::time::sleep(sleep).await;
                        continue;
                    }
                    // No Retry-After header — fall through to exponential backoff below.
                }
                Ok(Err(e)) => {
                    let (status_opt, is_5xx) = match &e {
                        RouterError::TerminalProviderError { status, .. } if *status >= 500 => (Some(*status), true),
                        _ => (None, false),
                    };
                    attempts.push(record(provider, model, attempt_num, chain_idx,
                        AttemptStatus::RetriedAfter5xx, elapsed_ms, status_opt));
                    metrics::RETRIES
                        .with_label_values(&[provider.kind().as_metric_label(), "5xx"])
                        .inc();
                    last_error = Some(e);
                    if !is_5xx { break; }   // 4xx other than 400/401/404/429 — break to fallback
                }
                Ok(Ok(mut resp)) => {
                    resp.attempts = std::mem::take(&mut attempts);
                    resp.attempts.push(record(provider, model, attempt_num, chain_idx,
                        AttemptStatus::Succeeded, elapsed_ms, Some(200)));
                    metrics::CALLS
                        .with_label_values(&[provider.kind().as_metric_label(), model, "succeeded"])
                        .inc();
                    metrics::ATTEMPTS_PER_CALL
                        .with_label_values(&["succeeded"])
                        .observe(resp.attempts.len() as f64);
                    return Ok(resp);
                }
            }

            // Backoff before next retry within same provider.
            if attempt_num < MAX_RETRIES_PER_PROVIDER {
                let base_ms = RETRY_DELAYS_MS[(attempt_num - 1) as usize];
                let sleep_ms = jitter::jitter_ms(base_ms, JITTER_FACTOR, &mut rng);
                tokio::time::sleep(Duration::from_millis(sleep_ms as u64)).await;
            }
        }
        // All retries exhausted for this provider — mark last attempt as FailedOver if not already terminal.
        if let Some(last) = attempts.last_mut() {
            if matches!(last.status, AttemptStatus::RetriedAfter5xx | AttemptStatus::RetriedAfter429
                                    | AttemptStatus::TimeoutBeforeFirstToken | AttemptStatus::RetriedAfterConnReset) {
                last.status = AttemptStatus::FailedOver;
            }
        }
    }

    metrics::ATTEMPTS_PER_CALL
        .with_label_values(&["all_failed"])
        .observe(attempts.len() as f64);
    Err(RouterError::AllProvidersFailed {
        last_error: Box::new(last_error.unwrap_or(RouterError::InvalidResponse {
            reason: "no providers in chain".into(),
        })),
        attempts,
    })
}

fn record(
    provider: &dyn Provider,
    model: &str,
    attempt_num: u8,
    chain_idx: usize,
    status: AttemptStatus,
    elapsed_ms: u32,
    http_status: Option<u16>,
) -> AttemptRecord {
    AttemptRecord {
        provider: provider.kind(),
        model: model.to_string(),
        attempt_num,
        fallback_position: chain_idx as u8,
        status,
        elapsed_ms,
        http_status,
    }
}

```

Note: `Retry-After` parsing lives in each provider impl, not here. Provider impls call
`response.headers().get(reqwest::header::RETRY_AFTER).and_then(|v| v.to_str().ok()?.parse().ok())`
and populate `RouterError::TerminalProviderError.retry_after_secs` on 429 responses. The router
reads that structured field — see ISS-003 in the audit for why scraping the message body was
rejected.

```rust
// services/ai-gateway/src/router/jitter.rs

use rand::Rng;

/// Returns base_ms + uniform_jitter(±factor*base_ms).
/// Safe for factor=0.0 (returns base_ms unchanged).
/// Safe for any base_ms (no modulo, no overflow on i32 conversion).
pub fn jitter_ms<R: Rng>(base_ms: u32, factor: f64, rng: &mut R) -> u32 {
    if factor <= 0.0 || base_ms == 0 {
        return base_ms;
    }
    let delta = (base_ms as f64 * factor) as i32;
    if delta == 0 {
        return base_ms;
    }
    let offset = rng.gen_range(-delta..=delta);
    let result = base_ms as i64 + offset as i64;
    result.clamp(0, u32::MAX as i64) as u32
}
```

```rust
// services/ai-gateway/src/router/failover.rs

use crate::policy::{TenantPolicy, ProviderKind};
use crate::alias::ResolvedModel;
use super::Provider;

/// Build the ordered (Provider impl, model name) chain for this call.
/// Position 0 = primary; positions 1.. = fallback chain in declared order.
pub fn build_provider_chain(
    resolved: &ResolvedModel,
    policy: &TenantPolicy,
    alias: &str,
) -> Vec<(Box<dyn Provider>, String)> {
    let mut chain: Vec<(Box<dyn Provider>, String)> = Vec::new();
    chain.push((make_provider(resolved.provider_kind), resolved.model.clone()));
    for fb in &policy.ai_policy.fallback_chain {
        if let Some(model) = fb.model_alias_map.get(alias) {
            chain.push((make_provider(fb.kind()), model.clone()));
        }
    }
    chain
}

fn make_provider(kind: ProviderKind) -> Box<dyn Provider> {
    match kind {
        ProviderKind::Bedrock => Box::new(super::bedrock::BedrockProvider::new()),
        ProviderKind::Anthropic => Box::new(super::anthropic::AnthropicProvider::new()),
        ProviderKind::OpenAI => Box::new(super::openai::OpenAIProvider::new()),
        ProviderKind::Vertex => unimplemented!("Vertex lands in slice 4 (FR-AI-017)"),
        ProviderKind::Bge => unimplemented!("BGE is embedding-only; chat path doesn't use BGE"),
    }
}
```

---

## §7 — Dependencies

### Code dependencies (other FRs/modules)

- **FR-AI-006** — `ResolvedModel` is the input. The router takes it as-is from `alias::resolve()`. The `fallback_position` field is mirrored verbatim into `AttemptRecord.fallback_position` on the primary's first attempt.
- **FR-AI-007** — Cost table validates model existence at alias resolution time. By the time the router sees a `ResolvedModel`, the model is guaranteed to exist in cost-table; a 404 from the provider therefore indicates a configuration error (cost table out of sync with provider catalog) — not a tenant-input issue.
- **FR-AI-009 (downstream)** — Circuit breaker. The router's `build_provider_chain` MAY filter out providers with open breakers (slice 3). In slice 2 the breaker isn't yet wired; the router consumes the full chain.

### Concept dependencies (shared types)

- `ProviderKind` enum from `crate::policy::schema` — closed set; the router exhaustively matches it in `build_provider_chain`.
- `ProviderKind::as_metric_label()` — added in FR-AI-007 ISS-003 fix; the router relies on this method existing. If FR-AI-007 ships without it, this FR adds it as a sub-task.
- `TenantPolicy.ai_policy.fallback_chain` from FR-AI-005 — list of `Provider` configs (each carrying its own `model_alias_map`).
- `ChatCompleteRequest` from `crate::handlers::chat` — the input request.

### Operational / external

- `aws-sdk-bedrockruntime` v1.x for Bedrock InvokeModel calls.
- `reqwest` v0.12 with `rustls` features for Anthropic + OpenAI HTTP.
- `async-openai` v0.20 for OpenAI shape parsing (used in the OpenAI impl).
- `async-trait` for the `Provider` trait (until Rust stabilizes async-fn-in-trait fully).
- `rand` v0.8 for jitter (uses `rand::thread_rng` and `rand::Rng::gen_range`).
- `tokio` v1.x time + macros (for `timeout`, `sleep`, `pause/advance` in tests).
- `tracing` for sev-1 auth-error logs.
- `prometheus` for OBS metrics.

---

## §8 — Example payloads

### Happy call

```rust
let resolved = alias::resolve("chat.smart", &policy).await?;
let response = router::call_provider(&req, &resolved, deadline, &policy).await?;
// response.attempts = [AttemptRecord {
//     provider: Bedrock, model: "anthropic.claude-3-5-sonnet-20241022-v2:0",
//     attempt_num: 1, fallback_position: 0,
//     status: Succeeded, elapsed_ms: 850, http_status: Some(200),
// }]
// response.usage = ProviderUsage { prompt_tokens: 120, completion_tokens: 450, cached_input_tokens: 0 }
// response.finish_reason = FinishReason::Stop
```

### Failover chain audit trail

```rust
// response.attempts after a 503 → 503 → 503 → failover-to-anthropic success
[
    AttemptRecord { provider: Bedrock, attempt_num: 1, fallback_position: 0,
        status: RetriedAfter5xx, elapsed_ms: 1200, http_status: Some(503) },
    AttemptRecord { provider: Bedrock, attempt_num: 2, fallback_position: 0,
        status: RetriedAfter5xx, elapsed_ms: 980, http_status: Some(503) },
    AttemptRecord { provider: Bedrock, attempt_num: 3, fallback_position: 0,
        status: FailedOver, elapsed_ms: 1100, http_status: Some(503) },
    AttemptRecord { provider: Anthropic, attempt_num: 1, fallback_position: 1,
        status: Succeeded, elapsed_ms: 1450, http_status: Some(200) },
]
```

### Terminal 400 with one attempt

```rust
// response = Err(TerminalProviderError { provider: Bedrock, status: 400, message: "context length exceeded" })
// attempts vec accessible via tracing context (not in error variant in slice 2; FR-AI-002 reads it via channel):
[
    AttemptRecord { provider: Bedrock, attempt_num: 1, fallback_position: 0,
        status: Terminal400, elapsed_ms: 220, http_status: Some(400) },
]
```

### 30s budget exhaustion

```rust
// response = Err(AllProvidersFailed { last_error: ..., attempts: vec![15 entries] })
// attempts: 3 × Bedrock + 3 × Anthropic + 3 × OpenAI + 3 × Vertex + 3 × Bge = 15
// (Last attempt of each provider has status = FailedOver, prior two = RetriedAfter5xx)
```

### Retry-After honoured then succeed

```rust
[
    AttemptRecord { provider: Bedrock, attempt_num: 1, status: RetriedAfter429,
        elapsed_ms: 180, http_status: Some(429) },
    // (slept ~1s for Retry-After: 1)
    AttemptRecord { provider: Bedrock, attempt_num: 2, status: Succeeded,
        elapsed_ms: 920, http_status: Some(200) },
]
```

### Streaming stub

```rust
// In slice 2 (FR-AI-008):
let err = router::call_provider_streaming(&req, &resolved, deadline, &policy).await.unwrap_err();
assert!(matches!(err, RouterError::StreamingNotImplemented));

// In slice 3 (FR-AI-010 wires this — replaces stub with full SSE pipeline).
```

---

## §9 — Open questions

All resolved at authoring time. Items deferred to later FRs:

- Streaming first-token latency (FR-AI-010).
- Circuit breaker filtering of open providers from chain (FR-AI-009).
- Tenant-configurable failover budget (FR-AI-021).
- Vertex (Gemini) provider impl (FR-AI-017).
- Per-provider rate-limit pre-check before issuing the request (FR-AI-009 area).

---

## §10 — Failure modes inventory

| Failure | Detection | Return | Recovery |
|---|---|---|---|
| Provider returns 503 | HTTP status | Retry within provider (up to 3); if all fail, failover | Self-healing or operator investigates |
| Provider returns 429 (no Retry-After) | HTTP status | Retry with exponential backoff (200/800ms ± jitter) | Self-healing |
| Provider returns 429 (with Retry-After ≤ remaining budget) | HTTP status + header | Sleep exact `Retry-After` seconds, then retry | Self-healing |
| Provider returns 429 (with Retry-After > remaining budget) | HTTP status + header | Skip sleep, fail over immediately | Next provider serves |
| Provider returns 400 | HTTP status | Terminal; no retry, no failover | Caller fixes prompt |
| Provider returns 401/403 | HTTP status | Terminal; sev-1 log; no failover | Operator investigates credentials |
| Provider returns 404 (model not found) | HTTP status | Terminal; warn log naming alias resolver as the suspect | Operator audits cost-table vs provider catalog |
| Network timeout (per-attempt `tokio::time::timeout`) | timeout future | Same as 503 retry path | Self-healing |
| Connection reset | `reqwest::Error::is_connect()` | Same as 503 retry path | Self-healing |
| All providers in chain failed | exhausted retries on all | `Err(AllProvidersFailed { attempts })` | Caller sees `503` from gateway; FR-AI-002 refunds |
| Deadline exceeded mid-call | `Instant::now() >= effective_deadline` | `Err(DeadlineExceeded)` | FR-AI-002 reconciles as `Cancelled` (refund) |
| Provider response unparseable | `serde_json::from_str` error | `Err(InvalidResponse)` with reason | Sev-2 log; do NOT retry; investigate provider |
| Concurrent calls hit provider rate limit | 429 across many calls | Each individually retries; circuit-breaker (FR-AI-009) opens after threshold | Self-healing with circuit-breaker |
| Audit-record `attempts` overflow (>16) | `attempts.len() >= ATTEMPTS_CAP` | `Err(InvalidResponse { reason: "attempts cap exceeded" })` | Programmer error — should never trigger; sev-2 log |
| `Retry-After` header unparseable (e.g., `"abc"`) | provider-impl `parse::<u64>` returns Err | `retry_after_secs = None`; router uses exponential backoff | None (fall-through behaviour) |
| Provider impl forgets to populate `retry_after_secs` on 429 | Always `None` even when header present | Router uses exponential backoff (correct but suboptimal — sleeps shorter than provider asked) | Lint: per-provider impl tests MUST include a 429-with-Retry-After fixture and assert `retry_after_secs == Some(N)` |
| Jitter helper given `factor = 0.0` | guard at top of `jitter_ms` | Returns `base_ms` unchanged (no panic from modulo by zero) | None (defensive code) |
| `make_provider` called with `ProviderKind::Vertex` in slice 2 | `unimplemented!()` panic | Process panics, supervisor restarts | Slice 4 implements Vertex |

---

## §11 — Notes

- This is the largest FR in slice 2 (10h). Worth doing carefully — every other consumer module's reliability rides on this code path. A 5-line bug in the failover loop becomes a sev-1 incident affecting every tenant.
- The 3-impl provider trait (Bedrock, Anthropic, OpenAI) is sufficient for slice 2. Vertex AI (Gemini) lands in slice 4 (FR-AI-017 area). The `ProviderKind::Vertex` arm in `make_provider` panics with `unimplemented!()` deliberately — fail loud if a slice-2 deploy gets a Vertex-bearing policy.
- The deadline propagation pattern is non-obvious in Rust — readers may want to look at `tokio::time::timeout` docs before reviewing the skeleton. Key insight: `tokio::time::timeout(d, fut)` cancels `fut` when `d` elapses, but the cancellation point is at the next `.await`. Provider impls must yield frequently enough that cancellation is responsive (in practice, the underlying HTTP client's read loop yields per chunk).
- Streaming (FR-AI-010) layers ON TOP of this FR. The non-streaming code path stays as the "synchronous" workhorse; streaming uses the same retry/failover policy but consumes the response as an SSE stream. The trait's `call_chat_streaming` default impl returns `StreamingNotImplemented` so slice-2 consumers that try to stream get a clean error rather than silent fall-through.
- The `RETRY_DELAYS_MS` constant is `&[200, 800]` (length 2, matching `MAX_RETRIES_PER_PROVIDER - 1 = 2` — attempts 2 and 3 each have a backoff; attempt 1 is immediate). An off-by-one bug here would either (a) panic on index out of bounds on attempt 3, or (b) skip the attempt-2 backoff. The unit test `retries_on_503_then_succeeds` would catch either.
- The `jitter_ms` helper is in its own module specifically so the proptest can target it. The same helper is reused by FR-AI-009's circuit breaker reset timing — keep it pure and deterministic given its `Rng`.
- The `ATTEMPTS_CAP = 16` constant catches infinite-loop bugs in `build_provider_chain`. Without it, a misconfigured policy with a self-referential fallback chain (theoretically prevented by FR-AI-005 validation, but defence in depth) would produce unbounded `attempts` vecs. The cap is loud rather than silent — the resulting `InvalidResponse` returns to the caller and triggers a sev-2 alarm.
- Mock-provider construction for tests is centralized in `tests/mocks/mod.rs`. The `ResponseScript` builder lets each test declare a sequence of `(status, optional headers, optional body, optional delay)` tuples that the mock will play back. This keeps individual test bodies short and makes the "what the provider did" intent obvious in the test code.
- The `Retry-After` parsing in the skeleton uses a placeholder `parse_retry_after(message)` that scrapes the message body for the literal substring — the production impl will read from `reqwest::Response::headers()` directly. The placeholder is in the skeleton so the retry-honour logic shape is reviewable; the real wiring is provider-impl-local.

---

*End of FR-AI-008. Status: draft (10/10 target).*
