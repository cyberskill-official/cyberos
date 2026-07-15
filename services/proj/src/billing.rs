//! TASK-PROJ-006/007 — billable cascade and billing-mode rollups.

use crate::rate_card::BillingRole;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskClass {
    Delivery,
    Discovery,
    Support,
    Internal,
    Warranty,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BillingSource {
    MemberOverride,
    TaskClass,
    RoleDefault,
    Fallback,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BillableDecision {
    pub billable: bool,
    pub source: BillingSource,
    pub reason: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BillableRules {
    pub member_overrides: BTreeMap<Uuid, bool>,
    pub task_class_defaults: BTreeMap<TaskClass, bool>,
    pub role_defaults: BTreeMap<BillingRole, bool>,
    pub fallback: bool,
}

pub fn resolve_billable(
    rules: &BillableRules,
    member_id: Uuid,
    task_class: TaskClass,
    role: BillingRole,
) -> BillableDecision {
    if let Some(v) = rules.member_overrides.get(&member_id) {
        return BillableDecision {
            billable: *v,
            source: BillingSource::MemberOverride,
            reason: "member override".into(),
        };
    }
    if let Some(v) = rules.task_class_defaults.get(&task_class) {
        return BillableDecision {
            billable: *v,
            source: BillingSource::TaskClass,
            reason: format!("task class {task_class:?}"),
        };
    }
    if let Some(v) = rules.role_defaults.get(&role) {
        return BillableDecision {
            billable: *v,
            source: BillingSource::RoleDefault,
            reason: format!("role {role:?}"),
        };
    }
    BillableDecision {
        billable: rules.fallback,
        source: BillingSource::Fallback,
        reason: "fallback".into(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BillingMode {
    TimeAndMaterials,
    FixedFee,
    Retainer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BillingConfig {
    pub mode: BillingMode,
    pub hourly_rate_minor: i64,
    pub fixed_fee_minor: i64,
    pub retainer_included_hours: u32,
    pub retainer_fee_minor: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeEntry {
    pub minutes: u32,
    pub billable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BillingRollup {
    pub mode: BillingMode,
    pub billable_minutes: u32,
    pub included_minutes: u32,
    pub overage_minutes: u32,
    pub amount_minor: i64,
}

pub fn rollup(entries: &[TimeEntry], config: BillingConfig) -> BillingRollup {
    let billable_minutes: u32 = entries
        .iter()
        .filter(|e| e.billable)
        .map(|e| e.minutes)
        .sum();
    match config.mode {
        BillingMode::TimeAndMaterials => BillingRollup {
            mode: config.mode,
            billable_minutes,
            included_minutes: 0,
            overage_minutes: billable_minutes,
            amount_minor: prorate_minutes(billable_minutes, config.hourly_rate_minor),
        },
        BillingMode::FixedFee => BillingRollup {
            mode: config.mode,
            billable_minutes,
            included_minutes: billable_minutes,
            overage_minutes: 0,
            amount_minor: config.fixed_fee_minor,
        },
        BillingMode::Retainer => {
            let included = config.retainer_included_hours * 60;
            let overage = billable_minutes.saturating_sub(included);
            BillingRollup {
                mode: config.mode,
                billable_minutes,
                included_minutes: included.min(billable_minutes),
                overage_minutes: overage,
                amount_minor: config.retainer_fee_minor
                    + prorate_minutes(overage, config.hourly_rate_minor),
            }
        }
    }
}

fn prorate_minutes(minutes: u32, hourly_rate_minor: i64) -> i64 {
    (minutes as i64 * hourly_rate_minor + 59) / 60
}
