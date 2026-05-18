//! Axum handlers.
//!
//! Routes:
//!   * `/healthz`                       GET   — liveness + Postgres check
//!   * `/v1/admin/tenants`              POST  — FR-AUTH-001
//!   * `/v1/admin/tenants`              GET   — FR-AUTH-005 (list, root only)
//!   * `/v1/admin/subjects`             POST  — FR-AUTH-002
//!   * `/v1/admin/subjects`             GET   — FR-AUTH-005 (list, tenant-scoped via RLS)
//!   * `/v1/admin/subjects/{id}/revoke`   POST — FR-AUTH-005
//!   * `/v1/admin/subjects/{id}/unrevoke` POST — FR-AUTH-005
//!   * `/v1/auth/token`                 POST  — FR-AUTH-004 (password-grant JWT)
//!   * `/.well-known/jwks.json`         GET   — FR-AUTH-004 (public JWKS)

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::{get, post},
    Extension, Router,
};
use cyberos_types::{SubjectId, TenantId};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::jwt::{Claims, JwtService};
use crate::models::{CreateSubjectRequest, CreateTenantRequest, Subject, Tenant};
use crate::AppState;

/// Build the full router. Wired by main.rs.
///
/// Layout:
///   * **Public** (no auth): `/healthz`, `POST /v1/auth/token`, `GET /.well-known/jwks.json`
///   * **Admin** (Bearer JWT with scope_grants ⊇ ["admin"]): `/v1/admin/*`
///
/// The JWT-verification middleware runs ahead of every admin route; it
/// attaches `Claims` to request extensions and sets the GUC for RLS.
pub fn router(state: AppState) -> Router {
    let public = Router::new()
        .route("/healthz", get(healthz))
        .route("/v1/auth/token", post(issue_token))
        .route("/.well-known/jwks.json", get(jwks))
        // FR-AUTH-104 OIDC SSO — public flow (no JWT required to initiate)
        .route("/v1/auth/oidc/initiate", get(crate::oidc::initiate))
        .route("/v1/auth/oidc/callback", get(crate::oidc::callback))
        // FR-AUTH-105 Passkey — login is public (the whole point is no-password auth)
        .route("/v1/auth/passkey/login/begin", post(crate::passkey::login_begin))
        .route("/v1/auth/passkey/login/finish", post(crate::passkey::login_finish))
        // FR-AUTH-103 SAML — initiate + ACS + SP metadata are PUBLIC
        .route("/v1/auth/saml/initiate", get(crate::saml::initiate))
        .route("/v1/auth/saml/acs", post(crate::saml::acs))
        .route("/v1/auth/saml/idp-configs/:id/sp-metadata", get(crate::saml::sp_metadata));

    let admin = Router::new()
        .route("/v1/admin/tenants", post(create_tenant).get(list_tenants))
        .route("/v1/admin/subjects", post(create_subject).get(list_subjects))
        .route("/v1/admin/subjects/:id/revoke", post(revoke_subject))
        .route("/v1/admin/subjects/:id/unrevoke", post(unrevoke_subject))
        // FR-AUTH-101 RBAC endpoints
        .route("/v1/admin/roles", get(crate::rbac::catalogue_endpoint::list_roles_with_etag_check))
        .route(
            "/v1/admin/subjects/:id/roles",
            post(crate::rbac::assignment::assign_role),
        )
        .route(
            "/v1/admin/subjects/:id/roles/:role",
            axum::routing::delete(crate::rbac::assignment::revoke_role),
        )
        // FR-AUTH-102 MFA — TOTP enrolment + verify (auth'd; the password
        // grant is what eventually CALLS verify, but enrolment requires an
        // authenticated session).
        .route("/v1/auth/mfa/factors/totp/enrol", post(crate::mfa::totp_enrol_start))
        .route("/v1/auth/mfa/factors/totp/enrol/finish", post(crate::mfa::totp_enrol_finish))
        .route("/v1/auth/mfa/verify", post(crate::mfa::totp_verify))
        // FR-AUTH-104 OIDC admin — create/update IdP config (JWT-gated)
        .route("/v1/admin/oidc/idp-configs", post(crate::oidc::create_idp_config))
        // FR-AUTH-105 Passkey enrol — requires authenticated session
        .route("/v1/auth/passkey/enrol/begin", post(crate::passkey::enrol_begin))
        .route("/v1/auth/passkey/enrol/finish", post(crate::passkey::enrol_finish))
        // FR-AUTH-103 SAML admin — create/update IdP config (JWT-gated)
        .route("/v1/admin/saml/idp-configs", post(crate::saml::create_idp_config))
        // FR-AUTH-109 stub→full migration enforcer (root-admin only)
        .route("/v1/admin/auth/migration/preview", get(crate::migration_state::preview))
        .route("/v1/admin/auth/migration/extend-grace", post(crate::migration_state::extend_grace))
        // FR-AUTH-108 Lumi tenant-identity JWT — admin-gated issue/revoke
        .route("/v1/auth/lumi/issue", post(crate::lumi::issue))
        .route("/v1/auth/lumi/verify", get(crate::lumi::verify))
        .route("/v1/admin/lumi/revoke/:jti", post(crate::lumi::revoke))
        // FR-AUTH-106 slice-3 — per-tenant travel-policy mutation
        .route(
            "/v1/admin/tenants/:tenant_id/travel-policy",
            axum::routing::put(crate::travel_admin::put_policy)
                .get(crate::travel_admin::get_policy),
        )
        .route(
            "/v1/admin/tenants/:tenant_id/travel-policy/cidrs",
            post(crate::travel_admin::add_cidr).get(crate::travel_admin::list_cidrs),
        )
        .route(
            "/v1/admin/tenants/:tenant_id/travel-policy/cidrs/:cidr_id",
            axum::routing::delete(crate::travel_admin::delete_cidr),
        )
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::verify_jwt,
        ));

    public.merge(admin).with_state(state)
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
// FR-AUTH-001 §1 #13 — emit OTel span `auth.create_tenant` around the whole
// handler. `outcome` is recorded dynamically (created | idempotent_replay |
// conflict | forbidden | invalid_input | error) at each return. The trace
// context propagates W3C TraceContext per FR-AI-022 — the verify_jwt
// middleware already extracts traceparent from the JWT into the request.
#[tracing::instrument(
    name = "auth.create_tenant",
    skip(state, claims, headers, req),
    fields(
        slug = %req.slug,
        caller_tenant_id = %claims.tenant_id,
        outcome = tracing::field::Empty,
    )
)]
async fn create_tenant(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    headers: HeaderMap,
    Json(req): Json<CreateTenantRequest>,
) -> Result<(StatusCode, Json<Tenant>), (StatusCode, Json<Value>)> {
    use tracing::Span;
    let span = Span::current();

    // FR-AUTH-001 §1 #1 — Handler-level authz: caller MUST be in tenant 0
    // AND hold the `root-admin` role. The `verify_jwt` middleware (FR-AUTH-004)
    // has already validated the signature + populated `claims`; this check
    // is the defence-in-depth role guard at the handler layer. The 403 body
    // is explicit about WHAT permission would have succeeded so operators
    // can grant it correctly.
    if let Err(e) = require_root_admin_in_tenant_0(&claims) {
        span.record("outcome", "forbidden");
        return Err(e);
    }

    // FR-AUTH-001 §1 #14 — Defence-in-depth: reject reserved slug "root"
    // before any DB work. Tenant 0 (the root tenant) is bootstrapped by
    // FR-AUTH-006 CLI; this endpoint MUST NOT create a second tenant
    // with slug "root" (would shadow the canonical root in operator
    // mental models even though id is unique). The DB UNIQUE constraint
    // also catches this, but the early-return saves a transaction round
    // trip and produces a structured error body.
    if req.slug == "root" {
        span.record("outcome", "invalid_input");
        return Err(invalid_input(
            "slug",
            "slug \"root\" is reserved for tenant 0 (use cyberos-auth-bootstrap)",
        ));
    }

    // FR-AUTH-001 §1 #2 + #11 — Per-field validation runs at API layer +
    // DB CHECK constraint (defence in depth). The 400 body identifies
    // exactly which input failed and why so the client can render
    // actionable error UI without inspecting logs.
    if let Err(e) = validate_slug(&req.slug) {
        span.record("outcome", "invalid_input");
        return Err(e);
    }
    if let Err(e) = validate_display_name(&req.display_name) {
        span.record("outcome", "invalid_input");
        return Err(e);
    }

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
    let key = match headers
        .get("idempotency-key")
        .and_then(|h| h.to_str().ok())
    {
        Some(k) => k,
        None => {
            span.record("outcome", "missing_header");
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "missing_header",
                    "field": "Idempotency-Key",
                    "reason": "header required on admin POSTs for idempotent retries (per FR-AUTH-001 §1 #5)"
                })),
            ));
        }
    };

    let route = "POST /v1/admin/tenants";
    let root_uuid = Uuid::nil();
    if let Some((status, body)) = crate::idempotency::lookup(&state.pg, key, route, root_uuid)
        .await
        .map_err(internal_err)?
    {
        // Replay the prior response bit-for-bit.
        span.record("outcome", "idempotent_replay");
        let tenant: Tenant = serde_json::from_value(body)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;
        return Ok((StatusCode::from_u16(status as u16).unwrap_or(StatusCode::OK), Json(tenant)));
    }

    let new_id = TenantId::new();
    let insert_result: Result<(Uuid, String, String, String, String, String, String, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>), sqlx::Error> = sqlx::query_as::<_, (Uuid, String, String, String, String, String, String, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
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
    .await;

    let row: Tenant = match insert_result {
        Err(sqlx::Error::Database(db)) if db.is_unique_violation() => {
            // FR-AUTH-001 §1 #4 — Structured 409 body so the client can
            // present the conflict without parsing free-form error strings.
            span.record("outcome", "conflict");
            return Err((
                StatusCode::CONFLICT,
                Json(json!({"error": "slug_taken", "slug": req.slug})),
            ));
        }
        Err(other) => {
            span.record("outcome", "error");
            return Err(internal_err(other));
        }
        Ok((id, slug, display_name, country, plan_tier, status, residency, created_at, updated_at)) => {
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
        }
    };

    // FR-AUTH-001 §1 #6 + §1 #12 — Emit `auth.tenant_created` BRAIN audit row
    // INSIDE the same transaction. If this write fails (or any later step
    // before commit), the entire tx rolls back — both the tenant row and
    // the audit row are discarded together. The partial state of "tenant
    // exists but no audit trail" is forbidden by construction.
    let caller_subject_id =
        Uuid::parse_str(&claims.sub).unwrap_or_else(|_| Uuid::nil());
    let request_id = headers
        .get("x-request-id")
        .and_then(|h| h.to_str().ok());
    let payload = crate::brain_bridge::TenantCreatedPayload {
        tenant_id: row.id.as_uuid(),
        slug: &row.slug,
        display_name: &row.display_name,
        created_by_subject_id: caller_subject_id,
        idempotency_key: Some(key),
        request_id,
    };
    if let Err(e) = crate::brain_bridge::emit_tenant_created(&mut tx, payload).await {
        span.record("outcome", "error");
        // tx will roll back on drop since we return Err here. The tenant
        // INSERT never commits — audit failure → no tenant.
        return Err(internal_err(e));
    }

    tx.commit().await.map_err(|e| {
        span.record("outcome", "error");
        internal_err(e)
    })?;

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

    span.record("outcome", "created");
    Ok((StatusCode::CREATED, Json(row)))
}

