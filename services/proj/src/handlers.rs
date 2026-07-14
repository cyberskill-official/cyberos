//! TASK-PROJ-001 §1 #5 — handler-layer orchestration.
//!
//! Maps HTTP-layer requests to repo operations. Each handler:
//!   1. Validates request shape.
//!   2. Performs cross-row validations (assignee tenant, cycle engagement).
//!   3. Runs the SQL mutation under the actor's RLS context.
//!   4. Builds the memory audit row (the binary wires the actual emit).

use crate::audit;
use crate::decisions::{self, DecisionPolicy, DecisionRetractionRow, DecisionRow};
use crate::errors::{IssueError, IssueResult};
use crate::memory_link::{self, LinkStrength, MemoryLinkRow, MemoryLinkType, MemoryTarget};
use crate::rate_card::{self, BillingRole, Currency, RateCardRow};
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
        repo::validate_cycle_in_engagement(db, actor.tenant_id, cycle_id, req.engagement_id)
            .await?;
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
    pub decision: Option<DecisionRow>,
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

    let mut decision = None;
    let (_, updated) = if let Some(new_status) = req.status {
        if new_status != current.status {
            let request_id = format!(
                "patch-{id}-{}",
                Utc::now().timestamp_nanos_opt().unwrap_or(0)
            );
            let (issue, row) = decisions::patch_issue_status_with_decision(
                db,
                actor,
                id,
                if_match,
                new_status,
                None,
                &request_id,
                &DecisionPolicy::default(),
                serde_json::Value::Object(Default::default()),
                None,
            )
            .await?;
            decision = Some(row);
            let rest = PatchIssueRequest {
                status: None,
                ..req.clone()
            };
            if rest.title.is_some()
                || rest.body.is_some()
                || rest.priority.is_some()
                || rest.assignee_subject_id.is_some()
                || rest.estimate_hours.is_some()
            {
                let (_, patched) = repo::patch_issue_row(db, actor, id, None, &rest).await?;
                (current.clone(), patched)
            } else {
                (current.clone(), issue)
            }
        } else {
            repo::patch_issue_row(db, actor, id, if_match, &req).await?
        }
    } else {
        repo::patch_issue_row(db, actor, id, if_match, &req).await?
    };

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

    Ok(PatchIssueResult {
        issue: updated,
        audit_rows,
        decision,
    })
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

pub struct DecisionTransitionResult {
    pub issue: Issue,
    pub decision: DecisionRow,
}

pub async fn transition_issue_with_decision(
    db: &PgPool,
    actor: Actor,
    id: Uuid,
    if_match: Option<DateTime<Utc>>,
    to_status: IssueStatus,
    reason: Option<&str>,
    request_id: &str,
    decision_attributes: serde_json::Value,
    memory_chain_hash: Option<String>,
) -> IssueResult<DecisionTransitionResult> {
    let (issue, decision) = decisions::patch_issue_status_with_decision(
        db,
        actor,
        id,
        if_match,
        to_status,
        reason,
        request_id,
        &DecisionPolicy::default(),
        decision_attributes,
        memory_chain_hash,
    )
    .await?;
    Ok(DecisionTransitionResult { issue, decision })
}

pub async fn retract_decision(
    db: &PgPool,
    actor: Actor,
    decision_id: Uuid,
    reason: &str,
    request_id: &str,
) -> IssueResult<DecisionRetractionRow> {
    decisions::retract_decision(db, actor, decision_id, reason, request_id).await
}

pub async fn list_issue_decisions(
    db: &PgPool,
    actor: Actor,
    issue_id: Uuid,
    limit: Option<i64>,
) -> IssueResult<Vec<DecisionRow>> {
    decisions::list_issue_decisions(db, actor, issue_id, limit.unwrap_or(100)).await
}

pub async fn create_rate_card(
    db: &PgPool,
    actor: Actor,
    engagement_id: Uuid,
    role: BillingRole,
    currency: Currency,
    hourly_rate_minor: i64,
    billable_default: bool,
    effective_from: chrono::NaiveDate,
) -> IssueResult<RateCardRow> {
    rate_card::create_rate_card_persisted(
        db,
        actor,
        engagement_id,
        role,
        currency,
        hourly_rate_minor,
        billable_default,
        effective_from,
    )
    .await
}

pub async fn lookup_rate_card(
    db: &PgPool,
    actor: Actor,
    engagement_id: Uuid,
    role: BillingRole,
    currency: Currency,
    at: chrono::NaiveDate,
) -> IssueResult<RateCardRow> {
    rate_card::lookup_rate_card_persisted(db, actor, engagement_id, role, currency, at).await
}

pub async fn list_rate_cards(
    db: &PgPool,
    actor: Actor,
    engagement_id: Uuid,
    include_archived: bool,
) -> IssueResult<Vec<RateCardRow>> {
    rate_card::list_rate_cards_persisted(db, actor, engagement_id, include_archived).await
}

pub async fn create_memory_link(
    db: &PgPool,
    actor: Actor,
    issue_id: Uuid,
    target: MemoryTarget,
    link_type: MemoryLinkType,
    annotation: Option<&str>,
    quoted_text: Option<&str>,
    link_strength: Option<LinkStrength>,
    review_pending: bool,
    metadata: serde_json::Value,
) -> IssueResult<MemoryLinkRow> {
    memory_link::create_link_persisted(
        db,
        actor,
        issue_id,
        target,
        link_type,
        annotation,
        quoted_text,
        link_strength,
        review_pending,
        metadata,
    )
    .await
}

pub async fn remove_memory_link(
    db: &PgPool,
    actor: Actor,
    issue_id: Uuid,
    link_id: Uuid,
    reason: &str,
) -> IssueResult<MemoryLinkRow> {
    memory_link::remove_link_persisted(db, actor, issue_id, link_id, reason).await
}

pub async fn list_outgoing_memory_links(
    db: &PgPool,
    actor: Actor,
    issue_id: Uuid,
    include_removed: bool,
) -> IssueResult<Vec<MemoryLinkRow>> {
    memory_link::list_outgoing_persisted(db, actor, issue_id, include_removed).await
}

pub async fn list_incoming_memory_links(
    db: &PgPool,
    actor: Actor,
    memory_path: &str,
    include_removed: bool,
) -> IssueResult<Vec<MemoryLinkRow>> {
    memory_link::list_incoming_persisted(db, actor, memory_path, include_removed).await
}
