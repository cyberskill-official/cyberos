//! FR-OBS-002 tenant-aware Grafana query proxy.
//!
//! The proxy is the Grafana security boundary: it authenticates the caller,
//! rewrites each backend query so it is scoped to the caller's tenant, rejects
//! user-supplied tenant filters, and records auditable query events.

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use anyhow::Context;
use axum::body::{to_bytes, Body, Bytes};
use axum::extract::State;
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::http::{HeaderMap, Method, Request, StatusCode, Uri};
use axum::response::{IntoResponse, Response as AxumResponse};
use axum::routing::{any, get};
use axum::Router;
use jsonwebtoken::jwk::JwkSet;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use promql_parser::label::{MatchOp, Matcher};
use promql_parser::parser::{self as promql, Expr, VectorSelector};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use thiserror::Error;
use tokio::net::TcpListener;
use uuid::Uuid;

const ROOT_TENANT_ID: &str = "00000000-0000-0000-0000-000000000000";
const MAX_QUERY_BODY_BYTES: usize = 1024 * 1024;

/// Runtime configuration for the Grafana query proxy.
#[derive(Debug, Clone)]
pub struct GrafanaProxyConfig {
    /// HTTP listen address.
    pub listen: SocketAddr,
    /// Upstream Prometheus base URL.
    pub prometheus_url: String,
    /// Upstream Loki base URL.
    pub loki_url: String,
    /// Upstream Tempo base URL.
    pub tempo_url: String,
    /// JWT verifier used to extract tenant identity.
    pub verifier: JwtVerifier,
}

/// Start the tenant-aware Grafana proxy.
pub async fn serve(config: GrafanaProxyConfig) -> anyhow::Result<()> {
    let listen = config.listen;
    let state = ProxyState::new(
        BackendUrls {
            prometheus: config.prometheus_url,
            loki: config.loki_url,
            tempo: config.tempo_url,
        },
        config.verifier,
    );
    let app = Router::new()
        .route("/ready", get(proxy_ready))
        .route("/metrics", get(proxy_metrics))
        .fallback(any(proxy_http))
        .with_state(state);
    let listener = TcpListener::bind(listen)
        .await
        .with_context(|| format!("bind Grafana proxy {listen}"))?;
    axum::serve(listener, app)
        .await
        .context("Grafana proxy stopped")
}

/// Upstream backend URLs.
#[derive(Debug, Clone)]
pub struct BackendUrls {
    /// Prometheus base URL, for example `http://prometheus:9090`.
    pub prometheus: String,
    /// Loki base URL, for example `http://loki:3100`.
    pub loki: String,
    /// Tempo base URL, for example `http://tempo:3200`.
    pub tempo: String,
}

impl BackendUrls {
    fn url_for(&self, backend: Backend) -> &str {
        match backend {
            Backend::Prometheus => &self.prometheus,
            Backend::Loki => &self.loki,
            Backend::Tempo => &self.tempo,
        }
    }
}

/// Supported Grafana datasource backend families.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Backend {
    /// Prometheus PromQL.
    Prometheus,
    /// Loki LogQL.
    Loki,
    /// Tempo TraceQL.
    Tempo,
}

impl Backend {
    /// Stable lowercase label used in audit rows and metrics.
    pub fn as_str(self) -> &'static str {
        match self {
            Backend::Prometheus => "prometheus",
            Backend::Loki => "loki",
            Backend::Tempo => "tempo",
        }
    }
}

/// JWT verifier. Production can use RS256 public keys exported from AUTH;
/// tests and local dev can use HS256.
#[derive(Debug, Clone)]
pub struct JwtVerifier {
    key_source: JwtKeySource,
    issuer: Option<String>,
    audience: Option<String>,
}

#[derive(Debug, Clone)]
enum JwtKeySource {
    Hs256(Arc<String>),
    Rs256Pem(Arc<String>),
    Rs256Jwks(Arc<JwkSet>),
}

impl JwtVerifier {
    /// Build an HS256 verifier.
    pub fn hs256(secret: impl Into<String>) -> Self {
        Self {
            key_source: JwtKeySource::Hs256(Arc::new(secret.into())),
            issuer: None,
            audience: None,
        }
    }

    /// Build an RS256 verifier from a PEM public key.
    pub fn rs256_public_pem(pem: impl Into<String>) -> Self {
        Self {
            key_source: JwtKeySource::Rs256Pem(Arc::new(pem.into())),
            issuer: None,
            audience: None,
        }
    }

    /// Build an RS256 verifier from a JWKS document fetched at startup.
    pub fn rs256_jwks_json(jwks_json: &str) -> Result<Self, ProxyError> {
        let jwks: JwkSet = serde_json::from_str(jwks_json)
            .map_err(|e| ProxyError::AuthFailed(format!("invalid jwks: {e}")))?;
        if jwks.keys.is_empty() {
            return Err(ProxyError::AuthFailed("jwks contains no keys".into()));
        }
        Ok(Self {
            key_source: JwtKeySource::Rs256Jwks(Arc::new(jwks)),
            issuer: None,
            audience: None,
        })
    }

