//! FR-PROJ-009 — typed Issue-to-memory links.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

pub fn outgoing<'a>(issue_id: Uuid, links: &'a [MemoryLink]) -> Vec<&'a MemoryLink> {
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
            &[link.clone()],
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
