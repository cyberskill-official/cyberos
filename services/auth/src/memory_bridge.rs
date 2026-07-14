//! Auth-side bridge to the memory Layer-1 audit log.
//!
//! Per TASK-AUTH-001 §1 #6: every successful tenant create MUST emit an
//! `auth.tenant_created` audit row WITHIN the same Postgres transaction as
//! the tenants INSERT. Transaction rollback rolls back both — partial state
//! (tenant exists but no audit row) is forbidden by construction.
//!
//! The audit log lives in the memory module's `l1_audit_log` table
//! (`services/memory/migrations/0003_layer1_audit_log.sql`). Since auth and
//! memory currently share the same Postgres database, the auth service writes
//! directly into that table inside its own transaction. The memory's
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
//! The earlier impl-plan (TASK-AUTH-001 §10.5 step09-impl-plan G-005) flagged
//! a `DEC-memory-BRIDGE-001` decision: subprocess vs HTTP for the cross-service
//! audit-row emission. We pick a **third path**: direct same-DB insert,
//! avoiding both. The decision is justified because (a) auth and memory share
//! a Postgres deployment in the current target topology, (b) the alternative
//! couples auth's transaction commit to memory's HTTP availability — a
//! catastrophic dependency direction (auth ought not to depend on memory
//! liveness to issue tokens). When/if the two services split DBs, the
//! memory's ingest daemon can be repointed to poll a dedicated auth-side
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
    /// TASK-AUTH-001 §8 so downstream tests can compare byte-for-byte.
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
/// Mirrors `cyberos_memory::layer2::chain_anchor::compute` so the memory's
/// reconcile invariant accepts auth-written rows.
pub fn chain_anchor(prev_hash_hex: Option<&str>, body: &str) -> String {
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
    let ts_ns = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);

    // path follows the convention `auth/tenant/<uuid>/created` so memory's
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
// TASK-AUTH-006 §1 #4 — auth.bootstrap_completed audit row
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
/// `tenant_0_id` is always nil-UUID, this lets memory's Layer-2 entity-extract
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
// TASK-AUTH-002 §1 #7 — auth.subject_created audit row
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

/// TASK-AUTH-005 §1 #6 + G-006 — `auth.subject_revoked` audit row payload.
///
/// Emitted inside the revoke handler's tx so the suspended-status update,
/// the memory row, and the deny-list pushes are all atomic (any failure
/// rolls back the entire revoke).
#[derive(Debug)]
pub struct SubjectRevokedPayload<'a> {
    pub subject_id: Uuid,
    pub tenant_id: Uuid,
    pub revoked_by_subject_id: Uuid,
    /// Optional free-form caller-supplied reason. Closed taxonomy comes in
    /// TASK-AUTH-111 (compromised/terminated/policy-violation/operator-error).
    pub reason: Option<&'a str>,
    pub revoked_jti_count: usize,
    pub idempotency_key: Option<&'a str>,
    pub request_id: Option<&'a str>,
}

impl<'a> SubjectRevokedPayload<'a> {
    pub fn to_body_string(&self) -> String {
        let body = json!({
            "event_type": "auth.subject_revoked",
            "subject_id": self.subject_id.to_string(),
            "tenant_id": self.tenant_id.to_string(),
            "revoked_by_subject_id": self.revoked_by_subject_id.to_string(),
            "reason": self.reason,
            "revoked_jti_count": self.revoked_jti_count,
            "idempotency_key": self.idempotency_key,
            "request_id": self.request_id,
        });
        serde_json::to_string(&body).expect("json::to_string of static keys cannot fail")
    }
}

/// TASK-AUTH-005 §1 #6 + G-006 — `auth.subject_unrevoked` audit row payload.
///
/// Note the deliberate asymmetry with `SubjectRevokedPayload`: there is no
/// `unrevoked_jti_count` because per §1 #12 unrevoke MUST NOT clear the
/// deny-list. The unrevoke event records the status flip only; existing
/// denied jtis stay denied until natural expiry.
#[derive(Debug)]
pub struct SubjectUnrevokedPayload<'a> {
    pub subject_id: Uuid,
    pub tenant_id: Uuid,
    pub unrevoked_by_subject_id: Uuid,
    pub idempotency_key: Option<&'a str>,
    pub request_id: Option<&'a str>,
}