    /// Require a specific issuer.
    pub fn with_issuer(mut self, issuer: impl Into<String>) -> Self {
        self.issuer = Some(issuer.into());
        self
    }

    /// Require a specific audience.
    pub fn with_audience(mut self, audience: impl Into<String>) -> Self {
        self.audience = Some(audience.into());
        self
    }

    /// Verify a bearer token and return CyberOS claims.
    pub fn verify_token(&self, token: &str) -> Result<ProxyClaims, ProxyError> {
        let (algorithm, key) = self.decoding_key(token)?;
        let mut validation = Validation::new(algorithm);
        validation.validate_exp = true;
        if let Some(issuer) = &self.issuer {
            validation.set_issuer(&[issuer]);
        }
        if let Some(audience) = &self.audience {
            validation.set_audience(&[audience]);
        } else {
            validation.validate_aud = false;
        }
        decode::<ProxyClaims>(token, &key, &validation)
            .map(|data| data.claims)
            .map_err(|e| ProxyError::AuthFailed(e.to_string()))
    }

    /// Verify an HTTP `Authorization: Bearer ...` header.
    pub fn verify_headers(&self, headers: &HeaderMap) -> Result<ProxyClaims, ProxyError> {
        let bearer = bearer_from_headers(headers)?;
        self.verify_token(&bearer)
    }

    fn decoding_key(&self, token: &str) -> Result<(Algorithm, DecodingKey), ProxyError> {
        match &self.key_source {
            JwtKeySource::Hs256(secret) => Ok((
                Algorithm::HS256,
                DecodingKey::from_secret(secret.as_bytes()),
            )),
            JwtKeySource::Rs256Pem(pem) => Ok((
                Algorithm::RS256,
                DecodingKey::from_rsa_pem(pem.as_bytes())
                    .map_err(|e| ProxyError::AuthFailed(e.to_string()))?,
            )),
            JwtKeySource::Rs256Jwks(jwks) => {
                let header =
                    decode_header(token).map_err(|e| ProxyError::AuthFailed(e.to_string()))?;
                if header.alg != Algorithm::RS256 {
                    return Err(ProxyError::AuthFailed(format!(
                        "unsupported jwks jwt algorithm {:?}",
                        header.alg
                    )));
                }
                let jwk = match header.kid.as_deref() {
                    Some(kid) => jwks
                        .find(kid)
                        .ok_or_else(|| ProxyError::AuthFailed(format!("unknown jwks kid {kid}")))?,
                    None if jwks.keys.len() == 1 => &jwks.keys[0],
                    None => return Err(ProxyError::AuthFailed("missing jwt kid".into())),
                };
                Ok((
                    Algorithm::RS256,
                    DecodingKey::from_jwk(jwk)
                        .map_err(|e| ProxyError::AuthFailed(e.to_string()))?,
                ))
            }
        }
    }
}

/// Minimal AUTH claims consumed by the proxy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyClaims {
    /// Subject id.
    pub sub: String,
    /// Tenant id. The nil UUID plus `root-admin` role bypasses injection.
    pub tenant_id: String,
    /// Expiry as seconds since epoch.
    pub exp: usize,
    /// Role names.
    #[serde(default)]
    pub roles: Vec<String>,
    /// Optional issuer.
    #[serde(default)]
    pub iss: Option<String>,
    /// Optional audience. AUTH uses an array; local test tokens may omit it.
    #[serde(default)]
    pub aud: Option<Value>,
}

impl ProxyClaims {
    fn is_root_admin(&self) -> bool {
        self.tenant_id == ROOT_TENANT_ID && self.roles.iter().any(|role| role == "root-admin")
    }
}

/// Proxy state shared by HTTP handlers and tests.
#[derive(Debug, Clone)]
pub struct ProxyState {
    backends: BackendUrls,
    verifier: JwtVerifier,
    client: Client,
    audit: AuditLog,
    metrics: Arc<ProxyMetrics>,
}

impl ProxyState {
    /// Build a proxy state.
    pub fn new(backends: BackendUrls, verifier: JwtVerifier) -> Self {
        Self {
            backends,
            verifier,
            client: Client::new(),
            audit: AuditLog::default(),
            metrics: Arc::new(ProxyMetrics::default()),
        }
    }

    /// Verify request headers.
    pub fn verify_headers(&self, headers: &HeaderMap) -> Result<ProxyClaims, ProxyError> {
        self.verifier.verify_headers(headers)
    }

    /// Return a snapshot of audit events emitted by this state.
    pub fn audit_events(&self) -> Vec<AuditEvent> {
        self.audit.events()
    }

