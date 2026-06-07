//! FR-OBS-003 — Per-service RED metric aggregation.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// One observed service request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestObservation {
    /// Service emitting the request metric.
    pub service: String,
    /// Tenant label propagated from auth.
    pub tenant_id: String,
    /// HTTP-ish status code.
    pub status_code: u16,
    /// Duration in milliseconds.
    pub duration_ms: u64,
}

/// RED metrics snapshot for a service+tenant pair.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedSnapshot {
    /// Request count.
    pub rate_count: u64,
    /// Error count (status >= 500).
    pub error_count: u64,
    /// p95 duration in milliseconds.
    pub duration_p95_ms: u64,
}

/// Aggregate observations by `(service, tenant_id)`.
pub fn aggregate_red(
    observations: &[RequestObservation],
) -> BTreeMap<(String, String), RedSnapshot> {
    let mut grouped: BTreeMap<(String, String), Vec<&RequestObservation>> = BTreeMap::new();
    for obs in observations {
        grouped
            .entry((obs.service.clone(), obs.tenant_id.clone()))
            .or_default()
            .push(obs);
    }
    grouped
        .into_iter()
        .map(|(key, vals)| {
            let mut durations: Vec<u64> = vals.iter().map(|o| o.duration_ms).collect();
            durations.sort_unstable();
            let idx = ((durations.len() as f64 * 0.95).ceil() as usize).saturating_sub(1);
            let p95 = durations.get(idx).copied().unwrap_or(0);
            let errors = vals.iter().filter(|o| o.status_code >= 500).count() as u64;
            (
                key,
                RedSnapshot {
                    rate_count: vals.len() as u64,
                    error_count: errors,
                    duration_p95_ms: p95,
                },
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn red_metrics_are_grouped_by_tenant() {
        let rows = vec![
            RequestObservation {
                service: "api".into(),
                tenant_id: "a".into(),
                status_code: 200,
                duration_ms: 10,
            },
            RequestObservation {
                service: "api".into(),
                tenant_id: "a".into(),
                status_code: 503,
                duration_ms: 30,
            },
            RequestObservation {
                service: "api".into(),
                tenant_id: "b".into(),
                status_code: 200,
                duration_ms: 5,
            },
        ];
        let out = aggregate_red(&rows);
        assert_eq!(out[&("api".into(), "a".into())].error_count, 1);
        assert_eq!(out[&("api".into(), "b".into())].rate_count, 1);
    }
}
