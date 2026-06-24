//! AC-02, AC-04, AC-05, AC-15, AC-19: regex matching and error specificity.

use cyberos_mcp_gateway::naming::validator::{validate_sync, NamingError};

#[test]
fn valid_skill_ids_pass() {
    // Every module here is in the approved registry and every verb is in the closed enum.
    let valid = [
        "cyberos.kb.list_articles",
        "cyberos.email.send_message",
        "cyberos.inv.create_invoice",
        "cyberos.hr.get_employee",
        "cyberos.mcp.validate_skill",
        "cyberos.memory.fetch_entries",
        "cyberos.ai.generate_summary",
        "cyberos.proj.update_task",
        "cyberos.auth.delete_session",
        "cyberos.crm.search_contacts",
        "cyberos.ten.sync_tenant",
        "cyberos.rew.accept_reward",
        "cyberos.skill.execute_step",
        "cyberos.obs.replay_event",
        "cyberos.portal.reject_request",
    ];
    for id in valid {
        assert!(validate_sync(id).is_ok(), "expected '{id}' to pass, got {:?}", validate_sync(id));
    }
}

#[test]
fn valid_id_extracts_correct_parts() {
    let vr = validate_sync("cyberos.email.send_message").expect("should be valid");
    assert_eq!(vr.module, "email");
    assert_eq!(vr.verb.as_str(), "send");
    assert_eq!(vr.noun, "message");
}

#[test]
fn missing_cyberos_prefix_rejected() {
    let err = validate_sync("calendar.list_events").expect_err("should fail");
    match err {
        NamingError::MalformedSkillId { reason, .. } => {
            assert!(reason.contains("prefix"), "error should mention the missing prefix: {reason}");
        }
        other => panic!("expected MalformedSkillId, got {other:?}"),
    }
}

#[test]
fn camelcase_noun_rejected() {
    let err = validate_sync("cyberos.email.listEvents").expect_err("should fail");
    assert!(matches!(err, NamingError::MalformedSkillId { .. }), "got {err:?}");
}

#[test]
fn uppercase_in_module_rejected() {
    let err = validate_sync("cyberos.Email.list_events").expect_err("should fail");
    assert!(matches!(err, NamingError::MalformedSkillId { .. }), "got {err:?}");
}

#[test]
fn uppercase_in_verb_rejected() {
    let err = validate_sync("cyberos.email.List_events").expect_err("should fail");
    assert!(matches!(err, NamingError::MalformedSkillId { .. }), "got {err:?}");
}

#[test]
fn no_underscore_separator_rejected() {
    // The pattern requires a verb_noun underscore; "listevents" has none.
    let err = validate_sync("cyberos.email.listevents").expect_err("should fail");
    assert!(
        matches!(err, NamingError::MalformedSkillId { .. } | NamingError::InvalidVerb { .. }),
        "got {err:?}"
    );
}

#[test]
fn double_dot_rejected() {
    let err = validate_sync("cyberos..list_events").expect_err("should fail");
    assert!(matches!(err, NamingError::MalformedSkillId { .. }), "got {err:?}");
}

#[test]
fn empty_string_rejected() {
    let err = validate_sync("").expect_err("should fail");
    assert!(matches!(err, NamingError::MalformedSkillId { .. }), "got {err:?}");
}

#[test]
fn error_message_is_specific_for_missing_prefix() {
    let msg = validate_sync("calendar.list_events").expect_err("should fail").to_string();
    assert!(msg.contains("cyberos"), "error should mention the 'cyberos' prefix: {msg}");
}

#[test]
fn error_message_is_specific_for_invalid_verb() {
    let msg = validate_sync("cyberos.email.retrieve_messages").expect_err("should fail").to_string();
    assert!(msg.contains("retrieve") || msg.contains("verb"), "error should name the bad verb: {msg}");
}

#[test]
fn error_message_is_specific_for_unknown_module() {
    let msg = validate_sync("cyberos.unknown.list_things").expect_err("should fail").to_string();
    assert!(msg.contains("unknown"), "error should name the bad module: {msg}");
}
