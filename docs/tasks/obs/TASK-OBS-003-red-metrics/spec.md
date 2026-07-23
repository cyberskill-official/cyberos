---
id: TASK-OBS-003
title: "Per-service RED metrics (rate/errors/duration) via cyberos-obs-sdk shared crate with macro + CI lint + standardised buckets"
eu_ai_act_risk_class: not_ai
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-15T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: obs
priority: p0
status: ready_to_implement
entered_via: rework
routed_back_count: 1
verify: T
phase: P0
milestone: P0 · slice 2
slice: 1
owner: Stephen Cheng (CTO)
created: 2026-05-15
shipped: null
memory_chain_hash: null
progress: "Substantially implemented (2026-06-20, commits bca06c8..fba003f). DONE: the cyberos-obs-sdk crate (record_request rate/errors/duration, status_class bands, 13 std buckets, cardinality guard); the axum RED middleware (one .layer per service, ADR-OBS-003-001 - chosen over the per-handler proc-macro), wired into the two services that serve HTTP today, auth (TenantCtx from verify_jwt claims) and memory (TenantCtx from the x-tenant-id header); and the OTLP meter-provider/exporter in init() (installs a gRPC meter provider as the global meter when OBS_OTLP_ENDPOINT is set, so the RED metrics export to the TASK-OBS-001 collector; unset = quiet no-op). All build + clippy-clean + tests green; obs gate (awh+caf) green. REMAINING before done: (1) live end-to-end validation against deploy/obs (services need OBS_OTLP_ENDPOINT set to the collector's OTLP receiver - a core-deploy env wiring + validation step); (2) ai-gateway - no HTTP listener exists yet, add the one-line layer + init + TenantCtx when it does; (3) chat - pinned image, no src here. The #[red_instrument] proc-macro + AST completeness lint (§1 #5/#11) are superseded by the middleware per ADR-OBS-003-001 (route coverage is structural)."
related_tasks: [TASK-OBS-001, TASK-OBS-002, TASK-OBS-007, TASK-AI-022]
depends_on: [TASK-OBS-001]
blocks: [TASK-OBS-007, TASK-OBS-005]

source_pages:
  - website/docs/modules/obs.html#red-metrics
source_decisions:
  - DEC-150 (RED metrics across all services; SLO measurement primitive)
  - DEC-151 (shared cyberos-obs-sdk crate; consistency over per-service flexibility)
  - DEC-152 (status_class labels (2xx/3xx/4xx/5xx); raw status would explode cardinality)
  - DEC-153 (standardised histogram buckets across services; cross-service aggregation depends on it)

language: rust 1.81
service: cyberos/crates/cyberos-obs-sdk/
new_files:
  - crates/cyberos-obs-sdk/Cargo.toml
  - crates/cyberos-obs-sdk/src/lib.rs
  - crates/cyberos-obs-sdk/src/red.rs
  - crates/cyberos-obs-sdk/src/macros.rs
  - crates/cyberos-obs-sdk/src/cardinality_guard.rs
  - crates/cyberos-obs-sdk/tests/red_test.rs
  - crates/cyberos-obs-sdk/tests/macro_test.rs
  - crates/cyberos-obs-sdk/tests/cardinality_test.rs
  - crates/cyberos-obs-sdk/tests/instrument_completeness_test.rs
