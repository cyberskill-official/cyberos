//! Compile-time per-tier caps (DEC-772, DEC-778..781).
//!
//! NULL / `None` means unlimited for that axis. Enterprise keeps a finite AI-token
//! cap (DEC-780) because tokens map to provider pass-through cost.

use super::tiers::PlanTier;

/// Caps along the four TASK-TEN-004 metering axes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TierCaps {
    pub seats: Option<u64>,
    pub api_calls_per_month: Option<u64>,
    pub ai_tokens_per_month: Option<u64>,
    pub storage_bytes: Option<u64>,
}

pub const GIB: u64 = 1024 * 1024 * 1024;
pub const TIB: u64 = 1024 * GIB;

/// Starter: 3 seats · 10k API · 100k tokens · 1 GiB (DEC-778..781).
pub const STARTER: TierCaps = TierCaps {
    seats: Some(3),
    api_calls_per_month: Some(10_000),
    ai_tokens_per_month: Some(100_000),
    storage_bytes: Some(GIB),
};

/// Team: 25 seats · 500k API · 5M tokens · 100 GiB.
pub const TEAM: TierCaps = TierCaps {
    seats: Some(25),
    api_calls_per_month: Some(500_000),
    ai_tokens_per_month: Some(5_000_000),
    storage_bytes: Some(100 * GIB),
};

/// Enterprise: unlimited seats/API · 50M tokens · 1 TiB.
pub const ENTERPRISE: TierCaps = TierCaps {
    seats: None,
    api_calls_per_month: None,
    ai_tokens_per_month: Some(50_000_000),
    storage_bytes: Some(TIB),
};

/// List price in USD cents / month (for proration math in plan_change).
pub const TIER_PRICE_CENTS_MONTHLY: [(PlanTier, u64); 3] = [
    (PlanTier::Starter, 0),
    (PlanTier::Team, 2_900),
    (PlanTier::Enterprise, 9_900),
];

pub const fn caps_for(tier: PlanTier) -> TierCaps {
    match tier {
        PlanTier::Starter => STARTER,
        PlanTier::Team => TEAM,
        PlanTier::Enterprise => ENTERPRISE,
    }
}

pub fn price_cents(tier: PlanTier) -> u64 {
    TIER_PRICE_CENTS_MONTHLY
        .iter()
        .find(|(t, _)| *t == tier)
        .map(|(_, p)| *p)
        .unwrap_or(0)
}

/// Axis that failed a downgrade cap check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapAxis {
    Seats,
    ApiCalls,
    AiTokens,
    StorageBytes,
}

impl CapAxis {
    pub fn as_str(self) -> &'static str {
        match self {
            CapAxis::Seats => "seats",
            CapAxis::ApiCalls => "api_calls",
            CapAxis::AiTokens => "ai_tokens",
            CapAxis::StorageBytes => "storage_bytes",
        }
    }
}

/// Current-period usage snapshot used for downgrade violation checks (DEC-774).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CurrentUsage {
    pub seats: u64,
    pub api_calls: u64,
    pub ai_tokens: u64,
    pub storage_bytes: u64,
}

/// Returns the first axis where usage exceeds the target-tier cap.
pub fn first_downgrade_violation(usage: &CurrentUsage, target: PlanTier) -> Option<(CapAxis, u64, u64)> {
    let caps = caps_for(target);
    if let Some(cap) = caps.seats {
        if usage.seats > cap {
            return Some((CapAxis::Seats, usage.seats, cap));
        }
    }
    if let Some(cap) = caps.api_calls_per_month {
        if usage.api_calls > cap {
            return Some((CapAxis::ApiCalls, usage.api_calls, cap));
        }
    }
    if let Some(cap) = caps.ai_tokens_per_month {
        if usage.ai_tokens > cap {
            return Some((CapAxis::AiTokens, usage.ai_tokens, cap));
        }
    }
    if let Some(cap) = caps.storage_bytes {
        if usage.storage_bytes > cap {
            return Some((CapAxis::StorageBytes, usage.storage_bytes, cap));
        }
    }
    None
}

/// Integer proration in cents (DEC-773 / §1 #11). Positive = tenant owes.
pub fn proration_cents(
    from: PlanTier,
    to: PlanTier,
    days_remaining: u32,
    days_in_period: u32,
) -> i64 {
    if days_in_period == 0 {
        return 0;
    }
    let delta = price_cents(to) as i64 - price_cents(from) as i64;
    (delta * days_remaining as i64) / days_in_period as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starter_caps_match_dec() {
        assert_eq!(STARTER.seats, Some(3));
        assert_eq!(STARTER.api_calls_per_month, Some(10_000));
        assert_eq!(STARTER.ai_tokens_per_month, Some(100_000));
        assert_eq!(STARTER.storage_bytes, Some(GIB));
    }

    #[test]
    fn enterprise_unlimited_axes() {
        assert!(ENTERPRISE.seats.is_none());
        assert!(ENTERPRISE.api_calls_per_month.is_none());
        assert_eq!(ENTERPRISE.ai_tokens_per_month, Some(50_000_000));
    }

    #[test]
    fn downgrade_violation_detects_seats() {
        let usage = CurrentUsage {
            seats: 10,
            ..Default::default()
        };
        let v = first_downgrade_violation(&usage, PlanTier::Starter).unwrap();
        assert_eq!(v.0, CapAxis::Seats);
        assert_eq!(v.1, 10);
        assert_eq!(v.2, 3);
    }

    #[test]
    fn proration_upgrade_is_positive() {
        // Half period remaining, Team→Enterprise: (9900-2900)*15/30 = 3500
        let p = proration_cents(PlanTier::Team, PlanTier::Enterprise, 15, 30);
        assert_eq!(p, 3500);
    }
}
