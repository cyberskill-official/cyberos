//! FR-PROJ-001 — sqlx repository layer.

use crate::errors::{IssueError, IssueResult};
use crate::types::*;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

/// Set the per-request RLS tenant GUC. Must be called inside the
/// connection-bound transaction so SET LOCAL scopes to that tx.
pub async fn set_tenant(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: Uuid,
) -> IssueResult<()> {
    sqlx::query("SET LOCAL app.current_tenant_id = $1")
        .bind(tenant_id.to_string())
        .execute(&mut **tx)
        .await?;
    Ok(())
}

/// §1 #10 — assignee MUST be a subject in the same tenant. We check via
/// the `subjects` table from FR-AUTH-002 / FR-AUTH-003. Since `subjects`
/// is RLS-scoped, querying with the current_tenant_id GUC set means a
/// cross-tenant subject returns 0 rows.
///
/// The query is run under the actor's RLS context (already SET LOCAL via
/// the caller's tx). A successful lookup implies same-tenant.
pub async fn validate_assignee_in_tenant(
    db: &PgPool,
    tenant_id: Uuid,
    subject_id: Uuid,
) -> IssueResult<()> {
    let mut tx = db.begin().await?;
    set_tenant(&mut tx, tenant_id).await?;
    let exists: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM subjects WHERE id = $1")
        .bind(subject_id)
        .fetch_optional(&mut *tx)
        .await?;
    tx.commit().await?;
    if exists.is_none() {
        return Err(IssueError::AssigneeCrossTenant {
            tenant_id,
            subject_id,
        });
    }
    Ok(())
}

/// §1 #11 — cycle MUST belong to the same engagement.
pub async fn validate_cycle_in_engagement(
    db: &PgPool,
    tenant_id: Uuid,
    cycle_id: Uuid,
    engagement_id: Uuid,
) -> IssueResult<()> {
    let mut tx = db.begin().await?;
    set_tenant(&mut tx, tenant_id).await?;
    let row: Option<(Uuid,)> = sqlx::query_as("SELECT engagement_id FROM cycles WHERE id = $1")
        .bind(cycle_id)
        .fetch_optional(&mut *tx)
        .await?;
    tx.commit().await?;
    let (cycle_eng,) = row.ok_or(IssueError::CycleEngagementMismatch { cycle_id })?;
    if cycle_eng != engagement_id {
        return Err(IssueError::CycleEngagementMismatch { cycle_id });
    }
    Ok(())
}

pub async fn insert_issue(
    db: &PgPool,
    actor: Actor,
    req: &CreateIssueRequest,
) -> IssueResult<Issue> {
    let mut tx = db.begin().await?;
    set_tenant(&mut tx, actor.tenant_id).await?;
    let status = req.status.unwrap_or(IssueStatus::Triage);
    let priority = req.priority.unwrap_or(IssuePriority::Normal);
    let row: Issue = sqlx::query_as(
        "INSERT INTO issues (tenant_id, engagement_id, cycle_id, title, body,
                             status, priority, assignee_subject_id, estimate_hours)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
         RETURNING *",
    )
    .bind(actor.tenant_id)
    .bind(req.engagement_id)
    .bind(req.cycle_id)
    .bind(&req.title)
    .bind(req.body.as_deref())
    .bind(status.as_str())
    .bind(match priority {
        IssuePriority::Urgent => "urgent",
        IssuePriority::High => "high",
        IssuePriority::Normal => "normal",
        IssuePriority::Low => "low",
    })
    .bind(req.assignee_subject_id)
    .bind(req.estimate_hours)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(row)
}

pub async fn get_issue(db: &PgPool, actor: Actor, id: Uuid) -> IssueResult<Issue> {
    let mut tx = db.begin().await?;
    set_tenant(&mut tx, actor.tenant_id).await?;
    let row: Option<Issue> = sqlx::query_as("SELECT * FROM issues WHERE id = $1")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;
    tx.commit().await?;
    row.ok_or(IssueError::NotFound)
}

