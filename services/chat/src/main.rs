//! cyberos-chat binary: load config, build state (pool + JWKS verifier + hub), and serve.

use std::sync::Arc;

use cyberos_chat::{
    auth::Authenticator,
    realtime::{Hub, Presence},
    router, AppState,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .json()
        .init();

    let database_url =
        std::env::var("DATABASE_URL").map_err(|_| anyhow::anyhow!("DATABASE_URL is required"))?;
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    let audit_pool = match std::env::var("CHAT_AUDIT_DATABASE_URL") {
        Ok(url) => Some(
            sqlx::postgres::PgPoolOptions::new()
                .max_connections(4)
                .connect(&url)
                .await?,
        ),
        Err(_) => {
            tracing::warn!("CHAT_AUDIT_DATABASE_URL unset; chat audit events are logged, not chained");
            None
        }
    };

    let authenticator = build_authenticator().await?;

    let state = AppState {
        pool,
        audit_pool,
        authenticator: Arc::new(authenticator),
        hub: Hub::default(),
        presence: Presence::default(),
    };

    let addr = std::env::var("CHAT_LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:7720".to_string());
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!(addr = %addr, "cyberos-chat listening");
    axum::serve(listener, router(state)).await?;
    Ok(())
}

/// Token verifier from, in priority order: the auth JWKS URL (fetched once at startup, the
/// production-aligned path), inline JWKS JSON, a JWKS file, or an HS256 secret (tests / local).
async fn build_authenticator() -> anyhow::Result<Authenticator> {
    if let Ok(url) = std::env::var("CHAT_AUTH_JWKS_URL") {
        let json = reqwest::get(&url)
            .await
            .map_err(|e| anyhow::anyhow!("fetch JWKS from {url}: {e}"))?
            .text()
            .await
            .map_err(|e| anyhow::anyhow!("read JWKS body from {url}: {e}"))?;
        return Authenticator::from_jwks(&json).map_err(|e| anyhow::anyhow!(e.to_string()));
    }
    if let Ok(json) = std::env::var("CHAT_AUTH_JWKS_JSON") {
        return Authenticator::from_jwks(&json).map_err(|e| anyhow::anyhow!(e.to_string()));
    }
    if let Ok(path) = std::env::var("CHAT_AUTH_JWKS_PATH") {
        let json = std::fs::read_to_string(&path)?;
        return Authenticator::from_jwks(&json).map_err(|e| anyhow::anyhow!(e.to_string()));
    }
    if let Ok(secret) = std::env::var("CHAT_AUTH_HS256_SECRET") {
        return Ok(Authenticator::from_hs256_secret(secret.as_bytes()));
    }
    anyhow::bail!(
        "no token verifier configured: set CHAT_AUTH_JWKS_JSON, CHAT_AUTH_JWKS_PATH, or CHAT_AUTH_HS256_SECRET"
    )
}
