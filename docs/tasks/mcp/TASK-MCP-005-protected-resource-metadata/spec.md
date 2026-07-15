---
id: TASK-MCP-005
title: "MCP Protected Resource Metadata (RFC 9728) at `/.well-known/oauth-protected-resource` — closed audience-binding advertisement for federated MCP clients"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-05-17T00:00:00+07:00
department: engineering
author: @stephencheng
template: task@1
module: MCP
priority: p0
status: implementing
verify: T
phase: P0
milestone: P0 · slice 2
slice: 2
owner: Stephen Cheng (CTO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-MCP-001, TASK-MCP-004, TASK-MCP-006, TASK-MCP-007, TASK-AUTH-004, TASK-AUTH-104, TASK-AI-003, TASK-MEMORY-101, TASK-OBS-005]
depends_on: [TASK-MCP-004]
blocks: []

source_pages:
  - website/docs/modules/mcp.html#oauth
  - https://datatracker.ietf.org/doc/html/rfc9728
  - https://modelcontextprotocol.io/specification/2025-11-25/server/authorization
  - https://datatracker.ietf.org/doc/html/rfc8414

source_decisions:
  - DEC-895 2026-05-17 — RFC 9728 Protected Resource Metadata is the canonical mechanism for an MCP server to advertise which authorization servers issue acceptable tokens; TASK-MCP-004 audience-bound tokens require this discovery
  - DEC-896 2026-05-17 — PRM document is served unauthenticated at `/.well-known/oauth-protected-resource` per RFC 9728 §3.1; metadata is public by spec
  - DEC-897 2026-05-17 — Per-MCP-server-namespace PRM at `/.well-known/oauth-protected-resource/cyberos.{module}` for the module-namespaced MCP servers (TASK-MCP-002 + TASK-MCP-003); top-level `/.well-known/oauth-protected-resource` returns the gateway-aggregate PRM
  - DEC-898 2026-05-17 — `resource` field of PRM = the canonical MCP gateway resource URI (`https://api.cyberos.world/mcp/v1`); per-module PRMs use `https://api.cyberos.world/mcp/v1/{module}`
  - "DEC-899 2026-05-17 — `authorization_servers` array lists the per-residency AUTH-server issuers (residency-derived per TASK-AUTH-004): `[https://auth.us-1.cyberos.world, https://auth.eu-1.cyberos.world, https://auth.sg-1.cyberos.world, https://auth.vn-1.cyberos.world]`; client picks the issuer that matches their tenant's residency"
  - DEC-900 2026-05-17 — `bearer_methods_supported` = `["header"]` only; `body` and `query` methods are FORBIDDEN per OAuth 2.1 + MCP-spec guidance
  - DEC-901 2026-05-17 — `resource_signing_alg_values_supported` = `["RS256", "EdDSA"]` matching TASK-AUTH-004's JWT signing algorithms; never include HS256 (symmetric — wrong for federated)
  - DEC-902 2026-05-17 — `resource_documentation` field points to `https://api.cyberos.world/mcp/v1/docs` for client developers
  - DEC-903 2026-05-17 — PRM responses cached client-side per RFC 9728 §3.4 with `Cache-Control: max-age=3600`; server-side hot-reload of upstream config invalidates the cached PRM via `ETag`
  - DEC-904 2026-05-17 — Content-Type MUST be `application/json` per RFC 9728 §3.1; charset UTF-8
  - DEC-905 2026-05-17 — `scopes_supported` is OMITTED at slice 2 (gateway-wide); per-module PRMs include `scopes_supported` enumerating the module's tool-level scopes per TASK-MCP-006
  - DEC-906 2026-05-17 — memory audit kinds: mcp.prm_served, mcp.prm_unknown_module_requested (sev-3 informational); mcp.prm_aggregate_drift (sev-2 — detected when per-module PRMs disagree with gateway-aggregate)
  - DEC-907 2026-05-17 — Per-module PRMs are GENERATED from TASK-MCP-002 module-registration metadata; no per-module hand-written PRM allowed (single source of truth = registration record)
  - DEC-908 2026-05-17 — Rate limit: 600 req/min/IP on PRM endpoints (clients should cache; high-rate signals misbehaving client)
  - DEC-909 2026-05-17 — PRM endpoint MUST respond < 50 ms p95 (static content + cache); alarm sev-3 if p95 > 200 ms sustained 5 min

