---
id: FR-OBS-002
title: "Tenant-aware Grafana proxy (Rust) — AST-injects tenant_id into PromQL/LogQL/TraceQL with anti-bypass + property test + audit log"
module: OBS
priority: MUST
status: accepted
verify: T
phase: P0
milestone: P0 · slice 2
slice: 1
owner: Stephen Cheng (CTO)
created: 2026-05-15
shipped: null
brain_chain_hash: null
related_frs: [FR-OBS-001, FR-OBS-007, FR-OBS-008, FR-OBS-009, FR-AUTH-004, FR-AUTH-108, FR-AI-018]
depends_on: [FR-OBS-001, FR-AUTH-004]
blocks: [FR-OBS-007, FR-OBS-008]

source_pages:
  - website/docs/modules/obs.html#tenant-proxy
source_decisions:
  - DEC-145 (Grafana OSS + Rust proxy; Grafana Enterprise multi-tenancy too expensive)
  - DEC-146 (AST-based injection, NEVER string concat — string concat invites injection bypass)
  - DEC-147 (cross-tenant query attempt = sev-1 audit + 400; never return data even by accident)
  - PDPL Art. 11 (data minimisation per role)

language: rust 1.81
service: cyberos/services/obs-proxy/
new_files:
  - services/obs-proxy/Cargo.toml
  - services/obs-proxy/src/main.rs
  - services/obs-proxy/src/proxy.rs
  - services/obs-proxy/src/auth.rs
  - services/obs-proxy/src/inject/promql.rs
  - services/obs-proxy/src/inject/logql.rs
  - services/obs-proxy/src/inject/traceql.rs
  - services/obs-proxy/src/audit.rs
  - services/obs-proxy/tests/proxy_test.rs
  - services/obs-proxy/tests/inject_promql_test.rs
  - services/obs-proxy/tests/inject_logql_test.rs
  - services/obs-proxy/tests/inject_traceql_test.rs
  - services/obs-proxy/tests/cross_tenant_property_test.rs
  - services/obs-proxy/tests/audit_log_test.rs
modified_files:
  - deploy/obs/grafana/datasources.yaml                     # point datasources at obs-proxy
  - deploy/obs/docker-compose.yml                          # add obs-proxy service
