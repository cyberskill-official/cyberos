//! TASK-PROJ-002 — memory-anchored decision rows for issue state changes.

use crate::errors::{IssueError, IssueResult};
use crate::repo;
use crate::status_fsm;
use crate::types::IssueStatus;
use crate::types::{Actor, Issue};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
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
    pub decision_attributes: serde_json::Value,
    pub chain_anchor_in_payload: Option<String>,
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

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct DecisionRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub issue_id: Uuid,
    pub from_status: String,
    pub to_status: String,
    pub reason: Option<String>,
    pub decided_by_subject_id: Uuid,
    pub prior_decision_chain: Option<String>,
    pub cross_module_links: Vec<String>,
    pub request_id: String,
    pub sync_class: String,
    pub acl: Vec<String>,
    pub decision_session_id: Uuid,
    pub decision_attributes: serde_json::Value,
    pub memory_chain_hash: Option<String>,
    pub chain_anchor_in_payload: Option<String>,
    pub retracted_at: Option<DateTime<Utc>>,
    pub retracted_by_subject_id: Option<Uuid>,
    pub retraction_reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct DecisionRetractionRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub retracts_decision_id: Uuid,
    pub retraction_reason: String,
    pub retracted_by_subject_id: Uuid,
    pub request_id: String,
    pub created_at: DateTime<Utc>,
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
        decision_attributes: serde_json::Value::Object(Default::default()),
        chain_anchor_in_payload: None,
    })
}

pub async fn latest_decision_chain(
    db: &PgPool,
    tenant_id: Uuid,
    issue_id: Uuid,
) -> IssueResult<Option<String>> {
    let mut tx = db.begin().await?;
    repo::set_tenant(&mut tx, tenant_id).await?;
    let chain: Option<String> = sqlx::query_scalar(
        "SELECT memory_chain_hash
         FROM proj_decisions
         WHERE tenant_id = $1 AND issue_id = $2 AND memory_chain_hash IS NOT NULL
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(tenant_id)
    .bind(issue_id)
    .fetch_optional(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(chain)
}

pub async fn patch_issue_status_with_decision(
    db: &PgPool,
    actor: Actor,
    issue_id: Uuid,
    if_match: Option<DateTime<Utc>>,
    to_status: IssueStatus,
    reason: Option<&str>,
    request_id: &str,
    policy: &DecisionPolicy,
    decision_attributes: serde_json::Value,
    memory_chain_hash: Option<String>,
) -> IssueResult<(Issue, DecisionRow)> {
    let mut tx = db.begin().await?;
    repo::set_tenant(&mut tx, actor.tenant_id).await?;
    let current: Option<Issue> = sqlx::query_as("SELECT * FROM issues WHERE id = $1")
        .bind(issue_id)
        .fetch_optional(&mut *tx)
        .await?;
    let current = current.ok_or(IssueError::NotFound)?;

    if let Some(expected) = if_match {
        if expected != current.updated_at {
            return Err(IssueError::ConcurrentUpdate {
                expected,
                actual: current.updated_at,
            });
        }
    }
    if to_status != current.status {
        status_fsm::validate(current.status, to_status)?;
    }

    let prior_chain: Option<String> = sqlx::query_scalar(
        "SELECT memory_chain_hash
         FROM proj_decisions
         WHERE tenant_id = $1 AND issue_id = $2 AND memory_chain_hash IS NOT NULL
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(actor.tenant_id)
    .bind(issue_id)
    .fetch_optional(&mut *tx)
    .await?;

    let mut payload = build_decision_payload(
        issue_id,
        actor.tenant_id,
        current.status,
        to_status,
        reason,
        actor.subject_id,
        prior_chain,
        request_id,
        policy,
    )
    .map_err(|e| IssueError::Validation(e.to_string()))?;
    payload.decision_attributes = decision_attributes;
    payload.chain_anchor_in_payload = memory_chain_hash.clone();

    let decision: DecisionRow = sqlx::query_as(
        "INSERT INTO proj_decisions (
            id, tenant_id, issue_id, from_status, to_status, reason,
            decided_by_subject_id, prior_decision_chain, cross_module_links,
            request_id, sync_class, acl, decision_session_id,
            decision_attributes, memory_chain_hash, chain_anchor_in_payload
         )
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16)
         ON CONFLICT (tenant_id, issue_id, request_id) DO UPDATE SET
            request_id = EXCLUDED.request_id
         RETURNING *",
    )
    .bind(payload.decision_id)
    .bind(payload.tenant_id)
    .bind(payload.issue_id)
    .bind(payload.from_status)
    .bind(payload.to_status)
    .bind(payload.reason)
    .bind(payload.decided_by_subject_id)
    .bind(payload.prior_decision_chain)
    .bind(payload.cross_module_links)
    .bind(payload.request_id)
    .bind(payload.sync_class)
    .bind(payload.acl)
    .bind(payload.decision_session_id)
    .bind(payload.decision_attributes)
    .bind(memory_chain_hash)
    .bind(payload.chain_anchor_in_payload)
    .fetch_one(&mut *tx)
    .await?;

    let updated: Issue = sqlx::query_as(
        "UPDATE issues
         SET status = $3
         WHERE tenant_id = $1 AND id = $2
         RETURNING *",
    )
    .bind(actor.tenant_id)
    .bind(issue_id)
    .bind(to_status.as_str())
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok((updated, decision))
}

pub async fn retract_decision(
    db: &PgPool,
    actor: Actor,
    decision_id: Uuid,
    retraction_reason: &str,
    request_id: &str,
) -> IssueResult<DecisionRetractionRow> {
    if retraction_reason.trim().is_empty() {
        return Err(IssueError::Validation("retraction_reason required".into()));
    }
    let mut tx = db.begin().await?;
    repo::set_tenant(&mut tx, actor.tenant_id).await?;
    let row: DecisionRetractionRow = sqlx::query_as(
        "INSERT INTO proj_decision_retractions (
            tenant_id, retracts_decision_id, retraction_reason,
            retracted_by_subject_id, request_id
         )
         VALUES ($1,$2,$3,$4,$5)
         ON CONFLICT (tenant_id, retracts_decision_id, request_id) DO UPDATE SET
            request_id = EXCLUDED.request_id
         RETURNING *",
    )
    .bind(actor.tenant_id)
    .bind(decision_id)
    .bind(redact_pii(retraction_reason))
    .bind(actor.subject_id)
    .bind(request_id)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(row)
}

pub async fn list_issue_decisions(
    db: &PgPool,
    actor: Actor,
    issue_id: Uuid,
    limit: i64,
) -> IssueResult<Vec<DecisionRow>> {
    let mut tx = db.begin().await?;
    repo::set_tenant(&mut tx, actor.tenant_id).await?;
    let rows: Vec<DecisionRow> = sqlx::query_as(
        "SELECT * FROM proj_decisions
         WHERE tenant_id = $1 AND issue_id = $2
         ORDER BY created_at ASC
         LIMIT $3",
    )
    .bind(actor.tenant_id)
    .bind(issue_id)
    .bind(limit.clamp(1, 500))
    .fetch_all(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(rows)
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
