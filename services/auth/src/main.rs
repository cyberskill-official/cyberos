//! cyberos-auth — main binary.

use cyberos_auth::{handlers, rbac, rls, AppState, VERSION};
use cyberos_cli_exit::ExitCode;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Notify;
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

    // TASK-AUTH-003 §1 #9 — boot-time RLS invariant check. Refuses to accept
    // traffic if any registered tenant-scoped table is missing RLS or has
    // zero policies. Catches the "registry says it's covered but the
    // migration was never written" failure mode before requests can leak.
    if let Err(diagnostic) = rls::verify_rls_at_boot(&state.pg).await {
        tracing::error!(error = %diagnostic, "RLS boot-check failed — refusing to start");
        return ExitCode::ConfigError;
    }

    // TASK-AUTH-101 §1 #9 — spawn the 60s RoleMatrix refresher.
    let shutdown = Arc::new(Notify::new());
    let refresher = rbac::refresher::spawn(
        state.pg.clone(),
        state.role_matrix.clone(),
        shutdown.clone(),
    );

    // TASK-OBS-003 - build the RED instruments off the global meter before serving.
    cyberos_obs_sdk::init("auth", cyberos_auth::VERSION);

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
            shutdown.notify_waiters();
            return ExitCode::NetworkError;
        }
    };

    let serve = axum::serve(listener, app).with_graceful_shutdown({
        let shutdown = shutdown.clone();
        async move {
            let _ = tokio::signal::ctrl_c().await;
            info!("ctrl-c received — shutting down");
            shutdown.notify_waiters();
        }
    });

    let result = serve.await;
    let _ = tokio::time::timeout(std::time::Duration::from_secs(5), refresher).await;

    match result {
        Ok(()) => ExitCode::Ok,
        Err(e) => {
            tracing::error!(error = %e, "axum serve failed");
            ExitCode::Generic
        }
    }
}
