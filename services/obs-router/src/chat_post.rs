//! HTTP client that posts a triage summary to the CHAT `#oncall` channel (FR-OBS-007 §1 #4). Targets a
//! CHAT (Mattermost, FR-CHAT-001) incoming-webhook URL and includes the alert badge, the CUO summary and
//! suspected cause, the suggested runbook, a trace_id link, and ack / escalate actions whose buttons call
//! back to this service. An unset URL fails, so the §1 #11 fallback to PagerDuty takes over.

use crate::alertmanager_webhook::Alert;
use crate::notify::{ChatClient, NotifyError};
use crate::triage::Triage;

pub struct HttpChatClient {
    client: reqwest::Client,
    webhook_url: Option<String>,
    /// The public base URL of this router, used to build the ack / escalate button callbacks.
    self_base_url: String,
}

impl HttpChatClient {
    pub fn new(webhook_url: Option<String>, self_base_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            webhook_url,
            self_base_url,
        }
    }

    fn message(&self, alert: &Alert, triage: &Triage, request_id: &str) -> serde_json::Value {
        let runbook = triage
            .suggested_runbook
            .as_deref()
            .map_or_else(|| "none".to_string(), |u| format!("[runbook]({u})"));
        let trace = alert
            .trace_id
            .as_deref()
            .map_or_else(|| "n/a".to_string(), |t| format!("`{t}`"));
        let text = format!(
            "**[{sev}] {name}**\n{summary}\nSuspected cause: {cause}\nRunbook: {runbook}  ·  trace_id: {trace}",
            sev = alert.severity.label(),
            name = alert.name,
            summary = triage.summary,
            cause = triage.suspected_cause,
        );
        let ack = format!("{}/ack/{}", self.self_base_url, alert.fingerprint);
        let escalate = format!("{}/escalate/{}", self.self_base_url, alert.fingerprint);
        serde_json::json!({
            "text": text,
            "props": {
                "attachments": [{
                    "actions": [
                        { "id": "ack", "name": "Ack", "integration": { "url": ack, "context": { "request_id": request_id } } },
                        { "id": "escalate", "name": "Escalate to PagerDuty", "integration": { "url": escalate } }
                    ]
                }]
            }
        })
    }
}

impl ChatClient for HttpChatClient {
    async fn post(
        &self,
        alert: &Alert,
        triage: &Triage,
        request_id: &str,
    ) -> Result<(), NotifyError> {
        let Some(url) = self.webhook_url.as_ref() else {
            return Err(NotifyError("OBS_CHAT_WEBHOOK_URL not configured".into()));
        };
        self.client
            .post(url)
            .json(&self.message(alert, triage, request_id))
            .send()
            .await
            .map_err(|e| NotifyError(e.to_string()))?
            .error_for_status()
            .map_err(|e| NotifyError(e.to_string()))?;
        Ok(())
    }
}
