//! The four compliance views (FR-OBS-008 §1 #1, #11). Each view selects a fixed set of audit row kinds
//! from the memory chain - that mapping is the auditable contract, so it is a pure table and is tested.

/// A compliance view, one per regulation (slice-3 scope, DEC-175).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    EuAiAct,
    Pdpl,
    Soc2,
    Iso27001,
}

impl View {
    /// Every view, for enumeration in tests and routing.
    pub const ALL: [View; 4] = [View::EuAiAct, View::Pdpl, View::Soc2, View::Iso27001];

    /// Parse the URL path slug (`/eu-ai-act/`, `/pdpl/`, `/soc2/`, `/iso27001/`).
    pub fn parse(path: &str) -> Option<View> {
        match path.trim_matches('/') {
            "eu-ai-act" => Some(View::EuAiAct),
            "pdpl" => Some(View::Pdpl),
            "soc2" => Some(View::Soc2),
            "iso27001" => Some(View::Iso27001),
            _ => None,
        }
    }

    /// The URL / metric slug.
    pub fn slug(self) -> &'static str {
        match self {
            View::EuAiAct => "eu-ai-act",
            View::Pdpl => "pdpl",
            View::Soc2 => "soc2",
            View::Iso27001 => "iso27001",
        }
    }

    /// The audit row kinds this view selects from the memory chain (§1 #11, slice-3 scope). The view is
    /// read-only and tenant-scoped; this table is only the kind filter.
    pub fn kinds(self) -> &'static [&'static str] {
        match self {
            View::EuAiAct => &[
                "ai.invocation",
                "ai.persona_loaded",
                "ai.zdr_violation",
                "ai.residency_violation",
            ],
            View::Pdpl => &[
                "memory.delete_data",
                "memory.export_data",
                "ai.residency_violation",
                "obs.langsmith_export",
            ],
            View::Soc2 => &[
                "auth.token_issued",
                "auth.token_failed",
                "ai.cli_policy_updated",
                "ai.cli_breaker_reset",
                "obs.alert_triaged",
                "obs.alert_acked",
            ],
            View::Iso27001 => &[
                "auth.subject_created",
                "auth.subject_revoked",
                "auth.role_assigned",
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_roundtrips_slug_for_every_view() {
        for v in View::ALL {
            assert_eq!(View::parse(v.slug()), Some(v));
            assert_eq!(View::parse(&format!("/{}/", v.slug())), Some(v));
        }
        assert_eq!(View::parse("pci-dss"), None); // slice-5+ view, not yet served
        assert_eq!(View::parse(""), None);
    }

    #[test]
    fn every_view_selects_a_non_empty_kind_set() {
        for v in View::ALL {
            assert!(!v.kinds().is_empty(), "{v:?} must select at least one kind");
        }
    }

    #[test]
    fn views_select_their_headline_kinds() {
        assert!(View::EuAiAct.kinds().contains(&"ai.invocation"));
        assert!(View::Soc2.kinds().contains(&"obs.alert_triaged"));
        assert!(View::Pdpl.kinds().contains(&"memory.delete_data"));
    }
}
