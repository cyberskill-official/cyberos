//! TASK-PROJ-001 §3 — domain types.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// DEC-210 — 5-status closed enum + reserved `Deleted` for soft-delete (§4 #17).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum IssueStatus {
    Triage,
    Todo,
    Doing,
    Review,
    Done,
    Deleted,
}

impl IssueStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Triage => "triage",
            Self::Todo => "todo",
            Self::Doing => "doing",
            Self::Review => "review",
            Self::Done => "done",
            Self::Deleted => "deleted",
        }
    }
}

/// DEC-211 — 4-priority closed enum with numeric sort mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum IssuePriority {
    Urgent,
    High,
    Normal,
    Low,
}

impl IssuePriority {
    /// Numeric sort key per §1 #4 (urgent=4 high=3 normal=2 low=1).
    pub fn numeric(&self) -> u8 {
        match self {
            Self::Urgent => 4,
            Self::High => 3,
            Self::Normal => 2,
            Self::Low => 1,
        }
    }
}

/// §1 #8 closed link-type enum. Symmetric pairs (blocks/blocked_by,
/// duplicates/duplicated_by) get bidirectional rows per §1 #9.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkType {
    Duplicates,
    DuplicatedBy,
    Blocks,
    BlockedBy,
    Related,
    DerivedFromEmailThread,
    DerivedFromChatThread,
    DerivedFromMeetingDecision,
}

impl LinkType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Duplicates => "duplicates",
            Self::DuplicatedBy => "duplicated_by",
            Self::Blocks => "blocks",
            Self::BlockedBy => "blocked_by",
            Self::Related => "related",
            Self::DerivedFromEmailThread => "derived_from_email_thread",
            Self::DerivedFromChatThread => "derived_from_chat_thread",
            Self::DerivedFromMeetingDecision => "derived_from_meeting_decision",
        }
    }

    /// Symmetric inverse for bidirectional auto-insertion (§1 #9).
    pub fn inverse(&self) -> Option<Self> {
        match self {
            Self::Blocks => Some(Self::BlockedBy),
            Self::BlockedBy => Some(Self::Blocks),
            Self::Duplicates => Some(Self::DuplicatedBy),
            Self::DuplicatedBy => Some(Self::Duplicates),
            _ => None,
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "duplicates" => Self::Duplicates,
            "duplicated_by" => Self::DuplicatedBy,
            "blocks" => Self::Blocks,
            "blocked_by" => Self::BlockedBy,
            "related" => Self::Related,
            "derived_from_email_thread" => Self::DerivedFromEmailThread,
            "derived_from_chat_thread" => Self::DerivedFromChatThread,
            "derived_from_meeting_decision" => Self::DerivedFromMeetingDecision,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Engagement {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub client_id: Option<Uuid>,
    pub name: String,
    pub status: String,
    pub started_at: NaiveDate,
    pub ended_at: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Cycle {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub engagement_id: Uuid,
    pub name: String,
    pub starts_at: NaiveDate,
    pub ends_at: NaiveDate,
    pub state: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Issue {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub engagement_id: Uuid,
    pub cycle_id: Option<Uuid>,
    pub title: String,
    pub body: Option<String>,
    pub status: IssueStatus,
    pub priority: IssuePriority,
    pub assignee_subject_id: Option<Uuid>,
    /// NUMERIC(6,2) in Postgres. Represented as `f64` here so the crate
    /// avoids the `sqlx/bigdecimal` feature flag; clients submitting
    /// estimates with more than ≈15 digits of significand will lose
    /// precision — acceptable since the SQL CHECK bounds it to ≤ 9,999.99.
    pub estimate_hours: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct IssueLink {
    pub issue_id: Uuid,
    pub linked_to_id: Uuid,
    pub link_type: String,
    pub created_at: DateTime<Utc>,
}

/// Identity of the actor making the change — carried by the JWT claims
/// in the HTTP path. Tests construct directly.
#[derive(Debug, Clone, Copy)]
pub struct Actor {
    pub tenant_id: Uuid,
    pub subject_id: Uuid,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateIssueRequest {
    pub engagement_id: Uuid,
    pub cycle_id: Option<Uuid>,
    pub title: String,
    pub body: Option<String>,
    pub status: Option<IssueStatus>,
    pub priority: Option<IssuePriority>,
    pub assignee_subject_id: Option<Uuid>,
    pub estimate_hours: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct PatchIssueRequest {
    pub title: Option<String>,
    pub body: Option<String>,
    pub status: Option<IssueStatus>,
    pub priority: Option<IssuePriority>,
    pub assignee_subject_id: Option<Option<Uuid>>, // double-Option: explicit-null clears
    pub estimate_hours: Option<Option<f64>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateLinkRequest {
    pub linked_to_id: Uuid,
    pub link_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn issue_status_strings_match_sql_check() {
        assert_eq!(IssueStatus::Triage.as_str(), "triage");
        assert_eq!(IssueStatus::Todo.as_str(), "todo");
        assert_eq!(IssueStatus::Doing.as_str(), "doing");
        assert_eq!(IssueStatus::Review.as_str(), "review");
        assert_eq!(IssueStatus::Done.as_str(), "done");
        assert_eq!(IssueStatus::Deleted.as_str(), "deleted");
    }

    #[test]
    fn priority_numeric_ordering() {
        assert!(IssuePriority::Urgent.numeric() > IssuePriority::High.numeric());
        assert!(IssuePriority::High.numeric() > IssuePriority::Normal.numeric());
        assert!(IssuePriority::Normal.numeric() > IssuePriority::Low.numeric());
    }

    #[test]
    fn link_type_inverses_are_symmetric() {
        // For each symmetric type, inverse-of-inverse == original.
        for lt in [
            LinkType::Blocks,
            LinkType::BlockedBy,
            LinkType::Duplicates,
            LinkType::DuplicatedBy,
        ] {
            let inv = lt.inverse().unwrap();
            assert_eq!(inv.inverse().unwrap(), lt);
        }
    }

    #[test]
    fn asymmetric_link_types_have_no_inverse() {
        for lt in [
            LinkType::Related,
            LinkType::DerivedFromEmailThread,
            LinkType::DerivedFromChatThread,
            LinkType::DerivedFromMeetingDecision,
        ] {
            assert!(lt.inverse().is_none(), "{lt:?} should have no inverse");
        }
    }

    #[test]
    fn link_type_parse_round_trip() {
        for raw in [
            "duplicates",
            "duplicated_by",
            "blocks",
            "blocked_by",
            "related",
            "derived_from_email_thread",
            "derived_from_chat_thread",
            "derived_from_meeting_decision",
        ] {
            let parsed = LinkType::parse(raw).unwrap();
            assert_eq!(parsed.as_str(), raw);
        }
        assert!(LinkType::parse("unknown").is_none());
    }
}
