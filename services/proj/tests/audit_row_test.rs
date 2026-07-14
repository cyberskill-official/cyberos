//! TASK-PROJ-001 §4 #1 + §4 #5 — memory audit row builders.

use chrono::Utc;
use cyberos_proj::audit::{issue_assigned, issue_created, issue_linked, issue_status_changed};
use cyberos_proj::types::{Issue, IssuePriority, IssueStatus, LinkType};
use uuid::Uuid;

fn fake_issue(status: IssueStatus, priority: IssuePriority, assignee: Option<Uuid>) -> Issue {
    Issue {
        id: Uuid::new_v4(),
        tenant_id: Uuid::new_v4(),
        engagement_id: Uuid::new_v4(),
        cycle_id: None,
        title: "Issue".into(),
        body: None,
        status,
        priority,
        assignee_subject_id: assignee,
        estimate_hours: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

#[test]
fn created_row_carries_initial_status_in_to_status() {
    let i = fake_issue(IssueStatus::Triage, IssuePriority::High, None);
    let by = Uuid::new_v4();
    let row = issue_created(&i, by);
    assert_eq!(row.kind, "proj.issue_created");
    assert_eq!(row.to_status, Some("triage"));
    assert_eq!(row.priority, Some("high"));
    assert_eq!(row.engagement_id, Some(i.engagement_id));
}

#[test]
fn status_changed_row_records_both_from_and_to() {
    let i = fake_issue(IssueStatus::Todo, IssuePriority::Normal, None);
    let row = issue_status_changed(&i, IssueStatus::Triage, IssueStatus::Todo, Uuid::new_v4());
    assert_eq!(row.kind, "proj.issue_status_changed");
    assert_eq!(row.from_status, Some("triage"));
    assert_eq!(row.to_status, Some("todo"));
}

#[test]
fn assigned_row_records_both_from_and_to_subjects() {
    let from = Uuid::new_v4();
    let to = Uuid::new_v4();
    let i = fake_issue(IssueStatus::Todo, IssuePriority::Normal, Some(to));
    let row = issue_assigned(&i, Some(from), Some(to), Uuid::new_v4());
    assert_eq!(row.kind, "proj.issue_assigned");
    assert_eq!(row.from_subject_id, Some(from));
    assert_eq!(row.to_subject_id, Some(to));
}

#[test]
fn assigned_row_records_clear_assignee() {
    let from = Uuid::new_v4();
    let i = fake_issue(IssueStatus::Todo, IssuePriority::Normal, None);
    let row = issue_assigned(&i, Some(from), None, Uuid::new_v4());
    assert_eq!(row.from_subject_id, Some(from));
    assert_eq!(row.to_subject_id, None);
}

#[test]
fn linked_row_carries_link_type_string() {
    let row = issue_linked(
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        LinkType::DerivedFromEmailThread,
        Uuid::new_v4(),
    );
    assert_eq!(row.kind, "proj.issue_linked");
    assert_eq!(row.link_type, Some("derived_from_email_thread"));
}

#[test]
fn four_canonical_kinds_distinct() {
    let i = fake_issue(IssueStatus::Triage, IssuePriority::Normal, None);
    let by = Uuid::new_v4();
    let kinds = [
        issue_created(&i, by).kind,
        issue_status_changed(&i, IssueStatus::Triage, IssueStatus::Todo, by).kind,
        issue_assigned(&i, None, Some(by), by).kind,
        issue_linked(i.tenant_id, i.id, Uuid::new_v4(), LinkType::Blocks, by).kind,
    ];
    let set: std::collections::HashSet<_> = kinds.iter().collect();
    assert_eq!(set.len(), 4);
}