    /// Rewrite a query for a verified caller and emit the matching audit row.
    pub fn process_query(
        &self,
        backend: Backend,
        original_query: &str,
        claims: &ProxyClaims,
        request_id: &str,
    ) -> Result<QueryRewrite, ProxyError> {
        let started = Instant::now();
        let query_sha256 = query_sha256(original_query);
        if user_supplied_tenant_label(backend, original_query)? {
            self.metrics.inc_counter(
                "obs_proxy_cross_tenant_attempts_total",
                claims.tenant_id.as_str(),
                backend,
                "rejected_user_supplied_tenant_id",
            );
            self.audit.record(AuditEvent::cross_tenant_attempt(
                claims,
                &query_sha256,
                attempted_tenant_value(original_query),
                request_id,
            ));
            return Err(ProxyError::UserSuppliedTenantId);
        }

        let (rewritten_query, outcome) = if claims.is_root_admin() {
            (
                original_query.to_string(),
                QueryOutcome::RootAdminUnfiltered,
            )
        } else {
            (
                inject_query(backend, original_query, &claims.tenant_id)?,
                QueryOutcome::Proxied,
            )
        };
        let latency_ms = started.elapsed().as_secs_f64() * 1000.0;
        self.metrics.inc_counter(
            "obs_proxy_requests_total",
            claims.tenant_id.as_str(),
            backend,
            outcome.as_str(),
        );
        self.metrics.observe_injection_latency(backend, latency_ms);
        self.audit.record(AuditEvent::query_proxied(
            claims,
            backend,
            &query_sha256,
            outcome,
            latency_ms,
            request_id,
        ));
        Ok(QueryRewrite {
            backend,
            original_query: original_query.to_string(),
            rewritten_query,
            outcome,
            request_id: request_id.to_string(),
            injection_latency_ms: latency_ms,
        })
    }

    fn record_backend_error(
        &self,
        claims: &ProxyClaims,
        backend: Backend,
        query: &str,
        request_id: &str,
    ) {
        self.metrics.inc_counter(
            "obs_proxy_requests_total",
            claims.tenant_id.as_str(),
            backend,
            "backend_error",
        );
        self.audit.record(AuditEvent::query_proxied(
            claims,
            backend,
            &query_sha256(query),
            QueryOutcome::BackendError,
            0.0,
            request_id,
        ));
    }
}

/// Query rewrite result.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryRewrite {
    /// Backend family.
    pub backend: Backend,
    /// Original caller query.
    pub original_query: String,
    /// Tenant-scoped query forwarded to the backend.
    pub rewritten_query: String,
    /// Audit outcome.
    pub outcome: QueryOutcome,
    /// Request id associated with the audit row.
    pub request_id: String,
    /// Injection overhead in milliseconds.
    pub injection_latency_ms: f64,
}

/// Query audit outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryOutcome {
    /// Query was rewritten and proxied.
    Proxied,
    /// Query was rejected because caller supplied a reserved tenant label.
    RejectedUserSuppliedTenantId,
    /// Request was unauthenticated.
    RejectedUnauthenticated,
    /// Backend returned or transport produced an error.
    BackendError,
    /// Root admin query forwarded without tenant injection.
    RootAdminUnfiltered,
}

impl QueryOutcome {
    fn as_str(self) -> &'static str {
        match self {
            QueryOutcome::Proxied => "proxied",
            QueryOutcome::RejectedUserSuppliedTenantId => "rejected_user_supplied_tenant_id",
            QueryOutcome::RejectedUnauthenticated => "rejected_unauthenticated",
            QueryOutcome::BackendError => "backend_error",
            QueryOutcome::RootAdminUnfiltered => "root_admin_unfiltered",
        }
    }
}

/// Durable audit event shape emitted by the proxy.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AuditEvent {
    /// Canonical audit row kind.
    pub kind: String,
    /// Severity label for incident routing.
    pub severity: Option<String>,
    /// Canonical payload.
    pub payload: Value,
}

impl AuditEvent {
    fn query_proxied(
        claims: &ProxyClaims,
        backend: Backend,
        query_sha256: &str,
        outcome: QueryOutcome,
        latency_ms: f64,
        request_id: &str,
    ) -> Self {
        Self {
            kind: "obs.query_proxied".to_string(),
            severity: (outcome == QueryOutcome::RootAdminUnfiltered).then(|| "sev-2".to_string()),
            payload: json!({
                "tenant_id": claims.tenant_id,
                "caller_subject_id": claims.sub,
                "backend": backend.as_str(),
                "query_sha256": query_sha256,
                "outcome": outcome.as_str(),
                "latency_ms": latency_ms,
                "request_id": request_id,
            }),
        }
    }

