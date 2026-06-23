//! obs-router boot config from env (FR-OBS-007). Every external dependency is optional so the service
//! starts and degrades safely: an unset CUO URL means triage errors -> confidence 0 -> PagerDuty; an
//! unset CHAT or PagerDuty target means that leg fails and the §1 #11 fallback chain takes over.

/// PagerDuty Events API v2 enqueue endpoint (the default; overridable for testing).
pub const DEFAULT_PAGERDUTY_ENDPOINT: &str = "https://events.pagerduty.com/v2/enqueue";

#[derive(Debug, Clone)]
pub struct Config {
    pub bind: String,
    /// Shared secret required in the `X-CyberOS-Webhook-Secret` header (§1 #13). Unset = auth disabled
    /// (dev only); a wrong secret is always 401 when this is set.
    pub webhook_secret: Option<String>,
    /// The CUO `obs.triage-alert` invocation URL (POST {skill, alert} -> TriageResult JSON).
    pub cuo_triage_url: Option<String>,
    /// The CHAT (`#oncall`) incoming-webhook URL.
    pub chat_webhook_url: Option<String>,
    /// The PagerDuty Events API v2 routing key.
    pub pagerduty_routing_key: Option<String>,
    pub pagerduty_endpoint: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            bind: env("OBS_ROUTER_BIND").unwrap_or_else(|| "0.0.0.0:7777".to_string()),
            webhook_secret: env("OBS_ROUTER_WEBHOOK_SECRET"),
            cuo_triage_url: env("OBS_CUO_TRIAGE_URL"),
            chat_webhook_url: env("OBS_CHAT_WEBHOOK_URL"),
            pagerduty_routing_key: env("OBS_PAGERDUTY_ROUTING_KEY"),
            pagerduty_endpoint: env("OBS_PAGERDUTY_ENDPOINT")
                .unwrap_or_else(|| DEFAULT_PAGERDUTY_ENDPOINT.to_string()),
        }
    }
}

/// A non-empty env var, trimmed; `None` if unset or blank.
fn env(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}
