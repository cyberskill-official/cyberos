//! FR-AUTH-004 §1 #13 — slice-1 audit-fix G-008.
//!
//! Maps a subject's coarse-grained role membership to the fine-grained
//! `scope_grants` claim in the issued JWT. Centralising the mapping here
//! ensures consistency: adding a new role automatically updates its grants
//! without per-gate code changes.
//!
//! ### Slice-1 mappings (per FR-AUTH-004 §1 #13)
//!
//! | Role           | Granted scopes                                       |
//! |----------------|------------------------------------------------------|
//! | `tenant-admin` | `chat:*`, `kb:*`, `proj:*`, `ai:read`, `ai:invoke`   |
//! | `tenant-member`| `chat:read`, `chat:write`, `kb:read`, `ai:invoke`    |
//! | `root-admin`   | `*` (root-tenant only; FR-AUTH-001 enforces tenancy)  |
//!
//! Unknown roles are silently skipped — `subjects.roles` is validated at
//! create time (FR-AUTH-002 §1 #?), so an unknown role here means either
//! (a) a typo that slipped past create-time validation (caught by the
//! `scope_map::tests::unknown_role_silently_skipped` test), or (b) a role
//! defined in FR-AUTH-101 RBAC that hasn't been added to this map yet
//! (caught at PR review when the role is rolled out).

use std::collections::BTreeSet;

/// Return the deduplicated, sorted list of scope grants for the given role
/// membership. BTreeSet for natural dedup + deterministic ordering (callers
/// like the JWT signer rely on stable claim ordering for byte-equality
/// tests).
pub fn for_roles(roles: &[String]) -> Vec<String> {
    let mut grants: BTreeSet<String> = BTreeSet::new();
    for role in roles {
        match role.as_str() {
            "tenant-admin" => {
                for g in ["chat:*", "kb:*", "proj:*", "ai:read", "ai:invoke"] {
                    grants.insert(g.to_string());
                }
            }
            "tenant-member" => {
                for g in ["chat:read", "chat:write", "kb:read", "ai:invoke"] {
                    grants.insert(g.to_string());
                }
            }
            "root-admin" => {
                grants.insert("*".to_string());
            }
            // Unknown role — silently skip. See module docs for rationale.
            _ => {}
        }
    }
    grants.into_iter().collect()
}

/// Intersect a caller-requested scope subset with the role-derived scope
/// set. Used by the password-grant handler: callers may pass `scope: ["chat:read"]`
/// in the token request to narrow the issued token; this function ensures
/// they NEVER widen beyond what their roles allow.
pub fn intersect(requested: &[String], roles: &[String]) -> Vec<String> {
    if requested.is_empty() {
        return for_roles(roles);
    }
    let allowed: BTreeSet<String> = for_roles(roles).into_iter().collect();
    requested
        .iter()
        .filter(|r| {
            // Direct match OR a glob entry like "chat:*" covers requested
            // "chat:read", etc. Allow "*" to cover everything.
            allowed.contains(*r)
                || allowed.contains("*")
                || allowed.iter().any(|a| {
                    if let Some(prefix) = a.strip_suffix(":*") {
                        r.starts_with(&format!("{prefix}:"))
                    } else {
                        false
                    }
                })
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn tenant_admin_gets_full_chat_kb_proj_set() {
        let grants = for_roles(&s(&["tenant-admin"]));
        assert!(grants.contains(&"chat:*".to_string()));
        assert!(grants.contains(&"kb:*".to_string()));
        assert!(grants.contains(&"proj:*".to_string()));
        assert!(grants.contains(&"ai:read".to_string()));
        assert!(grants.contains(&"ai:invoke".to_string()));
    }

    #[test]
    fn tenant_member_gets_reduced_set_no_admin_grants() {
        let grants = for_roles(&s(&["tenant-member"]));
        assert!(grants.contains(&"chat:read".to_string()));
        assert!(grants.contains(&"chat:write".to_string()));
        assert!(grants.contains(&"kb:read".to_string()));
        assert!(grants.contains(&"ai:invoke".to_string()));
        // Crucially, member does NOT get the admin globs.
        assert!(!grants.contains(&"chat:*".to_string()));
        assert!(!grants.contains(&"kb:*".to_string()));
        assert!(!grants.contains(&"proj:*".to_string()));
        assert!(!grants.contains(&"ai:read".to_string()));
    }

    #[test]
    fn root_admin_gets_universal_grant() {
        let grants = for_roles(&s(&["root-admin"]));
        assert_eq!(grants, vec!["*".to_string()]);
    }

    #[test]
    fn unknown_role_silently_skipped() {
        let grants = for_roles(&s(&["nonexistent-role-name"]));
        assert!(
            grants.is_empty(),
            "unknown role must contribute zero grants"
        );
    }

    #[test]
    fn multiple_roles_union_grants() {
        let grants = for_roles(&s(&["tenant-member", "tenant-admin"]));
        // Should be the admin superset.
        assert!(grants.contains(&"chat:*".to_string()));
        assert!(grants.contains(&"chat:read".to_string()));
    }

    #[test]
    fn intersect_narrows_to_requested() {
        // Tenant-admin requests only chat:read — should get chat:read
        // (because chat:* covers it).
        let got = intersect(&s(&["chat:read"]), &s(&["tenant-admin"]));
        assert_eq!(got, vec!["chat:read".to_string()]);
    }

    #[test]
    fn intersect_rejects_requests_beyond_role() {
        // tenant-member doesn't have proj:* — asking for proj:write should drop.
        let got = intersect(&s(&["chat:read", "proj:write"]), &s(&["tenant-member"]));
        assert_eq!(got, vec!["chat:read".to_string()]);
    }

    #[test]
    fn intersect_empty_requested_yields_full_role_grants() {
        let got = intersect(&[], &s(&["tenant-member"]));
        // Same as for_roles output.
        assert_eq!(got, for_roles(&s(&["tenant-member"])));
    }

    #[test]
    fn intersect_root_admin_accepts_anything() {
        let got = intersect(
            &s(&["anything:goes", "whatever:please"]),
            &s(&["root-admin"]),
        );
        assert_eq!(got.len(), 2);
    }
}
