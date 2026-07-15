---
task_id: TASK-AI-008
audited: 2026-05-16
auditor: manual (engineering-spec template)
verdict: PASS (after revision)
score_pre_revision: 8.5/10        # the first-pass compressed version
score_post_expansion: 9.0/10      # after expanding to TASK-AI-001 depth
score_post_revision: 10/10         # after 4 mechanical fixes
issues_open: 0
issues_resolved: 6
issues_critical: 0
template: engineering-spec@1
revised_at: 2026-05-16
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 ISSes)
---

## §1 — Verdict summary

TASK-AI-008 was expanded from 365 lines (compressed first-pass) to ~720 lines matching TASK-AI-001 depth. The expansion added 4 §1 clauses (#9 404-terminal, #10 Retry-After honour, #13 attempts-cap of 16, #16 streaming stub clause), 6 additional §2 paragraphs (3-vs-5 retries empirics, 200/800 vs Fibonacci/constant rationale, jitter helper safety, Retry-After short-circuit, attempts-cap defence-in-depth, why 30s constant not configurable), full schema types in §3 (`AttemptStatus` 11 variants, `ProviderUsage`, `Choice`, `FinishReason`, `CacheState`, `Provider` trait with default streaming impl, `ProviderStreamResponse`), 6 additional §4 ACs (404-terminal, Retry-After honoured, Retry-After exceeds budget, attempts cap, OBS metrics emit verification, streaming stub), full Rust integration test bodies in §5 (12 named tokio tests + 3 proptest cases for jitter), expanded ~250-line §6 skeleton with metrics module + jitter helper + failover module + record() helper + parse_retry_after stub, code/concept/operational deps in §7, 5 example payloads in §8, 16 failure modes in §10 (added Retry-After unparseable, jitter zero-factor, Vertex unimplemented panic), 8 implementation notes in §11.

Four residual issues prevent 10/10.

## §2 — Findings

### ISS-001 — §1 #16 SHOULD-clause for streaming contradicts AC #16 MUST stub
- **severity:** error
- **rule_id:** clause-vs-ac-consistency
- **location:** §1 #16, §4 AC #16, §3 trait default impl
- **status:** open

#### Description
§1 #16 reads: *"**SHOULD** stream the first token within 1500ms p95 of the call start (when streaming is enabled). … Slice 2 implementations MAY return `Err(RouterError::StreamingNotImplemented)`."*

But §4 AC #16 says: *"In slice 2, calling `call_provider_streaming` MUST return `Err(StreamingNotImplemented)`."*

The clause says "MAY return Err" (optional) while the AC says "MUST return Err" (required). A reader following only §1 might write a partial streaming impl in slice 2 and pass §1 but fail §4. The 1500ms first-token SHOULD belongs to TASK-AI-010 (the task that actually wires streaming), not here.

#### Suggested fix
Replace §1 #16 with a slice-2-scoped clause and move the latency SHOULD to TASK-AI-010:

```
16. **MUST** expose `call_provider_streaming` with the same signature as `call_provider` but
    returning `ProviderStreamResponse`. In slice 2 the implementation MUST return
    `Err(RouterError::StreamingNotImplemented)`. Slice 3's TASK-AI-010 replaces this stub with
    the SSE pipeline; TASK-AI-010 owns the 1500ms p95 first-token SLA.
```

This makes §1 #16 testable by AC #16 and aligns "MAY in slice 2 / MUST in slice 3" cleanly across the two tasks.

### ISS-002 — AC #15 (response normalization) test body has placeholder comments
- **severity:** error
- **rule_id:** test-coverage
- **location:** §4 AC #15, §5 (verification, `response_normalization_matches_across_providers`)
- **status:** open

#### Description
§5's `response_normalization_matches_across_providers` test body contains placeholder comments where actual setup code should appear:

```rust
let anthropic_resp = router::call_provider(/* same req, anthropic primary */).await.unwrap();
let openai_resp = router::call_provider(/* same req, openai primary */).await.unwrap();
```

A code-gen agent reading this can't generate the real call — it has to invent the missing `mocks::resolved_with_primary(anthropic)` / `mocks::resolved_with_primary(openai)` lines. The same shape that was rejected in TASK-AI-007 ISS-001 (proptest body promised but not shown).

#### Suggested fix
Replace the stub with a complete test body that constructs the three resolved-models explicitly:

```rust
#[tokio::test]
async fn response_normalization_matches_across_providers() {
    let canonical_body = mocks::canonical_chat_response_body();
    let bedrock_mock = mock_provider(ProviderKind::Bedrock,
        ResponseScript::ok_with_body(canonical_body.bedrock_shape()));
    let anthropic_mock = mock_provider(ProviderKind::Anthropic,
        ResponseScript::ok_with_body(canonical_body.anthropic_shape()));
    let openai_mock = mock_provider(ProviderKind::OpenAI,
        ResponseScript::ok_with_body(canonical_body.openai_shape()));

    let bedrock_resp = router::call_provider(
        &mocks::default_req(), &mocks::resolved_with_primary(bedrock_mock),
        Instant::now() + Duration::from_secs(30), &mocks::policy_no_fallbacks(),
    ).await.unwrap();
    let anthropic_resp = router::call_provider(
        &mocks::default_req(), &mocks::resolved_with_primary(anthropic_mock),
        Instant::now() + Duration::from_secs(30), &mocks::policy_no_fallbacks(),
    ).await.unwrap();
    let openai_resp = router::call_provider(
        &mocks::default_req(), &mocks::resolved_with_primary(openai_mock),
        Instant::now() + Duration::from_secs(30), &mocks::policy_no_fallbacks(),
    ).await.unwrap();

    // ID and latency_ms differ; everything else matches.
    assert_eq!(bedrock_resp.usage, anthropic_resp.usage);
    assert_eq!(anthropic_resp.usage, openai_resp.usage);
    assert_eq!(bedrock_resp.finish_reason, anthropic_resp.finish_reason);
    assert_eq!(bedrock_resp.choices[0].content, anthropic_resp.choices[0].content);
    assert_eq!(anthropic_resp.choices[0].content, openai_resp.choices[0].content);
}
```

### ISS-003 — `parse_retry_after` scrapes message body instead of reading headers
- **severity:** error
- **rule_id:** correctness / single-source-of-truth
- **location:** §1 #10, §6 skeleton (`parse_retry_after` function + 429 handling block)
- **status:** open

#### Description
§1 #10 says: *"MUST honour the provider's `Retry-After` response header on `429` responses if present and parsable as an integer number of seconds (RFC 9110 §10.2.3)."* This is a header concern.

But §6 skeleton implements it by parsing a substring out of the error `message`:

```rust
fn parse_retry_after(s: &str) -> Option<u64> {
    s.split_once("Retry-After:").and_then(|(_, rest)| rest.trim().parse().ok())
}
```

This couples the router to the exact format that provider impls put into their error messages. If `BedrockProvider::call_chat` returns `RouterError::TerminalProviderError { message: "rate limited; retry after 5s" }`, the parser misses it. If `AnthropicProvider` formats its message differently, the parser misses that too. The header value is the source of truth — it should propagate as a structured field, not via message-string scraping.

This is the same single-source-of-truth pattern called out in TASK-AI-006 ISS-001 (Provider trait having `is_zdr()` while TASK-AI-015 also has `zdr::is_zdr`).

#### Suggested fix
Add a `retry_after_secs: Option<u64>` field to `RouterError::TerminalProviderError` so the header value travels structurally:

```rust
pub enum RouterError {
    // ...
    TerminalProviderError {
        provider: ProviderKind,
        status: u16,
        message: String,
        retry_after_secs: Option<u64>,   // populated from Retry-After header on 429 responses
    },
    // ...
}
```

Provider impls populate it when they see the header; the router reads `retry_after_secs` directly:

```rust
Ok(Err(RouterError::TerminalProviderError { status: 429, retry_after_secs, message, .. })) => {
    // ... use retry_after_secs directly, no parsing
    if let Some(secs) = retry_after_secs {
        // ...
    }
}
```

Then delete `parse_retry_after` from §6 entirely. Add §10 row: *"Provider impl forgets to populate `retry_after_secs` on 429 → router falls back to exponential backoff; correct but suboptimal. Lint: provider impl tests MUST include a 429-with-Retry-After fixture."*

### ISS-004 — §1 #14 lists `outcome="retried"` but §6 never emits it
- **severity:** warning
- **rule_id:** observability-completeness
- **location:** §1 #14, §6 skeleton (`metrics::CALLS.with_label_values` calls)
- **status:** open

#### Description
§1 #14 enumerates the `outcome` label values: *"outcome ∈ `succeeded`/`retried`/`failed_over`/`terminal_4xx`/`auth_error`/`deadline_exceeded`/`all_failed`"* — 7 values.

But §6 skeleton only emits the counter at terminal-decision points: `succeeded`, `terminal_4xx`, `auth_error`, and (implicitly via `ATTEMPTS_PER_CALL`) `deadline_exceeded` / `all_failed`. The `retried` and `failed_over` outcome values are never used as `CALLS` outcome labels — those events are tracked by `RETRIES` and `FAILOVERS` counters separately.

The result: a Grafana panel querying `sum(ai_router_calls_total) by (outcome)` will only ever see 4-5 of the 7 documented outcomes. An operator filtering on `outcome="retried"` gets zero results and assumes "no retries happened" when the truth is "the value is never emitted on `CALLS`".

#### Suggested fix
Tighten §1 #14 to match what §6 actually emits:

```
14. **MUST** emit OTel metrics:
    - `ai_router_calls_total{provider,model,outcome}` (counter; outcome ∈
      `succeeded` / `terminal_4xx` / `auth_error` / `all_failed`) — emitted once per
      terminal decision (success or final failure)
    - `ai_router_retries_total{provider,reason}` (counter; reason ∈ `5xx` / `429` /
      `timeout` / `conn_reset`) — emitted on each retry
    - `ai_router_failovers_total{from,to}` (counter) — emitted on each provider switch
    - `ai_router_latency_ms{provider,model}` (histogram) — per-attempt latency
    - `ai_router_deadline_exceeded_total` (counter)
    - `ai_router_attempts_per_call{final_outcome}` (histogram) — total attempts in this call
```

This separates the two concerns cleanly: `CALLS` is one row per call (terminal outcome), `RETRIES`/`FAILOVERS` are per-event counters. Update AC #14 to match the new label set (drop the assertion on `outcome="failed_over"` which doesn't exist on `CALLS`; keep the `FAILOVERS` from→to assertion).

### ISS-005 — task-audit skill §3.7 rule 22 (traceparent on outbound) — header propagation not asserted in §1
- **severity:** warning
- **rule_id:** authoring-md-§3.7 (rule 22)
- **location:** §1 (no clause about traceparent), §3 (Provider trait `call_chat`), §6 skeleton (HTTP request construction)
- **status:** open

#### Description
The router opens one outbound HTTPS call per attempt (Bedrock, Anthropic, OpenAI, etc.). Task-audit skill §3.7 rule 22 requires "every outbound HTTP/RPC/queue write MUST carry W3C `traceparent`." The current spec relies on TASK-AI-022's `tracing-opentelemetry` layer auto-injecting the header at the `reqwest` middleware boundary, but no §1 clause asserts the contract. A provider impl using a different HTTP client (e.g., `hyper` directly) would silently skip the header. The §1 contract should be explicit: "every provider impl MUST propagate the inbound traceparent into the outbound HTTPS request headers (`traceparent` and `tracestate`)."

#### Suggested fix
Add §1 #17: "**MUST** propagate the inbound W3C `traceparent` and `tracestate` headers into every outbound provider HTTPS call. Provider impls using `reqwest` get this for free via TASK-AI-022's `reqwest-tracing` middleware; provider impls using other HTTP clients MUST add equivalent header propagation. AC #17 verifies via a header-capture mock that the outbound request to `mock_provider` carries the same `traceparent` value as the inbound `cost_ledger::precheck` call." Add corresponding AC #17.

### ISS-006 — task-audit skill §3.10 rule 30 (every MUST NOT has a negative test) — §1 has 3 MUST NOTs, only 2 covered
- **severity:** warning
- **rule_id:** authoring-md-§3.10 (rule 30)
- **location:** §1 (MUST NOT clauses), §5 (negative tests)
- **status:** open

#### Description
Task-audit skill §3.10 rule 30 says "Each `MUST NOT` in §1 corresponds to a negative test in §5." Scanning §1 for MUST NOT: "MUST NOT bypass the alias resolution," "MUST NOT mutate policy from inside the router," and "MUST NOT retry on terminal-4xx errors except 429." The first two have implicit coverage but no named negative test. The third is covered by AC #4. The first two need explicit `#[tokio::test]` bodies asserting: (a) calling `router::call_provider` with a hand-constructed `ResolvedModel` that bypasses alias resolution MUST work AT THE FUNCTION SIGNATURE LEVEL but a corresponding lint MUST flag any production-code call site that constructs `ResolvedModel` manually; (b) calling the router with `policy: &Arc<TenantPolicy>` MUST NOT result in any mutation observable via `Arc::strong_count`.

#### Suggested fix
Add §5 tests `test_router_does_not_mutate_policy` (asserts `Arc::strong_count(&policy)` unchanged before/after call) and `test_lint_flags_manual_resolved_model_construction` (clippy-style lint check via `#[deny(clippy::disallowed_methods)]` on the `ResolvedModel::new_manual` constructor). Add §4 AC #18, #19 to match.

## §3 — Strengths preserved through expansion

- §1 grew from 12 to 16 numbered MUST/SHOULD clauses, each individually testable in §4. Each invariant has at least one §6 enforcement point or §4 verification AC.
- §3 closed-set enums (`AttemptStatus` with 11 explicit variants, `FinishReason` with 5, `CacheState` with 3) match the TASK-AI-007 pattern of exhaustive enum coverage.
- §6 skeleton's `metrics` module preamble (added per TASK-AI-006 ISS-003 pattern) is the right level of detail and demonstrates `as_metric_label()` use throughout.
- §6 introduced `record()` helper to centralize `AttemptRecord` construction — reduces field-mismatch bugs across the 8+ places attempts are pushed.
- §6 jitter helper is in its own module (per ISS-pattern from prior tasks about pure-function testability), explicitly safe for `factor=0.0` and `base_ms=0`.
- §10 failure-modes inventory covers 16 distinct paths including the `unimplemented!()` panic for `ProviderKind::Vertex` in slice 2 and the `attempts.len() >= ATTEMPTS_CAP` defence-in-depth row.
- §7 split into code/concept/operational deps gives reviewers a clear "what other tasks do I need to understand first" answer.

## §4 — Resolution

All 6 mechanical revisions applied:
- ISS-001 RESOLVED (2026-05-16): §1 #16 rewritten to scope streaming behavior to slice 2 (MUST return `StreamingNotImplemented`); the 1500ms first-token SHOULD is moved to TASK-AI-010's domain. AC #16 unchanged (still passes).
- ISS-002 RESOLVED (2026-05-16): §5 `response_normalization_matches_across_providers` test body now constructs all three mock providers and resolved models explicitly; placeholder comments removed.
- ISS-003 RESOLVED (2026-05-16): §3 `RouterError::TerminalProviderError` now carries `retry_after_secs: Option<u64>`; §6 skeleton's 429 branch reads `retry_after_secs` directly; `parse_retry_after` helper deleted from §6. §10 adds the "provider impl forgets to populate retry_after_secs" row.
- ISS-004 RESOLVED (2026-05-16): §1 #14 metric description tightened — `CALLS` outcome label set is now `succeeded`/`terminal_4xx`/`auth_error`/`all_failed` only (4 values, not 7). `retried` and `failed_over` events live on `RETRIES`/`FAILOVERS` per-event counters. AC #14 updated to match.
- ISS-005 RESOLVED (2026-05-16, task-audit skill compliance pass): §1 #17 added asserting `traceparent` + `tracestate` propagation on every outbound provider HTTPS call; AC #17 added (header-capture mock test). TASK-AI-022 cross-ref noted.
- ISS-006 RESOLVED (2026-05-16, task-audit skill compliance pass): §5 added `test_router_does_not_mutate_policy` + `test_lint_flags_manual_resolved_model_construction`; §4 AC #18 + #19 added to match.

**Score = 10/10.** Ship as-is. Ready to transition `draft → accepted`.

---

*End of TASK-AI-008 audit (final). Status: PASS at 10/10.*
