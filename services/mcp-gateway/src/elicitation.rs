//! TASK-MCP-008 elicitation: server-initiated structured prompts for mid-call user input.
//!
//! In the synchronous gateway - no long-running tasks runtime, NATS, KMS, or S3 yet - this realizes the
//! build-plan slice ("an elicitation request round-trips; a declined elicitation aborts the call
//! cleanly") as an in-memory store, the dev-real analog of the in-memory
//! [`ToolRegistry`](crate::federation::registry::ToolRegistry). A destructive `tools/call` (TASK-MCP-006)
//! that needs confirmation creates a `confirmation` elicitation here, the caller responds via the REST
//! endpoints, and the gate consults [`ElicitationStore::is_confirmed`].
//!
//! Deferred to the DB slice (none load-bearing for the round-trip): the `mcp_elicitations` table + RLS,
//! KMS-encrypted payloads, NATS push, Postgres LISTEN/NOTIFY wakeups, S3 presigned `file_upload`, Redis
//! rate limiting, the timeout sweeper, and 30-day pruning. The closed enums, the five fixed per-type
//! response schemas, and the response validation are the durable contract and live here unchanged when
//! persistence lands.

use std::collections::HashMap;
use std::sync::RwLock;

use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

/// The five elicitation kinds (DEC-1141). [`ElicitationType::ALL`] pins the cardinality at 5.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ElicitationType {
    /// Free text (`{ value: string }`).
    StringInput,
    /// Pick one of the declared choices (`{ value: <choice> }`).
    SingleChoice,
    /// Pick a subset of the declared choices (`{ values: [<choice>...] }`).
    MultiChoice,
    /// Approve or reject an action (`{ confirmed: bool, reason?: string }`).
    Confirmation,
    /// Upload a file (`{ s3_key, sha256, size_bytes }`).
    FileUpload,
}

impl ElicitationType {
    /// Every variant, for the cardinality test and iteration.
    pub const ALL: [ElicitationType; 5] = [
        ElicitationType::StringInput,
        ElicitationType::SingleChoice,
        ElicitationType::MultiChoice,
        ElicitationType::Confirmation,
        ElicitationType::FileUpload,
    ];

    /// The snake_case wire label.
    pub fn as_str(&self) -> &'static str {
        match self {
            ElicitationType::StringInput => "string_input",
            ElicitationType::SingleChoice => "single_choice",
            ElicitationType::MultiChoice => "multi_choice",
            ElicitationType::Confirmation => "confirmation",
            ElicitationType::FileUpload => "file_upload",
        }
    }

    /// Parse the snake_case wire label (the Postgres `elicitation_type` enum text) back to a variant.
    /// Used by the DB-slice store ([`crate::elicitation_pg`]) when reading a persisted row.
    pub fn from_wire(s: &str) -> Option<ElicitationType> {
        ElicitationType::ALL.into_iter().find(|t| t.as_str() == s)
    }
}

/// The lifecycle of an elicitation - linear, no reverse transitions. [`ElicitationStatus::ALL`] pins 5.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ElicitationStatus {
    /// Awaiting a caller response.
    Pending,
    /// The caller responded and the payload validated.
    Responded,
    /// The timeout elapsed with no response (set by the deferred sweeper).
    Expired,
    /// The caller cancelled.
    Cancelled,
    /// The retry cap was hit with an invalid payload (terminal).
    ValidationFailed,
}

impl ElicitationStatus {
    /// Every variant, for the cardinality test.
    pub const ALL: [ElicitationStatus; 5] = [
        ElicitationStatus::Pending,
        ElicitationStatus::Responded,
        ElicitationStatus::Expired,
        ElicitationStatus::Cancelled,
        ElicitationStatus::ValidationFailed,
    ];

    /// The snake_case wire label.
    pub fn as_str(&self) -> &'static str {
        match self {
            ElicitationStatus::Pending => "pending",
            ElicitationStatus::Responded => "responded",
            ElicitationStatus::Expired => "expired",
            ElicitationStatus::Cancelled => "cancelled",
            ElicitationStatus::ValidationFailed => "validation_failed",
        }
    }
}

/// The retry cap for re-submitting an invalid response to one elicitation (DEC-1150): three invalid
/// submissions are answered 422-and-retry; the fourth makes the elicitation terminal.
pub const MAX_RETRIES: u32 = 3;

