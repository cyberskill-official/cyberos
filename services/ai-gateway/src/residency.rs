//! FR-AI-016 — Geographic residency matching.
//!
//! Enforces tenant residency at alias-resolution time. The `region_table.rs` mapping
//! is the single source of truth for residency → acceptable-region sets.
//!
//! See FR-AI-016 for normative behaviour and acceptance criteria.

use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use once_cell::sync::Lazy;
use prometheus::{register_counter_vec, CounterVec};
use regex::Regex;

use crate::policy::Residency;

// ─── Constants ────────────────────────────────────────────────────────────────

/// §1 #2: residency → acceptable-region mapping. Changes require FR amendment.
static REGIONS_BY_RESIDENCY: LazyLock<HashMap<Residency, HashSet<&'static str>>> =
    LazyLock::new(|| {
        let mut m = HashMap::new();
        m.insert(Residency::Sg1, {
            let mut s = HashSet::new();
            s.insert("ap-southeast-1"); // Singapore
            s
        });
        m.insert(Residency::Eu1, {
            let mut s = HashSet::new();
            s.insert("eu-central-1"); // Frankfurt
            s.insert("eu-west-1"); // Ireland
            s
        });
        m.insert(Residency::Us1, {
            let mut s = HashSet::new();
            s.insert("us-east-1"); // N. Virginia
            s.insert("us-east-2"); // Ohio
            s.insert("us-west-2"); // Oregon
            s
        });
        m.insert(Residency::Vn1, HashSet::new()); // §1 #6: empty until FR-AI-104
        m
    });

/// §1 #5: AZ-suffix strip regex. Captures the region portion before any trailing AZ letter.
static AZ_STRIP_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?P<region>[a-z]{2}-[a-z]+-\d+)[a-z]?$").unwrap());

// ─── Metrics (FR-AI-016 §1 #13) ──────────────────────────────────────────────

static RESIDENCY_MISMATCHES: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "ai_residency_mismatches_total",
        "Residency mismatches by policy residency and resolved region",
        &["policy_residency", "resolved_region"]
    )
    .unwrap()
});

static VN1_REFUSED: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "ai_residency_vn1_refused_total",
        "Vn1 residency refusals by tenant",
        &["tenant_id"]
    )
    .unwrap()
});

static RESIDENCY_OVERRIDES_USED: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "ai_residency_overrides_used_total",
        "Per-alias residency overrides used",
        &["tenant_id", "alias"]
    )
    .unwrap()
});

static RESIDENCY_DEFAULT_APPLIED: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "ai_residency_default_applied_total",
        "Missing residency default/refusal outcomes",
        &["outcome"]
    )
    .unwrap()
});

// ─── Public types ─────────────────────────────────────────────────────────────

/// Newtype wrapping a validated AWS region string (no AZ suffix).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Region(String);