/// FR-AUTH-002 — Subject create. Bcrypt-hashes the password before insert.
/// `tenant_id` is taken from the verified JWT claims — the route is gated by
/// `verify_jwt` middleware so `Extension<Claims>` is always present.
///
/// G-001..014 audit-fix loop slice-1 (session 21):
///   * G-001 email regex validation                      (validate_email)
///   * G-003 role allow-list                              (validate_roles)
///   * G-004 Idempotency-Key honoured                     (idempotency::lookup + record)
///   * G-005 auth.subject_created BRAIN audit row         (brain_bridge::emit_subject_created)
///   * G-006 structured 409 handle_taken / email_taken
///   * G-009 HIBP audit emit-in-transaction (atomicity)   (moved INTO the tx)
///   * G-010 OTel #[tracing::instrument]
///   * G-012 handler-level tenant-admin role check
#[tracing::instrument(
    name = "auth.create_subject",
    skip(state, claims, headers, req),
    fields(
        tenant_id = %claims.tenant_id,
        roles_count = req.roles.len(),
        email_hash16 = tracing::field::Empty,
        outcome = tracing::field::Empty,
    )
)]
async fn create_subject(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    headers: HeaderMap,
    Json(req): Json<CreateSubjectRequest>,
) -> Result<(StatusCode, Json<Subject>), (StatusCode, Json<Value>)> {
    use tracing::Span;
    let span = Span::current();

    let tenant_id = Uuid::parse_str(&claims.tenant_id).map_err(|_| {
        // Malformed tenant_id claim → 403 (not 500) so the failure mode
        // doesn't leak parser internals (same defence-in-depth as
        // FR-AUTH-001's require_root_admin_in_tenant_0).
        span.record("outcome", "forbidden");
        (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "forbidden", "needed": "tenant-admin or root-admin"})),
        )
    })?;

    // G-012 — handler-level role check (defence-in-depth on top of RLS).
    // Per §1 #1: caller MUST have `tenant-admin` in claims.roles, OR be
    // root-admin in tenant 0 (the latter implies tenant-admin everywhere).
    let has_tenant_admin = claims.roles.iter().any(|r| r == "tenant-admin");
    let is_root_admin_zero = tenant_id == Uuid::nil()
        && claims.roles.iter().any(|r| r == "root-admin");
    if !has_tenant_admin && !is_root_admin_zero {
        span.record("outcome", "forbidden");
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "forbidden",
                "needed": "tenant-admin role in caller's tenant (or root-admin in tenant 0)"
            })),
        ));
    }

    // G-008 — §1 #11 — HTTPS required for password-bearing requests. The
    // reverse-proxy sets `X-Forwarded-Proto: https` after TLS termination.
    // Production refuses without it. Test environment short-circuits via
    // `AUTH_TEST_ALLOW_HTTP=1` so integration tests don't need to spin up
    // a TLS listener. Plaintext password over HTTP is a categorical no.
    if req.kind == "human" && req.password.is_some() {
        if let Err(e) = require_https(&headers) {
            span.record("outcome", "https_required");
            return Err(e);
        }
    }

    // G-001 — Email regex validation (§1 #2). Mirrors the PG CHECK constraint
    // on `subjects.email` (loose RFC 5321 — disallows whitespace + requires
    // an @ + a dot in the domain).
    if let Some(email) = req.email.as_deref() {
        if let Err(e) = validate_email(email) {
            span.record("outcome", "invalid_input");
            return Err(e);
        }
        span.record("email_hash16", crate::brain_bridge::email_hash16(email).as_str());
    }

    // G-003 — Role allow-list (§1 #5). Closed allow-list for slice 1:
    // `{"tenant-admin", "tenant-member"}`. Expanded by FR-AUTH-101 to 22 roles.
    if let Err(e) = validate_roles(&req.roles) {
        span.record("outcome", "invalid_input");
        return Err(e);
    }

    // G-004 — Idempotency-Key honoured (§1 #6). Required on admin POSTs.
    let idem_key = match headers
        .get("idempotency-key")
        .and_then(|h| h.to_str().ok())
    {
        Some(k) => k,
        None => {
            span.record("outcome", "missing_header");
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "missing_header",
                    "field": "Idempotency-Key",
                    "reason": "header required on admin POSTs for idempotent retries (per FR-AUTH-002 §1 #6)"
                })),
            ));
        }
    };
    let route = "POST /v1/admin/subjects";
    if let Some((status, body)) = crate::idempotency::lookup(&state.pg, idem_key, route, tenant_id)
        .await
        .map_err(internal_err)?
    {
        // Replay prior response.
        span.record("outcome", "idempotent_replay");
        let subject: Subject = serde_json::from_value(body).map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ))?;
        return Ok((
            StatusCode::from_u16(status as u16).unwrap_or(StatusCode::OK),
            Json(subject),
        ));
    }

    // FR-AUTH-002 §1 #3 + §1 #4 (slice-2) — validate password complexity
    // BEFORE the network round-trip to HIBP. Catches weak passwords cheaply
    // (the HIBP call costs 50-200ms; complexity validation is microseconds).
    // The password is wrapped in `Zeroizing<String>` so the plaintext bytes
    // are overwritten on drop — even if a panic unwinds the stack mid-flow,
    // the wrapper's Drop runs.
    let zeroized_password: Option<crate::password::ZeroizedString<String>> = req
        .password
        .as_deref()
        .map(crate::password::wrap);
    if let (Some(z), "human") = (zeroized_password.as_deref(), req.kind.as_str()) {
        let email_local_part = req
            .email
            .as_deref()
            .and_then(|e| e.split_once('@').map(|(local, _)| local))
            .unwrap_or("");
        if let Err(e) = crate::password::validate_plaintext(z, email_local_part) {
            span.record("outcome", "weak_password");
            return Err(e);
        }
    }

    // FR-AUTH-107 — HIBP breach check on every password set. Runs OUTSIDE
    // the tx because the HIBP API call is a network round-trip — keeping
    // it inside would tie up a Postgres connection during 50-200ms of
    // latency. The HIBP AUDIT ROW (separate concern) lands INSIDE the tx
    // per G-009 below.
    let (pw_hash, hibp_record) = match (&req.kind[..], zeroized_password.as_deref()) {
        ("human", Some(plain)) => {
            let outcome = crate::hibp::check_password(plain).await;
            let (prefix, _suffix) = crate::hibp::sha1_split(plain);
            let outcome_str = match &outcome {
                crate::hibp::HibpOutcome::Allowed => "allowed",
                crate::hibp::HibpOutcome::Breached { .. } => "breached",
                crate::hibp::HibpOutcome::ApiUnreachable => "api-unreachable",
            };
            let breach_count = match &outcome {
                crate::hibp::HibpOutcome::Breached { count } => Some(*count as i32),
                _ => None,
            };
            // Refuse breached passwords.
            if let crate::hibp::HibpOutcome::Breached { count } = outcome {
                span.record("outcome", "password_breached");
                return Err((
                    StatusCode::CONFLICT,
                    Json(json!({
                        "error": "password_breached",
                        "detail": "password appears in known breach corpora",
                        "hibp_count": count,
                    })),
                ));
            }
            let hash = bcrypt::hash(plain, bcrypt::DEFAULT_COST).map_err(|e| {
                span.record("outcome", "error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": format!("bcrypt failed: {e}")})),
                )
            })?;
            (Some(hash), Some((outcome_str, breach_count, prefix)))
        }
        ("human", None) => {
            span.record("outcome", "invalid_input");
            return Err(invalid_input(
                "password",
                "human subject requires password",
            ));
        }
        _ => (None, None),
    };

    let mut tx = state.pg.begin().await.map_err(internal_err)?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(internal_err)?;

    // G-009 — HIBP audit row INSIDE the tx so subject-insert failure rolls
    // it back together with the subject row. Previously the HIBP audit
    // landed via &state.pg (own tx) → orphan rows on subject failure.
    if let Some((outcome_str, breach_count, prefix)) = &hibp_record {
        sqlx::query(
            "INSERT INTO hibp_audit (tenant_id, flow, outcome, breach_count, sha1_prefix)
                  VALUES ($1, 'admin-set', $2, $3, $4)",
        )
        .bind(tenant_id)
        .bind(*outcome_str)
        .bind(*breach_count)
        .bind(prefix)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            span.record("outcome", "error");
            internal_err(e)
        })?;
    }

    let new_id = SubjectId::new();
    let insert_result: Result<(Uuid, Uuid, String, Option<String>, Option<String>, String, String, Vec<String>, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>), sqlx::Error> = sqlx::query_as(
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
    .await;

    let row = match insert_result {
        // G-006 — Structured 409 body. UNIQUE on `(tenant_id, handle)` is
        // the most common conflict; UNIQUE on `(tenant_id, email)` covers
        // the email-taken case. We can't distinguish in the generic
        // is_unique_violation, but the message hint disambiguates.
        Err(sqlx::Error::Database(db)) if db.is_unique_violation() => {
            span.record("outcome", "conflict");
            let constraint = db.constraint().unwrap_or("").to_string();
            let (error, field, value) = if constraint.contains("email") {
                ("email_taken", "email", req.email.clone().unwrap_or_default())
            } else {
                ("handle_taken", "handle", req.handle.clone())
            };
            return Err((
                StatusCode::CONFLICT,
                Json(json!({
                    "error": error,
                    "field": field,
                    "value": value,
                    "tenant_id": tenant_id.to_string(),
                })),
            ));
        }
        Err(other) => {
            span.record("outcome", "error");
            return Err(internal_err(other));
        }
        Ok(r) => r,
    };

    let subject = Subject {
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
    };

    // G-005 — Emit auth.subject_created BRAIN audit row INSIDE the tx
    // (§1 #7 + §1 #12). email_hash16 is the privacy-safe identifier;
    // plaintext email + password hash NEVER appear in the audit chain.
    let caller_subject_id = Uuid::parse_str(&claims.sub).unwrap_or_else(|_| Uuid::nil());
    let request_id = headers.get("x-request-id").and_then(|h| h.to_str().ok());
    let email_hash = subject
        .email
        .as_deref()
        .map(crate::brain_bridge::email_hash16);
    let payload = crate::brain_bridge::SubjectCreatedPayload {
        subject_id: subject.id.as_uuid(),
        tenant_id,
        email_hash16: email_hash,
        roles: &subject.roles,
        created_by_subject_id: caller_subject_id,
        idempotency_key: Some(idem_key),
        request_id,
        kind: &subject.kind,
    };
    if let Err(e) = crate::brain_bridge::emit_subject_created(&mut tx, payload).await {
        span.record("outcome", "error");
        return Err(internal_err(e));
    }

    tx.commit().await.map_err(|e| {
        span.record("outcome", "error");
        internal_err(e)
    })?;

    // Record idempotency cache AFTER commit so failed creates aren't replayed.
    let body = serde_json::to_value(&subject).map_err(internal_err)?;
    crate::idempotency::record(
        &state.pg,
        idem_key,
        route,
        tenant_id,
        StatusCode::CREATED.as_u16() as i16,
        &body,
    )
    .await
    .map_err(internal_err)?;

    span.record("outcome", "created");
    Ok((StatusCode::CREATED, Json(subject)))
}