    fn cross_tenant_attempt(
        claims: &ProxyClaims,
        query_sha256: &str,
        attempted_label_value: Option<String>,
        request_id: &str,
    ) -> Self {
        Self {
            kind: "obs.cross_tenant_query_attempt".to_string(),
            severity: Some("sev-1".to_string()),
            payload: json!({
                "caller_tenant_id": claims.tenant_id,
                "caller_subject_id": claims.sub,
                "attempted_label_value": attempted_label_value,
                "query_sha256": query_sha256,
                "request_id": request_id,
            }),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct AuditLog {
    rows: Arc<Mutex<Vec<AuditEvent>>>,
}

impl AuditLog {
    fn record(&self, event: AuditEvent) {
        tracing::info!(
            audit_kind = %event.kind,
            audit_payload = %event.payload,
            "obs proxy audit event"
        );
        self.rows.lock().expect("audit lock").push(event);
    }

    fn events(&self) -> Vec<AuditEvent> {
        self.rows.lock().expect("audit lock").clone()
    }
}

#[derive(Debug, Default)]
struct ProxyMetrics {
    counters: Mutex<BTreeMap<(String, String, String, String), u64>>,
    latencies_ms: Mutex<Vec<(Backend, f64)>>,
}

impl ProxyMetrics {
    fn inc_counter(&self, metric: &str, tenant_id: &str, backend: Backend, outcome: &str) {
        let mut counters = self.counters.lock().expect("proxy metrics lock");
        *counters
            .entry((
                metric.to_string(),
                tenant_id.to_string(),
                backend.as_str().to_string(),
                outcome.to_string(),
            ))
            .or_insert(0) += 1;
    }

    fn observe_injection_latency(&self, backend: Backend, latency_ms: f64) {
        self.latencies_ms
            .lock()
            .expect("proxy latency lock")
            .push((backend, latency_ms));
    }

    fn render(&self) -> String {
        let mut out = String::new();
        let counters = self.counters.lock().expect("proxy metrics lock");
        for ((metric, tenant_id, backend, outcome), value) in counters.iter() {
            out.push_str(metric);
            out.push_str("{tenant_id=\"");
            out.push_str(&escape_label_value(tenant_id));
            out.push_str("\",backend=\"");
            out.push_str(backend);
            out.push_str("\",outcome=\"");
            out.push_str(outcome);
            out.push_str("\"} ");
            out.push_str(&value.to_string());
            out.push('\n');
        }
        let latencies = self.latencies_ms.lock().expect("proxy latency lock");
        for (backend, latency_ms) in latencies.iter() {
            out.push_str("obs_proxy_injection_latency_ms{backend=\"");
            out.push_str(backend.as_str());
            out.push_str("\"} ");
            out.push_str(&format!("{latency_ms:.6}"));
            out.push('\n');
        }
        out
    }
}

/// Proxy errors mapped to HTTP responses.
#[derive(Debug, Error)]
pub enum ProxyError {
    /// No bearer token.
    #[error("missing bearer token")]
    MissingBearer,
    /// Malformed bearer token.
    #[error("malformed bearer token")]
    MalformedBearer,
    /// JWT verification failed.
    #[error("auth failed: {0}")]
    AuthFailed(String),
    /// Unsupported proxy endpoint.
    #[error("unsupported Grafana proxy endpoint: {0}")]
    UnsupportedEndpoint(String),
    /// No query parameter was found.
    #[error("query parameter required")]
    MissingQuery,
    /// Caller supplied `tenant_id` or `resource.tenant_id`.
    #[error("user supplied tenant_id label")]
    UserSuppliedTenantId,
    /// Query parse failed.
    #[error("query parse failed: {backend:?}: {reason}")]
    ParseFailed {
        /// Backend whose parser failed.
        backend: Backend,
        /// Failure reason.
        reason: String,
    },
    /// Request body could not be read.
    #[error("request body read failed: {0}")]
    BodyRead(String),
    /// Backend transport failed.
    #[error("backend error: {0}")]
    Backend(String),
}

impl ProxyError {
    fn status(&self) -> StatusCode {
        match self {
            ProxyError::MissingBearer | ProxyError::MalformedBearer | ProxyError::AuthFailed(_) => {
                StatusCode::UNAUTHORIZED
            }
            ProxyError::UserSuppliedTenantId
            | ProxyError::MissingQuery
            | ProxyError::ParseFailed { .. } => StatusCode::BAD_REQUEST,
            ProxyError::Backend(_) => StatusCode::SERVICE_UNAVAILABLE,
            ProxyError::UnsupportedEndpoint(_) | ProxyError::BodyRead(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn body(&self) -> Value {
        match self {
            ProxyError::UserSuppliedTenantId => json!({
                "error": "user_supplied_tenant_id",
                "reason": "tenant_id label is reserved; do not include in query",
            }),
            ProxyError::MissingBearer | ProxyError::MalformedBearer | ProxyError::AuthFailed(_) => {
                json!({ "error": "unauthorized", "reason": self.to_string() })
            }
            ProxyError::ParseFailed { reason, .. } => {
                json!({ "error": "parse_error", "reason": reason })
            }
            ProxyError::Backend(reason) => json!({ "error": "backend_error", "reason": reason }),
            ProxyError::MissingQuery => json!({ "error": "missing_query" }),
            ProxyError::UnsupportedEndpoint(endpoint) => {
                json!({ "error": "unsupported_endpoint", "path": endpoint })
            }
            ProxyError::BodyRead(reason) => {
                json!({ "error": "body_read_failed", "reason": reason })
            }
        }
    }

    fn into_response(self) -> AxumResponse {
        (self.status(), self.body().to_string()).into_response()
    }
}

async fn proxy_ready() -> &'static str {
    "ready\n"
}

async fn proxy_metrics(State(state): State<ProxyState>) -> String {
    state.metrics.render()
}

async fn proxy_http(State(state): State<ProxyState>, req: Request<Body>) -> AxumResponse {
    match proxy_http_inner(state, req).await {
        Ok(response) => response,
        Err(error) => error.into_response(),
    }
}

async fn proxy_http_inner(
    state: ProxyState,
    req: Request<Body>,
) -> Result<AxumResponse, ProxyError> {
    let (parts, body) = req.into_parts();
    let request_id = parts
        .headers
        .get("x-request-id")
        .and_then(|h| h.to_str().ok())
        .map(str::to_string)
        .unwrap_or_else(|| format!("obs_{}", Uuid::new_v4()));
    let claims = state.verify_headers(&parts.headers).map_err(|error| {
        if matches!(
            error,
            ProxyError::MissingBearer | ProxyError::MalformedBearer | ProxyError::AuthFailed(_)
        ) {
            state.audit.record(AuditEvent {
                kind: "obs.query_proxied".to_string(),
                severity: None,
                payload: json!({
                    "tenant_id": null,
                    "caller_subject_id": null,
                    "backend": null,
                    "query_sha256": null,
                    "outcome": QueryOutcome::RejectedUnauthenticated.as_str(),
                    "latency_ms": 0.0,
                    "request_id": request_id,
                }),
            });
        }
        error
    })?;
    let backend = detect_backend(&parts.uri)?;
    let body_bytes = to_bytes(body, MAX_QUERY_BODY_BYTES)
        .await
        .map_err(|e| ProxyError::BodyRead(e.to_string()))?;
    let query = extract_query(&parts.method, &parts.uri, &parts.headers, &body_bytes)?;
    let rewrite = state.process_query(backend, &query, &claims, &request_id)?;
    let forward = forward_target(
        &state.backends,
        backend,
        &parts.uri,
        &rewrite.rewritten_query,
    )?;
    let response = forward_request(
        &state.client,
        parts.method,
        &parts.headers,
        forward,
        body_for_forward(&parts.headers, &body_bytes, &rewrite.rewritten_query),
    )
    .await;

    match response {
        Ok(resp) => Ok(resp),
        Err(error) => {
            state.record_backend_error(&claims, backend, &query, &request_id);
            Err(error)
        }
    }
}

async fn forward_request(
    client: &Client,
    method: Method,
    headers: &HeaderMap,
    url: String,
    body: Bytes,
) -> Result<AxumResponse, ProxyError> {
    let reqwest_method = reqwest::Method::from_bytes(method.as_str().as_bytes())
        .map_err(|e| ProxyError::Backend(e.to_string()))?;
    let mut request = client.request(reqwest_method, url);
    if let Some(content_type) = headers.get(CONTENT_TYPE).and_then(|h| h.to_str().ok()) {
        request = request.header(CONTENT_TYPE, content_type);
    }
    let response = request
        .body(body)
        .send()
        .await
        .map_err(|e| ProxyError::Backend(e.to_string()))?;
    let status =
        StatusCode::from_u16(response.status().as_u16()).unwrap_or(StatusCode::SERVICE_UNAVAILABLE);
    let bytes = response
        .bytes()
        .await
        .map_err(|e| ProxyError::Backend(e.to_string()))?;
    Ok((status, bytes).into_response())
}

fn body_for_forward(headers: &HeaderMap, body: &Bytes, rewritten_query: &str) -> Bytes {
    let content_type = headers
        .get(CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default();
    if content_type.contains("application/x-www-form-urlencoded") {
        let raw = String::from_utf8_lossy(body);
        Bytes::from(replace_query_param(raw.as_ref(), "query", rewritten_query))
    } else {
        body.clone()
    }
}

fn detect_backend(uri: &Uri) -> Result<Backend, ProxyError> {
    let path = uri.path();
    if path.starts_with("/loki/") {
        Ok(Backend::Loki)
    } else if path.starts_with("/tempo/")
        || path.starts_with("/api/search")
        || path.starts_with("/api/traces")
    {
        Ok(Backend::Tempo)
    } else if path.starts_with("/api/v1/") {
        Ok(Backend::Prometheus)
    } else {
        Err(ProxyError::UnsupportedEndpoint(path.to_string()))
    }
}

fn extract_query(
    method: &Method,
    uri: &Uri,
    headers: &HeaderMap,
    body: &Bytes,
) -> Result<String, ProxyError> {
    if let Some(value) = query_param(uri.query().unwrap_or_default(), "query")
        .or_else(|| query_param(uri.query().unwrap_or_default(), "q"))
    {
        return Ok(value);
    }
    if *method == Method::POST {
        let content_type = headers
            .get(CONTENT_TYPE)
            .and_then(|h| h.to_str().ok())
            .unwrap_or_default();
        if content_type.contains("application/x-www-form-urlencoded") {
            let raw = String::from_utf8_lossy(body);
            if let Some(value) =
                query_param(raw.as_ref(), "query").or_else(|| query_param(raw.as_ref(), "q"))
            {
                return Ok(value);
            }
        }
    }
    Err(ProxyError::MissingQuery)
}

fn forward_target(
    backends: &BackendUrls,
    backend: Backend,
    uri: &Uri,
    rewritten_query: &str,
) -> Result<String, ProxyError> {
    let base = backends.url_for(backend).trim_end_matches('/');
    let upstream_path = match backend {
        Backend::Loki => uri.path().to_string(),
        Backend::Prometheus => uri.path().to_string(),
        Backend::Tempo => uri
            .path()
            .strip_prefix("/tempo")
            .unwrap_or(uri.path())
            .to_string(),
    };
    let query = replace_query_param(uri.query().unwrap_or_default(), "query", rewritten_query);
    if query.is_empty() {
        Ok(format!("{base}{upstream_path}"))
    } else {
        Ok(format!("{base}{upstream_path}?{query}"))
    }
}

fn query_param(query: &str, key: &str) -> Option<String> {
    for part in query.split('&').filter(|part| !part.is_empty()) {
        let (raw_key, raw_value) = part.split_once('=').unwrap_or((part, ""));
        if percent_decode(raw_key) == key {
            return Some(percent_decode(raw_value));
        }
    }
    None
}

fn replace_query_param(query: &str, key: &str, value: &str) -> String {
    let mut found = false;
    let mut parts = Vec::new();
    for part in query.split('&').filter(|part| !part.is_empty()) {
        let (raw_key, raw_value) = part.split_once('=').unwrap_or((part, ""));
        let decoded_key = percent_decode(raw_key);
        if decoded_key == key || (key == "query" && decoded_key == "q") {
            found = true;
            parts.push(format!(
                "{}={}",
                percent_encode(&decoded_key),
                percent_encode(value)
            ));
        } else {
            parts.push(format!(
                "{}={}",
                percent_encode(&decoded_key),
                percent_encode(&percent_decode(raw_value))
            ));
        }
    }
    if !found {
        parts.push(format!("{}={}", percent_encode(key), percent_encode(value)));
    }
    parts.join("&")
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                if let Ok(hex) = u8::from_str_radix(&input[i + 1..i + 3], 16) {
                    out.push(hex);
                    i += 3;
                } else {
                    out.push(bytes[i]);
                    i += 1;
                }
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn percent_encode(input: &str) -> String {
    let mut out = String::new();
    for b in input.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

fn bearer_from_headers(headers: &HeaderMap) -> Result<String, ProxyError> {
    let raw = headers
        .get(AUTHORIZATION)
        .ok_or(ProxyError::MissingBearer)?
        .to_str()
        .map_err(|_| ProxyError::MalformedBearer)?;
    raw.strip_prefix("Bearer ")
        .filter(|token| !token.trim().is_empty())
        .map(str::to_string)
        .ok_or(ProxyError::MalformedBearer)
}

/// Inject a tenant filter into a backend query.
pub fn inject_query(backend: Backend, query: &str, tenant_id: &str) -> Result<String, ProxyError> {
    if tenant_id.trim().is_empty() {
        return Err(ProxyError::ParseFailed {
            backend,
            reason: "tenant_id_required".to_string(),
        });
    }
    match backend {
        Backend::Prometheus => inject_promql(query, tenant_id),
        Backend::Loki => {
            if has_user_supplied_tenant_id(query) {
                return Err(ProxyError::UserSuppliedTenantId);
            }
            inject_logql(query, tenant_id)
        }
        Backend::Tempo => {
            if has_user_supplied_tenant_id(query) {
                return Err(ProxyError::UserSuppliedTenantId);
            }
            inject_traceql(query, tenant_id)
        }
    }
}

/// Detect user-supplied reserved tenant labels.
pub fn has_user_supplied_tenant_id(query: &str) -> bool {
    query.contains("tenant_id") || query.contains("resource.tenant_id")
}

fn user_supplied_tenant_label(backend: Backend, query: &str) -> Result<bool, ProxyError> {
    match backend {
        Backend::Prometheus => {
            let expr = parse_promql(query)?;
            Ok(promql_expr_has_label(&expr, "tenant_id"))
        }
        Backend::Loki | Backend::Tempo => Ok(has_user_supplied_tenant_id(query)),
    }
}

fn inject_promql(query: &str, tenant_id: &str) -> Result<String, ProxyError> {
    let mut expr = parse_promql(query)?;
    if promql_expr_has_label(&expr, "tenant_id") {
        return Err(ProxyError::UserSuppliedTenantId);
    }
    inject_promql_expr(&mut expr, tenant_id);
    Ok(expr.prettify())
}

fn parse_promql(query: &str) -> Result<Expr, ProxyError> {
    promql::parse(query).map_err(|e| ProxyError::ParseFailed {
        backend: Backend::Prometheus,
        reason: e.to_string(),
    })
}

fn promql_expr_has_label(expr: &Expr, label: &str) -> bool {
    match expr {
        Expr::Aggregate(node) => {
            promql_expr_has_label(&node.expr, label)
                || node
                    .param
                    .as_ref()
                    .is_some_and(|param| promql_expr_has_label(param, label))
        }
        Expr::Unary(node) => promql_expr_has_label(&node.expr, label),
        Expr::Binary(node) => {
            promql_expr_has_label(&node.lhs, label) || promql_expr_has_label(&node.rhs, label)
        }
        Expr::Paren(node) => promql_expr_has_label(&node.expr, label),
        Expr::Subquery(node) => promql_expr_has_label(&node.expr, label),
        Expr::VectorSelector(selector) => !selector.matchers.find_matchers(label).is_empty(),
        Expr::MatrixSelector(selector) => !selector.vs.matchers.find_matchers(label).is_empty(),
        Expr::Call(node) => node
            .args
            .args
            .iter()
            .any(|arg| promql_expr_has_label(arg, label)),
        Expr::Extension(node) => node
            .expr
            .children()
            .iter()
            .any(|child| promql_expr_has_label(child, label)),
        Expr::NumberLiteral(_) | Expr::StringLiteral(_) => false,
    }
}

fn inject_promql_expr(expr: &mut Expr, tenant_id: &str) {
    match expr {
        Expr::Aggregate(node) => {
            inject_promql_expr(&mut node.expr, tenant_id);
            if let Some(param) = &mut node.param {
                inject_promql_expr(param, tenant_id);
            }
        }
        Expr::Unary(node) => inject_promql_expr(&mut node.expr, tenant_id),
        Expr::Binary(node) => {
            inject_promql_expr(&mut node.lhs, tenant_id);
            inject_promql_expr(&mut node.rhs, tenant_id);
        }
        Expr::Paren(node) => inject_promql_expr(&mut node.expr, tenant_id),
        Expr::Subquery(node) => inject_promql_expr(&mut node.expr, tenant_id),
        Expr::VectorSelector(selector) => inject_promql_selector(selector, tenant_id),
        Expr::MatrixSelector(selector) => inject_promql_selector(&mut selector.vs, tenant_id),
        Expr::Call(node) => {
            for arg in &mut node.args.args {
                inject_promql_expr(arg, tenant_id);
            }
        }
        Expr::Extension(_) | Expr::NumberLiteral(_) | Expr::StringLiteral(_) => {}
    }
}

fn inject_promql_selector(selector: &mut VectorSelector, tenant_id: &str) {
    let matcher = Matcher::new(MatchOp::Equal, "tenant_id", tenant_id);
    if selector.matchers.or_matchers.is_empty() {
        selector.matchers.matchers.push(matcher);
    } else {
        for group in &mut selector.matchers.or_matchers {
            group.push(matcher.clone());
        }
    }
}

fn inject_logql(query: &str, tenant_id: &str) -> Result<String, ProxyError> {
    ensure_balanced_brackets(query, Backend::Loki)?;
    let trimmed = query.trim_start();
    let leading_ws_len = query.len() - trimmed.len();
    if !trimmed.starts_with('{') {
        return Ok(format!(
            "{}{} {}",
            &query[..leading_ws_len],
            label_set("tenant_id", tenant_id),
            trimmed
        ));
    }
    let chars: Vec<char> = trimmed.chars().collect();
    let (selector, next) = read_balanced_selector(&chars, 0, Backend::Loki)?;
    let rest: String = chars[next..].iter().collect();
    Ok(format!(
        "{}{}{}",
        &query[..leading_ws_len],
        inject_selector(&selector, "tenant_id", tenant_id),
        rest
    ))
}

fn inject_traceql(query: &str, tenant_id: &str) -> Result<String, ProxyError> {
    ensure_balanced_brackets(query, Backend::Tempo)?;
    let trimmed = query.trim();
    let tenant_filter = format!(
        r#"resource.tenant_id = "{}""#,
        escape_label_value(tenant_id)
    );
    if trimmed.is_empty() {
        return Ok(format!("{{ {tenant_filter} }}"));
    }
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        let inner = trimmed.trim_start_matches('{').trim_end_matches('}').trim();
        if inner.is_empty() {
            Ok(format!("{{ {tenant_filter} }}"))
        } else {
            Ok(format!("{{ {inner} && {tenant_filter} }}"))
        }
    } else {
        Ok(format!("({trimmed}) && {tenant_filter}"))
    }
}

fn ensure_balanced_brackets(query: &str, backend: Backend) -> Result<(), ProxyError> {
    let mut curly_depth = 0i32;
    let mut square_depth = 0i32;
    let mut in_string = false;
    let mut escaped = false;
    for ch in query.chars() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '{' => curly_depth += 1,
            '}' => curly_depth -= 1,
            '[' => square_depth += 1,
            ']' => square_depth -= 1,
            _ => {}
        }
        if curly_depth < 0 || square_depth < 0 {
            return Err(ProxyError::ParseFailed {
                backend,
                reason: "unbalanced query delimiters".to_string(),
            });
        }
    }
    if in_string || curly_depth != 0 || square_depth != 0 {
        return Err(ProxyError::ParseFailed {
            backend,
            reason: "unbalanced query delimiters".to_string(),
        });
    }
    Ok(())
}

fn read_balanced_selector(
    chars: &[char],
    start: usize,
    backend: Backend,
) -> Result<(String, usize), ProxyError> {
    let mut i = start;
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escaped = false;
    while i < chars.len() {
        let ch = chars[i];
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
        } else {
            match ch {
                '"' => in_string = true,
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        let selector: String = chars[start..=i].iter().collect();
                        return Ok((selector, i + 1));
                    }
                }
                _ => {}
            }
        }
        i += 1;
    }
    Err(ProxyError::ParseFailed {
        backend,
        reason: "unclosed label selector".to_string(),
    })
}

fn inject_selector(selector: &str, key: &str, value: &str) -> String {
    let inner = selector
        .trim_start_matches('{')
        .trim_end_matches('}')
        .trim();
    if inner.is_empty() {
        label_set(key, value)
    } else {
        format!("{{{},{}=\"{}\"}}", inner, key, escape_label_value(value))
    }
}

fn label_set(key: &str, value: &str) -> String {
    format!(r#"{{{}="{}"}}"#, key, escape_label_value(value))
}

fn escape_label_value(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn query_sha256(query: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(query.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn attempted_tenant_value(query: &str) -> Option<String> {
    for needle in ["resource.tenant_id", "tenant_id"] {
        if let Some(pos) = query.find(needle) {
            let tail = &query[pos + needle.len()..];
            if let Some(eq_pos) = tail.find('=') {
                let raw = tail[eq_pos + 1..].trim_start();
                if let Some(stripped) = raw.strip_prefix('"') {
                    if let Some(end) = stripped.find('"') {
                        return Some(stripped[..end].to_string());
                    }
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn promql_query_gets_tenant_selector() {
        let rewritten = inject_query(Backend::Prometheus, "rate(foo[5m])", "tenant-a").unwrap();
        assert_eq!(rewritten, r#"rate(foo{tenant_id="tenant-a"}[5m])"#);
    }

    #[test]
    fn promql_existing_selector_gets_tenant_label() {
        let rewritten = inject_query(Backend::Prometheus, r#"foo{x="y"}"#, "tenant-a").unwrap();
        assert!(rewritten.starts_with("foo{"));
        assert!(rewritten.contains(r#"x="y""#));
        assert!(rewritten.contains(r#"tenant_id="tenant-a""#));
    }

    #[test]
    fn promql_complex_query_injects_every_selector() {
        let rewritten = inject_query(
            Backend::Prometheus,
            "sum(rate(foo[5m])) / sum(rate(bar[5m]))",
            "tenant-a",
        )
        .unwrap();
        assert_eq!(rewritten.matches(r#"tenant_id="tenant-a""#).count(), 2);
    }

    #[test]
    fn logql_preserves_pipe_stages() {
        let rewritten = inject_query(
            Backend::Loki,
            r#"{service="api"} | json | line_format "{{.message}}""#,
            "tenant-a",
        )
        .unwrap();
        assert!(rewritten.starts_with(r#"{service="api",tenant_id="tenant-a"}"#));
        assert!(rewritten.contains("| json"));
        assert!(rewritten.contains("| line_format"));
    }

    #[test]
    fn traceql_injects_resource_tenant_filter() {
        let rewritten = inject_query(
            Backend::Tempo,
            r#"{ service.name = "ai-gateway" }"#,
            "tenant-a",
        )
        .unwrap();
        assert_eq!(
            rewritten,
            r#"{ service.name = "ai-gateway" && resource.tenant_id = "tenant-a" }"#
        );
    }

    #[test]
    fn user_supplied_tenant_label_is_rejected() {
        let err = inject_query(
            Backend::Prometheus,
            r#"http_requests_total{tenant_id="other"}"#,
            "tenant-a",
        )
        .unwrap_err();
        assert!(matches!(err, ProxyError::UserSuppliedTenantId));
    }
}
