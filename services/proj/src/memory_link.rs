//! TASK-PROJ-009 — typed Issue-to-memory links.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::errors::{IssueError, IssueResult};
use crate::repo;
use crate::types::{Actor, Issue};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryLinkType {
    Cites,
    Implements,
    Supersedes,
    CitesWithQuote,
}

impl MemoryLinkType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cites => "cites",
            Self::Implements => "implements",
            Self::Supersedes => "supersedes",
            Self::CitesWithQuote => "cites_with_quote",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkStrength {
    Weak,
    Medium,
    Strong,
}

impl LinkStrength {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Weak => "weak",
            Self::Medium => "medium",
            Self::Strong => "strong",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryTarget {
    pub path: String,
    pub tenant_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub readable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryLink {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub issue_id: Uuid,
    pub issue_created_at: DateTime<Utc>,
    pub memory_path: String,
    pub memory_row_id: Option<String>,
    pub link_type: MemoryLinkType,
    pub annotation: Option<String>,
    pub link_strength: LinkStrength,
    pub removed_at: Option<DateTime<Utc>>,
    pub removal_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum MemoryLinkError {
    #[error("target memory does not exist: {0}")]
    TargetMissing(String),
    #[error("scope denied for path: {0}")]
    ScopeDenied(String),
    #[error("cross-tenant memory link forbidden")]
    CrossTenantForbidden,
    #[error("supersedes target must be older than issue")]
    SupersedeViolatesTime,
    #[error("duplicate active link")]
    DuplicateActive,
    #[error("removal_reason required")]
    RemovalReasonRequired,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct MemoryLinkRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub issue_id: Uuid,
    pub memory_path: String,
    pub memory_row_id: Option<String>,
    pub link_type: String,
    pub annotation: Option<String>,
    pub quoted_text: Option<String>,
    pub link_strength: String,
    pub review_pending: bool,
    pub metadata: serde_json::Value,
    pub created_by_subject_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub removed_at: Option<DateTime<Utc>>,
    pub removed_by_subject_id: Option<Uuid>,
    pub removal_reason: Option<String>,
}

pub fn create_link(
    tenant_id: Uuid,
    issue_id: Uuid,
    issue_created_at: DateTime<Utc>,
    target: Option<MemoryTarget>,
    link_type: MemoryLinkType,
    existing_links: &[MemoryLink],
    annotation: Option<&str>,
    link_strength: Option<LinkStrength>,
) -> Result<MemoryLink, MemoryLinkError> {
    let Some(target) = target else {
        return Err(MemoryLinkError::TargetMissing("<missing>".into()));
    };
    if !target.readable {
        return Err(MemoryLinkError::ScopeDenied(target.path));
    }
    if target.tenant_id != tenant_id {
        return Err(MemoryLinkError::CrossTenantForbidden);
    }
    if link_type == MemoryLinkType::Supersedes && target.created_at >= issue_created_at {
        return Err(MemoryLinkError::SupersedeViolatesTime);
    }
    if existing_links.iter().any(|link| {
        link.removed_at.is_none()
            && link.issue_id == issue_id
            && link.memory_path == target.path
            && link.link_type == link_type
    }) {
        return Err(MemoryLinkError::DuplicateActive);
    }
    Ok(MemoryLink {
        id: Uuid::new_v4(),
        tenant_id,
        issue_id,
        issue_created_at,
        memory_path: target.path,
        memory_row_id: None,
        link_type,
        annotation: annotation.map(redact_annotation),
        link_strength: link_strength.unwrap_or(LinkStrength::Medium),
        removed_at: None,
        removal_reason: None,
    })
}

pub async fn create_link_persisted(
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
    if !target.readable {
        return Err(IssueError::Validation(
            MemoryLinkError::ScopeDenied(target.path).to_string(),
        ));
    }
    if target.tenant_id != actor.tenant_id {
        return Err(IssueError::Validation(
            MemoryLinkError::CrossTenantForbidden.to_string(),
        ));
    }
    let mut tx = db.begin().await?;
    repo::set_tenant(&mut tx, actor.tenant_id).await?;
    let issue: Option<Issue> = sqlx::query_as("SELECT * FROM issues WHERE id = $1")
        .bind(issue_id)
        .fetch_optional(&mut *tx)
        .await?;
    let issue = issue.ok_or(IssueError::NotFound)?;
    if link_type == MemoryLinkType::Supersedes && target.created_at >= issue.created_at {
        return Err(IssueError::Validation(
            MemoryLinkError::SupersedeViolatesTime.to_string(),
        ));
    }

    let row: MemoryLinkRow = sqlx::query_as(
        "INSERT INTO memory_links (
            tenant_id, issue_id, memory_path, memory_row_id, link_type,
            annotation, quoted_text, link_strength, review_pending, metadata,
            created_by_subject_id
         )
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
         RETURNING *",
    )
    .bind(actor.tenant_id)
    .bind(issue_id)
    .bind(&target.path)
    .bind(Option::<String>::None)
    .bind(link_type.as_str())
    .bind(annotation.map(redact_annotation))
    .bind(quoted_text)
    .bind(link_strength.unwrap_or(LinkStrength::Medium).as_str())
    .bind(review_pending)
    .bind(metadata)
    .bind(actor.subject_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|err| {
        if let sqlx::Error::Database(db_err) = &err {
            if db_err.constraint() == Some("memory_links_active_unique") {
                return IssueError::Validation(MemoryLinkError::DuplicateActive.to_string());
            }
        }
        IssueError::Db(err)
    })?;
    tx.commit().await?;
    Ok(row)
}

pub async fn remove_link_persisted(
    db: &PgPool,
    actor: Actor,
    issue_id: Uuid,
    link_id: Uuid,
    reason: &str,
) -> IssueResult<MemoryLinkRow> {
    if reason.trim().is_empty() {
        return Err(IssueError::Validation(
            MemoryLinkError::RemovalReasonRequired.to_string(),
        ));
    }
    let mut tx = db.begin().await?;
    repo::set_tenant(&mut tx, actor.tenant_id).await?;
    let row: MemoryLinkRow = sqlx::query_as(
        "UPDATE memory_links
         SET removed_at = now(),
             removed_by_subject_id = $4,
             removal_reason = $5
         WHERE tenant_id = $1
           AND issue_id = $2
           AND id = $3
           AND removed_at IS NULL
         RETURNING *",
    )
    .bind(actor.tenant_id)
    .bind(issue_id)
    .bind(link_id)
    .bind(actor.subject_id)
    .bind(redact_annotation(reason))
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(IssueError::NotFound)?;
    tx.commit().await?;
    Ok(row)
}

pub async fn list_outgoing_persisted(
    db: &PgPool,
    actor: Actor,
    issue_id: Uuid,
    include_removed: bool,
) -> IssueResult<Vec<MemoryLinkRow>> {
    let mut tx = db.begin().await?;
    repo::set_tenant(&mut tx, actor.tenant_id).await?;
    let rows: Vec<MemoryLinkRow> = sqlx::query_as(
        "SELECT * FROM memory_links
         WHERE tenant_id = $1
           AND issue_id = $2
           AND ($3::bool = true OR removed_at IS NULL)
         ORDER BY created_at DESC",
    )
    .bind(actor.tenant_id)
    .bind(issue_id)
    .bind(include_removed)
    .fetch_all(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(rows)
}

pub async fn list_incoming_persisted(
    db: &PgPool,
    actor: Actor,
    memory_path: &str,
    include_removed: bool,
) -> IssueResult<Vec<MemoryLinkRow>> {
    let mut tx = db.begin().await?;
    repo::set_tenant(&mut tx, actor.tenant_id).await?;
    let rows: Vec<MemoryLinkRow> = sqlx::query_as(
        "SELECT * FROM memory_links
         WHERE tenant_id = $1
           AND memory_path = $2
           AND ($3::bool = true OR removed_at IS NULL)
         ORDER BY created_at DESC",
    )
    .bind(actor.tenant_id)
    .bind(memory_path)
    .bind(include_removed)
    .fetch_all(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(rows)
}

pub fn soft_remove(
    link: &mut MemoryLink,
    reason: &str,
    now: DateTime<Utc>,
) -> Result<(), MemoryLinkError> {
    if reason.trim().is_empty() {
        return Err(MemoryLinkError::RemovalReasonRequired);
    }
    link.removed_at = Some(now);
    link.removal_reason = Some(redact_annotation(reason));
    Ok(())
}

pub fn outgoing(issue_id: Uuid, links: &[MemoryLink]) -> Vec<&MemoryLink> {
    links
        .iter()
        .filter(|l| l.issue_id == issue_id && l.removed_at.is_none())
        .collect()
}

pub fn incoming<'a>(memory_path: &str, links: &'a [MemoryLink]) -> Vec<&'a MemoryLink> {
    links
        .iter()
        .filter(|l| l.memory_path == memory_path && l.removed_at.is_none())
        .collect()
}

fn redact_annotation(input: &str) -> String {
    input
        .split_whitespace()
        .map(|part| {
            if part.contains('@') && part.contains('.') {
                "[redacted-email]"
            } else {
                part
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[derive(Debug, Clone, Serialize)]
pub struct MemoryLinkAuditRow {
    pub kind: &'static str,
    pub tenant_id: Uuid,
    pub issue_id: Uuid,
    pub memory_path: String,
    pub link_type: &'static str,
}

pub fn audit_created(link: &MemoryLink) -> MemoryLinkAuditRow {
    MemoryLinkAuditRow {
        kind: "proj.memory_link_created",
        tenant_id: link.tenant_id,
        issue_id: link.issue_id,
        memory_path: link.memory_path.clone(),
        link_type: link.link_type.as_str(),
    }
}

pub fn audit_removed(link: &MemoryLink) -> MemoryLinkAuditRow {
    MemoryLinkAuditRow {
        kind: "proj.memory_link_removed",
        ..audit_created(link)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn target(tenant: Uuid, created_at: DateTime<Utc>) -> MemoryTarget {
        MemoryTarget {
            path: "memories/decisions/aa/bb/decision.md".into(),
            tenant_id: tenant,
            created_at,
            readable: true,
        }
    }

    #[test]
    fn creates_bidirectionally_queryable_link() {
        let tenant = Uuid::new_v4();
        let issue = Uuid::new_v4();
        let issue_created = Utc::now();
        let link = create_link(
            tenant,
            issue,
            issue_created,
            Some(target(tenant, issue_created - Duration::days(1))),
            MemoryLinkType::Implements,
            &[],
            Some("see alice@example.com"),
            Some(LinkStrength::Strong),
        )
        .unwrap();
        let links = vec![link.clone()];
        assert_eq!(outgoing(issue, &links).len(), 1);
        assert_eq!(incoming(&link.memory_path, &links).len(), 1);
        assert_eq!(link.annotation.as_deref(), Some("see [redacted-email]"));
        assert_eq!(link.link_strength, LinkStrength::Strong);
    }

    #[test]
    fn dangling_scope_cross_tenant_and_future_supersede_rejected() {
        let tenant = Uuid::new_v4();
        let issue_created = Utc::now();
        let err = create_link(
            tenant,
            Uuid::new_v4(),
            issue_created,
            None,
            MemoryLinkType::Cites,
            &[],
            None,
            None,
        )
        .unwrap_err();
        assert!(matches!(err, MemoryLinkError::TargetMissing(_)));

        let mut unreadable = target(tenant, issue_created);
        unreadable.readable = false;
        let err = create_link(
            tenant,
            Uuid::new_v4(),
            issue_created,
            Some(unreadable),
            MemoryLinkType::Cites,
            &[],
            None,
            None,
        )
        .unwrap_err();
        assert!(matches!(err, MemoryLinkError::ScopeDenied(_)));

        let err = create_link(
            tenant,
            Uuid::new_v4(),
            issue_created,
            Some(target(Uuid::new_v4(), issue_created)),
            MemoryLinkType::Cites,
            &[],
            None,
            None,
        )
        .unwrap_err();
        assert_eq!(err, MemoryLinkError::CrossTenantForbidden);

        let err = create_link(
            tenant,
            Uuid::new_v4(),
            issue_created,
            Some(target(tenant, issue_created + Duration::seconds(1))),
            MemoryLinkType::Supersedes,
            &[],
            None,
            None,
        )
        .unwrap_err();
        assert_eq!(err, MemoryLinkError::SupersedeViolatesTime);
    }

    #[test]
    fn duplicate_active_and_empty_remove_reason_rejected() {
        let tenant = Uuid::new_v4();
        let issue = Uuid::new_v4();
        let issue_created = Utc::now();
        let link = create_link(
            tenant,
            issue,
            issue_created,
            Some(target(tenant, issue_created)),
            MemoryLinkType::Cites,
            &[],
            None,
            None,
        )
        .unwrap();
        let err = create_link(
            tenant,
            issue,
            issue_created,
            Some(target(tenant, issue_created)),
            MemoryLinkType::Cites,
            std::slice::from_ref(&link),
            None,
            None,
        )
        .unwrap_err();
        assert_eq!(err, MemoryLinkError::DuplicateActive);

        let mut link = link;
        let err = soft_remove(&mut link, " ", Utc::now()).unwrap_err();
        assert_eq!(err, MemoryLinkError::RemovalReasonRequired);
        soft_remove(&mut link, "obsolete", Utc::now()).unwrap();
        assert!(link.removed_at.is_some());
    }
}
