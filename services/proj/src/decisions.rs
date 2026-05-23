//! FR-PROJ-002 — memory-anchored decision rows for issue state changes.

use crate::types::IssueStatus;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionPayload {
    pub decision_id: Uuid,
    pub issue_id: Uuid,
    pub tenant_id: Uuid,
    pub from_status: &'static str,
    pub to_status: &'static str,
    pub reason: Option<String>,
    pub decided_by_subject_id: Uuid,
    pub prior_decision_chain: Option<String>,
    pub cross_module_links: Vec<String>,
    pub request_id: String,
    pub sync_class: &'static str,
    pub acl: Vec<String>,
    pub decision_session_id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DecisionError {
    #[error("reason too long: {actual} chars (max 500)")]
    ReasonTooLong { actual: usize },
    #[error("reason required for status {0}")]
    ReasonRequired(&'static str),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DecisionPolicy {
    pub require_reason_for: Vec<IssueStatus>,
    pub default_acl: Vec<String>,
}

pub fn build_decision_payload(
    issue_id: Uuid,
    tenant_id: Uuid,
    from: IssueStatus,
    to: IssueStatus,
    reason: Option<&str>,
    decided_by_subject_id: Uuid,
    prior_decision_chain: Option<String>,
    request_id: &str,
    policy: &DecisionPolicy,
) -> Result<DecisionPayload, DecisionError> {
    if let Some(r) = reason {
        if r.chars().count() > 500 {
            return Err(DecisionError::ReasonTooLong {
                actual: r.chars().count(),
            });
        }
    }
    if policy.require_reason_for.contains(&to) && reason.map(str::trim).unwrap_or("").is_empty() {
        return Err(DecisionError::ReasonRequired(to.as_str()));
    }
    let redacted_reason = reason.map(redact_pii);
    let cross_module_links = redacted_reason
        .as_deref()
        .map(extract_cross_module_links)
        .unwrap_or_default();
    Ok(DecisionPayload {
        decision_id: Uuid::new_v4(),
        issue_id,
        tenant_id,
        from_status: from.as_str(),
        to_status: to.as_str(),
        reason: redacted_reason,
        decided_by_subject_id,
        prior_decision_chain,
        cross_module_links,
        request_id: request_id.to_owned(),
        sync_class: "shareable",
        acl: policy.default_acl.clone(),
        decision_session_id: Uuid::new_v4(),
    })
}

pub fn extract_cross_module_links(reason: &str) -> Vec<String> {
    reason
        .split_whitespace()
        .filter_map(|token| {
            let trimmed =
                token.trim_matches(|c: char| c == ',' || c == '.' || c == ')' || c == '(');
            let lower = trimmed.to_ascii_lowercase();
            if lower.starts_with("chat://")
                || lower.starts_with("email://")
                || lower.starts_with("meeting://")
            {
                Some(lower)
            } else {
                None
            }
        })
        .collect()
}

pub fn retraction_payload(
    tenant_id: Uuid,
    retracts_decision_id: Uuid,
    retraction_reason: &str,
    retracted_by_subject_id: Uuid,
) -> serde_json::Value {
    serde_json::json!({
        "tenant_id": tenant_id,
        "retracts_decision_id": retracts_decision_id,
        "retraction_reason": redact_pii(retraction_reason),
        "retracted_by_subject_id": retracted_by_subject_id,
    })
}

fn redact_pii(input: &str) -> String {
    input
        .split_whitespace()
        .map(|part| {
            if part.contains('@') && part.contains('.') {
                "[redacted-email]".to_string()
            } else {
                part.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decision_payload_extracts_links_and_defaults_shareable() {
        let payload = build_decision_payload(
            Uuid::new_v4(),
            Uuid::new_v4(),
            IssueStatus::Doing,
            IssueStatus::Done,
            Some("approved in chat://Thread/ABC and email://thread/XYZ"),
            Uuid::new_v4(),
            Some("a".repeat(64)),
            "req-1",
            &DecisionPolicy::default(),
        )
        .unwrap();
        assert_eq!(payload.sync_class, "shareable");
        assert_eq!(
            payload.cross_module_links,
            vec!["chat://thread/abc", "email://thread/xyz"]
        );
        assert_eq!(payload.prior_decision_chain, Some("a".repeat(64)));
    }

    #[test]
    fn reason_length_and_required_policy_enforced() {
        let policy = DecisionPolicy {
            require_reason_for: vec![IssueStatus::Done],
            default_acl: vec![],
        };
        let err = build_decision_payload(
            Uuid::new_v4(),
            Uuid::new_v4(),
            IssueStatus::Review,
            IssueStatus::Done,
            None,
            Uuid::new_v4(),
            None,
            "req",
            &policy,
        )
        .unwrap_err();
        assert_eq!(err, DecisionError::ReasonRequired("done"));

        let long = "x".repeat(501);
        let err = build_decision_payload(
            Uuid::new_v4(),
            Uuid::new_v4(),
            IssueStatus::Todo,
            IssueStatus::Doing,
            Some(&long),
            Uuid::new_v4(),
            None,
            "req",
            &DecisionPolicy::default(),
        )
        .unwrap_err();
        assert_eq!(err, DecisionError::ReasonTooLong { actual: 501 });
    }

    #[test]
    fn reason_redacts_email_addresses() {
        let payload = build_decision_payload(
            Uuid::new_v4(),
            Uuid::new_v4(),
            IssueStatus::Todo,
            IssueStatus::Doing,
            Some("ask alice@example.com"),
            Uuid::new_v4(),
            None,
            "req",
            &DecisionPolicy::default(),
        )
        .unwrap();
        assert_eq!(payload.reason.as_deref(), Some("ask [redacted-email]"));
    }
}
