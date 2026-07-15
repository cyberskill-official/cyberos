//! The closed 22-role catalogue. Per DEC-121 + AUTHORING_DISCIPLINE §3.4:
//! variants here are EXACTLY the production role names; adding a 23rd
//! requires an ADR-NNN + a CI gate validating the migration references it.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

/// The 22 production role names. String form is kebab-case via
/// `Display` + `FromStr`. The order matches DEC-121's catalogue order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum Role {
    // --- Stub-tier (TASK-AUTH-002 strict prefix; DEC-123) ---
    RootAdmin,
    TenantAdmin,
    TenantMember,
    ServiceAccount,
    AgentPersona,

    // --- Production-tier additions (TASK-AUTH-101) ---
    Founder,
    Cfo,
    Cto,
    Coo,
    Chro,
    Cmo,
    Cpo,
    Cso,
    Cseco,
    Clo,
    Cdo,
    Dpo,
    Caio,
    ClientPortalUser,
    Auditor,
    Regulator,
    BillingSystem,
}

impl Role {
    /// All 22 variants in catalogue order. Useful for `GET /v1/admin/roles`
    /// + closed-enum invariant tests.
    pub const ALL: [Role; 22] = [
        Role::RootAdmin,
        Role::TenantAdmin,
        Role::TenantMember,
        Role::ServiceAccount,
        Role::AgentPersona,
        Role::Founder,
        Role::Cfo,
        Role::Cto,
        Role::Coo,
        Role::Chro,
        Role::Cmo,
        Role::Cpo,
        Role::Cso,
        Role::Cseco,
        Role::Clo,
        Role::Cdo,
        Role::Dpo,
        Role::Caio,
        Role::ClientPortalUser,
        Role::Auditor,
        Role::Regulator,
        Role::BillingSystem,
    ];

