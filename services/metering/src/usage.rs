//! Period usage aggregates for overage admission (TASK-TEN-206).

use chrono::{DateTime, Utc};

use crate::axes::MeteringAxis;
use crate::recorder::{InMemoryRecorder, MeteringEvent};

/// Sum quantities for `tenant` + `axis` with `occurred_at >= period_start`.
pub fn sum_in_memory(
    recorder: &InMemoryRecorder,
    tenant_id: &str,
    axis: MeteringAxis,
    period_start: DateTime<Utc>,
) -> u64 {
    recorder.sum_for(tenant_id, axis, period_start)
}

/// Calendar-month UTC period start containing `now`.
pub fn utc_month_start(now: DateTime<Utc>) -> DateTime<Utc> {
    use chrono::Datelike;
    now.date_naive()
        .with_day(1)
        .expect("day 1")
        .and_hms_opt(0, 0, 0)
        .expect("midnight")
        .and_utc()
}

/// Helper used by tests to seed synthetic usage.
pub fn event(
    tenant_id: &str,
    axis: MeteringAxis,
    quantity: u64,
    key: &str,
    at: DateTime<Utc>,
) -> MeteringEvent {
    MeteringEvent {
        tenant_id: tenant_id.into(),
        axis,
        quantity,
        idempotency_key: key.into(),
        source_service: "test".into(),
        occurred_at: at,
        extra: serde_json::json!({}),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recorder::Recorder;
    use chrono::Duration;

    #[test]
    fn sums_only_in_period() {
        let r = InMemoryRecorder::default();
        let start = utc_month_start(Utc::now());
        r.record(event("t", MeteringAxis::ApiCalls, 3, "a", start + Duration::hours(1)))
            .unwrap();
        r.record(event(
            "t",
            MeteringAxis::ApiCalls,
            9,
            "b",
            start - Duration::days(1),
        ))
        .unwrap();
        assert_eq!(sum_in_memory(&r, "t", MeteringAxis::ApiCalls, start), 3);
    }
}