/// The RFC body of a caller response: `{ "response_payload": <value> }`.
#[derive(Debug, Deserialize)]
pub struct ElicitationRespondReq {
    /// The caller's answer, validated against the elicitation's fixed type schema.
    pub response_payload: Value,
}

/// The fixed JSON Schema describing a valid response for an elicitation `elicitation_type`
/// (DEC-1141/1152/1160). `choices` are the allowed values for the choice types (ignored otherwise).
pub fn response_schema(elicitation_type: ElicitationType, choices: &[String]) -> Value {
    match elicitation_type {
        ElicitationType::StringInput => json!({
            "type": "object",
            "properties": { "value": { "type": "string", "maxLength": 4096 } },
            "required": ["value"]
        }),
        ElicitationType::SingleChoice => json!({
            "type": "object",
            "properties": { "value": { "enum": choices } },
            "required": ["value"]
        }),
        ElicitationType::MultiChoice => json!({
            "type": "object",
            "properties": {
                "values": { "type": "array", "items": { "enum": choices }, "uniqueItems": true }
            },
            "required": ["values"]
        }),
        ElicitationType::Confirmation => json!({
            "type": "object",
            "properties": {
                "confirmed": { "type": "boolean" },
                "reason": { "type": "string", "maxLength": 512 }
            },
            "required": ["confirmed"]
        }),
        ElicitationType::FileUpload => json!({
            "type": "object",
            "properties": {
                "s3_key": { "type": "string", "pattern": "^elicitations/[a-f0-9-]{36}/[a-zA-Z0-9_.-]+$" },
                "sha256": { "type": "string", "pattern": "^[a-f0-9]{64}$" },
                "size_bytes": { "type": "integer", "minimum": 1, "maximum": 104857600 }
            },
            "required": ["s3_key", "sha256", "size_bytes"]
        }),
    }
}

/// Validate a caller `payload` against the fixed schema for `elicitation_type`. Returns the list of
/// human-readable validation errors (empty = valid). Hand-rolled over the five closed shapes so the
/// gateway needs no general JSON Schema engine; it mirrors [`response_schema`] exactly.
pub fn validate_response(
    elicitation_type: ElicitationType,
    choices: &[String],
    payload: &Value,
) -> Vec<String> {
    let mut errs = Vec::new();
    let Some(obj) = payload.as_object() else {
        errs.push("response must be a JSON object".to_string());
        return errs;
    };
    match elicitation_type {
        ElicitationType::StringInput => match obj.get("value") {
            Some(Value::String(s)) if s.chars().count() <= 4096 => {}
            Some(Value::String(_)) => errs.push("value exceeds maxLength 4096".to_string()),
            Some(_) => errs.push("value must be a string".to_string()),
            None => errs.push("missing required field: value".to_string()),
        },
        ElicitationType::SingleChoice => match obj.get("value") {
            Some(Value::String(s)) if choices.iter().any(|c| c == s) => {}
            Some(_) => errs.push("value must be one of the declared choices".to_string()),
            None => errs.push("missing required field: value".to_string()),
        },
        ElicitationType::MultiChoice => match obj.get("values") {
            Some(Value::Array(items)) => {
                let mut seen = std::collections::BTreeSet::new();
                for it in items {
                    match it.as_str() {
                        Some(s) if choices.iter().any(|c| c == s) => {
                            if !seen.insert(s.to_string()) {
                                errs.push("values must be unique".to_string());
                            }
                        }
                        _ => {
                            errs.push("each value must be one of the declared choices".to_string())
                        }
                    }
                }
            }
            Some(_) => errs.push("values must be an array".to_string()),
            None => errs.push("missing required field: values".to_string()),
        },
        ElicitationType::Confirmation => {
            match obj.get("confirmed") {
                Some(Value::Bool(_)) => {}
                Some(_) => errs.push("confirmed must be a boolean".to_string()),
                None => errs.push("missing required field: confirmed".to_string()),
            }
            if let Some(reason) = obj.get("reason") {
                match reason {
                    Value::String(s) if s.chars().count() <= 512 => {}
                    Value::String(_) => errs.push("reason exceeds maxLength 512".to_string()),
                    _ => errs.push("reason must be a string".to_string()),
                }
            }
        }
        ElicitationType::FileUpload => {
            match obj.get("s3_key").and_then(|v| v.as_str()) {
                Some(s) if s.starts_with("elicitations/") => {}
                _ => errs.push("s3_key missing or malformed".to_string()),
            }
            match obj.get("sha256").and_then(|v| v.as_str()) {
                Some(s) if s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()) => {}
                _ => errs.push("sha256 must be 64 hex chars".to_string()),
            }
            match obj.get("size_bytes").and_then(|v| v.as_i64()) {
                Some(n) if (1..=104_857_600).contains(&n) => {}
                _ => errs.push("size_bytes must be between 1 and 104857600".to_string()),
            }
        }
    }
    errs
}

