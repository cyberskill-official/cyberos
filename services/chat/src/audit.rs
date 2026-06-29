//! Audit emit. When an audit pool is configured (the memory module's Postgres, which holds l1_audit_log),
//! chat appends a hash-chained genesis row via `cyberos-audit-chain` - byte-compatible with the memory
//! chain, the same way obs-router and obs-compliance-view emit. Without a pool (tests/local), it logs.

use serde_json::{json, Value};
use uuid::Uuid;

pub async fn emit(
    state: &crate::AppState,
    tenant: Uuid,
    actor: Uuid,
    event_type: &str,
    payload: Value,
) {
    let body = json!({ "event_type": event_type, "payload": payload }).to_string();
    if let Some(pool) = &state.audit_pool {
        if let Err(e) =
            cyberos_audit_chain::emit_genesis(pool, tenant, actor, event_type, &body).await
        {
            tracing::warn!(target: "cyberos_chat::audit", event_type, error = %e, "audit emit failed (best-effort)");
        }
        return;
    }
    tracing::info!(
        target: "cyberos_chat::audit",
        event_type = event_type,
        tenant_id = %tenant,
        actor = %actor,
        payload = %body,
        "chat audit event (no audit pool configured; logged only)"
    );
}