// ─────────────────────────────────────────────────────────────────────────────
// FR-AUTH-002 §1 #2 + #5 validators
// ─────────────────────────────────────────────────────────────────────────────

/// Validate email against the spec's loose regex `^[^@\s]+@[^@\s]+\.[^@\s]+$`.
/// Implemented in plain Rust (no regex crate) to keep dep count bounded.
fn validate_email(email: &str) -> Result<(), (StatusCode, Json<Value>)> {
    if email.is_empty() {
        return Err(invalid_input("email", "email is empty"));
    }
    if email.contains(|c: char| c.is_whitespace()) {
        return Err(invalid_input("email", "email must not contain whitespace"));
    }
    let at_count = email.matches('@').count();
    if at_count != 1 {
        return Err(invalid_input(
            "email",
            format!("email must contain exactly one '@', got {at_count}"),
        ));
    }
    let (local, domain) = email.split_once('@').expect("@ count verified above");
    if local.is_empty() {
        return Err(invalid_input("email", "email local part is empty"));
    }
    if !domain.contains('.') {
        return Err(invalid_input(
            "email",
            "email domain must contain at least one dot",
        ));
    }
    // Reject leading/trailing dots in the domain.
    if domain.starts_with('.') || domain.ends_with('.') {
        return Err(invalid_input(
            "email",
            "email domain must not start or end with a dot",
        ));
    }
    Ok(())
}

