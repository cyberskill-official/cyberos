//! `cyberos-mcp` — MCP Gateway entrypoint.

use std::sync::Arc;

use clap::Parser;
use cyberos_mcp_gateway::{
    elicitation::ElicitationStore,
    federation::registry::ToolRegistry,
    router::{build_router, AppState},
    tasks::TaskStore,
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

    // TASK-MCP-004: connect Postgres for the OAuth endpoints when configured. Absent (dev), the OAuth
    // endpoints report unconfigured and the rest of the gateway runs unaffected.
    let oauth_pool = match std::env::var("MCP_DATABASE_URL") {
        Ok(url) if !url.trim().is_empty() => {
            match sqlx::postgres::PgPoolOptions::new()
                .max_connections(5)
                .connect(&url)
                .await
            {
                Ok(pool) => {
                    println!("OAuth: connected to Postgres - TASK-MCP-004 endpoints enabled");
                    Some(pool)
                }
                Err(e) => {
                    eprintln!(
                        "OAuth: MCP_DATABASE_URL set but connection failed ({e}); OAuth endpoints disabled"
                    );
                    None
                }
            }
        }
        _ => {
            println!("OAuth: MCP_DATABASE_URL not set - OAuth endpoints disabled (dev mode)");
            None
        }
    };

    // TASK-MCP-007/008 DB store-of-record: the payload sealer. `None` when MCP_KMS_KEY is unset (the
    // in-memory path). A malformed key fails the boot loudly rather than silently running unencrypted.
    let kms: Option<Arc<dyn cyberos_mcp_gateway::kms::Kms>> =
        match cyberos_mcp_gateway::kms::EnvKeyKms::from_env() {
            Ok(Some(k)) => {
                println!("KMS: MCP_KMS_KEY loaded - elicitation/task payloads sealed at rest");
                Some(Arc::new(k) as Arc<dyn cyberos_mcp_gateway::kms::Kms>)
            }
            Ok(None) => {
                // Fail closed: with auth + a database on but no key, the destructive-tool confirmation
                // flow is broken (responses cannot be sealed, so a held confirmation is un-respondable).
                // Refuse to start rather than run with a silently-broken safety control.
                if std::env::var("MCP_REQUIRE_AUTH").as_deref() == Ok("1") && oauth_pool.is_some() {
                    eprintln!(
                        "KMS: MCP_REQUIRE_AUTH=1 with a database but MCP_KMS_KEY is unset - destructive-tool \
                         confirmations would be un-respondable (payloads cannot be sealed); refusing to \
                         start. Set MCP_KMS_KEY (base64 of 32 bytes)."
                    );
                    std::process::exit(1);
                }
                None
            }
            Err(e) => {
                eprintln!("KMS: MCP_KMS_KEY invalid ({e}); refusing to start");
                std::process::exit(1);
            }
        };

    let state = AppState {
        registry,
        oauth_pool,
        elicitations: Arc::new(ElicitationStore::new()),
        tasks: Arc::new(TaskStore::new()),
        kms,
    };
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
