//! Plan-show view model (TASK-TEN-002).

use crate::plans::caps::{caps_for, TierCaps};
use crate::plans::tiers::PlanTier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanShow {
    pub tenant_id: String,
    pub tier: PlanTier,
    pub caps: TierCaps,
    pub is_founder_tenant: bool,
}

impl PlanShow {
    pub fn new(tenant_id: impl Into<String>, tier: PlanTier, is_founder_tenant: bool) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            tier,
            caps: caps_for(tier),
            is_founder_tenant,
        }
    }
}
