//! Axum handlers. Wave-2 first-slice covers `/healthz`, `POST /v1/admin/tenants`,
//! `POST /v1/admin/subjects`. Authentication is stubbed — the route guards
//! land with FR-AUTH-004 (JWT issuance + verification).

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::{get, post},
    Router,
};
use cyberos_types::{SubjectId, TenantId};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::models::{CreateSubjectRequest, CreateTenantRequest, Subject, Tenant};
use crate::AppState;

/// Build the full router. Wired by main.rs.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/v1/admin/tenants", post(create_tenant))
        .route("/v1/admin/subjects", post(create_subject))
        .with_state(state)
}

async fn healthz(State(state): State<AppState>) -> (StatusCode, Json<Value>) {
    let pg_ok = sqlx::query("SELECT 1").fetch_one(&state.pg).await.is_ok();
    let status = if pg_ok { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE };
    (
        status,
        Json(json!({
            "service": "cyberos-auth",
            "version": crate::VERSION,
            "postgres": if pg_ok { "ok" } else { "down" },
        })),
    )
}

/// FR-AUTH-001 — Tenant create. Idempotent on `Idempotency-Key` header.
async fn create_tenant(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateTenantRequest>,
) -> Result<(StatusCode, Json<Tenant>), (StatusCode, Json<Value>)> {
    // Only the root tenant can create new tenants. The auth middleware
    // (FR-AUTH-004) will validate the JWT and set `app.current_tenant_id`.
    // Until then, this handler runs in the root context.
    let mut tx = state
        .pg
        .begin()
        .await
        .map_err(internal_err)?;
    sqlx::query("SET LOCAL app.current_tenant_id = '00000000-0000-0000-0000-000000000000'")
        .execute(&mut *tx)
        .await
        .map_err(internal_err)?;

    // Idempotency-Key is required on admin POSTs.
    let key = headers
        .get("idempotency-key")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Idempotency-Key header required"}))
        ))?;

    let route = "POST /v1/admin/tenants";
    let root_uuid = Uuid::nil();
    if let Some((status, body)) = crate::idempotency::lookup(&state.pg, key, route, root_uuid)
        .await
        .map_err(internal_err)?
    {
        // Replay the prior response bit-for-bit.
        let tenant: Tenant = serde_json::from_value(body)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;
        return Ok((StatusCode::from_u16(status as u16).unwrap_or(StatusCode::OK), Json(tenant)));
    }

    let new_id = TenantId::new();
    let row: Tenant = sqlx::query_as::<_, (Uuid, String, String, String, String, String, String, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
        "INSERT INTO tenants (id, slug, display_name, country, plan_tier, status, residency)
              VALUES ($1, $2, $3, $4, $5, 'active', $6)
            RETURNING id, slug, display_name, country, plan_tier, status, residency, created_at, updated_at",
    )
    .bind(new_id.as_uuid())
    .bind(&req.slug)
    .bind(&req.display_name)
    .bind(&req.country)
    .bind(&req.plan_tier)
    .bind(&req.residency)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(db) if db.is_unique_violation() => (
            StatusCode::CONFLICT,
            Json(json!({"error": format!("slug '{}' already taken", req.slug)})),
        ),
        other => internal_err(other),
    })
    .map(|(id, slug, display_name, country, plan_tier, status, residency, created_at, updated_at)| {
        Tenant {
            id: TenantId(id),
            slug,
            display_name,
            country,
            plan_tier,
            status,
            residency,
            created_at,
            updated_at,
        }
    })?;

    tx.commit().await.map_err(internal_err)?;

    let body = serde_json::to_value(&row).map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"error": e.to_string()})),
    ))?;
    crate::idempotency::record(
        &state.pg,
        key,
        route,
        root_uuid,
        StatusCode::CREATED.as_u16() as i16,
        &body,
    )
    .await
    .map_err(internal_err)?;

    Ok((StatusCode::CREATED, Json(row)))
}

/// FR-AUTH-002 — Subject create. Bcrypt-hashes the password before insert.
async fn create_subject(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateSubjectRequest>,
) -> Result<(StatusCode, Json<Subject>), (StatusCode, Json<Value>)> {
    let tenant_id = headers
        .get("x-tenant-id")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "X-Tenant-Id header required (UUID)"})),
        ))?;

    let pw_hash = match (&req.kind[..], req.password.as_deref()) {
        ("human", Some(plain)) => Some(
            bcrypt::hash(plain, bcrypt::DEFAULT_COST).map_err(|e| (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("bcrypt failed: {e}")})),
            ))?
        ),
        ("human", None) => return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "human subject requires password"}))
        )),
        _ => None,
    };

    let mut tx = state.pg.begin().await.map_err(internal_err)?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(internal_err)?;

    let new_id = SubjectId::new();
    let row: (Uuid, Uuid, String, Option<String>, Option<String>, String, String, Vec<String>, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>) = sqlx::query_as(
        "INSERT INTO subjects (id, tenant_id, handle, display_name, email, kind, password_hash, roles)
              VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, tenant_id, handle, display_name, email, kind, status, roles, created_at, updated_at",
    )
    .bind(new_id.as_uuid())
    .bind(tenant_id)
    .bind(&req.handle)
    .bind(req.display_name.as_deref())
    .bind(req.email.as_deref())
    .bind(&req.kind)
    .bind(pw_hash.as_deref())
    .bind(&req.roles)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(db) if db.is_unique_violation() => (
            StatusCode::CONFLICT,
            Json(json!({"error": format!("handle '{}' already taken in this tenant", req.handle)})),
        ),
        other => internal_err(other),
    })?;

    tx.commit().await.map_err(internal_err)?;

    Ok((
        StatusCode::CREATED,
        Json(Subject {
            id: SubjectId(row.0),
            tenant_id: TenantId(row.1),
            handle: row.2,
            display_name: row.3,
            email: row.4,
            kind: row.5,
            status: row.6,
            roles: row.7,
            created_at: row.8,
            updated_at: row.9,
        }),
    ))
}

fn internal_err<E: std::fmt::Display>(e: E) -> (StatusCode, Json<Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"error": e.to_string()})),
    )
}
