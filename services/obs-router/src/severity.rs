//! Alert severity (TASK-OBS-007). Parsed from the Alertmanager `severity` label. Sev-1 is special: it
//! always pages both CHAT and PagerDuty, never trusting triage (§1 #5, DEC-171).

/// Alert severity, sev-1 (most severe) through sev-4 (least). Ordered so `Sev1 < Sev2 < ... < Sev4`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Sev1,
    Sev2,
    Sev3,
    Sev4,
}

impl Severity {
    /// Parse the Alertmanager `severity` label. Recognises numeric and `sev`/`p` forms (`"1"`, `"sev1"`,
    /// `"sev-1"`, `"p1"`) and common words (`critical`/`fatal`/`crit` -> sev1, `error`/`high` -> sev2,
    /// `warning`/`warn`/`medium` -> sev3, `info`/`low` -> sev4). An unknown or empty label maps to sev2,
    /// a cautious default - a mystery alert is treated as a real problem, not as noise.
    pub fn parse(label: &str) -> Self {
        match label.trim().to_ascii_lowercase().as_str() {
            "1" | "sev1" | "sev-1" | "p1" | "critical" | "crit" | "fatal" => Severity::Sev1,
            "2" | "sev2" | "sev-2" | "p2" | "error" | "high" => Severity::Sev2,
            "3" | "sev3" | "sev-3" | "p3" | "warning" | "warn" | "medium" => Severity::Sev3,
            "4" | "sev4" | "sev-4" | "p4" | "info" | "low" => Severity::Sev4,
            _ => Severity::Sev2,
        }
    }

    /// Sev-1 never trusts triage; it always pages both channels (§1 #5, DEC-171).
    pub fn is_sev1(self) -> bool {
        matches!(self, Severity::Sev1)
    }

    /// The metric / audit label form (`"sev1"` .. `"sev4"`).
    pub fn label(self) -> &'static str {
        match self {
            Severity::Sev1 => "sev1",
            Severity::Sev2 => "sev2",
            Severity::Sev3 => "sev3",
            Severity::Sev4 => "sev4",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_numeric_sev_and_p_forms() {
        for s in ["1", "sev1", "sev-1", "P1", " sev1 "] {
            assert_eq!(Severity::parse(s), Severity::Sev1, "{s}");
        }
        assert_eq!(Severity::parse("2"), Severity::Sev2);
        assert_eq!(Severity::parse("sev-3"), Severity::Sev3);
        assert_eq!(Severity::parse("p4"), Severity::Sev4);
    }

    #[test]
    fn parses_word_forms_case_insensitively() {
        assert_eq!(Severity::parse("Critical"), Severity::Sev1);
        assert_eq!(Severity::parse("FATAL"), Severity::Sev1);
        assert_eq!(Severity::parse("error"), Severity::Sev2);
        assert_eq!(Severity::parse("Warning"), Severity::Sev3);
        assert_eq!(Severity::parse("info"), Severity::Sev4);
    }

    #[test]
    fn unknown_label_defaults_to_sev2_not_silently_lowest() {
        assert_eq!(Severity::parse("banana"), Severity::Sev2);
        assert_eq!(Severity::parse(""), Severity::Sev2);
    }

    #[test]
    fn only_sev1_is_sev1_and_ordering_holds() {
        assert!(Severity::Sev1.is_sev1());
        assert!(!Severity::Sev2.is_sev1());
        assert!(Severity::Sev1 < Severity::Sev2);
        assert!(Severity::Sev2 < Severity::Sev4);
    }
}
