//! Shared application state passed to every axum handler.

use sqlx::{postgres::PgPoolOptions, PgPool};
use std::sync::Arc;
use std::time::Duration;

use crate::auth::Authenticator;

/// Wraps the Postgres connection pool plus any other singletons (metrics
/// recorders, redis pool, etc) needed by handlers.
#[derive(Clone)]
pub struct AppState {
    /// Postgres connection pool — used for l2_memory / l2_entity / l2_edge.
    pub pg: PgPool,
    /// MEM-001 (R73): the CyberOS access-token verifier. Behind an `Arc` so the `/v1/memory` auth middleware
    /// shares one key set and a background task can refresh rotated JWKS keys without a restart.
    pub authenticator: Arc<Authenticator>,
}

impl AppState {
    /// Connect to Postgres using the canonical `DATABASE_URL` env var and build the token verifier.
    ///
    /// Errors propagate up so the binary entry point can fail fast on boot — never silently start without a
    /// working DB connection, and (MEM-001) never start without a configured token verifier, so `/v1/memory`
    /// can never fall back to header-trust.
    pub async fn connect_from_env() -> Result<Self, sqlx::Error> {
        let url = std::env::var("DATABASE_URL")
            .map_err(|_| sqlx::Error::Configuration("DATABASE_URL env var must be set".into()))?;

        let pg = PgPoolOptions::new()
            .max_connections(8)
            .acquire_timeout(Duration::from_secs(3))
            .connect(&url)
            .await?;

        let authenticator = crate::auth::build_authenticator()
            .await
            .map_err(|e| sqlx::Error::Configuration(format!("token verifier: {e}").into()))?;

        Ok(Self {
            pg,
            authenticator: Arc::new(authenticator),
        })
    }
}
