//! cyberos-brain — main binary entry point.
//!
//! Wave 1 first-slice: boots an axum HTTP server with `/healthz` only.
//! Future FRs add `/v1/brain/search` (FR-BRAIN-108), Layer-2 ingest worker
//! (FR-BRAIN-101), and the cross-tenant RLS guard surface.

use axum::{extract::State, http::StatusCode, response::Json, routing::get, Router};
use cyberos_brain::{state::AppState, VERSION};
use cyberos_cli_exit::ExitCode;
use serde_json::{json, Value};
use std::net::SocketAddr;
use tracing::info;

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "cyberos_brain=info,info".into()),
        )
        .json()
        .init();

    let state = match AppState::connect_from_env().await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(error = %e, "failed to connect to Postgres");
            return ExitCode::ConfigError;
        }
    };

    let app = Router::new()
        .route("/healthz", get(healthz))
        .with_state(state);

    let addr: SocketAddr = std::env::var("BRAIN_LISTEN_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:7800".into())
        .parse()
        .expect("BRAIN_LISTEN_ADDR must be a valid socket address");

    info!(%addr, version = VERSION, "cyberos-brain starting");

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!(error = %e, %addr, "failed to bind");
            return ExitCode::NetworkError;
        }
    };

    if let Err(e) = axum::serve(listener, app).await {
        tracing::error!(error = %e, "axum serve failed");
        return ExitCode::Generic;
    }

    ExitCode::Ok
}

async fn healthz(State(state): State<AppState>) -> (StatusCode, Json<Value>) {
    // Cheapest possible Postgres roundtrip to confirm pool is alive.
    let pg_ok = sqlx::query("SELECT 1").fetch_one(&state.pg).await.is_ok();
    let status = if pg_ok { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE };
    (
        status,
        Json(json!({
            "service": "cyberos-brain",
            "version": VERSION,
            "postgres": if pg_ok { "ok" } else { "down" },
        })),
    )
}
