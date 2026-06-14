//! `cyberos-email` HTTP server binary.
//!
//! Slice 1 wires:
//!   - axum router with health + per-message status + list handlers.
//!   - sqlx pool initialised from DATABASE_URL.
//!   - Stalwart inbound webhook endpoint stub (real Stalwart wiring lands
//!     in FR-EMAIL-002).
//!
//! Run:
//!   DATABASE_URL=postgres://... cyberos-email

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
    Json,
};
use serde::Deserialize;
use sqlx::PgPool;
use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer().json())
        .with(cyberos_obs_sdk::logging::ObsContextLayer::new(
            "email-service",
        ))
        .init();

    if let Err(e) = cyberos_obs_sdk::init("email-service", env!("CARGO_PKG_VERSION")) {
        tracing::warn!(error = %e, "obs sdk init failed");
    }

    let db_url = std::env::var("DATABASE_URL").map_err(|_| {
        anyhow::anyhow!("DATABASE_URL not set — set it to a Postgres connection string")
    })?;
    let pool = sqlx::PgPool::connect(&db_url).await?;
    info!("connected to postgres");

    let bind: SocketAddr = std::env::var("EMAIL_BIND")
        .unwrap_or_else(|_| "0.0.0.0:8085".into())
        .parse()?;

    info!("cyberos-email listening on {bind}");

    let app = axum::Router::new()
        .route("/v1/email/healthz", get(healthz))
        .route(
            "/v1/admin/tenants/:tenant_id/email/dns-setup",
            post(dns_setup),
        )
        .route(
            "/v1/admin/tenants/:tenant_id/email/dns-verify",
            post(dns_verify),
        )
        .route(
            "/v1/admin/tenants/:tenant_id/email/bimi-enable",
            post(bimi_enable),
        )
        .route("/v1/email/camel/execute", post(camel_execute))
        .route("/v1/email/camel/audit-log", get(camel_audit_log))
        .route("/v1/email/camel/trust-list", put(camel_trust_list))
        .route("/v1/email/outbound", get(outbound_list))
        .route("/v1/email/outbound/compose", post(outbound_compose))
        .route("/v1/email/outbound/send", post(outbound_send))
        .route(
            "/v1/email/outbound/delivery-status",
            post(outbound_delivery_status),
        )
        .route(
            "/v1/admin/email/suppression/unsuppress",
            post(outbound_unsuppress),
        )
        .route("/v1/email/dsar/export", post(dsar_export))
        .route("/v1/email/dsar/jobs/:tenant_id/:job_id", get(dsar_job))
        .with_state(pool)
        .layer(cyberos_obs_sdk::red::RedLayer::new("email-service"));

    let listener = tokio::net::TcpListener::bind(bind).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn healthz(State(db): State<PgPool>) -> impl IntoResponse {
    match cyberos_email::handlers::healthz(&db).await {
        Ok(resp) => Json(resp).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn dns_setup(
    State(db): State<PgPool>,
    Path(tenant_id): Path<Uuid>,
    Json(req): Json<cyberos_email::handlers::delivery_auth::DnsSetupRequest>,
) -> impl IntoResponse {
    json_result(cyberos_email::handlers::delivery_auth::dns_setup(&db, tenant_id, req).await)
}

async fn dns_verify(
    State(db): State<PgPool>,
    Path(tenant_id): Path<Uuid>,
    Json(req): Json<cyberos_email::handlers::delivery_auth::DnsVerifyRequest>,
) -> impl IntoResponse {
    json_result(cyberos_email::handlers::delivery_auth::dns_verify(&db, tenant_id, req).await)
}

async fn bimi_enable(
    Json(req): Json<cyberos_email::handlers::delivery_auth::BimiEnableRequest>,
) -> impl IntoResponse {
    json_result(cyberos_email::handlers::delivery_auth::bimi_enable(req).await)
}

async fn camel_execute(
    State(db): State<PgPool>,
    Json(req): Json<cyberos_email::handlers::camel::CamelExecuteRequest>,
) -> impl IntoResponse {
    json_result(cyberos_email::handlers::camel::execute(&db, req).await)
}

#[derive(Debug, Deserialize)]
struct TenantLimitQuery {
    tenant_id: Uuid,
    limit: Option<i64>,
}

async fn camel_audit_log(
    State(db): State<PgPool>,
    Query(q): Query<TenantLimitQuery>,
) -> impl IntoResponse {
    json_result(cyberos_email::handlers::camel::audit_log(&db, q.tenant_id, q.limit).await)
}

async fn camel_trust_list(
    State(db): State<PgPool>,
    Json(req): Json<cyberos_email::handlers::camel::CamelTrustListRequest>,
) -> impl IntoResponse {
    json_result(cyberos_email::handlers::camel::trust_list_upsert(&db, req).await)
}

#[derive(Debug, Deserialize)]
struct OutboundListQuery {
    tenant_id: Uuid,
    status: Option<cyberos_email::outbound::SendStatus>,
    limit: Option<i64>,
}

async fn outbound_list(
    State(db): State<PgPool>,
    Query(q): Query<OutboundListQuery>,
) -> impl IntoResponse {
    json_pg_outbound(
        cyberos_email::handlers::outbound::list(&db, q.tenant_id, q.status, q.limit).await,
    )
}

async fn outbound_compose(
    State(db): State<PgPool>,
    Json(req): Json<cyberos_email::handlers::outbound::ComposeHttpRequest>,
) -> impl IntoResponse {
    json_pg_outbound(cyberos_email::handlers::outbound::compose(&db, req).await)
}

async fn outbound_send(
    State(db): State<PgPool>,
    Json(req): Json<cyberos_email::handlers::outbound::SendHttpRequest>,
) -> impl IntoResponse {
    json_pg_outbound(cyberos_email::handlers::outbound::send(&db, req).await)
}

async fn outbound_delivery_status(
    State(db): State<PgPool>,
    Json(req): Json<cyberos_email::handlers::outbound::DeliveryStatusRequest>,
) -> impl IntoResponse {
    json_pg_outbound(cyberos_email::handlers::outbound::delivery_status(&db, req).await)
}

async fn outbound_unsuppress(
    State(db): State<PgPool>,
    Json(req): Json<cyberos_email::handlers::outbound::UnsuppressRequest>,
) -> impl IntoResponse {
    match cyberos_email::handlers::outbound::unsuppress(&db, req).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => outbound_error(e),
    }
}

async fn dsar_export(
    State(db): State<PgPool>,
    Json(req): Json<cyberos_email::handlers::dsar::DsarExportRequest>,
) -> impl IntoResponse {
    json_result(cyberos_email::handlers::dsar::export(&db, req).await)
}

async fn dsar_job(
    State(db): State<PgPool>,
    Path((tenant_id, job_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    match cyberos_email::handlers::dsar::get_job(&db, tenant_id, job_id).await {
        Ok(Some(row)) => Json(row).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "dsar job not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

fn json_result<T: serde::Serialize>(
    result: cyberos_email::errors::EmailResult<T>,
) -> axum::response::Response {
    match result {
        Ok(value) => Json(value).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

fn json_pg_outbound<T: serde::Serialize>(
    result: Result<T, cyberos_email::outbound::PgOutboundError>,
) -> axum::response::Response {
    match result {
        Ok(value) => Json(value).into_response(),
        Err(e) => outbound_error(e),
    }
}

fn outbound_error(e: cyberos_email::outbound::PgOutboundError) -> axum::response::Response {
    let status = match &e {
        cyberos_email::outbound::PgOutboundError::NotFound => StatusCode::NOT_FOUND,
        cyberos_email::outbound::PgOutboundError::Policy(policy) => match policy {
            cyberos_email::outbound::OutboundError::RateLimitExceeded => {
                StatusCode::TOO_MANY_REQUESTS
            }
            cyberos_email::outbound::OutboundError::RecipientSuppressed(_) => StatusCode::CONFLICT,
            cyberos_email::outbound::OutboundError::ConfirmTokenInvalid
            | cyberos_email::outbound::OutboundError::ConfirmTokenExpired => StatusCode::FORBIDDEN,
        },
        cyberos_email::outbound::PgOutboundError::Sql(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (status, e.to_string()).into_response()
}
