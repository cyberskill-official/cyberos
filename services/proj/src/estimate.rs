//! FR-PROJ-013 — estimate calibration snapshots.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EstimateObservation {
    pub member_id: Uuid,
    pub estimated_hours: f64,
    pub actual_hours: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CalibrationSnapshot {
    pub member_id: Uuid,
    pub sample_count: u32,
    pub bias_ratio: f64,
    pub confidence: f64,
}

pub fn calibrate(member_id: Uuid, observations: &[EstimateObservation]) -> CalibrationSnapshot {
    let member_obs: Vec<_> = observations
        .iter()
        .filter(|o| o.member_id == member_id && o.estimated_hours > 0.0)
        .collect();
    let sample_count = member_obs.len() as u32;
    if sample_count == 0 {
        return CalibrationSnapshot {
            member_id,
            sample_count: 0,
            bias_ratio: 1.0,
            confidence: 0.0,
        };
    }
    let prior_weight = 2.0;
    let observed_sum: f64 = member_obs
        .iter()
        .map(|o| o.actual_hours / o.estimated_hours)
        .sum();
    let bias_ratio = (prior_weight * 1.0 + observed_sum) / (prior_weight + sample_count as f64);
    let confidence = (sample_count as f64 / 20.0).min(1.0);
    CalibrationSnapshot {
        member_id,
        sample_count,
        bias_ratio,
        confidence,
    }
}
