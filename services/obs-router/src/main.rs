//! obs-router binary - the axum HTTP shell (FR-OBS-007). `POST /alert` receives Alertmanager webhooks,
//! authenticates the shared secret (§1 #13), dedups by fingerprint (§1 #12), and routes each firing
//! alert through CUO triage to CHAT or PagerDuty (`route_alert`). The clients are env-configured and
//! degrade safely when a target is unset. The audit sink logs for now; writing obs.alert_triaged /
//! _acked to the memory chain is a follow-up. Live validation needs the CUO endpoint + CHAT webhook +
//! PagerDuty routing key.

use std::sync::Arc;
use std::time::Instant;

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
    sink: LogSink,
    started: Instant,
}

/// Logs each audit row. Replace with the memory-chain writer (best-effort) in a follow-up.
struct LogSink;
impl AuditSink for LogSink {
    fn emit(&self, row: &AuditRow) {
        eprintln!("audit {} {}", row.kind, row.payload);
    }
}

#[tokio::main]
async fn main() {
    let config = Config::from_env();
    let self_base = std::env::var("OBS_ROUTER_PUBLIC_URL")
        .unwrap_or_else(|_| format!("http://{}", config.bind));
    let state = Arc::new(AppState {
        triage: HttpTriageClient::new(config.cuo_triage_url.clone()),
        chat: HttpChatClient::new(config.chat_webhook_url.clone(), self_base),
        pagerduty: HttpPagerDutyClient::new(
            config.pagerduty_routing_key.clone(),
            config.pagerduty_endpoint.clone(),
        ),
        deduper: Deduper::new(),
        sink: LogSink,
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
                    &st.sink,
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
