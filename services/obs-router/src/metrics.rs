//! In-process FR-OBS-007 metrics.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use crate::severity::{Route, Severity};

/// Metric names emitted by the router.
pub mod names {
    /// Counter — alerts received by severity.
    pub const ALERTS_RECEIVED_TOTAL: &str = "obs_router_alerts_received_total";
    /// Counter — alerts routed by route, severity, and outcome.
    pub const ALERTS_ROUTED_TOTAL: &str = "obs_router_alerts_routed_total";
    /// Counter — CUO timeouts.
    pub const CUO_TIMEOUTS_TOTAL: &str = "obs_router_cuo_timeouts_total";
    /// Histogram — CUO confidence values.
    pub const TRIAGE_CONFIDENCE: &str = "obs_triage_confidence";
    /// Histogram — triage + route latency in ms.
    pub const TRIAGE_LATENCY_MS: &str = "obs_router_triage_latency_ms";
    /// Counter — CHAT ack callbacks.
    pub const ACKS_TOTAL: &str = "obs_router_acks_total";
    /// Counter — deduplicated alerts.
    pub const DEDUP_TOTAL: &str = "obs_router_dedup_total";
}

/// In-process metrics store used by tests and `/metrics`.
#[derive(Debug, Default)]
pub struct ObsRouterMetrics {
    alerts_received: Mutex<BTreeMap<String, u64>>,
    alerts_routed: Mutex<BTreeMap<(String, String, String), u64>>,
    acks: Mutex<BTreeMap<String, u64>>,
    triage_confidence: Mutex<Vec<f64>>,
    triage_latency_ms: Mutex<Vec<u128>>,
    cuo_timeouts: AtomicU64,
    dedup: AtomicU64,
}

impl ObsRouterMetrics {
    /// Increment received counter.
    pub fn inc_received(&self, severity: Severity) {
        *self
            .alerts_received
            .lock()
            .unwrap()
            .entry(severity.as_label().to_string())
            .or_default() += 1;
    }

    /// Increment routed counter.
    pub fn inc_routed(&self, route: Route, severity: Severity, outcome: &str) {
        *self
            .alerts_routed
            .lock()
            .unwrap()
            .entry((
                route.as_label().to_string(),
                severity.as_label().to_string(),
                outcome.to_string(),
            ))
            .or_default() += 1;
    }

    /// Increment CUO timeout counter.
    pub fn inc_cuo_timeout(&self) {
        self.cuo_timeouts.fetch_add(1, Ordering::Relaxed);
    }

    /// CUO timeout count.
    pub fn cuo_timeouts_total(&self) -> u64 {
        self.cuo_timeouts.load(Ordering::Relaxed)
    }

    /// Increment dedup counter.
    pub fn inc_dedup(&self) {
        self.dedup.fetch_add(1, Ordering::Relaxed);
    }

    /// Dedup count.
    pub fn dedup_total(&self) -> u64 {
        self.dedup.load(Ordering::Relaxed)
    }

    /// Observe confidence.
    pub fn observe_confidence(&self, confidence: f64) {
        self.triage_confidence.lock().unwrap().push(confidence);
    }

    /// Observe latency.
    pub fn observe_latency_ms(&self, latency_ms: u128) {
        self.triage_latency_ms.lock().unwrap().push(latency_ms);
    }

    /// Increment ack counter.
    pub fn inc_ack(&self, source: &str) {
        *self
            .acks
            .lock()
            .unwrap()
            .entry(source.to_string())
            .or_default() += 1;
    }

    /// Render a small Prometheus text exposition.
    pub fn render_prometheus(&self) -> String {
        let mut out = String::new();
        for (severity, value) in self.alerts_received.lock().unwrap().iter() {
            out.push_str(&format!(
                "{}{{severity=\"{}\"}} {}\n",
                names::ALERTS_RECEIVED_TOTAL,
                severity,
                value
            ));
        }
        for ((route, severity, outcome), value) in self.alerts_routed.lock().unwrap().iter() {
            out.push_str(&format!(
                "{}{{route=\"{}\",severity=\"{}\",outcome=\"{}\"}} {}\n",
                names::ALERTS_ROUTED_TOTAL,
                route,
                severity,
                outcome,
                value
            ));
        }
        out.push_str(&format!(
            "{} {}\n",
            names::CUO_TIMEOUTS_TOTAL,
            self.cuo_timeouts_total()
        ));
        out.push_str(&format!("{} {}\n", names::DEDUP_TOTAL, self.dedup_total()));
        for (source, value) in self.acks.lock().unwrap().iter() {
            out.push_str(&format!(
                "{}{{ack_source=\"{}\"}} {}\n",
                names::ACKS_TOTAL,
                source,
                value
            ));
        }
        out
    }
}
