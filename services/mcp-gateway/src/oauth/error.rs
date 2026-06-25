//! FR-MCP-004 §1.6 + DEC-820 + RFC 6749 §5.2.
//!
//! Closed 6-value OAuth error code; all token-endpoint error responses carry
//! `Cache-Control: no-store` and `Pragma: no-cache` per RFC 6749 §5.2.

use axum::{
    http::{
        header::{CACHE_CONTROL, PRAGMA},
        HeaderValue, StatusCode,
    },
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

/// The six error codes permitted by RFC 6749 §5.2 and the MCP profile (DEC-820).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OAuthErrorCode {
    /// The request is missing a parameter, malformed, or otherwise invalid.
    InvalidRequest,
    /// Client authentication failed or the client is unknown.
    InvalidClient,
    /// The grant (authorization code or refresh token) is invalid, expired, or revoked.
    InvalidGrant,
    /// The client is not authorized to use this grant type.
    UnauthorizedClient,
    /// The requested grant type is not supported by this server.
    UnsupportedGrantType,
    /// The requested scope is unknown or malformed.
    InvalidScope,
}

/// The JSON error body returned to the client, per RFC 6749 §5.2.
#[derive(Debug, Serialize)]
pub struct OAuthErrorBody {
    /// The error code.
    pub error: OAuthErrorCode,
    /// An optional human-readable explanation (already PII-scrubbed before construction).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_description: Option<String>,
}

/// A typed OAuth error carrying the HTTP status, the error code, and an optional description.
#[derive(Debug)]
pub struct OAuthError {
    /// The HTTP status to return.
    pub status: StatusCode,
    /// The RFC 6749 error code.
    pub code: OAuthErrorCode,
    /// Optional description. MUST be routed through the FR-MEMORY-111 PII scrubber before construction.
    pub description: Option<String>,
}

impl OAuthError {
    /// A `400 invalid_request` with the given static description.
    pub fn invalid_request(desc: &'static str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: OAuthErrorCode::InvalidRequest,
            description: Some(desc.to_string()),
        }
    }

    /// A `400 invalid_grant` with the given static description.
    pub fn invalid_grant(desc: &'static str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: OAuthErrorCode::InvalidGrant,
            description: Some(desc.to_string()),
        }
    }

    /// A `401 invalid_client` with the given static description.
    pub fn invalid_client(desc: &'static str) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code: OAuthErrorCode::InvalidClient,
            description: Some(desc.to_string()),
        }
    }

    /// A `400 invalid_scope`.
    pub fn invalid_scope() -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: OAuthErrorCode::InvalidScope,
            description: None,
        }
    }

    /// A `400 unsupported_grant_type`.
    pub fn unsupported_grant_type() -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: OAuthErrorCode::UnsupportedGrantType,
            description: None,
        }
    }

    /// A `401 unauthorized_client` with the given static description.
    pub fn unauthorized_client(desc: &'static str) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code: OAuthErrorCode::UnauthorizedClient,
            description: Some(desc.to_string()),
        }
    }
}

impl IntoResponse for OAuthError {
    fn into_response(self) -> Response {
        let body = Json(OAuthErrorBody {
            error: self.code,
            error_description: self.description,
        });
        let mut resp = (self.status, body).into_response();
        let headers = resp.headers_mut();
        headers.insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
        headers.insert(PRAGMA, HeaderValue::from_static("no-cache"));
        resp
    }
}
