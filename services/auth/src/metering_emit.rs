//! TASK-TEN-205 — api_calls emit into cyberos-metering WalQueue / Recorder.
//!
//! Non-blocking: emit failures are logged and never fail the HTTP response.

use std::sync::{Mutex, OnceLock};

use chrono::{DateTime, Utc};
use cyberos_metering::axes::MeteringAxis;
use cyberos_metering::recorder::{validate_quantity, InMemoryRecorder, MeteringEvent, Recorder};
use cyberos_metering::wal_queue::{WalError, WalQueue};
use serde_json::json;
use tracing::warn;

static WAL: OnceLock<Mutex<WalQueue>> = OnceLock::new();
static RECORDER: OnceLock<Mutex<InMemoryRecorder>> = OnceLock::new();

fn wal() -> &'static Mutex<WalQueue> {
    WAL.get_or_init(|| Mutex::new(WalQueue::default()))
}

fn recorder() -> &'static Mutex<InMemoryRecorder> {
    RECORDER.get_or_init(|| Mutex::new(InMemoryRecorder::default()))
}

/// Emit one api_calls event (quantity = 1). Returns whether a record was attempted.
pub fn emit_api_call(tenant_id: &str, idempotency_key: &str, method: &str, path: &str) -> bool {
    let quantity = 1u64;
    if let Err(e) = validate_quantity(MeteringAxis::ApiCalls, quantity) {
        warn!(error = %e, "metering api_calls quantity rejected");
        return false;
    }

    let event = MeteringEvent {
        tenant_id: tenant_id.to_string(),
        axis: MeteringAxis::ApiCalls,
        quantity,
        idempotency_key: idempotency_key.to_string(),
        source_service: "auth".into(),
        occurred_at: Utc::now(),
        extra: json!({
            "method": method,
            "path": path,
        }),
    };

    match recorder().lock() {
        Ok(rec) => {
            if let Err(e) = rec.record(event.clone()) {
                warn!(error = %e, "metering recorder rejected api_calls");
            }
        }
        Err(e) => warn!(error = %e, "metering recorder lock poisoned"),
    }

    match wal().lock() {
        Ok(mut q) => {
            if let Err(WalError::Overflow) = q.push(event) {
                warn!("metering wal overflow on api_calls; request continues");
            }
        }
        Err(e) => warn!(error = %e, "metering wal lock poisoned"),
    }
    true
}

/// Strip query string so metering paths stay PII-light.
pub fn path_without_query(uri_path_and_query: &str) -> &str {
    uri_path_and_query
        .split_once('?')
        .map(|(p, _)| p)
        .unwrap_or(uri_path_and_query)
}

pub fn recorded_len() -> usize {
    recorder().lock().map(|r| r.len()).unwrap_or(0)
}

/// Period sum of api_calls for admission (TASK-TEN-207).
pub fn api_calls_sum(tenant_id: &str, period_start: DateTime<Utc>) -> u64 {
    recorder()
        .lock()
        .map(|r| r.sum_for(tenant_id, MeteringAxis::ApiCalls, period_start))
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emit_records_one() {
        let before = recorded_len();
        let key = format!("test-api-{}", uuid::Uuid::new_v4());
        assert!(emit_api_call("ten_a", &key, "GET", "/v1/admin/tenants"));
        assert!(recorded_len() > before);
    }

    #[test]
    fn idempotent_same_key() {
        let key = format!("test-api-idem-{}", uuid::Uuid::new_v4());
        assert!(emit_api_call("ten_a", &key, "GET", "/x"));
        let mid = recorded_len();
        assert!(emit_api_call("ten_a", &key, "GET", "/x"));
        assert_eq!(recorded_len(), mid);
    }

    #[test]
    fn strips_query() {
        assert_eq!(path_without_query("/v1/x?email=a@b.c"), "/v1/x");
        assert_eq!(path_without_query("/v1/x"), "/v1/x");
    }
}
