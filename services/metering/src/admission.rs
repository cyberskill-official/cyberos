//! Pure overage admission helper (TASK-TEN-207).

use crate::axes::MeteringAxis;
use crate::policy::{evaluate, blocked_axis, OverageDecision, OveragePolicy};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionRequest {
    pub axis: MeteringAxis,
    pub current: u64,
    pub quantity: u64,
    pub cap: Option<u64>,
    pub policy: OveragePolicy,
    pub warn_threshold_bps: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdmissionOutcome {
    Allow,
    Warn { current: u64, cap: u64 },
    Block { current: u64, cap: u64, axis: &'static str },
}

/// Evaluate whether `quantity` may proceed given current usage and cap.
pub fn admit(req: &AdmissionRequest) -> AdmissionOutcome {
    match evaluate(
        req.policy,
        req.current,
        req.quantity,
        req.cap,
        req.warn_threshold_bps,
    ) {
        OverageDecision::Allow => AdmissionOutcome::Allow,
        OverageDecision::WarnThreshold => match req.cap {
            Some(cap) => AdmissionOutcome::Warn {
                current: req.current,
                cap,
            },
            None => AdmissionOutcome::Allow,
        },
        OverageDecision::Block => match req.cap {
            Some(cap) => AdmissionOutcome::Block {
                current: req.current,
                cap,
                axis: blocked_axis(req.axis),
            },
            None => AdmissionOutcome::Allow,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_when_over_cap() {
        let out = admit(&AdmissionRequest {
            axis: MeteringAxis::ApiCalls,
            current: 10_000,
            quantity: 1,
            cap: Some(10_000),
            policy: OveragePolicy::Block,
            warn_threshold_bps: 8_000,
        });
        assert_eq!(
            out,
            AdmissionOutcome::Block {
                current: 10_000,
                cap: 10_000,
                axis: "api_calls",
            }
        );
    }

    #[test]
    fn warn_policy_never_blocks() {
        let out = admit(&AdmissionRequest {
            axis: MeteringAxis::ApiCalls,
            current: 10_000,
            quantity: 1,
            cap: Some(10_000),
            policy: OveragePolicy::Warn,
            warn_threshold_bps: 8_000,
        });
        assert!(matches!(out, AdmissionOutcome::Warn { .. }));
    }

    #[test]
    fn unlimited_cap_allows() {
        let out = admit(&AdmissionRequest {
            axis: MeteringAxis::ApiCalls,
            current: 1_000_000,
            quantity: 1,
            cap: None,
            policy: OveragePolicy::Block,
            warn_threshold_bps: 8_000,
        });
        assert_eq!(out, AdmissionOutcome::Allow);
    }
}
