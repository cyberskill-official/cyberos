//! `wrap_in_marker` validator per TASK-SKILL-113 §3 (SKB-040..042).
//!
//! Serde + enum deserialization handles most of this — but we keep an
//! explicit validator for future-marker forward-compat (when TASK-SKILL-117
//! adds new MarkerName variants, this is where range-checks live).

use super::schema::{MarkerName, UntrustedInputs};

#[derive(Debug, PartialEq)]
pub enum MarkerViolation {
    UnregisteredMarker { value: String },
    MissingMarker,
}

pub fn validate(ui: &UntrustedInputs) -> Result<(), MarkerViolation> {
    match ui.wrap_in_marker {
        MarkerName::UntrustedContent => Ok(()),
        // Future markers add arms here. Until then, serde's enum
        // deserialization rejects unregistered values at parse time.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registered_marker_passes() {
        let ui = UntrustedInputs {
            wrap_in_marker: MarkerName::UntrustedContent,
            injection_scan: None,
            on_marker_hit: None,
        };
        assert!(validate(&ui).is_ok());
    }
}
