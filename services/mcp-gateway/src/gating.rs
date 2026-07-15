//! TASK-MCP-006 tool-annotation gating: a `destructiveHint` tool's `tools/call` is held for an elicited
//! confirmation (TASK-MCP-008) before the gateway forwards it.
//!
//! The decision is pure ([`evaluate`]); the router owns the side effects (creating the confirmation
//! elicitation, auditing, forwarding). A read-only tool calls straight through; a destructive tool is
//! held until the caller answers its confirmation, then forwards on approval or aborts on decline. This
//! de-stubs the elicit mode the annotation gate referenced - the `destructive()` annotation constructor
//! already noted "requires Elicitation per TASK-MCP-006".

use serde_json::{json, Value};
use uuid::Uuid;

use crate::elicitation::{response_schema, Elicitation, ElicitationType};
use crate::protocol::tools_call::{Content, ToolsCallResult};

/// What a `tools/call` needs before the gateway forwards it.
#[derive(Debug, Eq, PartialEq)]
pub enum ConfirmationOutcome {
    /// Forward to the module (not destructive, or the caller confirmed).
    Proceed,
    /// The caller declined the confirmation: abort the call cleanly.
    Declined,
    /// No valid confirmation yet: hold the call and raise a confirmation elicitation.
    NeedsConfirmation,
}

/// Decide from the tool's destructive hint and the confirmation verdict. `confirmed` is `Some(true)`
/// when the caller approved an elicited confirmation, `Some(false)` when they declined, and `None` when
/// no responded confirmation was referenced.
pub fn evaluate(destructive: bool, confirmed: Option<bool>) -> ConfirmationOutcome {
    if !destructive {
        return ConfirmationOutcome::Proceed;
    }
    match confirmed {
        Some(true) => ConfirmationOutcome::Proceed,
        Some(false) => ConfirmationOutcome::Declined,
        None => ConfirmationOutcome::NeedsConfirmation,
    }
}

/// The prompt for a destructive tool's confirmation elicitation.
pub fn confirmation_prompt(tool_name: &str) -> Value {
    json!({
        "title": "Confirm destructive action",
        "description": format!("The tool {tool_name} may modify state irreversibly. Confirm to proceed."),
        "action_summary": format!("Run {tool_name}"),
    })
}

/// The held-result body from primitives, for the DB store-of-record path where the gate holds only the
/// new confirmation's id (not an [`Elicitation`]). Confirmation-shaped: type and schema are the fixed
/// confirmation ones.
pub fn held_result_parts(elicitation_id: Uuid, tool_id: &str, prompt: &Value) -> ToolsCallResult {
    ToolsCallResult {
        content: vec![Content::Text {
            text: format!("Confirmation required before running {tool_id}."),
        }],
        is_error: false,
        structured_content: Some(json!({
            "elicitation_required": true,
            "elicitation_id": elicitation_id,
            "elicitation_type": ElicitationType::Confirmation.as_str(),
            "prompt": prompt,
            "response_schema": response_schema(ElicitationType::Confirmation, &[]),
        })),
    }
}

/// The `tools/call` result returned when a destructive call is held: not an error, but it carries the
/// confirmation elicitation the caller must answer (then re-invoke with `_meta.confirmation_id`). The
/// in-memory path passes the created [`Elicitation`]; the shape matches [`held_result_parts`].
pub fn held_result(e: &Elicitation) -> ToolsCallResult {
    held_result_parts(e.id, &e.tool_id, &e.prompt)
}

/// The `tools/call` result returned when the caller declined the confirmation: an in-band tool error.
pub fn user_rejected_result() -> ToolsCallResult {
    ToolsCallResult {
        content: vec![Content::Text {
            text: "user_rejected: the confirmation was declined.".to_string(),
        }],
        is_error: true,
        structured_content: Some(json!({ "user_rejected": true })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elicitation::ElicitationStore;

    #[test]
    fn non_destructive_always_proceeds() {
        assert_eq!(evaluate(false, None), ConfirmationOutcome::Proceed);
        assert_eq!(evaluate(false, Some(false)), ConfirmationOutcome::Proceed);
    }

    #[test]
    fn destructive_holds_then_proceeds_or_declines() {
        assert_eq!(evaluate(true, None), ConfirmationOutcome::NeedsConfirmation);
        assert_eq!(evaluate(true, Some(true)), ConfirmationOutcome::Proceed);
        assert_eq!(evaluate(true, Some(false)), ConfirmationOutcome::Declined);
    }

    #[test]
    fn held_result_carries_the_elicitation_and_is_not_an_error() {
        let store = ElicitationStore::new();
        let e = store.create_confirmation("cyberos.kb.bulk_delete", json!({ "title": "ok?" }));
        let r = held_result(&e);
        assert!(!r.is_error);
        let sc = r.structured_content.unwrap();
        assert_eq!(sc["elicitation_required"], true);
        assert_eq!(sc["elicitation_type"], "confirmation");
    }

    #[test]
    fn user_rejected_result_is_an_in_band_error() {
        let r = user_rejected_result();
        assert!(r.is_error);
        assert_eq!(r.structured_content.unwrap()["user_rejected"], true);
    }
}