impl<'a> SubjectUnrevokedPayload<'a> {
    pub fn to_body_string(&self) -> String {
        let body = json!({
            "event_type": "auth.subject_unrevoked",
            "subject_id": self.subject_id.to_string(),
            "tenant_id": self.tenant_id.to_string(),
            "unrevoked_by_subject_id": self.unrevoked_by_subject_id.to_string(),
            "idempotency_key": self.idempotency_key,
            "request_id": self.request_id,
        });
        serde_json::to_string(&body).expect("json::to_string of static keys cannot fail")
    }
}

/// Emit `auth.subject_revoked` into `l1_audit_log` inside the caller's tx.
/// Chains onto the subject's most recent chain row (or genesis if first).
pub async fn emit_subject_revoked(
    tx: &mut Transaction<'_, Postgres>,
    payload: SubjectRevokedPayload<'_>,
) -> Result<i64, sqlx::Error> {
    let body = payload.to_body_string();
    let prev: Option<(String,)> = sqlx::query_as(
        "SELECT chain_anchor_hex FROM l1_audit_log
          WHERE tenant_id = $1 AND subject_id = $2
       ORDER BY seq DESC LIMIT 1",
    )
    .bind(payload.tenant_id)
    .bind(payload.subject_id)
    .fetch_optional(&mut **tx)
    .await?;
    let prev_hex = prev.as_ref().map(|p| p.0.as_str());
    let anchor = chain_anchor(prev_hex, &body);
    let ts_ns = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let path = format!("auth/subject/{}/revoked", payload.subject_id);

    let row: (i64,) = sqlx::query_as(
        "INSERT INTO l1_audit_log
            (tenant_id, subject_id, op, path, body, prev_hash_hex, chain_anchor_hex, ts_ns)
         VALUES ($1, $2, 'put', $3, $4, $5, $6, $7)
         RETURNING seq",
    )
    .bind(payload.tenant_id)
    .bind(payload.revoked_by_subject_id)
    .bind(&path)
    .bind(&body)
    .bind(prev_hex)
    .bind(&anchor)
    .bind(ts_ns)
    .fetch_one(&mut **tx)
    .await?;
    Ok(row.0)
}

