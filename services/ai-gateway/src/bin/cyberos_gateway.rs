//! `cyberos-gateway` - the AI Gateway HTTP server. This is the listener the pipeline modules (policy,
//! alias, router, cost ledger) plug into; it serves `POST /v1/chat` plus `/healthz` and `/metrics`.
//!
//! Boot order (TASK-AI-007 / TASK-AI-015): initialise the per-tenant policy loader from the config dir before
//! binding the listener, so no request is served before policy is available. The provider call uses the
//! in-repo echo backend until the TASK-AI-008 provider adapters land.

use std::path::PathBuf;

use cyberos_ai_gateway::policy::init_loader;
use cyberos_ai_gateway::server::{build_router, GatewayState};

#[tokio::main]
async fn main() {
    // TASK-OBS-005 §1 #2 - JSON logs that render the request span scope, so every line emitted while
    // handling a request carries trace_id / span_id / tenant_id for cross-tool correlation.
    cyberos_obs_sdk::init_json_subscriber();

    let config_dir = std::env::var("AI_GATEWAY_CONFIG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("config/tenants"));

    // Keep the loader alive for the server's lifetime - it owns the hot-reload watcher.
    let _loader = match init_loader(&config_dir).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!(
                "fatal: policy loader init failed for {}: {e:?}",
                config_dir.display()
            );
            std::process::exit(1);
        }
    };

    // TASK-OBS-003 - build the RED instruments and install the OTLP meter provider (when
    // OBS_OTLP_ENDPOINT is set) before serving.
    cyberos_obs_sdk::init("ai-gateway", env!("CARGO_PKG_VERSION"));

    let app = build_router(GatewayState::production());
    let bind = std::env::var("AI_GATEWAY_BIND").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let listener = match tokio::net::TcpListener::bind(&bind).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("fatal: cannot bind {bind}: {e}");
            std::process::exit(1);
        }
    };
    eprintln!(
        "cyberos-ai-gateway listening on {bind} (config: {})",
        config_dir.display()
    );
    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("fatal: server error: {e}");
        std::process::exit(1);
    }
}
