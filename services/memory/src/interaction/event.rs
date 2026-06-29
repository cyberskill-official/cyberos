//! FR-MEMORY-121 §1 #2/#16 — the one work-interaction event shape every CyberOS module emits, plus the
//! closed enums, the canonical audit-row body builder, and a misuse-resistant typed builder.
//!
//! The event is serialised as the `payload` of an aux row on `l1_audit_log` (row kind
//! `memory.interaction_event`). The body is canonical + deterministic (§1 #11): plain `serde_json`
//! serialisation gives a fixed field order (struct field order) and sorted `attributes` keys
//! (`serde_json::Map` is a `BTreeMap` in this workspace — `preserve_order` is off), so the same logical
//! interaction yields byte-identical bytes — and therefore the same chain anchor — on any host.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The frozen contract version for this FR (§1 #2). Downstream emitters pin it; a real breaking change is
/// `schema_version: 2` with its own migration note (§1 #15).
pub const SCHEMA_VERSION: u16 = 1;

/// The `l1_audit_log` row kind for an interaction-event (DEC-2704). The row-level `event_type` column
/// (FR-OBS-008) equals this constant; the interaction's own verb lives in `payload.event_type`.
pub const AUDIT_ROW_KIND: &str = "memory.interaction_event";

/// `attributes` serialised must be <= 2 KiB (§1 #10) — the anti-leak cap that stops a module smuggling
/// raw content through the bag.
pub const MAX_ATTRIBUTES_BYTES: usize = 2 * 1024;

/// The whole audit-row body must be <= 16 KiB (§1 #10) — keeps the chain + the FR-MEMORY-101 ingest cheap.
pub const MAX_BODY_BYTES: usize = 16 * 1024;

/// The emitting module. Closed set (§1 #2, #18); an unknown value cannot be constructed and is rejected
/// at emit. Serialises lower-snake to match `payload.module` in the contract.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Module {
    Auth,
    Chat,
    Proj,
    Email,
    App,
    Mcp,
    Memory,
    Ai,
    Obs,
    Cuo,
}

impl Module {
    /// The lower-snake wire string (the `<module>` prefix `event_type` must carry). Kept in lockstep with
    /// the `#[serde(rename_all = "snake_case")]` mapping so validation and serialisation agree.
    pub fn as_str(self) -> &'static str {
        match self {
            Module::Auth => "auth",
            Module::Chat => "chat",
            Module::Proj => "proj",
            Module::Email => "email",
            Module::App => "app",
            Module::Mcp => "mcp",
            Module::Memory => "memory",
            Module::Ai => "ai",
            Module::Obs => "obs",
            Module::Cuo => "cuo",
        }
    }
}

/// Coarse class for ingestion + the `op` derivation (§1 #6); stable even as `event_type` grows.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventClass {
    Auth,
    Presence,
    Content,
    Activity,
    Admin,
    Read,
}

impl EventClass {
    pub fn as_str(self) -> &'static str {
        match self {
            EventClass::Auth => "auth",
            EventClass::Presence => "presence",
            EventClass::Content => "content",
            EventClass::Activity => "activity",
            EventClass::Admin => "admin",
            EventClass::Read => "read",
        }
    }

    /// The audit-row `op` this class chains as (§1 #6): a read interaction is recorded as `view`,
    /// everything else as `put`, so reads stay distinguishable from mutations on the existing `op` column.
    pub fn audit_op(self) -> &'static str {
        match self {
            EventClass::Read => "view",
            _ => "put",
        }
    }
}

/// How the interaction reached the platform (§1 #2). Closed set.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SourceChannel {
    Web,
    Desktop,
    Mobile,
    Api,
    Cli,
    System,
    Import,
}

/// What the interaction was about — a reference, never raw content (§1 #2). Serialises as an internally
/// tagged object `{kind, id}` (or `{kind:"none"}`), matching the contract's `target_ref.oneOf`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TargetRef {
    Channel { id: String },
    Dm { id: String },
    Message { id: String },
    Issue { id: String },
    Document { id: String },
    Thread { id: String },
    Tool { id: String },
    Session { id: String },
    Subject { id: String },
    None,
}

