---
id: FR-OBS-005
title: "W3C TraceContext correlation across logs/metrics/traces/AI-traces — propagate, embed, exemplar, end-to-end CI test"
module: OBS
priority: MUST
status: implementing
verify: T
phase: P0
milestone: P0 · slice 2
slice: 2
owner: Stephen Cheng (CTO)
created: 2026-05-15
shipped: null
memory_chain_hash: null
related_frs: [FR-OBS-001, FR-OBS-002, FR-OBS-003, FR-OBS-004, FR-AI-022]
depends_on: [FR-OBS-001, FR-OBS-003, FR-OBS-004]
blocks: []

source_pages:
  - website/docs/modules/obs.html#correlation
  - W3C TraceContext spec (https://www.w3.org/TR/trace-context/)
source_decisions:
  - DEC-160 (W3C TraceContext over B3/Jaeger; IETF-standard, broad support)
  - DEC-161 (trace_id embedded in EVERY log + metric exemplar; correlation primitive)
  - DEC-162 (CI gate enforces end-to-end correlation; single test verifies log/span/AI-trace alignment)

language: rust 1.81
service: cyberos/crates/cyberos-obs-sdk/
new_files:
  - crates/cyberos-obs-sdk/src/tracecontext.rs
  - crates/cyberos-obs-sdk/src/logging.rs
  - crates/cyberos-obs-sdk/src/exemplar.rs
  - crates/cyberos-obs-sdk/tests/tracecontext_test.rs
  - crates/cyberos-obs-sdk/tests/log_enrichment_test.rs
  - crates/cyberos-obs-sdk/tests/end_to_end_correlation_test.rs
  - .github/workflows/obs-correlation-gate.yml
modified_files:
  - services/ai-gateway/**                                # use with_trace_context wrapper
  - services/auth/**
  - services/chat/**
  - services/memory/**
  - crates/cyberos-obs-sdk/src/red.rs                     # exemplar emission
allowed_tools:
  - file_read: services/**, crates/cyberos-obs-sdk/**
  - file_write: services/**, crates/cyberos-obs-sdk/**
  - bash: cargo test -p cyberos-obs-sdk tracecontext
disallowed_tools:
  - strip trace_id from any structured log line (per §1 #2)
  - emit metric without trace_id exemplar where supported (per §1 #3)
  - generate trace_id at internal services without checking incoming header (per §1 #1)
  - use B3 or Jaeger-native propagation (per DEC-160)

effort_hours: 8
sub_tasks:
  - "0.5h: tracecontext.rs — `with_trace_context(req, f)` extracting/generating trace_id"
  - "1.0h: logging.rs — tracing-subscriber layer that adds trace_id + span_id + tenant_id to every log"
  - "1.0h: exemplar.rs — Prometheus exemplar emission per histogram bucket"
  - "1.0h: red.rs modification to emit exemplars on `cyberos_duration_ms` histogram"
  - "0.5h: HTTP client wrapper that injects traceparent into outgoing requests"
  - "0.5h: tracecontext_test — extract / generate / propagate / inject"
  - "0.5h: log_enrichment_test — every log line in test capture has trace_id + tenant_id"
  - "1.5h: end_to_end_correlation_test.rs — synthetic call → assert log + span + langsmith + metric all share trace_id"
  - "0.5h: obs-correlation-gate.yml CI workflow"
  - "1.0h: Service integration — apply to every axum middleware in AI Gateway + AUTH + CHAT + memory"
risk_if_skipped: "Investigation requires joining 4 tools (Loki + Prometheus + Tempo + LangSmith) by timestamp. 'Why was call X slow?' takes 30 minutes (search each tool, correlate by tenant + time window) instead of 30 seconds (paste trace_id everywhere). Without trace_id in logs, structured-log search by trace_id (the primary debug query) is impossible. Without exemplars, jumping from a Grafana latency spike to the offending trace requires manual time-window narrowing."
---

## §1 — Description (BCP-14 normative)

Every CyberOS service **MUST** propagate W3C TraceContext via HTTP headers AND embed `trace_id` + `span_id` into every structured log line + Prometheus histogram exemplar. Each piece:

1. **MUST** parse incoming HTTP `traceparent` header per W3C spec at every service boundary; if absent or malformed, generate a new trace_id (NEVER reject the request — TraceContext is operational, not security). Edge services (CHAT, AUTH) typically generate; internal services (AI Gateway, memory) typically receive.
2. **MUST** include `trace_id` + `span_id` + `tenant_id` fields in EVERY structured log line emitted by CyberOS services. Implementation: `tracing-subscriber` layer that pulls from current OTel context and adds the fields automatically. No manual `info!(trace_id = ..., ...)` boilerplate at call sites.
3. **MUST** include `trace_id` as Prometheus exemplar on the `cyberos_duration_ms` histogram (FR-OBS-003). Exemplars let Grafana operators click a histogram bucket and jump directly to the offending trace in Tempo. The OTel SDK's `record` method supports exemplar injection natively.
4. **MUST** forward `traceparent` (and `tracestate` + `baggage`) to downstream HTTP calls. The HTTP client wrapper auto-injects from the current span's context. Manual `req.headers_mut().insert("traceparent", ...)` at call sites is forbidden — use the wrapper.
5. **MUST** correlate LangSmith trace with operational trace via shared `trace_id` (FR-OBS-004 §1 #1 already requires this; this FR ensures the propagation chain ending at LangSmith preserves the value).
6. **MUST** include `tenant_id` in every log line + every metric label (extracted from current request context). Without tenant_id, multi-tenant filtering at the OBS layer (FR-OBS-002) doesn't work.
7. **MUST** be CI-gated by `obs-correlation-gate.yml`: the workflow runs `end_to_end_correlation_test.rs` which makes a synthetic AI call, waits for OTel batches to flush, then queries Loki + Tempo + LangSmith + Prometheus and asserts all 4 systems hold records for the same trace_id.
8. **MUST** preserve trace_id through tokio tasks (e.g., async background work). The `tracing::Instrument` extension propagates the span across `tokio::spawn`; this FR requires its use throughout.
9. **MUST** preserve trace_id through cross-process boundaries (subprocess spawns, e.g., memory_writer subprocess). The OTel context is serialised via env vars `OTEL_TRACE_ID` and `OTEL_SPAN_ID`; the subprocess restores at boot.
10. **MUST** generate W3C-compliant trace_id (16 random bytes, hex-encoded as 32-char string) when no incoming header. The `opentelemetry::trace::TraceId::from_random()` produces this.
11. **MUST** validate parsed `traceparent` strictly per W3C spec: `00-{32hex}-{16hex}-01` format. Malformed → generate new trace_id + log WARN with hash of bad value (NOT raw — bad values may be malicious).
12. **SHOULD** emit OTel metrics:
    - `obs_tracecontext_extracted_total{outcome}` (counter; outcome ∈ extracted | malformed | missing_generated_new).
    - `obs_log_enrichment_total{service}` (counter; tracks how many logs were enriched).
    - `obs_exemplar_emission_total` (counter; tracks histogram exemplar emissions).

---

## §2 — Why this design (rationale for humans)

**Why W3C TraceContext (DEC-160)?** IETF-standardised. Broad support across SDKs (Rust, Python, Go, JS, Java). B3 (Zipkin) and Jaeger-native are widely used historically but their non-standard nature creates interop friction. Choosing W3C aligns with where the ecosystem is going.

**Why generate new trace_id when traceparent absent (§1 #1)?** TraceContext is an OPERATIONAL signal, not a SECURITY one. Rejecting requests without traceparent would break callers who don't emit it (early-stage clients, tools, ad-hoc curl). Generating new trace_id at the boundary preserves observability without sacrificing availability.

**Why trace_id in EVERY log (§1 #2 + DEC-161)?** The primary debug query is "show me everything that happened during this call." Without trace_id in logs, that query becomes "show me everything in time window X for tenant Y" — noisy, slow, often wrong. With trace_id, it's `loki: {trace_id="abc"}` — instant.

**Why exemplars on histograms (§1 #3)?** A Grafana latency spike (p99 jumped from 100ms to 2s) raises the question "which call caused it?" Without exemplars, ops manually narrows the time window then greps Tempo. With exemplars, click the bucket → jump to the trace. Reduces investigation time from minutes to seconds.

**Why auto-inject traceparent at HTTP client (§1 #4)?** Manual injection is error-prone — missing one downstream call breaks the chain. Auto-inject via wrapper means every outgoing HTTP call is instrumented; the chain is unbreakable.

**Why CI gate on end-to-end correlation (§1 #7)?** Correlation is the kind of property that breaks silently. A propagation gap doesn't produce errors; it produces "the trace stops at service X." The CI test makes a synthetic call and verifies all 4 backends hold records for the same trace_id. Detection at PR time, not in production.

**Why preserve trace_id through tokio::spawn (§1 #8)?** Async background work (e.g., audit-row emission) is part of the same logical request. Without `Instrument`, the spawned task creates its own trace — the chain is broken; debugging "did the audit row write?" requires manual correlation.

**Why preserve through cross-process boundaries (§1 #9)?** memory_writer is a subprocess; without env-var propagation, the chain is broken at process boundaries. The serialisation via env vars is the standard pattern for OTel cross-process.

**Why strict traceparent validation + WARN on malformed (§1 #11)?** Malformed traceparent could be: (a) buggy client, (b) attempt to inject specific trace_id (security concern). Logging the bad value's hash (not raw) gives forensic capability without leaking attacker-controlled bytes.

**Why generate, not preserve, on malformed (§1 #11)?** Honoring an attacker-supplied trace_id would let them poison correlation (e.g., two unrelated requests with same trace_id appear linked in Tempo). Generating fresh ensures each request has an unbiased trace_id.

---

## §3 — API contract

```rust
// crates/cyberos-obs-sdk/src/tracecontext.rs
use axum::http::Request;
use opentelemetry::trace::TraceContextExt;
use tracing::Instrument;

/// Wraps an axum handler: extracts trace_id from headers OR generates new; sets context for the duration of `f`.
pub async fn with_trace_context<F, Fut, T>(req: &Request<axum::body::Body>, f: F) -> T
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = T>,
{
    let trace_id = match extract_traceparent(req.headers()) {
        Ok(tc) => {
            metrics::tracecontext_extracted("extracted");
            tc.trace_id
        }
        Err(ExtractError::Missing) => {
            metrics::tracecontext_extracted("missing_generated_new");
            opentelemetry::trace::TraceId::from_random()
        }
        Err(ExtractError::Malformed(hash16)) => {
            tracing::warn!(traceparent_hash16 = %hash16, "malformed traceparent; generating new");
            metrics::tracecontext_extracted("malformed");
            opentelemetry::trace::TraceId::from_random()
        }
    };
    let span = tracing::info_span!("request", trace_id = %format!("{trace_id:032x}"));
    f().instrument(span).await
}

pub fn extract_traceparent(headers: &HeaderMap) -> Result<TraceContext, ExtractError> {
    let h = headers.get("traceparent").ok_or(ExtractError::Missing)?;
    let s = h.to_str().map_err(|_| ExtractError::Malformed(hash16(h.as_bytes())))?;
    parse_w3c_traceparent(s).ok_or_else(|| ExtractError::Malformed(hash16(s.as_bytes())))
}

pub fn inject_traceparent(headers: &mut HeaderMap, ctx: &opentelemetry::Context) {
    let span_ctx = ctx.span().span_context();
    let value = format!("00-{:032x}-{:016x}-01", span_ctx.trace_id(), span_ctx.span_id());
    headers.insert("traceparent", value.parse().unwrap());
}

#[derive(Debug, thiserror::Error)]
pub enum ExtractError {
    #[error("traceparent header missing")]
    Missing,
    #[error("traceparent header malformed (hash16: {0})")]
    Malformed(String),
}
```

```rust
// crates/cyberos-obs-sdk/src/logging.rs
use tracing_subscriber::Layer;

/// tracing-subscriber layer that adds trace_id + span_id + tenant_id to every log record.
pub struct ObsContextLayer;

impl<S: tracing::Subscriber> Layer<S> for ObsContextLayer {
    fn on_event(&self, event: &Event, ctx: Context<S>) {
        let span_ctx = opentelemetry::Context::current().span().span_context();
        let trace_id = format!("{:032x}", span_ctx.trace_id());
        let span_id  = format!("{:016x}", span_ctx.span_id());
        let tenant_id = current_tenant_id().unwrap_or_else(|| "unknown".into());

        // Inject as event fields via a custom Visit that wraps the user's Visit
        // (implementation specific to tracing-subscriber Layer trait)
    }
}

pub fn current_tenant_id() -> Option<String> {
    opentelemetry::Context::current()
        .baggage()
        .get("tenant_id")
        .map(|v| v.value().to_string())
}
```

```rust
// crates/cyberos-obs-sdk/src/exemplar.rs
use opentelemetry::metrics::Histogram;
use opentelemetry::KeyValue;

pub fn record_with_exemplar(histogram: &Histogram<f64>, value: f64, labels: &[KeyValue]) {
    let trace_id = opentelemetry::Context::current().span().span_context().trace_id();
    // OTel SDK 0.21 supports exemplar via `record` overload; embed trace_id as exemplar attribute
    histogram.record(value, labels);
    // Prometheus exemplar emission via the OTel-Prometheus exporter is automatic if trace_id is in current context
}
```

```rust
// crates/cyberos-obs-sdk/src/red.rs (modified per FR-OBS-003 §1 #3 hook)
pub fn record_request(...) {
    // ... existing ...
    exemplar::record_with_exemplar(&DURATION.get().unwrap(), duration_ms as f64, &labels);
}
```

```rust
// HTTP client wrapper
pub struct InstrumentedClient(reqwest::Client);

impl InstrumentedClient {
    pub async fn post<B: Serialize>(&self, url: &str, body: &B) -> Result<reqwest::Response, reqwest::Error> {
        let mut req = self.0.post(url).json(body).build()?;
        tracecontext::inject_traceparent(req.headers_mut(), &opentelemetry::Context::current());
        // also inject baggage for tenant_id propagation
        let baggage = format!("tenant_id={}", current_tenant_id().unwrap_or_default());
        req.headers_mut().insert("baggage", baggage.parse().unwrap());
        self.0.execute(req).await
    }
}
```

CI workflow:

```yaml
# .github/workflows/obs-correlation-gate.yml
name: OBS End-to-End Correlation Gate
on:
  pull_request:
    paths:
      - 'crates/cyberos-obs-sdk/**'
      - 'services/**'
      - 'deploy/obs/**'
      - '.github/workflows/obs-correlation-gate.yml'

jobs:
  correlation:
    runs-on: ubuntu-22.04
    timeout-minutes: 15
    services:
      otel-stack:
        image: docker
    steps:
      - uses: actions/checkout@v4
      - run: docker compose -f deploy/obs/docker-compose.yml up -d
      - run: docker compose -f deploy/obs/langsmith-docker-compose.yml up -d
      - run: sleep 30
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - run: cargo test --release end_to_end_correlation -- --test-threads=1
```

---

## §4 — Acceptance criteria

1. **Incoming traceparent preserved** — Synthetic call with header `traceparent: 00-abc123...-def456...-01`; trace_id `abc123...` appears in all downstream services' logs + spans.
2. **Missing traceparent generates new trace_id** — Synthetic call without header → new 32-char hex trace_id; metric `obs_tracecontext_extracted_total{outcome=missing_generated_new}` increments.
3. **Malformed traceparent generates new + WARN log** — Header `traceparent: bad-value` → new trace_id; WARN log with hash16; metric `outcome=malformed`.
4. **Every log line in Loki carries trace_id** — Synthetic call; query Loki for the trace_id; >5 log lines returned across multiple services.
5. **Every log line carries tenant_id** — Same query returns lines with `tenant_id=<expected>`.
6. **Prometheus histogram has exemplar** — Query `cyberos_duration_ms_bucket{...}` with exemplars enabled; bucket has trace_id reference.
7. **Exemplar click in Grafana → Tempo trace** — Manual UI test (or screenshot proof); the click navigates to the right Tempo trace.
8. **Outgoing HTTP carries traceparent** — Test with mock downstream service; received request has `traceparent` header matching current trace_id.
9. **Outgoing HTTP carries baggage** — Same test; `baggage: tenant_id=<value>` header present.
10. **LangSmith trace correlates** — FR-OBS-004 export uses same trace_id; test queries LangSmith for trace_id and finds the AI call.
11. **End-to-end CI test passes** — `end_to_end_correlation_test.rs`: synthetic call → assert log + span + langsmith + metric all share trace_id.
12. **tokio::spawn preserves trace_id** — Background task spawned within request scope: log lines from the task carry the same trace_id.
13. **Subprocess preserves trace_id** — memory_writer subprocess invocation sets `OTEL_TRACE_ID` env var; subprocess logs carry the same trace_id.
14. **W3C-compliant generated trace_id** — `from_random()` produces 16 random bytes, 32 hex chars, all lowercase.
15. **Strict W3C parser rejects bad formats** — `traceparent: 01-...-...-01` (wrong version) → malformed; `traceparent: 00-{31hex}-{16hex}-01` (wrong length) → malformed.
16. **Hash16 of bad traceparent NOT raw** — WARN log contains 16-hex hash, not the raw value.

---

## §5 — Verification

```rust
// crates/cyberos-obs-sdk/tests/tracecontext_test.rs
#[tokio::test]
async fn extracts_valid_traceparent() {
    let mut headers = HeaderMap::new();
    headers.insert("traceparent", "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01".parse().unwrap());
    let ctx = tracecontext::extract_traceparent(&headers).unwrap();
    assert_eq!(format!("{:032x}", ctx.trace_id), "0af7651916cd43dd8448eb211c80319c");
}

#[tokio::test]
async fn missing_traceparent_returns_missing() {
    let headers = HeaderMap::new();
    let err = tracecontext::extract_traceparent(&headers).expect_err("expected Missing");
    assert!(matches!(err, tracecontext::ExtractError::Missing));
}

#[tokio::test]
async fn malformed_traceparent_returns_malformed_with_hash() {
    let mut headers = HeaderMap::new();
    headers.insert("traceparent", "totally-bogus".parse().unwrap());
    let err = tracecontext::extract_traceparent(&headers).expect_err("expected Malformed");
    match err {
        tracecontext::ExtractError::Malformed(hash) => assert_eq!(hash.len(), 16),
        e => panic!("wrong: {e:?}"),
    }
}

#[tokio::test]
async fn injection_to_outgoing_request() {
    let trace_id = TraceId::from_random();
    let mut headers = HeaderMap::new();
    let ctx = opentelemetry::Context::current_with_span_context(/* trace_id */);
    tracecontext::inject_traceparent(&mut headers, &ctx);
    let value = headers.get("traceparent").unwrap().to_str().unwrap();
    assert!(value.starts_with("00-"));
    assert!(value.contains(&format!("{trace_id:032x}")));
}

