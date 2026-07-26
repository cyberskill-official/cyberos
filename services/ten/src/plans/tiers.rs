//! Closed plan-tier and effective enums (DEC-770, DEC-771, DEC-777).

use serde::{Deserialize, Serialize};

/// Exactly three plan tiers (DEC-770 / DEC-771). Adding a fourth requires a migration + DEC.
pub const PLAN_TIER_CARDINALITY: usize = 3;

/// Closed plan-tier set (`plan_tier` Postgres enum).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanTier {
    Starter,
    Team,
    Enterprise,
}

impl PlanTier {
    pub const ALL: [PlanTier; PLAN_TIER_CARDINALITY] = [
        PlanTier::Starter,
        PlanTier::Team,
        PlanTier::Enterprise,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            PlanTier::Starter => "starter",
            PlanTier::Team => "team",
            PlanTier::Enterprise => "enterprise",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "starter" => Some(PlanTier::Starter),
            "team" => Some(PlanTier::Team),
            "enterprise" => Some(PlanTier::Enterprise),
            _ => None,
        }
    }

    /// Rank for upgrade/downgrade detection (higher = more generous).
    pub fn rank(self) -> u8 {
        match self {
            PlanTier::Starter => 0,
            PlanTier::Team => 1,
            PlanTier::Enterprise => 2,
        }
    }

    pub fn is_upgrade(from: Self, to: Self) -> bool {
        to.rank() > from.rank()
    }

    pub fn is_downgrade(from: Self, to: Self) -> bool {
        to.rank() < from.rank()
    }
}

/// When a plan change takes effect (DEC-773).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanChangeEffective {
    Immediate,
    NextPeriod,
    DeferBillingOnly,
}

pub const PLAN_CHANGE_EFFECTIVE_CARDINALITY: usize = 3;

impl PlanChangeEffective {
    pub const ALL: [PlanChangeEffective; PLAN_CHANGE_EFFECTIVE_CARDINALITY] = [
        PlanChangeEffective::Immediate,
        PlanChangeEffective::NextPeriod,
        PlanChangeEffective::DeferBillingOnly,
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_tier_cardinality_is_three() {
        assert_eq!(PlanTier::ALL.len(), PLAN_TIER_CARDINALITY);
        assert_eq!(PLAN_TIER_CARDINALITY, 3);
    }

    #[test]
    fn effective_cardinality_is_three() {
        assert_eq!(
            PlanChangeEffective::ALL.len(),
            PLAN_CHANGE_EFFECTIVE_CARDINALITY
        );
    }
}
