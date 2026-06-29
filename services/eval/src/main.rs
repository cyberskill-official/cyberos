//! cyberos-eval binary: load config, build state (governance pool + optional audit-chain pool), and serve.
//! Mirrors `cyberos-chat`'s main: env-driven DB URLs, a permissive-CORS opt-in for local dev, JSON logs.

use cyberos_eval::{db, router, AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .json()
        .init();

    // The EVAL governance Postgres (notice / ack / category / grant / retention tables, per-tenant RLS).
    let database_url = std::env::var("EVAL_DATABASE_URL")
        .map_err(|_| anyhow::anyhow!("EVAL_DATABASE_URL is required"))?;
    let pool: db::Pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    // The memory module's Postgres (holds l1_audit_log). When set, governance mutations are hash-chained;
    // when unset (tests / local single-DB), they are logged. Mirrors chat's CHAT_AUDIT_DATABASE_URL.
    let audit_pool: Option<db::Pool> = match std::env::var("EVAL_AUDIT_DATABASE_URL") {
        Ok(url) => Some(
            sqlx::postgres::PgPoolOptions::new()
                .max_connections(4)
                .connect(&url)
                .await?,
        ),
        Err(_) => {
            tracing::warn!(
                "EVAL_AUDIT_DATABASE_URL unset; eval governance audit events are logged, not chained"
            );
            None
        }
    };

    let state = AppState {
        pool,
        audit_pool,
        version: env!("CARGO_PKG_VERSION"),
    };

    let addr = std::env::var("EVAL_LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:7760".to_string());
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!(addr = %addr, "cyberos-eval listening");
    axum::serve(listener, router(state)).await?;
    Ok(())
}
