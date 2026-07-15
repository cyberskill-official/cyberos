//! TASK-AUTH-110 §1 #22 + DEC-2494 - the 7 OIDC-provider memory-audit payloads.
//!
//! Mirrors the `crate::memory_bridge` pattern: each payload is a borrow-only
//! struct with `to_body_string()` returning canonical JSON for the l1 audit
//! chain. Raw internal UUIDs are stored as-is (matching `SubjectRevokedPayload`);
//! genuinely-PII fields (source IP) go through `memory_bridge::source_ip_hash16`.
//! The client_secret is never present in any row (DEC-2497). The async chain
//! writers (the `emit_*` functions that anchor onto the prior chain row) are
//! wired in the endpoints (slice 1b), same as `memory_bridge::emit_subject_revoked`.

use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::memory_bridge::source_ip_hash16;

/// `auth.op_authorize_issued` - a code was issued to an RP after a successful,
/// non-revoked authorize.
#[derive(Debug)]
pub struct OpAuthorizeIssuedPayload<'a> {
    pub tenant_id: Uuid,
    pub rp_client_id: &'a str,
    pub subject_id: Uuid,
}

impl OpAuthorizeIssuedPayload<'_> {
    pub fn to_body_string(&self) -> String {
        let body = json!({
            "event_type": "auth.op_authorize_issued",
            "tenant_id": self.tenant_id.to_string(),
            "rp_client_id": self.rp_client_id,
            "subject_id": self.subject_id.to_string(),
        });
        serde_json::to_string(&body).expect("json::to_string of static keys cannot fail")
    }
}

/// `auth.op_authorize_denied` - authorize refused a code. The kick records here
/// (reason = "subject_revoked"), as do unknown_client / redirect_mismatch.
#[derive(Debug)]
pub struct OpAuthorizeDeniedPayload<'a> {
    pub tenant_id: Uuid,
    pub rp_client_id: &'a str,
    pub subject_id: Option<Uuid>,
    pub reason: &'a str,
    pub source_ip: Option<&'a str>,
}

impl OpAuthorizeDeniedPayload<'_> {
    pub fn to_body_string(&self) -> String {
        let body = json!({
            "event_type": "auth.op_authorize_denied",
            "tenant_id": self.tenant_id.to_string(),
            "rp_client_id": self.rp_client_id,
            "subject_id": self.subject_id.map(|s| s.to_string()),
            "reason": self.reason,
            "source_ip_hash16": self.source_ip.map(source_ip_hash16),
        });
        serde_json::to_string(&body).expect("json::to_string of static keys cannot fail")
    }
}

/// `auth.op_token_issued` - the token endpoint minted an id_token + access_token.
#[derive(Debug)]
pub struct OpTokenIssuedPayload<'a> {
    pub tenant_id: Uuid,
    pub rp_client_id: &'a str,
    pub subject_id: Uuid,
    pub scope: &'a str,
}

impl OpTokenIssuedPayload<'_> {
    pub fn to_body_string(&self) -> String {
        let body = json!({
            "event_type": "auth.op_token_issued",
            "tenant_id": self.tenant_id.to_string(),
            "rp_client_id": self.rp_client_id,
            "subject_id": self.subject_id.to_string(),
            "scope": self.scope,
        });
        serde_json::to_string(&body).expect("json::to_string of static keys cannot fail")
    }
}

/// `auth.op_userinfo_served` - userinfo returned identity claims to a bearer.
#[derive(Debug)]
pub struct OpUserinfoServedPayload<'a> {
    pub tenant_id: Uuid,
    pub rp_client_id: &'a str,
    pub subject_id: Uuid,
}

impl OpUserinfoServedPayload<'_> {
    pub fn to_body_string(&self) -> String {
        let body = json!({
            "event_type": "auth.op_userinfo_served",
            "tenant_id": self.tenant_id.to_string(),
            "rp_client_id": self.rp_client_id,
            "subject_id": self.subject_id.to_string(),
        });
        serde_json::to_string(&body).expect("json::to_string of static keys cannot fail")
    }
}

/// `auth.op_rp_client_changed` - an RP client was created / updated / deleted.
/// NEVER carries the client_secret value (DEC-2497).
#[derive(Debug)]
pub struct OpRpClientChangedPayload<'a> {
    pub tenant_id: Uuid,
    pub rp_client_id: &'a str,
    pub changed_by_subject_id: Uuid,
    /// One of `created` | `updated` | `deleted`.
    pub change: &'a str,
}

impl OpRpClientChangedPayload<'_> {
    pub fn to_body_string(&self) -> String {
        let body = json!({
            "event_type": "auth.op_rp_client_changed",
            "tenant_id": self.tenant_id.to_string(),
            "rp_client_id": self.rp_client_id,
            "changed_by_subject_id": self.changed_by_subject_id.to_string(),
            "change": self.change,
        });
        serde_json::to_string(&body).expect("json::to_string of static keys cannot fail")
    }
}

