//! Governance-mutation audit (FR-EVAL-001 clause 12). Reuse the canonical L1 chain emit so EVAL rows
//! verify under memory's reconcile, exactly like chat, obs-router, and auth do
//! (services/shared/cyberos-audit-chain::emit_genesis). We do NOT hand-roll the chain here.
//!
//! Every governance mutation appends one l1_audit_log row, tenant-scoped and subject-attributed. The
//! governance layer is therefore itself tamper-evident, exactly like the data it governs. When no audit
//! pool is configured (tests / local), the event is logged instead (best-effort), mirroring chat.

use serde_json::{json, Value};
use uuid::Uuid;

/// The clause-12 governance event kinds. Slice 1 emits the first five; the later sub-tasks
/// (DSR, sweeper) add `eval.dsr_filed`, `eval.dsr_resolved`, `eval.retention_swept`,
/// `eval.subject_erased`, and the gated-capture skip `eval.capture_gated`.
pub mod kind {
    pub const NOTICE_PUBLISHED: &str = "eval.notice_published";
    pub const ACK_RECORDED: &str = "eval.ack_recorded";
    pub const ACCESS_GRANTED: &str = "eval.access_granted";
    pub const ACCESS_REVOKED: &str = "eval.access_revoked";
    pub const RETENTION_CHANGED: &str = "eval.retention_changed";
    /// A cross-subject read of evaluation data (reader != target), per clause 9.
    pub const EVALUATION_READ: &str = "eval.evaluation_read";
    /// A subject reading their OWN record (lighter weight), per clause 9.
    pub const SELF_READ: &str = "eval.self_read";
}

/// Append one governance row to `l1_audit_log` via the shared chain, scoped to `tenant`,
/// attributed to `actor`. `event_type` is one of the `kind::*` constants; `payload` is the canonical
/// JSON body. Best-effort: a failure is logged and swallowed (mirrors chat's audit contract) so the
/// governing operation still completes; the absence is itself visible in the logs.
///
/// `audit_pool` is the memory module's Postgres (`AppState::audit_pool`). When `None`, the event is
/// logged only - the convention used in tests and local single-DB runs.
pub async fn emit_governance(
    audit_pool: Option<&crate::db::Pool>,
    tenant: Uuid,
    actor: Uuid,
    event_type: &str,
    payload: Value,
) {
    let body = json!({ "event_type": event_type, "payload": payload }).to_string();
    if let Some(pool) = audit_pool {
        if let Err(e) =
            cyberos_audit_chain::emit_genesis(pool, tenant, actor, event_type, &body).await
        {
            tracing::warn!(
                target: "cyberos_eval::audit",
                event_type,
                error = %e,
                "governance audit emit failed (best-effort)"
            );
        }
        return;
    }
    tracing::info!(
        target: "cyberos_eval::audit",
        event_type = event_type,
        tenant_id = %tenant,
        actor = %actor,
        payload = %body,
        "eval governance audit event (no audit pool configured; logged only)"
    );
}

/// Convenience over [`emit_governance`] for handler code that already holds `AppState`. Mirrors
/// `cyberos_chat::audit::emit(state, ...)`.
pub async fn emit(state: &crate::AppState, tenant: Uuid, actor: Uuid, event_type: &str, payload: Value) {
    emit_governance(state.audit_pool.as_ref(), tenant, actor, event_type, payload).await
}