/// A single elicitation and its current state.
#[derive(Clone, Debug)]
pub struct Elicitation {
    /// Server-generated id.
    pub id: Uuid,
    /// The tool that raised it (informational).
    pub tool_id: String,
    /// The elicitation kind.
    pub elicitation_type: ElicitationType,
    /// The prompt shown to the caller (free-form per type).
    pub prompt: Value,
    /// The fixed response schema for this type.
    pub response_schema: Value,
    /// Choices for the choice types (empty otherwise).
    pub choices: Vec<String>,
    /// Lifecycle status.
    pub status: ElicitationStatus,
    /// The validated response payload, once responded.
    pub response: Option<Value>,
    /// How many invalid responses have been submitted.
    pub retry_count: u32,
}

impl Elicitation {
    /// Whether this is a confirmation the caller approved (`confirmed == true`). The TASK-MCP-006 gate
    /// forwards a held destructive call only when this is true.
    pub fn is_confirmed(&self) -> bool {
        self.status == ElicitationStatus::Responded
            && self.elicitation_type == ElicitationType::Confirmation
            && self
                .response
                .as_ref()
                .and_then(|r| r.get("confirmed"))
                .and_then(|c| c.as_bool())
                .unwrap_or(false)
    }

    /// The spec-facing view (id, type, prompt, schema, status) a caller poll returns.
    pub fn to_request_view(&self) -> Value {
        json!({
            "elicitation_id": self.id,
            "tool_id": self.tool_id,
            "elicitation_type": self.elicitation_type.as_str(),
            "prompt": self.prompt,
            "response_schema": self.response_schema,
            "status": self.status.as_str(),
        })
    }
}

/// The outcome of submitting a response, which drives the HTTP status the handler returns.
#[derive(Debug, Eq, PartialEq)]
pub enum RespondOutcome {
    /// Validated and recorded (status now `Responded`). `confirmed` distinguishes accept vs decline for
    /// confirmation elicitations.
    Recorded {
        /// True when a confirmation elicitation was approved.
        confirmed: bool,
    },
    /// The identical response was already recorded (idempotent re-submit).
    AlreadyRecorded {
        /// True when a confirmation elicitation was approved.
        confirmed: bool,
    },
    /// The payload failed validation; the caller may retry. Carries the errors.
    Invalid(Vec<String>),
    /// The retry cap was hit; the elicitation is now terminal (`ValidationFailed`). Carries the errors.
    ValidationFailed(Vec<String>),
    /// No such elicitation.
    NotFound,
    /// The elicitation is no longer pending (cancelled/expired/terminal, or a different payload after a
    /// recorded one) - not re-respondable.
    NotPending,
}

/// In-memory elicitation store. Thread-safe via `RwLock`, matching the
/// [`ToolRegistry`](crate::federation::registry::ToolRegistry) pattern; the persistent table + RLS land
/// in the DB slice.
#[derive(Debug, Default)]
pub struct ElicitationStore {
    inner: RwLock<HashMap<Uuid, Elicitation>>,
}

impl ElicitationStore {
    /// Empty store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a pending elicitation and return it. `choices` matter only for the choice types.
    pub fn create(
        &self,
        tool_id: &str,
        elicitation_type: ElicitationType,
        prompt: Value,
        choices: Vec<String>,
    ) -> Elicitation {
        let id = Uuid::new_v4();
        let e = Elicitation {
            id,
            tool_id: tool_id.to_string(),
            elicitation_type,
            prompt,
            response_schema: response_schema(elicitation_type, &choices),
            choices,
            status: ElicitationStatus::Pending,
            response: None,
            retry_count: 0,
        };
        self.inner.write().expect("poisoned").insert(id, e.clone());
        e
    }

    /// Create a `confirmation` elicitation for a destructive action (the TASK-MCP-006 hold).
    pub fn create_confirmation(&self, tool_id: &str, prompt: Value) -> Elicitation {
        self.create(tool_id, ElicitationType::Confirmation, prompt, Vec::new())
    }

