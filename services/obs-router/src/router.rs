//! HTTP router and alert routing orchestration.

use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, Request, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Serialize;
use thiserror::Error;
use tower_http::trace::TraceLayer;
use uuid::Uuid;

use crate::ack_handler::{handle_ack, handle_escalate, AckRequest, EscalateRequest};
use crate::alertmanager_webhook::{Alert, AlertmanagerWebhook};
use crate::chat_post::{build_chat_message, ChatClient};
use crate::cuo_triage::{triage_with_timeout, TriageClient, TriageResult};
use crate::dedup::{AlertRecord, DedupOutcome, DedupStore};
use crate::memory::{AuditRow, AuditSink};
use crate::metrics::ObsRouterMetrics;
use crate::pagerduty::{build_incident, PagerDutyClient};
use crate::severity::{decide_route, Route, Severity};

/// Router configuration.
#[derive(Debug, Clone)]
pub struct RouterConfig {
    /// Shared Alertmanager webhook secret.
    pub webhook_secret: String,
    /// CHAT channel.
    pub oncall_channel: String,
    /// Tempo/Grafana trace base URL.
    pub tempo_base_url: String,
    /// Deduplication window.
    pub dedup_window: Duration,
    /// CUO triage timeout.
    pub cuo_timeout: Duration,
}

impl RouterConfig {
    /// Local defaults.
    pub fn new(webhook_secret: impl Into<String>) -> Self {
        Self {
            webhook_secret: webhook_secret.into(),
            oncall_channel: "#oncall".to_string(),
            tempo_base_url: "https://grafana.cyberos.world".to_string(),
            dedup_window: Duration::from_secs(300),
            cuo_timeout: Duration::from_secs(5),
        }
    }
}

/// Shared application state.
#[derive(Debug, Clone)]
pub struct RouterState {
    /// Config.
    pub config: Arc<RouterConfig>,
    /// CUO triage client.
    pub triage: Arc<dyn TriageClient>,
    /// CHAT client.
    pub chat: Arc<dyn ChatClient>,
    /// PagerDuty client.
    pub pagerduty: Arc<dyn PagerDutyClient>,
    /// Audit sink.
    pub audit: Arc<dyn AuditSink>,
    /// Metrics store.
    pub metrics: Arc<ObsRouterMetrics>,
    /// Dedup store.
    pub dedup: Arc<DedupStore>,
}

impl RouterState {
    /// Construct state.
    pub fn new(
        config: RouterConfig,
        triage: Arc<dyn TriageClient>,
        chat: Arc<dyn ChatClient>,
        pagerduty: Arc<dyn PagerDutyClient>,
        audit: Arc<dyn AuditSink>,
    ) -> Self {
        Self {
            config: Arc::new(config),
            triage,
            chat,
            pagerduty,
            audit,
            metrics: Arc::new(ObsRouterMetrics::default()),
            dedup: Arc::new(DedupStore::default()),
        }
    }