allowed_tools:
  - file_read: services/obs-proxy/**
  - file_write: services/obs-proxy/{src,tests}/**
  - file_write: deploy/obs/grafana/datasources.yaml
  - file_write: deploy/obs/docker-compose.yml
  - bash: cd services/obs-proxy && cargo test
disallowed_tools:
  - bypass tenant_id label injection on any query path (per §1 #2 — every query injected)
  - use string concat for label injection (per DEC-146 — AST only)
  - allow user-supplied tenant_id label (per §1 #4 — bypass attempt; 400 + sev-1)
  - skip audit log on any query (per §1 #9 — every query is auditable)

effort_hours: 12
sub_tasks:
  - "0.5h: Cargo.toml + crate skeleton (axum, reqwest, promql-parser, jsonwebtoken)"
  - "1.0h: auth.rs — JWT verification (FR-AUTH-004 JWKS) + tenant_id extraction"
  - "1.5h: inject/promql.rs — AST-based label injection using promql-parser"
  - "1.5h: inject/logql.rs — LogQL parser + label injection (handles pipe stages)"
  - "1.0h: inject/traceql.rs — TraceQL injection (resource.tenant_id filter)"
  - "1.0h: proxy.rs — backend detection + injection dispatch + forward + response stream"
  - "0.5h: anti-bypass: reject queries containing user-supplied tenant_id label"
  - "0.5h: cross-tenant attempt detection (caller_tenant != requested_tenant) = sev-1"
  - "0.5h: audit.rs — log every query with (tenant_id, backend, query, outcome) to BRAIN"
  - "0.5h: OTel metrics + spans"
  - "1.0h: proxy_test.rs — happy + 401 + 400 + cross-tenant + audit-emit"
  - "1.5h: inject tests per backend (PromQL/LogQL/TraceQL) — happy + complex query + bypass attempts"
  - "0.5h: cross_tenant_property_test.rs — proptest 1000 random queries × 2 tenant pairs (mirrors FR-AI-018)"
  - "0.5h: docker-compose integration + datasources.yaml pointing at proxy"
risk_if_skipped: "Every Grafana query sees ALL tenants' data. PDPL Art. 11 (data minimisation per role) violated. CHRO viewing a finance dashboard could accidentally see another tenant's payroll metrics. Multi-tenancy invariant breaks at the OBS layer — even though FR-OBS-001 stores tenant_id labels, queries returning all tenants makes the labels worthless. Without AST-based injection, string-concat label injection is bypass-able (e.g., a query containing `}` in a regex breaks the injected wrapper)."
---

## §1 — Description (BCP-14 normative)

A Rust HTTP proxy **MUST** sit between Grafana and the 3 OBS backends (Loki, Prometheus, Tempo), AST-injecting `tenant_id` label filters into every query. Each request:

1. **MUST** authenticate via FR-AUTH-004 JWT (verified against JWKS at boot + cached). The JWT carries `tenant_id` claim; extracted as the injection value. Missing or invalid JWT → `401 UNAUTHORIZED`.
2. **MUST** inject `tenant_id="<extracted>"` label filter into EVERY PromQL / LogQL / TraceQL query before forwarding. The injection is AST-based (parse → mutate → reserialize), NEVER string concatenation. AST-based injection is robust against escape attempts (regex characters in user query, multi-line queries, etc.).
3. **MUST** AND the injected filter with any user-supplied query (logical conjunction; never OR). Example: `rate(foo[5m])` with tenant `T` → `rate(foo{tenant_id="T"}[5m])`. The user's query is preserved exactly; only the label set is augmented.
4. **MUST** refuse queries that ALREADY contain `tenant_id` label as a user-supplied filter — this prevents bypass attempts where a user crafts `foo{tenant_id="other"}` hoping the proxy will OR-merge or pass through. Such queries return `400 BAD_REQUEST` with `{"error":"user_supplied_tenant_id","reason":"tenant_id label is reserved; do not include in query"}`. Sev-1 audit row `obs.cross_tenant_query_attempt` emitted.
5. **MUST** support all 3 query languages with per-language AST parser:
    - PromQL via `promql-parser@0.4` (Prometheus official Rust port).
    - LogQL via custom parser (no mature crate; ~300 LoC subset covering selector + pipe stages).
    - TraceQL via custom parser (~200 LoC subset).
6. **MUST** complete proxying in ≤ 5ms p95 overhead (parse + inject + serialise; backend latency excluded). The 5ms ceiling protects user experience — Grafana already round-trips multiple queries per dashboard refresh.
7. **MUST** forward responses unchanged (the backends already namespace by tenant_id label; nothing to filter response-side).
8. **MUST** detect cross-tenant attempts: if the caller's `tenant_id` claim differs from any explicit `tenant_id` in the query (caught by §1 #4) OR the JWT's tenant_id is the nil UUID (root-admin) — root-admin queries are special-cased per §1 #11.
9. **MUST** emit a BRAIN audit row `obs.query_proxied` per query with payload: `tenant_id`, `caller_subject_id`, `backend` (loki|prometheus|tempo), `query_sha256`, `outcome` (proxied | rejected_user_supplied_tenant_id | rejected_unauthenticated | backend_error), `latency_ms`, `request_id`. Logging the SHA-256 of the query (not the raw query) preserves privacy if the query contains tenant-business semantics.
10. **MUST** emit OTel metrics:
    - `obs_proxy_requests_total{tenant_id, backend, outcome}` (counter).
    - `obs_proxy_injection_latency_ms{backend}` (histogram; SLO p95 < 5ms).
    - `obs_proxy_cross_tenant_attempts_total{tenant_id}` (counter; sev-1 alarm on increment).
    - `obs_proxy_backend_latency_ms{backend}` (histogram).
11. **MUST** support root-admin (tenant 0) queries WITHOUT injection — root-admin legitimately queries cross-tenant for ops/compliance. The exception is logged via `obs.query_proxied` row with `outcome: root_admin_unfiltered` AND emits sev-2 (informational) so cross-tenant queries are visible to compliance review.
12. **SHOULD** cache JWKS lookups for 5 minutes (matches FR-AUTH-004 §1 #3 cache header).
13. **SHOULD** support all standard Grafana query endpoints:
    - Prometheus: `/api/v1/query`, `/api/v1/query_range`, `/api/v1/labels`, `/api/v1/series`.
    - Loki: `/loki/api/v1/query`, `/loki/api/v1/query_range`, `/loki/api/v1/labels`.
    - Tempo: `/api/search`, `/api/traces/<id>`.

---

## §2 — Why this design (rationale for humans)

**Why Rust proxy not Grafana Enterprise (DEC-145)?** Grafana Enterprise multi-tenancy costs $$$/month per user + adds operational complexity (separate auth integration, license management, version-locked features). A 500-LoC Rust proxy solves the same problem with zero licensing fee and full control.

**Why AST-based injection not string concat (DEC-146)?** String concat is bypass-able. Example: query `foo{x="}` (broken regex) + naive concat `foo{x="}, tenant_id="T"}` produces malformed PromQL. Worse: a query with multiple selector groups `foo or bar` would only get tenant_id added to one. AST-based parsing produces a structured tree; mutating every selector is straightforward and safe.

**Why reject user-supplied `tenant_id` label (§1 #4)?** A user crafting `foo{tenant_id="other"}` is ATTEMPTING a bypass. Two safe options: (a) overwrite their value, (b) reject. Rejecting is louder + auditable + signals intent. Overwriting hides the attempt. Reject + sev-1 audit + 400 is the chosen path.

**Why per-language AST parsers (§1 #5)?** PromQL, LogQL, and TraceQL have different syntax. A unified parser would compromise on each. Per-language parsers (PromQL via official crate; LogQL/TraceQL hand-rolled subsets) handle each correctly. The hand-rolled parsers cover only the syntax actually used by Grafana — full grammar coverage isn't needed.

**Why log query SHA-256 not raw query (§1 #9)?** Queries can contain tenant-business semantics (e.g., `kb_articles_total{kb="legal-hr-confidential"}`). Logging raw queries to BRAIN exposes those semantics to anyone with audit access. Hash preserves uniqueness for forensic correlation without leaking content. If forensic needs the raw query, ops re-runs against the proxy with the same tenant + observe what was sent.

**Why root-admin gets unfiltered access (§1 #11)?** Root-admin's job includes cross-tenant ops queries (compliance reports, ops investigations). Forcing root-admin to use a different tool would split the workflow. The audit-row `outcome: root_admin_unfiltered` makes legitimate cross-tenant queries visible — compliance reviews can ask "who did cross-tenant queries last week?" and get a list.

**Why 5ms p95 overhead (§1 #6)?** Grafana dashboards refresh every 5-30s; a typical refresh queries 10-50 panels. 5ms × 50 panels = 250ms per refresh — invisible to humans. Above 5ms, refreshes become sluggish.

**Why audit-row logging for every query (§1 #9)?** Compliance audit answer: "show me what queries this operator ran during the period under review." Without per-query audit, the answer is "we don't know." With it, the answer is a SQL query. The cost is one BRAIN row per query (~10K queries/day at slice-1 scale = trivial storage).

**Why proxy IS the security boundary (§11 note in original)?** Grafana itself runs as a non-tenant-aware app — its built-in auth + RBAC don't enforce tenant isolation at query time. The proxy enforces tenant isolation by injecting labels at EVERY query. Without the proxy, any Grafana user (regardless of role) sees all tenants' data.

---

## §3 — API contract

```rust
// services/obs-proxy/src/proxy.rs
pub async fn proxy(req: Request<Body>, state: &State) -> Result<Response<Body>, ProxyError> {
    let claims = state.auth.verify(&req).await?;
    let backend = detect_backend(&req.uri());
    let body = read_body(&mut req).await?;
    let original_query = extract_query(&body, backend)?;

    if has_user_supplied_tenant_id(&original_query, backend) {
        audit::emit(brain::canonical::cross_tenant_query_attempt(&claims, &original_query)).await;
        return Err(ProxyError::UserSuppliedTenantId);
    }

    let injected = if claims.tenant_id == Uuid::nil() && claims.roles.contains(&"root-admin".into()) {
        audit::emit(brain::canonical::root_admin_unfiltered(&claims, &original_query)).await;
        original_query   // root-admin: no injection
    } else {
        match backend {
            Backend::Prometheus => inject::promql::add_label(&original_query, "tenant_id", &claims.tenant_id.to_string())?,
            Backend::Loki       => inject::logql::add_label(&original_query, "tenant_id", &claims.tenant_id.to_string())?,
            Backend::Tempo      => inject::traceql::add_label(&original_query, "resource.tenant_id", &claims.tenant_id.to_string())?,
        }
    };

    let resp = forward(backend, &injected, &state.http).await?;
    audit::emit(brain::canonical::query_proxied(&claims, backend, &original_query, "proxied")).await;
    Ok(resp)
}

#[derive(Debug, thiserror::Error)]
pub enum ProxyError {
    #[error("auth failed: {0}")]
    AuthFailed(String),
    #[error("user supplied tenant_id label (bypass attempt)")]
    UserSuppliedTenantId,
    #[error("query parse failed: {backend:?}: {reason}")]
    ParseFailed { backend: Backend, reason: String },
    #[error("backend error: {0}")]
    Backend(#[from] reqwest::Error),
}

#[derive(Debug, Clone, Copy)]
pub enum Backend { Prometheus, Loki, Tempo }
```

```rust
// services/obs-proxy/src/inject/promql.rs
use promql_parser::parser::{parse, Expr};

pub fn add_label(query: &str, key: &str, value: &str) -> Result<String, ProxyError> {
    let mut ast = parse(query).map_err(|e| ProxyError::ParseFailed {
        backend: Backend::Prometheus, reason: e.to_string(),
    })?;
    visit_selectors_mut(&mut ast, |sel| {
        sel.matchers.push(promql_parser::label::Matcher::new(
            promql_parser::label::MatchOp::Equal, key.into(), value.into(),
        ));
    });
    Ok(ast.prettify())
}

pub fn has_label(query: &str, key: &str) -> Result<bool, ProxyError> {
    let ast = parse(query).map_err(/* ... */)?;
    let mut found = false;
    visit_selectors(&ast, |sel| {
        if sel.matchers.iter().any(|m| m.name == key) { found = true; }
    });
    Ok(found)
}
```

```rust
// services/obs-proxy/src/inject/logql.rs (sketch — hand-rolled)
pub fn add_label(query: &str, key: &str, value: &str) -> Result<String, ProxyError> {
    // Find the selector portion `{...}` at start of query
    let (selector, rest) = split_selector(query)?;
    let labels = parse_labels(selector)?;
    if labels.iter().any(|l| l.key == key) {
        return Err(ProxyError::ParseFailed {
            backend: Backend::Loki,
            reason: format!("query already contains {key} label"),
        });
    }
    let mut new_labels = labels;
    new_labels.push(Label { key: key.into(), op: "=".into(), value: format!("\"{value}\"") });
    Ok(format!("{{{}}}{}",
        new_labels.iter().map(Label::to_string).collect::<Vec<_>>().join(","),
        rest))
}
```

---

## §4 — Acceptance criteria

1. **PromQL without tenant_id → injected** — `rate(foo[5m])` for tenant T → `rate(foo{tenant_id="T"}[5m])`.
2. **PromQL with existing labels → tenant_id added** — `foo{x="y"}` → `foo{x="y",tenant_id="T"}`.
3. **PromQL complex query → tenant_id injected on every selector** — `sum(rate(foo[5m])) / sum(rate(bar[5m]))` → both `foo` AND `bar` get `tenant_id` label.
4. **LogQL `{service="x"}` → injected** — adds `tenant_id="T"`; pipe stages preserved.
5. **TraceQL `{service.name="x"}` → injected** — adds `resource.tenant_id="T"` filter.
6. **Query with user-supplied tenant_id rejected** — `foo{tenant_id="other"}` → `400` with `user_supplied_tenant_id`; sev-1 audit emitted.
7. **Missing JWT → 401**.
8. **Invalid JWT signature → 401**.
9. **Expired JWT → 401**.
10. **Cross-tenant attempt logged** — `obs.cross_tenant_query_attempt` BRAIN row + sev-1 alarm.
11. **Audit row per query** — every successful proxy emits `obs.query_proxied` with SHA-256 of query.
12. **Root-admin unfiltered query allowed** — caller with tenant_id=nil + role root-admin gets query forwarded WITHOUT injection; `obs.query_proxied` row carries `outcome: root_admin_unfiltered`; sev-2 informational.
13. **Property test: 1000 random queries × 2 tenant pairs → zero cross-tenant data**.
14. **p95 injection overhead < 5ms** — measured via `obs_proxy_injection_latency_ms{backend}`.
15. **Malformed PromQL → 400 with parse error** (NOT 500).
16. **Backend unreachable → 503** with `backend_error`.
17. **Datasources point at proxy** — Grafana datasources URL = `http://obs-proxy:8088/...`.

