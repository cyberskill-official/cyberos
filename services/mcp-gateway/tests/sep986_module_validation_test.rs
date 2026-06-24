//! AC-03, AC-17: module list validation.

use cyberos_mcp_gateway::naming::module_registry::{all_modules, is_valid_module};
use cyberos_mcp_gateway::naming::validator::{validate_sync, NamingError};

#[test]
fn all_approved_modules_pass() {
    for module in [
        "ai", "auth", "chat", "crm", "cuo", "doc", "email", "esop", "hr", "inv", "kb", "learn",
        "mcp", "memory", "obs", "okr", "portal", "proj", "res", "rew", "skill", "ten", "time",
    ] {
        assert!(is_valid_module(module), "module '{module}' should be approved");
    }
}

#[test]
fn module_count_is_23() {
    assert_eq!(all_modules().len(), 23, "approved module count must be 23 per FR-MCP-003");
}

#[test]
fn unknown_modules_rejected() {
    for module in ["calendar", "unknown", "payroll", "finance", "MEMORY", "Mcp", "", "hr2"] {
        assert!(!is_valid_module(module), "module '{module}' should NOT be approved");
    }
}

#[test]
fn validate_sync_unknown_module_gives_correct_error() {
    let err = validate_sync("cyberos.calendar.list_events").expect_err("calendar is unknown");
    match err {
        NamingError::UnknownModule { module, .. } => assert_eq!(module, "calendar"),
        other => panic!("expected UnknownModule, got {other:?}"),
    }
}
