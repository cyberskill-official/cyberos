//! FR-MCP-004 memory audit kinds (clause #25). Each is a best-effort genesis row appended to the shared
//! `l1_audit_log` via [`cyberos_audit_chain::emit_genesis`] - the same chain the obs services write to.
//! A failed audit never fails the OAuth operation (the security action already happened); the error is
//! logged and swallowed. Bodies carry only non-PII identifiers; any reason text must already be
//! scrubbed (clause #26).

use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

/// Append one audit row, swallowing errors (best-effort, like the obs sink).
async fn emit(pool: &PgPool, tenant: Uuid, subject: Uuid, kind: &str, body: serde_json::Value) {
    if let Err(e) = cyberos_audit_chain::emit_genesis(pool, tenant, subject, kind, &body.to_string()).await {
        eprintln!("mcp oauth audit emit failed (best-effort) {kind}: {e}");
    }
}

/// `mcp.oauth_authorize_started` (sev-3) - an authorization code was issued and a redirect returned.
pub async fn authorize_started(pool: &PgPool, tenant: Uuid, subject: Uuid, client_id: Uuid) {
    emit(pool, tenant, subject, "mcp.oauth_authorize_started",
         json!({ "sev": 3, "client_id": client_id })).await;
}

/// `mcp.oauth_token_issued` (sev-3) - an access token was issued via the authorization_code grant.
pub async fn token_issued(pool: &PgPool, tenant: Uuid, subject: Uuid, client_id: Uuid, jti: &str, scope: &str) {
    emit(pool, tenant, subject, "mcp.oauth_token_issued",
         json!({ "sev": 3, "client_id": client_id, "jti": jti, "scope": scope })).await;
}

/// `mcp.oauth_token_refreshed` (sev-3) - a refresh token was rotated and a new access token issued.
pub async fn token_refreshed(pool: &PgPool, tenant: Uuid, subject: Uuid, client_id: Uuid, jti: &str) {
    emit(pool, tenant, subject, "mcp.oauth_token_refreshed",
         json!({ "sev": 3, "client_id": client_id, "jti": jti })).await;
}

/// `mcp.oauth_token_revoked` (sev-2) - a refresh family was compromised by a revocation request.
pub async fn token_revoked(pool: &PgPool, family_id: Uuid) {
    emit(pool, Uuid::nil(), Uuid::nil(), "mcp.oauth_token_revoked",
         json!({ "sev": 2, "family_id": family_id })).await;
}

/// `mcp.oauth_refresh_reuse_detected` (sev-1) - a used/compromised refresh token was presented; the
/// whole family was poisoned.
pub async fn refresh_reuse_detected(pool: &PgPool, family_id: Uuid) {
    emit(pool, Uuid::nil(), Uuid::nil(), "mcp.oauth_refresh_reuse_detected",
         json!({ "sev": 1, "family_id": family_id })).await;
}

/// `mcp.oauth_code_reuse_detected` (sev-2) - an already-consumed authorization code was replayed.
pub async fn code_reuse_detected(pool: &PgPool, client_id: Uuid) {
    emit(pool, Uuid::nil(), Uuid::nil(), "mcp.oauth_code_reuse_detected",
         json!({ "sev": 2, "client_id": client_id })).await;
}

/// `mcp.oauth_audience_mismatch` (sev-2) - a tools/call presented a token bound to another resource.
pub async fn audience_mismatch(pool: &PgPool) {
    emit(pool, Uuid::nil(), Uuid::nil(), "mcp.oauth_audience_mismatch",
         json!({ "sev": 2 })).await;
}

/// `mcp.oauth_client_registered` (sev-2) - a client registered via RFC 7591 DCR.
pub async fn client_registered(pool: &PgPool, tenant: Uuid, subject: Uuid, client_id: Uuid, client_type: &str) {
    emit(pool, tenant, subject, "mcp.oauth_client_registered",
         json!({ "sev": 2, "client_id": client_id, "client_type": client_type })).await;
}

/// `mcp.prm_unknown_module_requested` (sev-3) - FR-MCP-005 Protected Resource Metadata was requested
/// for a module not in the registry (client misconfiguration or a scanner probing module namespaces).
pub async fn prm_unknown_module_requested(pool: &PgPool, module: &str) {
    emit(pool, Uuid::nil(), Uuid::nil(), "mcp.prm_unknown_module_requested",
         json!({ "sev": 3, "module": module })).await;
}

// ---- FR-MCP-008 elicitation (DEC-1149) ---------------------------------------------

/// `mcp.elicitation_requested` (sev-3) - a tool/gate raised a server-initiated elicitation.
pub async fn elicitation_requested(pool: &PgPool, id: Uuid, tool_id: &str, elicitation_type: &str) {
    emit(pool, Uuid::nil(), Uuid::nil(), "mcp.elicitation_requested",
         json!({ "sev": 3, "elicitation_id": id, "tool_id": tool_id, "elicitation_type": elicitation_type })).await;
}

