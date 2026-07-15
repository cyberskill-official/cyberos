//! Read side of the shared hash-chained audit log (l1_audit_log), tenant-scoped, for the console's
//! Memory & Audit browser (TASK-APP-005). It reads from the chat audit pool (the memory DB that holds the
//! chain). The long-term home for this read is the memory module; it lives here for now because chat
//! already verifies the CyberOS token and holds the audit pool. Read-only.

use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{auth, AppState};

#[derive(Debug, Serialize)]
pub struct AuditRow {
    pub seq: i64,
    pub op: String,
    pub path: String,
    pub event_type: Option<String>,
    pub subject_id: Option<Uuid>,
    pub chain_anchor_hex: Option<String>,
    pub ingested_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AuditQuery {
    pub limit: Option<i64>,
}

type Row = (
    i64,
    String,
    String,
    Option<String>,
    Option<Uuid>,
    Option<String>,
    chrono::DateTime<chrono::Utc>,
);

/// GET /v1/chat/audit?limit= - the caller's tenant's most recent audit-chain rows, newest first.
pub async fn list(
    State(st): State<AppState>,
    Query(q): Query<AuditQuery>,
    headers: HeaderMap,
) -> Result<Json<Vec<AuditRow>>, (StatusCode, String)> {
    let claims = auth::authenticate(&st, &headers)?;
    let tenant = claims
        .tenant_uuid()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let pool = match &st.audit_pool {
        Some(p) => p,
        None => return Ok(Json(vec![])),
    };
    let limit = q.limit.unwrap_or(50).clamp(1, 500);
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT seq, op, path, event_type, subject_id, chain_anchor_hex, ingested_at
         FROM l1_audit_log WHERE tenant_id = $1 ORDER BY seq DESC LIMIT $2",
    )
    .bind(tenant)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(crate::internal)?;
    Ok(Json(
        rows.into_iter()
            .map(|r| AuditRow {
                seq: r.0,
                op: r.1,
                path: r.2,
                event_type: r.3,
                subject_id: r.4,
                chain_anchor_hex: r.5,
                ingested_at: r.6,
            })
            .collect(),
    ))
}
