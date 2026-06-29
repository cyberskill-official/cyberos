//! cyberos-eval binary: load config, build state (governance pool + optional audit-chain pool + the
//! CyberOS token verifier), and serve. Mirrors `cyberos-chat`'s main: env-driven DB URLs, a JWKS-backed
//! authenticator fetched once at boot, a permissive-CORS opt-in for local dev, JSON logs.

use std::sync::Arc;

use cyberos_eval::{auth::Authenticator, db, router, AppState};

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

    let authenticator = build_authenticator().await?;

    let state = AppState {
        pool,
        audit_pool,
        authenticator: Arc::new(authenticator),
        version: env!("CARGO_PKG_VERSION"),
    };

    let addr = std::env::var("EVAL_LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:7760".to_string());
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!(addr = %addr, "cyberos-eval listening");
    axum::serve(listener, router(state)).await?;
    Ok(())
}

/// Token verifier from, in priority order: the auth JWKS URL (fetched once at startup, the
/// production-aligned path - default `http://auth:7700/.well-known/jwks.json`), inline JWKS JSON, a JWKS
/// file, or an HS256 secret (tests / local). Mirrors `cyberos_chat::main::build_authenticator`; the env
/// vars are the EVAL_AUTH_* analogues of CHAT_AUTH_*.
async fn build_authenticator() -> anyhow::Result<Authenticator> {
    if let Ok(json) = std::env::var("EVAL_AUTH_JWKS_JSON") {
        return Authenticator::from_jwks(&json).map_err(|e| anyhow::anyhow!(e.to_string()));
    }
    if let Ok(path) = std::env::var("EVAL_AUTH_JWKS_PATH") {
        let json = std::fs::read_to_string(&path)?;
        return Authenticator::from_jwks(&json).map_err(|e| anyhow::anyhow!(e.to_string()));
    }
    if let Ok(secret) = std::env::var("EVAL_AUTH_HS256_SECRET") {
        return Ok(Authenticator::from_hs256_secret(secret.as_bytes()));
    }
    // Default to the AUTH JWKS URL so a fresh deployment verifies real CyberOS tokens with no extra
    // config. The keys are fetched once here and cached in the Authenticator for the process lifetime.
    let url = std::env::var("EVAL_AUTH_JWKS_URL")
        .unwrap_or_else(|_| "http://auth:7700/.well-known/jwks.json".to_string());
    let json = reqwest::get(&url)
        .await
        .map_err(|e| anyhow::anyhow!("fetch JWKS from {url}: {e}"))?
        .text()
        .await
        .map_err(|e| anyhow::anyhow!("read JWKS body from {url}: {e}"))?;
    Authenticator::from_jwks(&json).map_err(|e| anyhow::anyhow!(e.to_string()))
}
