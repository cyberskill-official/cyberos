//! TASK-MCP-003 — DEC-2364 naming audit kind surface is present and wired.
//!
//! The emit helpers need a live Postgres pool; this test pins the kind *names* and the public
//! function surface so a rename cannot silently drop the four DEC-2364 kinds. Router register
//! success/reject paths call `skill_name_validated` / `skill_name_rejected` (see router.rs).

use std::fs;

#[test]
fn four_dec2364_audit_kinds_are_named_in_oauth_audit() {
    let src = fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/oauth/audit.rs"),
    )
    .expect("read oauth/audit.rs");
    for kind in [
        "mcp.skill_name_validated",
        "mcp.skill_name_rejected",
        "mcp.naming_ci_check_passed",
        "mcp.naming_ci_check_failed",
    ] {
        assert!(
            src.contains(&format!("\"{kind}\"")),
            "oauth/audit.rs must emit kind {kind}"
        );
    }
    for fn_name in [
        "pub async fn skill_name_validated",
        "pub async fn skill_name_rejected",
        "pub async fn naming_ci_check_passed",
        "pub async fn naming_ci_check_failed",
    ] {
        assert!(
            src.contains(fn_name),
            "oauth/audit.rs must expose {fn_name}"
        );
    }
}

#[test]
fn register_router_emits_validated_and_rejected() {
    let src = fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/router.rs"),
    )
    .expect("read router.rs");
    assert!(
        src.contains("skill_name_validated"),
        "router must emit skill_name_validated on successful register"
    );
    assert!(
        src.contains("skill_name_rejected"),
        "router must emit skill_name_rejected on naming reject"
    );
}