    /// Verify the Alertmanager shared secret.
    pub fn authenticate(&self, headers: &HeaderMap) -> bool {
        if headers
            .get("X-CyberOS-Webhook-Secret")
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value == self.config.webhook_secret)
        {
            return true;
        }
        headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| {
                value == self.config.webhook_secret
                    || value == format!("X-CyberOS-Webhook-Secret {}", self.config.webhook_secret)
                    || value == format!("Bearer {}", self.config.webhook_secret)
            })
    }

    /// Process one webhook and route all contained alerts.
    pub async fn process_webhook(
        &self,
        payload: AlertmanagerWebhook,
    ) -> Result<Vec<RoutingReport>, RouterError> {
        let mut reports = Vec::new();
        for alert in payload.alerts {
            reports.push(self.route_one(alert).await?);
        }
        Ok(reports)
    }

    async fn route_one(&self, alert: Alert) -> Result<RoutingReport, RouterError> {
        let start = Instant::now();
        let alert_id = alert.alert_id();
        let alert_name = alert.alert_name();
        let severity = alert.severity();
        self.metrics.inc_received(severity);

        if let DedupOutcome::Duplicate { count, record } =
            self.dedup.check(&alert_id, self.config.dedup_window)
        {
            self.metrics.inc_dedup();
            if let Some(message_id) = record.and_then(|r| r.chat_message_id) {
                self.chat
                    .update_dedup_counter(&message_id, count)
                    .await
                    .map_err(|err| RouterError::Chat(err.to_string()))?;
            }
            return Ok(RoutingReport {
                alert_id,
                alert_name,
                severity,
                decision_route: None,
                actual_route: None,
                confidence: None,
                trace_id: alert.trace_id().map(ToOwned::to_owned),
                chat_message_id: None,
                pagerduty_dedup_key: None,
                cuo_fallback: false,
                deduped: true,
                audit_emitted: false,
                outcome: "deduped".to_string(),
            });
        }

        let (triage, cuo_fallback, cuo_failed) = match triage_with_timeout(
            self.triage.as_ref(),
            &alert,
            self.config.cuo_timeout,
        )
        .await
        {
            Ok(result) => (result, false, false),
            Err(crate::cuo_triage::CuoError::Timeout) => {
                self.metrics.inc_cuo_timeout();
                (TriageResult::fallback(), true, true)
            }
            Err(_) => (TriageResult::fallback(), true, true),
        };
        self.metrics.observe_confidence(triage.confidence);

        let decision_route = decide_route(severity, triage.confidence);
        let delivery = self
            .deliver(&alert, &triage, severity, decision_route)
            .await;
        let (actual_route, chat_message_id, pagerduty_dedup_key, outcome) = match delivery {
            Ok(result) => result,
            Err(err) => {
                let _ = self.chat.post_emergency(&alert, &err.to_string()).await;
                return Err(err);
            }
        };
        let outcome = if cuo_failed { "cuo_failed" } else { outcome };
        self.metrics.inc_routed(actual_route, severity, outcome);

        let request_id = format!("obs_router_{}", Uuid::new_v4());
        let audit_emitted = self
            .audit
            .emit(alert_triaged_row(
                &alert,
                severity,
                triage.confidence,
                actual_route,
                &triage,
                &request_id,
            ))
            .await
            .is_ok();

        self.dedup.mark_routed(AlertRecord {
            alert_id: alert_id.clone(),
            alert_name: alert_name.clone(),
            severity,
            trace_id: alert.trace_id().map(ToOwned::to_owned),
            route: actual_route,
            chat_message_id: chat_message_id.clone(),
            pagerduty_dedup_key: pagerduty_dedup_key.clone(),
        });
        self.metrics.observe_latency_ms(start.elapsed().as_millis());

        Ok(RoutingReport {
            alert_id,
            alert_name,
            severity,
            decision_route: Some(decision_route),
            actual_route: Some(actual_route),
            confidence: Some(triage.confidence),
            trace_id: alert.trace_id().map(ToOwned::to_owned),
            chat_message_id,
            pagerduty_dedup_key,
            cuo_fallback,
            deduped: false,
            audit_emitted,
            outcome: outcome.to_string(),
        })
    }

    async fn deliver(
        &self,
        alert: &Alert,
        triage: &TriageResult,
        severity: Severity,
        route: Route,
    ) -> Result<(Route, Option<String>, Option<String>, &'static str), RouterError> {
        match route {
            Route::Chat => {
                let message = build_chat_message(
                    alert,
                    triage,
                    severity,
                    &self.config.oncall_channel,
                    &self.config.tempo_base_url,
                );
                match self.chat.post(message).await {
                    Ok(receipt) => Ok((Route::Chat, Some(receipt.message_id), None, "ok")),
                    Err(_) => {
                        let incident = build_incident(alert, triage, severity);
                        let receipt = self
                            .pagerduty
                            .trigger(incident)
                            .await
                            .map_err(|err| RouterError::PagerDuty(err.to_string()))?;
                        Ok((
                            Route::PagerDuty,
                            None,
                            Some(receipt.dedup_key),
                            "chat_failed",
                        ))
                    }
                }
            }
            Route::PagerDuty => {
                let incident = build_incident(alert, triage, severity);
                match self.pagerduty.trigger(incident).await {
                    Ok(receipt) => Ok((Route::PagerDuty, None, Some(receipt.dedup_key), "ok")),
                    Err(err) => {
                        let receipt = self
                            .chat
                            .post_emergency(alert, &err.to_string())
                            .await
                            .map_err(|chat_err| RouterError::Chat(chat_err.to_string()))?;
                        Ok((
                            Route::Chat,
                            Some(receipt.message_id),
                            None,
                            "pagerduty_failed",
                        ))
                    }
                }
            }
            Route::Both => {
                let message = build_chat_message(
                    alert,
                    triage,
                    severity,
                    &self.config.oncall_channel,
                    &self.config.tempo_base_url,
                );
                let incident = build_incident(alert, triage, severity);
                let chat = self.chat.post(message).await;
                let pd = self.pagerduty.trigger(incident).await;
                match (chat, pd) {
                    (Ok(chat), Ok(pd)) => {
                        Ok((Route::Both, Some(chat.message_id), Some(pd.dedup_key), "ok"))
                    }
                    (Ok(chat), Err(_)) => {
                        Ok((Route::Chat, Some(chat.message_id), None, "pagerduty_failed"))
                    }
                    (Err(_), Ok(pd)) => {
                        Ok((Route::PagerDuty, None, Some(pd.dedup_key), "chat_failed"))
                    }
                    (Err(chat_err), Err(pd_err)) => {
                        let receipt = self
                            .chat
                            .post_emergency(
                                alert,
                                &format!("chat_failed={chat_err}; pagerduty_failed={pd_err}"),
                            )
                            .await
                            .map_err(|err| RouterError::Chat(err.to_string()))?;
                        Ok((
                            Route::Chat,
                            Some(receipt.message_id),
                            None,
                            "pagerduty_failed",
                        ))
                    }
                }
            }
        }
    }
}