---

## §5 — Verification

```rust
// services/obs-proxy/tests/inject_promql_test.rs
#[test]
fn injects_into_simple_selector() {
    let result = inject::promql::add_label("rate(foo[5m])", "tenant_id", "T").unwrap();
    assert!(result.contains("tenant_id=\"T\""));
    assert!(result.contains("foo"));
}

#[test]
fn injects_into_every_selector_in_complex_query() {
    let result = inject::promql::add_label(
        "sum(rate(foo[5m])) / sum(rate(bar[5m]))", "tenant_id", "T",
    ).unwrap();
    let count = result.matches("tenant_id=\"T\"").count();
    assert_eq!(count, 2);
}

#[test]
fn detects_user_supplied_tenant_id() {
    let result = inject::promql::has_label("foo{tenant_id=\"other\"}", "tenant_id").unwrap();
    assert!(result);
}

#[test]
fn malformed_promql_returns_parse_error() {
    let err = inject::promql::add_label("rate(foo[bad", "tenant_id", "T").expect_err("expected parse fail");
    assert!(matches!(err, ProxyError::ParseFailed { .. }));
}

// inject_logql_test.rs
#[test]
fn injects_logql_simple_selector() {
    let result = inject::logql::add_label("{service=\"x\"}", "tenant_id", "T").unwrap();
    assert_eq!(result, "{service=\"x\",tenant_id=\"T\"}");
}

#[test]
fn preserves_pipe_stages() {
    let result = inject::logql::add_label("{service=\"x\"} | json | line_format \"...\"", "tenant_id", "T").unwrap();
    assert!(result.starts_with("{service=\"x\",tenant_id=\"T\"}"));
    assert!(result.contains("| json"));
    assert!(result.contains("| line_format"));
}
```

