//! obs-proxy binary - the axum HTTP shell around `handler::handle` (FR-OBS-002).
//!
//! Every Grafana datasource query lands on the fallback handler, is tenant-scoped by `handle`, and is
//! forwarded to the matching backend. The verifier is built from the auth JWKS at boot (or a dev HS256
//! secret when `OBS_AUTH_JWKS_URL` is unset). The audit sink is the in-memory recorder for now; wiring
//! the real memory sink (l1_audit_log) is a follow-up.

use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::{OriginalUri, State},
    http::{header, HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response},
    Router,
};
use cyberos_obs_proxy::audit::{AuditSink, RecordingSink};
use cyberos_obs_proxy::auth::Authenticator;
use cyberos_obs_proxy::error::ProxyError;
use cyberos_obs_proxy::forwarder::{BackendUrls, HttpForwarder};
use cyberos_obs_proxy::handler;

struct AppState {
    auth: Authenticator,
    forwarder: HttpForwarder,
    sink: Arc<dyn AuditSink>,
}

#[tokio::main]
async fn main() {
    let auth = build_authenticator().await;
    let state = Arc::new(AppState {
        auth,
        forwarder: HttpForwarder::new(BackendUrls::from_env()),
        sink: Arc::new(RecordingSink::default()),
    });

    let app = Router::new()
        .fallback(proxy_handler)
        .with_state(state);

    let bind = std::env::var("OBS_PROXY_BIND").unwrap_or_else(|_| "0.0.0.0:8088".into());
    let listener = tokio::net::TcpListener::bind(&bind)
        .await
        .unwrap_or_else(|e| panic!("obs-proxy: cannot bind {bind}: {e}"));
    eprintln!("obs-proxy listening on {bind}");
    axum::serve(listener, app).await.expect("obs-proxy: serve failed");
}

/// Build the JWT verifier: from the auth JWKS in production, or a dev HS256 secret locally.
async fn build_authenticator() -> Authenticator {
    match std::env::var("OBS_AUTH_JWKS_URL") {
        Ok(url) => {
            let jwks = reqwest::get(&url)
                .await
                .and_then(|r| r.error_for_status())
                .unwrap_or_else(|e| panic!("obs-proxy: fetch jwks {url}: {e}"));
            let body = jwks
                .text()
                .await
                .unwrap_or_else(|e| panic!("obs-proxy: read jwks body: {e}"));
            Authenticator::from_jwks(&body).unwrap_or_else(|e| panic!("obs-proxy: parse jwks: {e}"))
        }
        Err(_) => {
            let secret =
                std::env::var("OBS_DEV_HS256_SECRET").unwrap_or_else(|_| "dev-insecure-secret".into());
            eprintln!(
                "obs-proxy: OBS_AUTH_JWKS_URL unset - using a DEV HS256 secret (NOT for production)"
            );
            Authenticator::from_hs256_secret(secret.as_bytes())
        }
    }
}

async fn proxy_handler(
    State(state): State<Arc<AppState>>,
    method: Method,
    OriginalUri(uri): OriginalUri,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let token = bearer(&headers);
    let path = uri.path().to_string();
    // The query string is in the URL (GET) or the form body (POST) - both are form-urlencoded.
    let raw_query = if method == Method::POST && !body.is_empty() {
        String::from_utf8_lossy(&body).into_owned()
    } else {
        uri.query().unwrap_or("").to_string()
    };
    let request_id = format!("obs_{}", request_nonce());

    match handler::handle(
        &state.auth,
        &state.forwarder,
        state.sink.as_ref(),
        token.as_deref(),
        &path,
        &raw_query,
        &request_id,
    )
    .await
    {
        Ok(body) => (StatusCode::OK, body).into_response(),
        Err(e) => error_response(e),
    }
}

fn bearer(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .map(|s| s.to_string())
}

fn error_response(e: ProxyError) -> Response {
    let status = match e {
        ProxyError::AuthFailed(_) => StatusCode::UNAUTHORIZED,
        ProxyError::UserSuppliedTenantId => StatusCode::BAD_REQUEST,
        ProxyError::ParseFailed { .. } => StatusCode::BAD_REQUEST,
        ProxyError::UnsupportedPath(_) => StatusCode::NOT_FOUND,
        ProxyError::BackendUnreachable(_) => StatusCode::SERVICE_UNAVAILABLE,
    };
    (status, e.to_string()).into_response()
}

fn request_nonce() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}
