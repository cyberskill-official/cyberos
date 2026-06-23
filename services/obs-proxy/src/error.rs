//! Proxy error and backend enums (FR-OBS-002 §3).
//!
//! Slice 1 carries the variants the LogQL injector needs. The proxy router slice adds a
//! `Backend(#[from] reqwest::Error)` variant for backend-forward failures.

/// Which OBS backend a request targets. The injection rules differ per query language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Prometheus,
    Loki,
    Tempo,
}

/// Errors the proxy can return. Each maps to an HTTP status at the router layer
/// (AuthFailed -> 401, UserSuppliedTenantId -> 400 + sev-1 audit, ParseFailed -> 400).
#[derive(Debug, thiserror::Error)]
pub enum ProxyError {
    #[error("auth failed: {0}")]
    AuthFailed(String),

    #[error("user supplied tenant_id label (bypass attempt)")]
    UserSuppliedTenantId,

    #[error("query parse failed ({backend:?}): {reason}")]
    ParseFailed { backend: Backend, reason: String },

    #[error("unsupported request path: {0}")]
    UnsupportedPath(String),

    #[error("backend unreachable: {0}")]
    BackendUnreachable(String),
}