    /// Look up an elicitation by id.
    pub fn get(&self, id: Uuid) -> Option<Elicitation> {
        self.inner.read().expect("poisoned").get(&id).cloned()
    }

    /// Whether the elicitation is a caller-approved confirmation (used by the TASK-MCP-006 gate).
    pub fn is_confirmed(&self, id: Uuid) -> bool {
        self.get(id).map(|e| e.is_confirmed()).unwrap_or(false)
    }

    /// The confirmation verdict for the TASK-MCP-006 gate: `Some(true)` approved, `Some(false)` declined,
    /// `None` when there is no responded confirmation for that id (unknown, pending, or non-confirmation).
    pub fn confirmation_state(&self, id: Uuid) -> Option<bool> {
        self.get(id).and_then(|e| {
            (e.elicitation_type == ElicitationType::Confirmation
                && e.status == ElicitationStatus::Responded)
                .then(|| e.is_confirmed())
        })
    }

    /// The spec-facing views of every pending elicitation (caller poll), sorted by id.
    pub fn pending(&self) -> Vec<Value> {
        let g = self.inner.read().expect("poisoned");
        let mut out: Vec<&Elicitation> = g
            .values()
            .filter(|e| e.status == ElicitationStatus::Pending)
            .collect();
        out.sort_by_key(|e| e.id);
        out.into_iter().map(|e| e.to_request_view()).collect()
    }

    /// Submit a caller response. Validates against the type schema, transitions the elicitation, and
    /// returns the outcome (idempotent on an identical re-submit of an already-recorded response).
    pub fn respond(&self, id: Uuid, payload: Value) -> RespondOutcome {
        let mut g = self.inner.write().expect("poisoned");
        let Some(e) = g.get_mut(&id) else {
            return RespondOutcome::NotFound;
        };
        if e.status == ElicitationStatus::Responded {
            return if e.response.as_ref() == Some(&payload) {
                RespondOutcome::AlreadyRecorded {
                    confirmed: e.is_confirmed(),
                }
            } else {
                RespondOutcome::NotPending
            };
        }
        if e.status != ElicitationStatus::Pending {
            return RespondOutcome::NotPending;
        }
        let errs = validate_response(e.elicitation_type, &e.choices, &payload);
        if !errs.is_empty() {
            e.retry_count += 1;
            if e.retry_count > MAX_RETRIES {
                e.status = ElicitationStatus::ValidationFailed;
                return RespondOutcome::ValidationFailed(errs);
            }
            return RespondOutcome::Invalid(errs);
        }
        e.status = ElicitationStatus::Responded;
        e.response = Some(payload);
        RespondOutcome::Recorded {
            confirmed: e.is_confirmed(),
        }
    }

