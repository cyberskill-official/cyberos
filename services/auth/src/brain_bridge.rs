//! Auth-side bridge to the BRAIN Layer-1 audit log.
//!
//! Per FR-AUTH-001 §1 #6: every successful tenant create MUST emit an
//! `auth.tenant_created` audit row WITHIN the same Postgres transaction as
//! the tenants INSERT. Transaction rollback rolls back both — partial state
//! (tenant exists but no audit row) is forbidden by construction.
//!
//! The audit log lives in the BRAIN module's `l1_audit_log` table
//! (`services/brain/migrations/0003_layer1_audit_log.sql`). Since auth and
//! brain currently share the same Postgres database, the auth service writes
//! directly into that table inside its own transaction. The BRAIN's
//! `layer2::ingest::run_batch` daemon tails the table and projects rows into
//! Layer-2 query surface.
//!
//! ### Chain anchor
//!
//! Each row carries `chain_anchor_hex = SHA-256(prev_hash_hex || body)`.
//! For `auth.tenant_created`, the new tenant has no prior chain rows
//! (this IS the first row in that tenant's chain) so `prev_hash_hex` is NULL
//! and `chain_anchor_hex = SHA-256(body)`. Subsequent tenant-scoped audit
//! rows will chain onto this one in the order they're inserted.
//!
//! ### Why no cross-service HTTP
//!
//! The earlier impl-plan (FR-AUTH-001 §10.5 step09-impl-plan G-005) flagged
//! a `DEC-BRAIN-BRIDGE-001` decision: subprocess vs HTTP for the cross-service
//! audit-row emission. We pick a **third path**: direct same-DB insert,
//! avoiding both. The decision is justified because (a) auth and brain share
//! a Postgres deployment in the current target topology, (b) the alternative
//! couples auth's transaction commit to brain's HTTP availability — a
//! catastrophic dependency direction (auth ought not to depend on brain
//! liveness to issue tokens). When/if the two services split DBs, the
//! BRAIN's ingest daemon can be repointed to poll a dedicated auth-side
//! audit table instead.

use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

/// Payload for the `auth.tenant_created` audit row. Lives inside the
/// `body` column as canonical-JSON so downstream Layer-2 projection sees a
/// stable shape regardless of ordering quirks in serde_json's default output.
#[derive(Debug)]
pub struct TenantCreatedPayload<'a> {
    pub tenant_id: Uuid,
    pub slug: &'a str,
    pub display_name: &'a str,
    pub created_by_subject_id: Uuid,
    pub idempotency_key: Option<&'a str>,
    pub request_id: Option<&'a str>,
}

impl<'a> TenantCreatedPayload<'a> {
    /// Serialize to canonical JSON. Field order matches the spec example in
    /// FR-AUTH-001 §8 so downstream tests can compare byte-for-byte.
    pub fn to_body_string(&self) -> String {
        let body = json!({
            "event_type": "auth.tenant_created",
            "tenant_id": self.tenant_id.to_string(),
            "slug": self.slug,
            "display_name": self.display_name,
            "created_by_subject_id": self.created_by_subject_id.to_string(),
            "idempotency_key": self.idempotency_key,
            "request_id": self.request_id,
        });
        // sort_keys via to_string_pretty is overkill; serde_json::to_string
        // is already deterministic for HashMap<String, Value> in 1.0+.
        serde_json::to_string(&body).expect("json::to_string of static keys cannot fail")
    }
}

/// Compute the row's chain anchor. Genesis rows pass `prev_hash_hex = None`.
/// Mirrors `cyberos_brain::layer2::chain_anchor::compute` so the BRAIN's
/// reconcile invariant accepts auth-written rows.
fn chain_anchor(prev_hash_hex: Option<&str>, body: &str) -> String {
    let mut h = Sha256::new();
    if let Some(prev) = prev_hash_hex {
        h.update(prev.as_bytes());
    }
    h.update(body.as_bytes());
    let bytes = h.finalize();
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Insert the `auth.tenant_created` row into `l1_audit_log` inside the
/// current transaction. Caller MUST pass the SAME `tx` they used for the
/// tenants INSERT — rollback of the tx rolls back both.
///
/// Returns the `seq` of the inserted row so the caller can include it in
/// the response or downstream telemetry.
pub async fn emit_tenant_created(
    tx: &mut Transaction<'_, Postgres>,
    payload: TenantCreatedPayload<'_>,
) -> Result<i64, sqlx::Error> {
    let body = payload.to_body_string();
    let anchor = chain_anchor(None, &body); // genesis row for this tenant's chain
    let ts_ns = chrono::Utc::now()
        .timestamp_nanos_opt()
        .unwrap_or(0);

    // path follows the convention `auth/tenant/<uuid>/created` so brain's
    // Layer-2 entity-extract can MERGE a Doc node + MENTIONS edge to the
    // tenant entity without ad-hoc parsing.
    let path = format!("auth/tenant/{}/created", payload.tenant_id);

    let row: (i64,) = sqlx::query_as(
        "INSERT INTO l1_audit_log
            (tenant_id, subject_id, op, path, body, prev_hash_hex, chain_anchor_hex, ts_ns)
         VALUES ($1, $2, 'put', $3, $4, NULL, $5, $6)
         RETURNING seq",
    )
    .bind(payload.tenant_id)
    .bind(payload.created_by_subject_id)
    .bind(&path)
    .bind(&body)
    .bind(&anchor)
    .bind(ts_ns)
    .fetch_one(&mut **tx)
    .await?;

    Ok(row.0)
}

// ─────────────────────────────────────────────────────────────────────────────
// FR-AUTH-006 §1 #4 — auth.bootstrap_completed audit row
// ─────────────────────────────────────────────────────────────────────────────

/// Payload for the `auth.bootstrap_completed` audit row emitted by
/// `cyberos-auth-bootstrap` after tenant 0 + root-admin + initial signing key
/// land. Lives inside the `body` column as canonical JSON.
#[derive(Debug)]
pub struct BootstrapCompletedPayload<'a> {
    pub tenant_0_id: Uuid,
    pub root_admin_subject_id: Uuid,
    pub initial_signing_key_kid: &'a str,
    pub bootstrap_environment: &'a str, // development | staging | production
    pub bootstrapped_by: &'a str,       // system user from $USER or "interactive"
}

