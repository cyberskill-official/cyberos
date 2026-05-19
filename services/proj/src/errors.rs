//! FR-PROJ-001 — structured error type.

use crate::types::IssueStatus;
use chrono::{DateTime, Utc};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum IssueError {
    #[error("issue not found")]
    NotFound,

    #[error("illegal status transition: {from:?} → {to:?}; allowed = {allowed:?}")]
    IllegalStatusTransition {
        from: IssueStatus,
        to: IssueStatus,
        allowed: Vec<IssueStatus>,
    },

    #[error("assignee {subject_id} is not a member of tenant {tenant_id}")]
    AssigneeCrossTenant { tenant_id: Uuid, subject_id: Uuid },

    #[error("cycle {cycle_id} belongs to a different engagement than the target issue")]
    CycleEngagementMismatch { cycle_id: Uuid },

    #[error("concurrent update: expected updated_at = {expected}, got {actual}")]
    ConcurrentUpdate {
        expected: DateTime<Utc>,
        actual: DateTime<Utc>,
    },

    #[error("invalid link type: {0}")]
    InvalidLinkType(String),

    #[error("self-link forbidden: issue cannot link to itself")]
    SelfLink,

    #[error("validation error: {0}")]
    Validation(String),

    #[error("only root-admin may soft-delete issues")]
    SoftDeleteForbidden,

    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),
}

impl IssueError {
    /// Stable kebab-case code for OTel attributes + JSON `error` field.
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound => "not_found",
            Self::IllegalStatusTransition { .. } => "illegal_status_transition",
            Self::AssigneeCrossTenant { .. } => "assignee_cross_tenant",
            Self::CycleEngagementMismatch { .. } => "cycle_engagement_mismatch",
            Self::ConcurrentUpdate { .. } => "concurrent_update",
            Self::InvalidLinkType(_) => "invalid_link_type",
            Self::SelfLink => "self_link_forbidden",
            Self::Validation(_) => "validation",
            Self::SoftDeleteForbidden => "soft_delete_forbidden",
            Self::Db(_) => "db_error",
        }
    }

    /// HTTP status mapping for the handler layer.
    pub fn http_status(&self) -> u16 {
        match self {
            Self::NotFound => 404,
            Self::ConcurrentUpdate { .. } => 412,
            Self::Db(_) => 500,
            _ => 400,
        }
    }
}

pub type IssueResult<T> = Result<T, IssueError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_status_412_for_optimistic_lock() {
        let now = Utc::now();
        let later = now + chrono::Duration::seconds(1);
        let err = IssueError::ConcurrentUpdate {
            expected: now,
            actual: later,
        };
        assert_eq!(err.http_status(), 412);
        assert_eq!(err.code(), "concurrent_update");
    }

    #[test]
    fn illegal_transition_400() {
        let err = IssueError::IllegalStatusTransition {
            from: IssueStatus::Triage,
            to: IssueStatus::Done,
            allowed: vec![IssueStatus::Todo],
        };
        assert_eq!(err.http_status(), 400);
    }

    #[test]
    fn not_found_404() {
        assert_eq!(IssueError::NotFound.http_status(), 404);
    }
}
