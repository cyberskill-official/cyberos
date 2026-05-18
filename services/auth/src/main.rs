//! cyberos-auth — main binary.

use cyberos_auth::{handlers, AppState, VERSION};
use cyberos_cli_exit::ExitCode;
use std::net::SocketAddr;
use tracing::info;

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "cyberos_auth=info,info".into()),
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

    let app = handlers::router(state);

    let addr: SocketAddr = std::env::var("AUTH_LISTEN_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:7700".into())
        .parse()
        .expect("AUTH_LISTEN_ADDR must be a valid socket address");

    info!(%addr, version = VERSION, "cyberos-auth starting");

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
