//! Idempotent metering recorder (TASK-TEN-004 §1 #3 / #19).

use std::collections::HashSet;
use std::sync::Mutex;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::axes::MeteringAxis;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeteringEvent {
    pub tenant_id: String,
    pub axis: MeteringAxis,
    pub quantity: u64,
    pub idempotency_key: String,
    pub source_service: String,
    pub occurred_at: DateTime<Utc>,
    #[serde(default)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum QuantityError {
    #[error("metering_quantity_out_of_range axis={axis} quantity={quantity} max={max}")]
    OutOfRange {
        axis: &'static str,
        quantity: u64,
        max: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RecordError {
    #[error(transparent)]
    Quantity(#[from] QuantityError),
    #[error("tenant_not_found")]
    TenantNotFound,
    #[error("period_frozen")]
    PeriodFrozen,
    #[error("storage_error")]
    Storage,
}

pub fn validate_quantity(axis: MeteringAxis, quantity: u64) -> Result<(), QuantityError> {
    let (min, max) = match axis {
        MeteringAxis::Seats => (0, 100_000),
        MeteringAxis::ApiCalls => (1, 1_000_000),
        MeteringAxis::AiTokens => (1, 10_000_000),
        MeteringAxis::StorageBytes => (0, 10_000_000_000_000),
    };
    if quantity < min || quantity > max {
        return Err(QuantityError::OutOfRange {
            axis: axis.as_str(),
            quantity,
            max,
        });
    }
    Ok(())
}

pub trait Recorder {
    /// Record an event. Duplicate idempotency keys return `Ok(None)` (no second audit).
    fn record(&self, event: MeteringEvent) -> Result<Option<Uuid>, RecordError>;
}

#[derive(Debug, Default)]
pub struct InMemoryRecorder {
    seen: Mutex<HashSet<(String, String, String)>>, // tenant, axis, key
    events: Mutex<Vec<(Uuid, MeteringEvent)>>,
}

impl InMemoryRecorder {
    pub fn len(&self) -> usize {
        self.events.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Sum quantities for tenant+axis with `occurred_at >= period_start`.
    pub fn sum_for(
        &self,
        tenant_id: &str,
        axis: MeteringAxis,
        period_start: DateTime<Utc>,
    ) -> u64 {
        self.events
            .lock()
            .unwrap()
            .iter()
            .filter(|(_, e)| {
                e.tenant_id == tenant_id && e.axis == axis && e.occurred_at >= period_start
            })
            .map(|(_, e)| e.quantity)
            .sum()
    }
}

impl Recorder for InMemoryRecorder {
    fn record(&self, event: MeteringEvent) -> Result<Option<Uuid>, RecordError> {
        validate_quantity(event.axis, event.quantity)?;
        let key = (
            event.tenant_id.clone(),
            event.axis.as_str().to_string(),
            event.idempotency_key.clone(),
        );
        let mut seen = self.seen.lock().unwrap();
        if !seen.insert(key) {
            return Ok(None);
        }
        let id = Uuid::new_v4();
        self.events.lock().unwrap().push((id, event));
        Ok(Some(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idempotent_second_insert() {
        let r = InMemoryRecorder::default();
        let e = MeteringEvent {
            tenant_id: "t".into(),
            axis: MeteringAxis::ApiCalls,
            quantity: 1,
            idempotency_key: "k1".into(),
            source_service: "auth".into(),
            occurred_at: Utc::now(),
            extra: serde_json::json!({}),
        };
        assert!(r.record(e.clone()).unwrap().is_some());
        assert!(r.record(e).unwrap().is_none());
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn rejects_zero_api_calls() {
        assert!(validate_quantity(MeteringAxis::ApiCalls, 0).is_err());
    }
}