pub use super::content_ref::ContentRef;

/// The one work-interaction event every module emits. `schema_version` is frozen at 1 for FR-MEMORY-121.
///
/// Field order here IS the canonical body field order (§1 #11) — do not reorder without a contract review.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InteractionEvent {
    /// const 1.
    pub schema_version: u16,
    /// UUIDv7 — time-sortable, dedup-safe (§1 #17).
    pub event_id: Uuid,
    pub tenant_id: Uuid,
    /// `None` ONLY for genuine system actors (§1 #8); never to anonymise a real person.
    pub subject_id: Option<Uuid>,
    /// Nanoseconds since the Unix epoch at the moment of the interaction (not ingest time).
    pub occurred_at_ns: i64,
    pub module: Module,
    /// `"<module>.<verb>"` (§1 #2). The leading segment MUST equal `module.as_str()`.
    pub event_type: String,
    pub event_class: EventClass,
    pub target_ref: TargetRef,
    pub content_ref: ContentRef,
    pub session_id: Option<Uuid>,
    pub trace_id: Option<String>,
    pub source_channel: SourceChannel,
    /// A small, bounded bag of module-specific scalars (§1 #2, #10). Serialised even when empty so the
    /// canonical body shape is stable; `serde_json::Map` keys serialise sorted in this workspace.
    #[serde(default)]
    pub attributes: serde_json::Map<String, serde_json::Value>,
}

impl InteractionEvent {
    /// Start a builder for an event of (`module`, `event_type`, `event_class`). Defaults: a fresh UUIDv7
    /// `event_id`, nil tenant/subject (caller sets them), `occurred_at_ns = 0` (call `occurred_now()`),
    /// `target_ref = None`, `content_ref = None`, `source_channel = System`, empty attributes. The
    /// builder validates §1 #9–#10 at `build()` so an emitter physically cannot construct an invalid event.
    pub fn builder(
        module: Module,
        event_type: impl Into<String>,
        event_class: EventClass,
    ) -> InteractionEventBuilder {
        InteractionEventBuilder {
            ev: InteractionEvent {
                schema_version: SCHEMA_VERSION,
                event_id: Uuid::now_v7(),
                tenant_id: Uuid::nil(),
                subject_id: None,
                occurred_at_ns: 0,
                module,
                event_type: event_type.into(),
                event_class,
                target_ref: TargetRef::None,
                content_ref: ContentRef::None,
                session_id: None,
                trace_id: None,
                source_channel: SourceChannel::System,
                attributes: serde_json::Map::new(),
            },
        }
    }

    /// The module's wire string — convenience for emit / metrics labels / path building.
    pub fn module_str(&self) -> &'static str {
        self.module.as_str()
    }

    /// The event-class wire string — convenience for metrics labels.
    pub fn event_class_str(&self) -> &'static str {
        self.event_class.as_str()
    }

    /// The subject string for path building: the subject UUID, or the nil UUID for system actors. (The
    /// row's `subject_id` column carries the same nil for system rows.)
    pub fn subject_str(&self) -> String {
        self.subject_id.unwrap_or_else(Uuid::nil).to_string()
    }

    /// The `iev/<tenant>/<module>/<subject>/<event_id>` audit path (§1 emit sub-task). Stable + low
    /// cardinality so the chain's path column stays meaningful.
    pub fn audit_path(&self) -> String {
        format!(
            "iev/{}/{}/{}/{}",
            self.tenant_id,
            self.module_str(),
            self.subject_str(),
            self.event_id
        )
    }
}