modified_files:
  # apply #[red_instrument]
  - services/ai-gateway/src/handlers/*.rs
  - services/auth/src/admin/*.rs
  - services/chat/src/*
  - services/memory/src/*
  # add cyberos-obs-sdk dep
  - services/ai-gateway/Cargo.toml
  - services/auth/Cargo.toml
  - services/chat/Cargo.toml
  - services/memory/Cargo.toml
allowed_tools:
  - file_read: services/**, crates/cyberos-obs-sdk/**
  - file_write: crates/cyberos-obs-sdk/**, services/**
  - bash: cargo test -p cyberos-obs-sdk
disallowed_tools:
  #2 — required for TASK-OBS-002 tenant filtering)
  - emit RED without tenant_id label (per §1
  - use raw status (e.g., 200 vs 201 distinct) (per DEC-152 — cardinality explosion)
  - hand-roll metrics in services (per DEC-151 — use the shared crate)
  #11 — every handler MUST be instrumented)
  - skip the CI completeness lint (per §1

effort_hours: 8
subtasks:
  - "0.5h: Cargo.toml + lib.rs skeleton"
  - "1.0h: red.rs — `record_request` API + status_class derivation + label set"
  - "1.0h: Standardised histogram buckets (1/2.5/5/10/25/50/100/250/500/1000/2500/5000/10000ms)"
  - "1.0h: macros.rs — `#[red_instrument]` proc macro for axum handlers"
  - "1.0h: cardinality_guard.rs — refuse to register metrics with > 1000 unique label combos"
  - "0.5h: Custom-dimension support (per-service `extra_labels`)"
  - "0.5h: OTel SDK init wiring (services call `obs_sdk::init` at boot)"
  - "1.0h: Apply `#[red_instrument]` to every handler in AI Gateway + AUTH + CHAT + memory"
  - "0.5h: instrument_completeness_test — CI lint asserts every axum handler has the macro"
  - "1.0h: Tests — record + status-class + histogram-buckets + cardinality-guard + macro behaviour"
risk_if_skipped: "TASK-OBS-007 (auto-runbook router) has no signal to trigger on. SLO compliance can't be measured. Performance regressions invisible. Without standardised buckets, cross-service p95 aggregation is impossible (different bucket boundaries → no comparable percentiles). Without the macro + CI lint, services drift away from instrumentation discipline — by slice 4 most handlers are uninstrumented and the OBS pillar's value is gone."
---

## §1 — Description (BCP-14 normative)

A shared Rust crate `cyberos-obs-sdk` **MUST** expose RED-metric helpers; every CyberOS service **MUST** emit these metrics on every request. Each component:

1. **MUST** expose `red::record_request(service, route, tenant_id, status, duration_ms, extra_labels)` API. The first 5 params are mandatory; `extra_labels: &[(&str, String)]` is optional per-service customisation.
2. **MUST** emit 3 metrics per request:
- `cyberos_requests_total{service, route, tenant_id, status_class}` (counter; increments by 1 per call).
- `cyberos_errors_total{service, route, tenant_id, error_class}` (counter; increments only on errors; `error_class` ∈ `client_error | server_error`).
- `cyberos_duration_ms{service, route, tenant_id}` (histogram; bucket boundaries in §1 #4).
3. **MUST** label by `status_class` (string: `"2xx" | "3xx" | "4xx" | "5xx" | "other"`) NOT raw status code. Per DEC-152: raw status (200, 201, 204, ...) would multiply cardinality by ~30; the class-level granularity is sufficient for SLO calculation.
4. **MUST** use standardised histogram bucket boundaries: `[1, 2.5, 5, 10, 25, 50, 100, 250, 500, 1000, 2500, 5000, 10000]` ms. Per DEC-153: cross-service p95 aggregation requires identical bucket boundaries (Prometheus's `histogram_quantile` won't merge differently-bucketed histograms correctly).
5. **MUST** be called from every HTTP/RPC handler in every CyberOS service. The `#[red_instrument(service = "...", route = "...")]` proc macro on axum handlers is the standard application path; manual calls are reserved for non-HTTP code paths (background jobs, scheduled tasks).
6. **MUST** add ≤ 100ns overhead per call (atomic counter increment + HDR-histogram bucket increment + Vec push for label array). The OTel SDK in async mode is built for this. Benchmarked via `red_overhead_test.rs`.
7. **MUST** integrate via OTel SDK (NOT direct prometheus crate). Same emission path as TASK-AI-022 traces; same OTLP exporter; same backend.
8. **MUST** support custom dimensions per service via `extra_labels`. AI Gateway adds `model_alias`; CHAT adds `workspace_id`; AUTH adds `endpoint_type` (admin vs token-issue). Custom labels MUST pass cardinality guard (§1 #9).
9. **MUST** enforce a cardinality guard: refuses to register a metric series with > 1000 unique label combinations per service. Cardinality explosion is the most common observability failure mode (Prometheus stores one time-series per label combo; thousands of series per metric balloons storage). The guard logs `cardinality_overflow_blocked` and emits `obs_sdk_cardinality_blocked_total{service, metric}` counter; sev-2 alarm.
10. **MUST** init OTel SDK at service boot via `obs_sdk::init(service_name, version)`. The init function configures the OTLP exporter to point at the OBS collector (TASK-OBS-001) using the per-service bearer token from env.
11. **MUST** be CI-gated by `instrument_completeness_test`: an integration test in `cyberos-obs-sdk/tests/` AST-walks every CyberOS service's handler files and asserts each axum-handler-shaped function has `#[red_instrument]`. Missing macro → CI failure with file:line of the offending handler.
12. **SHOULD** emit OTel metrics about itself:
- `obs_sdk_record_calls_total{service}` (counter; total record_request calls).
- `obs_sdk_record_latency_ns` (histogram; per-call overhead).
- `obs_sdk_cardinality_blocked_total{service, metric}` (counter; sev-2 alarm).

---

## §2 — Why this design (rationale for humans)

**Why RED specifically (DEC-150)?** RED (Rate, Errors, Duration) is the standard service-level SLO trinity. Three metrics tell ops everything they need to triage: how busy am I, what's failing, how slow am I. USE (Utilization, Saturation, Errors) is the alternative for resource-level monitoring; RED is the right level for service-oriented systems.

**Why a shared crate (DEC-151)?** Every service emitting metrics with slightly different label sets would make cross-service queries impossible. The shared crate enforces one label vocabulary (`service`, `route`, `tenant_id`, `status_class`, `error_class`); operators learn one query language.

**Why status_class not raw status (DEC-152)?** Cardinality. Each unique label combo = one Prometheus time-series. Raw status × routes × tenants = N × M × T series; status_class × routes × tenants = 4 × M × T (4 = 2xx/3xx/4xx/5xx). 30× cardinality reduction with negligible operational cost (operators rarely care about 201 vs 200; they care about 2xx vs 4xx).

**Why standardised histogram buckets (DEC-153)?** Cross-service aggregation requires identical bucket boundaries. `histogram_quantile(0.95, sum by (service) (rate(cyberos_duration_ms_bucket[5m])))` only works if every service's histogram uses the same buckets. The 13 standard buckets cover 1ms-10s with reasonable resolution at each magnitude.

**Why the macro form (#[red_instrument]) (§1 #5)?** Manual calls scatter `record_request` lines across the codebase, easy to forget. The proc macro wraps the handler function transparently — no per-handler boilerplate. The CI completeness test (§1 #11) ensures the macro is applied; together they enforce instrumentation as a structural property.

**Why ≤100ns overhead (§1 #6)?** Per-request metric overhead must be invisible. At 1000 req/s, 100ns × 1000 = 100µs/sec — 0.01% CPU. Above 100ns, the overhead becomes measurable; above 1µs, observably slows the service. The OTel Rust SDK is built for this overhead profile.

**Why cardinality guard (§1 #9)?** The most common observability failure: a developer adds `user_id` as a metric label, intending to track per-user latency. With 10K users + 50 routes × 50 tenants = 25M time-series. Prometheus storage explodes; queries slow to seconds. The cardinality guard refuses to register such metrics — operators see the rejection at deploy time, not after Prometheus crashes.

**Why CI lint for instrument completeness (§1 #11)?** Without enforcement, instrumentation discipline drifts. New handlers ship without `#[red_instrument]`; the metric coverage gradually shrinks. The CI lint catches uninstrumented handlers at PR time — a code-review surface that's unambiguous.

**Why custom dimensions per service (§1 #8)?** Some services have natural per-request dimensions worth tracking. AI Gateway: `model_alias` (chat.fast vs chat.smart). CHAT: `workspace_id`. AUTH: `endpoint_type`. The shared label vocabulary covers the basics; per-service custom dims add detail without forcing every service to carry every dimension.

---

## §3 — API contract

```rust
// crates/cyberos-obs-sdk/src/red.rs
use opentelemetry::{global, KeyValue, metrics::{Counter, Histogram, Meter}};
use std::sync::OnceLock;

const HISTOGRAM_BUCKETS_MS: &[f64] = &[1.0, 2.5, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0, 10000.0];

static REQUESTS: OnceLock<Counter<u64>> = OnceLock::new();
static ERRORS: OnceLock<Counter<u64>> = OnceLock::new();
static DURATION: OnceLock<Histogram<f64>> = OnceLock::new();

pub fn init(service: &str, version: &str) {
    let meter = global::meter("cyberos");
    REQUESTS.set(meter.u64_counter("cyberos_requests_total").build()).unwrap_or(());
    ERRORS.set(meter.u64_counter("cyberos_errors_total").build()).unwrap_or(());
    DURATION.set(meter.f64_histogram("cyberos_duration_ms")
        .with_unit("ms")
        .with_boundaries(HISTOGRAM_BUCKETS_MS.to_vec())
        .build()).unwrap_or(());
    cardinality_guard::init(service);
}

pub fn record_request(
    service: &str, route: &str, tenant_id: &str,
    status: u16, duration_ms: u32, extra_labels: &[(&str, String)],
) {
    let status_class = status_class(status);
    let mut labels = vec![
        KeyValue::new("service", service.to_string()),
        KeyValue::new("route", route.to_string()),
        KeyValue::new("tenant_id", tenant_id.to_string()),
        KeyValue::new("status_class", status_class.to_string()),
    ];
    for (k, v) in extra_labels {
        labels.push(KeyValue::new(k.to_string(), v.clone()));
    }
    if !cardinality_guard::check(service, "cyberos_requests_total", &labels) {
        return;   // refused — would overflow cardinality
    }
    REQUESTS.get().unwrap().add(1, &labels);
    DURATION.get().unwrap().record(duration_ms as f64, &labels);
    if status >= 400 {
        let mut err_labels = labels.clone();
        let error_class = if status >= 500 { "server_error" } else { "client_error" };
        err_labels.push(KeyValue::new("error_class", error_class.to_string()));
        ERRORS.get().unwrap().add(1, &err_labels);
    }
}

fn status_class(status: u16) -> &'static str {
    match status {
        200..=299 => "2xx", 300..=399 => "3xx",
        400..=499 => "4xx", 500..=599 => "5xx",
        _ => "other",
    }
}
```

```rust
// crates/cyberos-obs-sdk/src/macros.rs
#[proc_macro_attribute]
pub fn red_instrument(args: TokenStream, item: TokenStream) -> TokenStream {
    let attrs: RedInstrumentArgs = parse_macro_input!(args);   // service = "...", route = "..."
    let func = parse_macro_input!(item as ItemFn);
    quote! {
        async fn #func_name(#args) -> #return_type {
            let __t0 = std::time::Instant::now();
            let __tenant_id = extract_tenant_id_from_request(&request);   // helper or fallback
            let __result = (#original_body)(request).await;
            let __status = match &__result {
                Ok(resp) => resp.status().as_u16(),
                Err(e)   => 500,
            };
            cyberos_obs_sdk::red::record_request(
                #service, #route, &__tenant_id, __status,
                __t0.elapsed().as_millis() as u32, &[],
            );
            __result
        }
    }.into()
}
```

```rust
// crates/cyberos-obs-sdk/src/cardinality_guard.rs
use std::collections::HashMap;
use std::sync::Mutex;

const MAX_CARDINALITY_PER_METRIC: usize = 1000;

static SEEN: Mutex<HashMap<String, HashSet<String>>> = Mutex::new(HashMap::new());

pub fn check(service: &str, metric: &str, labels: &[KeyValue]) -> bool {
    let key = format!("{service}:{metric}");
    let label_combo = labels.iter().map(|kv| format!("{}={}", kv.key, kv.value.as_str())).collect::<Vec<_>>().join(",");
    let mut seen = SEEN.lock().unwrap();
    let entry = seen.entry(key.clone()).or_default();
    if entry.contains(&label_combo) { return true; }
    if entry.len() >= MAX_CARDINALITY_PER_METRIC {
        eprintln!("cardinality_overflow_blocked: {service}/{metric} at {} unique combos", entry.len());
        cardinality_blocked_counter().add(1, &[KeyValue::new("service", service.to_string()), KeyValue::new("metric", metric.to_string())]);
        return false;
    }
    entry.insert(label_combo);
    true
}
```

---

## §4 — Acceptance criteria

1. **100 chat calls produce 100 `cyberos_requests_total{service=ai-gateway,...}` increments**.
2. **5xx errors produce both requests AND errors increments** — `errors_total{error_class="server_error"}` increments by 1.
3. **4xx errors produce errors with error_class=client_error**.
4. **Histogram bucket fills correct** — call with duration 7ms increments the `le="10"` bucket.
5. **Per-tenant labels visible** — PromQL `cyberos_duration_ms{tenant_id="org:cyberskill"}` returns data.
6. **Status class derivation correct** — 200/201/204 all → `status_class=2xx`.
7. **Macro applied to all handlers** — `instrument_completeness_test` AST-walks every CyberOS service's handlers; asserts every axum-handler-shaped fn has `#[red_instrument]`.
8. **Overhead < 100ns per call** — `red_overhead_test` benchmark assertion.
9. **Cardinality guard refuses > 1000 combos** — synthetic test inserts 1001 unique combos; 1001st call returns false; `obs_sdk_cardinality_blocked_total` increments; sev-2 alarm.
10. **Custom dimensions work** — `extra_labels: &[("model_alias", "chat.smart".into())]` appears as label in emitted metric.
11. **Standardised buckets across services** — query `cyberos_duration_ms_bucket{service=ai-gateway} or cyberos_duration_ms_bucket{service=auth-service}` returns same `le` values.
12. **OTel SDK init successful** — boot logs `obs_sdk_initialized service=ai-gateway version=0.4.1`.
13. **Unrecognised status returns "other" class** — status=999 → `status_class=other`.
14. **Concurrent record_request thread-safe** — 100 threads × 1000 calls; final counter = 100,000.
15. **Macro preserves handler signature** — handler's params + return type unchanged after macro expansion.

---

## §5 — Verification

```rust
// crates/cyberos-obs-sdk/tests/red_test.rs
#[test]
fn record_emits_counter_and_histogram() {
    let exporter = test_helper::test_exporter();
    obs_sdk::red::init("test", "0.0.0");

    obs_sdk::red::record_request("test", "/foo", "tenant_a", 200, 42, &[]);

    let m = exporter.find_counter("cyberos_requests_total")
        .with_labels(&[("service", "test"), ("route", "/foo"), ("tenant_id", "tenant_a"), ("status_class", "2xx")])
        .unwrap();
    assert_eq!(m.value(), 1);

    let h = exporter.find_histogram("cyberos_duration_ms")
        .with_labels(&[("service", "test"), ("route", "/foo"), ("tenant_id", "tenant_a")])
        .unwrap();
    assert!(h.bucket_count_at(50.0) >= 1);
}

#[test]
fn server_error_emits_error_counter() {
    obs_sdk::red::record_request("test", "/foo", "t", 503, 100, &[]);
    let err = test_helper::find_counter("cyberos_errors_total")
        .with_labels(&[("error_class", "server_error")]).unwrap();
    assert_eq!(err.value(), 1);
}

#[test]
fn client_error_emits_error_counter_with_client_class() {
    obs_sdk::red::record_request("test", "/foo", "t", 404, 50, &[]);
    let err = test_helper::find_counter("cyberos_errors_total")
        .with_labels(&[("error_class", "client_error")]).unwrap();
    assert_eq!(err.value(), 1);
}

#[test]
fn status_class_derivation() {
    for (status, expected) in &[(200, "2xx"), (201, "2xx"), (301, "3xx"), (404, "4xx"), (503, "5xx"), (999, "other")] {
        assert_eq!(red::status_class(*status), *expected);
    }
}

#[test]
fn cardinality_guard_blocks_at_1001() {
    obs_sdk::cardinality_guard::reset();
    for i in 0..1000 {
        obs_sdk::red::record_request("test", &format!("/route_{i}"), "t", 200, 1, &[]);
    }
    let allowed = obs_sdk::cardinality_guard::check("test", "cyberos_requests_total", &[KeyValue::new("route", "/route_1000".to_string())]);
    assert!(!allowed);
    let blocked: u64 = test_helper::counter_value("obs_sdk_cardinality_blocked_total", &[("service", "test"), ("metric", "cyberos_requests_total")]);
    assert!(blocked > 0);
}

#[bench]
fn red_overhead_under_100ns(b: &mut Bencher) {
    obs_sdk::red::init("test", "0.0.0");
    b.iter(|| obs_sdk::red::record_request("test", "/foo", "t", 200, 1, &[]));
    // Cargo bench output: each iter measured; assert < 100ns
}

#[tokio::test]
async fn concurrent_record_thread_safe() {
    let mut joinset = tokio::task::JoinSet::new();
    for _ in 0..100 {
        joinset.spawn(async {
            for _ in 0..1000 {
                obs_sdk::red::record_request("test", "/c", "t", 200, 1, &[]);
            }
        });
    }
    while let Some(_) = joinset.join_next().await {}
    let m = test_helper::counter_value("cyberos_requests_total",
        &[("service", "test"), ("route", "/c"), ("status_class", "2xx")]);
    assert_eq!(m, 100_000);
}
```

```rust
// crates/cyberos-obs-sdk/tests/instrument_completeness_test.rs
#[test]
fn every_axum_handler_has_red_instrument() {
    use std::path::Path;
    use syn::visit::Visit;
    let services = ["services/ai-gateway", "services/auth", "services/chat", "services/memory"];
    let mut missing = vec![];
    for service in services {
        for entry in walkdir::WalkDir::new(format!("{service}/src/handlers")) {
            let entry = entry.unwrap();
            if !entry.path().extension().map(|e| e == "rs").unwrap_or(false) { continue; }
            let src = std::fs::read_to_string(entry.path()).unwrap();
            let file: syn::File = syn::parse_str(&src).unwrap();
            for item in &file.items {
                if let syn::Item::Fn(f) = item {
                    if is_axum_handler(f) {
                        let has_macro = f.attrs.iter().any(|a| a.path().is_ident("red_instrument"));
                        if !has_macro {
                            missing.push(format!("{}::{} (line {})", entry.path().display(), f.sig.ident, f.sig.span().start().line));
                        }
                    }
                }
            }
        }
    }
    assert!(missing.is_empty(), "axum handlers missing #[red_instrument]:\n  {}", missing.join("\n  "));
}
```

```bash
cargo test -p cyberos-obs-sdk
cargo bench -p cyberos-obs-sdk red_overhead
```

---

## §6 — Implementation skeleton

See §3.

```rust
// services/ai-gateway/src/handlers/chat.rs (after macro applied)
#[red_instrument(service = "ai-gateway", route = "/v1/chat/completions")]
async fn handle_chat(req: ChatCompleteRequest, headers: HeaderMap) -> Result<HttpResponse, ApiError> {
    // existing logic unchanged
}

// Custom dimension example
#[red_instrument(service = "ai-gateway", route = "/v1/chat/completions",
                 extra_labels = "&[(\"model_alias\", req.model_alias.clone())]")]
async fn handle_chat(req: ChatCompleteRequest, ...) -> ... { ... }
```

Boot:

```rust
// services/ai-gateway/src/main.rs
fn main() {
    cyberos_obs_sdk::red::init("ai-gateway", env!("CARGO_PKG_VERSION"));
    // ... rest of boot ...
}
```

---

## §7 — Dependencies

- **TASK-OBS-001** — OTel collector live (otherwise metrics queue locally).
- **TASK-OBS-002 (downstream)** — Tenant proxy depends on tenant_id label being on every metric.
- **TASK-OBS-007 (downstream)** — Alertmanager rules query RED metrics.
- Crates: `opentelemetry@0.21`, `opentelemetry-otlp`, `once_cell`, `proc-macro2`, `syn@2`, `quote`, `walkdir@2` (test-only), `criterion@0.5` (bench).

---

## §8 — Example payloads

### PromQL queries operators run

```promql
# Error rate by service
sum(rate(cyberos_errors_total[5m])) by (service)

# p95 latency per route
histogram_quantile(0.95, sum(rate(cyberos_duration_ms_bucket[5m])) by (route, le))

# 5xx rate per tenant
sum(rate(cyberos_errors_total{error_class="server_error"}[5m])) by (tenant_id)

# RED triplet for ai-gateway
rate(cyberos_requests_total{service="ai-gateway"}[5m])
rate(cyberos_errors_total{service="ai-gateway"}[5m])
histogram_quantile(0.95, rate(cyberos_duration_ms_bucket{service="ai-gateway"}[5m]))
```

### Cardinality-guard alert

```text
WARN  service=ai-gateway metric=cyberos_requests_total
      cardinality_overflow_blocked at 1000 unique label combos
sev-2 obs_sdk_cardinality_blocked_total{service=ai-gateway, metric=cyberos_requests_total} incremented
```

---

## §9 — Open questions

All resolved. Deferred:
- Per-handler SLO targets in code annotations — slice 4+.
- Synthetic-traffic generator for SLO measurement under no-load — slice 5+.
- Streaming-response duration measurement (start vs first byte vs last byte) — slice 4+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| OTel SDK panic on init | Process exit at boot | Service refuses to start | Operator investigates SDK version |
| OTel SDK runtime error | Catch-all in record | Log + continue (no-op) | Investigate SDK |
| High cardinality (>1000 combos) | cardinality_guard.check | Refuse to register; sev-2 alarm | Engineer fixes label set |
| Service forgets to instrument | `instrument_completeness_test` | PR blocked | Engineer adds macro |
| Service forgets `obs_sdk::init` at boot | `record_request` panic on `unwrap()` | Boot fails | Operator adds init call |
| Standardised buckets violated (operator changes per service) | cross-service aggregation breaks | Detection in dashboards | Lint enforcement |
| Macro doesn't preserve handler signature | Compile error | PR blocked | Fix macro impl |
| Concurrent record race | Atomic counters | No race | By design |
| Tenant_id label missing | Macro extracts; falls back to "unknown" | Sev-3 alarm if "unknown" > 1% | Operator investigates extraction |
| Histogram bucket overflow (latency > 10s) | `+Inf` bucket increments | Sev-3 alarm; investigate slow handler | By design |
| Custom dimension explodes cardinality | cardinality_guard catches | Refuse to register | Engineer fixes custom-label set |
| Metric not in OTel collector pipeline | Collector config missing exporter | Metrics not visible in Prometheus | Operator updates collector config |
| Status code 0 (timeout pre-response) | status_class derives "other" | Visible as "other" | By design |
| Macro syntax breakage on edge-case handler | Compile error | PR blocked | Fix macro impl |
| Disabled in test (no metrics emitted) | Tests use no-op exporter | N/A | By design |
| OTel collector down → metrics queue | OTel SDK buffers | Local buffer; eventual delivery | TASK-OBS-001 file_storage |
| Cardinality 999 → adds 1 → 1000 → at limit | Allow that 1000th combo | By design (the 1001st is blocked) | By design |

---

## §11 — Notes

- The macro form (`#[red_instrument]`) keeps call sites clean. Manual calls are reserved for non-HTTP code paths (background jobs).
- Histogram buckets MUST match across services for cross-service aggregation. The 13 standard boundaries cover 1ms-10s with reasonable resolution at each magnitude.
- Cardinality guard at 1000 unique combos is conservative. Slice 1 services typically have 5-10 routes × 50 tenants × 4 status classes = 2000 combos. Above 1000 per metric, operators investigate; the limit can be raised with explicit task amendment.
- The `instrument_completeness_test` is the structural enforcement of "every handler is instrumented." Without it, drift is inevitable.
- Status_class labels (2xx/3xx/4xx/5xx/other) trade fine-grained visibility for cardinality. Operators rarely care about 201 vs 200; they care about 2xx vs 4xx. The trade-off is worth it.
- The `extra_labels` parameter is the per-service flexibility surface. AI Gateway adds `model_alias`; CHAT adds `workspace_id`. Each service's PR includes the rationale for new labels.
- Overhead < 100ns is achievable because OTel SDK uses lock-free atomic counters + HDR-style histogram bucket increments. The benchmark catches regressions.
- The OTel SDK init at boot configures the OTLP exporter. If the OBS collector is down at boot, OTel buffers locally; TASK-OBS-001's file_storage extension persists across restart.
- Future SLO annotations (`#[red_instrument(slo_p95 = "100ms")]`) would let the macro auto-generate Prometheus rules. Slice 4+ work.

---

*End of TASK-OBS-003. Status: draft (10/10 target).*
