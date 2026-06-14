//! HTTP router for compliance views.

use std::sync::Arc;
use std::time::Instant;

use axum::extract::{Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use tower_http::trace::TraceLayer;

use crate::auth::{verify_authorization, AuthConfig};
use crate::chain_proof::ChainProofSigner;
use crate::error::ViewError;
use crate::export::{json, pdf};
use crate::memory::MemoryBackend;
use crate::metrics::Metrics;
use crate::views::{self, Format, ViewDefinition};

/// Query params for every view.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ViewQuery {
    /// Start time.
    pub since: DateTime<Utc>,
    /// End time.
    pub until: DateTime<Utc>,
    /// Export format.
    #[serde(default)]
    pub format: Format,
    /// Rejected tenant override.
    #[serde(default)]
    pub tenant_id: Option<String>,
}

/// Shared app state.
#[derive(Debug, Clone)]
pub struct AppState {
    /// Auth config.
    pub auth: AuthConfig,
    /// Memory backend.
    pub memory: Arc<dyn MemoryBackend>,
    /// Chain proof signer.
    pub signer: ChainProofSigner,
    /// Metrics.
    pub metrics: Arc<Metrics>,
}

impl AppState {
    /// Construct state.
    pub fn new(auth: AuthConfig, memory: Arc<dyn MemoryBackend>, signer: ChainProofSigner) -> Self {
        Self {
            auth,
            memory,
            signer,
            metrics: Arc::new(Metrics::default()),
        }
    }
}

/// Build Axum app.
pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/ready", get(|| async { "ready\n" }))
        .route("/metrics", get(metrics_handler))
        .route("/eu-ai-act/", get(eu_ai_act_handler))
        .route("/pdpl/", get(pdpl_handler))
        .route("/soc2/", get(soc2_handler))
        .route("/iso27001/", get(iso27001_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(Arc::new(state))
}

async fn metrics_handler(State(state): State<Arc<AppState>>) -> String {
    state.metrics.render_prometheus()
}

async fn eu_ai_act_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<ViewQuery>,
) -> Result<Response, ViewError> {
    handle(state, headers, query, views::eu_ai_act::definition()).await
}

async fn pdpl_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<ViewQuery>,
) -> Result<Response, ViewError> {
    handle(state, headers, query, views::pdpl::definition()).await
}

async fn soc2_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<ViewQuery>,
) -> Result<Response, ViewError> {
    handle(state, headers, query, views::soc2::definition()).await
}

async fn iso27001_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<ViewQuery>,
) -> Result<Response, ViewError> {
    handle(state, headers, query, views::iso27001::definition()).await
}

async fn handle(
    state: Arc<AppState>,
    headers: HeaderMap,
    query: ViewQuery,
    definition: ViewDefinition,
) -> Result<Response, ViewError> {
    let start = Instant::now();
    let format = query.format;
    let claims = verify_authorization(
        headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok()),
        &state.auth,
    )?;
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        views::build_view(
            state.memory.as_ref(),
            &state.signer,
            definition,
            query,
            claims,
        ),
    )
    .await
    .map_err(|_| ViewError::QueryTimeout)?;
    let response = match result {
        Ok(response) => response,
        Err(ViewError::PiiLeakAttempt) => {
            state.metrics.inc_pii_leak();
            state
                .metrics
                .inc_request(definition.id, format.as_label(), "pii_leak");
            return Err(ViewError::PiiLeakAttempt);
        }
        Err(err) => {
            state
                .metrics
                .inc_request(definition.id, format.as_label(), err.code());
            return Err(err);
        }
    };
    state
        .metrics
        .observe_latency(definition.id, start.elapsed().as_millis());
    state
        .metrics
        .observe_rows(definition.id, response.rows.len());
    state
        .metrics
        .inc_request(definition.id, format.as_label(), "ok");

    match format {
        Format::Json => {
            let body = json::render_json(&response)?;
            Ok(([(header::CONTENT_TYPE, "application/json")], body).into_response())
        }
        Format::Pdf => {
            let body = pdf::render_pdf(&response)?;
            Ok(([(header::CONTENT_TYPE, "application/pdf")], body).into_response())
        }
    }
}

/// Helper for tests.
pub fn auth_header(token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        format!("Bearer {token}").parse().unwrap(),
    );
    headers
}

/// Empty OK response.
pub fn ok() -> StatusCode {
    StatusCode::OK
}
