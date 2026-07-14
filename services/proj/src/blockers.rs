//! TASK-PROJ-011 — blocker detector from comment streams.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Comment {
    pub issue_id: Uuid,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockerSignal {
    pub issue_id: Uuid,
    pub blocker_text: String,
    pub dwell_hours: i64,
    pub notify_cuo: bool,
}

pub fn detect_blockers(
    comments: &[Comment],
    now: DateTime<Utc>,
    threshold: Duration,
) -> Vec<BlockerSignal> {
    comments
        .iter()
        .filter_map(|comment| {
            let lower = comment.body.to_ascii_lowercase();
            let idx = lower.find("blocked by")?;
            let text = comment.body[idx + "blocked by".len()..]
                .trim_matches(|c: char| c == ':' || c == '-' || c.is_whitespace())
                .to_string();
            let dwell = now - comment.created_at;
            Some(BlockerSignal {
                issue_id: comment.issue_id,
                blocker_text: if text.is_empty() {
                    "unspecified".into()
                } else {
                    text
                },
                dwell_hours: dwell.num_hours(),
                notify_cuo: dwell >= threshold,
            })
        })
        .collect()
}
