//! FR-MCP-004 §1.6 + DEC-820 + RFC 6749 §5.2
//!
//! Closed 6-value oauth_error_code enum; all token-endpoint responses include
//! Cache-Control: no-store and Pragma: no-cache per RFC 6749 §5.2.

use axum::{
    http::{
        header::{CACHE_CONTROL, PRAGMA},
        StatusCode,
    },
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

/// The 6 error codes permitted by RFC 6749 §5.2 + MCP profile (DEC-820).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OAuthErrorCode {
    InvalidRequest,
    InvalidClient,
    InvalidGrant,
    UnauthorizedClient,
    UnsupportedGrantType,
    InvalidScope,
}

#[derive(Debug, Serialize)]
pub struct OAuthErrorBody {
    pub error: OAuthErrorCode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_description: Option<String>,
}

#[derive(Debug)]
pub struct OAuthError {
    pub status: StatusCode,
    pub code: OAuthErrorCode,
    /// MUST be routed through FR-MEMORY-111 PII scrubber before construction.
    pub description: Option<String>,
}

impl OAuthError {
    pub fn invalid_request(desc: &'static str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: OAuthErrorCode::InvalidRequest,
            description: Some(desc.to_string()),
        }
    }

    pub fn invalid_grant(desc: &'static str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: OAuthErrorCode::InvalidGrant,
            description: Some(desc.to_string()),
        }
    }

    pub fn invalid_client(desc: &'static str) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code: OAuthErrorCode::InvalidClient,
            description: Some(desc.to_string()),
        }
    }

    pub fn invalid_scope() -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: OAuthErrorCode::InvalidScope,
            description: None,
        }
    }

    pub fn unsupported_grant_type() -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: OAuthErrorCode::UnsupportedGrantType,
            description: None,
        }
    }

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
        headers.insert(CACHE_CONTROL, "no-store".parse().unwrap());
        headers.insert(PRAGMA, "no-cache".parse().unwrap());
        resp
    }
}
