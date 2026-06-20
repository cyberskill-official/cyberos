//! obs-router binary - the axum HTTP shell (FR-OBS-007). `POST /alert` receives Alertmanager webhooks,
//! authenticates the shared secret (§1 #13), dedups by fingerprint (§1 #12), and routes each firing
//! alert through CUO triage to CHAT or PagerDuty (`route_alert`). The clients are env-configured and
//! degrade safely when a target is unset. The audit sink writes obs.alert_triaged / _acked to the memory
//! chain when `DATABASE_URL` is set (best-effort, off the request path); unset, it logs. Live validation
//! needs the CUO endpoint + CHAT webhook + PagerDuty routing key.

use std::sync::Arc;
use std::time::Instant;

use sqlx::PgPool;
use uuid::Uuid;

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};

use cyberos_obs_router::alertmanager_webhook::{AlertStatus, Webhook};
use cyberos_obs_router::audit::{self, AuditRow, AuditSink};
use cyberos_obs_router::chat_post::HttpChatClient;
use cyberos_obs_router::config::Config;
use cyberos_obs_router::cuo_triage::HttpTriageClient;
use cyberos_obs_router::dedup::{DedupOutcome, Deduper};
use cyberos_obs_router::pagerduty::HttpPagerDutyClient;
use cyberos_obs_router::route_alert;

struct AppState {
    config: Config,
    triage: HttpTriageClient,
    chat: HttpChatClient,
    pagerduty: HttpPagerDutyClient,
    deduper: Deduper,
    sink: Box<dyn AuditSink>,
    started: Instant,
}

/// Logs each audit row - the fallback sink when no `DATABASE_URL` is configured.
struct LogSink;
impl AuditSink for LogSink {
    fn emit(&self, row: &AuditRow) {
        eprintln!("audit {} {}", row.kind, row.payload);
    }
}

/// Appends each audit row to the memory chain as a genesis row (FR-OBS-007 §1 #6). The write runs off
/// the request path via `tokio::spawn` and is best-effort: a failure logs and is dropped, because the
/// alert route already completed (§10, "audit emit fails -> route still completes"). Infra alerts have no
/// tenant or subject, so the row is scoped to the root tenant (nil UUID), like auth's tenant-unknown path.
struct PgAuditSink {
    pool: PgPool,
}

impl AuditSink for PgAuditSink {
    fn emit(&self, row: &AuditRow) {
        let pool = self.pool.clone();
        let kind = row.kind;
        let mut body = row.payload.clone();
        if let serde_json::Value::Object(ref mut m) = body {
            m.insert(
                "event_type".to_string(),
                serde_json::Value::String(kind.to_string()),
            );
        }
        // Path key: the fingerprint (acked) or the trace_id (triaged), else the bare event.
        let key = body
            .get("alert_fingerprint")
            .or_else(|| body.get("trace_id"))
            .and_then(|v| v.as_str())
            .unwrap_or("event")
            .to_string();
        let suffix = kind.rsplit('.').next().unwrap_or(kind).to_string();
        let path = format!("obs/alert/{key}/{suffix}");
        tokio::spawn(async move {
            let body_str = serde_json::to_string(&body).unwrap_or_else(|_| "{}".to_string());
            if let Err(e) =
                cyberos_audit_chain::emit_genesis(&pool, Uuid::nil(), Uuid::nil(), &path, &body_str)
                    .await
            {
                eprintln!("audit emit failed (best-effort) {kind}: {e}");
            }
        });
    }
}

/// Build the audit sink: the memory-chain writer when `DATABASE_URL` is set and reachable, else the log
/// sink. A set-but-unreachable `DATABASE_URL` falls back to logging rather than refusing to start - the
/// router's job (routing alerts) must not depend on the audit DB being up.
async fn build_sink() -> Box<dyn AuditSink> {
    match std::env::var("DATABASE_URL") {
        Ok(url) if !url.trim().is_empty() => {
            match sqlx::postgres::PgPoolOptions::new()
                .max_connections(2)
                .connect(url.trim())
                .await
            {
                Ok(pool) => {
                    eprintln!("obs-router: audit sink -> memory chain (l1_audit_log)");
                    Box::new(PgAuditSink { pool })
                }
                Err(e) => {
                    eprintln!(
                        "obs-router: DATABASE_URL set but connect failed ({e}); audit sink -> log"
                    );
                    Box::new(LogSink)
                }
            }
        }
        _ => Box::new(LogSink),
    }
}