/// FR-AUTH-002 §1 #11 — HTTPS-required check (G-008 slice-3).
///
/// Returns 400 with `{error:"https_required"}` if the request was not
/// proxied over TLS. Detection: the reverse proxy MUST set
/// `X-Forwarded-Proto: https` after TLS termination. Tests short-circuit
/// via `AUTH_TEST_ALLOW_HTTP=1` so integration tests don't need TLS.
///
/// Rationale (§2): plaintext password over HTTP is a credentials-on-the-wire
/// failure mode. Reject as early as possible — before validation runs.
fn require_https(headers: &HeaderMap) -> Result<(), (StatusCode, Json<Value>)> {
    // Test/dev escape hatch — explicit env var so it never leaks into prod
    // by accident. CI integration job + local dev set this; production
    // images MUST NOT.
    if std::env::var("AUTH_TEST_ALLOW_HTTP").is_ok() {
        return Ok(());
    }
    let proto = headers
        .get("x-forwarded-proto")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    if proto.eq_ignore_ascii_case("https") {
        Ok(())
    } else {
        Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "https_required",
                "reason": "password-bearing requests MUST be proxied over TLS (X-Forwarded-Proto: https)"
            })),
        ))
    }
}

/// FR-AUTH-002 §1 #5 role allow-list — closed set for slice 1. FR-AUTH-101
/// expands to 22 roles; this list is the strict subset that callers can
/// assign via the subject-create endpoint. Unknown roles → 400 with the
/// full allow-list for client-side display.
const SLICE1_ROLE_ALLOWLIST: &[&str] = &["tenant-admin", "tenant-member"];

fn validate_roles(roles: &[String]) -> Result<(), (StatusCode, Json<Value>)> {
    for role in roles {
        if !SLICE1_ROLE_ALLOWLIST.contains(&role.as_str()) {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "unknown_role",
                    "role": role,
                    "allowed": SLICE1_ROLE_ALLOWLIST,
                })),
            ));
        }
    }
    Ok(())
}

fn internal_err<E: std::fmt::Display>(e: E) -> (StatusCode, Json<Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"error": e.to_string()})),
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// FR-AUTH-001 §1 #11 — structured 400 error helpers
// ─────────────────────────────────────────────────────────────────────────────
//
// All API-layer validation errors share a single body shape:
//   { "error": "invalid_input", "field": "<name>", "reason": "<human msg>" }
// so the client can render actionable per-field error UI without inspecting
// logs. The constant `error` enum-value is "invalid_input" for validation
// failures; other 400-class errors (e.g. missing required header) use
// "missing_header" / etc. but keep the same {error, field, reason} triple.

/// FR-AUTH-001 §1 #1 + §1 #10 — assert caller is root-admin in tenant 0.
///
/// Two conjunctive conditions:
///   1. `claims.tenant_id` parses as the nil-UUID (the canonical "tenant 0").
///   2. `claims.roles` contains the literal `"root-admin"`.
///
/// Either failing → 403 FORBIDDEN with body
/// `{"error":"forbidden","needed":"root-admin in tenant 0"}` — explicit about
/// WHAT permission would have granted access (per §1 #10) so operators can
/// fix the role assignment without inspecting logs or source.
///
/// The `verify_jwt` middleware (FR-AUTH-004) has already validated the JWT
/// signature + issuer + expiry; this is a defence-in-depth role guard at
/// the handler layer. Even if the middleware were bypassed (e.g. by a
/// route-mounting mistake), this check still fires.
fn require_root_admin_in_tenant_0(claims: &Claims) -> Result<(), (StatusCode, Json<Value>)> {
    const NEEDED: &str = "root-admin in tenant 0";
    let caller_tenant = uuid::Uuid::parse_str(&claims.tenant_id).map_err(|_| {
        // A claims.tenant_id that doesn't parse as a UUID is suspicious —
        // surface as forbidden (not 500) so attackers don't learn which
        // form is "almost right".
        (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "forbidden", "needed": NEEDED})),
        )
    })?;
    if caller_tenant != uuid::Uuid::nil() {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "forbidden", "needed": NEEDED})),
        ));
    }
    if !claims.roles.iter().any(|r| r == "root-admin") {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "forbidden", "needed": NEEDED})),
        ));
    }
    Ok(())
}

/// Build a 400 BAD_REQUEST with structured `{error, field, reason}` body.
fn invalid_input(field: &str, reason: impl Into<String>) -> (StatusCode, Json<Value>) {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({
            "error": "invalid_input",
            "field": field,
            "reason": reason.into(),
        })),
    )
}

