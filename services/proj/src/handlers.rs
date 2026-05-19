//! FR-PROJ-001 §1 #5 — handler-layer orchestration.
//!
//! Maps HTTP-layer requests to repo operations. Each handler:
//!   1. Validates request shape.
//!   2. Performs cross-row validations (assignee tenant, cycle engagement).
//!   3. Runs the SQL mutation under the actor's RLS context.
//!   4. Builds the memory audit row (the binary wires the actual emit).

use crate::audit;
use crate::errors::{IssueError, IssueResult};
use crate::repo;
use crate::status_fsm;
use crate::types::*;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

/// Result of a create-issue handler call. The audit row body is
/// returned for the binary to forward to memory.
pub struct CreateIssueResult {
    pub issue: Issue,
    pub audit_row: audit::ProjAuditRow,
}

pub async fn create_issue(
    db: &PgPool,
    actor: Actor,
    req: CreateIssueRequest,
) -> IssueResult<CreateIssueResult> {
    // §1 #11 — cycle-engagement membership.
    if let Some(cycle_id) = req.cycle_id {
        repo::validate_cycle_in_engagement(db, actor.tenant_id, cycle_id, req.engagement_id).await?;
    }
    // §1 #10 — assignee tenant check.
    if let Some(assignee) = req.assignee_subject_id {
        repo::validate_assignee_in_tenant(db, actor.tenant_id, assignee).await?;
    }

    // §1 #3 — initial status defaults to Triage; any explicit non-Triage
    // initial status must be Todo (which is the only direct triage→X edge).
    if let Some(s) = req.status {
        if s != IssueStatus::Triage {
            status_fsm::validate(IssueStatus::Triage, s)?;
        }
    }

    let issue = repo::insert_issue(db, actor, &req).await?;
    let audit_row = audit::issue_created(&issue, actor.subject_id);
    Ok(CreateIssueResult { issue, audit_row })
}

pub struct PatchIssueResult {
    pub issue: Issue,
    pub audit_rows: Vec<audit::ProjAuditRow>,
}

pub async fn patch_issue(
    db: &PgPool,
    actor: Actor,
    id: Uuid,
    if_match: Option<DateTime<Utc>>,
    req: PatchIssueRequest,
) -> IssueResult<PatchIssueResult> {
    // We need the current row to (a) check the FSM, (b) check assignee
    // change, (c) build accurate audit rows. The repo::patch_issue_row
    // call returns both current + updated; we run the validations between.
    let mut tx = db.begin().await?;
    repo::set_tenant(&mut tx, actor.tenant_id).await?;
    let current: Option<Issue> = sqlx::query_as("SELECT * FROM issues WHERE id = $1")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;
    let current = current.ok_or(IssueError::NotFound)?;
    drop(tx);

    // §1 #13 optimistic lock — performed in repo path, but also here as
    // a fast-fail before any other validation.
    if let Some(expected) = if_match {
        if expected != current.updated_at {
            return Err(IssueError::ConcurrentUpdate {
                expected,
                actual: current.updated_at,
            });
        }
    }

    // §1 #3 — status FSM.
    if let Some(new_status) = req.status {
        if new_status != current.status {
            status_fsm::validate(current.status, new_status)?;
        }
    }

    // §1 #10 — assignee change tenant check.
    if let Some(Some(new_assignee)) = req.assignee_subject_id {
        if Some(new_assignee) != current.assignee_subject_id {
            repo::validate_assignee_in_tenant(db, actor.tenant_id, new_assignee).await?;
        }
    }

    let (_, updated) = repo::patch_issue_row(db, actor, id, if_match, &req).await?;

    let mut audit_rows = Vec::new();
    if let Some(new_status) = req.status {
        if new_status != current.status {
            audit_rows.push(audit::issue_status_changed(
                &updated,
                current.status,
                new_status,
                actor.subject_id,
            ));
        }
    }
    if let Some(new_assignee_opt) = req.assignee_subject_id {
        if new_assignee_opt != current.assignee_subject_id {
            audit_rows.push(audit::issue_assigned(
                &updated,
                current.assignee_subject_id,
                new_assignee_opt,
                actor.subject_id,
            ));
        }
    }

    Ok(PatchIssueResult { issue: updated, audit_rows })
}

pub async fn get_issue(db: &PgPool, actor: Actor, id: Uuid) -> IssueResult<Issue> {
    repo::get_issue(db, actor, id).await
}

pub async fn list_issues(
    db: &PgPool,
    actor: Actor,
    engagement_id: Option<Uuid>,
    cycle_id: Option<Uuid>,
    assignee: Option<Uuid>,
    status: Option<IssueStatus>,
    limit: Option<i64>,
) -> IssueResult<Vec<Issue>> {
    let limit = limit.unwrap_or(50);
    repo::list_issues(db, actor, engagement_id, cycle_id, assignee, status, limit).await
}
