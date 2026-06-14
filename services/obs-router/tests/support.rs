#![allow(dead_code)]

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use cyberos_obs_router::alertmanager_webhook::Alert;
use cyberos_obs_router::chat_post::{ChatClient, ChatError, ChatMessage, ChatReceipt};
use cyberos_obs_router::cuo_triage::{CuoError, RunbookRef, TriageClient, TriageResult};
use cyberos_obs_router::memory::{AuditError, AuditRow, AuditSink};
use cyberos_obs_router::pagerduty::{
    PagerDutyClient, PagerDutyError, PagerDutyIncident, PagerDutyReceipt,
};
use cyberos_obs_router::{RouterConfig, RouterState};

#[derive(Debug)]
pub struct MockCuo {
    result: Mutex<VecDeque<Result<TriageResult, CuoError>>>,
    delay: Duration,
}

impl MockCuo {
    pub fn confidence(confidence: f64) -> Arc<Self> {
        Arc::new(Self {
            result: Mutex::new(VecDeque::from([Ok(TriageResult {
                confidence,
                summary: "Recent surge in RED latency.".to_string(),
                suspected_cause: "index rebalance".to_string(),
                suggested_runbook: Some(RunbookRef {
                    kb_article_id: "kb-obs-1".to_string(),
                    title: "Pause ingest".to_string(),
                    url: "https://kb.cyberos.world/runbooks/pause-ingest".to_string(),
                }),
            })])),
            delay: Duration::ZERO,
        })
    }

    pub fn failing() -> Arc<Self> {
        Arc::new(Self {
            result: Mutex::new(VecDeque::from([Err(CuoError::Http("boom".to_string()))])),
            delay: Duration::ZERO,
        })
    }

    pub fn timing_out() -> Arc<Self> {
        Arc::new(Self {
            result: Mutex::new(VecDeque::from([Ok(TriageResult::fallback())])),
            delay: Duration::from_millis(50),
        })
    }
}

#[async_trait]
impl TriageClient for MockCuo {
    async fn triage(&self, _alert: &Alert) -> Result<TriageResult, CuoError> {
        if !self.delay.is_zero() {
            tokio::time::sleep(self.delay).await;
        }
        self.result
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_else(|| Ok(TriageResult::fallback()))
    }
}

#[derive(Debug, Default)]
pub struct MockChat {
    pub posts: Mutex<Vec<ChatMessage>>,
    pub emergency: Mutex<Vec<String>>,
    pub acks: Mutex<Vec<(String, String)>>,
    pub dedup_updates: Mutex<Vec<(String, u64)>>,
    fail_post: bool,
    fail_emergency: bool,
}

impl MockChat {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub fn failing_post() -> Arc<Self> {
        Arc::new(Self {
            fail_post: true,
            ..Self::default()
        })
    }

    pub fn failing_emergency() -> Arc<Self> {
        Arc::new(Self {
            fail_emergency: true,
            ..Self::default()
        })
    }
}

#[async_trait]
impl ChatClient for MockChat {
    async fn post(&self, message: ChatMessage) -> Result<ChatReceipt, ChatError> {
        if self.fail_post {
            return Err(ChatError::Http("chat down".to_string()));
        }
        let mut posts = self.posts.lock().unwrap();
        let message_id = format!("chat-{}", posts.len() + 1);
        posts.push(message);
        Ok(ChatReceipt { message_id })
    }

    async fn update_ack(
        &self,
        message_id: &str,
        user: &str,
        _timestamp: &str,
    ) -> Result<(), ChatError> {
        self.acks
            .lock()
            .unwrap()
            .push((message_id.to_string(), user.to_string()));
        Ok(())
    }

    async fn update_dedup_counter(&self, message_id: &str, count: u64) -> Result<(), ChatError> {
        self.dedup_updates
            .lock()
            .unwrap()
            .push((message_id.to_string(), count));
        Ok(())
    }

    async fn post_emergency(&self, alert: &Alert, reason: &str) -> Result<ChatReceipt, ChatError> {
        if self.fail_emergency {
            return Err(ChatError::Http("emergency chat down".to_string()));
        }
        self.emergency
            .lock()
            .unwrap()
            .push(format!("{}:{}", alert.alert_name(), reason));
        Ok(ChatReceipt {
            message_id: "emergency-1".to_string(),
        })
    }
}

#[derive(Debug, Default)]
pub struct MockPagerDuty {
    pub incidents: Mutex<Vec<PagerDutyIncident>>,
    pub resolves: Mutex<Vec<String>>,
    fail_trigger: bool,
}

impl MockPagerDuty {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub fn failing_trigger() -> Arc<Self> {
        Arc::new(Self {
            fail_trigger: true,
            ..Self::default()
        })
    }
}

#[async_trait]
impl PagerDutyClient for MockPagerDuty {
    async fn trigger(
        &self,
        incident: PagerDutyIncident,
    ) -> Result<PagerDutyReceipt, PagerDutyError> {
        if self.fail_trigger {
            return Err(PagerDutyError::Http("pd down".to_string()));
        }
        let dedup_key = incident.dedup_key.clone();
        self.incidents.lock().unwrap().push(incident);
        Ok(PagerDutyReceipt { dedup_key })
    }

    async fn resolve(&self, dedup_key: &str) -> Result<(), PagerDutyError> {
        self.resolves.lock().unwrap().push(dedup_key.to_string());
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct RecordingAudit {
    pub rows: Mutex<Vec<AuditRow>>,
}

#[async_trait]
impl AuditSink for RecordingAudit {
    async fn emit(&self, row: AuditRow) -> Result<(), AuditError> {
        self.rows.lock().unwrap().push(row);
        Ok(())
    }
}

pub fn test_state(
    cuo: Arc<MockCuo>,
    chat: Arc<MockChat>,
    pd: Arc<MockPagerDuty>,
    audit: Arc<RecordingAudit>,
) -> RouterState {
    let mut config = RouterConfig::new("secret");
    config.cuo_timeout = Duration::from_millis(10);
    RouterState::new(config, cuo, chat, pd, audit)
}

pub fn webhook(severity: &str, alertname: &str, fingerprint: &str) -> serde_json::Value {
    serde_json::json!({
        "version": "4",
        "groupKey": "group",
        "status": "firing",
        "receiver": "cyberos-obs-router",
        "alerts": [{
            "status": "firing",
            "labels": {
                "alertname": alertname,
                "severity": severity,
                "tenant_id": "tenant-a",
                "trace_id": "0af7651916cd43dd8448eb211c80319c"
            },
            "annotations": {
                "summary": "p99 > 500ms",
                "runbook_url": "https://kb.cyberos.world/runbooks/memory-latency"
            },
            "startsAt": "2026-06-15T00:00:00Z",
            "fingerprint": fingerprint
        }]
    })
}