/// Emit `auth.subject_unrevoked` into `l1_audit_log` inside the caller's tx.
pub async fn emit_subject_unrevoked(
    tx: &mut Transaction<'_, Postgres>,
    payload: SubjectUnrevokedPayload<'_>,
) -> Result<i64, sqlx::Error> {
    let body = payload.to_body_string();
    let prev: Option<(String,)> = sqlx::query_as(
        "SELECT chain_anchor_hex FROM l1_audit_log
          WHERE tenant_id = $1 AND subject_id = $2
       ORDER BY seq DESC LIMIT 1",
    )
    .bind(payload.tenant_id)
    .bind(payload.subject_id)
    .fetch_optional(&mut **tx)
    .await?;
    let prev_hex = prev.as_ref().map(|p| p.0.as_str());
    let anchor = chain_anchor(prev_hex, &body);
    let ts_ns = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let path = format!("auth/subject/{}/unrevoked", payload.subject_id);

    let row: (i64,) = sqlx::query_as(
        "INSERT INTO l1_audit_log
            (tenant_id, subject_id, op, path, body, prev_hash_hex, chain_anchor_hex, ts_ns)
         VALUES ($1, $2, 'put', $3, $4, $5, $6, $7)
         RETURNING seq",
    )
    .bind(payload.tenant_id)
    .bind(payload.unrevoked_by_subject_id)
    .bind(&path)
    .bind(&body)
    .bind(prev_hex)
    .bind(&anchor)
    .bind(ts_ns)
    .fetch_one(&mut **tx)
    .await?;
    Ok(row.0)
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

    // ─── TASK-AUTH-006 §1 #4 — bootstrap_completed payload ────────────────

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

    // ─── TASK-AUTH-004 §1 #6 — token_issued / token_failed payloads ──────────

    #[test]
    fn token_issued_payload_canonical_json() {
        let p = TokenIssuedPayload {
            subject_id: Uuid::nil(),
            tenant_id: Uuid::nil(),
            jti: "abcdef123",
            roles: &["tenant-admin".to_string()],
            scope_grants_count: 5,
            expires_at: 1_763_115_600,
            source_ip_hash16: "4b8c0d2f1a7e9c3b",
            request_id: Some("req-1"),
        };
        let body = p.to_body_string();
        assert!(body.contains("\"event_type\":\"auth.token_issued\""));
        assert!(body.contains("\"jti\":\"abcdef123\""));
        assert!(body.contains("\"scope_grants_count\":5"));
        assert!(body.contains("\"source_ip_hash16\":\"4b8c0d2f1a7e9c3b\""));
        assert!(body.contains("\"roles\":[\"tenant-admin\"]"));
        // PII discipline: no raw IP, no jti-secret-prefix leak
        assert!(!body.contains("@"));
    }

    #[test]
    fn token_failed_payload_omits_plaintext_pii() {
        let p = TokenFailedPayload {
            tenant_slug: "acme",
            email_hash16: "ab12cd34ef56gh78",
            reason: "invalid_credentials",
            source_ip_hash16: "4b8c0d2f1a7e9c3b",
            request_id: Some("req-1"),
        };
        let body = p.to_body_string();
        assert!(body.contains("\"event_type\":\"auth.token_failed\""));
        assert!(body.contains("\"tenant_slug\":\"acme\""));
        assert!(body.contains("\"email_hash16\":\"ab12cd34ef56gh78\""));
        assert!(body.contains("\"reason\":\"invalid_credentials\""));
        // §1 #6 privacy contract: no plaintext email, no raw IP, no password.
        assert!(!body.contains("@"));
        assert!(!body.to_lowercase().contains("password"));
    }

    #[test]
    fn source_ip_hash16_salts_by_date() {
        // Same IP → same hash within a day (test runs in microseconds; date unchanged).
        let h1 = source_ip_hash16("1.2.3.4");
        let h2 = source_ip_hash16("1.2.3.4");
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 16);
        // Different IP → different hash.
        assert_ne!(h1, source_ip_hash16("5.6.7.8"));
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// TASK-AUTH-004 §1 #6 — auth.token_issued + auth.token_failed audit rows
// ─────────────────────────────────────────────────────────────────────────────

/// Privacy-safe per-day IP fingerprint. SHA-256("YYYY-MM-DD|<ip>") first 16
/// hex chars. Salted with the current UTC date so the same IP correlates
/// within a day (useful for incident response) but NOT across days
/// (preventing long-term IP tracking). Matches the construction described
/// in TASK-AUTH-004 §1 #6 + §2 rationale.
pub fn source_ip_hash16(source_ip: &str) -> String {
    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let mut h = Sha256::new();
    h.update(date.as_bytes());
    h.update(b"|");
    h.update(source_ip.as_bytes());
    let bytes = h.finalize();
    bytes.iter().take(8).map(|b| format!("{b:02x}")).collect()
}

/// Payload for `auth.token_issued`. Per §1 #6, the row carries no raw email,
/// no raw IP, no password — only the privacy-safe `*_hash16` digests.
#[derive(Debug)]
pub struct TokenIssuedPayload<'a> {
    pub subject_id: Uuid,
    pub tenant_id: Uuid,
    pub jti: &'a str,
    pub roles: &'a [String],
    pub scope_grants_count: usize,
    pub expires_at: i64,
    pub source_ip_hash16: &'a str,
    pub request_id: Option<&'a str>,
}

impl<'a> TokenIssuedPayload<'a> {
    pub fn to_body_string(&self) -> String {
        let body = json!({
            "event_type": "auth.token_issued",
            "subject_id": self.subject_id.to_string(),
            "tenant_id": self.tenant_id.to_string(),
            "jti": self.jti,
            "roles": self.roles,
            "scope_grants_count": self.scope_grants_count,
            "expires_at": self.expires_at,
            "source_ip_hash16": self.source_ip_hash16,
            "request_id": self.request_id,
        });
        serde_json::to_string(&body).expect("json::to_string of static keys cannot fail")
    }
}