/// Routing result.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RoutingReport {
    /// Alert id.
    pub alert_id: String,
    /// Alert name.
    pub alert_name: String,
    /// Parsed severity.
    pub severity: Severity,
    /// Initial route decision.
    pub decision_route: Option<Route>,
    /// Actual route after fallbacks.
    pub actual_route: Option<Route>,
    /// CUO confidence.
    pub confidence: Option<f64>,
    /// Trace id.
    pub trace_id: Option<String>,
    /// CHAT message id.
    pub chat_message_id: Option<String>,
    /// PagerDuty dedup key.
    pub pagerduty_dedup_key: Option<String>,
    /// True when CUO failed/timed out and confidence=0 fallback was used.
    pub cuo_fallback: bool,
    /// True when the alert was deduplicated.
    pub deduped: bool,
    /// True when audit sink accepted the row.
    pub audit_emitted: bool,
    /// Outcome label.
    pub outcome: String,
}

/// Router errors.
#[derive(Debug, Error)]
pub enum RouterError {
    /// CHAT failure.
    #[error("chat: {0}")]
    Chat(String),
    /// PagerDuty failure.
    #[error("pagerduty: {0}")]
    PagerDuty(String),
    /// Audit failure.
    #[error("audit: {0}")]
    Audit(String),
    /// Unknown alert id.
    #[error("not_found: {0}")]
    NotFound(String),
}

impl IntoResponse for RouterError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::BAD_GATEWAY,
        };
        (status, self.to_string()).into_response()
    }
}

/// Build the Axum app.
pub fn app(state: RouterState) -> Router {
    Router::new()
        .route("/ready", get(|| async { "ready\n" }))
        .route("/metrics", get(metrics_handler))
        .route("/alert", post(alert_handler))
        .route("/ack/:alert_id", post(ack_handler))
        .route("/escalate/:alert_id", post(escalate_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(Arc::new(state))
}

async fn metrics_handler(State(state): State<Arc<RouterState>>) -> String {
    state.metrics.render_prometheus()
}

async fn alert_handler(
    State(state): State<Arc<RouterState>>,
    headers: HeaderMap,
    Json(payload): Json<AlertmanagerWebhook>,
) -> Result<Json<Vec<RoutingReport>>, Response> {
    if !state.authenticate(&headers) {
        return Err(StatusCode::UNAUTHORIZED.into_response());
    }
    state
        .process_webhook(payload)
        .await
        .map(Json)
        .map_err(IntoResponse::into_response)
}

async fn ack_handler(
    State(state): State<Arc<RouterState>>,
    Path(alert_id): Path<String>,
    Json(payload): Json<AckRequest>,
) -> Result<StatusCode, RouterError> {
    handle_ack(&state, &alert_id, &payload.user).await?;
    Ok(StatusCode::OK)
}

async fn escalate_handler(
    State(state): State<Arc<RouterState>>,
    Path(alert_id): Path<String>,
    Json(payload): Json<EscalateRequest>,
) -> Result<StatusCode, RouterError> {
    handle_escalate(&state, &alert_id, &payload.user).await?;
    Ok(StatusCode::OK)
}

fn alert_triaged_row(
    alert: &Alert,
    severity: Severity,
    confidence: f64,
    route: Route,
    triage: &TriageResult,
    request_id: &str,
) -> AuditRow {
    AuditRow {
        kind: "obs.alert_triaged".to_string(),
        payload: serde_json::json!({
            "alert_name": alert.alert_name(),
            "severity": severity.as_label(),
            "cuo_confidence": confidence,
            "route": route.as_label(),
            "suggested_runbook": triage.suggested_runbook.as_ref().map(|r| r.url.clone()),
            "trace_id": alert.trace_id(),
            "request_id": request_id,
        }),
    }
}

/// Helper for tests.
pub fn json_request(uri: &str, secret: Option<&str>, body: serde_json::Value) -> Request<Body> {
    let mut builder = Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json");
    if let Some(secret) = secret {
        builder = builder.header("X-CyberOS-Webhook-Secret", secret);
    }
    builder.body(Body::from(body.to_string())).unwrap()
}