pub async fn patch_issue_row(
    db: &PgPool,
    actor: Actor,
    id: Uuid,
    if_match: Option<DateTime<Utc>>,
    req: &PatchIssueRequest,
) -> IssueResult<(Issue, Issue)> {
    let mut tx = db.begin().await?;
    set_tenant(&mut tx, actor.tenant_id).await?;
    let current: Option<Issue> = sqlx::query_as("SELECT * FROM issues WHERE id = $1")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;
    let current = current.ok_or(IssueError::NotFound)?;

    // §1 #13 optimistic lock.
    if let Some(expected) = if_match {
        if expected != current.updated_at {
            return Err(IssueError::ConcurrentUpdate {
                expected,
                actual: current.updated_at,
            });
        }
    }

    let new_status_str = req.status.map(|s| s.as_str());
    let new_priority_str = req.priority.map(|p| match p {
        IssuePriority::Urgent => "urgent",
        IssuePriority::High => "high",
        IssuePriority::Normal => "normal",
        IssuePriority::Low => "low",
    });

    // Assignee uses double-Option semantics. `None` = leave unchanged,
    // `Some(None)` = explicit clear, `Some(Some(uuid))` = new assignee.
    let assignee_change: Option<Option<Uuid>> = req.assignee_subject_id;
    let estimate_change: Option<Option<f64>> = req.estimate_hours;

    let updated: Issue = sqlx::query_as(
        "UPDATE issues SET
            title = COALESCE($1, title),
            body  = CASE WHEN $2::bool THEN $3 ELSE body END,
            status = COALESCE($4, status),
            priority = COALESCE($5, priority),
            assignee_subject_id = CASE WHEN $6::bool THEN $7 ELSE assignee_subject_id END,
            estimate_hours = CASE WHEN $8::bool THEN $9 ELSE estimate_hours END
         WHERE id = $10
         RETURNING *",
    )
    .bind(req.title.as_deref())
    .bind(req.body.is_some())
    .bind(req.body.as_deref())
    .bind(new_status_str)
    .bind(new_priority_str)
    .bind(assignee_change.is_some())
    .bind(assignee_change.flatten())
    .bind(estimate_change.is_some())
    .bind(estimate_change.flatten())
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok((current, updated))
}

/// List issues with the FR §1 #5 filter set.
pub async fn list_issues(
    db: &PgPool,
    actor: Actor,
    engagement_id: Option<Uuid>,
    cycle_id: Option<Uuid>,
    assignee: Option<Uuid>,
    status: Option<IssueStatus>,
    limit: i64,
) -> IssueResult<Vec<Issue>> {
    let mut tx = db.begin().await?;
    set_tenant(&mut tx, actor.tenant_id).await?;
    let rows: Vec<Issue> = sqlx::query_as(
        "SELECT * FROM issues
         WHERE ($1::uuid IS NULL OR engagement_id = $1)
           AND ($2::uuid IS NULL OR cycle_id = $2)
           AND ($3::uuid IS NULL OR assignee_subject_id = $3)
           AND ($4::text IS NULL OR status = $4)
         ORDER BY created_at DESC
         LIMIT $5",
    )
    .bind(engagement_id)
    .bind(cycle_id)
    .bind(assignee)
    .bind(status.map(|s| s.as_str()))
    .bind(limit.clamp(1, 1000))
    .fetch_all(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(rows)
}

pub async fn insert_engagement(
    db: &PgPool,
    actor: Actor,
    name: &str,
    started_at: chrono::NaiveDate,
) -> IssueResult<Engagement> {
    let mut tx = db.begin().await?;
    set_tenant(&mut tx, actor.tenant_id).await?;
    let row: Engagement = sqlx::query_as(
        "INSERT INTO engagements (tenant_id, name, started_at)
         VALUES ($1, $2, $3)
         RETURNING *",
    )
    .bind(actor.tenant_id)
    .bind(name)
    .bind(started_at)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(row)
}

pub async fn insert_cycle(
    db: &PgPool,
    actor: Actor,
    engagement_id: Uuid,
    name: &str,
    starts_at: chrono::NaiveDate,
    ends_at: chrono::NaiveDate,
) -> IssueResult<Cycle> {
    if ends_at <= starts_at {
        // The SQL CHECK catches this too, but surfacing the constraint
        // at the API layer gives a structured error code.
        return Err(IssueError::Validation(format!(
            "cycle ends_at ({ends_at}) must be > starts_at ({starts_at})"
        )));
    }
    let mut tx = db.begin().await?;
    set_tenant(&mut tx, actor.tenant_id).await?;
    let row: Cycle = sqlx::query_as(
        "INSERT INTO cycles (tenant_id, engagement_id, name, starts_at, ends_at)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING *",
    )
    .bind(actor.tenant_id)
    .bind(engagement_id)
    .bind(name)
    .bind(starts_at)
    .bind(ends_at)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(row)
}
