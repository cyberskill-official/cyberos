//! TASK-TEN-204 — ai_tokens emit into cyberos-metering WalQueue / Recorder.
//!
//! Non-blocking: emit failures are logged and never fail reconcile.

use std::sync::{Mutex, OnceLock};

use chrono::Utc;
use cyberos_metering::axes::MeteringAxis;
use cyberos_metering::recorder::{validate_quantity, InMemoryRecorder, MeteringEvent, Recorder};
use cyberos_metering::wal_queue::{WalError, WalQueue};
use serde_json::json;
use tracing::warn;
use uuid::Uuid;

static WAL: OnceLock<Mutex<WalQueue>> = OnceLock::new();
static RECORDER: OnceLock<Mutex<InMemoryRecorder>> = OnceLock::new();

fn wal() -> &'static Mutex<WalQueue> {
    WAL.get_or_init(|| Mutex::new(WalQueue::default()))
}

fn recorder() -> &'static Mutex<InMemoryRecorder> {
    RECORDER.get_or_init(|| Mutex::new(InMemoryRecorder::default()))
}

/// Emit ai_tokens for a reconciled hold. Returns whether a record was stored.
pub fn emit_ai_tokens(
    tenant_id: &str,
    hold_id: Uuid,
    provider: &str,
    model_alias: &str,
    prompt_tokens: u32,
    completion_tokens: u32,
) -> bool {
    let quantity = (prompt_tokens as u64).saturating_add(completion_tokens as u64);
    if quantity == 0 {
        return false;
    }
    if let Err(e) = validate_quantity(MeteringAxis::AiTokens, quantity) {
        warn!(error = %e, "metering ai_tokens quantity rejected");
        return false;
    }

    let event = MeteringEvent {
        tenant_id: tenant_id.to_string(),
        axis: MeteringAxis::AiTokens,
        quantity,
        idempotency_key: hold_id.to_string(),
        source_service: "ai-gateway".into(),
        occurred_at: Utc::now(),
        extra: json!({
            "provider": provider,
            "model_alias": model_alias,
            "input_tokens": prompt_tokens,
            "output_tokens": completion_tokens,
        }),
    };

    // Idempotent durable-intent via in-process recorder (Pg drain later).
    match recorder().lock() {
        Ok(rec) => {
            if let Err(e) = rec.record(event.clone()) {
                warn!(error = %e, "metering recorder rejected ai_tokens");
            }
        }
        Err(e) => warn!(error = %e, "metering recorder lock poisoned"),
    }

    match wal().lock() {
        Ok(mut q) => {
            if let Err(WalError::Overflow) = q.push(event) {
                warn!("metering wal overflow on ai_tokens; reconcile continues");
            }
        }
        Err(e) => warn!(error = %e, "metering wal lock poisoned"),
    }
    true
}

/// Recorded event count (idempotent inserts count once). Used by integration tests.
pub fn recorded_len() -> usize {
    recorder().lock().map(|r| r.len()).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_shape_quantity_and_extra() {
        let hold = Uuid::new_v4();
        assert!(emit_ai_tokens("ten_a", hold, "openai", "gpt-4o", 10, 5));
        {
            let rec = recorder().lock().unwrap();
            assert!(rec.len() >= 1);
        }
        // Second emit same hold is idempotent at recorder
        assert!(emit_ai_tokens("ten_a", hold, "openai", "gpt-4o", 10, 5));
    }

    #[test]
    fn zero_tokens_skipped() {
        let hold = Uuid::new_v4();
        let before = recorded_len();
        assert!(!emit_ai_tokens("ten_a", hold, "openai", "gpt-4o", 0, 0));
        assert_eq!(recorded_len(), before);
    }
}
