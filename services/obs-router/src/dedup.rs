//! Five-minute alert fingerprint deduplication.

use std::collections::BTreeMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::severity::{Route, Severity};

/// Stored alert route state for ack/escalate callbacks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlertRecord {
    /// Alert id/fingerprint.
    pub alert_id: String,
    /// Alert name.
    pub alert_name: String,
    /// Severity.
    pub severity: Severity,
    /// Trace id.
    pub trace_id: Option<String>,
    /// Last route used.
    pub route: Route,
    /// CHAT message id, when posted.
    pub chat_message_id: Option<String>,
    /// PagerDuty dedup key, when triggered.
    pub pagerduty_dedup_key: Option<String>,
}

/// Dedup outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DedupOutcome {
    /// First alert in the window.
    New,
    /// Duplicate alert with fire count.
    Duplicate {
        /// Number of fires seen in the active dedup window.
        count: u64,
        /// Prior route record, if the first fire has already been delivered.
        record: Option<AlertRecord>,
    },
}

#[derive(Debug, Clone)]
struct DedupEntry {
    first_seen: Instant,
    count: u64,
    record: Option<AlertRecord>,
}

/// In-memory dedup store.
#[derive(Debug, Default)]
pub struct DedupStore {
    entries: Mutex<BTreeMap<String, DedupEntry>>,
}

impl DedupStore {
    /// Check and record an alert fire.
    pub fn check(&self, alert_id: &str, window: Duration) -> DedupOutcome {
        let now = Instant::now();
        let mut entries = self.entries.lock().unwrap();
        match entries.get_mut(alert_id) {
            Some(entry) if now.duration_since(entry.first_seen) <= window => {
                entry.count += 1;
                DedupOutcome::Duplicate {
                    count: entry.count,
                    record: entry.record.clone(),
                }
            }
            _ => {
                entries.insert(
                    alert_id.to_string(),
                    DedupEntry {
                        first_seen: now,
                        count: 1,
                        record: None,
                    },
                );
                DedupOutcome::New
            }
        }
    }

    /// Store route metadata after first delivery.
    pub fn mark_routed(&self, record: AlertRecord) {
        let mut entries = self.entries.lock().unwrap();
        entries
            .entry(record.alert_id.clone())
            .and_modify(|entry| entry.record = Some(record.clone()))
            .or_insert(DedupEntry {
                first_seen: Instant::now(),
                count: 1,
                record: Some(record),
            });
    }

    /// Lookup a routed alert.
    pub fn get(&self, alert_id: &str) -> Option<AlertRecord> {
        self.entries
            .lock()
            .unwrap()
            .get(alert_id)
            .and_then(|entry| entry.record.clone())
    }
}
