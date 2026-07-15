//! TASK-MEMORY-121 §1 #15 / §4 AC 1, 2, 4, 17 — the published contract is the dependency surface.
//!
//! Every emitted interaction-event body MUST validate against contracts/interaction-event.schema.json
//! (JSON Schema draft 2020-12); the schema pins `schema_version` const 1; a malformed content_ref arm
//! (a hash without `sha256`) and an unknown `module` are rejected. This test needs NO Postgres — it is a
//! pure serde + schema check — so it runs by default (not `#[ignore]`), guarding the contract on every
//! `cargo test -p cyberos-memory`.
//!
//! Uses jsonschema 0.18 (the version already in the workspace lockfile) with the `draft202012` feature.

use cyberos_memory::interaction::{
    canonical_audit_body, ContentRef, EventClass, InteractionEvent, Module, SourceChannel,
    TargetRef, AUDIT_ROW_KIND,
};
use jsonschema::{Draft, JSONSchema};
use serde_json::json;
use uuid::Uuid;

/// Compile the published contract under draft 2020-12 (the `$schema` the file declares). `$id` is a
/// non-resolvable URL, but nothing `$ref`s it, so no remote retrieval happens.
fn compile() -> JSONSchema {
    let schema: serde_json::Value =
        serde_json::from_str(include_str!("../contracts/interaction-event.schema.json"))
            .expect("schema is valid JSON");
    JSONSchema::options()
        .with_draft(Draft::Draft202012)
        .compile(&schema)
        .expect("schema compiles")
}

/// Validity as a bool, mirroring the codebase's existing jsonschema-0.18 usage (`validate(..).is_ok()`).
fn valid(schema: &JSONSchema, instance: &serde_json::Value) -> bool {
    schema.validate(instance).is_ok()
}

fn sample(
    module: Module,
    verb: &str,
    class: EventClass,
    target: TargetRef,
    content: ContentRef,
    subject: Option<Uuid>,
) -> InteractionEvent {
    InteractionEvent {
        schema_version: cyberos_memory::interaction::SCHEMA_VERSION,
        event_id: Uuid::now_v7(),
        tenant_id: Uuid::new_v4(),
        subject_id: subject,
        occurred_at_ns: 1_782_950_400_000_000_000,
        module,
        event_type: verb.to_string(),
        event_class: class,
        target_ref: target,
        content_ref: content,
        session_id: Some(Uuid::new_v4()),
        trace_id: Some("0af7651916cd43dd8448eb211c80319c".to_string()),
        source_channel: SourceChannel::Web,
        attributes: serde_json::Map::new(),
    }
}

#[test]
fn every_emitted_body_validates_against_schema() {
    let schema = compile();

    // One representative event per class + each target/content arm.
    let cases = vec![
        sample(
            Module::Auth,
            "auth.signed_in",
            EventClass::Auth,
            TargetRef::Session {
                id: "jti-abc".into(),
            },
            ContentRef::None,
            Some(Uuid::now_v7()),
        ),
        sample(
            Module::Chat,
            "chat.message_created",
            EventClass::Content,
            TargetRef::Channel {
                id: "chan-1".into(),
            },
            ContentRef::pointer("chat_messages", "msg-1"),
            Some(Uuid::now_v7()),
        ),
        sample(
            Module::Proj,
            "proj.document_opened",
            EventClass::Read,
            TargetRef::Document { id: "doc-1".into() },
            ContentRef::None,
            Some(Uuid::now_v7()),
        ),
        sample(
            Module::Email,
            "email.message_received",
            EventClass::Activity,
            TargetRef::None,
            ContentRef::hash_of(b"subject+body canonical bytes"),
            Some(Uuid::now_v7()),
        ),
        sample(
            Module::Cuo,
            "cuo.dream_tick",
            EventClass::Activity,
            TargetRef::None,
            ContentRef::None,
            None, // system actor — subject_id null is allowed by the schema
        ),
        sample(
            Module::App,
            "app.tile_opened",
            EventClass::Presence,
            TargetRef::Subject {
                id: Uuid::new_v4().to_string(),
            },
            ContentRef::None,
            Some(Uuid::now_v7()),
        ),
    ];

    for ev in &cases {
        // The schema validates the PAYLOAD object (the event itself), which is what canonical_audit_body
        // nests under "payload".
        let payload = serde_json::to_value(ev).unwrap();
        if let Err(errors) = schema.validate(&payload) {
            let msgs: Vec<String> = errors.map(|e| e.to_string()).collect();
            panic!("event {} failed schema: {:?}", ev.event_type, msgs);
        }

        // And the wrapped audit-row body carries the row kind + the same payload (sanity on the wrapping).
        let body: serde_json::Value =
            serde_json::from_str(&canonical_audit_body(AUDIT_ROW_KIND, ev)).unwrap();
        assert_eq!(body["event_type"], "memory.interaction_event");
        assert!(valid(&schema, &body["payload"]));
    }
}

