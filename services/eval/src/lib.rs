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
pub mod db;
pub mod gate;

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};

#[derive(Clone)]
pub struct AppState {
    /// The EVAL governance Postgres (holds the notice / ack / category / grant / retention tables, all
    /// per-tenant RLS).
    pub pool: db::Pool,
    /// The memory module's Postgres (holds l1_audit_log). When set, EVAL appends a hash-chained audit row
    /// per governance mutation (clause 12); when unset (tests/local), the event is logged. Mirrors chat.
    pub audit_pool: Option<db::Pool>,
    /// Crate version, surfaced in /healthz for at-a-glance build identification.
    pub version: &'static str,
}

impl AppState {
    /// Build state with no audit pool (tests / local). Governance mutations are logged, not chained.
    pub fn new(pool: db::Pool) -> Self {
        Self {
            pool,
            audit_pool: None,
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