```rust
// services/obs-proxy/tests/proxy_test.rs
#[tokio::test]
async fn missing_jwt_returns_401() {
    let state = test_state().await;
    let req = Request::builder().uri("/api/v1/query?query=rate(foo[5m])").body(Body::empty()).unwrap();
    let err = proxy(req, &state).await.expect_err("expected AuthFailed");
    assert!(matches!(err, ProxyError::AuthFailed(_)));
}

#[tokio::test]
async fn user_supplied_tenant_id_returns_400_and_audits() {
    let state = test_state().await;
    let req = test_request_with_jwt("/api/v1/query?query=foo{tenant_id=\"other\"}", &test_jwt("T")).await;
    let err = proxy(req, &state).await.expect_err("expected UserSuppliedTenantId");
    assert!(matches!(err, ProxyError::UserSuppliedTenantId));
    assert!(brain_test_helper::has_recent_row("obs.cross_tenant_query_attempt", "T"));
}

#[tokio::test]
async fn root_admin_query_unfiltered() {
    let state = test_state().await;
    let req = test_request_with_jwt("/api/v1/query?query=rate(foo[5m])", &root_admin_jwt()).await;
    let mock_backend = state.mock_backend();
    let _ = proxy(req, &state).await.unwrap();
    let last_query = mock_backend.last_query().await;
    assert!(!last_query.contains("tenant_id"));
    assert!(brain_test_helper::has_recent_row("obs.query_proxied", "root_admin_unfiltered"));
}
```

