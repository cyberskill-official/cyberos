//! `cyberos-obs-router` binary.

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use cyberos_obs_router::chat_post::HttpChatClient;
use cyberos_obs_router::cuo_triage::HttpCuoClient;
use cyberos_obs_router::memory::JsonlAuditSink;
use cyberos_obs_router::pagerduty::HttpPagerDutyClient;
use cyberos_obs_router::{app, RouterConfig, RouterState, SERVICE_BANNER};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "cyberos_obs_router=info,tower_http=info".to_string()),
        )
        .init();

    let webhook_secret = if let Ok(value) = std::env::var("OBS_ROUTER_WEBHOOK_SECRET") {
        value.trim().to_string()
    } else if let Ok(path) = std::env::var("OBS_ROUTER_WEBHOOK_SECRET_FILE") {
        std::fs::read_to_string(&path)
            .with_context(|| format!("read OBS_ROUTER_WEBHOOK_SECRET_FILE={path}"))?
            .trim()
            .to_string()
    } else {
        "cyberos-local-dev".to_string()
    };
    let mut config = RouterConfig::new(webhook_secret);
    if let Ok(channel) = std::env::var("OBS_ROUTER_CHAT_CHANNEL") {
        config.oncall_channel = channel;
    }
    if let Ok(url) = std::env::var("OBS_ROUTER_TEMPO_BASE_URL") {
        config.tempo_base_url = url;
    }

    let cuo_url = std::env::var("CUO_TRIAGE_URL")
        .unwrap_or_else(|_| "http://cuo:8080/skills/invoke".to_string());
    let chat_url = std::env::var("CHAT_WEBHOOK_URL")
        .unwrap_or_else(|_| "http://chat:8065/hooks/cyberos-obs-router".to_string());
    let pd_url = std::env::var("PAGERDUTY_EVENTS_URL")
        .unwrap_or_else(|_| "https://events.pagerduty.com/v2/enqueue".to_string());
    let pd_key = std::env::var("PAGERDUTY_ROUTING_KEY").unwrap_or_default();
    let audit_path = std::env::var("OBS_ROUTER_AUDIT_JSONL")
        .map(Into::into)
        .unwrap_or_else(|_| JsonlAuditSink::default_target_path());

    let state = RouterState::new(
        config,
        Arc::new(HttpCuoClient::new(cuo_url)),
        Arc::new(HttpChatClient::new(chat_url)),
        Arc::new(HttpPagerDutyClient::new(pd_url, pd_key)),
        Arc::new(JsonlAuditSink::new(audit_path)),
    );

    let addr: SocketAddr = std::env::var("OBS_ROUTER_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:7777".to_string())
        .parse()?;
    tracing::info!(%addr, "{SERVICE_BANNER}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app(state)).await?;
    Ok(())
}
