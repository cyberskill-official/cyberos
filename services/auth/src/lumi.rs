//! FR-AUTH-108 — Lumi tenant-identity JWT issuance + verify + revoke.
//!
//! Lumi is the cloud-hosted org memory. Personal-memory sync (FR-MEMORY-103)
//! needs a JWT identifying which Lumi workspace this memory's shareable
//! memories can push to. This module mints those tokens, logs every
//! issuance, and provides a revoke endpoint for the operator.
//!
//! Three admin-router endpoints (root-admin / tenant-admin):
//!   * `POST /v1/auth/lumi/issue`  — mint a fresh Lumi JWT for (subject, workspace).
//!   * `GET  /v1/auth/lumi/verify` — verify a presented Lumi JWT + check
//!     against the revocation table.
//!   * `POST /v1/admin/lumi/revoke/{jti}` — operator revoke.
//!
//! Token shape: standard CyberOS JWT with `aud: ["lumi", "memory-sync"]` and
//! a new claim `lumi_workspace: <id>`. Verifier in slice 1 reuses the
//! existing `JwtService::verify` then checks `lumi_token_issuance_log` for
//! a matching `jti` with `revoked_at IS NULL`. Slice 2 will move the
//! revocation list into the in-memory cache that backs `RoleMatrix`.

use axum::{
    extract::{Json as JsonInput, Path, Query, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use chrono::{DateTime, Duration, Utc};
use cyberos_types::{SubjectId, TenantId};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::str::FromStr;
use uuid::Uuid;

use crate::jwt::{Claims, JwtService};
use crate::rbac::Role;
use crate::AppState;

const DEFAULT_LUMI_TTL_SECS: i64 = 60 * 60 * 24 * 7; // 7d

// ---------------------------------------------------------------------------
// POST /v1/auth/lumi/issue
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct IssueBody {
    /// Lumi workspace ID — opaque to AUTH; meaningful to Lumi-side sync code.
    pub lumi_workspace_id: String,
    /// Subject the token is FOR. Must belong to the caller's tenant.
    pub subject_id: Uuid,
    /// Optional scope grants beyond the defaults (e.g. ["memory-sync:read", "memory-sync:write"]).
    #[serde(default)]
    pub scope_grants: Vec<String>,
    /// Optional TTL override in seconds (capped at 30 days).
    #[serde(default)]
    pub ttl_secs: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct IssueResponse {
    pub access_token: String,
    pub jti: String,
    pub kid: String,
    pub expires_in: i64,
    pub lumi_workspace_id: String,
}

pub async fn issue(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    JsonInput(body): JsonInput<IssueBody>,
) -> Result<(StatusCode, Json<IssueResponse>), (StatusCode, Json<Value>)> {
    let caller_can_issue = claims.roles.iter().any(|s| {
        Role::from_str(s)
            .map(|r| matches!(r, Role::RootAdmin | Role::TenantAdmin))
            .unwrap_or(false)
    });
    if !caller_can_issue {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "tenant-admin or root-admin required to mint Lumi JWTs"})),
        ));
    }

    let tenant_id = Uuid::parse_str(&claims.tenant_id).map_err(internal)?;
    let ttl_secs = body
        .ttl_secs
        .unwrap_or(DEFAULT_LUMI_TTL_SECS)
        .clamp(60, 60 * 60 * 24 * 30); // floor 1m, ceil 30d

    // Build the JWT via the existing issuer. The audience widens to include
    // 'lumi' so Lumi-side verifiers accept it.
    let svc = JwtService::new(state.pg.clone(), state.jwt_issuer.clone());
    let kind = "agent".to_string(); // Lumi tokens represent the personal-memory agent.
    let agent_persona = Some(format!("lumi-bridge@{}", body.lumi_workspace_id));

    let mut scopes = body.scope_grants.clone();
    if !scopes.iter().any(|s| s.starts_with("memory-sync")) {
        scopes.push("memory-sync:push".to_string());
    }

    let tokens = svc
        .issue(
            TenantId(tenant_id),
            SubjectId(body.subject_id),
            "",              // FR-AUTH-004 §1 #2 — Lumi tokens are agent-scoped, no email
            &kind,
            scopes.clone(),
            vec![],          // roles intentionally empty — Lumi token isn't a session JWT
            None,            // rbac_v unused on Lumi tokens
            agent_persona,
            None,
        )
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("lumi jwt issuance failed: {e}")})),
        ))?;

    // Extract the `jti` we just minted by re-verifying our own token (round-trip).
    let verified = svc.verify(&tokens.access_token).await.map_err(|e| internal(e))?;
    let jti = verified.jti.clone();
    let expires_at: DateTime<Utc> = DateTime::from_timestamp(verified.exp, 0)
        .unwrap_or_else(|| Utc::now() + Duration::seconds(ttl_secs));

    // Audit row.
    let mut tx = state.pg.begin().await.map_err(internal)?;
    sqlx::query("SET LOCAL app.current_tenant_id = $1")
        .bind(tenant_id.to_string())
        .execute(&mut *tx).await.map_err(internal)?;
    sqlx::query(
        "INSERT INTO lumi_token_issuance_log
                (tenant_id, subject_id, lumi_workspace_id,
                 aud, scope_grants, expires_at, kid, jti, issued_via)
         VALUES ($1, $2, $3, ARRAY['lumi','memory-sync','cyberos'], $4, $5, $6, $7, 'admin')",
    )
    .bind(tenant_id)
    .bind(body.subject_id)
    .bind(&body.lumi_workspace_id)
    .bind(&scopes)
    .bind(expires_at)
    .bind(&tokens.kid)
    .bind(&jti)
    .execute(&mut *tx).await.map_err(internal)?;
    tx.commit().await.map_err(internal)?;

    Ok((StatusCode::CREATED, Json(IssueResponse {
        access_token: tokens.access_token,
        jti,
        kid: tokens.kid,
        expires_in: ttl_secs,
        lumi_workspace_id: body.lumi_workspace_id,
    })))
}

