//! TEN — tenancy plan tiers and caps (TASK-TEN-002).
//!
//! Compile-time plan caps are the single source of truth (DEC-772). HTTP handlers and
//! SQL migrations live beside this library; metering (TASK-TEN-004) consumes [`plans::caps`].

#![deny(missing_debug_implementations)]

pub mod handlers;
pub mod plans;

pub use plans::caps::{caps_for, TierCaps, ENTERPRISE, STARTER, TEAM};
pub use plans::tiers::{PlanChangeEffective, PlanTier, PLAN_TIER_CARDINALITY};
