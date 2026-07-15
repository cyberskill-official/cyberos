//! In-memory `RoleMatrix` — the load-bearing performance contract.
//!
//! Per TASK-AUTH-101 §1 #9 + §1 #21 + DEC-126: role checks MUST complete in
//! < 50 µs at p99 against an in-memory snapshot. Per-request DB lookups are
//! forbidden. The matrix is loaded at boot and refreshed every 60 s by a
//! tokio task (refresher not yet wired — load-once-at-boot is the Wave 2
//! first slice).

use crate::rbac::{Action, Resource, Role};
use sqlx::PgPool;
use std::collections::HashSet;

/// A snapshot of `role_permissions` resolved into an O(1) set lookup.
#[derive(Debug, Clone)]
pub struct RoleMatrix {
    /// Permitted `(role, resource, action)` tuples. ~280 entries typical.
    grants: HashSet<(Role, Resource, Action)>,
    /// Catalogue version (TASK-AUTH-101 §1 #8 — `rbac_v` JWT claim).
    version: i32,
}

impl RoleMatrix {
    /// Empty matrix used during tests + initial boot before the first load.
    pub fn empty() -> Self {
        Self {
            grants: HashSet::new(),
            version: 0,
        }
    }

    /// Construct from an explicit list — used by tests and seeding.
    pub fn from_grants(grants: Vec<(Role, Resource, Action)>, version: i32) -> Self {
        Self {
            grants: grants.into_iter().collect(),
            version,
        }
    }

    /// O(1) permission check.
    pub fn has_permission(&self, role: Role, resource: Resource, action: Action) -> bool {
        self.grants.contains(&(role, resource, action))
    }

    /// Convenience for "does *any* of these roles grant the permission?"
    pub fn any_role_has_permission(
        &self,
        roles: impl IntoIterator<Item = Role>,
        resource: Resource,
        action: Action,
    ) -> bool {
        roles
            .into_iter()
            .any(|r| self.has_permission(r, resource, action))
    }

    pub fn version(&self) -> i32 {
        self.version
    }

    pub fn len(&self) -> usize {
        self.grants.len()
    }

    pub fn is_empty(&self) -> bool {
        self.grants.is_empty()
    }

    /// Load the matrix from Postgres. Joins `role_permissions` against the
    /// `role_catalogue_version` singleton to capture the live version.
    pub async fn load_from_db(pool: &PgPool) -> Result<Self, sqlx::Error> {
        let rows: Vec<(String, String, String)> =
            sqlx::query_as("SELECT role, resource, action FROM role_permissions")
                .fetch_all(pool)
                .await?;

        let version: i32 =
            sqlx::query_scalar("SELECT version FROM role_catalogue_version WHERE id = 1")
                .fetch_optional(pool)
                .await?
                .unwrap_or(1);

        let mut grants = HashSet::with_capacity(rows.len());
        for (r, res, a) in rows {
            // We silently drop tuples that don't parse — the migration writes
            // canonical names, so this only fires if someone hand-inserts.
            if let (Ok(role), Ok(resource), Ok(action)) = (
                r.parse::<Role>(),
                res.parse::<Resource>(),
                a.parse::<Action>(),
            ) {
                grants.insert((role, resource, action));
            }
        }

        Ok(Self { grants, version })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_matrix_grants_nothing() {
        let m = RoleMatrix::empty();
        assert!(!m.has_permission(Role::Cfo, Resource::InvInvoice, Action::Read));
    }

    #[test]
    fn explicit_grant_is_found() {
        let m = RoleMatrix::from_grants(vec![(Role::Cfo, Resource::InvInvoice, Action::Read)], 1);
        assert!(m.has_permission(Role::Cfo, Resource::InvInvoice, Action::Read));
        assert!(!m.has_permission(Role::Cfo, Resource::InvInvoice, Action::Write));
        assert!(!m.has_permission(Role::Cto, Resource::InvInvoice, Action::Read));
    }

    #[test]
    fn any_role_helper_short_circuits() {
        let m = RoleMatrix::from_grants(
            vec![(Role::TenantAdmin, Resource::Subject, Action::Admin)],
            1,
        );
        let roles = [Role::TenantMember, Role::TenantAdmin];
        assert!(m.any_role_has_permission(roles, Resource::Subject, Action::Admin));
    }
}
