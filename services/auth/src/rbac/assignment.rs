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
    let caller_roles = parse_caller_roles(&claims.scope_grants);
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

    // 4. WebAuthn-required gate (DEC-128) — `founder` requires a registered factor.
    //    FR-AUTH-105 (WebAuthn enrolment) is not yet shipped; treat as "no factor"
    //    so this gate always refuses founder for now. Future fix: query mfa_factors.
    if role.requires_webauthn() {
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
    let caller_roles = parse_caller_roles(&claims.scope_grants);
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

/// Treat scope_grants as a placeholder for the caller's effective roles.
/// FR-AUTH-101 introduces a dedicated `roles` claim — once `Claims` carries
/// that field, swap this for `claims.roles`. For now we recognise the
/// stub strings "admin", "tenant-admin", "root-admin" as `TenantAdmin`-level.
fn parse_caller_roles(scopes: &[String]) -> Vec<Role> {
    let mut out = Vec::new();
    for s in scopes {
        if let Ok(r) = Role::from_str(s) {
            out.push(r);
        } else if s == "admin" {
            // Until FR-AUTH-101's `roles` claim ships, scope `admin` is treated
            // as TenantAdmin (matches the matrix seed in migration 0007).
            out.push(Role::TenantAdmin);
        }
    }
    out
}

fn internal<E: std::fmt::Display>(e: E) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response()
}
