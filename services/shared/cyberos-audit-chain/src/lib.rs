//! Shared memory audit-chain emit for the obs services (FR-OBS-007 §1 #6, FR-OBS-008 §1 #10).
//!
//! obs-router and obs-compliance-view both need to append an audit row to the memory module's
//! `l1_audit_log`. Like auth's `memory_bridge`, they write directly into that table (auth, memory, and
//! obs share one Postgres deployment in the current topology), which avoids coupling the alert route or
//! the compliance response to memory's HTTP liveness.
//!
//! The chain anchor is byte-identical to `cyberos_memory::layer2::chain_anchor::compute` and
//! `auth::memory_bridge::chain_anchor` - `SHA-256(prev_hash_hex || body)`, hex - so memory's reconcile
//! invariant accepts obs-written rows. obs events are independent, so each is a genesis row
//! (`prev_hash_hex = NULL`), exactly like auth's best-effort `emit_token_issued` / `emit_token_failed`.

use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

/// The canonical chain anchor: `SHA-256(prev_hash_hex || body)`, lowercase hex. Mirrors the memory
/// module's `chain_anchor::compute` exactly so a row written here verifies under memory's reconcile.
pub fn chain_anchor(prev_hash_hex: Option<&str>, body: &str) -> String {
    let mut h = Sha256::new();
    if let Some(prev) = prev_hash_hex {
        h.update(prev.as_bytes());
    }
    h.update(body.as_bytes());
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// The `op` values the `l1_audit_log` `l1_op_enum` CHECK accepts (migration 0003). The chain stores a
/// mutation-vs-read distinction here; `emit_genesis` only ever produced `'put'`, but read-only producers
/// (FR-MEMORY-121 interaction-events) need `'view'`. Keep this in sync with the migration's CHECK.
const ALLOWED_OPS: [&str; 4] = ["put", "move", "delete", "view"];

/// Best-effort insert of a genesis audit row into `l1_audit_log` (`op = 'put'`, `prev_hash_hex = NULL`).
/// Returns the new `seq`. The caller decides what to do on error - the obs callers log and swallow so the
/// alert route or the view response still completes (the FR's best-effort contract).
///
/// This is now a thin `'put'` shim over [`emit_genesis_with_op`]; every existing caller (auth, chat, eval,
/// obs-router, obs-compliance-view, mcp-gateway) keeps its exact 5-argument signature and `'put'` behaviour.
pub async fn emit_genesis(
    pool: &PgPool,
    tenant_id: Uuid,
    subject_id: Uuid,
    path: &str,
    body: &str,
) -> Result<i64, sqlx::Error> {
    emit_genesis_with_op(pool, tenant_id, subject_id, "put", path, body).await
}

/// Best-effort insert of a genesis audit row into `l1_audit_log` with an explicit `op`
/// (`prev_hash_hex = NULL`). Returns the new `seq`. Added for FR-MEMORY-121 §1 #6 so read-only
/// interaction-events chain as `'view'` rather than `'put'`; `emit_genesis` is the `'put'` shim over it,
/// so no existing caller changes.
///
/// `op` MUST be one of the values the `l1_op_enum` CHECK accepts (`put | move | delete | view`). An
/// unknown `op` is rejected here (returning a `sqlx` protocol error) so a typo never reaches the INSERT
/// only to fail on the CHECK with a less specific message; in debug builds it also trips a `debug_assert`.
pub async fn emit_genesis_with_op(
    pool: &PgPool,
    tenant_id: Uuid,
    subject_id: Uuid,
    op: &str,
    path: &str,
    body: &str,
) -> Result<i64, sqlx::Error> {
    debug_assert!(
        ALLOWED_OPS.contains(&op),
        "op must be one of {ALLOWED_OPS:?}, got {op:?}"
    );
    if !ALLOWED_OPS.contains(&op) {
        return Err(sqlx::Error::Protocol(format!(
            "invalid audit op {op:?}; must be one of {ALLOWED_OPS:?}"
        )));
    }
    let anchor = chain_anchor(None, body);
    let ts_ns = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let row: (i64,) = sqlx::query_as(
        "INSERT INTO l1_audit_log
            (tenant_id, subject_id, op, path, body, prev_hash_hex, chain_anchor_hex, ts_ns)
         VALUES ($1, $2, $3, $4, $5, NULL, $6, $7)
         RETURNING seq",
    )
    .bind(tenant_id)
    .bind(subject_id)
    .bind(op)
    .bind(path)
    .bind(body)
    .bind(&anchor)
    .bind(ts_ns)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn genesis_anchor_is_sha256_of_body() {
        // SHA-256("hello") - the same vector auth::memory_bridge pins, proving byte-compatibility.
        assert_eq!(
            chain_anchor(None, "hello"),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn anchor_includes_prev_when_present_and_is_deterministic() {
        assert_ne!(chain_anchor(Some("ab"), "x"), chain_anchor(None, "x"));
        assert_eq!(chain_anchor(Some("ab"), "x"), chain_anchor(Some("ab"), "x"));
    }

    #[test]
    fn anchor_is_64_hex_chars() {
        assert_eq!(chain_anchor(None, "anything").len(), 64);
    }

    #[test]
    fn allowed_ops_match_the_l1_op_enum_check() {
        // The set must stay in lockstep with migration 0003's CHECK (op IN (...)). FR-MEMORY-121 relies on
        // 'view' being accepted so read-only interaction-events chain as view, not put.
        assert!(ALLOWED_OPS.contains(&"put"));
        assert!(ALLOWED_OPS.contains(&"view"));
        assert!(ALLOWED_OPS.contains(&"move"));
        assert!(ALLOWED_OPS.contains(&"delete"));
        assert!(!ALLOWED_OPS.contains(&"frobnicate"));
        assert_eq!(ALLOWED_OPS.len(), 4);
    }
}
