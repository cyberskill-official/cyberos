//! CyberOS-native chat (FR-CHAT-101 slice 1): router, shared state, health, and a small error helper.
//! A first-party Rust service on the CyberOS identity (FR-AUTH-110), Postgres with per-tenant RLS, and
//! the memory audit chain - replacing the Mattermost dependency.

pub mod ai;
pub mod attachments;
pub mod audit;
pub mod auditlog;
pub mod auth;
pub mod blocks;
pub mod capture;
pub mod channels;
pub mod db;
pub mod devices;
pub mod members;
pub mod messages;
pub mod moderation;
pub mod notify;
pub mod prefs;
pub mod push;
pub mod reactions;
pub mod read;
pub mod realtime;
pub mod reports;
pub mod storage;
pub mod translate;

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};

#[derive(Clone)]
pub struct AppState {
    pub pool: db::Pool,
    /// The memory module's Postgres (holds l1_audit_log). When set, chat appends a hash-chained audit
    /// row per channel/message event; when unset (tests/local), the event is logged. DEC-2713 makes this
    /// the chat->brain link.
    pub audit_pool: Option<db::Pool>,
    /// FR-MEMORY-122 §1 #4 — the BRAIN capture mechanism over `audit_pool`. `Some` ONLY when
    /// `CAPTURE_ENABLED` is truthy AND `audit_pool` is configured (default off). When `None`, every chat
    /// emitter (`capture::emit_*`) is a complete no-op, so capture costs nothing and message send / channel
    /// activity are unchanged — safe to deploy during a live team load-test.
    pub capturer: Option<cyberos_capture::Capturer>,
    pub authenticator: Arc<auth::Authenticator>,
    pub hub: realtime::Hub,
    pub presence: realtime::Presence,
    /// Per-user notification fan-out: one broadcast per subject carrying cross-channel `NotifyEvent`s, so a
    /// client learns about activity in channels it is not currently viewing (unread badges, tab count,
    /// desktop notifications). See notify.rs.
    pub notifier: notify::Notifier,
    /// Attachment byte-store backend + limits (richer-messages cluster). See storage.rs.
    pub attachments: storage::AttachmentConfig,
    /// FR-CHAT-268 — memoised blocker -> blocked-set, invalidated on every block/unblock. Read by all
    /// four enforcement points; the realtime one needs it per-frame, so it must not hit the DB.
    pub blocks: blocks::BlockCache,
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
        .route("/v1/chat/dms", post(channels::create_dm))
        // Static segments win over :id in axum, so /channels/browse never collides with /channels/:id.
        .route("/v1/chat/channels/browse", get(channels::browse))
        .route(
            "/v1/chat/channels/:id",
            axum::routing::patch(channels::update),
        )
        .route("/v1/chat/channels/:id/join", post(channels::join))
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
            axum::routing::delete(members::remove).patch(members::set_role),
        )
        .route("/v1/chat/channels/:id/search", get(messages::search))
        .route("/v1/chat/search", get(messages::search_all))
        .route(
            "/v1/chat/channels/:id/messages/:msg/reactions",
            post(reactions::add),
        )
        .route(
            "/v1/chat/channels/:id/messages/:msg/reactions/:emoji",
            axum::routing::delete(reactions::remove),
        )
        .route("/v1/chat/translate", post(translate::translate))
        // FR-CHAT-267. Deliberately tenant-wide, not nested under a channel: a report can target a person
        // with whom the reporter shares no channel (§1 #3 - the DM harassment path).
        .route("/v1/chat/reports", post(reports::create))
        // FR-CHAT-268. Tenant-wide like reports: you can block someone you share no channel with.
        .route("/v1/chat/blocks", post(blocks::block).get(blocks::list))
        .route(
            "/v1/chat/blocks/:subject_id",
            axum::routing::delete(blocks::unblock),
        )
        // FR-CHAT-269 — the moderation queue. Every one of these is gated on a WORKSPACE role
        // (auth::require_moderator, fail-closed); a channel role grants nothing here.
        .route("/v1/chat/admin/reports", get(moderation::queue))
        .route("/v1/chat/admin/reports/:id", get(moderation::detail))
        .route(
            "/v1/chat/admin/reports/:id/resolve",
            post(moderation::resolve),
        )
        .route("/v1/chat/channels/:id/ai/summarize", post(ai::summarize))
        .route("/v1/chat/channels/:id/ai/actions", post(ai::actions))
        .route("/v1/chat/channels/:id/ai/replies", post(ai::replies))
        .route(
            "/v1/chat/channels/:id/attachments",
            post(attachments::upload),
        )
        .route(
            "/v1/chat/channels/:id/uploads",
            post(attachments::upload_raw),
        )
        .route("/v1/chat/attachments/:att", get(attachments::download))
        .route("/v1/chat/attachments/:att/meta", get(attachments::meta))
        .route("/v1/chat/config", get(client_config))
        .route(
            "/v1/chat/channels/:id/presence",
            get(realtime::presence_list),
        )
        .route("/v1/chat/channels/:id/read", post(read::mark))
        .route("/v1/chat/channels/:id/unread", get(read::unread))
        .route("/v1/chat/unread", get(read::unread_summary))
        .route("/v1/chat/channels/:id/receipts", get(read::receipts))
        .route("/v1/chat/prefs", get(prefs::list))
        .route(
            "/v1/chat/channels/:id/prefs",
            axum::routing::put(prefs::set),
        )
        .route("/v1/chat/devices", post(devices::register))
        .route("/v1/chat/audit", get(auditlog::list))
        .route("/v1/chat/ws", get(realtime::ws_handler))
        .route("/v1/chat/notify", get(notify::notify_ws))
        // Axum's default body limit is 2 MB, which silently capped the base64 upload route below the
        // advertised attachment limit. Size the limit from the configured cap: raw uploads need max_bytes,
        // the base64 JSON route needs ~4/3 of it, plus slack for headers/JSON framing.
        .layer(axum::extract::DefaultBodyLimit::max(
            state.attachments.max_bytes + state.attachments.max_bytes / 2 + 1024 * 1024,
        ))
        .with_state(state);

    // FR-APP-007: opt-in permissive CORS so a local browser (the CDS chat web client) can call the
    // service cross-origin in dev. In production Caddy serves the page and proxies the service under
    // one origin, so this stays off. Set CHAT_DEV_CORS=1 only for local development.
    if std::env::var("CHAT_DEV_CORS").is_ok() {
        app = app.layer(tower_http::cors::CorsLayer::permissive());
    }
    app
}

/// GET /v1/chat/config - the limits a client needs to mirror server-side validation (attachment cap and
/// per-message file count), authenticated like every other route.
async fn client_config(
    State(st): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let _ = auth::authenticate(&st, &headers)?;
    Ok(Json(serde_json::json!({
        "attachment_max_bytes": st.attachments.max_bytes,
        "attachment_max_files": st.attachments.max_files,
    })))
}

async fn health(
    State(st): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    sqlx::query("SELECT 1")
        .execute(&st.pool)
        .await
        .map_err(internal)?;
    Ok(Json(
        serde_json::json!({"status":"ok","service":"cyberos-chat"}),
    ))
}
