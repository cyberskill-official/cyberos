//! FR-PROJ-001 §4 #3 + §4 #6 + §4 #7 + §4 #14 — error → HTTP status mapping.

use chrono::Utc;
use cyberos_proj::types::IssueStatus;
use cyberos_proj::IssueError;
use uuid::Uuid;

#[test]
fn illegal_status_transition_400() {
    let err = IssueError::IllegalStatusTransition {
        from: IssueStatus::Triage,
        to: IssueStatus::Done,
        allowed: vec![IssueStatus::Todo],
    };
    assert_eq!(err.http_status(), 400);
    assert_eq!(err.code(), "illegal_status_transition");
}

#[test]
fn assignee_cross_tenant_400() {
    let err = IssueError::AssigneeCrossTenant {
        tenant_id: Uuid::new_v4(),
        subject_id: Uuid::new_v4(),
    };
    assert_eq!(err.http_status(), 400);
    assert_eq!(err.code(), "assignee_cross_tenant");
}

#[test]
fn cycle_engagement_mismatch_400() {
    let err = IssueError::CycleEngagementMismatch {
        cycle_id: Uuid::new_v4(),
    };
    assert_eq!(err.http_status(), 400);
    assert_eq!(err.code(), "cycle_engagement_mismatch");
}

#[test]
fn concurrent_update_412() {
    let now = Utc::now();
    let err = IssueError::ConcurrentUpdate {
        expected: now,
        actual: now + chrono::Duration::seconds(1),
    };
    assert_eq!(err.http_status(), 412);
    assert_eq!(err.code(), "concurrent_update");
}

#[test]
fn not_found_404() {
    assert_eq!(IssueError::NotFound.http_status(), 404);
    assert_eq!(IssueError::NotFound.code(), "not_found");
}

#[test]
fn self_link_400() {
    assert_eq!(IssueError::SelfLink.http_status(), 400);
    assert_eq!(IssueError::SelfLink.code(), "self_link_forbidden");
}
