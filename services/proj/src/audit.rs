//! TASK-PROJ-001 §1 #6 — memory audit row builders.
//!
//! Four canonical kinds:
//!   - `proj.issue_created`         per POST /v1/proj/issues
//!   - `proj.issue_status_changed`  per status mutation (old, new, by_subject)
//!   - `proj.issue_assigned`        per assignee change (from, to)
//!   - `proj.issue_linked`          per link added (source, target, link_type)

use crate::types::{Issue, IssuePriority, IssueStatus, LinkType};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct ProjAuditRow {
    pub kind: &'static str,
    pub tenant_id: Uuid,
    pub issue_id: Option<Uuid>,
    pub engagement_id: Option<Uuid>,
    pub cycle_id: Option<Uuid>,
    pub priority: Option<&'static str>,
    pub by_subject_id: Option<Uuid>,
    pub from_status: Option<&'static str>,
    pub to_status: Option<&'static str>,
    pub from_subject_id: Option<Uuid>,
    pub to_subject_id: Option<Uuid>,
    pub linked_to_id: Option<Uuid>,
    pub link_type: Option<&'static str>,
    pub ts_ns: i128,
}

pub fn now_ns() -> i128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as i128)
        .unwrap_or(0)
}

const fn priority_str(p: IssuePriority) -> &'static str {
    match p {
        IssuePriority::Urgent => "urgent",
        IssuePriority::High => "high",
        IssuePriority::Normal => "normal",
        IssuePriority::Low => "low",
    }
}

const fn status_str(s: IssueStatus) -> &'static str {
    match s {
        IssueStatus::Triage => "triage",
        IssueStatus::Todo => "todo",
        IssueStatus::Doing => "doing",
        IssueStatus::Review => "review",
        IssueStatus::Done => "done",
        IssueStatus::Deleted => "deleted",
    }
}

pub fn issue_created(issue: &Issue, by: Uuid) -> ProjAuditRow {
    ProjAuditRow {
        kind: "proj.issue_created",
        tenant_id: issue.tenant_id,
        issue_id: Some(issue.id),
        engagement_id: Some(issue.engagement_id),
        cycle_id: issue.cycle_id,
        priority: Some(priority_str(issue.priority)),
        by_subject_id: Some(by),
        from_status: None,
        to_status: Some(status_str(issue.status)),
        from_subject_id: None,
        to_subject_id: issue.assignee_subject_id,
        linked_to_id: None,
        link_type: None,
        ts_ns: now_ns(),
    }
}

pub fn issue_status_changed(
    issue: &Issue,
    from: IssueStatus,
    to: IssueStatus,
    by: Uuid,
) -> ProjAuditRow {
    ProjAuditRow {
        kind: "proj.issue_status_changed",
        tenant_id: issue.tenant_id,
        issue_id: Some(issue.id),
        engagement_id: Some(issue.engagement_id),
        cycle_id: issue.cycle_id,
        priority: Some(priority_str(issue.priority)),
        by_subject_id: Some(by),
        from_status: Some(status_str(from)),
        to_status: Some(status_str(to)),
        from_subject_id: None,
        to_subject_id: None,
        linked_to_id: None,
        link_type: None,
        ts_ns: now_ns(),
    }
}

pub fn issue_assigned(
    issue: &Issue,
    from: Option<Uuid>,
    to: Option<Uuid>,
    by: Uuid,
) -> ProjAuditRow {
    ProjAuditRow {
        kind: "proj.issue_assigned",
        tenant_id: issue.tenant_id,
        issue_id: Some(issue.id),
        engagement_id: Some(issue.engagement_id),
        cycle_id: issue.cycle_id,
        priority: Some(priority_str(issue.priority)),
        by_subject_id: Some(by),
        from_status: None,
        to_status: None,
        from_subject_id: from,
        to_subject_id: to,
        linked_to_id: None,
        link_type: None,
        ts_ns: now_ns(),
    }
}

pub fn issue_linked(
    tenant_id: Uuid,
    issue_id: Uuid,
    linked_to_id: Uuid,
    link_type: LinkType,
    by: Uuid,
) -> ProjAuditRow {
    ProjAuditRow {
        kind: "proj.issue_linked",
        tenant_id,
        issue_id: Some(issue_id),
        engagement_id: None,
        cycle_id: None,
        priority: None,
        by_subject_id: Some(by),
        from_status: None,
        to_status: None,
        from_subject_id: None,
        to_subject_id: None,
        linked_to_id: Some(linked_to_id),
        link_type: Some(link_type.as_str()),
        ts_ns: now_ns(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn fake_issue() -> Issue {
        Issue {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            engagement_id: Uuid::new_v4(),
            cycle_id: None,
            title: "x".into(),
            body: None,
            status: IssueStatus::Triage,
            priority: IssuePriority::Normal,
            assignee_subject_id: None,
            estimate_hours: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn created_row_carries_engagement_and_priority() {
        let i = fake_issue();
        let by = Uuid::new_v4();
        let row = issue_created(&i, by);
        assert_eq!(row.kind, "proj.issue_created");
        assert_eq!(row.tenant_id, i.tenant_id);
        assert_eq!(row.engagement_id, Some(i.engagement_id));
        assert_eq!(row.priority, Some("normal"));
        assert_eq!(row.by_subject_id, Some(by));
    }

    #[test]
    fn status_change_carries_from_and_to() {
        let i = fake_issue();
        let row = issue_status_changed(&i, IssueStatus::Triage, IssueStatus::Todo, Uuid::new_v4());
        assert_eq!(row.kind, "proj.issue_status_changed");
        assert_eq!(row.from_status, Some("triage"));
        assert_eq!(row.to_status, Some("todo"));
    }

    #[test]
    fn assignment_carries_from_and_to_subject() {
        let i = fake_issue();
        let from = Uuid::new_v4();
        let to = Uuid::new_v4();
        let row = issue_assigned(&i, Some(from), Some(to), Uuid::new_v4());
        assert_eq!(row.kind, "proj.issue_assigned");
        assert_eq!(row.from_subject_id, Some(from));
        assert_eq!(row.to_subject_id, Some(to));
    }

    #[test]
    fn link_row_carries_link_type() {
        let row = issue_linked(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            LinkType::Blocks,
            Uuid::new_v4(),
        );
        assert_eq!(row.kind, "proj.issue_linked");
        assert_eq!(row.link_type, Some("blocks"));
    }
}
