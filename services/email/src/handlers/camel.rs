//! HTTP-facing orchestration for TASK-EMAIL-005.

use crate::camel::{
    execute_persisted, list_audit_log, upsert_trust_list_entry, CamelAuditLogRow, CamelDecision,
    CamelTrustListRow,
};
use crate::errors::EmailResult;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct CamelExecuteRequest {
    pub tenant_id: Uuid,
    pub user_intent: String,
    pub email_id: Uuid,
    pub email_content: String,
    pub tools_available: Vec<String>,
    pub trace_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CamelExecuteResponse {
    pub decision: CamelDecision,
    pub audit_rows: Vec<CamelAuditLogRow>,
}

pub async fn execute(db: &PgPool, req: CamelExecuteRequest) -> EmailResult<CamelExecuteResponse> {
    let tools: Vec<&str> = req.tools_available.iter().map(String::as_str).collect();
    let (decision, audit_rows) = execute_persisted(
        db,
        req.tenant_id,
        &req.user_intent,
        req.email_id,
        &req.email_content,
        &tools,
        req.trace_id.as_deref(),
    )
    .await?;
    Ok(CamelExecuteResponse {
        decision,
        audit_rows,
    })
}

#[derive(Debug, Clone, Deserialize)]
pub struct CamelTrustListRequest {
    pub tenant_id: Uuid,
    pub domain: String,
    pub op_kind: String,
    pub full_bypass: bool,
    pub ciso_audit_row_id: Option<Uuid>,
    pub created_by: Uuid,
}

pub async fn trust_list_upsert(
    db: &PgPool,
    req: CamelTrustListRequest,
) -> EmailResult<CamelTrustListRow> {
    upsert_trust_list_entry(
        db,
        req.tenant_id,
        &req.domain,
        &req.op_kind,
        req.full_bypass,
        req.ciso_audit_row_id,
        req.created_by,
    )
    .await
    .map_err(Into::into)
}

pub async fn audit_log(
    db: &PgPool,
    tenant_id: Uuid,
    limit: Option<i64>,
) -> EmailResult<Vec<CamelAuditLogRow>> {
    list_audit_log(db, tenant_id, limit.unwrap_or(100))
        .await
        .map_err(Into::into)
}
