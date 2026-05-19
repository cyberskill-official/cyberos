//! FR-PROJ-001 §1 #3 — Issue status finite-state machine.
//!
//! Legal transitions:
//!   triage  → todo
//!   todo    → doing | triage | done
//!   doing   → review | todo | done
//!   review  → done | doing | todo
//!   done    → (terminal; reopen requires explicit API path)
//!   deleted → (terminal; reserved for soft-delete)
//!
//! Illegal transitions are rejected with `IllegalStatusTransition` carrying
//! the list of `allowed` states so clients can recover without a round-trip.

use crate::errors::{IssueError, IssueResult};
use crate::types::IssueStatus;

pub const fn allowed_transitions(from: IssueStatus) -> &'static [IssueStatus] {
    match from {
        IssueStatus::Triage => &[IssueStatus::Todo],
        IssueStatus::Todo => &[IssueStatus::Doing, IssueStatus::Triage, IssueStatus::Done],
        IssueStatus::Doing => &[IssueStatus::Review, IssueStatus::Todo, IssueStatus::Done],
        IssueStatus::Review => &[IssueStatus::Done, IssueStatus::Doing, IssueStatus::Todo],
        IssueStatus::Done => &[],
        IssueStatus::Deleted => &[],
    }
}

pub fn validate(from: IssueStatus, to: IssueStatus) -> IssueResult<()> {
    if from == to {
        return Ok(()); // §4 #15 — same-status PATCH is a no-op
    }
    let allowed = allowed_transitions(from);
    if allowed.contains(&to) {
        Ok(())
    } else {
        Err(IssueError::IllegalStatusTransition {
            from,
            to,
            allowed: allowed.to_vec(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triage_to_todo_allowed() {
        assert!(validate(IssueStatus::Triage, IssueStatus::Todo).is_ok());
    }

    #[test]
    fn triage_to_done_rejected() {
        let r = validate(IssueStatus::Triage, IssueStatus::Done);
        assert!(matches!(r, Err(IssueError::IllegalStatusTransition { .. })));
        if let Err(IssueError::IllegalStatusTransition { allowed, .. }) = r {
            assert_eq!(allowed, vec![IssueStatus::Todo]);
        }
    }

    #[test]
    fn done_is_terminal() {
        for to in [
            IssueStatus::Triage,
            IssueStatus::Todo,
            IssueStatus::Doing,
            IssueStatus::Review,
        ] {
            assert!(
                validate(IssueStatus::Done, to).is_err(),
                "done → {to:?} must be rejected (use reopen API)"
            );
        }
    }

    #[test]
    fn same_status_is_noop() {
        for s in [
            IssueStatus::Triage,
            IssueStatus::Todo,
            IssueStatus::Doing,
            IssueStatus::Review,
            IssueStatus::Done,
        ] {
            assert!(validate(s, s).is_ok(), "{s:?} → {s:?} should be allowed (no-op)");
        }
    }

    #[test]
    fn doing_can_step_back_to_todo() {
        // Workflow nuance: a developer pauses an in-progress issue.
        assert!(validate(IssueStatus::Doing, IssueStatus::Todo).is_ok());
    }

    #[test]
    fn review_can_step_back_to_doing_or_todo() {
        // Workflow nuance: reviewer rejects → back to doing, or significant rework → todo.
        assert!(validate(IssueStatus::Review, IssueStatus::Doing).is_ok());
        assert!(validate(IssueStatus::Review, IssueStatus::Todo).is_ok());
    }

    #[test]
    fn deleted_is_terminal() {
        for to in [
            IssueStatus::Triage,
            IssueStatus::Todo,
            IssueStatus::Doing,
            IssueStatus::Review,
            IssueStatus::Done,
        ] {
            assert!(validate(IssueStatus::Deleted, to).is_err());
        }
    }
}
