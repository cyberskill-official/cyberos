//! CyberOS-native chat (FR-CHAT-101 slice 1): router, shared state, health, and a small error helper.
//! A first-party Rust service on the CyberOS identity (FR-AUTH-110), Postgres with per-tenant RLS, and
//! the memory audit chain - replacing the Mattermost dependency.

pub mod attachments;
pub mod audit;
pub mod auth;
pub mod channels;
pub mod db;
pub mod devices;
pub mod members;
pub mod messages;
pub mod push;
pub mod read;
pub mod realtime;

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};

#[derive(Clone)]
pub struct AppState {
    pub pool: db::Pool,
    /// The memory module's Postgres (holds l1_audit_log). When set, chat appends a hash-chained audit
    /// row per channel/message event; when unset (tests/local), the event is logged.
    pub audit_pool: Option<db::Pool>,
    pub authenticator: Arc<auth::Authenticator>,
    pub hub: realtime::Hub,
    pub presence: realtime::Presence,
}

/// Map any error to a 500 carrying its text.
pub fn internal<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

pub fn router(state: AppState) -> Router {
    let mut app = Router::new()
        .route("/healthz", get(health))
        .route(
            "/v1/chat/channels",
            post(channels::create).get(channels::list),
        )
        .route(
            "/v1/chat/channels/:id/messages",
            post(messages::post).get(messages::list),
        )
        .route(
            "/v1/chat/channels/:id/messages/:msg",
            axum::routing::patch(messages::edit).delete(messages::delete),
        )
        .route(
            "/v1/chat/channels/:id/members",
            post(members::add).get(members::list),
        )
        .route(
            "/v1/chat/channels/:id/members/:subject",
            axum::routing::delete(members::remove),
        )
        .route("/v1/chat/channels/:id/search", get(messages::search))
        .route(
            "/v1/chat/channels/:id/attachments",
            post(attachments::upload),
        )
        .route("/v1/chat/attachments/:att", get(attachments::download))
        .route(
            "/v1/chat/channels/:id/presence",
            get(realtime::presence_list),
        )
        .route("/v1/chat/channels/:id/read", post(read::mark))
        .route("/v1/chat/channels/:id/unread", get(read::unread))
        .route("/v1/chat/devices", post(devices::register))
        .route("/v1/chat/ws", get(realtime::ws_handler))
        .with_state(state);

    // FR-APP-007: opt-in permissive CORS so a local browser (the CDS chat web client) can call the
    // service cross-origin in dev. In production Caddy serves the page and proxies the service under
    // one origin, so this stays off. Set CHAT_DEV_CORS=1 only for local development.
    if std::env::var("CHAT_DEV_CORS").is_ok() {
        app = app.layer(tower_http::cors::CorsLayer::permissive());
    }
    app
}

async fn health(
    State(st): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    sqlx::query("SELECT 1")
        .execute(&st.pool)
        .await
        .map_err(internal)?;
    Ok(Json(serde_json::json!({"status":"ok","service":"cyberos-chat"})))
}
