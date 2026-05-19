//! Shared application state passed to every axum handler.

use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

/// Wraps the Postgres connection pool plus any other singletons (metrics
/// recorders, redis pool, etc) needed by handlers.
#[derive(Clone)]
pub struct AppState {
    /// Postgres connection pool — used for l2_memory / l2_entity / l2_edge.
    pub pg: PgPool,
}

impl AppState {
    /// Connect to Postgres using the canonical `DATABASE_URL` env var.
    ///
    /// Errors propagate up so the binary entry point can fail fast on
    /// boot — never silently start without a working DB connection.
    pub async fn connect_from_env() -> Result<Self, sqlx::Error> {
        let url = std::env::var("DATABASE_URL").map_err(|_| {
            sqlx::Error::Configuration("DATABASE_URL env var must be set".into())
        })?;

        let pg = PgPoolOptions::new()
            .max_connections(8)
            .acquire_timeout(Duration::from_secs(3))
            .connect(&url)
            .await?;

        Ok(Self { pg })
    }
}
