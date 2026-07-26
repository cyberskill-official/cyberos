//! WAL → Recorder drain (TASK-TEN-206).

use std::sync::Mutex;

use tracing::warn;

use crate::recorder::{RecordError, Recorder};
use crate::wal_queue::WalQueue;

/// Pop all WAL items into `recorder`. Returns number of newly recorded rows
/// (`Ok(Some(_))` inserts). Duplicate keys and quantity errors are skipped.
pub fn drain_to_recorder(wal: &Mutex<WalQueue>, recorder: &dyn Recorder) -> usize {
    let mut drained = 0usize;
    loop {
        let item = match wal.lock() {
            Ok(mut q) => q.pop(),
            Err(e) => {
                warn!(error = %e, "metering wal lock poisoned during drain");
                break;
            }
        };
        let Some(event) = item else {
            break;
        };
        match recorder.record(event) {
            Ok(Some(_)) => drained += 1,
            Ok(None) => {}
            Err(RecordError::Quantity(e)) => warn!(error = %e, "drain skipped bad quantity"),
            Err(e) => warn!(error = %e, "drain record failed"),
        }
    }
    drained
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::axes::MeteringAxis;
    use crate::recorder::{InMemoryRecorder, MeteringEvent};
    use chrono::Utc;

    #[test]
    fn drain_empties_wal_into_recorder() {
        let wal = Mutex::new(crate::wal_queue::WalQueue::new(100));
        let rec = InMemoryRecorder::default();
        for i in 0..3 {
            wal.lock()
                .unwrap()
                .push(MeteringEvent {
                    tenant_id: "t".into(),
                    axis: MeteringAxis::ApiCalls,
                    quantity: 1,
                    idempotency_key: format!("k{i}"),
                    source_service: "auth".into(),
                    occurred_at: Utc::now(),
                    extra: serde_json::json!({}),
                })
                .unwrap();
        }
        assert_eq!(drain_to_recorder(&wal, &rec), 3);
        assert_eq!(wal.lock().unwrap().depth(), 0);
        assert_eq!(rec.len(), 3);
        // Second drain of same keys via re-push of duplicate — only new keys count.
        wal.lock()
            .unwrap()
            .push(MeteringEvent {
                tenant_id: "t".into(),
                axis: MeteringAxis::ApiCalls,
                quantity: 1,
                idempotency_key: "k0".into(),
                source_service: "auth".into(),
                occurred_at: Utc::now(),
                extra: serde_json::json!({}),
            })
            .unwrap();
        assert_eq!(drain_to_recorder(&wal, &rec), 0);
        assert_eq!(rec.len(), 3);
    }
}