// ---------------------------------------------------------------------------
// GET /v1/auth/lumi/verify
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct VerifyQuery {
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyResponse {
    pub valid: bool,
    pub jti: String,
    pub tenant_id: String,
    pub subject_id: String,
    pub lumi_workspace_id: Option<String>,
    pub aud: Vec<String>,
    pub scope_grants: Vec<String>,
    pub expires_at: i64,
    pub reason: Option<String>,
}

pub async fn verify(
    State(state): State<AppState>,
    Query(q): Query<VerifyQuery>,
) -> Result<(StatusCode, Json<VerifyResponse>), (StatusCode, Json<Value>)> {
    let svc = JwtService::new(state.pg.clone(), state.jwt_issuer.clone());

    let claims = match svc.verify(&q.token).await {
        Ok(c) => c,
        Err(e) => {
            return Ok((StatusCode::OK, Json(VerifyResponse {
                valid: false,
                jti: String::new(),
                tenant_id: String::new(),
                subject_id: String::new(),
                lumi_workspace_id: None,
                aud: vec![],
                scope_grants: vec![],
                expires_at: 0,
                reason: Some(format!("signature/expiry check failed: {e}")),
            })));
        }
    };

    // Audience MUST include 'lumi' for this endpoint to consider it valid.
    if !claims.aud.iter().any(|a| a == "lumi") {
        return Ok((StatusCode::OK, Json(VerifyResponse {
            valid: false,
            jti: claims.jti,
            tenant_id: claims.tenant_id,
            subject_id: claims.sub,
            lumi_workspace_id: None,
            aud: claims.aud,
            scope_grants: claims.scope_grants,
            expires_at: claims.exp,
            reason: Some("token aud does not include 'lumi'".into()),
        })));
    }

    // Revocation check.
    let revoked: Option<(Option<DateTime<Utc>>,)> = sqlx::query_as(
        "SELECT revoked_at FROM lumi_token_issuance_log WHERE jti = $1 LIMIT 1",
    )
    .bind(&claims.jti)
    .fetch_optional(&state.pg)
    .await
    .map_err(internal)?;

    let revoked = revoked.and_then(|(r,)| r);
    if revoked.is_some() {
        return Ok((StatusCode::OK, Json(VerifyResponse {
            valid: false,
            jti: claims.jti,
            tenant_id: claims.tenant_id,
            subject_id: claims.sub,
            lumi_workspace_id: None,
            aud: claims.aud,
            scope_grants: claims.scope_grants,
            expires_at: claims.exp,
            reason: Some("token was revoked".into()),
        })));
    }

    // Pull the workspace ID + persona from the agent_persona claim shape `lumi-bridge@<workspace>`.
    let lumi_workspace_id = claims
        .agent_persona
        .as_ref()
        .and_then(|s| s.strip_prefix("lumi-bridge@"))
        .map(String::from);

    Ok((StatusCode::OK, Json(VerifyResponse {
        valid: true,
        jti: claims.jti,
        tenant_id: claims.tenant_id,
        subject_id: claims.sub,
        lumi_workspace_id,
        aud: claims.aud,
        scope_grants: claims.scope_grants,
        expires_at: claims.exp,
        reason: None,
    })))
}

// ---------------------------------------------------------------------------
// POST /v1/admin/lumi/revoke/{jti}
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct RevokeBody {
    pub reason: String,
}

pub async fn revoke(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(jti): Path<String>,
    JsonInput(body): JsonInput<RevokeBody>,
) -> Result<StatusCode, (StatusCode, Json<Value>)> {
    let caller_can_revoke = claims.roles.iter().any(|s| {
        Role::from_str(s)
            .map(|r| matches!(r, Role::RootAdmin | Role::TenantAdmin))
            .unwrap_or(false)
    });
    if !caller_can_revoke {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "tenant-admin or root-admin required"})),
        ));
    }
    if body.reason.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "reason required"})),
        ));
    }
    let revoker = Uuid::parse_str(&claims.sub).map_err(internal)?;
    let tenant_id = Uuid::parse_str(&claims.tenant_id).map_err(internal)?;

    let mut tx = state.pg.begin().await.map_err(internal)?;
    sqlx::query("SET LOCAL app.current_tenant_id = $1")
        .bind(tenant_id.to_string())
        .execute(&mut *tx).await.map_err(internal)?;
    let res = sqlx::query(
        "UPDATE lumi_token_issuance_log
            SET revoked_at = NOW(), revoked_by = $1, revoke_reason = $2
          WHERE jti = $3 AND revoked_at IS NULL",
    )
    .bind(revoker)
    .bind(&body.reason)
    .bind(&jti)
    .execute(&mut *tx)
    .await
    .map_err(internal)?;
    tx.commit().await.map_err(internal)?;

    if res.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "jti not found or already revoked"})),
        ));
    }
    Ok(StatusCode::NO_CONTENT)
}

fn internal<E: std::fmt::Display>(e: E) -> (StatusCode, Json<Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"error": e.to_string()})),
    )
}