/// Build the canonical audit-row body for an interaction-event (§1 #5, #11):
/// `{"event_type":"memory.interaction_event","payload":<event>}`, serialised deterministically. The
/// row-level `event_type` is the row kind (so FR-OBS-008's generated column + the FR-APP-005 viewer keep
/// working); the interaction's own verb is `payload.event_type`. Serialisation cannot realistically fail
/// (the event is plain data), but on the impossible error we fall back to a minimal well-formed body so a
/// caller never panics on the capture path.
pub fn canonical_audit_body(row_kind: &str, ev: &InteractionEvent) -> String {
    let payload = serde_json::to_value(ev).unwrap_or(serde_json::Value::Null);
    let wrapped = serde_json::json!({ "event_type": row_kind, "payload": payload });
    serde_json::to_string(&wrapped)
        .unwrap_or_else(|_| format!("{{\"event_type\":\"{row_kind}\",\"payload\":null}}"))
}

/// A misuse-resistant front door for emitters (§1 #16). The builder makes the common case correct by
/// construction; `build()` runs the same validation `emit` re-runs (defence in depth).
#[derive(Clone, Debug)]
pub struct InteractionEventBuilder {
    ev: InteractionEvent,
}

impl InteractionEventBuilder {
    pub fn tenant(mut self, tenant_id: Uuid) -> Self {
        self.ev.tenant_id = tenant_id;
        self
    }

    pub fn subject(mut self, subject_id: Uuid) -> Self {
        self.ev.subject_id = Some(subject_id);
        self
    }

    /// Mark this a system-actor event (no subject; exempt from the consent gate, §1 #8).
    pub fn system_actor(mut self) -> Self {
        self.ev.subject_id = None;
        self
    }

    pub fn event_id(mut self, event_id: Uuid) -> Self {
        self.ev.event_id = event_id;
        self
    }

    pub fn occurred_at_ns(mut self, ns: i64) -> Self {
        self.ev.occurred_at_ns = ns;
        self
    }

    /// Stamp `occurred_at_ns` with the current wall clock (ns since epoch).
    pub fn occurred_now(mut self) -> Self {
        self.ev.occurred_at_ns = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
        self
    }

    pub fn target(mut self, target_ref: TargetRef) -> Self {
        self.ev.target_ref = target_ref;
        self
    }

    pub fn content(mut self, content_ref: ContentRef) -> Self {
        self.ev.content_ref = content_ref;
        self
    }

    pub fn session(mut self, session_id: Uuid) -> Self {
        self.ev.session_id = Some(session_id);
        self
    }