```rust
// services/obs-proxy/tests/cross_tenant_property_test.rs
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn no_cross_tenant_data_in_responses(
        (t_a, t_b) in any_tenant_pair(),
        query in any_promql_query(),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let state = test_state().await;
            test_helper::insert_metric_in_tenant(&t_a, "value_a").await;
            test_helper::insert_metric_in_tenant(&t_b, "value_b").await;

            let req = test_request_with_jwt(&format!("/api/v1/query?query={query}"), &test_jwt(&t_a)).await;
            let resp = proxy(req, &state).await.unwrap();
            let body = response_body(&resp).await;
            prop_assert!(!body.contains("value_b"), "tenant B's data leaked into tenant A's response");
        });
    }
}
```

```rust
// services/obs-proxy/tests/audit_log_test.rs
#[tokio::test]
async fn every_query_emits_audit_row() {
    let state = test_state().await;
    let _ = proxy(test_request_with_jwt("/api/v1/query?query=rate(foo[5m])", &test_jwt("T")).await, &state).await.unwrap();
    let row = brain_test_helper::find_latest("obs.query_proxied").unwrap();
    assert_eq!(row.payload["tenant_id"], "T");
    assert_eq!(row.payload["backend"], "prometheus");
    assert!(row.payload["query_sha256"].as_str().unwrap().len() == 64);
    assert!(row.payload["latency_ms"].as_f64().unwrap() < 5.0);
}
```