#[tokio::main]
async fn main() {
    let config = Config::from_env();
    let self_base = std::env::var("OBS_ROUTER_PUBLIC_URL")
        .unwrap_or_else(|_| format!("http://{}", config.bind));
    let sink = build_sink().await;
    let state = Arc::new(AppState {
        triage: HttpTriageClient::new(config.cuo_triage_url.clone()),
        chat: HttpChatClient::new(config.chat_webhook_url.clone(), self_base),
        pagerduty: HttpPagerDutyClient::new(
            config.pagerduty_routing_key.clone(),
            config.pagerduty_endpoint.clone(),
        ),
        deduper: Deduper::new(),
        sink,
        started: Instant::now(),
        config,
    });

    let bind = state.config.bind.clone();
    let app = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/alert", post(handle_alert))
        .route("/ack/:fingerprint", post(handle_ack))
        .route("/escalate/:fingerprint", post(handle_escalate))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&bind)
        .await
        .unwrap_or_else(|e| panic!("obs-router: cannot bind {bind}: {e}"));
    eprintln!("obs-router listening on {bind}");
    axum::serve(listener, app).await.expect("obs-router: serve failed");
}

async fn handle_alert(State(st): State<Arc<AppState>>, headers: HeaderMap, body: String) -> Response {
    if let Some(secret) = st.config.webhook_secret.as_ref() {
        let got = headers
            .get("x-cyberos-webhook-secret")
            .and_then(|h| h.to_str().ok());
        if got != Some(secret.as_str()) {
            return (StatusCode::UNAUTHORIZED, "unauthenticated").into_response();
        }
    }
    let webhook = match Webhook::parse(&body) {
        Ok(w) => w,
        Err(e) => return (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    };

    let now_ms = u64::try_from(st.started.elapsed().as_millis()).unwrap_or(u64::MAX);
    let mut routed = 0u32;
    let mut deduped = 0u32;
    for alert in webhook.alerts() {
        if alert.status == AlertStatus::Resolved {
            continue; // slice 4 routes firing alerts; resolve-close is a follow-up.
        }
        match st.deduper.observe(&alert.fingerprint, now_ms) {
            // single CHAT post per 5m window; PagerDuty dedups re-fires via dedup_key (§1 #12).
            DedupOutcome::Repeat { .. } => deduped += 1,
            DedupOutcome::FirstInWindow => {
                let request_id = format!("alert_{}", now_ms.wrapping_add(u64::from(routed)));
                route_alert(
                    &st.triage,
                    &st.chat,
                    &st.pagerduty,
                    st.sink.as_ref(),
                    &alert,
                    &request_id,
                )
                .await;
                routed += 1;
            }
        }
    }
    Json(serde_json::json!({ "routed": routed, "deduped": deduped })).into_response()
}

async fn handle_ack(State(st): State<Arc<AppState>>, Path(fingerprint): Path<String>) -> Response {
    // Minimal: record the ack. Updating the CHAT post + closing the PagerDuty incident needs the stored
    // alert/post state (a follow-up, §1 #10).
    st.sink
        .emit(&audit::alert_acked(&fingerprint, "chat-user", "ack"));
    (StatusCode::OK, "acked").into_response()
}

async fn handle_escalate(State(_st): State<Arc<AppState>>, Path(_fingerprint): Path<String>) -> Response {
    // Minimal stub: a full escalate re-pages PagerDuty for the stored alert (follow-up needs alert state).
    (StatusCode::OK, "escalation noted").into_response()
}
