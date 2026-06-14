//! In-process compliance-view metrics.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

/// Metric names.
pub mod names {
    /// Counter — requests by view, format, and outcome.
    pub const REQUESTS_TOTAL: &str = "obs_compliance_view_requests_total";
    /// Histogram samples — latency.
    pub const LATENCY_MS: &str = "obs_compliance_view_latency_ms";
    /// Histogram samples — rows returned.
    pub const ROWS_RETURNED: &str = "obs_compliance_view_rows_returned";
    /// Counter — PII leak attempts.
    pub const PII_LEAK_ATTEMPTED_TOTAL: &str = "obs_compliance_pii_leak_attempted_total";
}

/// Metrics store.
#[derive(Debug, Default)]
pub struct Metrics {
    requests: Mutex<BTreeMap<(String, String, String), u64>>,
    latency: Mutex<BTreeMap<String, Vec<u128>>>,
    rows: Mutex<BTreeMap<String, Vec<usize>>>,
    pii: AtomicU64,
}

impl Metrics {
    /// Increment request counter.
    pub fn inc_request(&self, view: &str, format: &str, outcome: &str) {
        *self
            .requests
            .lock()
            .unwrap()
            .entry((view.to_string(), format.to_string(), outcome.to_string()))
            .or_default() += 1;
    }

    /// Observe latency.
    pub fn observe_latency(&self, view: &str, ms: u128) {
        self.latency
            .lock()
            .unwrap()
            .entry(view.to_string())
            .or_default()
            .push(ms);
    }

    /// Observe rows returned.
    pub fn observe_rows(&self, view: &str, rows: usize) {
        self.rows
            .lock()
            .unwrap()
            .entry(view.to_string())
            .or_default()
            .push(rows);
    }

    /// Increment PII leak counter.
    pub fn inc_pii_leak(&self) {
        self.pii.fetch_add(1, Ordering::Relaxed);
    }

    /// PII leak counter.
    pub fn pii_leak_total(&self) -> u64 {
        self.pii.load(Ordering::Relaxed)
    }

    /// Render Prometheus text.
    pub fn render_prometheus(&self) -> String {
        let mut out = String::new();
        for ((view, format, outcome), value) in self.requests.lock().unwrap().iter() {
            out.push_str(&format!(
                "{}{{view=\"{}\",format=\"{}\",outcome=\"{}\"}} {}\n",
                names::REQUESTS_TOTAL,
                view,
                format,
                outcome,
                value
            ));
        }
        out.push_str(&format!(
            "{} {}\n",
            names::PII_LEAK_ATTEMPTED_TOTAL,
            self.pii_leak_total()
        ));
        out
    }
}