```bash
cd services/obs-proxy && cargo test
```

---

## §6 — Implementation skeleton

See §3.

```yaml
# deploy/obs/grafana/datasources.yaml (modified)
apiVersion: 1
datasources:
  - name: Loki
    type: loki
    url: http://obs-proxy:8088/loki/api          # was http://loki:3100
    httpHeaderName1: "Authorization"
    secureJsonData: { httpHeaderValue1: "Bearer ${GRAFANA_USER_JWT}" }
  - name: Prometheus
    type: prometheus
    url: http://obs-proxy:8088/api/v1
    httpHeaderName1: "Authorization"
    secureJsonData: { httpHeaderValue1: "Bearer ${GRAFANA_USER_JWT}" }
  - name: Tempo
    type: tempo
    url: http://obs-proxy:8088/tempo
    httpHeaderName1: "Authorization"
    secureJsonData: { httpHeaderValue1: "Bearer ${GRAFANA_USER_JWT}" }
```

---

## §7 — Dependencies

- **FR-OBS-001** — Backends running.
- **FR-AUTH-004** — JWT verification (JWKS).
- **FR-AUTH-108 (downstream)** — Lumi identity; current FR uses standard FR-AUTH-004 JWT.
- Crates: `axum`, `reqwest`, `promql-parser@0.4`, `jsonwebtoken@9`, `proptest@1`, `sha2`.

---

## §8 — Example payloads

### PromQL injection

```
input:  rate(ai_gateway_precheck_calls_total[5m])
output: rate(ai_gateway_precheck_calls_total{tenant_id="org:cyberskill"}[5m])
```

### LogQL injection with pipe stages

```
input:  {service="ai-gateway"} | json | line_format "{{.message}}"
output: {service="ai-gateway",tenant_id="org:cyberskill"} | json | line_format "{{.message}}"
```

### TraceQL injection

```
input:  { service.name = "ai-gateway" }
output: { service.name = "ai-gateway" && resource.tenant_id = "org:cyberskill" }
```

### Cross-tenant attempt response

```http
HTTP/1.1 400 Bad Request
{ "error": "user_supplied_tenant_id", "reason": "tenant_id label is reserved; do not include in query" }
```

### Audit row `obs.query_proxied`

```json
{
  "kind": "obs.query_proxied",
  "payload": {
    "tenant_id": "org:cyberskill",
    "caller_subject_id": "550e...",
    "backend": "prometheus",
    "query_sha256": "4b8c0d2f1a7e9c3b...",
    "outcome": "proxied",
    "latency_ms": 2.3,
    "request_id": "obs_..."
  }
}
```

### Audit row `obs.cross_tenant_query_attempt` (sev-1)

