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

    let mut authenticator = build_authenticator().await?;
    // Opt-in audience check: only enforced when CHAT_TOKEN_AUD is set to the auth service's exact audience, so
    // enabling it can never lock everyone out by surprise (a wrong/blank value simply leaves the check off).
    if let Some(aud) = std::env::var("CHAT_TOKEN_AUD")
        .ok()
        .filter(|v| !v.trim().is_empty())
    {
        authenticator.require_audience(aud);
        tracing::info!("chat token audience validation enabled");
    }
    let authenticator = Arc::new(authenticator);
    // Refresh the JWKS on an interval so a rotated auth signing key is picked up without a chat restart. Only
    // runs when the key set came from a URL; the interval is CHAT_JWKS_REFRESH_SECS (default 300s).
    if authenticator.has_jwks_url() {
        let refresher = authenticator.clone();
        let secs = std::env::var("CHAT_JWKS_REFRESH_SECS")
            .ok()
            .and_then(|v| v.trim().parse::<u64>().ok())
            .filter(|n| *n > 0)
            .unwrap_or(300);
        tokio::spawn(async move {
            let mut tick = tokio::time::interval(std::time::Duration::from_secs(secs));
            tick.tick().await; // consume the immediate first tick; the boot fetch already loaded the keys
            loop {
                tick.tick().await;
                match refresher.refresh().await {
                    Ok(()) => tracing::debug!(target: "cyberos_chat::auth", "jwks refreshed"),
                    Err(e) => {
                        tracing::warn!(target: "cyberos_chat::auth", error = %e, "jwks refresh failed")
                    }
                }
            }
        });
    }

    // Attachment byte store + limits (richer-messages cluster). Default = db backend at the historical 5 MB
    // cap; production sets CHAT_ATTACHMENT_STORE=fs + a volume for large files off Postgres.
    let attachments = cyberos_chat::storage::AttachmentConfig::from_env()?;
    tracing::info!(
        store = attachments.store.kind(),
        max_bytes = attachments.max_bytes,
        max_files = attachments.max_files,
        "attachment storage configured"
    );

    let state = AppState {
        pool,
        audit_pool,
        capturer,
        authenticator,
        hub: Hub::default(),
        presence: Presence::default(),
        notifier: Notifier::default(),
        attachments,
        // FR-CHAT-268 — starts cold; each blocker's set is loaded on first use and invalidated
        // on every block/unblock.
        blocks: cyberos_chat::blocks::BlockCache::default(),
    };

    // FR-CHAT-269 §1 #17 — purge resolved reports (snapshot included) once past their 90-day window.
    //
    // A job, not a DB trigger: a trigger would delete rows out from under an administrator mid-read. Hourly
    // is ample for a 90-day window, and the first sweep runs at boot so a long-stopped instance catches up.
    // Best-effort: a failed sweep logs and the next tick retries. It must never take the service down —
    // being late to delete is recoverable, refusing to serve chat is not.
    {
        let pool = state.pool.clone();
        tokio::spawn(async move {
            let mut tick = tokio::time::interval(std::time::Duration::from_secs(3600));
            loop {
                tick.tick().await;
                if let Err(e) = cyberos_chat::moderation::purge_resolved_reports(&pool).await {
                    tracing::warn!(target: "cyberos_chat::moderation", error = %e, "report purge failed; retrying next tick");
                }
            }
        });
    }

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
        let mut auth =
            Authenticator::from_jwks(&json).map_err(|e| anyhow::anyhow!(e.to_string()))?;
        // Remember the URL so the background refresher can pick up a rotated signing key without a restart.
        auth.set_jwks_url(url);
        return Ok(auth);
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
