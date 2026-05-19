//! FR-PROJ-001 §4 #3 + §4 #4 — FSM transition coverage.

use cyberos_proj::status_fsm::{allowed_transitions, validate};
use cyberos_proj::types::IssueStatus;
use cyberos_proj::IssueError;

#[test]
fn complete_transition_matrix() {
    // Forward path: triage → todo → doing → review → done.
    assert!(validate(IssueStatus::Triage, IssueStatus::Todo).is_ok());
    assert!(validate(IssueStatus::Todo, IssueStatus::Doing).is_ok());
    assert!(validate(IssueStatus::Doing, IssueStatus::Review).is_ok());
    assert!(validate(IssueStatus::Review, IssueStatus::Done).is_ok());

    // Backward + sideways:
    assert!(validate(IssueStatus::Todo, IssueStatus::Triage).is_ok()); // deferral
    assert!(validate(IssueStatus::Doing, IssueStatus::Todo).is_ok()); // pause
    assert!(validate(IssueStatus::Review, IssueStatus::Doing).is_ok()); // rejected
    assert!(validate(IssueStatus::Review, IssueStatus::Todo).is_ok()); // significant rework

    // Direct-to-done from intermediate states (cancel-as-done).
    assert!(validate(IssueStatus::Todo, IssueStatus::Done).is_ok());
    assert!(validate(IssueStatus::Doing, IssueStatus::Done).is_ok());
}

#[test]
fn done_is_terminal_for_all_targets() {
    for to in [
        IssueStatus::Triage,
        IssueStatus::Todo,
        IssueStatus::Doing,
        IssueStatus::Review,
    ] {
        let r = validate(IssueStatus::Done, to);
        assert!(
            matches!(r, Err(IssueError::IllegalStatusTransition { .. })),
            "done → {to:?} should be illegal"
        );
    }
}

#[test]
fn illegal_transition_returns_allowed_list() {
    // triage → doing is illegal (must go through todo).
    let err = validate(IssueStatus::Triage, IssueStatus::Doing).unwrap_err();
    match err {
        IssueError::IllegalStatusTransition { from, to, allowed } => {
            assert_eq!(from, IssueStatus::Triage);
            assert_eq!(to, IssueStatus::Doing);
            assert_eq!(allowed, vec![IssueStatus::Todo]);
        }
        _ => panic!("expected IllegalStatusTransition"),
    }
}

#[test]
fn allowed_transitions_done_is_empty() {
    assert!(allowed_transitions(IssueStatus::Done).is_empty());
    assert!(allowed_transitions(IssueStatus::Deleted).is_empty());
}

#[test]
fn allowed_transitions_count_per_state() {
    assert_eq!(allowed_transitions(IssueStatus::Triage).len(), 1);
    assert_eq!(allowed_transitions(IssueStatus::Todo).len(), 3);
    assert_eq!(allowed_transitions(IssueStatus::Doing).len(), 3);
    assert_eq!(allowed_transitions(IssueStatus::Review).len(), 3);
    assert_eq!(allowed_transitions(IssueStatus::Done).len(), 0);
}

#[test]
fn http_status_for_illegal_is_400() {
    let err = validate(IssueStatus::Triage, IssueStatus::Done).unwrap_err();
    assert_eq!(err.http_status(), 400);
    assert_eq!(err.code(), "illegal_status_transition");
}
