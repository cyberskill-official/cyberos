//! `POST /v1/admin/subjects/{id}/roles` + `DELETE /…/{role}` (FR-AUTH-101 §1 #5-#6).
//!
//! Auth: caller MUST hold `Resource::RoleAssignment + Action::Admin` (typically
//! `tenant-admin` or `root-admin`). The verify_jwt middleware ensures `Claims`
//! are in extensions; the matrix check is here.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    Extension,
};
use serde::Deserialize;
use serde_json::json;
use std::str::FromStr;
use uuid::Uuid;

use crate::jwt::Claims;
use crate::rbac::{Action, Resource, Role};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct AssignRoleBody {
    pub role: String,
}

/// `POST /v1/admin/subjects/{subject_id}/roles`
pub async fn assign_role(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(subject_id): Path<Uuid>,
    Json(body): Json<AssignRoleBody>,
) -> Response {
    // 1. Caller authz.
    let caller_roles = parse_caller_roles(&claims);
    let matrix = state.role_matrix.read().await;
    let authorised = matrix.any_role_has_permission(
        caller_roles.iter().copied(),
        Resource::RoleAssignment,
        Action::Admin,
    );
    if !authorised {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "caller lacks role-assignment admin"})),
        )
            .into_response();
    }

    // 2. Parse the requested role.
    let role = match Role::from_str(&body.role) {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "unknown_role",
                    "role": body.role,
                    "detail": e.to_string(),
                    "allowed": Role::ALL.iter().map(|r| r.as_str()).collect::<Vec<_>>(),
                })),
            )
                .into_response();
        }
    };

    // 3. Reserved-role gate (DEC-127).
    if role.is_reserved() {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "reserved_role",
                "role": role.as_str(),
                "detail": "reserved roles require a dedicated elevated-privilege endpoint",
            })),
        )
            .into_response();
    }

    // 4. WebAuthn-required gate (DEC-128) — `founder` requires a registered
    //    WebAuthn factor. Query mfa_factors directly so this works the moment
    //    FR-AUTH-105 WebAuthn enrolment lands (no code change needed here).
    if role.requires_webauthn() {
        let has_webauthn = subject_has_active_factor(&state.pg, subject_id, "webauthn")
            .await
            .unwrap_or(false);
        if !has_webauthn {
            return (
                StatusCode::CONFLICT,
                Json(json!({
                    "error": "webauthn_required",
                    "role": role.as_str(),
                    "detail": "founder role assignment requires a registered WebAuthn factor (FR-AUTH-105)",
                })),
            )
                .into_response();
        }
    }

    // 5. Tenant scope from JWT.
    let tenant_id = match Uuid::parse_str(&claims.tenant_id) {
        Ok(t) => t,
        Err(e) => return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("bad tenant_id claim: {e}")})),
        )
            .into_response(),
    };

    // 6. Persist.
    let granter = Uuid::parse_str(&claims.sub).unwrap_or_else(|_| Uuid::nil());
    let mut tx = match state.pg.begin().await {
        Ok(t) => t,
        Err(e) => return internal(e),
    };
    if let Err(e) = sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await
    { return internal(e); }

    let res = sqlx::query(
        "INSERT INTO subject_roles (tenant_id, subject_id, role, granted_by)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(tenant_id)
    .bind(subject_id)
    .bind(role.as_str())
    .bind(granter)
    .execute(&mut *tx)
    .await;

    match res {
        Ok(_) => {
            if let Err(e) = tx.commit().await { return internal(e); }
            (
                StatusCode::CREATED,
                Json(json!({
                    "subject_id": subject_id,
                    "role": role.as_str(),
                    "granted_by": granter,
                })),
            )
                .into_response()
        }
        Err(sqlx::Error::Database(db)) if db.is_unique_violation() => (
            StatusCode::CONFLICT,
            Json(json!({"error": "already_granted", "role": role.as_str()})),
        )
            .into_response(),
        Err(e) => internal(e),
    }
}

/// `DELETE /v1/admin/subjects/{subject_id}/roles/{role}`
pub async fn revoke_role(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path((subject_id, role)): Path<(Uuid, String)>,
) -> Response {
    let caller_roles = parse_caller_roles(&claims);
    let matrix = state.role_matrix.read().await;
    let authorised = matrix.any_role_has_permission(
        caller_roles.iter().copied(),
        Resource::RoleAssignment,
        Action::Admin,
    );
    if !authorised {
        return (StatusCode::FORBIDDEN, Json(json!({"error": "caller lacks role-assignment admin"}))).into_response();
    }
    let role_typed = match Role::from_str(&role) {
        Ok(r) => r,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"error": "unknown_role", "role": role}))).into_response(),
    };
    let tenant_id = Uuid::parse_str(&claims.tenant_id).unwrap_or(Uuid::nil());
    let mut tx = match state.pg.begin().await { Ok(t) => t, Err(e) => return internal(e) };
    let _ = sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx).await;
    let _ = sqlx::query("DELETE FROM subject_roles WHERE subject_id = $1 AND role = $2")
        .bind(subject_id)
        .bind(role_typed.as_str())
        .execute(&mut *tx).await;
    if let Err(e) = tx.commit().await { return internal(e); }
    StatusCode::NO_CONTENT.into_response()
}

/// Parse the caller's effective Role membership from the JWT.
/// FR-AUTH-101 §1 #8 — `Claims.roles` is the canonical source; the prior
/// `scope_grants` fallback handles tokens issued before this FR shipped
/// (the 30-day grace window per DEC-125).
fn parse_caller_roles(claims: &Claims) -> Vec<Role> {
    // Prefer the canonical `roles` claim if present (FR-AUTH-101 era).
    if !claims.roles.is_empty() {
        return claims
            .roles
            .iter()
            .filter_map(|s| Role::from_str(s).ok())
            .collect();
    }
    // Grace-window fallback: parse scope_grants the old way.
    let mut out = Vec::new();
    for s in &claims.scope_grants {
        if let Ok(r) = Role::from_str(s) {
            out.push(r);
        } else if s == "admin" {
            out.push(Role::TenantAdmin);
        }
    }
    out
}

fn internal<E: std::fmt::Display>(e: E) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response()
}

/// Returns true if `subject_id` has an active MFA factor of the requested
/// `factor_type`. Used by the founder webauthn gate.
async fn subject_has_active_factor(
    pool: &sqlx::PgPool,
    subject_id: Uuid,
    factor_type: &str,
) -> Result<bool, sqlx::Error> {
    let (count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM mfa_factors
          WHERE subject_id = $1 AND factor_type = $2 AND status = 'active'",
    )
    .bind(subject_id)
    .bind(factor_type)
    .fetch_one(pool)
    .await?;
    Ok(count > 0)
}
