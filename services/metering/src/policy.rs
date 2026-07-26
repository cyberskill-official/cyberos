//! Overage policy (DEC-710) — consumes plan-tier caps from cyberos-ten when wired.

use serde::{Deserialize, Serialize};

use crate::axes::MeteringAxis;

pub const OVERAGE_POLICY_CARDINALITY: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OveragePolicy {
    Block,
    Warn,
    Allow,
}

impl OveragePolicy {
    pub const ALL: [OveragePolicy; OVERAGE_POLICY_CARDINALITY] = [
        OveragePolicy::Block,
        OveragePolicy::Warn,
        OveragePolicy::Allow,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverageDecision {
    Allow,
    WarnThreshold,
    Block,
}

/// Evaluate overage for a prospective +1 (or +qty) against a monthly cap.
/// `warn_threshold` is a fraction in basis points (default 8000 = 80%).
pub fn evaluate(
    policy: OveragePolicy,
    current: u64,
    qty: u64,
    cap: Option<u64>,
    warn_threshold_bps: u32,
) -> OverageDecision {
    let Some(cap) = cap else {
        return OverageDecision::Allow;
    };
    let next = current.saturating_add(qty);
    match policy {
        OveragePolicy::Allow => OverageDecision::Allow,
        OveragePolicy::Block => {
            if next > cap {
                OverageDecision::Block
            } else if next * 10_000 >= cap * warn_threshold_bps as u64 {
                OverageDecision::WarnThreshold
            } else {
                OverageDecision::Allow
            }
        }
        OveragePolicy::Warn => {
            if next * 10_000 >= cap * warn_threshold_bps as u64 {
                OverageDecision::WarnThreshold
            } else {
                OverageDecision::Allow
            }
        }
    }
}

/// Helper: axis label for 402 bodies.
pub fn blocked_axis(axis: MeteringAxis) -> &'static str {
    axis.as_str()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_when_over_cap() {
        assert_eq!(
            evaluate(OveragePolicy::Block, 10_000, 1, Some(10_000), 8_000),
            OverageDecision::Block
        );
    }

    #[test]
    fn warn_at_80_percent() {
        assert_eq!(
            evaluate(OveragePolicy::Warn, 7_999, 1, Some(10_000), 8_000),
            OverageDecision::WarnThreshold
        );
    }
}
