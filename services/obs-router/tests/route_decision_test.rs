//! FR-OBS-007 §1 #3 - the full (severity, confidence) routing table, as a held-out integration test,
//! plus the §1 #11 no-silent-drop invariant over a grid of inputs.

use cyberos_obs_router::{decide, Route, Severity, CONFIDENCE_FLOOR};

#[test]
fn routing_table_matches_spec_section_1_3() {
    // sev-1: Both, regardless of confidence (DEC-171).
    assert_eq!(decide(Severity::Sev1, 0.0), Route::Both);
    assert_eq!(decide(Severity::Sev1, 1.0), Route::Both);

    // sev-2..4 with confidence at or above the floor: CHAT.
    for sev in [Severity::Sev2, Severity::Sev3, Severity::Sev4] {
        assert_eq!(decide(sev, CONFIDENCE_FLOOR), Route::Chat);
        assert_eq!(decide(sev, 1.0), Route::Chat);
    }
    // sev-2..4 below the floor: PagerDuty.
    for sev in [Severity::Sev2, Severity::Sev3, Severity::Sev4] {
        assert_eq!(decide(sev, CONFIDENCE_FLOOR - 0.01), Route::PagerDuty);
        assert_eq!(decide(sev, 0.0), Route::PagerDuty);
    }
}

#[test]
fn every_alert_routes_somewhere_no_silent_drop() {
    // §1 #11 - exhaustive over a grid of severities x confidences: the decision is always a real route.
    for sev in [Severity::Sev1, Severity::Sev2, Severity::Sev3, Severity::Sev4] {
        for c in [f64::NAN, -1.0, 0.0, 0.5, 0.7, 1.0, 2.0] {
            let r = decide(sev, c);
            assert!(matches!(r, Route::Chat | Route::PagerDuty | Route::Both), "{sev:?} {c}");
        }
    }
}
