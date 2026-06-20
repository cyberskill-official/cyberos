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

/// Best-effort insert of a genesis audit row into `l1_audit_log` (`op = 'put'`, `prev_hash_hex = NULL`).
/// Returns the new `seq`. The caller decides what to do on error - the obs callers log and swallow so the
/// alert route or the view response still completes (the FR's best-effort contract).
pub async fn emit_genesis(
    pool: &PgPool,
    tenant_id: Uuid,
    subject_id: Uuid,
    path: &str,
    body: &str,
) -> Result<i64, sqlx::Error> {
    let anchor = chain_anchor(None, body);
    let ts_ns = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let row: (i64,) = sqlx::query_as(
        "INSERT INTO l1_audit_log
            (tenant_id, subject_id, op, path, body, prev_hash_hex, chain_anchor_hex, ts_ns)
         VALUES ($1, $2, 'put', $3, $4, NULL, $5, $6)
         RETURNING seq",
    )
    .bind(tenant_id)
    .bind(subject_id)
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
}
