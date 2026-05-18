//! FR-AUTH-106 slice-3 — admin REST for travel-policy mutation.
//!
//! Routes (all gated by `verify_jwt` middleware + role check inside each
//! handler — `security_admin` required to mutate, `tenant_admin` enough to
//! read):
//!
//!   GET    /v1/admin/tenants/:id/travel-policy
//!   PUT    /v1/admin/tenants/:id/travel-policy           — upsert one row
//!   GET    /v1/admin/tenants/:id/travel-policy/cidrs
//!   POST   /v1/admin/tenants/:id/travel-policy/cidrs     — add a CIDR
//!   DELETE /v1/admin/tenants/:id/travel-policy/cidrs/:id — remove a CIDR
//!
//! Every mutation writes a `travel_policy_audit` row (reason ≥10 chars) AND
//! invalidates the policy-cache entry so subsequent logins see the change
//! immediately rather than after the 60s TTL.

use axum::{
    extract::{Json as JsonInput, Path, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::jwt::Claims;
use crate::AppState;

const ROLE_SECURITY_ADMIN: &str = "security-admin";
const ROLE_TENANT_ADMIN: &str = "tenant-admin";

#[derive(Debug, Serialize, Deserialize)]
pub struct PolicyView {
    pub action: String,
    pub threshold_kmh: f64,
    pub block_anonymous_ip: bool,
    pub sticky_suppress_min: i32,
}

#[derive(Debug, Deserialize)]
pub struct PutPolicyBody {
    pub action: String,
    pub threshold_kmh: f64,
    pub block_anonymous_ip: bool,
    pub sticky_suppress_min: i32,
    pub reason: String,
}

pub async fn get_policy(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(tenant_id): Path<Uuid>,
) -> Result<Json<PolicyView>, (StatusCode, Json<Value>)> {
    require_tenant_admin(&claims, tenant_id)?;
    set_tenant(&state, tenant_id).await?;
    let row: Option<(String, f64, bool, i32)> = sqlx::query_as(
        "SELECT action, threshold_kmh, block_anonymous_ip, sticky_suppress_min
             FROM travel_policy WHERE tenant_id = $1",
    )
    .bind(tenant_id)
    .fetch_optional(&state.pg)
    .await
    .map_err(internal)?;
    let (action, threshold_kmh, block_anon, sticky) = row.unwrap_or_else(|| {
        ("challenge".to_string(), 1000.0, false, 30)
    });
    Ok(Json(PolicyView {
        action,
        threshold_kmh,
        block_anonymous_ip: block_anon,
        sticky_suppress_min: sticky,
    }))
}

pub async fn put_policy(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(tenant_id): Path<Uuid>,
    JsonInput(body): JsonInput<PutPolicyBody>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    require_security_admin(&claims, tenant_id)?;
    if body.reason.trim().len() < 10 {
        return Err(bad_req("reason_too_short", "reason must be >= 10 characters"));
    }
    if !["challenge", "block", "warn_only"].contains(&body.action.as_str()) {
        return Err(bad_req("invalid_action", "action must be one of challenge|block|warn_only"));
    }
    if !(200.0..=5000.0).contains(&body.threshold_kmh) {
        return Err(bad_req("threshold_out_of_range", "threshold_kmh must be in [200, 5000]"));
    }
    if !(0..=1440).contains(&body.sticky_suppress_min) {
        return Err(bad_req("sticky_out_of_range", "sticky_suppress_min must be in [0, 1440]"));
    }

    let actor_id = parse_actor(&claims)?;
    set_tenant(&state, tenant_id).await?;

    sqlx::query(
        "INSERT INTO travel_policy
                (tenant_id, action, threshold_kmh, block_anonymous_ip, sticky_suppress_min)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (tenant_id) DO UPDATE SET
                action = EXCLUDED.action,
                threshold_kmh = EXCLUDED.threshold_kmh,
                block_anonymous_ip = EXCLUDED.block_anonymous_ip,
                sticky_suppress_min = EXCLUDED.sticky_suppress_min,
                updated_at = NOW()",
    )
    .bind(tenant_id)
    .bind(&body.action)
    .bind(body.threshold_kmh)
    .bind(body.block_anonymous_ip)
    .bind(body.sticky_suppress_min)
    .execute(&state.pg)
    .await
    .map_err(internal)?;

    write_audit(
        &state, tenant_id, actor_id, "policy_updated",
        json!({
            "action": body.action,
            "threshold_kmh": body.threshold_kmh,
            "block_anonymous_ip": body.block_anonymous_ip,
            "sticky_suppress_min": body.sticky_suppress_min,
        }),
        &body.reason,
    ).await?;

    // Invalidate the cache so the next assess_login picks up the new policy.
    state.travel_policy.invalidate(tenant_id).await;

    Ok(Json(json!({"status": "ok"})))
}

#[derive(Debug, Deserialize)]
pub struct AddCidrBody {
    pub cidr: String,
    pub label: String,
    pub reason: String,
}

pub async fn add_cidr(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(tenant_id): Path<Uuid>,
    JsonInput(body): JsonInput<AddCidrBody>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    require_security_admin(&claims, tenant_id)?;
    if body.reason.trim().len() < 10 {
        return Err(bad_req("reason_too_short", "reason must be >= 10 characters"));
    }
    if body.label.trim().is_empty() {
        return Err(bad_req("missing_label", "label is required"));
    }
    let net: IpNetwork = body.cidr.parse().map_err(|e: ipnetwork::IpNetworkError| {
        bad_req("invalid_cidr", &format!("{e}"))
    })?;

    let actor_id = parse_actor(&claims)?;
    set_tenant(&state, tenant_id).await?;

    let (id,): (Uuid,) = sqlx::query_as(
        "INSERT INTO travel_cidr_allowlist (tenant_id, cidr, label, added_by)
         VALUES ($1, $2, $3, $4)
         RETURNING id",
    )
    .bind(tenant_id)
    .bind(net)
    .bind(&body.label)
    .bind(actor_id)
    .fetch_one(&state.pg)
    .await
    .map_err(|e| {
        // Surface the CHECK constraint violation explicitly so the admin UI
        // can show a useful error.
        let s = e.to_string();
        if s.contains("travel_cidr_prefix_tight") {
            return bad_req(
                "cidr_too_loose",
                "CIDR must be /9 or tighter (IPv4) or /17 or tighter (IPv6)",
            );
        }
        internal(e)
    })?;

    write_audit(
        &state, tenant_id, actor_id, "cidr_added",
        json!({"cidr": net.to_string(), "label": body.label, "id": id}),
        &body.reason,
    ).await?;
    state.travel_policy.invalidate(tenant_id).await;

    Ok(Json(json!({"status": "ok", "id": id})))
}

#[derive(Debug, Serialize)]
pub struct CidrView {
    pub id: Uuid,
    pub cidr: String,
    pub label: String,
}

pub async fn list_cidrs(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(tenant_id): Path<Uuid>,
) -> Result<Json<Vec<CidrView>>, (StatusCode, Json<Value>)> {
    require_tenant_admin(&claims, tenant_id)?;
    set_tenant(&state, tenant_id).await?;
    let rows: Vec<(Uuid, IpNetwork, String)> = sqlx::query_as(
        "SELECT id, cidr, label FROM travel_cidr_allowlist WHERE tenant_id = $1 ORDER BY added_at DESC",
    )
    .bind(tenant_id)
    .fetch_all(&state.pg)
    .await
    .map_err(internal)?;
    Ok(Json(
        rows.into_iter()
            .map(|(id, cidr, label)| CidrView { id, cidr: cidr.to_string(), label })
            .collect(),
    ))
}

#[derive(Debug, Deserialize)]
pub struct DeleteCidrBody {
    pub reason: String,
}

pub async fn delete_cidr(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path((tenant_id, cidr_id)): Path<(Uuid, Uuid)>,
    JsonInput(body): JsonInput<DeleteCidrBody>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    require_security_admin(&claims, tenant_id)?;
    if body.reason.trim().len() < 10 {
        return Err(bad_req("reason_too_short", "reason must be >= 10 characters"));
    }
    let actor_id = parse_actor(&claims)?;
    set_tenant(&state, tenant_id).await?;

    let row: Option<(IpNetwork, String)> = sqlx::query_as(
        "SELECT cidr, label FROM travel_cidr_allowlist
            WHERE tenant_id = $1 AND id = $2",
    )
    .bind(tenant_id)
    .bind(cidr_id)
    .fetch_optional(&state.pg)
    .await
    .map_err(internal)?;
    let (cidr, label) = row.ok_or_else(|| (
        StatusCode::NOT_FOUND,
        Json(json!({"error": "cidr_not_found"})),
    ))?;

    sqlx::query("DELETE FROM travel_cidr_allowlist WHERE tenant_id = $1 AND id = $2")
        .bind(tenant_id)
        .bind(cidr_id)
        .execute(&state.pg)
        .await
        .map_err(internal)?;

    write_audit(
        &state, tenant_id, actor_id, "cidr_removed",
        json!({"cidr": cidr.to_string(), "label": label, "id": cidr_id}),
        &body.reason,
    ).await?;
    state.travel_policy.invalidate(tenant_id).await;

    Ok(Json(json!({"status": "ok"})))
}

// ---------------------------------------------------------------------------
// Internals
// ---------------------------------------------------------------------------

fn require_security_admin(
    claims: &Claims,
    tenant_id: Uuid,
) -> Result<(), (StatusCode, Json<Value>)> {
    if claims.tenant_id != tenant_id.to_string() {
        return Err((StatusCode::FORBIDDEN, Json(json!({"error": "wrong_tenant"}))));
    }
    if claims.roles.iter().any(|r| r == ROLE_SECURITY_ADMIN) {
        return Ok(());
    }
    Err((
        StatusCode::FORBIDDEN,
        Json(json!({"error": "role_required", "role": ROLE_SECURITY_ADMIN})),
    ))
}

fn require_tenant_admin(
    claims: &Claims,
    tenant_id: Uuid,
) -> Result<(), (StatusCode, Json<Value>)> {
    if claims.tenant_id != tenant_id.to_string() {
        return Err((StatusCode::FORBIDDEN, Json(json!({"error": "wrong_tenant"}))));
    }
    if claims.roles.iter().any(|r| r == ROLE_TENANT_ADMIN || r == ROLE_SECURITY_ADMIN) {
        return Ok(());
    }
    Err((
        StatusCode::FORBIDDEN,
        Json(json!({"error": "role_required", "role": ROLE_TENANT_ADMIN})),
    ))
}

fn parse_actor(claims: &Claims) -> Result<Uuid, (StatusCode, Json<Value>)> {
    Uuid::parse_str(&claims.sub).map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"error": format!("bad subject in JWT: {e}")})),
    ))
}

async fn set_tenant(state: &AppState, tenant_id: Uuid) -> Result<(), (StatusCode, Json<Value>)> {
    sqlx::query("SET LOCAL app.current_tenant_id = $1")
        .bind(tenant_id.to_string())
        .execute(&state.pg)
        .await
        .map_err(internal)
        .map(|_| ())
}

async fn write_audit(
    state: &AppState,
    tenant_id: Uuid,
    actor_id: Uuid,
    change_kind: &str,
    detail: Value,
    reason: &str,
) -> Result<(), (StatusCode, Json<Value>)> {
    sqlx::query(
        "INSERT INTO travel_policy_audit
                (tenant_id, actor_id, change_kind, detail, reason)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(tenant_id)
    .bind(actor_id)
    .bind(change_kind)
    .bind(detail)
    .bind(reason)
    .execute(&state.pg)
    .await
    .map_err(internal)
    .map(|_| ())
}

fn bad_req(err: &str, detail: &str) -> (StatusCode, Json<Value>) {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({"error": err, "detail": detail})),
    )
}

fn internal<E: std::fmt::Display>(e: E) -> (StatusCode, Json<Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"error": format!("internal: {e}")})),
    )
}