    /// Cancel a pending elicitation (caller abort). Returns whether it was pending.
    pub fn cancel(&self, id: Uuid) -> bool {
        let mut g = self.inner.write().expect("poisoned");
        match g.get_mut(&id) {
            Some(e) if e.status == ElicitationStatus::Pending => {
                e.status = ElicitationStatus::Cancelled;
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_and_status_have_exactly_five_variants() {
        assert_eq!(ElicitationType::ALL.len(), 5);
        assert_eq!(ElicitationStatus::ALL.len(), 5);
        // Labels are the snake_case wire forms the (deferred) Postgres enums use.
        let mut labels: Vec<&str> = ElicitationType::ALL.iter().map(|t| t.as_str()).collect();
        labels.sort_unstable();
        assert_eq!(
            labels,
            vec![
                "confirmation",
                "file_upload",
                "multi_choice",
                "single_choice",
                "string_input"
            ]
        );
    }

    #[test]
    fn from_wire_round_trips_every_variant_and_rejects_unknown() {
        for t in ElicitationType::ALL {
            assert_eq!(ElicitationType::from_wire(t.as_str()), Some(t));
        }
        assert_eq!(ElicitationType::from_wire("nope"), None);
    }

    #[test]
    fn confirmation_schema_requires_confirmed_boolean() {
        let s = response_schema(ElicitationType::Confirmation, &[]);
        assert_eq!(s["required"], json!(["confirmed"]));
        assert_eq!(s["properties"]["confirmed"]["type"], "boolean");
    }

    #[test]
    fn validate_confirmation_accepts_bool_rejects_missing() {
        assert!(validate_response(
            ElicitationType::Confirmation,
            &[],
            &json!({ "confirmed": true })
        )
        .is_empty());
        assert!(!validate_response(ElicitationType::Confirmation, &[], &json!({})).is_empty());
        assert!(!validate_response(
            ElicitationType::Confirmation,
            &[],
            &json!({ "confirmed": "yes" })
        )
        .is_empty());
    }

    #[test]
    fn validate_string_and_choice_types() {
        assert!(
            validate_response(ElicitationType::StringInput, &[], &json!({ "value": "hi" }))
                .is_empty()
        );
        assert!(
            !validate_response(ElicitationType::StringInput, &[], &json!({})).is_empty(),
            "string_input requires value"
        );
        let choices = vec!["a".to_string(), "b".to_string()];
        assert!(validate_response(
            ElicitationType::SingleChoice,
            &choices,
            &json!({ "value": "a" })
        )
        .is_empty());
        assert!(!validate_response(
            ElicitationType::SingleChoice,
            &choices,
            &json!({ "value": "z" })
        )
        .is_empty());
        assert!(
            !validate_response(
                ElicitationType::MultiChoice,
                &choices,
                &json!({ "values": ["a", "a"] })
            )
            .is_empty(),
            "duplicates rejected"
        );
    }

    #[test]
    fn confirmation_round_trip_accept_and_decline() {
        let store = ElicitationStore::new();
        let approved =
            store.create_confirmation("cyberos.kb.bulk_delete", json!({ "title": "ok?" }));
        assert_eq!(
            store.respond(approved.id, json!({ "confirmed": true })),
            RespondOutcome::Recorded { confirmed: true }
        );
        assert!(store.is_confirmed(approved.id));

        let declined =
            store.create_confirmation("cyberos.kb.bulk_delete", json!({ "title": "ok?" }));
        assert_eq!(
            store.respond(declined.id, json!({ "confirmed": false })),
            RespondOutcome::Recorded { confirmed: false }
        );
        assert!(!store.is_confirmed(declined.id), "decline aborts the call");
    }

    #[test]
    fn invalid_responses_retry_then_go_terminal() {
        let store = ElicitationStore::new();
        let e = store.create("t", ElicitationType::StringInput, json!({}), Vec::new());
        for _ in 0..MAX_RETRIES {
            assert!(matches!(
                store.respond(e.id, json!({})),
                RespondOutcome::Invalid(_)
            ));
        }
        assert!(matches!(
            store.respond(e.id, json!({})),
            RespondOutcome::ValidationFailed(_)
        ));
        assert_eq!(
            store.get(e.id).unwrap().status,
            ElicitationStatus::ValidationFailed
        );
    }

    #[test]
    fn confirmation_state_reflects_approval_decline_and_absence() {
        let store = ElicitationStore::new();
        let yes = store.create_confirmation("t", json!({}));
        store.respond(yes.id, json!({ "confirmed": true }));
        assert_eq!(store.confirmation_state(yes.id), Some(true));

        let no = store.create_confirmation("t", json!({}));
        store.respond(no.id, json!({ "confirmed": false }));
        assert_eq!(store.confirmation_state(no.id), Some(false));

        // Pending confirmation, unknown id, and a non-confirmation type all read as None.
        let pending = store.create_confirmation("t", json!({}));
        assert_eq!(store.confirmation_state(pending.id), None);
        assert_eq!(store.confirmation_state(Uuid::new_v4()), None);
        let str_input = store.create("t", ElicitationType::StringInput, json!({}), Vec::new());
        store.respond(str_input.id, json!({ "value": "x" }));
        assert_eq!(store.confirmation_state(str_input.id), None);
    }

    #[test]
    fn idempotent_resubmit_and_cancel() {
        let store = ElicitationStore::new();
        let e = store.create("t", ElicitationType::StringInput, json!({}), Vec::new());
        assert_eq!(
            store.respond(e.id, json!({ "value": "x" })),
            RespondOutcome::Recorded { confirmed: false }
        );
        assert_eq!(
            store.respond(e.id, json!({ "value": "x" })),
            RespondOutcome::AlreadyRecorded { confirmed: false }
        );
        // A pending elicitation cancels; an already-resolved one does not.
        let p = store.create("t", ElicitationType::StringInput, json!({}), Vec::new());
        assert!(store.cancel(p.id));
        assert!(!store.cancel(p.id));
        assert_eq!(store.pending().len(), 0, "no pending left");
    }
}
