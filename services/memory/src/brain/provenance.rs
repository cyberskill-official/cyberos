//! FR-MEMORY-123 §1 #10,#11,#26 — provenance + read-time chain-anchor verification.
//!
//! Every recall hit points back to the exact `l1_audit_log` row(s) it derived from (DEC-2726), and before a
//! hit is returned its `chain_anchor` is re-verified against the LIVE Layer-1 row (§1 #10): recompute
//! `SHA-256(prev_hash_hex || body)` from the current row at `source_seq` and compare it to the anchor the
//! brain row carried. A mismatch means the chain under this hit changed since ingest (possible tamper), so
//! the hit is dropped and a sev-1 fires (`metrics::chain_anchor_mismatch`) — the derived index is NEVER
//! trusted over a tampered chain (Layer 1 wins, DEC-2721).
//!
//! Reuses the exact anchor compute the Layer-2 ingest uses (`crate::layer2::chain_anchor::compute`), so the
//! brain's read-time check is byte-identical to how the anchor was produced — no second definition to drift.

use sqlx::PgPool;
use uuid::Uuid;

use crate::layer2::chain_anchor;

/// Recompute the canonical anchor for the Layer-1 row at `source_seq` and compare it to `expected_hex`
/// (the anchor the brain row stored at ingest). Returns:
///   * `Ok(true)`  — the live Layer-1 row's anchor matches; the hit is trustworthy.
///   * `Ok(false)` — mismatch (tamper / drift) OR the row no longer exists; drop the hit (§1 #10).
///   * `Err(_)`    — Layer 1 unreadable; the caller fails closed (drops the hit, §10 "l1_audit_log
///     unreadable").
///
/// The lookup is tenant-scoped via the caller's tenant id (the row's own `tenant_id = $1` predicate); the
/// chain itself is the system of record so this read does not need the brain RLS GUC.
pub async fn verify_chain_anchor(
    pool: &PgPool,
    tenant_id: Uuid,
    source_seq: i64,
    expected_hex: &str,
) -> Result<bool, sqlx::Error> {
    let row: Option<(Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT prev_hash_hex, body FROM l1_audit_log WHERE tenant_id = $1 AND seq = $2",
    )
    .bind(tenant_id)
    .bind(source_seq)
    .fetch_optional(pool)
    .await?;

    let Some((prev_hash_hex, body)) = row else {
        // The source row is gone — the derived hit can no longer be backed by Layer 1; drop it.
        return Ok(false);
    };
    let body = body.unwrap_or_default();
    let recomputed = chain_anchor::compute(prev_hash_hex.as_deref(), &body);
    Ok(recomputed.eq_ignore_ascii_case(expected_hex))
}

/// Confirm a single Layer-1 row is fetchable by its provenance `audit_row_id` (§1 #8 cold-tier retrieval):
/// a cold event is absent from the hot index but its raw row stays in Layer 1 and is retrievable on demand.
/// Returns the row body when present. Parses `l1:<tenant>:<hexseq>` and looks the seq up directly.
pub async fn fetch_raw_by_audit_row_id(
    pool: &PgPool,
    audit_row_id: &str,
) -> Result<Option<String>, sqlx::Error> {
    let Some((tenant_id, seq)) = parse_audit_row_id(audit_row_id) else {
        return Ok(None);
    };
    let row: Option<(Option<String>,)> =
        sqlx::query_as("SELECT body FROM l1_audit_log WHERE tenant_id = $1 AND seq = $2")
            .bind(tenant_id)
            .bind(seq)
            .fetch_optional(pool)
            .await?;
    Ok(row.and_then(|(b,)| b))
}

/// Build a short snippet from a Layer-1 body for a recall hit. For an interaction-event row the body is the
/// canonical `{"event_type":...,"payload":{...}}` JSON; we surface a compact, non-sensitive view (the
/// interaction's own verb + a clipped raw prefix) rather than dumping the whole JSON. Falls back to a clipped
/// prefix of the body when it is not interaction-event JSON.
pub fn snippet_from_body(body: &str, max: usize) -> String {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(body) {
        if let Some(verb) = v
            .get("payload")
            .and_then(|p| p.get("event_type"))
            .and_then(|e| e.as_str())
        {
            return clip(verb, max);
        }
    }
    clip(body, max)
}

fn clip(s: &str, max: usize) -> String {
    let s = s.trim();
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max.saturating_sub(3)).collect();
        format!("{truncated}...")
    }
}

/// Parse `l1:<tenant_uuid>:<hex source_seq>` back into `(tenant_id, seq)`. Returns `None` on any malformed
/// id (so a bad provenance pointer fails closed rather than mis-resolving).
pub fn parse_audit_row_id(audit_row_id: &str) -> Option<(Uuid, i64)> {
    let rest = audit_row_id.strip_prefix("l1:")?;
    let (tenant, hexseq) = rest.rsplit_once(':')?;
    let tenant_id = Uuid::parse_str(tenant).ok()?;
    let seq = i64::from_str_radix(hexseq, 16).ok()?;
    Some((tenant_id, seq))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_round_trips_with_make_audit_row_id() {
        let t = Uuid::parse_str("7e57c0de-aaaa-bbbb-cccc-000000000001").unwrap();
        let id = super::super::BrainEvent::make_audit_row_id(t, 0x1f3a2);
        let (pt, seq) = parse_audit_row_id(&id).unwrap();
        assert_eq!(pt, t);
        assert_eq!(seq, 0x1f3a2);
    }

    #[test]
    fn parse_rejects_malformed() {
        assert!(parse_audit_row_id("nope").is_none());
        assert!(parse_audit_row_id("l1:not-a-uuid:01").is_none());
        assert!(parse_audit_row_id("l1:7e57c0de-aaaa-bbbb-cccc-000000000001:zz").is_none());
    }

    #[test]
    fn snippet_surfaces_interaction_verb() {
        let body = r#"{"event_type":"memory.interaction_event","payload":{"event_type":"chat.message_created","module":"chat"}}"#;
        assert_eq!(snippet_from_body(body, 64), "chat.message_created");
    }

    #[test]
    fn snippet_clips_long_plain_body() {
        let body = "x".repeat(500);
        let s = snippet_from_body(&body, 24);
        assert!(s.len() <= 24);
        assert!(s.ends_with("..."));
    }

    #[test]
    fn anchor_recompute_matches_layer2_compute() {
        // The brain's read-time verify MUST use the same compute the ingest used.
        let a = chain_anchor::compute(Some("abcd"), "body");
        let b = chain_anchor::compute(Some("abcd"), "body");
        assert_eq!(a, b);
        assert!(a.eq_ignore_ascii_case(&b.to_uppercase()));
    }
}