    pub fn trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.ev.trace_id = Some(trace_id.into());
        self
    }

    pub fn source(mut self, source_channel: SourceChannel) -> Self {
        self.ev.source_channel = source_channel;
        self
    }

    /// Set a single bounded attribute scalar. Raw content MUST NOT go here (§1 #3, #10) — the 2 KiB cap is
    /// enforced at `build()`.
    pub fn attribute(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.ev.attributes.insert(key.into(), value);
        self
    }

    /// Replace the whole attributes bag.
    pub fn attributes(mut self, attributes: serde_json::Map<String, serde_json::Value>) -> Self {
        self.ev.attributes = attributes;
        self
    }

    /// Validate (§1 #9, #10) and return the event, or the same [`super::emit::EmitError::Invalid`] the free
    /// `emit` would raise. This is the misuse-resistant boundary: an unknown-prefixed `event_type`, a wrong
    /// `schema_version`, an oversize attributes bag, or an oversize body are caught here, before any emit.
    /// Sharing the error type with `emit` means an emitter that builds then emits handles one error shape.
    pub fn build(self) -> Result<InteractionEvent, super::emit::EmitError> {
        super::emit::validate(&self.ev)?;
        Ok(self.ev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nil_event(class: EventClass, verb: &str, module: Module) -> InteractionEvent {
        InteractionEvent {
            schema_version: SCHEMA_VERSION,
            event_id: Uuid::nil(),
            tenant_id: Uuid::nil(),
            subject_id: Some(Uuid::nil()),
            occurred_at_ns: 0,
            module,
            event_type: verb.to_string(),
            event_class: class,
            target_ref: TargetRef::None,
            content_ref: ContentRef::None,
            session_id: None,
            trace_id: None,
            source_channel: SourceChannel::Web,
            attributes: serde_json::Map::new(),
        }
    }

    #[test]
    fn canonical_body_is_deterministic() {
        let ev = nil_event(EventClass::Content, "chat.message_created", Module::Chat);
        assert_eq!(
            canonical_audit_body(AUDIT_ROW_KIND, &ev),
            canonical_audit_body(AUDIT_ROW_KIND, &ev)
        );
    }

    #[test]
    fn canonical_body_wraps_with_row_kind_and_payload() {
        let ev = nil_event(EventClass::Auth, "auth.signed_in", Module::Auth);
        let body = canonical_audit_body(AUDIT_ROW_KIND, &ev);
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(v["event_type"], "memory.interaction_event");
        assert_eq!(v["payload"]["event_type"], "auth.signed_in");
        assert_eq!(v["payload"]["module"], "auth");
        assert_eq!(v["payload"]["schema_version"], 1);
    }

    #[test]
    fn attributes_keys_serialise_sorted_for_determinism() {
        // Insertion order differs from sorted order; the canonical body must be identical regardless.
        let mut a = nil_event(EventClass::Activity, "proj.issue_assigned", Module::Proj);
        a.attributes.insert("zeta".into(), serde_json::json!(1));
        a.attributes.insert("alpha".into(), serde_json::json!(2));
        let mut b = nil_event(EventClass::Activity, "proj.issue_assigned", Module::Proj);
        b.attributes.insert("alpha".into(), serde_json::json!(2));
        b.attributes.insert("zeta".into(), serde_json::json!(1));
        assert_eq!(
            canonical_audit_body(AUDIT_ROW_KIND, &a),
            canonical_audit_body(AUDIT_ROW_KIND, &b)
        );
    }

    #[test]
    fn round_trips_through_serde_with_all_fields() {
        let ev = nil_event(EventClass::Content, "chat.message_created", Module::Chat);
        let json = serde_json::to_string(&ev).unwrap();
        let back: InteractionEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(back.schema_version, 1);
        assert_eq!(back.event_type, "chat.message_created");
        assert_eq!(back.module, Module::Chat);
        assert_eq!(back.event_class, EventClass::Content);
    }

    #[test]
    fn op_derives_from_class() {
        assert_eq!(EventClass::Read.audit_op(), "view");
        assert_eq!(EventClass::Content.audit_op(), "put");
        assert_eq!(EventClass::Auth.audit_op(), "put");
    }

    #[test]
    fn builder_rejects_unknown_module_prefix() {
        let r = InteractionEvent::builder(Module::Chat, "proj.document_opened", EventClass::Read)
            .tenant(Uuid::nil())
            .subject(Uuid::nil())
            .build();
        assert!(
            r.is_err(),
            "event_type without the module prefix must not build"
        );
    }

    #[test]
    fn builder_accepts_well_formed_event() {
        let r =
            InteractionEvent::builder(Module::Chat, "chat.message_created", EventClass::Content)
                .tenant(Uuid::nil())
                .subject(Uuid::nil())
                .occurred_now()
                .target(TargetRef::Message { id: "msg-1".into() })
                .content(ContentRef::pointer("chat_messages", "msg-1"))
                .source(SourceChannel::Web)
                .build();
        assert!(r.is_ok());
    }

    #[test]
    fn builder_rejects_oversize_attributes() {
        let r =
            InteractionEvent::builder(Module::Chat, "chat.message_created", EventClass::Content)
                .tenant(Uuid::nil())
                .subject(Uuid::nil())
                .attribute("blob", serde_json::json!("x".repeat(3000)))
                .build();
        assert!(r.is_err());
    }

    #[test]
    fn audit_path_is_stable_and_low_cardinality() {
        let ev = nil_event(EventClass::Auth, "auth.signed_in", Module::Auth);
        assert_eq!(
            ev.audit_path(),
            "iev/00000000-0000-0000-0000-000000000000/auth/00000000-0000-0000-0000-000000000000/00000000-0000-0000-0000-000000000000"
        );
    }
}
