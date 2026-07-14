//! obs-router boot config from env (TASK-OBS-007). Every external dependency is optional so the service
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
    /// Exact runbook URLs the CUO triage skill is allowed to suggest (the KB runbook index; OBS-007 P2
    /// hardening). A suggested runbook is shown in CHAT only if its URL is EXACTLY in this set; anything
    /// else - including the SKILL.md example URL a local model may copy - is dropped. Empty = fail-closed:
    /// no runbook link is ever shown until the index is configured. Set via `OBS_RUNBOOK_ALLOWLIST`
    /// (comma- or whitespace-separated).
    pub runbook_allowlist: Vec<String>,
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
            runbook_allowlist: list_env("OBS_RUNBOOK_ALLOWLIST"),
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

/// A comma- or whitespace-separated env var parsed into a trimmed, de-duplicated, non-empty list.
/// Unset or blank yields an empty list.
fn list_env(key: &str) -> Vec<String> {
    let raw = std::env::var(key).unwrap_or_default();
    let mut out: Vec<String> = Vec::new();
    for item in raw.split([',', ' ', '\t', '\n', '\r']) {
        let s = item.trim();
        if !s.is_empty() && !out.iter().any(|e| e == s) {
            out.push(s.to_string());
        }
    }
    out
}
