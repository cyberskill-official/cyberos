//! `cyberos-types` — shared newtypes used across CyberOS services.
//!
//! AUTHORING_DISCIPLINE §3.1 rule 1: the root tenant is `Uuid::nil()` —
//! never numeric zero. `TenantId::ROOT` exposes that as a typed constant.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Tenant identifier. Always a UUID. The root tenant is `Uuid::nil()`
/// per AUTHORING_DISCIPLINE §3.1 rule 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TenantId(pub Uuid);

impl TenantId {
    /// The nil-UUID root tenant.
    pub const ROOT: TenantId = TenantId(Uuid::nil());

    /// Generate a fresh tenant id (UUIDv4).
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Is this the root tenant?
    pub fn is_root(self) -> bool {
        self.0.is_nil()
    }

    /// Underlying UUID accessor.
    pub fn as_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for TenantId {
    /// Default constructor — generates a fresh tenant id (UUIDv4).
    /// NOT the root tenant; use `TenantId::ROOT` for that.
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for TenantId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for TenantId {
    fn from(u: Uuid) -> Self {
        Self(u)
    }
}

/// Subject (user / agent / system principal) identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SubjectId(pub Uuid);

impl SubjectId {
    /// Generate a fresh subject id.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Underlying UUID accessor.
    pub fn as_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for SubjectId {
    /// Default constructor — generates a fresh subject id (UUIDv4).
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SubjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for SubjectId {
    fn from(u: Uuid) -> Self {
        Self(u)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_tenant_is_nil_uuid() {
        // AUTHORING_DISCIPLINE §3.1 rule 1 — root is nil UUID, NOT numeric zero.
        assert_eq!(TenantId::ROOT.as_uuid(), Uuid::nil());
        assert!(TenantId::ROOT.is_root());
    }

    #[test]
    fn fresh_tenant_is_not_root() {
        let fresh = TenantId::new();
        assert!(!fresh.is_root());
    }

    #[test]
    fn display_renders_uuid_hex() {
        let t = TenantId::ROOT;
        assert_eq!(t.to_string(), "00000000-0000-0000-0000-000000000000");
    }
}