#[test]
fn schema_pins_version_const_1() {
    let schema = compile();
    let mut ev = serde_json::to_value(sample(
        Module::Chat,
        "chat.message_created",
        EventClass::Content,
        TargetRef::Channel { id: "c".into() },
        ContentRef::pointer("chat_messages", "m"),
        Some(Uuid::now_v7()),
    ))
    .unwrap();
    // schema_version 2 must be rejected by the const-1 contract (a v2 emitter validates against a v2
    // schema, not this one).
    ev["schema_version"] = json!(2);
    assert!(!valid(&schema, &ev), "schema_version != 1 must be rejected");
}

#[test]
fn unknown_module_is_rejected_by_schema() {
    let schema = compile();
    let mut ev = serde_json::to_value(sample(
        Module::Chat,
        "chat.message_created",
        EventClass::Content,
        TargetRef::None,
        ContentRef::None,
        Some(Uuid::now_v7()),
    ))
    .unwrap();
    ev["module"] = json!("payroll"); // not in the closed enum
    assert!(!valid(&schema, &ev), "an unknown module must be rejected");
}

#[test]
fn hash_content_ref_without_sha256_is_rejected() {
    let schema = compile();
    let mut ev = serde_json::to_value(sample(
        Module::Email,
        "email.message_received",
        EventClass::Activity,
        TargetRef::None,
        ContentRef::None,
        Some(Uuid::now_v7()),
    ))
    .unwrap();
    // A hash arm missing the required `sha256` must fail the content_ref oneOf.
    ev["content_ref"] = json!({ "kind": "hash", "bytes": 10, "preview_len": 0 });
    assert!(
        !valid(&schema, &ev),
        "a hash content_ref without sha256 must be rejected by the oneOf"
    );
}

#[test]
fn event_type_must_carry_module_dot_verb_pattern() {
    let schema = compile();
    let mut ev = serde_json::to_value(sample(
        Module::Chat,
        "chat.message_created",
        EventClass::Content,
        TargetRef::None,
        ContentRef::None,
        Some(Uuid::now_v7()),
    ))
    .unwrap();
    ev["event_type"] = json!("not-namespaced"); // lacks the '.' — fails the pattern
    assert!(
        !valid(&schema, &ev),
        "event_type must match ^[a-z]+\\.[a-z0-9_]+$"
    );
}

#[test]
fn well_formed_event_with_all_optionals_null_validates() {
    let schema = compile();
    // session_id / trace_id null, attributes omitted entirely — all permitted.
    let mut ev = serde_json::to_value(sample(
        Module::Auth,
        "auth.signed_in",
        EventClass::Auth,
        TargetRef::None,
        ContentRef::None,
        Some(Uuid::now_v7()),
    ))
    .unwrap();
    ev["session_id"] = json!(null);
    ev["trace_id"] = json!(null);
    ev.as_object_mut().unwrap().remove("attributes");
    assert!(
        valid(&schema, &ev),
        "optional fields null/absent must validate"
    );
}