build_envelope:
  language: rust 1.81
  service: cyberos/services/mcp/
  new_files:
    - services/mcp/migrations/0005_prm_drift_log.sql                  # aggregate-vs-module drift audit
    - services/mcp/src/prm/mod.rs                                     # PRM orchestrator
    - services/mcp/src/prm/gateway.rs                                 # gateway-aggregate PRM builder
    - services/mcp/src/prm/per_module.rs                              # per-module PRM builder
    - services/mcp/src/prm/cache.rs                                   # ETag + Cache-Control
    - services/mcp/src/handlers/prm_routes.rs                         # public route registration
    - services/mcp/src/audit/prm_events.rs                            # 3 memory row builders
    - services/mcp/tests/prm_gateway_test.rs
    - services/mcp/tests/prm_per_module_test.rs
    - services/mcp/tests/prm_etag_cache_test.rs
    - services/mcp/tests/prm_unknown_module_test.rs
    - services/mcp/tests/prm_unauth_public_test.rs
    - services/mcp/tests/prm_rate_limit_test.rs
    - services/mcp/tests/prm_drift_detection_test.rs
    - services/mcp/tests/prm_audit_emission_test.rs

  modified_files:
    - services/mcp/src/lib.rs                                          # mount prm_routes
    - services/mcp/src/server_registry.rs                              # add PRM-fields to ModuleRegistration
    - services/mcp/Cargo.toml                                          # +etag

  allowed_tools:
    - file_read: services/mcp/**
    - file_read: services/auth/src/**                                  # to read JWT issuer / signing-alg metadata
    - file_write: services/mcp/{src,tests,migrations}/**
    - bash: cd services/mcp && cargo test prm

  disallowed_tools:
    - require auth on PRM endpoint (per RFC 9728 + DEC-896)
    - include HS256 in resource_signing_alg_values_supported (per DEC-901)
    - hand-write per-module PRM JSON (must derive from registration per DEC-907)
    - allow `body` or `query` bearer method (per DEC-900)

effort_hours: 3
subtasks:
  - "0.3h: 0005_prm_drift_log.sql migration"
  - "0.4h: prm/gateway.rs — aggregate PRM builder from per-module registrations"
  - "0.4h: prm/per_module.rs — single-module PRM builder"
  - "0.3h: prm/cache.rs — ETag + Cache-Control: max-age=3600"
  - "0.3h: handlers/prm_routes.rs — `/.well-known/oauth-protected-resource[/cyberos.<module>]`"
  - "0.3h: audit/prm_events.rs — 3 builders"
  - "0.7h: tests — 8 test files"
  - "0.3h: wire-up — lib.rs + server_registry.rs extension"

risk_if_skipped: "Without PRM, federated MCP clients (Claude Desktop, ChatGPT, third-party agents) cannot discover which authorization server issues acceptable tokens for our MCP gateway. The OAuth 2.1 PKCE flow from TASK-MCP-004 is incomplete without RFC 9728 metadata: clients receive 401 + `WWW-Authenticate: Bearer resource_metadata=...` per MCP spec §2.1, follow the URL, get the PRM, then know which issuer to talk to. Without PRM, every client requires hand-config of issuer URL per residency — non-scalable + footgun (wrong-issuer tokens get 401, no actionable error). Without DEC-907's generated-from-registration rule, per-module PRMs drift from per-module reality (registered scopes ≠ advertised scopes = silent breakage for clients). The 3h effort lands the discovery primitive that completes the OAuth flow."
---

## §1 — Description (BCP-14 normative)

The MCP service **MUST** expose RFC 9728 Protected Resource Metadata at `/.well-known/oauth-protected-resource` (gateway-aggregate) and `/.well-known/oauth-protected-resource/cyberos.{module}` (per-module), generated from the TASK-MCP-002 registration record, with ETag-based caching, rate limiting, and 3 memory audit kinds.

1. **MUST** expose `GET /.well-known/oauth-protected-resource` returning the gateway-aggregate PRM as `application/json; charset=utf-8` per RFC 9728 §3.1. Unauthenticated per DEC-896.

2. **MUST** include in the gateway-aggregate PRM the following fields per RFC 9728 §2:
   - `resource`: `https://api.cyberos.world/mcp/v1` (canonical gateway URI per DEC-898).
   - `authorization_servers`: array of per-residency AUTH issuer URLs per DEC-899 (4 entries: us-1, eu-1, sg-1, vn-1).
   - `bearer_methods_supported`: `["header"]` per DEC-900 (header-only; body/query forbidden).
   - `resource_signing_alg_values_supported`: `["RS256", "EdDSA"]` per DEC-901.
   - `resource_documentation`: `https://api.cyberos.world/mcp/v1/docs` per DEC-902.

3. **MUST** expose per-module PRM at `GET /.well-known/oauth-protected-resource/cyberos.{module}` for each module registered via TASK-MCP-002 per DEC-897. The per-module PRM differs from the aggregate by:
   - `resource`: `https://api.cyberos.world/mcp/v1/{module}` per DEC-898.
   - `scopes_supported`: array of tool-level scopes the module exposes (derived from TASK-MCP-002 registration record per DEC-907 + DEC-905).
   - All other fields identical to aggregate (same auth servers, same bearer methods, same signing algs).

4. **MUST** GENERATE per-module PRMs from the TASK-MCP-002 registration record per DEC-907. The `services/mcp/src/server_registry.rs` ModuleRegistration struct is extended with PRM-relevant fields (`exposed_scopes: Vec<String>`) populated at registration. The per-module PRM handler reads from this registry — never from a hand-written JSON file. If the registry has no entry for the requested `{module}`, return 404 + emit `mcp.prm_unknown_module_requested` informational row.

5. **MUST NOT** require authentication on PRM endpoints per RFC 9728 §3.1 + DEC-896 — metadata is public by spec. Other gateway protection layers (rate limit + Turnstile for non-discovery endpoints) DO NOT apply.

6. **MUST** include `ETag` header on PRM responses per DEC-903 + RFC 7232 §2.3. ETag is the SHA-256-truncated hex of the canonical-JSON body (first 16 hex chars). Client requests with `If-None-Match: <etag>` matching the current ETag receive `304 Not Modified` with no body.

7. **MUST** include `Cache-Control: max-age=3600` per DEC-903 (RFC 9728 §3.4 recommends caching but does not pin a value; 1h matches our hot-reload SLA for upstream config changes).

8. **MUST** detect drift between gateway-aggregate and per-module PRMs per DEC-906. The drift detector runs on every PRM serve: if the aggregate's `authorization_servers` or `resource_signing_alg_values_supported` differs from any per-module's (which should never happen — derived from same registry), emit `mcp.prm_aggregate_drift` sev-2 memory row + log to `prm_drift_log` table.

9. **MUST** define `prm_drift_log` table at migration `0005`: `(id BIGSERIAL PRIMARY KEY, detected_at TIMESTAMPTZ NOT NULL DEFAULT now(), aggregate_sha256 CHAR(64) NOT NULL, module_name TEXT NOT NULL, module_sha256 CHAR(64) NOT NULL, drift_fields TEXT[] NOT NULL)`. Append-only via REVOKE per task-audit skill rule 12. RLS not required (system-tenant scope only — drift is global, not per-tenant).

10. **MUST** rate-limit PRM endpoints at 600 req/min/IP per DEC-908 (clients should cache; high rate signals misbehaving client). Excess returns `429` + `Retry-After`.

11. **MUST** complete PRM response in < 50 ms p95 per DEC-909. The PRM bodies are small (<2 KB) and built from in-memory registry data; sub-50ms is straightforward. OTel histogram `mcp_prm_serve_duration_seconds`; alarm sev-3 if p95 > 200 ms sustained 5 min.

12. **MUST** emit 3 memory audit row kinds (DEC-906 + task-audit skill rule 6 namespace pattern):
    - `mcp.prm_served` (sev-3 — high-volume; sampled at 1% per task-audit skill tail-sampling pattern via TASK-OBS-006)
    - `mcp.prm_unknown_module_requested` (sev-3 — informational; could indicate client misconfig or scanner)
    - `mcp.prm_aggregate_drift` (sev-2 — should never happen; indicates registry corruption)

13. **MUST** be Content-Type `application/json` with `charset=utf-8` per DEC-904 + RFC 9728 §3.1.

14. **MUST** include the `X-Robots-Tag: noindex` header on PRM responses to prevent search-engine indexing (defense-in-depth — PRM is metadata for OAuth clients, not for general web crawling).

15. **MUST** support CORS for cross-origin client fetches: `Access-Control-Allow-Origin: *` (per RFC 9728 — metadata is public + same content for every requester), `Access-Control-Allow-Methods: GET, HEAD`, `Access-Control-Max-Age: 3600`.

16. **MUST** respond to HEAD requests with the same headers as GET (status, ETag, Cache-Control, Content-Length) but empty body. Standard HTTP behaviour required by CORS preflight + some HTTP clients.

17. **MUST** invalidate the cached PRM body + ETag when the TASK-MCP-002 registry changes (module register / heartbeat / unregister). Implementation: registry change publishes to NATS subject `cyberos.mcp.registry.changed`; PRM cache subscriber rebuilds + bumps ETag.

18. **MUST** route per-module PRM 404 (unknown module) via the same RLS/rate-limit middleware as the gateway-aggregate path — do not branch the middleware. Single ingress.

19. **MUST** emit the `WWW-Authenticate` resource-metadata header on 401 responses from TASK-MCP-001 endpoints per MCP spec §2.1, pointing at this task's PRM URL. Implementation note for TASK-MCP-001 integration (this task provides the endpoint; TASK-MCP-001 references it).

20. **SHOULD** include `client_uri` field in the PRM pointing at `https://cyberos.world/mcp/clients` (a public landing page documenting registered MCP clients per DEC-902 derivative). OPTIONAL; included for client discoverability.

---

## §2 — Why this design (rationale for humans)

**Why PRM at all (§1 #1)?** RFC 9728 is the OAuth 2.0 protected-resource discovery standard. MCP spec 2025-11-25 §2.1 mandates it: when an MCP client receives a 401, the response includes `WWW-Authenticate: Bearer resource_metadata=<url>`, the client fetches that URL, gets the PRM, and learns (a) which authorization server to talk to, (b) which signing algorithms are accepted, (c) which scopes exist. Without PRM, every MCP client requires hand-configuration per CyberOS deployment — non-scalable.

**Why per-residency authorization_servers list (§1 #2, DEC-899)?** TASK-AUTH-004 mints residency-pinned JWTs (eu-1 tenants get tokens from auth.eu-1; us-1 from auth.us-1). PRM advertises all 4 issuers; the client picks the one matching their tenant's residency. Wrong-issuer token = 401 at the gateway with a clearer error message.

**Why generate per-module PRMs from registry (§1 #4, DEC-907)?** Single source of truth. Hand-writing per-module PRM JSON means drift — a module ships a new tool with a new scope, the PRM doesn't update, clients can't discover the scope, the tool is silently un-callable. Generating from TASK-MCP-002 registration ensures advertised scopes always match registered tools.

**Why ETag + 1h cache (§1 #6, §1 #7, DEC-903)?** PRM is high-traffic (every MCP client fetches at startup + periodically refreshes). ETag enables `If-None-Match` 304 responses — 100 bytes of header instead of 2 KB of body. 1h cache reduces server load while matching our hot-reload SLA (registry changes propagate within 1h worst-case; faster via NATS invalidation per §1 #17).

**Why bearer method header-only (§1 #2, DEC-900)?** OAuth 2.1 + MCP spec both deprecate `body` and `query` bearer methods (token in URL = log leakage; token in body = breaks idempotent GETs). Header-only is the modern standard. Advertising other methods would invite client implementations that we'd then need to reject.

**Why `noindex` (§1 #14)?** PRM is metadata for OAuth clients, not for search engines. While the content isn't sensitive, indexing it adds noise to web search results pointing at API endpoints (confusing for non-developers who Google "cyberos api"). Defense-in-depth on metadata exposure.

**Why drift detection (§1 #8, DEC-906)?** The aggregate-PRM and per-module-PRMs SHOULD always agree on shared fields (auth servers, signing algs). If they don't, the registry is corrupted or a code change introduced a divergence path. Emitting sev-2 alerts forces operator review before clients see inconsistent metadata.

**Why CORS `*` (§1 #15)?** PRM is public + identical for every requester (no per-client variance). Restricting CORS gains no security and breaks cross-origin client implementations (browser-based MCP clients) which are common.

**Why rate-limit at 600/min (§1 #10, DEC-908)?** Generous for legitimate clients (cache hit ratio ~99%; misses on 1h boundaries means ~1 req/h/client). 600/min = 1 req per 100ms which only a misbehaving client (no cache, retry loop) would hit. Block sooner = false positives on legitimate clients during configuration changes.

---

## §3 — API contract

### 3.1 Postgres schema

```sql
-- 0005_prm_drift_log.sql
CREATE TABLE prm_drift_log (
  id BIGSERIAL PRIMARY KEY,
  detected_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  aggregate_sha256 CHAR(64) NOT NULL,
  module_name TEXT NOT NULL,
  module_sha256 CHAR(64) NOT NULL,
  drift_fields TEXT[] NOT NULL,
  trace_id CHAR(32)
);
CREATE INDEX idx_prm_drift_detected_at ON prm_drift_log(detected_at);
REVOKE UPDATE, DELETE ON prm_drift_log FROM cyberos_app;
-- RLS not needed: system-tenant scope only (drift is a global system event).
```

### 3.2 Rust types

```rust
// services/mcp/src/prm/mod.rs
#[derive(serde::Serialize, Debug)]
pub struct PrmDocument {
    pub resource: String,
    pub authorization_servers: Vec<String>,
    pub bearer_methods_supported: Vec<String>,
    pub resource_signing_alg_values_supported: Vec<String>,
    pub resource_documentation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes_supported: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_uri: Option<String>,
}

impl PrmDocument {
    pub fn canonical_etag(&self) -> String {
        let body = serde_json::to_vec(self).unwrap();  // serde sorts keys
        let digest = sha2::Sha256::digest(&body);
        hex::encode(&digest[..8])  // 16-char hex
    }
}
```

### 3.3 REST endpoints

```text
GET    /.well-known/oauth-protected-resource                    (public, gateway-aggregate)
GET    /.well-known/oauth-protected-resource/cyberos.{module}   (public, per-module)
HEAD   /.well-known/oauth-protected-resource                    (same as GET, empty body)
HEAD   /.well-known/oauth-protected-resource/cyberos.{module}
OPTIONS /.well-known/oauth-protected-resource(/cyberos.{module})  (CORS preflight)
```

---

## §4 — Acceptance criteria

1. **Gateway PRM unauthenticated** — GET without Authorization header returns 200 + valid PRM JSON.
2. **PRM JSON conforms to RFC 9728** — body matches RFC 9728 §2 schema; fields `resource`, `authorization_servers`, `bearer_methods_supported`, `resource_signing_alg_values_supported` present.
3. **Per-module PRM derived from registry** — module registered with `exposed_scopes=["projects.read","projects.write"]` returns PRM with that exact `scopes_supported` array.
4. **Unknown module 404** — GET `/.well-known/oauth-protected-resource/cyberos.bogus` returns 404 + emits `mcp.prm_unknown_module_requested`.
5. **ETag returned + 304 on match** — first GET returns 200 + ETag; second GET with `If-None-Match: <etag>` returns 304 + no body.
6. **Cache-Control: max-age=3600** — header present on every 200 response.
7. **Bearer methods header-only** — PRM `bearer_methods_supported` = `["header"]` exactly.
8. **Signing algs RS256+EdDSA** — PRM `resource_signing_alg_values_supported` = `["RS256","EdDSA"]` exactly.
9. **CORS headers present** — `Access-Control-Allow-Origin: *` + `Methods: GET, HEAD` + `Max-Age: 3600`.
10. **HEAD returns headers + empty body** — HEAD response Content-Length matches GET body length but body is empty.
11. **noindex header** — `X-Robots-Tag: noindex` on every PRM response.
12. **Drift detected when registry divergence injected** — manual test injects divergent module PRM → `mcp.prm_aggregate_drift` sev-2 emitted + `prm_drift_log` row written.
13. **Rate limit at 600/min/IP** — 601st request in 60s returns 429.
14. **P95 latency < 50ms** — synthetic load test with 1000 requests; p95 < 50ms.
15. **PRM cache invalidated on registry change** — module register/unregister event triggers ETag bump within 1 s.
16. **`mcp.prm_served` row emitted at 1% sample** — 1000 sampled requests produces ~10 audit rows (TASK-OBS-006 tail-sampling).
17. **OAuth issuer URL list matches TASK-AUTH-004 residency map** — assertion against AUTH-004 `RESIDENCY_ISSUERS` const.
18. **`resource` URI matches TASK-MCP-004 audience-binding** — assertion against TASK-MCP-004's audience-bound JWT `aud` claim.
19. **Content-Type application/json; charset=utf-8** — header present + correct.
20. **Registry-write fails if registration lacks `exposed_scopes`** — TASK-MCP-002 ModuleRegistration validation rejects missing field.

---

## §5 — Verification

### 5.1 `prm_gateway_test.rs`

```rust
#[tokio::test]
async fn gateway_prm_unauth_and_well_formed() {
    let ctx = TestContext::new().await;
    let r = ctx.get("/.well-known/oauth-protected-resource").send().await.unwrap();
    assert_eq!(r.status(), 200);
    assert_eq!(r.headers()["content-type"], "application/json; charset=utf-8");
    assert!(r.headers().contains_key("etag"));
    assert_eq!(r.headers()["cache-control"], "max-age=3600");
    assert_eq!(r.headers()["x-robots-tag"], "noindex");
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["resource"], "https://api.cyberos.world/mcp/v1");
    assert_eq!(body["bearer_methods_supported"], serde_json::json!(["header"]));
    assert_eq!(body["resource_signing_alg_values_supported"], serde_json::json!(["RS256","EdDSA"]));
    let issuers = body["authorization_servers"].as_array().unwrap();
    assert_eq!(issuers.len(), 4);
}
```

### 5.2 `prm_per_module_test.rs`

```rust
#[tokio::test]
async fn per_module_prm_derived_from_registry() {
    let ctx = TestContext::new().await;
    ctx.register_module("projects", vec!["projects.read","projects.write","projects.delete"]).await;
    let r = ctx.get("/.well-known/oauth-protected-resource/cyberos.projects").send().await.unwrap();
    assert_eq!(r.status(), 200);
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["resource"], "https://api.cyberos.world/mcp/v1/projects");
    assert_eq!(body["scopes_supported"], serde_json::json!(["projects.read","projects.write","projects.delete"]));
}
```

### 5.3 `prm_etag_cache_test.rs`

```rust
#[tokio::test]
async fn etag_returns_304_on_match() {
    let ctx = TestContext::new().await;
    let r1 = ctx.get("/.well-known/oauth-protected-resource").send().await.unwrap();
    let etag = r1.headers()["etag"].to_str().unwrap().to_owned();
    let r2 = ctx.get("/.well-known/oauth-protected-resource")
        .header("if-none-match", &etag).send().await.unwrap();
    assert_eq!(r2.status(), 304);
    assert_eq!(r2.bytes().await.unwrap().len(), 0);
}
```

### 5.4 `prm_unknown_module_test.rs`

```rust
#[tokio::test]
async fn unknown_module_404_audited() {
    let ctx = TestContext::new().await;
    let r = ctx.get("/.well-known/oauth-protected-resource/cyberos.bogus").send().await.unwrap();
    assert_eq!(r.status(), 404);
    let audit = ctx.memory_rows().await;
    assert!(audit.iter().any(|r| r.kind == "mcp.prm_unknown_module_requested"));
}
```

### 5.5 `prm_drift_detection_test.rs`

```rust
#[tokio::test]
async fn drift_emits_sev2_alert() {
    let ctx = TestContext::new().await;
    ctx.register_module("projects", vec!["projects.read"]).await;
    ctx.inject_per_module_signing_alg_drift("projects", vec!["HS256"]).await;  // bad: HS256
    let _ = ctx.get("/.well-known/oauth-protected-resource/cyberos.projects").send().await.unwrap();
    let audit = ctx.memory_rows().await;
    let drift = audit.iter().find(|r| r.kind == "mcp.prm_aggregate_drift").unwrap();
    assert_eq!(drift.severity, 2);
    assert!(drift.payload["drift_fields"].as_array().unwrap()
        .iter().any(|f| f == "resource_signing_alg_values_supported"));

    let drift_count: i64 = sqlx::query_scalar("SELECT count(*) FROM prm_drift_log").fetch_one(&ctx.pool).await.unwrap();
    assert_eq!(drift_count, 1);
}
```

### 5.6 `prm_rate_limit_test.rs`

```rust
#[tokio::test]
async fn rate_limit_at_600_per_min() {
    let ctx = TestContext::new().await;
    for _ in 0..600 {
        let r = ctx.get("/.well-known/oauth-protected-resource").send().await.unwrap();
        assert_eq!(r.status(), 200);
    }
    let r = ctx.get("/.well-known/oauth-protected-resource").send().await.unwrap();
    assert_eq!(r.status(), 429);
}
```

### 5.7 `prm_unauth_public_test.rs`

```rust
#[tokio::test]
async fn prm_responds_without_authentication() {
    let ctx = TestContext::new().await;
    // No Authorization header
    let r = ctx.get("/.well-known/oauth-protected-resource").send().await.unwrap();
    assert_eq!(r.status(), 200);
    // CORS preflight
    let opt = ctx.request(reqwest::Method::OPTIONS, "/.well-known/oauth-protected-resource")
        .header("origin", "https://example.com").send().await.unwrap();
    assert_eq!(opt.headers()["access-control-allow-origin"], "*");
}
```

### 5.8 `prm_audit_emission_test.rs`

```rust
#[tokio::test]
async fn prm_served_sampled_at_1_percent() {
    let ctx = TestContext::with_tail_sampling(1.0).await;  // 100% in test for determinism
    for _ in 0..10 {
        ctx.get("/.well-known/oauth-protected-resource").send().await.unwrap();
    }
    let audit = ctx.memory_rows().await;
    assert_eq!(audit.iter().filter(|r| r.kind == "mcp.prm_served").count(), 10);
}
```

---

## §6 — Implementation skeleton

(API contract in §3 is the skeleton. The handler is trivially short.)

```rust
// services/mcp/src/prm/gateway.rs
pub async fn build_gateway_prm(ctx: &AppCtx) -> PrmDocument {
    PrmDocument {
        resource: "https://api.cyberos.world/mcp/v1".to_string(),
        authorization_servers: ctx.auth.residency_issuers(),  // 4 URLs from TASK-AUTH-004
        bearer_methods_supported: vec!["header".into()],
        resource_signing_alg_values_supported: vec!["RS256".into(), "EdDSA".into()],
        resource_documentation: "https://api.cyberos.world/mcp/v1/docs".into(),
        scopes_supported: None,
        client_uri: Some("https://cyberos.world/mcp/clients".into()),
    }
}

// services/mcp/src/prm/per_module.rs
pub async fn build_per_module_prm(ctx: &AppCtx, module: &str) -> Option<PrmDocument> {
    let reg = ctx.registry.get_module(module).await?;
    Some(PrmDocument {
        resource: format!("https://api.cyberos.world/mcp/v1/{module}"),
        authorization_servers: ctx.auth.residency_issuers(),
        bearer_methods_supported: vec!["header".into()],
        resource_signing_alg_values_supported: vec!["RS256".into(), "EdDSA".into()],
        resource_documentation: "https://api.cyberos.world/mcp/v1/docs".into(),
        scopes_supported: Some(reg.exposed_scopes),
        client_uri: Some("https://cyberos.world/mcp/clients".into()),
    })
}
```

---

## §7 — Dependencies

**Upstream (depends_on):**
- **TASK-MCP-004** OAuth 2.1 PKCE — audience-bound tokens require this discovery endpoint to be useful.

**Cross-module (related_tasks):**
- **TASK-MCP-001** spec compliance — `WWW-Authenticate: Bearer resource_metadata=<this-task's-URL>` referenced from 401 responses.
- **TASK-MCP-002** per-module registration — registry feeds per-module PRM `scopes_supported`.
- **TASK-MCP-006** Tool-annotation gating — scopes advertised here reflect the tool-annotation set.
- **TASK-MCP-007** Tasks primitive — tasks tools' scopes appear in per-module PRM.
- **TASK-AUTH-004** JWT mint — `authorization_servers` list matches TASK-AUTH-004 residency issuers.
- **TASK-AUTH-104** OIDC SSO — IdP discovery follows analogous well-known pattern.
- **TASK-AI-003** memory audit-row bridge — 3 new kinds register.
- **TASK-OBS-005** Trace correlation — `mcp.prm_served` sampled via OBS tail-sampling.

**Downstream (blocks):** None at this slice. Future per-tool authz hooks (task-MCP-2xx) may consume PRM.

---

## §8 — Example payloads

### 8.1 Gateway PRM response

```http
HTTP/1.1 200 OK
Content-Type: application/json; charset=utf-8
ETag: "f8a1b2c3d4e5f607"
Cache-Control: max-age=3600
X-Robots-Tag: noindex
Access-Control-Allow-Origin: *
Access-Control-Allow-Methods: GET, HEAD
Access-Control-Max-Age: 3600

{
  "resource": "https://api.cyberos.world/mcp/v1",
  "authorization_servers": [
    "https://auth.us-1.cyberos.world",
    "https://auth.eu-1.cyberos.world",
    "https://auth.sg-1.cyberos.world",
    "https://auth.vn-1.cyberos.world"
  ],
  "bearer_methods_supported": ["header"],
  "resource_signing_alg_values_supported": ["RS256", "EdDSA"],
  "resource_documentation": "https://api.cyberos.world/mcp/v1/docs",
  "client_uri": "https://cyberos.world/mcp/clients"
}
```

### 8.2 Per-module PRM response

```http
HTTP/1.1 200 OK
Content-Type: application/json; charset=utf-8
ETag: "9c4e7a8b6d2f1e3a"
...

{
  "resource": "https://api.cyberos.world/mcp/v1/projects",
  "authorization_servers": ["https://auth.us-1.cyberos.world", ...],
  "bearer_methods_supported": ["header"],
  "resource_signing_alg_values_supported": ["RS256", "EdDSA"],
  "resource_documentation": "https://api.cyberos.world/mcp/v1/docs",
  "scopes_supported": ["projects.read", "projects.write", "projects.delete"],
  "client_uri": "https://cyberos.world/mcp/clients"
}
```

### 8.3 `mcp.prm_aggregate_drift` memory row

```json
{
  "kind": "mcp.prm_aggregate_drift",
  "severity": 2,
  "tenant_id": "00000000-0000-0000-0000-000000000001",
  "actor_id": "system.mcp.prm",
  "trace_id": "0af7651916cd43dd8448eb211c80319c",
  "occurred_at": "2026-05-17T09:14:32.847Z",
  "payload": {
    "module_name": "projects",
    "aggregate_sha256": "ab12...",
    "module_sha256": "cd34...",
    "drift_fields": ["resource_signing_alg_values_supported"]
  }
}
```

### 8.4 401 from TASK-MCP-001 endpoint pointing at PRM

```http
HTTP/1.1 401 Unauthorized
WWW-Authenticate: Bearer realm="cyberos-mcp", resource_metadata="https://api.cyberos.world/.well-known/oauth-protected-resource"
Content-Type: application/json

{ "error": "invalid_token", "error_description": "Token expired or wrong audience" }
```

---

## §9 — Open questions

All resolved. Deferred:

- **Deferred:** Per-tenant PRM override (vanity URLs at `<slug>.cyberos.world/.well-known/...`) — slice 3, task-MCP-2xx.
- **Deferred:** Multi-language `resource_documentation` URLs — slice 3.
- **Deferred:** Localised PRM scope descriptions (`scopes_supported_descriptions`) — slice 3.
- **Deferred:** PRM signing per RFC 9728 §3.5 (signed metadata) — slice 3; current trust model is HTTPS-only.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Unknown module requested | registry lookup miss | 404 + `mcp.prm_unknown_module_requested` | Client refreshes their module list via gateway-aggregate PRM (which lists modules) |
| Drift between aggregate + per-module | drift detector cmp at serve | sev-2 `mcp.prm_aggregate_drift` + `prm_drift_log` row | Operator investigates registry corruption; manual config rebuild |
| PRM cache stale after registry change | NATS invalidation event missed | Up to 1h staleness (Cache-Control) | NATS-side replay; manual cache flush via `cyberos-mcp prm-cache-flush` CLI |
| Rate limit hit | counter ≥ 600/min | 429 + `Retry-After: 60` | Client backs off; investigate cache misconfig |
| Invalid `If-None-Match` header (malformed) | parse fails | Treated as no-If-None-Match; serve 200 + body | Inherent — robust to malformed clients |
| Registry returns empty `authorization_servers` | empty list | 503 + sev-1 alert `mcp.prm_no_auth_servers` | Operator verifies TASK-AUTH-004 residency config |
| Disk I/O failure when writing `prm_drift_log` | Postgres write error | sev-2 alert; PRM still served (don't fail request on audit failure) | Postgres recovers; manual drift re-emit via CLI |
| OPTIONS request with disallowed Method header | CORS handler check | 200 + allowed-Methods header (no error; explicit reject not needed) | Inherent CORS conformance |
| ETag collision (same SHA-256 across different bodies — astronomically unlikely) | client receives 304 with wrong body | Client sees stale PRM until cache expiry | Inherent — 64-bit truncation enough for 1h cache scope |
| Module registers with malformed scope name (invalid chars) | TASK-MCP-002 validation | Registration rejected upstream; PRM never sees bad data | TASK-MCP-002 owns validation |
| HEAD request fails to set Content-Length | handler bug | Test asserts Content-Length matches GET body | CI catches; fix |
| `X-Robots-Tag` header missing | regression | Test asserts presence | CI catches |
| TASK-AUTH-004 residency issuer list mid-rotation | aggregate PRM advertises stale issuer | Client gets 401 from old issuer; falls back to next | Inherent — multi-issuer list provides resilience |

---

## §11 — Implementation notes

**§11.1** PRM body is canonical JSON (serde_json with sort_keys via custom serializer) so ETag SHA-256 is deterministic. Without sorting, different request handlers' bodies hash differently.

**§11.2** ETag truncation to 16 hex chars (64 bits) is enough for 1h-cache scope; risk of collision = ~10⁻¹⁰ per cache lifetime. Full SHA-256 hex (64 chars) is overkill.

**§11.3** The `Cache-Control: max-age=3600` is independent of ETag — clients can cache for 1h, then revalidate with ETag. Aggressive clients hit our endpoint once an hour; conservative clients hit only on miss.

**§11.4** The CORS preflight OPTIONS response is cached by browsers for 3600s (matching `Access-Control-Max-Age`). Eliminates preflight chatter on subsequent requests.

**§11.5** `Access-Control-Allow-Headers` is intentionally NOT set — PRM responses don't require any non-standard request headers (Authorization is optional + safelisted via CORS spec).

**§11.6** Per-module PRM Content-Type is identical to gateway. Content negotiation (Accept header) not supported — RFC 9728 mandates application/json.

**§11.7** Drift detection compares canonical-JSON SHA-256 of the SHARED fields only (not the full body — `scopes_supported` differs by design). Implementation: extract shared field subset, canonical-serialize, hash.

**§11.8** The `cyberos-mcp prm-cache-flush` CLI (slice 3 future) clears the in-memory cache + bumps the ETag base. Slice 2 has no CLI — NATS invalidation is the only flush mechanism.

**§11.9** Per-module PRM at unknown module returns 404, NOT 200 with empty `scopes_supported`. Differentiation lets clients distinguish "module exists but has no scopes" (empty list) from "module doesn't exist" (404).

**§11.10** The `prm_drift_log` table has no RLS (system-tenant scope only) because PRM is a system-level concept — no tenant owns metadata drift. Operator-only access via separate audit role.

**§11.11** Server registration records (`ModuleRegistration`) extended with `exposed_scopes: Vec<String>`; TASK-MCP-002 modified_files lists the type update.

**§11.12** Tail-sampling of `mcp.prm_served` rows (1% per task-audit skill / TASK-OBS-006) keeps the memory chain manageable at high request volume. The two other kinds are always emitted (low volume by design).

**§11.13** The `X-Robots-Tag: noindex` header also includes `, nofollow, nosnippet` per Google's documented best-practice for metadata endpoints.

**§11.14** Registry change events on NATS subject `cyberos.mcp.registry.changed` carry the changed module name; PRM cache subscriber rebuilds just that module's PRM (not all) for efficiency at scale.

**§11.15** The empty body on HEAD is enforced by axum middleware — Content-Length is set from a dry-run serialize without writing the body.

---

*End of TASK-MCP-005 spec.*