impl Region {
    /// §1 #5: Strip AZ suffix from provider-returned region string.
    ///
    /// `"ap-southeast-1a"` → `Region("ap-southeast-1")`.
    /// Returns `Err` for strings that don't match the AWS region format.
    pub fn from_provider_string(raw: &str) -> Result<Self, RegionParseError> {
        let caps = AZ_STRIP_RE
            .captures(raw)
            .ok_or_else(|| RegionParseError::Invalid(raw.into()))?;
        Ok(Region(caps.name("region").unwrap().as_str().to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RegionParseError {
    #[error("invalid region string {0:?}; expected AWS region format")]
    Invalid(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ResidencyParseError {
    #[error("invalid residency value {0:?}; expected sg-1 | eu-1 | us-1 | vn-1")]
    Invalid(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedResidencyOverride {
    pub pattern: String,
    pub residency: Residency,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum OverrideError {
    #[error("ambiguous residency override for alias {alias:?}: {patterns:?}")]
    OverrideAmbiguous {
        alias: String,
        patterns: Vec<String>,
    },
    #[error("invalid residency override pattern {pattern:?}: {reason}")]
    InvalidPattern { pattern: String, reason: String },
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// §1 #1: Check if a provider region satisfies the policy's residency pin.
///
/// Returns `true` if and only if the provider's region is in the residency's
/// acceptable-region set per the `REGIONS_BY_RESIDENCY` mapping.
pub fn matches(policy_residency: Residency, provider_region: &Region) -> bool {
    REGIONS_BY_RESIDENCY
        .get(&policy_residency)
        .map(|set| set.contains(provider_region.as_str()))
        .unwrap_or(false)
}

/// Parse a residency string from YAML wire form.
pub fn parse_residency(s: &str) -> Result<Residency, ResidencyParseError> {
    match s {
        "sg-1" => Ok(Residency::Sg1),
        "eu-1" => Ok(Residency::Eu1),
        "us-1" => Ok(Residency::Us1),
        "vn-1" => Ok(Residency::Vn1),
        _ => Err(ResidencyParseError::Invalid(s.to_string())),
    }
}

/// Resolve a per-alias residency override for an alias.
///
/// Patterns are deterministic, shell-style subsets: exact aliases or `*` wildcards.
/// Multiple matching patterns are rejected because the policy would be ambiguous.
pub fn resolve_override(
    overrides: &HashMap<String, Residency>,
    alias: &str,
) -> Result<Option<ResolvedResidencyOverride>, OverrideError> {
    let mut matches = Vec::new();
    for (pattern, residency) in overrides {
        validate_override_pattern(pattern)?;
        if alias_glob_matches(pattern, alias) {
            matches.push(ResolvedResidencyOverride {
                pattern: pattern.clone(),
                residency: *residency,
            });
        }
    }

    matches.sort_by(|a, b| a.pattern.cmp(&b.pattern));

    match matches.len() {
        0 => Ok(None),
        1 => Ok(matches.pop()),
        _ => Err(OverrideError::OverrideAmbiguous {
            alias: alias.to_string(),
            patterns: matches.into_iter().map(|m| m.pattern).collect(),
        }),
    }
}

/// Validate the supported alias-glob subset for policy loading.
pub fn validate_override_pattern(pattern: &str) -> Result<(), OverrideError> {
    if pattern.is_empty() {
        return Err(OverrideError::InvalidPattern {
            pattern: pattern.to_string(),
            reason: "empty".to_string(),
        });
    }
    if !pattern
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '*'))
    {
        return Err(OverrideError::InvalidPattern {
            pattern: pattern.to_string(),
            reason: "allowed characters are [A-Za-z0-9._-*]".to_string(),
        });
    }
    Ok(())
}

/// Record a residency mismatch metric.
pub fn record_mismatch(policy_residency: Residency, resolved_region: &Region) {
    RESIDENCY_MISMATCHES
        .with_label_values(&[residency_label(policy_residency), resolved_region.as_str()])
        .inc();
}

/// Record a Vn1 refusal metric.
pub fn record_vn1_refused(tenant_id: &str) {
    VN1_REFUSED.with_label_values(&[tenant_id]).inc();
}

/// Record a per-alias override usage metric.
pub fn record_override_used(tenant_id: &str, alias: &str) {
    RESIDENCY_OVERRIDES_USED
        .with_label_values(&[tenant_id, alias])
        .inc();
}

/// Stable wire/metric label for residency values.
pub fn residency_label(r: Residency) -> &'static str {
    match r {
        Residency::Sg1 => "sg-1",
        Residency::Eu1 => "eu-1",
        Residency::Us1 => "us-1",
        Residency::Vn1 => "vn-1",
    }
}

/// Record missing-residency default/refusal outcomes.
pub fn record_default_applied(outcome: &str) {
    RESIDENCY_DEFAULT_APPLIED
        .with_label_values(&[outcome])
        .inc();
}

fn alias_glob_matches(pattern: &str, alias: &str) -> bool {
    if !pattern.contains('*') {
        return pattern == alias;
    }

    let parts: Vec<&str> = pattern.split('*').collect();
    let mut pos = 0usize;

    for (idx, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if idx == 0 && !pattern.starts_with('*') {
            let remaining = &alias[pos..];
            if !remaining.starts_with(part) {
                return false;
            }
            pos += part.len();
            continue;
        }

        let remaining = &alias[pos..];
        let Some(found) = remaining.find(part) else {
            return false;
        };
        pos += found + part.len();
    }

    if !pattern.ends_with('*') {
        let Some(last) = parts.iter().rev().find(|part| !part.is_empty()) else {
            return true;
        };
        return alias.ends_with(last);
    }

    true
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // AC #1: Sg1 → ap-southeast-1 matches
    #[test]
    fn sg1_accepts_ap_southeast_1() {
        let region = Region::from_provider_string("ap-southeast-1").unwrap();
        assert!(matches(Residency::Sg1, &region));
    }

    // AC #2: Sg1 → us-east-1 mismatches
    #[test]
    fn sg1_rejects_us_east_1() {
        let region = Region::from_provider_string("us-east-1").unwrap();
        assert!(!matches(Residency::Sg1, &region));
    }

    // AC #3: Eu1 → eu-central-1 AND eu-west-1 match
    #[test]
    fn eu1_accepts_central_and_west() {
        for r in &["eu-central-1", "eu-west-1"] {
            let region = Region::from_provider_string(r).unwrap();
            assert!(matches(Residency::Eu1, &region), "Eu1 should accept {}", r);
        }
    }

    // AC #4: Us1 → all three US regions match
    #[test]
    fn us1_accepts_all_three_us_regions() {
        for r in &["us-east-1", "us-east-2", "us-west-2"] {
            let region = Region::from_provider_string(r).unwrap();
            assert!(matches(Residency::Us1, &region), "Us1 should accept {}", r);
        }
    }

    // AC #5: Vn1 → empty set always returns false
    #[test]
    fn vn1_empty_set_always_returns_false() {
        for r in &["ap-southeast-1", "us-east-1", "eu-central-1", "eu-west-1"] {
            let region = Region::from_provider_string(r).unwrap();
            assert!(!matches(Residency::Vn1, &region), "Vn1 should reject {}", r);
        }
    }

    // AC #6: AZ-suffix stripped
    #[test]
    fn az_suffix_stripped() {
        let region = Region::from_provider_string("ap-southeast-1a").unwrap();
        assert_eq!(region.as_str(), "ap-southeast-1");
        assert!(matches(Residency::Sg1, &region));
    }

    #[test]
    fn az_suffix_stripped_eu() {
        let region = Region::from_provider_string("eu-central-1b").unwrap();
        assert_eq!(region.as_str(), "eu-central-1");
        assert!(matches(Residency::Eu1, &region));
    }

    #[test]
    fn region_without_az_suffix_works() {
        let region = Region::from_provider_string("us-west-2").unwrap();
        assert_eq!(region.as_str(), "us-west-2");
        assert!(matches(Residency::Us1, &region));
    }

    // AC #7: Invalid region string rejected
    #[test]
    fn invalid_region_string_rejected() {
        assert!(matches!(
            Region::from_provider_string("not-a-region"),
            Err(RegionParseError::Invalid(_))
        ));
    }

    #[test]
    fn empty_region_rejected() {
        assert!(matches!(
            Region::from_provider_string(""),
            Err(RegionParseError::Invalid(_))
        ));
    }

    // AC #8: parse_residency
    #[test]
    fn parse_residency_valid_values() {
        assert_eq!(parse_residency("sg-1").unwrap(), Residency::Sg1);
        assert_eq!(parse_residency("eu-1").unwrap(), Residency::Eu1);
        assert_eq!(parse_residency("us-1").unwrap(), Residency::Us1);
        assert_eq!(parse_residency("vn-1").unwrap(), Residency::Vn1);
    }

    #[test]
    fn parse_residency_invalid_value() {
        assert!(matches!(
            parse_residency("apac-2"),
            Err(ResidencyParseError::Invalid(_))
        ));
    }

    // Additional: cross-residency checks
    #[test]
    fn eu1_rejects_us_regions() {
        for r in &["us-east-1", "us-west-2"] {
            let region = Region::from_provider_string(r).unwrap();
            assert!(!matches(Residency::Eu1, &region), "Eu1 should reject {}", r);
        }
    }

    #[test]
    fn us1_rejects_eu_regions() {
        for r in &["eu-central-1", "eu-west-1"] {
            let region = Region::from_provider_string(r).unwrap();
            assert!(!matches(Residency::Us1, &region), "Us1 should reject {}", r);
        }
    }

    #[test]
    fn us1_rejects_ap_regions() {
        let region = Region::from_provider_string("ap-southeast-1").unwrap();
        assert!(!matches(Residency::Us1, &region));
    }

    // Deterministic: same pair → same result
    #[test]
    fn deterministic() {
        let region = Region::from_provider_string("ap-southeast-1").unwrap();
        let r1 = matches(Residency::Sg1, &region);
        let r2 = matches(Residency::Sg1, &region);
        assert_eq!(r1, r2);
    }

    #[test]
    fn exact_residency_override_matches_alias() {
        let overrides = HashMap::from([("chat.smart".to_string(), Residency::Eu1)]);
        let resolved = resolve_override(&overrides, "chat.smart").unwrap().unwrap();
        assert_eq!(resolved.pattern, "chat.smart");
        assert_eq!(resolved.residency, Residency::Eu1);
        assert!(resolve_override(&overrides, "chat.fast").unwrap().is_none());
    }

    #[test]
    fn wildcard_residency_override_matches_alias() {
        let overrides = HashMap::from([("chat.eu-*".to_string(), Residency::Eu1)]);
        let resolved = resolve_override(&overrides, "chat.eu-customer-data")
            .unwrap()
            .unwrap();
        assert_eq!(resolved.pattern, "chat.eu-*");
        assert_eq!(resolved.residency, Residency::Eu1);
    }

    #[test]
    fn ambiguous_residency_override_is_rejected() {
        let overrides = HashMap::from([
            ("chat.*".to_string(), Residency::Eu1),
            ("chat.eu-*".to_string(), Residency::Sg1),
        ]);
        let err = resolve_override(&overrides, "chat.eu-customer-data").unwrap_err();
        assert!(matches!(err, OverrideError::OverrideAmbiguous { .. }));
    }

    #[test]
    fn invalid_residency_override_pattern_is_rejected() {
        let overrides = HashMap::from([("chat.?".to_string(), Residency::Eu1)]);
        let err = resolve_override(&overrides, "chat.smart").unwrap_err();
        assert!(matches!(err, OverrideError::InvalidPattern { .. }));
    }
}
