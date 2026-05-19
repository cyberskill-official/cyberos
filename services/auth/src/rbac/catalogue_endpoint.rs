//! `GET /v1/admin/roles` — serve the closed catalogue + live version.
//!
//! Per FR-AUTH-101 §1 #7. Reads from the in-memory `RoleMatrix` only;
//! never hits the DB. The handler is mounted on the admin sub-router so
//! `verify_jwt` runs first.

use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Json, Response},
};
use serde::Serialize;
use serde_json::json;

use crate::rbac::Role;
use crate::AppState;

#[derive(Debug, Serialize)]
struct RoleDescriptor {
    name: &'static str,
    display: &'static str,
    reserved: bool,
    requires_webauthn: bool,
    stub_tier: bool,
}

impl From<Role> for RoleDescriptor {
    fn from(r: Role) -> Self {
        Self {
            name: r.as_str(),
            display: r.display(),
            reserved: r.is_reserved(),
            requires_webauthn: r.requires_webauthn(),
            stub_tier: r.is_stub_tier(),
        }
    }
}

pub async fn list_roles(State(state): State<AppState>) -> Response {
    // Even though the catalogue is compile-time, the version comes from the
    // live RoleMatrix in AppState (loaded at boot). Empty matrix → version 0
    // → ETag still meaningful so clients see the boot transition.
    let version = {
        let guard = state.role_matrix.read().await;
        guard.version()
    };
    let body = json!({
        "version": version,
        "roles": Role::ALL.iter().copied().map(RoleDescriptor::from).collect::<Vec<_>>(),
    });

    let etag = format!("W/\"rbac-v{version}\"");
    (StatusCode::OK, [(header::ETAG, etag)], Json(body)).into_response()
}

/// 304 short-circuit when the client's If-None-Match matches the current rbac_v.
pub async fn list_roles_with_etag_check(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    let version = {
        let guard = state.role_matrix.read().await;
        guard.version()
    };
    let etag = format!("W/\"rbac-v{version}\"");
    if let Some(inm) = headers.get(header::IF_NONE_MATCH) {
        if inm.to_str().map(|s| s == etag).unwrap_or(false) {
            return (StatusCode::NOT_MODIFIED, [(header::ETAG, etag)]).into_response();
        }
    }
    let body = json!({
        "version": version,
        "roles": Role::ALL.iter().copied().map(RoleDescriptor::from).collect::<Vec<_>>(),
    });
    (StatusCode::OK, [(header::ETAG, etag)], Json(body)).into_response()
}
