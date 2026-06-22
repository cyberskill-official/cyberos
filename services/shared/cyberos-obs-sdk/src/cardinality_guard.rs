//! FR-OBS-003 §1 #9 - the cardinality guard. It refuses to register a metric series whose label set
//! would push a `service:metric` past `MAX_CARDINALITY_PER_METRIC` unique label combinations - the most
//! common observability failure mode (one Prometheus series per label combo; an unbounded label like
//! `user_id` balloons storage and slows every query). A refusal is counted on
//! `obs_sdk_cardinality_blocked_total{service, metric}` and logged.

use opentelemetry::metrics::Counter;
use opentelemetry::{global, KeyValue};
use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};

const MAX_CARDINALITY_PER_METRIC: usize = 1000;

/// `service:metric` -> the set of label combos seen for it. `None` until first use.
static SEEN: Mutex<Option<HashMap<String, HashSet<String>>>> = Mutex::new(None);
static BLOCKED: OnceLock<Counter<u64>> = OnceLock::new();

/// Initialise the blocked-counter (idempotent across services).
pub fn init(_service: &str) {
    let _ = BLOCKED.set(
        global::meter("cyberos")
            .u64_counter("obs_sdk_cardinality_blocked_total")
            .build(),
    );
}

/// Order-independent identity for a label set, so `{a,b}` and `{b,a}` count as one combo.
fn label_combo(labels: &[KeyValue]) -> String {
    let mut parts: Vec<String> = labels
        .iter()
        .map(|kv| format!("{}={}", kv.key.as_str(), kv.value.as_str()))
        .collect();
    parts.sort();
    parts.join(",")
}

/// True if this label combo may be emitted (already seen, or still under the budget). False if
/// registering it would exceed `MAX_CARDINALITY_PER_METRIC` for this `service:metric` - the series is
/// refused, counted, and logged.
pub fn check(service: &str, metric: &str, labels: &[KeyValue]) -> bool {
    let key = format!("{service}:{metric}");
    let combo = label_combo(labels);

    {
        let mut guard = SEEN.lock().unwrap();
        let map = guard.get_or_insert_with(HashMap::new);
        let entry = map.entry(key).or_default();
        if entry.contains(&combo) {
            return true;
        }
        if entry.len() < MAX_CARDINALITY_PER_METRIC {
            entry.insert(combo);
            return true;
        }
    }

    // Over budget: refuse, count, log (lock released above so emission never holds the guard lock).
    if let Some(blocked) = BLOCKED.get() {
        blocked.add(
            1,
            &[
                KeyValue::new("service", service.to_string()),
                KeyValue::new("metric", metric.to_string()),
            ],
        );
    }
    eprintln!(
        "cardinality_overflow_blocked: {service}/{metric} at {MAX_CARDINALITY_PER_METRIC} unique label combos"
    );
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn route(r: &str) -> Vec<KeyValue> {
        vec![KeyValue::new("route", r.to_string())]
    }

    #[test]
    fn blocks_past_the_budget_and_is_idempotent_under_it() {
        let svc = "card-test-a";
        for i in 0..MAX_CARDINALITY_PER_METRIC {
            assert!(
                check(svc, "m", &route(&format!("/r{i}"))),
                "combo {i} should be allowed"
            );
        }
        // the 1001st distinct combo is refused
        assert!(!check(svc, "m", &route("/r-overflow")));
        // an already-seen combo is still allowed (idempotent, does not consume budget)
        assert!(check(svc, "m", &route("/r0")));
    }

    #[test]
    fn isolates_budget_per_service_and_metric() {
        assert!(check("card-test-b", "m", &route("/only")));
        assert!(check("card-test-c", "m", &route("/only")));
        assert!(check("card-test-b", "other-metric", &route("/only")));
    }

    #[test]
    fn label_order_does_not_change_identity() {
        let a = vec![
            KeyValue::new("x", "1".to_string()),
            KeyValue::new("y", "2".to_string()),
        ];
        let b = vec![
            KeyValue::new("y", "2".to_string()),
            KeyValue::new("x", "1".to_string()),
        ];
        assert_eq!(label_combo(&a), label_combo(&b));
    }
}
