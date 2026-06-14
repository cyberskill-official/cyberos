//! Compliance view implementations.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::Claims;
use crate::chain_proof::ChainProof;
use crate::error::ViewError;
use crate::memory::{AuditRow, MemoryBackend};
use crate::router::ViewQuery;

pub mod eu_ai_act;
pub mod iso27001;
pub mod pdpl;
pub mod soc2;

/// Response format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    /// JSON response.
    Json,
    /// PDF response.
    Pdf,
}

impl Default for Format {
    fn default() -> Self {
        Self::Json
    }
}

impl Format {
    /// Stable label.
    pub fn as_label(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Pdf => "pdf",
        }
    }
}

/// View response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ViewResponse {
    /// Regulation name.
    pub regulation: String,
    /// Tenant id.
    pub tenant_id: String,
    /// Time range.
    pub time_range: (DateTime<Utc>, DateTime<Utc>),
    /// Evidence rows.
    pub rows: Vec<AuditRow>,
    /// Summary.
    pub summary: serde_json::Value,
    /// Chain proof.
    pub chain_proof: ChainProof,
}

/// View definition.
#[derive(Debug, Clone, Copy)]
pub struct ViewDefinition {
    /// Path id.
    pub id: &'static str,
    /// Regulation label.
    pub regulation: &'static str,
    /// Row kinds or prefixes.
    pub kinds: &'static [&'static str],
    /// Summary function.
    pub summarize: fn(&[AuditRow]) -> serde_json::Value,
}

/// Build a view.
pub async fn build_view(
    backend: &dyn MemoryBackend,
    signer: &crate::chain_proof::ChainProofSigner,
    definition: ViewDefinition,
    query: ViewQuery,
    claims: Claims,
) -> Result<ViewResponse, ViewError> {
    validate_query(&query)?;
    let rows = backend
        .query(
            &claims.tenant_id,
            definition.kinds,
            query.since,
            query.until,
        )
        .await?;
    reject_raw_pii(&rows)?;
    let summary = (definition.summarize)(&rows);
    let sth = backend.current_signed_tree_head().await?;
    let chain_proof = signer.sign(&rows, &summary, &sth)?;
    let response = ViewResponse {
        regulation: definition.regulation.to_string(),
        tenant_id: claims.tenant_id.clone(),
        time_range: (query.since, query.until),
        rows,
        summary,
        chain_proof,
    };
    emit_accessed(backend, definition.id, &query, &claims).await?;
    Ok(response)
}

fn validate_query(query: &ViewQuery) -> Result<(), ViewError> {
    if query.tenant_id.is_some() {
        return Err(ViewError::TenantIdInQuery);
    }
    if query.until < query.since {
        return Err(ViewError::TimeRangeTooLarge);
    }
    let days = query.until.signed_duration_since(query.since).num_days();
    if days > 365 {
        return Err(ViewError::TimeRangeTooLarge);
    }
    Ok(())
}

fn reject_raw_pii(rows: &[AuditRow]) -> Result<(), ViewError> {
    let raw =
        serde_json::to_string(rows).map_err(|err| ViewError::ExportFailed(err.to_string()))?;
    if contains_email(&raw) || contains_vn_phone(&raw) || contains_cccd(&raw) {
        return Err(ViewError::PiiLeakAttempt);
    }
    Ok(())
}

fn contains_email(raw: &str) -> bool {
    raw.split(|c: char| c.is_whitespace() || [',', '"', '<', '>', '(', ')'].contains(&c))
        .any(|token| {
            let parts: Vec<_> = token.split('@').collect();
            parts.len() == 2 && parts[0].len() >= 2 && parts[1].contains('.')
        })
}

fn contains_vn_phone(raw: &str) -> bool {
    raw.split(|c: char| !c.is_ascii_digit())
        .any(|token| token.len() == 10 && token.starts_with('0'))
}

fn contains_cccd(raw: &str) -> bool {
    raw.split(|c: char| !c.is_ascii_digit())
        .any(|token| token.len() == 12)
}

async fn emit_accessed(
    backend: &dyn MemoryBackend,
    view: &str,
    query: &ViewQuery,
    claims: &Claims,
) -> Result<(), ViewError> {
    let days = query
        .until
        .signed_duration_since(query.since)
        .num_days()
        .max(0);
    let row = AuditRow::new(
        Utc::now(),
        "obs.compliance_view_accessed",
        &claims.tenant_id,
        serde_json::json!({
            "auditor_subject_id": claims.sub,
            "tenant_id": claims.tenant_id,
            "view": view,
            "time_range_days": days,
            "request_id": claims.request_id(),
        }),
    );
    backend.emit_accessed(row).await
}

fn count_kind(rows: &[AuditRow], kind: &str) -> usize {
    rows.iter().filter(|row| row.kind == kind).count()
}

fn count_prefix(rows: &[AuditRow], prefix: &str) -> usize {
    rows.iter()
        .filter(|row| row.kind.starts_with(prefix))
        .count()
}

fn unique_payload_field(rows: &[AuditRow], field: &str) -> usize {
    let values: std::collections::BTreeSet<_> = rows
        .iter()
        .filter_map(|row| row.payload.get(field).and_then(serde_json::Value::as_str))
        .collect();
    values.len()
}

/// Shared test helper.
pub async fn handle_for_test(
    backend: Arc<dyn MemoryBackend>,
    signer: &crate::chain_proof::ChainProofSigner,
    definition: ViewDefinition,
    query: ViewQuery,
    claims: Claims,
) -> Result<ViewResponse, ViewError> {
    build_view(backend.as_ref(), signer, definition, query, claims).await
}

/// Nil UUID marker used by dashboard examples.
pub fn nil_tenant() -> String {
    Uuid::nil().to_string()
}