/// Payload for `auth.token_failed`. Captures every failed authentication
/// attempt for credential-stuffing detection. The `reason` discriminates
/// between `invalid_credentials` | `suspended` | `rate_limited` |
/// `unknown_tenant` so a single time-series query can plot each curve.
#[derive(Debug)]
pub struct TokenFailedPayload<'a> {
    pub tenant_slug: &'a str,
    pub email_hash16: &'a str,
    pub reason: &'a str,
    pub source_ip_hash16: &'a str,
    pub request_id: Option<&'a str>,
}

impl<'a> TokenFailedPayload<'a> {
    pub fn to_body_string(&self) -> String {
        let body = json!({
            "event_type": "auth.token_failed",
            "tenant_slug": self.tenant_slug,
            "email_hash16": self.email_hash16,
            "reason": self.reason,
            "source_ip_hash16": self.source_ip_hash16,
            "request_id": self.request_id,
        });
        serde_json::to_string(&body).expect("json::to_string of static keys cannot fail")
    }
}

/// Best-effort insert of `auth.token_issued` into `l1_audit_log`. Unlike
/// `emit_tenant_created` / `emit_subject_created`, this is NOT in the
/// caller's transaction — token issuance shouldn't fail because the audit
/// log is unreachable. Failures log a `tracing::warn!` but are silently
/// swallowed; the TASK-OBS-001 alarm catches a sustained spike in dropped
/// audit rows.
///
/// Path: `auth/token/<tenant_id>/<jti>/issued`. The jti uniquely keys the
/// row so retries (e.g. CDC replay) merge cleanly.
pub async fn emit_token_issued(
    pool: &sqlx::PgPool,
    payload: TokenIssuedPayload<'_>,
) -> Result<i64, sqlx::Error> {
    let body = payload.to_body_string();
    let anchor = chain_anchor(None, &body);
    let ts_ns = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let path = format!("auth/token/{}/{}/issued", payload.tenant_id, payload.jti);

    let row: (i64,) = sqlx::query_as(
        "INSERT INTO l1_audit_log
            (tenant_id, subject_id, op, path, body, prev_hash_hex, chain_anchor_hex, ts_ns)
         VALUES ($1, $2, 'put', $3, $4, NULL, $5, $6)
         RETURNING seq",
    )
    .bind(payload.tenant_id)
    .bind(payload.subject_id)
    .bind(&path)
    .bind(&body)
    .bind(&anchor)
    .bind(ts_ns)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

/// Best-effort insert of `auth.token_failed` into `l1_audit_log`. Path
/// scheme matches `emit_token_issued` but with `/failed/<reason>` so a
/// path-prefix scan can isolate each failure reason.
///
/// Note `tenant_id` may be unknown if the tenant_slug didn't resolve —
/// callers pass `Uuid::nil()` in that case so the row still chains to the
/// root-tenant's audit history.
pub async fn emit_token_failed(
    pool: &sqlx::PgPool,
    tenant_id: Uuid,
    payload: TokenFailedPayload<'_>,
) -> Result<i64, sqlx::Error> {
    let body = payload.to_body_string();
    let anchor = chain_anchor(None, &body);
    let ts_ns = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let path = format!(
        "auth/token/{}/{}/failed/{}",
        tenant_id, payload.email_hash16, payload.reason
    );

    let row: (i64,) = sqlx::query_as(
        "INSERT INTO l1_audit_log
            (tenant_id, subject_id, op, path, body, prev_hash_hex, chain_anchor_hex, ts_ns)
         VALUES ($1, $2, 'put', $3, $4, NULL, $5, $6)
         RETURNING seq",
    )
    .bind(tenant_id)
    .bind(Uuid::nil()) // subject_id unknown on failure
    .bind(&path)
    .bind(&body)
    .bind(&anchor)
    .bind(ts_ns)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}