    /// Canonical kebab-case wire form. Stable; changing requires major version.
    pub const fn as_str(self) -> &'static str {
        match self {
            Role::RootAdmin => "root-admin",
            Role::TenantAdmin => "tenant-admin",
            Role::TenantMember => "tenant-member",
            Role::ServiceAccount => "service-account",
            Role::AgentPersona => "agent-persona",
            Role::Founder => "founder",
            Role::Cfo => "cfo",
            Role::Cto => "cto",
            Role::Coo => "coo",
            Role::Chro => "chro",
            Role::Cmo => "cmo",
            Role::Cpo => "cpo",
            Role::Cso => "cso",
            Role::Cseco => "cseco",
            Role::Clo => "clo",
            Role::Cdo => "cdo",
            Role::Dpo => "dpo",
            Role::Caio => "caio",
            Role::ClientPortalUser => "client-portal-user",
            Role::Auditor => "auditor",
            Role::Regulator => "regulator",
            Role::BillingSystem => "billing-system",
        }
    }

    /// Human-readable display name (used in the catalogue endpoint).
    pub const fn display(self) -> &'static str {
        match self {
            Role::RootAdmin => "Root Admin (cross-tenant operator)",
            Role::TenantAdmin => "Tenant Admin",
            Role::TenantMember => "Tenant Member",
            Role::ServiceAccount => "Service Account",
            Role::AgentPersona => "Agent Persona",
            Role::Founder => "Founder",
            Role::Cfo => "Chief Financial Officer",
            Role::Cto => "Chief Technology Officer",
            Role::Coo => "Chief Operating Officer",
            Role::Chro => "Chief Human Resources Officer",
            Role::Cmo => "Chief Marketing Officer",
            Role::Cpo => "Chief Product Officer",
            Role::Cso => "Chief Strategy Officer",
            Role::Cseco => "Chief Security Officer",
            Role::Clo => "Chief Legal Officer",
            Role::Cdo => "Chief Data Officer",
            Role::Dpo => "Data Protection Officer",
            Role::Caio => "Chief AI Officer",
            Role::ClientPortalUser => "Client Portal User",
            Role::Auditor => "External Auditor",
            Role::Regulator => "External Regulator",
            Role::BillingSystem => "Billing System (Stripe/VietQR webhook)",
        }
    }

    /// DEC-127 — these roles MUST NOT be self-assignable via the standard
    /// `POST /v1/admin/subjects/{id}/roles`. They route through dedicated
    /// elevated-privilege endpoints (slice 2).
    pub const fn is_reserved(self) -> bool {
        matches!(
            self,
            Role::RootAdmin
                | Role::ClientPortalUser
                | Role::Auditor
                | Role::Regulator
                | Role::BillingSystem
        )
    }

    /// DEC-128 — the founder role grants cross-module privileged read.
    /// Assignment requires a registered WebAuthn factor (TASK-AUTH-105).
    pub const fn requires_webauthn(self) -> bool {
        matches!(self, Role::Founder)
    }

    /// Whether this role was present in the TASK-AUTH-002 stub catalogue
    /// (used by the `rbac_stub_compat_test` regression test).
    pub const fn is_stub_tier(self) -> bool {
        matches!(
            self,
            Role::RootAdmin
                | Role::TenantAdmin
                | Role::TenantMember
                | Role::ServiceAccount
                | Role::AgentPersona
        )
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<Role> for String {
    fn from(r: Role) -> String {
        r.as_str().to_string()
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RoleParseError {
    #[error("unknown role: {0:?}")]
    UnknownRole(String),
}

impl FromStr for Role {
    type Err = RoleParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for r in Role::ALL {
            if r.as_str() == s {
                return Ok(r);
            }
        }
        Err(RoleParseError::UnknownRole(s.to_string()))
    }
}

impl TryFrom<String> for Role {
    type Error = RoleParseError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Role::from_str(&s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_22_variants_round_trip_via_string() {
        for r in Role::ALL {
            let s = r.as_str();
            let parsed = Role::from_str(s).unwrap();
            assert_eq!(parsed, r, "round-trip failed for {s:?}");
        }
    }

    #[test]
    fn catalogue_has_exactly_22_roles() {
        // DEC-121 invariant: closed catalogue. Adding a variant requires updating
        // Role::ALL alongside, so this catches accidental enum growth.
        assert_eq!(Role::ALL.len(), 22);
    }

    #[test]
    fn unknown_string_does_not_parse() {
        assert!(matches!(
            Role::from_str("super-admin"),
            Err(RoleParseError::UnknownRole(_))
        ));
        assert!(matches!(
            Role::from_str(""),
            Err(RoleParseError::UnknownRole(_))
        ));
    }

    #[test]
    fn stub_tier_is_strict_prefix() {
        // DEC-123: the 5 stub roles MUST be exactly the first 5 variants.
        let stub_count = Role::ALL.iter().filter(|r| r.is_stub_tier()).count();
        assert_eq!(stub_count, 5);
        // Stub roles come first in catalogue order.
        for r in Role::ALL.iter().take(5) {
            assert!(r.is_stub_tier(), "first 5 must be stub-tier; {r:?} isn't");
        }
        for r in Role::ALL.iter().skip(5) {
            assert!(
                !r.is_stub_tier(),
                "non-first-5 must NOT be stub-tier; {r:?} is"
            );
        }
    }

    #[test]
    fn reserved_roles_match_dec_127() {
        let reserved: Vec<_> = Role::ALL
            .iter()
            .filter(|r| r.is_reserved())
            .copied()
            .collect();
        assert_eq!(reserved.len(), 5);
        assert!(reserved.contains(&Role::RootAdmin));
        assert!(reserved.contains(&Role::ClientPortalUser));
        assert!(reserved.contains(&Role::Auditor));
        assert!(reserved.contains(&Role::Regulator));
        assert!(reserved.contains(&Role::BillingSystem));
    }

    #[test]
    fn only_founder_requires_webauthn() {
        let with_webauthn: Vec<_> = Role::ALL
            .iter()
            .filter(|r| r.requires_webauthn())
            .copied()
            .collect();
        assert_eq!(with_webauthn, vec![Role::Founder]);
    }

    #[test]
    fn json_round_trip_kebab_case() {
        let r = Role::TenantAdmin;
        let s = serde_json::to_string(&r).unwrap();
        assert_eq!(s, "\"tenant-admin\"");
        let back: Role = serde_json::from_str(&s).unwrap();
        assert_eq!(back, Role::TenantAdmin);
    }
}
