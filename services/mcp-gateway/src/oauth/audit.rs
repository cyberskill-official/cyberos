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