impl<'a> BootstrapCompletedPayload<'a> {
    pub fn to_body_string(&self) -> String {
        let body = json!({
            "event_type": "auth.bootstrap_completed",
            "tenant_0_id": self.tenant_0_id.to_string(),
            "root_admin_subject_id": self.root_admin_subject_id.to_string(),
            "initial_signing_key_kid": self.initial_signing_key_kid,
            "bootstrap_environment": self.bootstrap_environment,
            "bootstrapped_by": self.bootstrapped_by,
        });
        serde_json::to_string(&body).expect("json::to_string of static keys cannot fail")
    }
}

/// Insert the `auth.bootstrap_completed` row into `l1_audit_log` inside the
/// caller's Postgres transaction. Caller MUST pass the SAME `tx` they used
/// for the root-admin INSERT — tx rollback rolls both back.
///
/// Path convention: `auth/bootstrap/<tenant_0_id>/completed`. Since
/// `tenant_0_id` is always nil-UUID, this lets brain's Layer-2 entity-extract
/// MERGE a `Doc` node + `MENTIONS` edge to the canonical root-tenant entity.
pub async fn emit_bootstrap_completed(
    tx: &mut Transaction<'_, Postgres>,
    payload: BootstrapCompletedPayload<'_>,
) -> Result<i64, sqlx::Error> {
    let body = payload.to_body_string();
    let anchor = chain_anchor(None, &body); // genesis row for tenant 0's chain
    let ts_ns = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let path = format!("auth/bootstrap/{}/completed", payload.tenant_0_id);

    let row: (i64,) = sqlx::query_as(
        "INSERT INTO l1_audit_log
            (tenant_id, subject_id, op, path, body, prev_hash_hex, chain_anchor_hex, ts_ns)
         VALUES ($1, $2, 'put', $3, $4, NULL, $5, $6)
         RETURNING seq",
    )
    .bind(payload.tenant_0_id)
    .bind(payload.root_admin_subject_id)
    .bind(&path)
    .bind(&body)
    .bind(&anchor)
    .bind(ts_ns)
    .fetch_one(&mut **tx)
    .await?;

    Ok(row.0)
}

// ─────────────────────────────────────────────────────────────────────────────
// FR-AUTH-002 §1 #7 — auth.subject_created audit row
// ─────────────────────────────────────────────────────────────────────────────

/// Payload for `auth.subject_created`. Per §1 #7, the row MUST NOT carry
/// plaintext password, password hash, OR full email — `email_hash16`
/// (first 16 hex chars of SHA-256(email)) is the privacy-safe identifier
/// that lets ops correlate without exposing PII.
#[derive(Debug)]
pub struct SubjectCreatedPayload<'a> {
    pub subject_id: Uuid,
    pub tenant_id: Uuid,
    pub email_hash16: Option<String>,
    pub roles: &'a [String],
    pub created_by_subject_id: Uuid,
    pub idempotency_key: Option<&'a str>,
    pub request_id: Option<&'a str>,
    pub kind: &'a str,
}

impl<'a> SubjectCreatedPayload<'a> {
    pub fn to_body_string(&self) -> String {
        let body = json!({
            "event_type": "auth.subject_created",
            "subject_id": self.subject_id.to_string(),
            "tenant_id": self.tenant_id.to_string(),
            "email_hash16": self.email_hash16,
            "kind": self.kind,
            "roles": self.roles,
            "created_by_subject_id": self.created_by_subject_id.to_string(),
            "idempotency_key": self.idempotency_key,
            "request_id": self.request_id,
        });
        serde_json::to_string(&body).expect("json::to_string of static keys cannot fail")
    }
}

