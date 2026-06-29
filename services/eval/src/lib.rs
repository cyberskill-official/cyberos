//! CyberOS EVAL governance core (FR-EVAL-001 slice 1): router, shared state, health, and a small error
//! helper. A first-party Rust service on the CyberOS identity (FR-AUTH-003 tenant RLS), Postgres, and the
//! memory audit chain (services/shared/cyberos-audit-chain). This is the Phase-0 gate every BRAIN/EVAL
//! capture (FR-MEMORY-121/122) and evaluation (FR-EVAL-003) consults before it records or evaluates a
//! subject. Slice 1 = the governance tables, the per-subject consent/acknowledgment gate, the
//! founder/manager-of/self access check, a /healthz, and an l1_audit_log row on every governance mutation.
//!
//! OUT OF SCOPE (DEC-2525): fully-covert / no-notice collection. The disclosed monitoring notice plus the
//! acknowledgment gate are the boundary of what this system does.
//!
//! QUIET OPERATING MODE (founder decision 2026-06-30): the product shows employees NO monitoring /
//! evaluation surface by default; access is founder + managers only; acknowledgment is normally the signed
//! employment-document clause recorded by HR (ack_source 'signed_contract'), not an in-app click. There is
//! no switch that captures a subject who has no acknowledgment row at all.

pub mod access;
pub mod audit;
pub mod auth;
pub mod db;
pub mod gate;
pub mod handlers;

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};

#[derive(Clone)]
pub struct AppState {
    /// The EVAL governance Postgres (holds the notice / ack / category / grant / retention tables, all
    /// per-tenant RLS).
    pub pool: db::Pool,
    /// The memory module's Postgres (holds l1_audit_log). When set, EVAL appends a hash-chained audit row
    /// per governance mutation (clause 12); when unset (tests/local), the event is logged. Mirrors chat.
    pub audit_pool: Option<db::Pool>,
    /// Verifies the CyberOS access token (RS256 against AUTH's JWKS, FR-AUTH-004) and yields the caller
    /// identity for every governance endpoint (slice 2). Built once at boot; mirrors chat.
    pub authenticator: Arc<auth::Authenticator>,
    /// Crate version, surfaced in /healthz for at-a-glance build identification.
    pub version: &'static str,
}

impl AppState {
    /// Build state with no audit pool and an HS256 test verifier (tests / local). Governance mutations
    /// are logged, not chained. Production builds state in `main` with the JWKS-backed authenticator.
    pub fn new(pool: db::Pool) -> Self {
        Self {
            pool,
            audit_pool: None,
            authenticator: Arc::new(auth::Authenticator::from_hs256_secret(b"eval-dev-secret")),
            version: env!("CARGO_PKG_VERSION"),
        }
    }
}

/// Map any error to a 500 carrying its text. Mirrors `cyberos_chat::internal`.
pub fn internal<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

pub fn router(state: AppState) -> Router {
    let mut app = Router::new()
        .route("/healthz", get(health))
        // Notice (clause 1): publish a new version (founder), read the current one (founder/manager).
        .route(
            "/v1/eval/notice",
            post(handlers::publish_notice).get(handlers::get_notice),
        )
        // Data-category registry (clause 4, 5): register/update a category (founder).
        .route("/v1/eval/categories", post(handlers::register_category))
        // Acknowledgment (clause 2, 3): the HR action that flips the consent gate for a subject.
        .route("/v1/eval/ack", post(handlers::record_ack))
        // Access grants (clause 7, 8): grant (founder) and revoke (founder).
        .route("/v1/eval/access", post(handlers::grant_access))
        .route("/v1/eval/access/revoke", post(handlers::revoke_access))
        // Retention (clause 6): set/replace a per-category retention policy (founder).
        .route("/v1/eval/retention", post(handlers::set_retention))
        // Data-subject self surface (clause 10): own record + file a request.
        .route("/v1/eval/me", get(handlers::get_me))
        .route("/v1/eval/me/requests", post(handlers::file_request))
        .with_state(state);

    // Opt-in permissive CORS so a local browser (a future EVAL governance-status console) can call the
    // service cross-origin in dev. Off in production (one origin behind Caddy). Mirrors CHAT_DEV_CORS.
    if std::env::var("EVAL_DEV_CORS").is_ok() {
        app = app.layer(tower_http::cors::CorsLayer::permissive());
    }
    app
}

/// Liveness + a Postgres `SELECT 1` round-trip, mirroring chat's health. Confirms the governance DB is
/// reachable before any caller relies on the gate.
async fn health(
    State(st): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    sqlx::query("SELECT 1")
        .execute(&st.pool)
        .await
        .map_err(internal)?;
    Ok(Json(serde_json::json!({
        "status": "ok",
        "service": "cyberos-eval",
        "version": st.version,
    })))
}
