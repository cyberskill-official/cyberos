//! TASK-AUTH-101 catalogue invariants — pure-Rust tests (no Postgres).
//!
//! These run in CI's lint-and-test job. The DB-backed tests for the
//! matrix loader + endpoints live in `rbac_endpoint_test.rs` (Postgres-required).

use cyberos_auth::rbac::{Action, Resource, Role};
use std::str::FromStr;

#[test]
fn role_catalogue_count_is_locked_at_22() {
    assert_eq!(
        Role::ALL.len(),
        22,
        "DEC-121 invariant — adding a role requires an ADR"
    );
}

#[test]
fn resource_catalogue_count_is_locked_at_40() {
    assert_eq!(Resource::ALL.len(), 40, "DEC-122 invariant");
}

#[test]
fn action_catalogue_count_is_locked_at_5() {
    assert_eq!(Action::ALL.len(), 5, "DEC-122 invariant");
}

#[test]
fn role_strings_are_kebab_case_no_caps_no_underscores() {
    for r in Role::ALL {
        let s = r.as_str();
        assert!(
            s.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
            "role name must be kebab-case ASCII: {s:?}"
        );
        assert!(!s.is_empty());
        assert!(!s.starts_with('-') && !s.ends_with('-'));
    }
}

#[test]
fn resource_strings_are_kebab_case_no_caps_no_underscores() {
    for r in Resource::ALL {
        let s = r.as_str();
        assert!(
            s.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
            "resource name must be kebab-case ASCII: {s:?}"
        );
    }
}

#[test]
fn no_duplicate_role_names() {
    let mut seen = std::collections::HashSet::new();
    for r in Role::ALL {
        assert!(
            seen.insert(r.as_str()),
            "duplicate role string {:?}",
            r.as_str()
        );
    }
}

#[test]
fn stub_tier_is_strict_prefix_of_catalogue() {
    // DEC-123 — the 5 stub roles MUST be the first 5 variants of ALL.
    for r in Role::ALL.iter().take(5) {
        assert!(r.is_stub_tier(), "expected {r:?} in stub tier");
    }
    for r in Role::ALL.iter().skip(5) {
        assert!(!r.is_stub_tier(), "{r:?} must NOT be stub tier");
    }
}

#[test]
fn reserved_roles_are_exactly_dec_127_list() {
    let reserved: Vec<_> = Role::ALL
        .iter()
        .filter(|r| r.is_reserved())
        .copied()
        .collect();
    let expected = [
        Role::RootAdmin,
        Role::ClientPortalUser,
        Role::Auditor,
        Role::Regulator,
        Role::BillingSystem,
    ];
    assert_eq!(reserved.len(), expected.len());
    for r in expected {
        assert!(reserved.contains(&r), "expected reserved: {r:?}");
    }
}

#[test]
fn only_founder_requires_webauthn() {
    let webauthn: Vec<_> = Role::ALL
        .iter()
        .filter(|r| r.requires_webauthn())
        .copied()
        .collect();
    assert_eq!(webauthn, vec![Role::Founder]);
}

#[test]
fn unknown_role_string_fails_to_parse() {
    assert!(Role::from_str("super-admin").is_err());
    assert!(Role::from_str("ROOT-ADMIN").is_err()); // case-sensitive
    assert!(Role::from_str("").is_err());
    assert!(Role::from_str(" tenant-admin ").is_err()); // no leading/trailing whitespace
}
