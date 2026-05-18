//! Shared application state.

use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

#[derive(Clone)]
pub struct AppState {
    pub pg: PgPool,
}

impl AppState {
    pub async fn connect_from_env() -> Result<Self, sqlx::Error> {
        let url = std::env::var("DATABASE_URL").map_err(|_| {
            sqlx::Error::Configuration("DATABASE_URL env var must be set".into())
        })?;

        let pg = PgPoolOptions::new()
            .max_connections(8)
            .acquire_timeout(Duration::from_secs(3))
            .after_connect(|conn, _meta| {
                Box::pin(async move {
                    // Switch every connection to the least-privilege app role
                    // so RLS policies apply. Root-tenant ops use SET ROLE in
                    // the bootstrap path; the default for everyday queries
                    // must be cyberos_app.
                    sqlx::query("SET ROLE cyberos_app")
                        .execute(conn)
                        .await
                        .ok();
                    Ok(())
                })
            })
            .connect(&url)
            .await?;

        Ok(Self { pg })
    }
}
