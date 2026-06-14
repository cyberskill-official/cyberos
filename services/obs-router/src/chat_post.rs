//! CHAT posting client and message rendering.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::alertmanager_webhook::Alert;
use crate::cuo_triage::TriageResult;
use crate::severity::Severity;

/// CHAT button.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatButton {
    /// Button label.
    pub label: String,
    /// Action id.
    pub action_id: String,
    /// Callback value.
    pub value: String,
}

/// Rendered CHAT message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Channel, usually `#oncall`.
    pub channel: String,
    /// Alert fingerprint.
    pub alert_id: String,
    /// Message title.
    pub title: String,
    /// Message body.
    pub text: String,
    /// Action buttons.
    pub buttons: Vec<ChatButton>,
}

/// CHAT post receipt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatReceipt {
    /// CHAT message id.
    pub message_id: String,
}

/// CHAT client error.
#[derive(Debug, Error)]
pub enum ChatError {
    /// HTTP transport or status failure.
    #[error("chat_http: {0}")]
    Http(String),
}

/// Client capable of posting and updating CHAT messages.
#[async_trait]
pub trait ChatClient: Send + Sync + std::fmt::Debug {
    /// Post a triage message.
    async fn post(&self, message: ChatMessage) -> Result<ChatReceipt, ChatError>;
    /// Update a message to show an ack.
    async fn update_ack(
        &self,
        message_id: &str,
        user: &str,
        timestamp: &str,
    ) -> Result<(), ChatError>;
    /// Update a deduplicated message counter.
    async fn update_dedup_counter(&self, message_id: &str, count: u64) -> Result<(), ChatError>;
    /// Post last-resort emergency CHAT notification.
    async fn post_emergency(&self, alert: &Alert, reason: &str) -> Result<ChatReceipt, ChatError>;
}

/// HTTP CHAT client.
#[derive(Debug, Clone)]
pub struct HttpChatClient {
    endpoint: String,
    client: reqwest::Client,
}

impl HttpChatClient {
    /// Create a client for a CHAT webhook endpoint.
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl ChatClient for HttpChatClient {
    async fn post(&self, message: ChatMessage) -> Result<ChatReceipt, ChatError> {
        self.post_action("post", serde_json::to_value(message).unwrap())
            .await
    }

    async fn update_ack(
        &self,
        message_id: &str,
        user: &str,
        timestamp: &str,
    ) -> Result<(), ChatError> {
        self.post_action(
            "ack",
            serde_json::json!({
                "message_id": message_id,
                "acked_by": user,
                "acked_at": timestamp,
            }),
        )
        .await
        .map(|_| ())
    }

    async fn update_dedup_counter(&self, message_id: &str, count: u64) -> Result<(), ChatError> {
        self.post_action(
            "dedup",
            serde_json::json!({
                "message_id": message_id,
                "count": count,
            }),
        )
        .await
        .map(|_| ())
    }

    async fn post_emergency(&self, alert: &Alert, reason: &str) -> Result<ChatReceipt, ChatError> {
        self.post_action(
            "emergency",
            serde_json::json!({
                "alert_id": alert.alert_id(),
                "alert_name": alert.alert_name(),
                "reason": reason,
            }),
        )
        .await
    }
}

impl HttpChatClient {
    async fn post_action(
        &self,
        action: &str,
        payload: serde_json::Value,
    ) -> Result<ChatReceipt, ChatError> {
        let response = self
            .client
            .post(&self.endpoint)
            .json(&serde_json::json!({
                "action": action,
                "payload": payload,
            }))
            .send()
            .await
            .map_err(|err| ChatError::Http(err.to_string()))?;
        let response = response
            .error_for_status()
            .map_err(|err| ChatError::Http(err.to_string()))?;
        response
            .json::<ChatReceipt>()
            .await
            .map_err(|err| ChatError::Http(err.to_string()))
    }
}

/// Build the required FR-OBS-007 CHAT post.
pub fn build_chat_message(
    alert: &Alert,
    triage: &TriageResult,
    severity: Severity,
    channel: &str,
    tempo_base_url: &str,
) -> ChatMessage {
    let alert_id = alert.alert_id();
    let alert_name = alert.alert_name();
    let runbook = triage
        .suggested_runbook
        .as_ref()
        .map(|r| format!("Runbook: <{}|{}>", r.url, r.title))
        .or_else(|| {
            alert
                .annotation("runbook_url")
                .map(|url| format!("Runbook: <{url}|alert runbook>"))
        })
        .unwrap_or_else(|| "Runbook: none".to_string());
    let trace = alert
        .trace_id()
        .map(|trace_id| {
            format!(
                "Trace: <{}/trace/{}|{}>",
                tempo_base_url, trace_id, trace_id
            )
        })
        .unwrap_or_else(|| "Trace: none".to_string());
    let text = format!(
        "CUO triage (confidence {:.2}): {}\nSuspected cause: {}\n{}\n{}",
        triage.confidence, triage.summary, triage.suspected_cause, runbook, trace
    );
    ChatMessage {
        channel: channel.to_string(),
        alert_id: alert_id.clone(),
        title: format!("[{}] {}", severity.as_label(), alert_name),
        text,
        buttons: vec![
            ChatButton {
                label: "Ack".to_string(),
                action_id: "ack".to_string(),
                value: alert_id.clone(),
            },
            ChatButton {
                label: "Escalate to PD".to_string(),
                action_id: "escalate".to_string(),
                value: alert_id,
            },
        ],
    }
}
