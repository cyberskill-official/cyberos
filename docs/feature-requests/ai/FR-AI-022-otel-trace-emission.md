---
# ───── Machine-readable frontmatter (parsed by feature-request-audit + future fr-catalog renderer) ─────
id: FR-AI-022
title: "OpenTelemetry trace + span emission for every call (caller → router → provider → response) with W3C TraceContext + PII-safe attributes"
module: AI
priority: MUST
status: ready_to_implement
verify: T
phase: P0
milestone: P0 · slice 5
slice: 5
owner: Stephen Cheng
created: 2026-05-15
shipped: null
memory_chain_hash: null
related_frs: [FR-AI-001, FR-AI-002, FR-AI-008, FR-AI-009, FR-AI-010, FR-AI-014, FR-AI-017, FR-AI-019, FR-AI-021, FR-OBS-001, FR-OBS-004, FR-OBS-005, FR-OBS-006]
depends_on: [FR-AI-008, FR-AI-003, FR-OBS-001]
blocks: [FR-OBS-004]

# ───── Source contracts ─────
source_pages:
  - website/docs/modules/ai.html#observability
  - website/docs/modules/obs.html#trace-correlation
  - W3C TraceContext spec (https://www.w3.org/TR/trace-context/)
  - OTel Semantic Conventions for HTTP/Genai (https://opentelemetry.io/docs/specs/semconv/)
source_decisions:
  - DEC-103 (W3C TraceContext is the cross-pillar correlation primitive; not B3, not Jaeger-native)
  - DEC-104 (PII-in-attributes is spec-violating; prevention at caller, not collector)
  - archive/2026-05-14/RESEARCH_REVIEW.md §8.1 (tail-based sampling at OBS collector; gateway emits 100%)

# ───── Build envelope ─────
language: rust 1.81
service: cyberos/services/ai-gateway/
new_files:
  - services/ai-gateway/src/otel/mod.rs
  - services/ai-gateway/src/otel/init.rs
  - services/ai-gateway/src/otel/attributes.rs                 # typed attribute-key constants
  - services/ai-gateway/src/otel/propagation.rs                # W3C extract + inject
  - services/ai-gateway/src/otel/pii_lint.rs                   # AST-walk gate against PII attribute keys
  - services/ai-gateway/tests/otel_test.rs
  - services/ai-gateway/tests/otel_propagation_test.rs
  - services/ai-gateway/tests/otel_pii_lint_test.rs
  - services/ai-gateway/tests/otel_overhead_benchmark_test.rs
  - services/ai-gateway/docs/span-names.md                     # span/attribute naming reference
modified_files:
  - services/ai-gateway/src/handlers/chat.rs                   # #[instrument] on handler
  - services/ai-gateway/src/cost_ledger.rs                     # #[instrument] on precheck/reconcile
  - services/ai-gateway/src/router/mod.rs                      # #[instrument] on call_provider
  - services/ai-gateway/src/persona/mod.rs                     # #[instrument] on load
  - services/ai-gateway/src/cache/mod.rs                       # #[instrument] on lookup/insert
  - services/ai-gateway/src/zdr/mod.rs                         # #[instrument] on is_zdr
  - services/ai-gateway/src/residency/mod.rs                   # #[instrument] on matches
  - services/ai-gateway/src/lib.rs                             # otel::init at boot
  - services/ai-gateway/Cargo.toml                             # opentelemetry@0.21, opentelemetry-otlp, tracing-opentelemetry
allowed_tools:
  - file_read: services/ai-gateway/**
  - file_write: services/ai-gateway/{src,tests,docs}/**
  - bash: docker run -d --name otel-collector -p 4317:4317 otel/opentelemetry-collector
  - bash: cargo test -p cyberos-ai-gateway otel
disallowed_tools:
  - emit PII in span attributes (per §1 #6 — typed attribute keys forbid this; AST lint enforces)
  - skip span emission on any code path (per §1 #1 — every request emits a root span)
  - use B3 or Jaeger-native propagation (per DEC-103 — W3C only)
  - encode full prompts or response text in attributes (per §1 #6 — token counts only)
  - drop trace context from incoming caller HTTP without preserving (per §1 #2 — propagate or generate new)

# ───── Estimated work ─────
effort_hours: 8
sub_tasks:
  - "0.5h: OTel SDK init (`init.rs`) with OTLP gRPC exporter + tonic transport"
  - "0.5h: typed attribute-key constants in `attributes.rs` (no string literals at call sites)"
  - "0.5h: pii_lint.rs AST walk asserting recognized attribute keys only"
  - "1.0h: W3C TraceContext extraction (incoming HTTP) + injection (outgoing provider HTTP)"
  - "0.5h: Root span `ai_gateway.chat_completion` with full attribute set"
  - "1.0h: Child spans for precheck / alias_resolve / redact / persona_load / cache_lookup / zdr / residency / provider_call / reconcile"
  - "0.5h: Per-attempt provider span with `attempt_num`, `fallback_position`, `status_code`, `retried` attrs"
  - "0.5h: Span status codes (Ok/Error/Unset) per OTel semantic conventions"
  - "0.5h: 100% sampling on errors (sampling decision in gateway; tail-based reduction in OBS collector)"
  - "0.5h: Span events for retry boundaries (`retry.attempt`, `retry.backoff_ms`)"
  - "0.5h: Baggage propagation for `tenant_id`, `agent_persona`, `request_id` to downstream services"
  - "0.5h: OTLP buffer config (max_queue_size 10K; max_export_batch_size 512; export_timeout 30s)"
  - "0.5h: Collector-unreachable graceful degradation (drop spans + sev-2 alarm at >1% drop rate)"
  - "0.5h: <1ms overhead benchmark (otel_overhead_benchmark_test.rs comparing spans-on vs spans-off)"
  - "0.5h: span-names.md reference doc (canonical names + attributes per span)"
  - "1.0h: Tests — span tree well-formed, trace context propagated, PII lint, overhead, error sampling"
risk_if_skipped: "FR-OBS-004 (LangSmith integration) and FR-OBS-005 (cross-pillar correlation) have nothing to correlate against. OBS investigators can't trace 'why was this call slow at 14:32 for tenant_alpha?' end-to-end — every request becomes a black box. NFR-PERF-01 latency SLOs become unmeasurable (you can measure aggregate latency but can't decompose into precheck/router/provider/reconcile). First production performance regression takes days to debug instead of minutes; first incident review can't reconstruct the failure path. The OBS pillar's value proposition rests on this FR; without it, the pillar is a logs-only graveyard."
---

## §1 — Description (BCP-14 normative)

The AI Gateway service **MUST** emit OpenTelemetry traces for every request (chat, embed, rerank). The instrumentation and attribute discipline together obey the following:

1. **MUST** initialise the OTel SDK at gateway startup with an OTLP gRPC exporter pointed at `http://localhost:4317` (the OBS collector deployed by FR-OBS-001). Initialisation failure is fatal — gateway refuses to bind. Sampler is `AlwaysOn` at the gateway; tail-based reduction happens in the OBS collector per FR-OBS-006.
2. **MUST** extract W3C TraceContext from incoming HTTP headers (`traceparent`, `tracestate` per W3C spec). If present and valid → root span uses the extracted trace_id and parent_span_id. If absent or malformed → generate a new trace_id; log WARN with the malformed header value (not the raw bytes — the SHA-16 hash). This preserves cross-pillar correlation when callers carry context.
3. **MUST** emit exactly **one root span per request** named `ai_gateway.chat_completion` (for chat) or `ai_gateway.embed` (for embeddings) or `ai_gateway.rerank` (for rerank). Root span attributes are the canonical request-level metadata: `tenant_id`, `model_alias`, `agent_persona`, `idempotency_key`, `stream` (bool), `outcome` (allow | refuse | error). Span status: `Ok` if outcome=allow; `Error` if refuse or error.
4. **MUST** emit child spans for each gateway pipeline stage: `ai_gateway.precheck`, `ai_gateway.alias_resolve`, `ai_gateway.persona_load`, `ai_gateway.zdr_check`, `ai_gateway.residency_check`, `ai_gateway.cache_lookup`, `ai_gateway.redact`, `ai_gateway.provider_call`, `ai_gateway.reconcile`. Each child span carries duration + outcome. Stages skipped due to early-exit (e.g., cache hit short-circuits provider_call) emit nothing — the absence of the span IS the signal.
5. **MUST** emit one child span per provider attempt: `ai_gateway.provider_call`, with attributes `provider` (e.g., `"bedrock"`), `model` (e.g., `"claude-3-5-sonnet-20241022-v2:0"`), `attempt_num` (1-indexed), `fallback_position` (0 = primary; 1+ = fallback chain), `status_code` (HTTP from provider), `retried` (bool). Failover produces multiple `provider_call` spans within the same root span.
6. **MUST NOT** include PII or full prompts/responses in span attributes. The `attributes.rs` module declares typed attribute-key constants; CALL SITES ONLY use these constants — string-literal attribute keys are forbidden by the AST lint (`pii_lint.rs`). Permitted attribute payloads: tenant_id (which is an org-level identifier, not PII), token counts, model names, status codes, durations, timestamps, request_ids, hash16 of redacted prompt for caching debug — never the prompt content itself.
7. **MUST** propagate trace context to outgoing provider HTTP calls via the W3C `traceparent` header. Provider responses (Bedrock, OpenAI) propagate trace context downstream where supported (LangSmith via FR-OBS-004 ingests these). The propagation function lives in `propagation.rs` and is invoked by every provider adapter.
8. **MUST** emit span EVENTS (not separate spans) for retries within a single provider_call span: `retry.attempt` event with `attempt_num`, `backoff_ms`, `prior_status_code`. Retries on the same provider attempt within FR-AI-009's retry budget produce events on the same span; failover to a different provider creates a NEW provider_call span (per §1 #5).
9. **MUST** propagate **baggage** for `tenant_id`, `agent_persona`, `request_id` via `baggage` header alongside `traceparent`. Downstream services (KB module, OBS auto-triage) read baggage to attribute their own spans without re-deriving from request bodies.
10. **MUST** sample 100% of error traces at the gateway. The sampling decision is made at root-span start; a request that ends in error MUST emit the full trace (all child spans) regardless of probabilistic sampling. Tail-based downsampling per FR-OBS-006 happens at the OBS collector, NOT the gateway.
11. **MUST** add < 1ms overhead per call. Benchmark methodology (`otel_overhead_benchmark_test.rs`): compare 1000 sequential calls with OTel disabled vs. enabled; assert `(p95_enabled - p95_disabled) < 1ms`. The OTel SDK is built for this overhead profile; if exceeded, investigate the exporter (likely a slow OTLP endpoint).
12. **MUST** apply standardised span status codes per OTel semantic conventions:
    - `Ok` for successful operations.
    - `Error` for any operation that returned an error (including refuses like ZdrViolation, ResidencyViolation, CapExceeded — these are operationally errors even if semantically intentional).
    - `Unset` is not used; every span has an explicit status.
13. **MUST** configure OTLP exporter with the following defaults:
    - `max_queue_size: 10240` (spans buffered when collector is slow).
    - `max_export_batch_size: 512`.
    - `export_timeout: 30s`.
    - Schedule delay: 5s (export on either max_export_batch_size OR 5s elapsed).
14. **MUST** gracefully degrade on collector unreachability: dropped spans increment `ai_gateway_otel_spans_dropped_total{reason}` (reason ∈ `queue_full | export_timeout | collector_unreachable`). Sustained drop rate > 1% over 5 minutes triggers OBS sev-2 alarm. The gateway never blocks on OTel — span emission is fire-and-forget.
15. **MUST** lint at compile-time (via `pii_lint.rs` AST walk in CI) that EVERY span attribute uses a key declared in `attributes.rs`. New attributes require explicit addition to `attributes.rs` with a comment explaining why it's PII-safe. The lint is the structural defence against accidental PII leakage.
16. **SHOULD** emit OTel METRICS in parallel to traces (the metrics enumerated in each prior FR's `SHOULD emit OTel metrics` clause). Metrics use the same OTLP endpoint; the OBS collector demuxes traces vs. metrics. Histogram metrics report p50, p95, p99 per attribute combination.

---

## §2 — Why this design (rationale for humans)

**Why OTel and not native (e.g., Datadog/Honeycomb SDKs)?** OTel is the vendor-neutral standard. CyberOS's OBS pillar (FR-OBS-001) ships an OTel collector; tenants who self-host or run their own OBS stack get OTLP-compatible traces "for free." Vendor-locked SDKs would force tenants into Datadog or Honeycomb specifically. The small overhead of running through OTel's intermediate representation buys vendor-portability.

**Why W3C TraceContext, not B3 or Jaeger-native (§1 #2)?** W3C TraceContext is the only IETF-standardised propagation format. B3 (Zipkin's format) and Jaeger-native are widely used but not standardised; future tracing tools may not support them. Choosing W3C aligns with where the ecosystem is going. The DEC-103 decision predates this FR; this FR is the implementation.

**Why PII-out-of-attributes is a structural concern, not a runtime one (§1 #6)?** Span attributes are stored, queried, indexed, and often mirrored to multiple analysis tools (Tempo, Honeycomb, Datadog, internal SIEM). A PII leak into one attribute can fan out to many storage locations — each with different access controls and retention. Preventing the leak at the call site (typed attribute keys + AST lint) is much cheaper than detecting and scrubbing at the collector level (which is reactive and incomplete). This is the same prevention-vs-detection principle applied elsewhere (FR-AI-011 redaction, FR-AI-018 cache isolation).

**Why typed attribute-key constants instead of string literals?** A call site like `span.set_attribute("user_email", req.email)` looks innocent — until the lint catches that `user_email` isn't an approved attribute and `req.email` is PII. Typed constants force the developer to ADD the key to `attributes.rs` (with a comment explaining its PII-safety) before using it. The PR-review surface is "is this new key really PII-safe?" rather than the much-more-error-prone "did anyone add a PII-bearing attribute somewhere in this 200-line PR?".

**Why 100% sampling on errors (§1 #10)?** Errors are the high-value debugging cases. A trace of a successful happy-path call is mildly interesting; a trace of a 500 is extremely interesting. Tail-based sampling at the collector reduces happy-path volume by 90% while preserving every error — the right shape for ops investigation. Doing the sampling at the gateway would force a decision before knowing the outcome (root span starts before status is known); collector-side tail sampling waits until all child spans are seen.

**Why baggage propagation (§1 #9)?** Without baggage, downstream services (KB module spans for the same logical request) can't easily attribute their work to a tenant or a request. They'd have to re-derive from request bodies (parsing, authentication, etc.) — duplicating work. Baggage is the thin context-propagation primitive: small set of trustworthy values, propagated by header. The OTel SDK handles baggage natively; this FR just specifies which values to put in.

**Why span events for retries (§1 #8) instead of separate spans?** Retries on the same provider attempt are operationally one logical call (the developer cares about end-to-end provider call latency including retries). Separate spans would inflate the trace tree and obscure the failover-vs-retry distinction. Events on the same span give debugging context (when did the retry happen? what was the prior failure?) without polluting the tree structure. Failover IS a separate span because it represents a different provider — a structurally distinct operation.

**Why OTLP gRPC over HTTP (§1 #1)?** gRPC is more efficient (Protobuf binary; HTTP/2 multiplexing) than OTLP-over-HTTP (JSON; HTTP/1.1). At our trace volume (~1000 spans/sec), gRPC's lower per-span overhead matters. The OBS collector supports both; gateway choice is gRPC for efficiency.

**Why graceful degradation on collector unreachability (§1 #14)?** OTel is operational telemetry; collector outages happen. A hard-fail on collector-down would take down the gateway. Buffering then dropping with metrics + alarms preserves availability — the OBS pillar's outage doesn't cascade to the AI pillar's outage. This is the same separation-of-concerns principle as FR-AI-017 cache's Redis-down handling.

**Why the <1ms overhead budget (§1 #11)?** Latency budgets are precious; OTel's overhead must be invisible. The OTel Rust SDK is built for this — span creation is ~1µs, attribute setting is ~100ns. The 1ms ceiling protects against adversarial cases (slow OTLP exporter, collector backpressure causing in-process queueing). The benchmark is the safety net.

**Why span-names.md as a separate reference doc (§1 sub_tasks)?** Span names are the developer-facing API for trace queries. An investigator types `service.name=ai-gateway AND name=ai_gateway.precheck` to find precheck spans. If names drift across PRs, queries break. A canonical reference doc + PR-review process for additions is the discipline that keeps names stable.

---

## §3 — API contract (formal spec for AI-agent implementers)

### Initialisation

```rust
// services/ai-gateway/src/otel/init.rs
use opentelemetry::{global, KeyValue};
use opentelemetry_sdk::{trace::TracerProvider, Resource};
use opentelemetry_otlp::{WithExportConfig, SpanExporterBuilder};

pub fn init_otel(endpoint: &str) -> Result<TracerProvider, OtelInitError> {
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(endpoint)
        .with_timeout(std::time::Duration::from_secs(30));

    let provider = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(
            opentelemetry_sdk::trace::config()
                .with_max_queue_size(10_240)
                .with_max_export_batch_size(512)
                .with_resource(Resource::new(vec![
                    KeyValue::new("service.name", "ai-gateway"),
                    KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                ])),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;

    global::set_tracer_provider(provider.clone());
    Ok(provider)
}

#[derive(Debug, thiserror::Error)]
pub enum OtelInitError {
    #[error("OTLP exporter setup failed: {0}")]
    Exporter(String),
    #[error("collector unreachable at {0} during boot health-check")]
    CollectorUnreachable(String),
}
```

### Typed attribute-key constants (PII defence)

```rust
// services/ai-gateway/src/otel/attributes.rs
//! Approved attribute keys. PII-safe by construction.
//! New keys MUST be added here with a `// PII-safe because: ...` comment.
//! AST lint in `pii_lint.rs` rejects unrecognised keys at CI time.

pub const TENANT_ID:           &str = "ai_gateway.tenant_id";          // PII-safe: org-level id, not personal
pub const MODEL_ALIAS:         &str = "ai_gateway.model_alias";        // PII-safe: e.g., "chat.smart"
pub const AGENT_PERSONA:       &str = "ai_gateway.agent_persona";      // PII-safe: e.g., "cuo-cpo@0.4.1"
pub const IDEMPOTENCY_KEY:     &str = "ai_gateway.idempotency_key";    // PII-safe: caller-generated UUID-shape
pub const STREAM:              &str = "ai_gateway.stream";             // PII-safe: bool
pub const OUTCOME:             &str = "ai_gateway.outcome";            // PII-safe: enum (allow|refuse|error)
pub const PROVIDER:            &str = "ai_gateway.provider";           // PII-safe: enum (bedrock|anthropic|openai|...)
pub const MODEL:               &str = "ai_gateway.model";              // PII-safe: model id like "claude-3-5-sonnet"
pub const ATTEMPT_NUM:         &str = "ai_gateway.attempt_num";        // PII-safe: integer
pub const FALLBACK_POSITION:   &str = "ai_gateway.fallback_position";  // PII-safe: integer
pub const STATUS_CODE:         &str = "ai_gateway.status_code";        // PII-safe: HTTP integer
pub const RETRIED:             &str = "ai_gateway.retried";            // PII-safe: bool
pub const PROMPT_TOKENS:       &str = "ai_gateway.prompt_tokens";      // PII-safe: count, not content
pub const COMPLETION_TOKENS:   &str = "ai_gateway.completion_tokens";  // PII-safe: count, not content
pub const ESTIMATED_USD:       &str = "ai_gateway.estimated_usd";      // PII-safe: number
pub const ACTUAL_USD:          &str = "ai_gateway.actual_usd";         // PII-safe: number
pub const CACHE_STATE:         &str = "ai_gateway.cache_state";        // PII-safe: enum (hit|miss|skipped|error)
pub const CACHE_KEY_HASH16:    &str = "ai_gateway.cache_key_hash16";   // PII-safe: hash, not content
pub const REQUEST_ID:          &str = "ai_gateway.request_id";         // PII-safe: UUID-shape, not personal
pub const REGION:              &str = "ai_gateway.region";             // PII-safe: AWS region string

// Span event attribute keys (used in events, not spans)
pub const RETRY_ATTEMPT:       &str = "retry.attempt";
pub const RETRY_BACKOFF_MS:    &str = "retry.backoff_ms";
pub const RETRY_PRIOR_STATUS:  &str = "retry.prior_status_code";

// FORBIDDEN at compile time (these are PII; if a future need emerges, requires FR amendment + DPO sign-off):
// pub const USER_EMAIL:        &str = ... — would leak personal email
// pub const PROMPT_TEXT:       &str = ... — would leak prompt content
// pub const RESPONSE_TEXT:     &str = ... — would leak response content
// pub const PHONE:             &str = ... — would leak phone number
// pub const CCCD:              &str = ... — would leak Vietnamese government ID
```

### W3C TraceContext propagation

```rust
// services/ai-gateway/src/otel/propagation.rs
use opentelemetry::propagation::{Extractor, Injector, TextMapPropagator};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use std::collections::HashMap;

pub struct HeaderExtractor<'a>(pub &'a http::HeaderMap);
impl<'a> Extractor for HeaderExtractor<'a> {
    fn get(&self, key: &str) -> Option<&str> { self.0.get(key).and_then(|v| v.to_str().ok()) }
    fn keys(&self) -> Vec<&str> { self.0.keys().map(|k| k.as_str()).collect() }
}

pub struct HeaderInjector<'a>(pub &'a mut http::HeaderMap);
impl<'a> Injector for HeaderInjector<'a> {
    fn set(&mut self, key: &str, value: String) {
        if let (Ok(name), Ok(val)) = (
            http::HeaderName::from_bytes(key.as_bytes()),
            http::HeaderValue::from_str(&value),
        ) {
            self.0.insert(name, val);
        }
    }
}

pub fn extract_context_from_headers(headers: &http::HeaderMap) -> opentelemetry::Context {
    let propagator = TraceContextPropagator::new();
    propagator.extract(&HeaderExtractor(headers))
}

pub fn inject_context_into_headers(ctx: &opentelemetry::Context, headers: &mut http::HeaderMap) {
    let propagator = TraceContextPropagator::new();
    propagator.inject_context(ctx, &mut HeaderInjector(headers));
}
```

### PII lint (CI gate)

```rust
// services/ai-gateway/src/otel/pii_lint.rs
use std::path::Path;
use syn::{visit::Visit, ExprMethodCall, LitStr};

const ALLOWED_KEYS_FILE: &str = "src/otel/attributes.rs";

pub fn lint_no_unknown_attribute_keys(src_dir: &Path) -> Result<(), Vec<LintFailure>> {
    let allowed = parse_allowed_keys(ALLOWED_KEYS_FILE)?;
    let mut failures = vec![];

    for entry in walkdir::WalkDir::new(src_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.path().extension().map(|e| e == "rs").unwrap_or(false) {
            let src = std::fs::read_to_string(entry.path()).unwrap();
            let file: syn::File = syn::parse_str(&src).unwrap();
            let mut visitor = AttributeKeyVisitor { allowed: &allowed, failures: &mut failures, path: entry.path() };
            visitor.visit_file(&file);
        }
    }
    if failures.is_empty() { Ok(()) } else { Err(failures) }
}

struct AttributeKeyVisitor<'a> { allowed: &'a HashSet<String>, failures: &'a mut Vec<LintFailure>, path: &'a Path }
impl<'ast> Visit<'ast> for AttributeKeyVisitor<'_> {
    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        // Check for span.set_attribute("string-literal", ...) — string literal forbidden.
        if node.method == "set_attribute" {
            if let Some(syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. })) = node.args.first() {
                let key = s.value();
                if !self.allowed.contains(&key) {
                    self.failures.push(LintFailure {
                        path: self.path.into(), key,
                        reason: "attribute key not in attributes.rs allow-list",
                    });
                }
            }
        }
        syn::visit::visit_expr_method_call(self, node);
    }
}
```

### Handler integration

```rust
// services/ai-gateway/src/handlers/chat.rs (additions)
use opentelemetry::trace::{SpanKind, Status, Tracer};
use crate::otel::{attributes::*, propagation};

#[tracing::instrument(skip_all, fields(
    tenant_id = %req.tenant_id,
    model_alias = %req.model_alias,
    agent_persona = %req.agent_persona,
    idempotency_key = %req.idempotency_key,
    stream = req.stream,
))]
pub async fn handle_chat(req: ChatCompleteRequest, headers: HeaderMap)
    -> Result<HttpResponse, ApiError>
{
    let ctx = propagation::extract_context_from_headers(&headers);

    let result = async {
        let _hold = cost_ledger::precheck(&req).await?;
        let persona = persona::load(&req.persona_handle()).await?;
        let (provider, model, region) = alias::resolve(&req.alias, &policy).await?;
        // ZDR + residency happen inside alias::resolve

        let response = if !req.stream {
            cache::lookup(&key).await
                .or_else_async(|_| async { router::call_provider(&req, region).await })
                .await?
        } else {
            router::call_provider(&req, region).await?
        };
        cost_ledger::reconcile(&_hold, &response).await?;
        Ok(response)
    }.await;

    // Set span outcome + status per §1 #12.
    let span = tracing::Span::current();
    match &result {
        Ok(_) => { span.record(OUTCOME, "allow"); span.record_status(Status::Ok); }
        Err(e) => { span.record(OUTCOME, "error"); span.record_status(Status::error(e.to_string())); }
    }
    result
}
```

---

## §4 — Acceptance criteria (testable, ordered, numbered)

1. **OTel init succeeds at boot** — Gateway connects to collector at localhost:4317; logs `otel_initialized`; init failure refuses bind.
2. **Root span emitted per request** — Every chat request produces exactly one span named `ai_gateway.chat_completion`; same for embed/rerank.
3. **Child spans tree-structured** — Trace UI shows precheck → persona_load → alias_resolve → zdr_check → residency_check → (cache_lookup OR provider_call) → reconcile as nested children.
4. **W3C trace propagation extracted** — Caller's `traceparent` header populates the gateway's root span's trace_id; provider HTTP calls carry the propagated header.
5. **W3C trace propagation injected** — Outgoing provider HTTP calls (Bedrock, OpenAI) carry `traceparent` header with the gateway's trace_id and child span_id.
6. **Malformed traceparent generates new trace_id** — Send a malformed `traceparent: bad`; root span has a new trace_id; WARN log emitted with hash of bad value.
7. **No PII in attributes** — `otel_pii_lint_test` AST-walks all `*.rs` files; every `set_attribute` call uses a key from `attributes.rs`. Adding a non-allowlisted key `"user_email"` causes test failure.
8. **100% error sampling** — A call returning HTTP 500 produces a fully-sampled trace (all child spans); the trace appears in the collector even if the gateway sample rate is configured to 10%.
9. **Overhead < 1ms** — `otel_overhead_benchmark_test` runs 1000 calls with OTel disabled vs. enabled; `(p95_enabled - p95_disabled) < 1ms`.
10. **Provider span includes attempt metadata** — Failover trace shows 3 `provider_call` spans (3 attempts on primary at fallback_position=0) + 1 successful span (fallback_position=1). Each carries `attempt_num`, `status_code`, `retried`.
11. **Span events for retries** — Same provider attempt with 2 retries produces ONE `provider_call` span with TWO `retry.attempt` events; events carry `attempt_num`, `backoff_ms`, `prior_status_code`.
12. **Baggage propagation** — Outgoing HTTP carries `baggage: tenant_id=org:test,agent_persona=cuo-cpo@0.4.1,request_id=req_X`. Downstream service receiving this can read baggage values from its own OTel context.
13. **Span status set per outcome** — Successful call → status `Ok`; refuse (e.g., ZdrViolation) → status `Error` with description; provider 500 → status `Error`.
14. **Collector-unreachable graceful degrade** — Stop collector; gateway continues serving requests; `ai_gateway_otel_spans_dropped_total{reason=collector_unreachable}` increments; sustained drop > 1% over 5 min triggers OBS sev-2 alarm.
15. **Span buffer config** — OTLP exporter configured with `max_queue_size=10240`, `max_export_batch_size=512`, `export_timeout=30s` (asserted by `otel_test::test_exporter_config`).
16. **Cache-hit path emits no provider_call span** — Request that hits cache emits root + precheck + persona_load + alias_resolve + cache_lookup + reconcile (no provider_call).
17. **Span names match span-names.md catalogue** — `otel_test::span_names_match_doc` asserts every emitted span name is documented in `docs/span-names.md`.

---

## §5 — Verification

```rust
// services/ai-gateway/tests/otel_test.rs
use opentelemetry_sdk::testing::trace::InMemorySpanExporter;
use opentelemetry::trace::Status;

fn test_tracer_with_in_memory_exporter() -> (TracerProvider, InMemorySpanExporter) {
    let exporter = InMemorySpanExporter::default();
    let provider = TracerProvider::builder()
        .with_simple_exporter(exporter.clone()).build();
    (provider, exporter)
}

#[tokio::test]
async fn root_span_emitted_per_request() {
    let (provider, exporter) = test_tracer_with_in_memory_exporter();
    opentelemetry::global::set_tracer_provider(provider);
    let _ = handle_chat_test_request().await;

    let spans = exporter.get_finished_spans().unwrap();
    let roots: Vec<_> = spans.iter()
        .filter(|s| s.parent_span_id == opentelemetry::trace::SpanId::INVALID).collect();
    assert_eq!(roots.len(), 1);
    assert_eq!(roots[0].name, "ai_gateway.chat_completion");
}

#[tokio::test]
async fn child_spans_tree_structured() {
    let (provider, exporter) = test_tracer_with_in_memory_exporter();
    opentelemetry::global::set_tracer_provider(provider);
    let _ = handle_chat_test_request().await;

    let spans = exporter.get_finished_spans().unwrap();
    let root = spans.iter().find(|s| s.name == "ai_gateway.chat_completion").unwrap();
    let children: Vec<_> = spans.iter().filter(|s| s.parent_span_id == root.span_context.span_id()).collect();
    let names: HashSet<_> = children.iter().map(|s| s.name.as_str()).collect();
    assert!(names.contains("ai_gateway.precheck"));
    assert!(names.contains("ai_gateway.persona_load"));
    assert!(names.contains("ai_gateway.alias_resolve"));
    assert!(names.contains("ai_gateway.provider_call"));
    assert!(names.contains("ai_gateway.reconcile"));
}

#[tokio::test]
async fn w3c_trace_context_propagated() {
    let (provider, exporter) = test_tracer_with_in_memory_exporter();
    opentelemetry::global::set_tracer_provider(provider);

    let mut headers = HeaderMap::new();
    headers.insert("traceparent", "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01".parse().unwrap());
    let _ = handle_chat_with_headers(test_request(), headers).await;

    let spans = exporter.get_finished_spans().unwrap();
    let root = spans.iter().find(|s| s.name == "ai_gateway.chat_completion").unwrap();
    // OTel `TraceId`'s `Display` impl yields the 32-char lower-hex W3C form.
    // Do NOT use `{:?}` here — Debug prints `TraceId(0af7…)`, which fails this
    // assertion. The string compared on the right is the W3C `trace_id` in the
    // `traceparent` header injected above.
    assert_eq!(format!("{}", root.span_context.trace_id()), "0af7651916cd43dd8448eb211c80319c");
}

#[tokio::test]
async fn malformed_traceparent_generates_new_trace_id() {
    let (provider, exporter) = test_tracer_with_in_memory_exporter();
    opentelemetry::global::set_tracer_provider(provider);

    let mut headers = HeaderMap::new();
    headers.insert("traceparent", "malformed-bad-value".parse().unwrap());
    let _ = handle_chat_with_headers(test_request(), headers).await;

    let spans = exporter.get_finished_spans().unwrap();
    let root = spans.iter().find(|s| s.name == "ai_gateway.chat_completion").unwrap();
    assert!(root.span_context.trace_id().to_bytes() != [0u8; 16]);
}

#[tokio::test]
async fn outgoing_provider_call_carries_traceparent() {
    let (provider, _) = test_tracer_with_in_memory_exporter();
    opentelemetry::global::set_tracer_provider(provider);
    let mock_provider = MockProvider::start();
    let _ = handle_chat_with_provider(test_request(), &mock_provider).await;

    let last_outgoing = mock_provider.last_request_headers();
    assert!(last_outgoing.contains_key("traceparent"));
}

#[tokio::test]
async fn provider_call_spans_per_attempt_with_attributes() {
    let (provider, exporter) = test_tracer_with_in_memory_exporter();
    opentelemetry::global::set_tracer_provider(provider);

    // Force 3 attempts on primary then fallback succeeds.
    let _ = handle_chat_with_failover_3_attempts().await;

    let spans = exporter.get_finished_spans().unwrap();
    let provider_calls: Vec<_> = spans.iter().filter(|s| s.name == "ai_gateway.provider_call").collect();
    assert_eq!(provider_calls.len(), 4);   // 3 primary + 1 fallback
    assert_eq!(provider_calls[0].attributes.get("ai_gateway.fallback_position").unwrap().as_str(), "0");
    assert_eq!(provider_calls[3].attributes.get("ai_gateway.fallback_position").unwrap().as_str(), "1");
    assert_eq!(provider_calls[3].attributes.get("ai_gateway.status_code").unwrap().as_str(), "200");
}

#[tokio::test]
async fn retry_events_on_same_span() {
    let (provider, exporter) = test_tracer_with_in_memory_exporter();
    opentelemetry::global::set_tracer_provider(provider);

    let _ = handle_chat_with_2_retries_then_success().await;

    let spans = exporter.get_finished_spans().unwrap();
    let provider_call = spans.iter().find(|s| s.name == "ai_gateway.provider_call").unwrap();
    let retry_events: Vec<_> = provider_call.events.iter().filter(|e| e.name == "retry.attempt").collect();
    assert_eq!(retry_events.len(), 2);
}

#[tokio::test]
async fn span_status_set_per_outcome() {
    let (provider, exporter) = test_tracer_with_in_memory_exporter();
    opentelemetry::global::set_tracer_provider(provider);

    // Successful call.
    let _ = handle_chat_test_request().await;
    let success_root = exporter.get_finished_spans().unwrap().into_iter()
        .find(|s| s.name == "ai_gateway.chat_completion").unwrap();
    assert_eq!(success_root.status, Status::Ok);

    exporter.reset();

    // ZdrViolation (refuse).
    let _ = handle_chat_with_zdr_required_non_zdr_provider().await;
    let refuse_root = exporter.get_finished_spans().unwrap().into_iter()
        .find(|s| s.name == "ai_gateway.chat_completion").unwrap();
    assert!(matches!(refuse_root.status, Status::Error { .. }));
}

#[tokio::test]
async fn 100_percent_sampling_on_errors() {
    // Configure gateway sample rate to 10%; force an error; assert trace appears.
    std::env::set_var("OTEL_TRACES_SAMPLER_ARG", "0.10");
    let (provider, exporter) = test_tracer_with_in_memory_exporter();
    opentelemetry::global::set_tracer_provider(provider);

    let _ = handle_chat_that_returns_500().await;
    let spans = exporter.get_finished_spans().unwrap();
    assert!(!spans.is_empty(), "error trace should always sample regardless of rate");
}

#[tokio::test]
async fn collector_unreachable_increments_drop_counter() {
    // Configure exporter to point at unreachable endpoint.
    let _ = otel::init::init_otel("http://127.0.0.1:9999").unwrap();   // nothing listens here
    for _ in 0..1000 { let _ = handle_chat_test_request().await; }

    tokio::time::sleep(std::time::Duration::from_secs(35)).await;   // export_timeout
    let dropped = otel_test_helper::counter_value(
        "ai_gateway_otel_spans_dropped_total",
        &[("reason", "collector_unreachable")],
    );
    assert!(dropped > 0);
}

#[tokio::test]
async fn cache_hit_path_emits_no_provider_call_span() {
    let (provider, exporter) = test_tracer_with_in_memory_exporter();
    opentelemetry::global::set_tracer_provider(provider);

    test_helper::warm_cache_for(test_request());
    let _ = handle_chat_test_request().await;
    let spans = exporter.get_finished_spans().unwrap();
    assert!(!spans.iter().any(|s| s.name == "ai_gateway.provider_call"));
    assert!(spans.iter().any(|s| s.name == "ai_gateway.cache_lookup"));
}
```

```rust
// services/ai-gateway/tests/otel_pii_lint_test.rs
#[test]
fn no_unknown_attribute_keys_in_codebase() {
    let result = otel::pii_lint::lint_no_unknown_attribute_keys(Path::new("src/"));
    assert!(result.is_ok(), "unknown attribute keys: {result:?}");
}

#[test]
fn lint_rejects_planted_user_email_attribute() {
    // Test fixture: a file with a forbidden attribute.
    let fixture = "src/otel/test_fixtures/has_pii.rs";
    let result = otel::pii_lint::lint_one_file(Path::new(fixture));
    assert!(result.is_err());
    let failures = result.unwrap_err();
    assert!(failures.iter().any(|f| f.key == "user_email"));
}
```

```rust
// services/ai-gateway/tests/otel_overhead_benchmark_test.rs
#[tokio::test]
#[ignore = "long-running benchmark; run with --ignored"]
async fn otel_overhead_under_1ms_p95() {
    let mut samples_off = vec![];
    let mut samples_on = vec![];

    std::env::set_var("OTEL_SDK_DISABLED", "true");
    for _ in 0..1000 {
        let t0 = std::time::Instant::now();
        let _ = handle_chat_test_request().await;
        samples_off.push(t0.elapsed().as_micros() as u64);
    }

    std::env::set_var("OTEL_SDK_DISABLED", "false");
    let _ = otel::init::init_otel("http://localhost:4317").unwrap();
    for _ in 0..1000 {
        let t0 = std::time::Instant::now();
        let _ = handle_chat_test_request().await;
        samples_on.push(t0.elapsed().as_micros() as u64);
    }

    samples_off.sort(); samples_on.sort();
    let p95_off = samples_off[(samples_off.len() as f64 * 0.95) as usize];
    let p95_on = samples_on[(samples_on.len() as f64 * 0.95) as usize];
    let overhead_us = p95_on as i64 - p95_off as i64;
    assert!(overhead_us < 1000, "OTel overhead p95 = {overhead_us}µs (budget 1000µs)");
}
```

```bash
docker run -d --name otel-collector -p 4317:4317 otel/opentelemetry-collector
cd services/ai-gateway
cargo test otel
cargo test otel -- --ignored   # runs the overhead benchmark
```

---

## §6 — Implementation skeleton

See §3 for init, attributes, propagation, lint, handler integration. Boot order:

```rust
// services/ai-gateway/src/lib.rs (additions)
pub async fn run() -> Result<(), Error> {
    let _ = otel::init::init_otel("http://localhost:4317").map_err(|e| {
        eprintln!("OTel init failed: {e}; gateway refusing to bind");
        std::process::exit(1);
    })?;
    // ... existing initialisations ...
}
```

Span on persona load (using `tracing::instrument` macro):

```rust
// services/ai-gateway/src/persona/mod.rs (additions)
#[tracing::instrument(skip(handle), fields(persona_handle = %handle.display()))]
pub fn load(handle: &PersonaHandle) -> Result<Arc<Persona>, PersonaError> {
    // existing logic
}
```

Provider call with attempt metadata:

```rust
// services/ai-gateway/src/router/mod.rs (additions)
#[tracing::instrument(skip(req), fields(
    provider = ?provider, model = %model,
    attempt_num = attempt, fallback_position = fp,
))]
pub async fn call_provider_attempt(req: &ChatCompleteRequest, provider: ProviderKind,
                                    model: &str, attempt: u32, fp: u32)
    -> Result<ProviderResponse, RouterError>
{
    let mut headers = HeaderMap::new();
    propagation::inject_context_into_headers(&opentelemetry::Context::current(), &mut headers);
    // ... call provider with these headers ...
    let span = tracing::Span::current();
    match &result {
        Ok(resp) => { span.record(STATUS_CODE, resp.status); span.record(RETRIED, false); }
        Err(_)   => { span.record(STATUS_CODE, 500); span.record_status(Status::error("provider failed")); }
    }
    result
}
```

---

## §7 — Dependencies

### Code dependencies (other FRs/modules)

- **FR-AI-008** — Router exposes `call_provider`; this FR wraps it with span instrumentation.
- **FR-AI-001** — `cost_ledger::precheck` and `reconcile` get span instrumentation.
- **FR-AI-002** — `ai.invocation` rows correlate with traces via `request_id` attribute.
- **FR-AI-009** — Circuit breaker retries become span events on the same `provider_call` span.
- **FR-AI-010** — Streaming bypasses cache; `cache_state: skipped` attribute on root span.
- **FR-AI-014** — `persona_load` span carries `agent_persona` from FR-AI-014's handle.
- **FR-AI-017** — `cache_lookup` span carries `cache_state` (hit/miss/skipped/error).
- **FR-AI-019/020** — Embedding/rerank sidecar HTTP calls carry traceparent.
- **FR-AI-021** — CLI mutations emit memory audit rows; this FR adds spans for the CLI commands themselves (separate concern).
- **FR-OBS-001** — Deploys the OTel collector at localhost:4317.
- **FR-OBS-004** — LangSmith integration consumes propagated trace context to correlate LLM-side traces.
- **FR-OBS-005** — Cross-pillar correlation depends on W3C TraceContext propagation.
- **FR-OBS-006** — Tail-based sampling at the collector; this FR samples 100% at the gateway.

### Concept dependencies (shared types)

- W3C TraceContext is the cross-pillar correlation primitive (DEC-103).
- Typed attribute keys in `attributes.rs` are the PII-safety primitive — call sites use constants only.
- Span name conventions in `docs/span-names.md` are the developer-facing query API.
- Span events for retries, child spans for failovers — the operational distinction is structural.

### Operational / external

- Rust crates: `opentelemetry@0.21`, `opentelemetry-otlp@0.14`, `opentelemetry_sdk@0.21`, `tracing@0.1`, `tracing-opentelemetry@0.22`, `tracing-subscriber@0.3`, `syn@2` (lint), `walkdir@2` (lint).
- OTel collector at `localhost:4317` (FR-OBS-001 deployment).
- gRPC transport (tonic).
- `OTEL_SDK_DISABLED` env var for benchmark control; `OTEL_TRACES_SAMPLER_ARG` for sample rate config.

---

## §8 — Example payloads

### Trace tree (success path)

```text
ai_gateway.chat_completion (1850ms, status=Ok)
  ├─ tenant_id: org:cyberskill
  ├─ model_alias: chat.smart
  ├─ agent_persona: cuo-cpo@0.4.1
  ├─ outcome: allow
  ├─ ai_gateway.precheck (35ms, status=Ok)
  │     └─ estimated_usd: 0.012, current_spent_usd: 47.23
  ├─ ai_gateway.persona_load (0.05ms, status=Ok)
  │     └─ persona_handle: cuo-cpo@0.4.1
  ├─ ai_gateway.alias_resolve (0.5ms, status=Ok)
  │     └─ provider: bedrock, model: anthropic.claude-3-5-sonnet-20241022-v2:0
  ├─ ai_gateway.zdr_check (0.01ms, status=Ok)
  ├─ ai_gateway.residency_check (0.01ms, status=Ok)
  ├─ ai_gateway.cache_lookup (3ms, status=Ok)
  │     └─ cache_state: miss
  ├─ ai_gateway.redact (28ms, status=Ok)
  ├─ ai_gateway.provider_call (1450ms, status=Ok)
  │     ├─ provider: bedrock, model: claude-3-5-sonnet
  │     ├─ attempt_num: 1, fallback_position: 0
  │     ├─ status_code: 200, retried: false
  │     └─ prompt_tokens: 142, completion_tokens: 86
  └─ ai_gateway.reconcile (60ms, status=Ok)
        └─ actual_usd: 0.0078
```

### Trace tree (failover with retries)

```text
ai_gateway.chat_completion (8200ms, status=Ok, outcome=allow)
  ├─ ai_gateway.precheck (32ms)
  ├─ ai_gateway.persona_load (0.04ms)
  ├─ ai_gateway.alias_resolve (0.4ms)
  ├─ ai_gateway.zdr_check (0.01ms)
  ├─ ai_gateway.residency_check (0.01ms)
  ├─ ai_gateway.cache_lookup (2ms, cache_state=miss)
  ├─ ai_gateway.redact (24ms)
  ├─ ai_gateway.provider_call (3100ms, status=Error)
  │     ├─ provider: bedrock, attempt_num: 1, fallback_position: 0
  │     ├─ status_code: 503, retried: true
  │     ├─ events: [
  │     │   { name: retry.attempt, attempt_num: 2, backoff_ms: 100, prior_status_code: 503 },
  │     │   { name: retry.attempt, attempt_num: 3, backoff_ms: 250, prior_status_code: 503 },
  │     │ ]
  ├─ ai_gateway.provider_call (4800ms, status=Ok)         ← fallback
  │     ├─ provider: anthropic, attempt_num: 1, fallback_position: 1
  │     └─ status_code: 200
  └─ ai_gateway.reconcile (55ms)
```

### Trace tree (cache hit)

```text
ai_gateway.chat_completion (12ms, status=Ok, outcome=allow)
  ├─ ai_gateway.precheck (3ms)
  ├─ ai_gateway.persona_load (0.05ms)
  ├─ ai_gateway.alias_resolve (0.5ms)
  ├─ ai_gateway.cache_lookup (3ms, cache_state=hit)        ← cache hit
  └─ ai_gateway.reconcile (5ms)                            ← no provider_call span
```

### W3C traceparent header (incoming → outgoing)

```text
Incoming:  traceparent: 00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01

Gateway root span:
  trace_id:        0af7651916cd43dd8448eb211c80319c (preserved)
  parent_span_id:  b7ad6b7169203331
  span_id:         <new-id>

Outgoing to provider:
  traceparent: 00-0af7651916cd43dd8448eb211c80319c-<gateway-child-span-id>-01
  baggage:     tenant_id=org:cyberskill,agent_persona=cuo-cpo@0.4.1,request_id=req_01HZK...
```

### PII lint failure example

```text
$ cargo test otel_pii_lint_test::no_unknown_attribute_keys
running 1 test
test no_unknown_attribute_keys ... FAILED

unknown attribute keys:
  src/handlers/chat.rs:147 — key="user_email" — attribute key not in attributes.rs allow-list
  src/router/mod.rs:23     — key="prompt_text" — attribute key not in attributes.rs allow-list

Add the keys to src/otel/attributes.rs WITH a `// PII-safe because: ...` comment,
OR remove the call site if the value is PII-bearing.
```

### Collector-unreachable WARN log

```text
WARN  reason=collector_unreachable dropped_count=42 elapsed=35s
      OTel exporter cannot reach localhost:4317 — spans queued, oldest evicted at queue cap
```

---

## §9 — Open questions

All resolved at authoring time. Items deferred to later FRs:

- Tail-based sampling configuration AT the collector — FR-OBS-006 owns; this FR samples 100% at gateway.
- Span-attribute schema versioning (e.g., `ai_gateway.outcome.v2 = ...`) — out of scope; current convention is "rename = breaking change; add new key for new semantics."
- Continuous-profiling integration (pyroscope, OTel profile signal) — slice 6+; FR-OBS-008 area.
- Custom span exporter for tenant-self-hosted observability — slice 6+; current FR ships gateway-side OBS pillar only.
- LLM-call-content tracing (prompt + response in trace, like LangSmith does) — FR-OBS-004 owns; structured to NOT leak PII via redacted-prompt-only.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Collector unreachable at boot | OTLP connect error in `init_otel` | Gateway refuses to bind (sev-1) | Operator investigates collector OR config |
| Collector unreachable mid-run | Export error | Spans buffered up to `max_queue_size`; oldest evicted; metric `spans_dropped_total{collector_unreachable}` | Sev-2 alarm at >1% drop sustained 5min |
| Collector overload (4317 backpressure) | Slow export | Spans dropped after `export_timeout` (30s); metric `spans_dropped_total{export_timeout}` | Auto-scale collector |
| Span attribute too large | OTel SDK truncates internally | Truncation logged at INFO | Caller checks attribute sizes |
| PII accidentally added to span (string-literal key) | `otel_pii_lint_test` AST walk in CI | PR blocked | Add key to `attributes.rs` with PII-safe comment OR remove call site |
| PII accidentally added via dynamic key | Runtime detection by FR-OBS PII-scrub on collector | Spans flagged sev-1 | Engineer fixes call site; PR with attributes.rs update |
| Trace context invalid (malformed traceparent) | Parse error | New trace_id generated; WARN log with hash16 of bad value | Self-resolves; investigate caller for malformed-emit pattern |
| Outgoing HTTP missing traceparent | Integration test asserts header presence | PR blocked | Add `propagation::inject_context_into_headers` to provider adapter |
| Span tree malformed (orphan child without parent) | Trace UI shows orphans; integration test asserts parent_id matches | PR blocked | Fix `tracing::instrument` invocation order |
| Span overhead exceeds 1ms p95 | `otel_overhead_benchmark_test` fails | CI fails | Investigate exporter latency; check for synchronous span ops |
| Boot-time OTel init failure | Process exits 1 | Gateway never binds | Operator checks collector + endpoint config |
| Sampler rate misconfiguration (errors not 100%) | `100_percent_sampling_on_errors` test fails | PR blocked | Fix sampler to AlwaysOn at gateway level |
| Span status not set | Integration test asserts every span has Ok or Error status | PR blocked | Add `span.record_status(...)` call |
| `tracing::instrument` macro applied to wrong function | Span emitted but with wrong attributes | Trace UI inspection during testing | Fix macro args |
| Baggage not propagated | Test asserts baggage header on outgoing | PR blocked | Configure baggage propagator |
| OTel SDK panic (rare; library bug) | Process logs panic; tracing::Span operations should never panic | Sev-1 if reproducible | Upstream issue; pin OTel version |
| `OTEL_SDK_DISABLED=true` in production | Integration test verifies env in production deployment | Deploy script asserts var unset | Operator fixes env vars |
| Span name drift (developer adds new name not in span-names.md) | `span_names_match_doc` test fails | PR blocked | Update span-names.md or rename span |
| Failover spans missing fallback_position attribute | Integration test asserts attribute presence | PR blocked | Update router span builder |
| Cache-hit path emits provider_call span (regression) | Integration test asserts no provider_call on cache hit | PR blocked | Fix handler control flow |
| Trace context not propagated to downstream sidecar (BGE) | Integration test verifies sidecar receives traceparent | PR blocked | BGE adapter must call `inject_context_into_headers` |

---

## §11 — Notes

- The `#[tracing::instrument]` macro is the Rust-idiomatic way to add spans. Keep call sites clean; the macro handles span creation, attribute setting, and Drop-based span finalisation. Manual span management is reserved for the few cases where instrumentation logic is non-trivial (e.g., per-attempt provider spans).
- Trace export to Jaeger/Tempo/Honeycomb via OTLP is standard; FR-OBS-001 owns the collector configuration. The gateway is collector-agnostic — it speaks OTLP and lets the collector route.
- Token counts in span attributes (not full prompts) preserve privacy while keeping debug utility. An investigator can correlate "this span has 5000 prompt tokens, latency 5s" → "this is a long-context request" without seeing the prompt content.
- The PII lint (§1 #15) is the structural defence. Reactive scrubbing at the collector is a fallback; preventing the leak at the call site is the primary control. The lint runs in CI on every PR; new attributes require explicit allowlist addition with rationale.
- W3C TraceContext propagation is the cross-pillar correlation primitive. CyberOS's other pillars (KB, OBS, memory) all consume the same `traceparent`; the AI Gateway is the trace-context EMITTER for AI calls but a propagator for upstream callers (e.g., a CUO request that flows through AI then KB).
- The failover-vs-retry distinction in span structure (§1 #5 + #8) matters operationally. An investigator looking at a slow trace sees "3 retries on bedrock, 1 successful on anthropic" as ONE provider_call span with retry events PLUS one fresh provider_call span — the visual contrast IS the diagnostic.
- The 100% sampling on errors + tail-based reduction at collector (§1 #10) is the "have your cake and eat it" pattern. Errors are always investigatable; happy-path volume is manageable.
- The OTel SDK's batch export is asynchronous; spans accumulate in a queue and export every 5s OR at batch-size-512 fill. The 30s `export_timeout` is the upper bound; sustained collector failures cause queue eviction (oldest spans dropped). The `spans_dropped_total` metric is the visibility primitive.
- The `<1ms overhead` budget (§1 #11) is achievable because the OTel Rust SDK is written for this; span creation is ~1µs, attribute setting is ~100ns. The budget protects against worst-case scenarios (slow exporter blocking the hot path); the benchmark catches regressions.
- `span-names.md` is the developer-facing query API. Investigators learn the catalogue once and write Tempo/Honeycomb queries against stable names. PR-discipline keeps the catalogue stable; the `span_names_match_doc` test (AC #17) prevents drift.

---

*End of FR-AI-022. Status: draft (10/10 target).*
