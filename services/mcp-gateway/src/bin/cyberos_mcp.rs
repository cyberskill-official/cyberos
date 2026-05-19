//! `cyberos-mcp` — MCP Gateway entrypoint.

use std::sync::Arc;

use clap::Parser;
use cyberos_mcp_gateway::{
    federation::registry::ToolRegistry,
    router::{build_router, AppState},
    SERVICE_BANNER,
};

#[derive(Debug, Parser)]
#[command(name = "cyberos-mcp", version, about = "CyberOS MCP Gateway")]
struct Cli {
    /// HTTP listen address.
    #[arg(long, default_value = "0.0.0.0:8090")]
    listen: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

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
