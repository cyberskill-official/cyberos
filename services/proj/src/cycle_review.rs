//! FR-PROJ-012 — deterministic cycle-review draft inputs for CUO/COO.

use crate::types::{Issue, IssueStatus};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CycleReviewStats {
    pub total: usize,
    pub done: usize,
    pub in_progress: usize,
    pub blocked: usize,
}

pub fn summarize_cycle(issues: &[Issue], blocked_issue_count: usize) -> CycleReviewStats {
    CycleReviewStats {
        total: issues.len(),
        done: issues
            .iter()
            .filter(|i| i.status == IssueStatus::Done)
            .count(),
        in_progress: issues
            .iter()
            .filter(|i| matches!(i.status, IssueStatus::Doing | IssueStatus::Review))
            .count(),
        blocked: blocked_issue_count,
    }
}

pub fn draft_cycle_review(stats: &CycleReviewStats) -> String {
    let completion = if stats.total == 0 {
        100
    } else {
        stats.done * 100 / stats.total
    };
    format!(
        "Cycle review\n\n- Completion: {completion}% ({}/{})\n- In progress: {}\n- Blocked: {}\n\nFocus next cycle on clearing blockers before adding new scope.",
        stats.done, stats.total, stats.in_progress, stats.blocked
    )
}