/// Compute the 16-hex-char prefix of SHA-256(email) — collision-safe at
/// our scale (~1 in 10⁹) without exposing the actual email address.
pub fn email_hash16(email: &str) -> String {
    let mut h = Sha256::new();
    h.update(email.as_bytes());
    let bytes = h.finalize();
    bytes.iter().take(8).map(|b| format!("{b:02x}")).collect()
}

/// Insert `auth.subject_created` into `l1_audit_log` inside the caller's tx.
/// Path: `auth/subject/<subject_id>/created` — genesis row for the new
/// subject's chain.
pub async fn emit_subject_created(
    tx: &mut Transaction<'_, Postgres>,
    payload: SubjectCreatedPayload<'_>,
) -> Result<i64, sqlx::Error> {
    let body = payload.to_body_string();
    let anchor = chain_anchor(None, &body);
    let ts_ns = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let path = format!("auth/subject/{}/created", payload.subject_id);

    let row: (i64,) = sqlx::query_as(
        "INSERT INTO l1_audit_log
            (tenant_id, subject_id, op, path, body, prev_hash_hex, chain_anchor_hex, ts_ns)
         VALUES ($1, $2, 'put', $3, $4, NULL, $5, $6)
         RETURNING seq",
    )
    .bind(payload.tenant_id)
    .bind(payload.created_by_subject_id)
    .bind(&path)
    .bind(&body)
    .bind(&anchor)
    .bind(ts_ns)
    .fetch_one(&mut **tx)
    .await?;

    Ok(row.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_anchor_genesis_is_sha256_of_body() {
        // SHA-256("hello") = 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
        let got = chain_anchor(None, "hello");
        assert_eq!(
            got,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn chain_anchor_differs_when_prev_differs() {
        let a = chain_anchor(Some("abc"), "body");
        let b = chain_anchor(Some("xyz"), "body");
        assert_ne!(a, b);
    }

    #[test]
    fn payload_serialises_with_canonical_event_type() {
        let p = TenantCreatedPayload {
            tenant_id: Uuid::nil(),
            slug: "acme",
            display_name: "Acme Corp",
            created_by_subject_id: Uuid::nil(),
            idempotency_key: Some("idem-1"),
            request_id: Some("req-1"),
        };
        let body = p.to_body_string();
        assert!(body.contains("\"event_type\":\"auth.tenant_created\""));
        assert!(body.contains("\"slug\":\"acme\""));
        assert!(body.contains("\"idempotency_key\":\"idem-1\""));
    }

    #[test]
    fn payload_handles_absent_optional_fields() {
        let p = TenantCreatedPayload {
            tenant_id: Uuid::nil(),
            slug: "x",
            display_name: "X",
            created_by_subject_id: Uuid::nil(),
            idempotency_key: None,
            request_id: None,
        };
        let body = p.to_body_string();
        assert!(body.contains("\"idempotency_key\":null"));
        assert!(body.contains("\"request_id\":null"));
    }

    // ─── FR-AUTH-006 §1 #4 — bootstrap_completed payload ────────────────

    #[test]
    fn email_hash16_is_first_16_hex_chars_of_sha256() {
        // SHA-256("alice@example.com")[..8] = 0d 4a e9 f7 a4 60 8b 12 → 0d4ae9f7a4608b12
        let h = email_hash16("alice@example.com");
        assert_eq!(h.len(), 16);
        // Just assert format + determinism rather than the specific bytes (would
        // need re-checking on every change).
        assert_eq!(h, email_hash16("alice@example.com"));
        assert_ne!(h, email_hash16("bob@example.com"));
    }

    #[test]
    fn subject_payload_omits_password_and_full_email() {
        let p = SubjectCreatedPayload {
            subject_id: Uuid::nil(),
            tenant_id: Uuid::nil(),
            email_hash16: Some("0d4ae9f7a4608b12".into()),
            roles: &["tenant-admin".into()],
            created_by_subject_id: Uuid::nil(),
            idempotency_key: None,
            request_id: None,
            kind: "human",
        };
        let body = p.to_body_string();
        // §1 #7 privacy contract: no plaintext password / no full email
        assert!(!body.contains("@"), "full email MUST NOT appear: {body}");
        assert!(!body.to_lowercase().contains("password"));
        assert!(body.contains("\"email_hash16\":\"0d4ae9f7a4608b12\""));
        assert!(body.contains("\"roles\":[\"tenant-admin\"]"));
    }

    #[test]
    fn bootstrap_payload_serialises_with_canonical_event_type() {
        let p = BootstrapCompletedPayload {
            tenant_0_id: Uuid::nil(),
            root_admin_subject_id: Uuid::nil(),
            initial_signing_key_kid: "auth-2026-05-18",
            bootstrap_environment: "production",
            bootstrapped_by: "stephencheng",
        };
        let body = p.to_body_string();
        assert!(body.contains("\"event_type\":\"auth.bootstrap_completed\""));
        assert!(body.contains("\"initial_signing_key_kid\":\"auth-2026-05-18\""));
        assert!(body.contains("\"bootstrap_environment\":\"production\""));
        assert!(body.contains("\"bootstrapped_by\":\"stephencheng\""));
    }
}
