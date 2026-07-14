//! The (severity, confidence) routing decision (TASK-OBS-007 §1 #3), as a pure function separate from the
//! HTTP I/O. This is the correctness core: sev-1 always pages both channels; otherwise CUO's confidence
//! decides CHAT (at or above the floor) vs PagerDuty (below it). A CUO timeout or error is passed as
//! confidence 0 (§1 #9), so it falls through to PagerDuty - never a silent drop (§1 #11).

use crate::severity::Severity;

/// The confidence floor for trusting CUO triage enough to post to CHAT instead of paging (DEC-170).
pub const CONFIDENCE_FLOOR: f64 = 0.70;

/// Where an alert routes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    /// Post the triage summary to the CHAT `#oncall` channel.
    Chat,
    /// Page on-call via PagerDuty.
    PagerDuty,
    /// Both - a sev-1 alert always pages CHAT and PagerDuty (DEC-171).
    Both,
}

impl Route {
    /// The metric / audit label form (`"chat" | "pagerduty" | "both"`).
    pub fn label(self) -> &'static str {
        match self {
            Route::Chat => "chat",
            Route::PagerDuty => "pagerduty",
            Route::Both => "both",
        }
    }
}

/// Clamp a CUO confidence into `[0.0, 1.0]`. A skill bug returning above 1.0 is treated as 1.0; a NaN or
/// negative value is treated as 0.0 - the cautious end, which routes to PagerDuty (§10).
pub fn clamp_confidence(confidence: f64) -> f64 {
    if confidence.is_nan() {
        0.0
    } else {
        confidence.clamp(0.0, 1.0)
    }
}

/// Decide the route for an alert (§1 #3). A sev-1 alert routes to `Both` regardless of confidence;
/// otherwise CHAT when the clamped confidence is at or above the floor, else PagerDuty. Pass a CUO
/// timeout or error as `confidence = 0.0` so it falls through to PagerDuty (§1 #9, #11).
pub fn decide(severity: Severity, confidence: f64) -> Route {
    if severity.is_sev1() {
        return Route::Both;
    }
    if clamp_confidence(confidence) >= CONFIDENCE_FLOOR {
        Route::Chat
    } else {
        Route::PagerDuty
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sev1_always_routes_both_regardless_of_confidence() {
        for c in [0.0, 0.5, 0.69, 0.7, 1.0, -1.0, f64::NAN, 2.0] {
            assert_eq!(decide(Severity::Sev1, c), Route::Both, "conf {c}");
        }
    }

    #[test]
    fn non_sev1_at_or_above_floor_goes_to_chat() {
        for sev in [Severity::Sev2, Severity::Sev3, Severity::Sev4] {
            assert_eq!(decide(sev, 0.70), Route::Chat, "{sev:?} at floor");
            assert_eq!(decide(sev, 0.95), Route::Chat, "{sev:?} high");
        }
    }

    #[test]
    fn non_sev1_below_floor_pages_pagerduty() {
        assert_eq!(decide(Severity::Sev2, 0.69), Route::PagerDuty);
        assert_eq!(decide(Severity::Sev3, 0.0), Route::PagerDuty);
    }

    #[test]
    fn cuo_failure_as_zero_confidence_pages_never_drops() {
        // §1 #9/#11: a timeout/error is passed as confidence 0 -> PagerDuty, not a silent drop.
        assert_eq!(decide(Severity::Sev2, 0.0), Route::PagerDuty);
    }

    #[test]
    fn confidence_is_clamped() {
        assert_eq!(clamp_confidence(2.0), 1.0);
        assert_eq!(clamp_confidence(-0.5), 0.0);
        assert_eq!(clamp_confidence(f64::NAN), 0.0);
        assert_eq!(clamp_confidence(0.42), 0.42);
        // a skill bug returning > 1.0 still trusts -> CHAT for a non-sev1
        assert_eq!(decide(Severity::Sev2, 1.5), Route::Chat);
    }

    #[test]
    fn route_labels() {
        assert_eq!(Route::Chat.label(), "chat");
        assert_eq!(Route::PagerDuty.label(), "pagerduty");
        assert_eq!(Route::Both.label(), "both");
    }
}
