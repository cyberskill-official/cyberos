//! FR-MCP-002 — module server health lifecycle (DEC-2350 cadence, DEC-2351 enum).
//!
//! A module registers at startup and then sends a heartbeat every `HEARTBEAT_INTERVAL_SECS`.
//! After `UNHEALTHY_AFTER_SECS` without a heartbeat (3 missed 10s beats) the gateway treats
//! its server as unhealthy and stops offering/dispatching its tools; one or two missed beats
//! is `degraded` (still served). An explicit deregister is terminal until re-registration.
//!
//! Status is computed lazily from the last-heartbeat timestamp at read time (on `tools/list`,
//! `tools/call`, and `/mcp/healthz`), so there is no background reaper to coordinate; a
//! sweeper is a possible later optimisation, not a correctness requirement.

use std::time::Duration;

use serde::Serialize;

/// Cadence a registered module is expected to keep (DEC-2350).
pub const HEARTBEAT_INTERVAL_SECS: u64 = 10;
/// No heartbeat for this long => unhealthy (3 missed 10s beats; DEC-2350).
pub const UNHEALTHY_AFTER_SECS: u64 = 30;

/// Closed module-server health enum (DEC-2351). Serialises as snake_case strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ServerHealthStatus {
    /// Heartbeat within the last interval.
    Healthy,
    /// One or two missed beats; still served, but slipping.
    Degraded,
    /// Three or more missed beats; tools withdrawn until heartbeats resume.
    Unhealthy,
    /// Explicitly deregistered by the module; terminal until re-registration.
    Deregistered,
}

impl ServerHealthStatus {
    /// Stable string form (matches the serde representation).
    pub fn as_str(&self) -> &'static str {
        match self {
            ServerHealthStatus::Healthy => "healthy",
            ServerHealthStatus::Degraded => "degraded",
            ServerHealthStatus::Unhealthy => "unhealthy",
            ServerHealthStatus::Deregistered => "deregistered",
        }
    }

    /// Whether a server in this state may have its tools offered and dispatched. Degraded
    /// still counts as available (a transient miss should not withdraw the tool); unhealthy
    /// and deregistered do not.
    pub fn is_available(&self) -> bool {
        matches!(
            self,
            ServerHealthStatus::Healthy | ServerHealthStatus::Degraded
        )
    }
}

/// Classify a server from how long since its last heartbeat. Pure: the single source of
/// truth for the DEC-2350 thresholds.
pub fn classify(since_last_heartbeat: Duration, deregistered: bool) -> ServerHealthStatus {
    if deregistered {
        return ServerHealthStatus::Deregistered;
    }
    let age = since_last_heartbeat.as_secs_f64();
    if age <= HEARTBEAT_INTERVAL_SECS as f64 {
        ServerHealthStatus::Healthy
    } else if age <= UNHEALTHY_AFTER_SECS as f64 {
        ServerHealthStatus::Degraded
    } else {
        ServerHealthStatus::Unhealthy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_and_within_one_interval_is_healthy() {
        assert_eq!(
            classify(Duration::from_secs(0), false),
            ServerHealthStatus::Healthy
        );
        assert_eq!(
            classify(Duration::from_secs(10), false),
            ServerHealthStatus::Healthy
        );
    }

    #[test]
    fn one_to_two_missed_beats_is_degraded() {
        assert_eq!(
            classify(Duration::from_secs(11), false),
            ServerHealthStatus::Degraded
        );
        assert_eq!(
            classify(Duration::from_secs(30), false),
            ServerHealthStatus::Degraded
        );
    }

    #[test]
    fn three_or_more_missed_beats_is_unhealthy() {
        assert_eq!(
            classify(Duration::from_millis(30_001), false),
            ServerHealthStatus::Unhealthy
        );
        assert_eq!(
            classify(Duration::from_secs(120), false),
            ServerHealthStatus::Unhealthy
        );
    }

    #[test]
    fn deregistered_beats_any_age() {
        assert_eq!(
            classify(Duration::from_secs(0), true),
            ServerHealthStatus::Deregistered
        );
        assert_eq!(
            classify(Duration::from_secs(120), true),
            ServerHealthStatus::Deregistered
        );
    }

    #[test]
    fn availability_matches_spec() {
        assert!(ServerHealthStatus::Healthy.is_available());
        assert!(ServerHealthStatus::Degraded.is_available());
        assert!(!ServerHealthStatus::Unhealthy.is_available());
        assert!(!ServerHealthStatus::Deregistered.is_available());
    }
}
