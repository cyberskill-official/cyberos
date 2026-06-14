//! `cyberos-mcp` — MCP Gateway entrypoint.

use std::sync::Arc;

use clap::Parser;
use cyberos_mcp_gateway::{
    federation::registry::ToolRegistry,
    router::{build_router, AppState},
    SERVICE_BANNER,
};
use tracing_subscriber::prelude::*;

#[derive(Debug, Parser)]
#[command(name = "cyberos-mcp", version, about = "CyberOS MCP Gateway")]
struct Cli {
    /// HTTP listen address.
    #[arg(long, default_value = "0.0.0.0:8090")]
    listen: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(cyberos_obs_sdk::logging::ObsContextLayer::new(
            "mcp-gateway",
        ))
        .init();

    if let Err(e) = cyberos_obs_sdk::init("mcp-gateway", env!("CARGO_PKG_VERSION")) {
        tracing::warn!(error = %e, "obs sdk init failed");
    }

    println!("{SERVICE_BANNER}");
    let cli = Cli::parse();

    let registry = Arc::new(ToolRegistry::new());
    let state = AppState { registry };
    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind(&cli.listen)
        .await
        .expect("bind");
    println!(
        "MCP Gateway listening on http://{} — POST /mcp · GET /mcp/healthz",
        cli.listen
    );
    axum::serve(listener, app).await.expect("serve");
}