```json
{
  "kind": "obs.cross_tenant_query_attempt",
  "payload": {
    "caller_tenant_id": "org:cyberskill",
    "attempted_label_value": "org:other-tenant",
    "query_sha256": "...",
    "request_id": "obs_..."
  }
}
```

---

## §9 — Open questions

All resolved. Deferred:
- Per-tenant query rate limiting (slice 4+).
- Query-result caching (slice 5+).
- Streaming responses for long-running queries (slice 5+).

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Token missing | Auth check | 401 | Caller refreshes token |
| Token tenant_id missing | JWT parse | 401 (malformed_token) | Operator fixes JWT minter |
| JWT signature invalid | jsonwebtoken verify | 401 | Suspect attack; investigate |
| JWT expired | exp check | 401 | Caller re-authenticates |
| Query parse fails | promql-parser/logql/traceql error | 400 with parse error | Caller fixes query |
| Backend unreachable | reqwest connect error | 503 | Operator investigates backend |
| User-supplied tenant_id label | has_label check | 400 + sev-1 audit | Investigate caller (likely malicious or buggy client) |
| Cross-tenant data leak (regression) | property test fails in CI | PR blocked | Fix injection logic |
| Slow injection (> 5ms p95) | OTel histogram | sev-3 alarm | Investigate query complexity OR parser |
| Audit-row emit fails (BRAIN down) | brain_writer error | Query still proxied; sev-2 alarm | Operator investigates BRAIN |
| Root-admin query without expected role claim | claims check | Treated as regular tenant; injection applies | Operator updates JWT minter |
| LogQL parser doesn't handle new syntax | parse error | 400 | Update logql parser |
| TraceQL parser doesn't handle new syntax | parse error | 400 | Update traceql parser |
| Datasources misconfigured (point at backend not proxy) | Operator sees unfiltered data | Sev-1 (security) | Fix datasources.yaml; redeploy |
| Grafana sends multiple queries per request (batched) | proxy handles each independently | Each gets injected | By design |
| Proxy itself becomes unauthenticated path (network bypass) | network policy enforcement | Should be impossible by topology | Network ACLs (deploy-time) |
| Promql-parser crate has a bug | parse fails or wrong AST | Investigate; possibly patch upstream | Upstream issue |
| Memory exhaustion on huge query | parser allocates | Process restart | Set per-process memory limit |
| Concurrent query bypass (race) | not possible — each request independent | N/A | By design |
| Audit log too large (every query) | BRAIN storage growth | Retention policy on `obs.*` rows | 30-day retention |

---

## §11 — Notes

- `promql-parser` crate is the load-bearing dep. If it lacks a feature (e.g., specific PromQL functions), patch upstream OR use the official Prometheus Go parser via FFI as fallback.
- LogQL + TraceQL parsers are hand-rolled because no mature Rust crates exist. The hand-rolled parsers cover only the subset Grafana actually emits — full grammar coverage would be over-engineering. Coverage tests in §5 ensure the subset works.
- The proxy IS the security boundary. Grafana itself runs as a non-tenant-aware app; the proxy makes it multi-tenant. Bypassing the proxy (e.g., querying Loki directly via `localhost:3100`) is prevented by network policy — Loki/Prometheus/Tempo only accept connections from the proxy.
- AST-based injection prevents the bypass class of "user crafts a query that escapes the wrapper." String concat would be susceptible; AST is robust.
- The cross-tenant property test (§5) mirrors FR-AI-018's pattern. 1000 random queries × tenant pairs ensures the AST injection works across the full PromQL grammar.
- Root-admin gets unfiltered access (§1 #11) but every such query is audited — compliance reviews can ask "who did cross-tenant queries last week?" The sev-2 informational alarm makes the activity visible without being a false-positive incident.
- Audit-row payload uses query_sha256 (not raw query) because queries can contain tenant-business semantics. The hash preserves uniqueness for forensic correlation; ops can re-run the query to inspect the actual content if needed.
- The 5ms overhead budget is achievable because parsing is pure-CPU (no I/O); promql-parser is fast (~10µs per typical query).
- Query-result caching (deferred to slice 5+) would reduce backend load further; for now the proxy is stateless.

---

*End of FR-OBS-002. Status: draft (10/10 target).*
