//! Service error types.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use thiserror::Error;

/// Compliance view error.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ViewError {
    /// JWT missing or invalid.
    #[error("auth_failed")]
    AuthFailed,
    /// JWT lacks the auditor role.
    #[error("forbidden")]
    Forbidden,
    /// Query range exceeds 365 days.
    #[error("time_range_too_large")]
    TimeRangeTooLarge,
    /// Caller tried to pass tenant_id in query params.
    #[error("tenant_id_in_query")]
    TenantIdInQuery,
    /// Query exceeded the service budget.
    #[error("query_timeout")]
    QueryTimeout,
    /// Raw PII was detected.
    #[error("pii_leak_attempted")]
    PiiLeakAttempt,
    /// Memory query failed.
    #[error("memory_query_failed: {0}")]
    MemoryQueryFailed(String),
    /// Chain proof failed.
    #[error("chain_proof_failed: {0}")]
    ChainProofFailed(String),
    /// Export failed.
    #[error("export_failed: {0}")]
    ExportFailed(String),
}

impl ViewError {
    /// Stable error code.
    pub fn code(&self) -> &'static str {
        match self {
            Self::AuthFailed => "auth_failed",
            Self::Forbidden => "forbidden",
            Self::TimeRangeTooLarge => "time_range_too_large",
            Self::TenantIdInQuery => "tenant_id_in_query",
            Self::QueryTimeout => "query_timeout",
            Self::PiiLeakAttempt => "pii_leak_attempted",
            Self::MemoryQueryFailed(_) => "memory_query_failed",
            Self::ChainProofFailed(_) => "chain_proof_failed",
            Self::ExportFailed(_) => "export_failed",
        }
    }

    /// HTTP status.
    pub fn status(&self) -> StatusCode {
        match self {
            Self::AuthFailed => StatusCode::UNAUTHORIZED,
            Self::Forbidden | Self::TenantIdInQuery => StatusCode::FORBIDDEN,
            Self::TimeRangeTooLarge => StatusCode::BAD_REQUEST,
            Self::QueryTimeout => StatusCode::TOO_MANY_REQUESTS,
            Self::PiiLeakAttempt
            | Self::MemoryQueryFailed(_)
            | Self::ChainProofFailed(_)
            | Self::ExportFailed(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: &'static str,
    reason: String,
}

impl IntoResponse for ViewError {
    fn into_response(self) -> Response {
        let status = self.status();
        let body = ErrorBody {
            error: self.code(),
            reason: self.to_string(),
        };
        (status, axum::Json(body)).into_response()
    }
}
