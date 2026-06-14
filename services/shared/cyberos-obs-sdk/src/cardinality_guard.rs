//! Cardinality guard for RED metric labels.

use std::collections::{HashMap, HashSet};
use std::sync::{LazyLock, Mutex};

use crate::red::Label;

/// Maximum unique label combinations accepted per service+metric.
pub const MAX_CARDINALITY_PER_METRIC: usize = 1000;

static SEEN: LazyLock<Mutex<HashMap<String, HashSet<String>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Return true when a metric series may be recorded.
pub fn check(service: &str, metric: &str, labels: &[Label]) -> bool {
    let guard_key = format!("{service}:{metric}");
    let combo = canonical_combo(labels);
    let mut seen = SEEN.lock().expect("cardinality guard lock");
    let entry = seen.entry(guard_key).or_default();
    if entry.contains(&combo) {
        return true;
    }
    if entry.len() >= MAX_CARDINALITY_PER_METRIC {
        tracing::warn!(
            service,
            metric,
            current = entry.len(),
            limit = MAX_CARDINALITY_PER_METRIC,
            "cardinality_overflow_blocked"
        );
        return false;
    }
    entry.insert(combo);
    true
}

/// Number of distinct label combinations observed for a service+metric.
pub fn seen_count(service: &str, metric: &str) -> usize {
    let seen = SEEN.lock().expect("cardinality guard lock");
    seen.get(&format!("{service}:{metric}"))
        .map(HashSet::len)
        .unwrap_or(0)
}

/// Reset guard state for deterministic tests.
pub fn reset_for_tests() {
    SEEN.lock().expect("cardinality guard lock").clear();
}

fn canonical_combo(labels: &[Label]) -> String {
    let mut pairs: Vec<_> = labels
        .iter()
        .map(|label| (label.key.as_str(), label.value.as_str()))
        .collect();
    pairs.sort_unstable();
    pairs
        .into_iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join(",")
}
