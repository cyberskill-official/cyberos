//! FR-OBS-006 — Tail-based sampling policy.

use serde::{Deserialize, Serialize};

/// Tail-sampling decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SamplingDecision {
    /// Keep the trace.
    Keep,
    /// Drop the trace.
    Drop,
}

/// Deterministic sampling rule: keep all errors and sample normal traces by rate.
pub fn decide(status_code: u16, trace_id_hex: &str, normal_rate: f64) -> SamplingDecision {
    if status_code >= 500 {
        return SamplingDecision::Keep;
    }
    let rate = normal_rate.clamp(0.0, 1.0);
    let prefix = trace_id_hex.get(..8).unwrap_or("0");
    let value = u32::from_str_radix(prefix, 16).unwrap_or(0) as f64 / u32::MAX as f64;
    if value < rate {
        SamplingDecision::Keep
    } else {
        SamplingDecision::Drop
    }
}

/// Validate the canonical FR policy.
pub fn validate_policy(error_rate: f64, normal_rate: f64) -> Result<(), String> {
    if (error_rate - 1.0).abs() > f64::EPSILON {
        return Err("errors_must_sample_at_100_percent".into());
    }
    if (normal_rate - 0.10).abs() > f64::EPSILON {
        return Err("normal_must_sample_at_10_percent".into());
    }
    Ok(())
}