// crates/cyberos-obs-sdk/tests/log_enrichment_test.rs
#[tokio::test]
async fn every_log_line_has_trace_id_and_tenant_id() {
    let trace_id = TraceId::from_random();
    let captured = test_helper::capture_logs(|| {
        with_trace_context(&test_request(trace_id, "tenant_a"), || async {
            tracing::info!("test message");
            tracing::warn!("another");
        }).await;
    }).await;
    for line in captured {
        assert!(line.contains(&format!("trace_id={trace_id:032x}")));
        assert!(line.contains("tenant_id=tenant_a"));
    }
}
```

```rust
// crates/cyberos-obs-sdk/tests/end_to_end_correlation_test.rs
#[tokio::test]
#[ignore = "requires full OTel stack; run with --ignored in obs-correlation-gate workflow"]
async fn synthetic_call_correlates_across_loki_tempo_langsmith_prometheus() {
    let trace_id = TraceId::from_random();
    let trace_id_hex = format!("{trace_id:032x}");

    // Synthesise an AI call with the trace_id
    let resp = test_helper::synthetic_ai_call(&trace_id, &test_tenant()).await;
    assert!(resp.is_ok());

    // Wait for OTel batches to flush
    tokio::time::sleep(Duration::from_secs(15)).await;

    // Query Loki
    let loki_logs = loki_query(&format!(r#"{{trace_id="{trace_id_hex}"}}"#)).await.unwrap();
    assert!(!loki_logs.is_empty(), "expected logs in Loki for trace_id {trace_id_hex}");

    // Query Tempo
    let tempo_trace = tempo_query(&trace_id_hex).await.unwrap();
    assert!(tempo_trace.spans.len() > 1, "expected multi-span trace");

    // Query LangSmith
    let langsmith_trace = langsmith_query(&trace_id_hex).await.unwrap();
    assert_eq!(langsmith_trace.trace_id, trace_id_hex);

    // Query Prometheus exemplars
    let exemplars = prometheus_exemplar_query(
        &format!("cyberos_duration_ms_bucket{{tenant_id=\"{}\"}}", test_tenant()),
    ).await.unwrap();
    let trace_ids_in_exemplars: HashSet<String> = exemplars.iter().filter_map(|e| e.trace_id.clone()).collect();
    assert!(trace_ids_in_exemplars.contains(&trace_id_hex), "expected exemplar referencing {trace_id_hex}");
}
```

---

## §6 — Implementation skeleton

See §3.

```rust
// services/ai-gateway/src/main.rs (using the wrapper)
use axum::middleware;
use cyberos_obs_sdk::tracecontext::with_trace_context;

let app = Router::new()
    .route("/v1/chat/completions", post(handle_chat))
    .layer(middleware::from_fn(|req, next| async move {
        with_trace_context(&req, || async { next.run(req).await }).await
    }));
```

---

## §7 — Dependencies

- **FR-OBS-003** — RED metrics; this FR adds exemplar emission to histograms.
- **FR-OBS-004** — LangSmith trace_id correlation.
- **FR-AI-022** — OTel trace emission; this FR ensures the trace_id is universal.
- Crates: `opentelemetry@0.21`, `opentelemetry-otlp@0.14`, `opentelemetry_sdk@0.21`, `tracing@0.1`, `tracing-subscriber@0.3` with `Layer`, `tracing-opentelemetry`, `axum-tracing-opentelemetry@0.18`.

---

## §8 — Example payloads

### Log line in Loki

```json
{
  "ts": "2026-05-15T14:00:00.123Z",
  "level": "info",
  "msg": "ai_gateway.precheck_allow",
  "trace_id": "0af7651916cd43dd8448eb211c80319c",
  "span_id": "b7ad6b7169203331",
  "tenant_id": "org:cyberskill",
  "estimated_usd": 0.0085
}
```

### Prometheus histogram with exemplar

```text
cyberos_duration_ms_bucket{service="ai-gateway",route="/v1/chat/completions",tenant_id="org:cyberskill",le="500"} 42 # {trace_id="0af7651916cd43dd8448eb211c80319c"} 437 1747526400.123
```

### Outgoing HTTP with propagation

```http
POST /api/v1/messages HTTP/1.1
Host: api.anthropic.com
Authorization: Bearer ...
traceparent: 00-0af7651916cd43dd8448eb211c80319c-c1d2e3f4a5b6c7d8-01
baggage: tenant_id=org:cyberskill
```

### Malformed traceparent log

```text
WARN  traceparent_hash16=4b8c0d2f1a7e9c3b
      malformed traceparent; generating new
```

---

## §9 — Open questions

All resolved. Deferred:
- Span links (correlate two unrelated traces) — slice 5+.
- Probabilistic sampling decision propagation (FR-OBS-006 owns) — already covered.
- Cross-region trace correlation (federation) — slice 6+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Service drops trace_id propagation | end_to_end_correlation_test fails | CI blocks PR | Engineer fixes propagation in that service |
| Invalid traceparent on incoming | strict W3C parser | Generate new + WARN with hash16 | Self-resolves; investigate caller |
| Cross-call clock skew | trace_id correct but timestamps inconsistent | Trace UI may misorder spans | Tempo span_id-based ordering compensates |
| Missing tenant_id in baggage | log enrichment uses "unknown" | Sev-3 alarm if "unknown" > 1% | Operator fixes upstream caller |
| Tokio task spawn without `Instrument` | logs from task lack trace_id | Sev-3 detected via log search | Add `.instrument(Span::current())` |
| Subprocess without env propagation | subprocess logs lack trace_id | Sev-3 alarm | Add OTEL_TRACE_ID to subprocess env |
| Exemplar emission breaks Prometheus | scrape error | Sev-3 alarm | Investigate exporter version |
| Bad traceparent floods malformed-counter | metric `outcome=malformed` spike | Sev-2 alarm; investigate caller | Possibly attack |
| Outgoing HTTP wrapper bypassed | Integration test asserts traceparent presence | Test fails → PR blocked | Use InstrumentedClient |
| Trace_id in logs but field name differs (e.g., trace_id vs traceID) | Loki query fails | Investigate field name | Standardise field name in `logging.rs` |
| Exemplar UI broken (Grafana version) | Manual test fails | Fix Grafana datasource config | Pin Grafana version |
| Subprocess env clears at shell boundary | Tests assert env is set | Use `command.env(...)` not shell `export` | By design |
| LangSmith trace_id format mismatch | Integration test asserts hex format | Test fails | FR-OBS-004 fix |
| Multiple trace_ids per request (regression) | end_to_end_correlation_test asserts unique | PR blocked | Investigate |
| Performance regression from logging overhead | Benchmark | Investigate; reduce per-line work | Profile |
| Span context lost across thread boundary | Integration test fails | Add `Instrument` | By design |
| OTel SDK version mismatch | Compile error | Pin versions | By design |
| Header injection bypass | InstrumentedClient enforced | Code-review check | By design |

---

## §11 — Notes

- W3C TraceContext is widely supported in OTel SDKs across languages — works seamlessly with Python (presidio sidecar, BGE sidecar) if those forward the header.
- The `with_trace_context` wrapper is the entry point for every CyberOS service. Applying it as axum middleware ensures every handler runs in the right context.
- `tracing-subscriber` ObsContextLayer adds trace_id + span_id + tenant_id to every log record without per-call boilerplate. The layer reads from current OTel context; correctness depends on `with_trace_context` being applied.
- Exemplar emission is automatic if the OTel-Prometheus exporter sees a trace_id in current context when `record` is called. The wrapper ensures the context is set.
- The HTTP client wrapper (`InstrumentedClient`) is the only sanctioned way to make outgoing HTTP calls. Direct `reqwest::Client::post` calls would skip injection — code review enforces; future lint could check.
- The end-to-end CI test is the structural enforcement of correlation. It runs the full OTel stack + LangSmith + a synthetic call, then verifies all 4 systems agree on the trace_id. Any propagation gap fails the test.
- Subprocess propagation via env vars (`OTEL_TRACE_ID`, `OTEL_SPAN_ID`) is the standard pattern for OTel cross-process. memory_writer subprocess restores from env at boot.
- Malformed traceparent generates new trace_id (not honors malformed) — security-aware default. Honoring would let attackers poison correlation.
- The `unknown` tenant_id fallback is the operational signal for "caller didn't set baggage." Sev-3 alarm if rate > 1% — investigate which caller is missing the upstream call.

---

*End of FR-OBS-005. Status: draft (10/10 target).*
