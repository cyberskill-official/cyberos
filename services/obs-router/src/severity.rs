//! Severity parsing and route decision rules.

use serde::{Deserialize, Serialize};

/// Alert severity used by FR-OBS-007 routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Severity {
    /// Sev-1/P1: customer-facing outage or security incident.
    Sev1,
    /// Sev-2/P2.
    Sev2,
    /// Sev-3/P3.
    Sev3,
    /// Sev-4/P4 or unknown.
    Sev4,
}

impl Severity {
    /// Stable label used in metrics and audit rows.
    pub fn as_label(self) -> &'static str {
        match self {
            Self::Sev1 => "sev-1",
            Self::Sev2 => "sev-2",
            Self::Sev3 => "sev-3",
            Self::Sev4 => "sev-4",
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_label())
    }
}

/// Routing target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Route {
    /// Post to CHAT only.
    Chat,
    /// Trigger PagerDuty only.
    PagerDuty,
    /// Route to CHAT and PagerDuty.
    Both,
}

impl Route {
    /// Stable label used in metrics and audit rows.
    pub fn as_label(self) -> &'static str {
        match self {
            Self::Chat => "chat",
            Self::PagerDuty => "pagerduty",
            Self::Both => "both",
        }
    }
}

impl std::fmt::Display for Route {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_label())
    }
}

/// Parse common Alertmanager severity labels into canonical severities.
pub fn parse_severity(label: &str) -> Severity {
    match label.trim().to_ascii_uppercase().as_str() {
        "P1" | "SEV1" | "SEV-1" | "SEV_1" | "CRITICAL" | "CRIT" => Severity::Sev1,
        "P2" | "SEV2" | "SEV-2" | "SEV_2" | "ERROR" => Severity::Sev2,
        "P3" | "SEV3" | "SEV-3" | "SEV_3" | "WARNING" | "WARN" => Severity::Sev3,
        _ => Severity::Sev4,
    }
}

/// Decide the FR-OBS-007 route.
pub fn decide_route(severity: Severity, confidence: f64) -> Route {
    if severity == Severity::Sev1 {
        Route::Both
    } else if confidence >= 0.70 {
        Route::Chat
    } else {
        Route::PagerDuty
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_common_labels() {
        assert_eq!(parse_severity("P1"), Severity::Sev1);
        assert_eq!(parse_severity("critical"), Severity::Sev1);
        assert_eq!(parse_severity("error"), Severity::Sev2);
        assert_eq!(parse_severity("warn"), Severity::Sev3);
        assert_eq!(parse_severity("info"), Severity::Sev4);
    }

    #[test]
    fn route_matrix_matches_fr() {
        assert_eq!(decide_route(Severity::Sev1, 0.99), Route::Both);
        assert_eq!(decide_route(Severity::Sev2, 0.70), Route::Chat);
        assert_eq!(decide_route(Severity::Sev2, 0.69), Route::PagerDuty);
    }
}