/// Validate tenant `slug` per FR-AUTH-001 §1 #2:
///   * 1..=40 chars
///   * first char `[a-z]`
///   * remaining chars `[a-z0-9-]`
///
/// Matches the Postgres CHECK constraint at services/auth/migrations/0001_tenants.sql
/// (defence in depth — the DB will also reject malformed slugs, but the API-layer
/// check produces a structured body instead of a generic 500 on constraint failure).
fn validate_slug(slug: &str) -> Result<(), (StatusCode, Json<Value>)> {
    if slug.is_empty() {
        return Err(invalid_input("slug", "slug is empty (1..=40 chars required)"));
    }
    if slug.len() > 40 {
        return Err(invalid_input(
            "slug",
            format!("slug is {} chars (max 40)", slug.len()),
        ));
    }
    let mut chars = slug.chars();
    let first = chars.next().expect("non-empty verified above");
    if !first.is_ascii_lowercase() {
        return Err(invalid_input(
            "slug",
            format!("slug must start with [a-z], got '{first}'"),
        ));
    }
    for c in chars {
        if !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
            return Err(invalid_input(
                "slug",
                format!("slug contains invalid char '{c}'; allowed: [a-z0-9-]"),
            ));
        }
    }
    Ok(())
}

/// Validate tenant `display_name` per FR-AUTH-001 §1 #2:
///   * 1..=80 chars
///   * no null bytes
fn validate_display_name(name: &str) -> Result<(), (StatusCode, Json<Value>)> {
    if name.is_empty() {
        return Err(invalid_input(
            "display_name",
            "display_name is empty (1..=80 chars required)",
        ));
    }
    if name.chars().count() > 80 {
        return Err(invalid_input(
            "display_name",
            format!("display_name is {} chars (max 80)", name.chars().count()),
        ));
    }
    if name.contains('\0') {
        return Err(invalid_input(
            "display_name",
            "display_name contains a null byte",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod validate_tests {
    use super::*;

    // ECM-003 — slug at exact length boundaries.
    #[test]
    fn slug_min_length_one_char_accepted() {
        assert!(validate_slug("a").is_ok());
    }

    #[test]
    fn slug_max_length_forty_chars_accepted() {
        let s = "a".repeat(40);
        assert!(validate_slug(&s).is_ok());
    }

    #[test]
    fn slug_forty_one_chars_rejected() {
        let s = "a".repeat(41);
        let (status, _) = validate_slug(&s).unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    // ECM-006 — malformed slug.
    #[test]
    fn slug_starting_with_digit_rejected() {
        let (status, _) = validate_slug("1foo").unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn slug_with_uppercase_rejected() {
        let (status, _) = validate_slug("Acme").unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn slug_with_special_char_rejected() {
        let (status, _) = validate_slug("a_b").unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn slug_empty_rejected() {
        let (status, _) = validate_slug("").unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    // ECM-004 — display_name boundaries.
    #[test]
    fn display_name_min_one_char_accepted() {
        assert!(validate_display_name("x").is_ok());
    }

    #[test]
    fn display_name_max_eighty_chars_accepted() {
        let s = "n".repeat(80);
        assert!(validate_display_name(&s).is_ok());
    }

    #[test]
    fn display_name_eighty_one_chars_rejected() {
        let s = "n".repeat(81);
        let (status, _) = validate_display_name(&s).unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    // ECM-007 — null byte in display_name.
    #[test]
    fn display_name_with_null_byte_rejected() {
        let (status, _) = validate_display_name("foo\0bar").unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn invalid_input_body_shape_is_error_field_reason() {
        let (_, Json(body)) = invalid_input("slug", "reason msg");
        assert_eq!(body["error"], "invalid_input");
        assert_eq!(body["field"], "slug");
        assert_eq!(body["reason"], "reason msg");
    }

    // ─── G-003 — root-admin-in-tenant-0 authz (FR-AUTH-001 §1 #1) ────────

    fn build_claims(tenant_id: &str, roles: Vec<&str>) -> Claims {
        Claims {
            iss: "https://auth.cyberos.local".into(),
            sub: uuid::Uuid::new_v4().to_string(),
            aud: vec!["cyberos".into()],
            exp: 0,
            iat: 0,
            nbf: 0,
            jti: uuid::Uuid::new_v4().to_string(),
            tenant_id: tenant_id.into(),
            kind: "human".into(),
            scope_grants: vec!["admin:tenants".into()],
            roles: roles.into_iter().map(String::from).collect(),
            rbac_v: Some(1),
            agent_persona: None,
            traceparent: None,
        }
    }

    #[test]
    fn root_admin_in_tenant_0_passes() {
        let claims = build_claims("00000000-0000-0000-0000-000000000000", vec!["root-admin"]);
        assert!(require_root_admin_in_tenant_0(&claims).is_ok());
    }

    // ECM-012 — caller is in a non-root tenant.
    #[test]
    fn non_root_tenant_caller_returns_403() {
        let claims = build_claims("11111111-1111-1111-1111-111111111111", vec!["root-admin"]);
        let (status, Json(body)) = require_root_admin_in_tenant_0(&claims).unwrap_err();
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(body["error"], "forbidden");
        assert_eq!(body["needed"], "root-admin in tenant 0");
    }

    // ECM-013 — caller is in tenant 0 but lacks the role.
    #[test]
    fn root_tenant_without_root_admin_role_returns_403() {
        let claims = build_claims("00000000-0000-0000-0000-000000000000", vec!["tenant-admin"]);
        let (status, Json(body)) = require_root_admin_in_tenant_0(&claims).unwrap_err();
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(body["needed"], "root-admin in tenant 0");
    }

    #[test]
    fn root_tenant_with_empty_roles_returns_403() {
        let claims = build_claims("00000000-0000-0000-0000-000000000000", vec![]);
        let (status, _) = require_root_admin_in_tenant_0(&claims).unwrap_err();
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[test]
    fn malformed_tenant_id_returns_403_not_500() {
        let claims = build_claims("not-a-uuid", vec!["root-admin"]);
        let (status, Json(body)) = require_root_admin_in_tenant_0(&claims).unwrap_err();
        // 403 (not 500) so the failure mode doesn't leak parser internals
        // to a hostile caller probing for "almost right" tenant IDs.
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(body["error"], "forbidden");
    }

    // ─── FR-AUTH-002 G-001 — email validation ────────────────────────────

    #[test]
    fn valid_email_with_one_at_and_dotted_domain_passes() {
        assert!(validate_email("alice@example.com").is_ok());
        assert!(validate_email("a@b.co").is_ok());
    }

    #[test]
    fn email_without_at_rejected() {
        let (s, _) = validate_email("noat").unwrap_err();
        assert_eq!(s, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn email_with_two_ats_rejected() {
        let (s, _) = validate_email("a@b@c.com").unwrap_err();
        assert_eq!(s, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn email_with_whitespace_rejected() {
        let (s, _) = validate_email("alice@example .com").unwrap_err();
        assert_eq!(s, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn email_without_dotted_domain_rejected() {
        let (s, _) = validate_email("alice@localhost").unwrap_err();
        assert_eq!(s, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn email_empty_rejected() {
        let (s, _) = validate_email("").unwrap_err();
        assert_eq!(s, StatusCode::BAD_REQUEST);
    }

    // ─── FR-AUTH-002 G-003 — role allow-list ─────────────────────────────

    #[test]
    fn empty_roles_list_passes() {
        assert!(validate_roles(&[]).is_ok());
    }

    #[test]
    fn allowed_roles_pass() {
        assert!(validate_roles(&["tenant-admin".into()]).is_ok());
        assert!(validate_roles(&["tenant-member".into()]).is_ok());
        assert!(validate_roles(&["tenant-admin".into(), "tenant-member".into()]).is_ok());
    }

    #[test]
    fn unknown_role_rejected_with_allowlist_in_body() {
        let (s, Json(body)) = validate_roles(&["tenant-superadmin".into()]).unwrap_err();
        assert_eq!(s, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"], "unknown_role");
        assert_eq!(body["role"], "tenant-superadmin");
        // Allowlist included so the client can render an actionable error.
        let allowed = body["allowed"].as_array().expect("allowed is array");
        assert!(allowed.iter().any(|v| v == "tenant-admin"));
        assert!(allowed.iter().any(|v| v == "tenant-member"));
    }

    // ─── FR-AUTH-002 G-008 — HTTPS gate ──────────────────────────────────

    fn mk_headers(pairs: &[(&str, &str)]) -> HeaderMap {
        let mut h = HeaderMap::new();
        for (k, v) in pairs {
            h.insert(
                axum::http::HeaderName::from_bytes(k.as_bytes()).unwrap(),
                axum::http::HeaderValue::from_str(v).unwrap(),
            );
        }
        h
    }

    #[test]
    fn https_proto_accepted() {
        std::env::remove_var("AUTH_TEST_ALLOW_HTTP");
        let h = mk_headers(&[("x-forwarded-proto", "https")]);
        assert!(require_https(&h).is_ok());
    }

    #[test]
    fn http_proto_rejected() {
        std::env::remove_var("AUTH_TEST_ALLOW_HTTP");
        let h = mk_headers(&[("x-forwarded-proto", "http")]);
        let (s, Json(body)) = require_https(&h).unwrap_err();
        assert_eq!(s, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"], "https_required");
    }

    #[test]
    fn missing_proto_header_rejected() {
        std::env::remove_var("AUTH_TEST_ALLOW_HTTP");
        let h = mk_headers(&[]);
        let (s, _) = require_https(&h).unwrap_err();
        assert_eq!(s, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_env_var_short_circuits() {
        std::env::set_var("AUTH_TEST_ALLOW_HTTP", "1");
        let h = mk_headers(&[]); // no x-forwarded-proto at all
        assert!(require_https(&h).is_ok());
        std::env::remove_var("AUTH_TEST_ALLOW_HTTP");
    }

    #[test]
    fn case_insensitive_proto_match() {
        std::env::remove_var("AUTH_TEST_ALLOW_HTTP");
        let h = mk_headers(&[("x-forwarded-proto", "HTTPS")]);
        assert!(require_https(&h).is_ok());
    }

    #[test]
    fn typo_in_role_caught() {
        // §2 rationale: "free-form role strings invite typos
        // (`\"tenant_admin\"` vs `\"tenant-admin\"`) — the closed allow-list
        // catches typos at the API boundary." Pin this.
        let (s, _) = validate_roles(&["tenant_admin".into()]).unwrap_err(); // underscore not hyphen
        assert_eq!(s, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn root_admin_alongside_other_roles_still_passes() {
        let claims = build_claims(
            "00000000-0000-0000-0000-000000000000",
            vec!["tenant-admin", "root-admin", "auditor"],
        );
        assert!(require_root_admin_in_tenant_0(&claims).is_ok());
    }
}

// ---------------------------------------------------------------------------
// FR-AUTH-005 — list + revoke + unrevoke
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    /// Opaque cursor — base64 of the last seen id.
    pub cursor: Option<String>,
    /// Page size; capped at 100.
    pub limit: Option<i64>,
}

async fn list_tenants(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    // List tenants under root context only. Non-root sees zero rows via RLS.
    let mut tx = state.pg.begin().await.map_err(internal_err)?;
    sqlx::query("SET LOCAL app.current_tenant_id = '00000000-0000-0000-0000-000000000000'")
        .execute(&mut *tx).await.map_err(internal_err)?;

    let limit = q.limit.unwrap_or(50).clamp(1, 100);
    let cursor_uuid = parse_cursor(q.cursor.as_deref());

    let rows: Vec<(Uuid, String, String, String, String, String, String, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        "SELECT id, slug, display_name, country, plan_tier, status, residency, created_at, updated_at
            FROM tenants
           WHERE ($1::uuid IS NULL OR id > $1)
        ORDER BY id ASC
           LIMIT $2",
    )
    .bind(cursor_uuid)
    .bind(limit)
    .fetch_all(&mut *tx)
    .await
    .map_err(internal_err)?;
    tx.commit().await.map_err(internal_err)?;

    let next_cursor = rows.last().map(|r| make_cursor(r.0));
    let items: Vec<Tenant> = rows.into_iter().map(|r| Tenant {
        id: TenantId(r.0), slug: r.1, display_name: r.2, country: r.3,
        plan_tier: r.4, status: r.5, residency: r.6,
        created_at: r.7, updated_at: r.8,
    }).collect();

    Ok((StatusCode::OK, Json(json!({"items": items, "next_cursor": next_cursor}))))
}

async fn list_subjects(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<ListQuery>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id).map_err(internal_err)?;
    let mut tx = state.pg.begin().await.map_err(internal_err)?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string()).execute(&mut *tx).await.map_err(internal_err)?;

    let limit = q.limit.unwrap_or(50).clamp(1, 100);
    let cursor_uuid = parse_cursor(q.cursor.as_deref());

    let rows: Vec<(Uuid, Uuid, String, Option<String>, Option<String>, String, String, Vec<String>, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        "SELECT id, tenant_id, handle, display_name, email, kind, status, roles, created_at, updated_at
            FROM subjects
           WHERE ($1::uuid IS NULL OR id > $1)
        ORDER BY id ASC
           LIMIT $2",
    )
    .bind(cursor_uuid)
    .bind(limit)
    .fetch_all(&mut *tx)
    .await
    .map_err(internal_err)?;
    tx.commit().await.map_err(internal_err)?;

    let next_cursor = rows.last().map(|r| make_cursor(r.0));
    let items: Vec<Subject> = rows.into_iter().map(|r| Subject {
        id: SubjectId(r.0), tenant_id: TenantId(r.1), handle: r.2,
        display_name: r.3, email: r.4, kind: r.5, status: r.6,
        roles: r.7, created_at: r.8, updated_at: r.9,
    }).collect();
    Ok((StatusCode::OK, Json(json!({"items": items, "next_cursor": next_cursor}))))
}

async fn revoke_subject(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<Value>)> {
    flip_subject_status(state, claims, id, "revoked").await
}

async fn unrevoke_subject(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<Value>)> {
    flip_subject_status(state, claims, id, "active").await
}

async fn flip_subject_status(
    state: AppState,
    claims: Claims,
    id: Uuid,
    new_status: &str,
) -> Result<StatusCode, (StatusCode, Json<Value>)> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id).map_err(internal_err)?;
    let mut tx = state.pg.begin().await.map_err(internal_err)?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string()).execute(&mut *tx).await.map_err(internal_err)?;

    let result = sqlx::query("UPDATE subjects SET status = $1, updated_at = NOW() WHERE id = $2")
        .bind(new_status).bind(id)
        .execute(&mut *tx).await.map_err(internal_err)?;
    tx.commit().await.map_err(internal_err)?;
    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, Json(json!({"error": "subject not found in this tenant"}))));
    }
    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// FR-AUTH-004 — JWT issuance + JWKS
// ---------------------------------------------------------------------------

/// Body for `POST /v1/auth/token`. `grant_type` selects the variant —
/// `password` requires `tenant_slug` + `handle` + `password`; `refresh_token`
/// requires only `refresh_token`. Each variant ignores the other's fields.
#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    pub grant_type: String,

    // password grant
    pub tenant_slug: Option<String>,
    pub handle: Option<String>,
    pub password: Option<String>,

    // refresh_token grant
    pub refresh_token: Option<String>,

    #[serde(default)]
    pub scope: Vec<String>,
}

async fn issue_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<TokenRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let traceparent = headers
        .get("traceparent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());
    let caller = caller_ip(&headers);
    let ua = headers.get("user-agent").and_then(|h| h.to_str().ok()).map(String::from);

    match req.grant_type.as_str() {
        "password" => password_grant(&state, req, traceparent, caller, ua).await,
        "refresh_token" => refresh_grant(&state, req, traceparent).await,
        other => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": format!("unsupported grant_type '{other}' — supported: 'password', 'refresh_token'"),
            })),
        )),
    }
}

async fn password_grant(
    state: &AppState,
    req: TokenRequest,
    traceparent: Option<String>,
    caller_ip: std::net::IpAddr,
    user_agent: Option<String>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let tenant_slug = req.tenant_slug.as_deref().ok_or_else(|| (
        StatusCode::BAD_REQUEST,
        Json(json!({"error": "password grant requires `tenant_slug`"}))
    ))?;
    let handle = req.handle.as_deref().ok_or_else(|| (
        StatusCode::BAD_REQUEST,
        Json(json!({"error": "password grant requires `handle`"}))
    ))?;
    let password = req.password.as_deref().ok_or_else(|| (
        StatusCode::BAD_REQUEST,
        Json(json!({"error": "password grant requires `password`"}))
    ))?;

    // Look up tenant + subject under root context.
    let mut tx = state.pg.begin().await.map_err(internal_err)?;
    sqlx::query("SET LOCAL app.current_tenant_id = '00000000-0000-0000-0000-000000000000'")
        .execute(&mut *tx).await.map_err(internal_err)?;

    let row: Option<(Uuid, Uuid, String, String, Option<String>, Vec<String>)> = sqlx::query_as(
        "SELECT s.id, s.tenant_id, s.kind, s.status, s.password_hash, s.roles
             FROM subjects s
             JOIN tenants t ON t.id = s.tenant_id
            WHERE t.slug = $1 AND s.handle = $2",
    )
    .bind(tenant_slug)
    .bind(handle)
    .fetch_optional(&mut *tx)
    .await
    .map_err(internal_err)?;
    tx.commit().await.map_err(internal_err)?;

    let (sub_id, tenant_id, kind, status, pw_hash, roles) = match row {
        Some(r) => r,
        None => return Err((StatusCode::UNAUTHORIZED, Json(json!({"error": "invalid credentials"})))),
    };
    if status != "active" {
        return Err((StatusCode::UNAUTHORIZED, Json(json!({"error": "subject is not active"}))));
    }
    let pw_hash = match pw_hash {
        Some(h) => h,
        None => return Err((StatusCode::UNAUTHORIZED, Json(json!({"error": "agent/system subjects use a different grant"})))),
    };
    let ok = bcrypt::verify(password, &pw_hash).map_err(|e| internal_err(e))?;
    if !ok {
        return Err((StatusCode::UNAUTHORIZED, Json(json!({"error": "invalid credentials"}))));
    }

    let svc = JwtService::new(state.pg.clone(), state.jwt_issuer.clone());
    let granted = effective_scopes(req.scope, &roles);

    // FR-AUTH-101 §1 #8 — embed the subject's full role membership + the
    // live catalogue version. Falls back to the legacy `subjects.roles`
    // array column if `subject_roles` table doesn't exist (pre-101 schema).
    let assigned_roles = load_subject_roles(&state, tenant_id, sub_id, &roles).await;
    let rbac_v = state.role_matrix.read().await.version();

    let tokens = svc.issue(
        TenantId(tenant_id),
        SubjectId(sub_id),
        &kind,
        granted,
        assigned_roles,
        Some(rbac_v),
        None,
        traceparent,
    ).await.map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"error": format!("jwt issuance failed: {e}")})),
    ))?;

    // FR-AUTH-106 — record login + assess. Slice-3 wraps the detector chain
    // with per-tenant policy + CIDR allowlist + anonymous-IP + sticky-
    // suppression. `assess_login` returns one of Clear / Challenge / Block.
    let deps = crate::travel::AssessDeps {
        pool: &state.pg,
        geoip: &state.geoip,
        policy_cache: &state.travel_policy,
        sticky_suppress: &state.sticky_suppress,
    };
    let travel = crate::travel::assess_login(
        &deps,
        tenant_id,
        sub_id,
        "password",
        caller_ip,
        user_agent.as_deref(),
    )
    .await
    .ok();
    let body = match travel {
        Some(crate::travel::TravelOutcome::Block { kind, .. }) => {
            // Policy says block — refuse the login outright. No token issued.
            return Err((
                StatusCode::FORBIDDEN,
                Json(json!({
                    "error": "impossible_travel_blocked",
                    "kind": kind,
                })),
            ));
        }
        Some(crate::travel::TravelOutcome::Challenge { kind, login_id, .. }) => {
            // Token is issued but the client MUST complete an MFA challenge
            // before using it. The needs_mfa_challenge flag tells the client
            // to call /v1/auth/mfa/verify (TOTP) or /v1/auth/passkey/login.
            json!({
                "access_token": tokens.access_token,
                "refresh_token": tokens.refresh_token,
                "token_type": tokens.token_type,
                "expires_in": tokens.expires_in,
                "kid": tokens.kid,
                "needs_mfa_challenge": true,
                "challenge_reason": kind,
                "challenge_login_id": login_id,
            })
        }
        _ => token_response_body(&tokens),
    };

    Ok((StatusCode::OK, Json(body)))
}

/// Public wrapper for `load_subject_roles` — used by OIDC callback handler
/// to embed roles + rbac_v in the access token it mints.
pub async fn load_subject_roles_pub(
    state: &AppState,
    tenant_id: Uuid,
    subject_id: Uuid,
    legacy_roles: &[String],
) -> Vec<String> {
    load_subject_roles(state, tenant_id, subject_id, legacy_roles).await
}

/// FR-AUTH-106 — extract caller IP from request headers (prefers `X-Forwarded-For`
/// first hop, falls back to a synthesised 0.0.0.0 if no header — slice 1 only;
/// production reverse-proxy is configured to always send this).
pub fn caller_ip(headers: &HeaderMap) -> std::net::IpAddr {
    if let Some(v) = headers.get("x-forwarded-for").and_then(|h| h.to_str().ok()) {
        if let Some(first) = v.split(',').next() {
            if let Ok(ip) = first.trim().parse::<std::net::IpAddr>() {
                return ip;
            }
        }
    }
    if let Some(v) = headers.get("x-real-ip").and_then(|h| h.to_str().ok()) {
        if let Ok(ip) = v.trim().parse::<std::net::IpAddr>() {
            return ip;
        }
    }
    std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0))
}

/// Pull the subject's role list. Prefers `subject_roles` (FR-AUTH-101);
/// falls back to the legacy `subjects.roles` array column.
async fn load_subject_roles(
    state: &AppState,
    tenant_id: Uuid,
    subject_id: Uuid,
    legacy_roles: &[String],
) -> Vec<String> {
    let mut tx = match state.pg.begin().await {
        Ok(t) => t,
        Err(_) => return legacy_roles.to_vec(),
    };
    let _ = sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx).await;

    let res: Result<Vec<(String,)>, sqlx::Error> = sqlx::query_as(
        "SELECT role FROM subject_roles WHERE subject_id = $1",
    )
    .bind(subject_id)
    .fetch_all(&mut *tx)
    .await;
    let _ = tx.commit().await;

    match res {
        Ok(rows) if !rows.is_empty() => rows.into_iter().map(|(r,)| r).collect(),
        // No FR-AUTH-101 rows yet — fall back to the legacy text[] column.
        _ => legacy_roles.to_vec(),
    }
}

/// FR-AUTH-004 — `grant_type=refresh_token` exchange.
///
/// Validates the presented refresh JWT (must carry aud=`refresh`), confirms
/// the subject is still `active`, then mints a fresh access+refresh pair.
/// Note: refresh-token revocation is FR-AUTH-005's responsibility; here we
/// only check the subject's `status` field, which is the operator's
/// disable-now lever.
async fn refresh_grant(
    state: &AppState,
    req: TokenRequest,
    traceparent: Option<String>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let refresh = req.refresh_token.as_deref().ok_or_else(|| (
        StatusCode::BAD_REQUEST,
        Json(json!({"error": "refresh_token grant requires `refresh_token`"}))
    ))?;

    let svc = JwtService::new(state.pg.clone(), state.jwt_issuer.clone());
    let claims = svc.verify(refresh).await.map_err(|e| (
        StatusCode::UNAUTHORIZED,
        Json(json!({"error": format!("invalid refresh token: {e}")})),
    ))?;

    // Audience must include "refresh".
    if !claims.aud.iter().any(|a| a == "refresh") {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "token aud does not include 'refresh' — not a refresh token"})),
        ));
    }

    let sub_id = Uuid::parse_str(&claims.sub).map_err(|e| internal_err(e))?;
    let tenant_id = Uuid::parse_str(&claims.tenant_id).map_err(|e| internal_err(e))?;

    // Confirm the subject is still active under root context (refresh flow
    // crosses tenant scope checks).
    let mut tx = state.pg.begin().await.map_err(internal_err)?;
    sqlx::query("SET LOCAL app.current_tenant_id = '00000000-0000-0000-0000-000000000000'")
        .execute(&mut *tx).await.map_err(internal_err)?;
    let status_row: Option<(String, String, Vec<String>)> = sqlx::query_as(
        "SELECT status, kind, roles FROM subjects WHERE id = $1 AND tenant_id = $2",
    )
    .bind(sub_id)
    .bind(tenant_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(internal_err)?;
    tx.commit().await.map_err(internal_err)?;

    let (status, kind, roles) = match status_row {
        Some(r) => r,
        None => return Err((StatusCode::UNAUTHORIZED, Json(json!({"error": "subject no longer exists"})))),
    };
    if status != "active" {
        return Err((StatusCode::UNAUTHORIZED, Json(json!({"error": "subject is no longer active"}))));
    }

    // Refresh re-issues with the previously-granted scopes by default, unless
    // the caller narrows via `scope`. Never widens.
    let prior = claims.scope_grants.clone();
    let granted = if req.scope.is_empty() {
        prior
    } else {
        req.scope.into_iter()
            .filter(|s| prior.iter().any(|p| s == p))
            .collect()
    };
    // Also re-restrict by current subject roles in case roles changed.
    let granted = effective_scopes(granted, &roles);

    // Refresh re-mints roles + rbac_v from the live state, not the prior token —
    // catches catalogue bumps + subject role-revokes that happened mid-session.
    let fresh_roles = load_subject_roles(state, tenant_id, sub_id, &roles).await;
    let live_rbac_v = state.role_matrix.read().await.version();

    let tokens = svc.issue(
        TenantId(tenant_id),
        SubjectId(sub_id),
        &kind,
        granted,
        fresh_roles,
        Some(live_rbac_v),
        claims.agent_persona,
        traceparent,
    ).await.map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"error": format!("jwt issuance failed: {e}")})),
    ))?;

    Ok((StatusCode::OK, Json(token_response_body(&tokens))))
}

