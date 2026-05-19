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

use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .json()
        .init();

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
        .route("/v1/email/healthz", axum::routing::get({
            let pool = pool.clone();
            move || {
                let pool = pool.clone();
                async move {
                    let h = cyberos_email::handlers::healthz(&pool).await
                        .map(axum::Json)
                        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));
                    match h {
                        Ok(j) => j.into_response(),
                        Err((c, s)) => (c, s).into_response(),
                    }
                }
            }
        }))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind(bind).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

use axum::response::IntoResponse;