/// `auth.op_sso_session_started` - a new AUTH SSO browser session was created.
#[derive(Debug)]
pub struct OpSsoSessionStartedPayload {
    pub tenant_id: Uuid,
    pub subject_id: Uuid,
    pub sso_session_id: Uuid,
}

impl OpSsoSessionStartedPayload {
    pub fn to_body_string(&self) -> String {
        let body = json!({
            "event_type": "auth.op_sso_session_started",
            "tenant_id": self.tenant_id.to_string(),
            "subject_id": self.subject_id.to_string(),
            "sso_session_id": self.sso_session_id.to_string(),
        });
        serde_json::to_string(&body).expect("json::to_string of static keys cannot fail")
    }
}

/// `auth.op_sso_session_revoked` - a subject revoke cascaded onto SSO sessions
/// (§1 #26); `revoked_count` rows were marked revoked.
#[derive(Debug)]
pub struct OpSsoSessionRevokedPayload {
    pub tenant_id: Uuid,
    pub subject_id: Uuid,
    pub revoked_count: u64,
}

impl OpSsoSessionRevokedPayload {
    pub fn to_body_string(&self) -> String {
        let body = json!({
            "event_type": "auth.op_sso_session_revoked",
            "tenant_id": self.tenant_id.to_string(),
            "subject_id": self.subject_id.to_string(),
            "revoked_count": self.revoked_count,
        });
        serde_json::to_string(&body).expect("json::to_string of static keys cannot fail")
    }
}

/// Best-effort: anchor an op_* audit row into the l1 chain in its own tx
/// (mirrors `crate::memory_bridge::emit_subject_revoked` and the codebase's
/// best-effort `oidc_login_history` insert). Keyed by `(tenant_id, subject_id)`.
/// Callers `let _ =` it, so a failed audit write never breaks the auth path.
pub async fn emit(
    pool: &PgPool,
    tenant_id: Uuid,
    subject_id: Uuid,
    path: &str,
    body: String,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await?;
    let prev: Option<(String,)> = sqlx::query_as(
        "SELECT chain_anchor_hex FROM l1_audit_log
          WHERE tenant_id = $1 AND subject_id = $2
       ORDER BY seq DESC LIMIT 1",
    )
    .bind(tenant_id)
    .bind(subject_id)
    .fetch_optional(&mut *tx)
    .await?;
    let prev_hex = prev.as_ref().map(|p| p.0.as_str());
    let anchor = crate::memory_bridge::chain_anchor(prev_hex, &body);
    let ts_ns = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    sqlx::query(
        "INSERT INTO l1_audit_log
            (tenant_id, subject_id, op, path, body, prev_hash_hex, chain_anchor_hex, ts_ns)
         VALUES ($1, $2, 'put', $3, $4, $5, $6, $7)",
    )
    .bind(tenant_id)
    .bind(subject_id)
    .bind(path)
    .bind(&body)
    .bind(prev_hex)
    .bind(&anchor)
    .bind(ts_ns)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn token_issued_has_event_type_and_no_secret() {
        let p = OpTokenIssuedPayload {
            tenant_id: Uuid::nil(),
            rp_client_id: "cyberos-chat",
            subject_id: Uuid::nil(),
            scope: "openid profile email",
        };
        let v: Value = serde_json::from_str(&p.to_body_string()).unwrap();
        assert_eq!(v["event_type"], "auth.op_token_issued");
        assert_eq!(v["rp_client_id"], "cyberos-chat");
        assert_eq!(v["scope"], "openid profile email");
        assert!(v.get("client_secret").is_none());
    }

    #[test]
    fn authorize_denied_carries_reason_and_optional_subject() {
        let p = OpAuthorizeDeniedPayload {
            tenant_id: Uuid::nil(),
            rp_client_id: "cyberos-chat",
            subject_id: None,
            reason: "subject_revoked",
            source_ip: Some("203.0.113.7"),
        };
        let v: Value = serde_json::from_str(&p.to_body_string()).unwrap();
        assert_eq!(v["event_type"], "auth.op_authorize_denied");
        assert_eq!(v["reason"], "subject_revoked");
        assert!(v["subject_id"].is_null());
        // IP is hashed, never raw.
        assert_ne!(v["source_ip_hash16"], "203.0.113.7");
        assert!(v["source_ip_hash16"].is_string());
    }

    #[test]
    fn rp_client_changed_never_serialises_a_secret() {
        let p = OpRpClientChangedPayload {
            tenant_id: Uuid::nil(),
            rp_client_id: "cyberos-chat",
            changed_by_subject_id: Uuid::nil(),
            change: "created",
        };
        let body = p.to_body_string();
        assert!(!body.contains("secret"));
        let v: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(v["change"], "created");
    }
}