/// Intersect requested scope with the subject's role-granted scopes.
/// The full RBAC mapping lives with FR-AUTH-101; until then we treat
/// "admin" as an umbrella role that grants any scope.
fn effective_scopes(requested: Vec<String>, roles: &[String]) -> Vec<String> {
    if requested.is_empty() {
        // Default to mirroring roles 1:1 — gives password-grant clients
        // something to work with before FR-AUTH-101 lands.
        return roles.to_vec();
    }
    requested
        .into_iter()
        .filter(|s| roles.iter().any(|r| s.starts_with(r) || r == "admin"))
        .collect()
}

fn token_response_body(tokens: &crate::jwt::IssuedTokens) -> Value {
    json!({
        "access_token": tokens.access_token,
        "refresh_token": tokens.refresh_token,
        "token_type": tokens.token_type,
        "expires_in": tokens.expires_in,
        "kid": tokens.kid,
    })
}

async fn jwks(State(state): State<AppState>) -> (StatusCode, Json<Value>) {
    let svc = JwtService::new(state.pg.clone(), state.jwt_issuer.clone());
    match svc.jwks_for_publication().await {
        Ok(doc) => (StatusCode::OK, Json(json!({"keys": doc.keys}))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("jwks load failed: {e}")})),
        ),
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn parse_cursor(s: Option<&str>) -> Option<Uuid> {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    let raw = s?;
    let bytes = URL_SAFE_NO_PAD.decode(raw).ok()?;
    Uuid::from_slice(&bytes).ok()
}

fn make_cursor(id: Uuid) -> String {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    URL_SAFE_NO_PAD.encode(id.as_bytes())
}

// `require_tenant_header` was removed once the verify_jwt middleware landed —
// admin handlers now read `tenant_id` from the verified `Claims` extension
// instead of trusting an `X-Tenant-Id` header.
