//! cyberos-chat binary: load config, build state (pool + JWKS verifier + hub), and serve.

use std::sync::Arc;

use cyberos_chat::{
    auth::Authenticator,
    notify::Notifier,
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

    let database_url = std::env::var("DATABASE_URL")
        .ok()
        .filter(|u| !u.trim().is_empty())
        .ok_or_else(|| anyhow::anyhow!("DATABASE_URL is required (and must be non-empty)"))?;
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    // DEC-2713 — CHAT_AUDIT_DATABASE_URL is the chat->brain link: when set, chat's audit rows (and, when
    // CAPTURE_ENABLED is on, its FR-MEMORY-122 interaction-events) chain into MEMORY's l1_audit_log. P0
    // left it unset; this is REQUIRED in production (see docs/deploy/p0-google-chat-runbook.md). The "unset"
    // line is now info, not warn, because dev/local legitimately runs without it.
    // An empty value (the compose default `${CHAT_AUDIT_DATABASE_URL:-}`) is "set but blank", which must be
    // treated as unset - otherwise sqlx tries to connect to "" and fails with "error with configuration:
    // relative URL without a base", crashing chat at boot.
    let audit_pool = match std::env::var("CHAT_AUDIT_DATABASE_URL")
        .ok()
        .filter(|u| !u.trim().is_empty())
    {
        Some(url) => Some(
            sqlx::postgres::PgPoolOptions::new()
                .max_connections(4)
                .connect(&url)
                .await?,
        ),
        None => {
            tracing::info!(
                "CHAT_AUDIT_DATABASE_URL unset or empty; chat audit events are logged, not chained \
                 (dev/local only — production MUST set this to the brain audit DB)"
            );
            None
        }
    };

    // FR-MEMORY-122 §1 #4 — build the capturer over the chat->brain audit pool. `Some` ONLY when
    // CAPTURE_ENABLED is truthy AND audit_pool is present (default off), so capture is dormant by default.
    // audit_pool is cloned so chat's existing audit::emit path keeps using it unchanged.
    let capturer = cyberos_capture::maybe_capturer(audit_pool.clone());

    let authenticator = build_authenticator().await?;

    let state = AppState {
        pool,
        audit_pool,
        capturer,
        authenticator: Arc::new(authenticator),
        hub: Hub::default(),
        presence: Presence::default(),
        notifier: Notifier::default(),
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
    // Read an env var only when it is set AND non-empty. An empty value (e.g. `CHAT_AUTH_JWKS_URL=` left in
    // .env) must fall through to the next verifier, not crash `reqwest::get("")` with "relative URL without
    // a base" - which takes the whole chat service down at boot.
    let env_set = |k: &str| std::env::var(k).ok().filter(|v| !v.trim().is_empty());

    if let Some(url) = env_set("CHAT_AUTH_JWKS_URL") {
        let json = reqwest::get(&url)
            .await
            .map_err(|e| anyhow::anyhow!("fetch JWKS from {url}: {e}"))?
            .text()
            .await
            .map_err(|e| anyhow::anyhow!("read JWKS body from {url}: {e}"))?;
        return Authenticator::from_jwks(&json).map_err(|e| anyhow::anyhow!(e.to_string()));
    }
    if let Some(json) = env_set("CHAT_AUTH_JWKS_JSON") {
        return Authenticator::from_jwks(&json).map_err(|e| anyhow::anyhow!(e.to_string()));
    }
    if let Some(path) = env_set("CHAT_AUTH_JWKS_PATH") {
        let json = std::fs::read_to_string(&path)?;
        return Authenticator::from_jwks(&json).map_err(|e| anyhow::anyhow!(e.to_string()));
    }
    if let Some(secret) = env_set("CHAT_AUTH_HS256_SECRET") {
        return Ok(Authenticator::from_hs256_secret(secret.as_bytes()));
    }
    anyhow::bail!(
        "no token verifier configured: set a non-empty CHAT_AUTH_JWKS_URL, CHAT_AUTH_JWKS_JSON, CHAT_AUTH_JWKS_PATH, or CHAT_AUTH_HS256_SECRET"
    )
}
