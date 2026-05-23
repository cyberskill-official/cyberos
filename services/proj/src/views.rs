//! FR-PROJ-014..018 — board/timeline/gantt/brief view models + tokens.

use crate::types::{Issue, IssueStatus};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KanbanColumn {
    pub status: IssueStatus,
    pub issue_ids: Vec<Uuid>,
}

pub fn kanban_columns(issues: &[Issue]) -> Vec<KanbanColumn> {
    let statuses = [
        IssueStatus::Triage,
        IssueStatus::Todo,
        IssueStatus::Doing,
        IssueStatus::Review,
        IssueStatus::Done,
    ];
    statuses
        .into_iter()
        .map(|status| KanbanColumn {
            status,
            issue_ids: issues
                .iter()
                .filter(|i| i.status == status)
                .map(|i| i.id)
                .collect(),
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimelineLane {
    pub assignee_subject_id: Option<Uuid>,
    pub issue_ids: Vec<Uuid>,
}

pub fn timeline_lanes(issues: &[Issue]) -> Vec<TimelineLane> {
    let mut map: BTreeMap<Option<Uuid>, Vec<Uuid>> = BTreeMap::new();
    for issue in issues {
        map.entry(issue.assignee_subject_id)
            .or_default()
            .push(issue.id);
    }
    map.into_iter()
        .map(|(assignee_subject_id, issue_ids)| TimelineLane {
            assignee_subject_id,
            issue_ids,
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GanttTask {
    pub issue_id: Uuid,
    pub starts_at: NaiveDate,
    pub ends_at: NaiveDate,
    pub depends_on: Vec<Uuid>,
    pub critical: bool,
}

pub fn mark_critical_path(tasks: &mut [GanttTask]) {
    let mut dependents: BTreeMap<Uuid, usize> = BTreeMap::new();
    for task in tasks.iter() {
        for dep in &task.depends_on {
            *dependents.entry(*dep).or_default() += 1;
        }
    }
    for task in tasks.iter_mut() {
        task.critical = task.depends_on.is_empty()
            && dependents.get(&task.issue_id).copied().unwrap_or(0) > 0
            || !task.depends_on.is_empty();
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BriefModalModel {
    pub issue_id: Uuid,
    pub title: String,
    pub description_doc_id: Uuid,
    pub comment_doc_ids: Vec<Uuid>,
    pub meta: BTreeMap<String, String>,
}

pub const TOKENS_PROJ_CSS: &str = r#":root {
  --proj-radius: 8px;
  --proj-focus-ring: #2563eb;
  --proj-status-triage: #64748b;
  --proj-status-todo: #0f766e;
  --proj-status-doing: #b45309;
  --proj-status-review: #7c3aed;
  --proj-status-done: #15803d;
}
"#;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct A11yFinding {
    pub rule: String,
    pub selector: String,
}

pub fn axe_gate(findings: &[A11yFinding]) -> Result<(), Vec<A11yFinding>> {
    let blocking = findings
        .iter()
        .filter(|finding| {
            matches!(
                finding.rule.as_str(),
                "color-contrast" | "button-name" | "aria-required-attr" | "keyboard"
            )
        })
        .cloned()
        .collect::<Vec<_>>();
    if blocking.is_empty() {
        Ok(())
    } else {
        Err(blocking)
    }
}