/// `mcp.elicitation_responded` (sev-3) - the caller answered and the payload validated.
pub async fn elicitation_responded(pool: &PgPool, id: Uuid) {
    emit(pool, Uuid::nil(), Uuid::nil(), "mcp.elicitation_responded",
         json!({ "sev": 3, "elicitation_id": id })).await;
}

/// `mcp.elicitation_timeout` (sev-3) - a pending elicitation elapsed its timeout (deferred sweeper).
pub async fn elicitation_timeout(pool: &PgPool, id: Uuid) {
    emit(pool, Uuid::nil(), Uuid::nil(), "mcp.elicitation_timeout",
         json!({ "sev": 3, "elicitation_id": id })).await;
}

/// `mcp.elicitation_cancelled` (sev-3) - the caller cancelled a pending elicitation.
pub async fn elicitation_cancelled(pool: &PgPool, id: Uuid) {
    emit(pool, Uuid::nil(), Uuid::nil(), "mcp.elicitation_cancelled",
         json!({ "sev": 3, "elicitation_id": id })).await;
}

/// `mcp.elicitation_validation_failed` (sev-2) - the retry cap was hit with an invalid response
/// (bad tool schema or a misbehaving client).
pub async fn elicitation_validation_failed(pool: &PgPool, id: Uuid) {
    emit(pool, Uuid::nil(), Uuid::nil(), "mcp.elicitation_validation_failed",
         json!({ "sev": 2, "elicitation_id": id })).await;
}

// ---- FR-MCP-007 tasks (DEC-1124) ---------------------------------------------------

/// `mcp.task_started` (sev-3) - a long-running task began.
pub async fn task_started(pool: &PgPool, id: Uuid, tool_id: &str) {
    emit(pool, Uuid::nil(), Uuid::nil(), "mcp.task_started",
         json!({ "sev": 3, "task_id": id, "tool_id": tool_id })).await;
}

/// `mcp.task_completed` (sev-3) - a task finished successfully.
pub async fn task_completed(pool: &PgPool, id: Uuid) {
    emit(pool, Uuid::nil(), Uuid::nil(), "mcp.task_completed",
         json!({ "sev": 3, "task_id": id })).await;
}

/// `mcp.task_failed` (sev-2) - a task finished with an error.
pub async fn task_failed(pool: &PgPool, id: Uuid, code: &str) {
    emit(pool, Uuid::nil(), Uuid::nil(), "mcp.task_failed",
         json!({ "sev": 2, "task_id": id, "code": code })).await;
}

/// `mcp.task_cancelled` (sev-3) - a task was cancelled by the caller.
pub async fn task_cancelled(pool: &PgPool, id: Uuid) {
    emit(pool, Uuid::nil(), Uuid::nil(), "mcp.task_cancelled",
         json!({ "sev": 3, "task_id": id })).await;
}

/// `mcp.task_expired` (sev-3) - a task elapsed its TTL (deferred sweeper).
pub async fn task_expired(pool: &PgPool, id: Uuid) {
    emit(pool, Uuid::nil(), Uuid::nil(), "mcp.task_expired",
         json!({ "sev": 3, "task_id": id })).await;
}

// ---- FR-MCP-003 SEP-986 naming (DEC-2364) ------------------------------------------

/// `mcp.skill_name_validated` (sev-3) - a module registered with SEP-986-conforming tool IDs.
pub async fn skill_name_validated(pool: &PgPool, module: &str, tool_count: usize) {
    emit(pool, Uuid::nil(), Uuid::nil(), "mcp.skill_name_validated",
         json!({ "sev": 3, "module": module, "tools": tool_count })).await;
}

/// `mcp.skill_name_rejected` (sev-2) - a registration was refused for a non-conforming tool ID.
pub async fn skill_name_rejected(pool: &PgPool, module: &str, detail: &str) {
    emit(pool, Uuid::nil(), Uuid::nil(), "mcp.skill_name_rejected",
         json!({ "sev": 2, "module": module, "detail": detail })).await;
}

/// `mcp.naming_ci_check_passed` (sev-3) - the CI grep gate (scripts/check_sep986_naming.sh) passed.
/// Emitted when CI can reach the memory chain; until then the gate's exit code is the signal.
pub async fn naming_ci_check_passed(pool: &PgPool) {
    emit(pool, Uuid::nil(), Uuid::nil(), "mcp.naming_ci_check_passed", json!({ "sev": 3 })).await;
}

/// `mcp.naming_ci_check_failed` (sev-2) - the CI grep gate found a non-conforming skill ID.
pub async fn naming_ci_check_failed(pool: &PgPool, detail: &str) {
    emit(pool, Uuid::nil(), Uuid::nil(), "mcp.naming_ci_check_failed",
         json!({ "sev": 2, "detail": detail })).await;
}
