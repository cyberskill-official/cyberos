//! Shared error rendering for the TASK-MCP-004 OAuth endpoints.
//!
//! Endpoint logic returns `Result<T, EndpointError>`. Client errors are the RFC 6749 §5.2 shapes from
//! [`OAuthError`]; database/key/encoding failures collapse to a generic 500 that leaks no internal
//! detail; and an unconfigured gateway (no `MCP_DATABASE_URL`) returns 503. `From` impls let handlers
//! use `?` on `sqlx::Error`, `JwtError`, and `OAuthError`.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

use super::error::OAuthError;
use super::jwt::JwtError;

/// An OAuth endpoint failure.
#[derive(Debug)]
pub enum EndpointError {
    /// A client error (RFC 6749 §5.2) rendered as its 4xx JSON body.
    OAuth(OAuthError),
    /// An internal failure (database, signing key, encoding) rendered as a generic 500.
    Internal,
    /// The OAuth endpoints are not configured (no database) - 503.
    Unconfigured,
}

impl From<OAuthError> for EndpointError {
    fn from(e: OAuthError) -> Self {
        EndpointError::OAuth(e)
    }
}

impl From<sqlx::Error> for EndpointError {
    fn from(_: sqlx::Error) -> Self {
        EndpointError::Internal
    }
}

impl From<JwtError> for EndpointError {
    fn from(_: JwtError) -> Self {
        EndpointError::Internal
    }
}

impl IntoResponse for EndpointError {
    fn into_response(self) -> Response {
        match self {
            EndpointError::OAuth(e) => e.into_response(),
            EndpointError::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "server_error" })),
            )
                .into_response(),
            EndpointError::Unconfigured => (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "error": "oauth_not_configured",
                    "detail": "set MCP_DATABASE_URL to enable the TASK-MCP-004 OAuth endpoints"
                })),
            )
                .into_response(),
        }
    }
}
