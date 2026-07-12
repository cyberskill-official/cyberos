---
id: FR-MEMORY-121
title: "interaction-event schema & contract — the one work-interaction event shape (event_id, subject, module, event_type, target_ref, content_ref) every module emits into l1_audit_log, versioned + RLS + emit API; the single BRAIN capture primitive"
module: MEMORY
priority: MUST
status: implementing
verify: T
phase: P1
milestone: P1 · BRAIN capture · slice 1
slice: 1
owner: Stephen Cheng (CDO)
created: 2026-06-29
shipped: null
memory_chain_hash: null
related_frs: [FR-MEMORY-101, FR-MEMORY-108, FR-MEMORY-122, FR-MEMORY-123, FR-EVAL-001, FR-AUTH-002, FR-AUTH-004, FR-CHAT-101, FR-OBS-008]
depends_on: [FR-MEMORY-101, FR-EVAL-001]
blocks: [FR-MEMORY-122, FR-MEMORY-123]

source_pages:
  - docs/strategy/cyberos-brain-evaluation-plan.md#phase-1-capture
source_decisions:
  - DEC-2700 (one interaction-event schema, every module emits it — chat, sign-in, presence, module-open, task/doc/IP activity — from the moment a person logs in; Stephen 2026-06-29)
  - DEC-2701 (platform work-interactions ONLY — no keystroke logging, no screen capture, no private life; the schema carries pointers/hashes (content_ref), not raw sensitive content, wherever practical)
  - DEC-2702 (capture is gated on the FR-EVAL-001 monitoring-notice acknowledgment — a subject that has not acknowledged emits no interaction-event; this FR defines the gate hook, FR-MEMORY-122 wires it)
  - DEC-2703 (the interaction-event is an aux audit row on the existing hash-chained l1_audit_log — one capture substrate, not a second store; it reuses cyberos-audit-chain byte-for-byte so memory's reconcile + FR-MEMORY-101 ingest accept it unchanged)
  - "DEC-2704 (the schema is versioned: `iev` schema_version is a field, the row kind is `memory.interaction_event`, and the contract is published so AUTH/CHAT/PROJ/EMAIL/APP/MCP depend on a frozen shape, not on each other)"

ai_authorship: assisted
eu_ai_act_risk_class: limited
language: rust 1.81
service: cyberos/services/memory/
new_files:
  - services/memory/src/interaction/mod.rs
  - services/memory/src/interaction/event.rs
  - services/memory/src/interaction/emit.rs
  - services/memory/src/interaction/content_ref.rs
  - services/memory/src/interaction/consent_gate.rs
  - services/memory/migrations/0005_interaction_event.sql
  - services/memory/contracts/interaction-event.schema.json
  - services/memory/tests/interaction_event_test.rs
  - services/memory/tests/interaction_event_rls_test.rs
  - services/memory/tests/interaction_event_contract_test.rs
modified_files:
  - services/memory/src/lib.rs                       # pub mod interaction
  - services/shared/cyberos-audit-chain/src/lib.rs   # emit_genesis_with_op: allow op='view' for read-only interaction events
allowed_tools:
  - file_read: services/memory/**
  - file_read: services/shared/cyberos-audit-chain/**
  - file_write: services/memory/{src,tests,migrations,contracts}/**
  - file_write: services/shared/cyberos-audit-chain/src/**
  - bash: cd services/memory && cargo test interaction
  - bash: cd services/shared/cyberos-audit-chain && cargo test
disallowed_tools:
  - add a second capture store or table for interaction events (per DEC-2703 — they are aux rows on l1_audit_log; a parallel store would split the chain and break reconcile)
  - put raw sensitive content (message bodies, document text, email bodies) in the event payload where a content_ref pointer/hash suffices (per DEC-2701)
  - emit an interaction-event for a subject that has not acknowledged the FR-EVAL-001 notice (per DEC-2702 — the consent gate is mandatory)
  - break the frozen field set without bumping `schema_version` (per DEC-2704 — downstream emitters pin the version)

effort_hours: 9
sub_tasks:
  - "0.5h: 0005_interaction_event.sql — generated columns (iev_subject, iev_module, iev_event_type) over body + partial indexes; no new table"
  - "1.0h: interaction/event.rs — InteractionEvent struct + Module/SourceChannel/EventClass enums + canonical JSON body (frozen field order)"
  - "1.0h: interaction/content_ref.rs — ContentRef::pointer{store,id} | ContentRef::hash{sha256,bytes,preview_len} | ContentRef::none; redaction-safe Display"
  - "1.0h: interaction/emit.rs — emit(pool, &InteractionEvent) -> seq via cyberos-audit-chain; op derived from EventClass (read→view, else put); path scheme iev/<tenant>/<module>/<subject>/<event_id>"
  - "1.0h: interaction/consent_gate.rs — has_acknowledged(pool, tenant, subject) reading the FR-EVAL-001 notice-ack ledger; emit() returns Skipped{reason:consent_not_acknowledged} when false"
  - "1.0h: contracts/interaction-event.schema.json — JSON Schema (draft 2020-12) for the row kind, frozen required set, schema_version const, additive evolution rules"
  - "0.5h: cyberos-audit-chain emit_genesis_with_op — accept op so read-only events chain as 'view'"
  - "0.5h: OTel metrics — memory_interaction_events_total{module,event_class,outcome}; memory_interaction_event_emit_seconds; memory_interaction_consent_skipped_total{module}"
  - "1.0h: interaction_event_test.rs — shape, canonical-body determinism, op derivation, content_ref redaction, chain-anchor verifies under reconcile"
  - "0.5h: interaction_event_rls_test.rs — tenant A's interaction rows invisible to tenant B via the l1_audit_log RLS read path"
  - "1.0h: interaction_event_contract_test.rs — every emitted body validates against the JSON Schema; schema_version mismatch rejected; unknown module rejected"
risk_if_skipped: "Without one frozen event shape, every module (AUTH, CHAT, PROJ, EMAIL, APP, MCP) invents its own audit payload, and the BRAIN ingestion (FR-MEMORY-123) and the evaluation engine (FR-EVAL-003) have to special-case each one forever — the 'record all interactions' goal degrades into per-module archaeology. Without content_ref discipline, raw message/document/email bodies land verbatim in a years-retained audit chain (a PDPD minimisation breach). Without the consent gate defined here, capture can start before the FR-EVAL-001 notice is acknowledged — the exact covert-monitoring failure the governance phase exists to prevent. Without versioning, the first schema change silently breaks every downstream emitter that pinned the old shape."
---

## §1 — Description (BCP-14 normative)

This FR defines **one** interaction-event shape that every CyberOS module emits for every platform work-interaction, and the API + contract + storage + access rules around it. It is the single BRAIN capture primitive (Phase 1 of `docs/strategy/cyberos-brain-evaluation-plan.md`). It captures emitters for no module itself — FR-MEMORY-122 wires AUTH + CHAT and defines the emitter contract for the rest. Each piece:

1. **MUST** define the row kind `memory.interaction_event` and enumerate it in `services/memory/contracts/interaction-event.schema.json` (JSON Schema draft 2020-12), alongside the existing aux audit kinds (`memory.precondition_failed`, `memory.acl_denied`, `memory.status_overridden`, `memory.awh_gate_result`). The schema is the published contract other modules depend on.
2. **MUST** define the event field set, frozen for `schema_version: 1`:
    - `schema_version` (int, const `1` for this FR) — downstream emitters pin it.
    - `event_id` (UUIDv7) — globally unique; v7 so it is time-sortable and dedup-safe across replay.
    - `tenant_id` (UUID) — the emitting subject's tenant; drives RLS.
    - `subject_id` (UUID) — the person who acted (FR-AUTH-002 subject); the unit of evaluation. `null` only for genuine system actors (e.g. a scheduled job), never to hide a person.
    - `occurred_at_ns` (int64) — nanoseconds since epoch at the moment of the interaction (not ingest time).
    - `module` (enum) — the emitting module: one of `auth | chat | proj | email | app | mcp | memory | ai | obs | cuo`. Closed set; unknown values rejected at emit (§1 #9).
    - `event_type` (string, namespaced `"<module>.<verb>"`, e.g. `chat.message_created`, `auth.signed_in`, `proj.issue_assigned`) — the specific interaction. Free-form within the `module.` prefix so modules add verbs without a schema bump.
    - `event_class` (enum) — `auth | presence | content | activity | admin | read`. Coarse class for ingestion + the `op` derivation (§1 #6); stable even as `event_type` grows.
    - `target_ref` (object, nullable) — what the interaction was about: `{kind, id}` where `kind` ∈ `channel | dm | message | issue | document | thread | tool | session | subject | none` and `id` is the target's UUID or stable id. Not raw content — a reference.
    - `content_ref` (object, nullable) — a pointer or hash of any content the interaction carried, per §1 #4. NEVER the raw body where a pointer/hash suffices.
    - `session_id` (UUID, nullable) — the AUTH session (FR-AUTH-004 jti-derived) the action happened in.
    - `trace_id` (string, nullable) — the W3C traceparent trace-id (FR-OBS-003) so an interaction correlates with its request span.
    - `source_channel` (enum) — `web | desktop | mobile | api | cli | system | import` — how the interaction reached the platform.
    - `attributes` (object, optional) — a small, bounded bag of module-specific scalars (e.g. `{prev_status, new_status}` for a status change). MUST NOT carry raw content; MUST be ≤ 2 KiB serialised (§1 #10).
3. **MUST** carry no raw sensitive content in the row where a reference suffices (DEC-2701). Specifically: message bodies, document/IP text, email subjects+bodies, and file contents are referenced via `content_ref`, never inlined. Privacy-safe digests follow the AUTH `*_hash16` precedent (e.g. an email address that must appear is a 16-hex SHA-256 prefix, never plaintext).
4. **MUST** define `content_ref` as a closed union:
    - `pointer` — `{kind:"pointer", store, id}` where `store` ∈ `chat_messages | proj_documents | email_objects | memory | attachments` and `id` is the row/object id. The raw content stays in the owning store (e.g. chat's own DB), under that store's own RLS; the BRAIN holds only the pointer.
    - `hash` — `{kind:"hash", sha256, bytes, preview_len}` where `sha256` is the lowercase-hex digest of the canonical content, `bytes` is its length, and `preview_len` is `0` by default (a non-zero preview is opt-in per module and MUST be a short, non-sensitive prefix only).
    - `none` — `{kind:"none"}` for interactions with no content (e.g. a sign-in, a presence change, opening a module).
5. **MUST** chain each interaction-event into the existing hash-chained `l1_audit_log` via the shared `cyberos-audit-chain` writer (DEC-2703) — byte-identical anchor `SHA-256(prev_hash_hex ‖ body)` so the memory reconcile invariant (FR-MEMORY-101) and the layer-2 ingest accept it with no special case. The event body is the canonical-JSON serialisation of §1 #2 with `"event_type"` set to the row kind `memory.interaction_event` at the audit-row level and the interaction's own `event_type` inside the payload (so the existing FR-OBS-008 `event_type` generated column and the FR-APP-005 viewer keep working).
6. **MUST** derive the audit-row `op` from `event_class`: `read → 'view'`; everything else → `'put'`. This requires `cyberos-audit-chain::emit_genesis_with_op` (an `op` parameter; the existing `emit_genesis` stays as a `'put'` shim). Read interactions (e.g. opening a document, viewing a channel) are recorded as `view` so they are distinguishable from mutations and so the chain's `op` enum stays meaningful.
7. **MUST** expose the emit API in `services/memory/src/interaction/emit.rs`:
    ```rust
    pub async fn emit(pool: &PgPool, ev: &InteractionEvent) -> Result<EmitOutcome, EmitError>;
    ```
    returning `EmitOutcome::Recorded { seq }` on success or `EmitOutcome::Skipped { reason }` when the consent gate (§1 #8) blocks it. `emit` is best-effort from the caller's perspective: it MUST NOT make the calling interaction (a sign-in, a message send) fail if the audit pool is unreachable — it returns `EmitError` and the caller logs + swallows, exactly as AUTH's `emit_token_issued` and chat's `audit::emit` already do.
8. **MUST** gate emit on the FR-EVAL-001 monitoring-notice acknowledgment (DEC-2702). `consent_gate::has_acknowledged(pool, tenant_id, subject_id)` reads the notice-ack ledger that FR-EVAL-001 owns; when the subject has not acknowledged the current notice version, `emit` returns `Skipped { reason: ConsentNotAcknowledged }` and writes NO row. System actors (`subject_id = null`) are exempt (there is no person to notify). The gate result MUST be cache-safe for a bounded TTL (§1 #12) so it does not add a DB round-trip to every interaction.
9. **MUST** validate every event before it is written: `module` ∈ the closed enum; `event_type` begins with `"<module>."`; `event_class` ∈ the closed enum; `content_ref` is one of the three union arms; `attributes` serialises to ≤ 2 KiB; `schema_version == 1`. A validation failure returns `EmitError::Invalid` and writes no row (a malformed interaction-event must never enter the chain).
10. **MUST** bound the row size: `attributes` ≤ 2 KiB serialised and the whole body ≤ 16 KiB. Over-size → `EmitError::Invalid`. This keeps the audit chain and the FR-MEMORY-101 ingest cheap and stops a module from smuggling raw content through `attributes`.
11. **MUST** make the event body canonical and deterministic: a fixed field order, no insignificant whitespace, UTF-8, so the same logical interaction produces the same bytes (and the same chain anchor) on any host. The canonicaliser is shared with the rest of the memory chain (matches `chain_anchor::canonicalise`).
12. **MUST** cache the consent-gate verdict per `(tenant_id, subject_id)` for ≤ 60 s in-process, so a burst of interactions from one signed-in person does not issue a consent-ledger query per event. A revocation (subject withdraws acknowledgment) takes effect within the TTL; the 60 s window is the documented bound.
13. **MUST** emit OTel metrics:
    - `memory_interaction_events_total{module, event_class, outcome}` (counter; outcome ∈ `recorded | skipped_consent | invalid | emit_error`).
    - `memory_interaction_event_emit_seconds{module}` (histogram).
    - `memory_interaction_consent_skipped_total{module}` (counter — visibility into how much is suppressed by the gate; a sudden spike means a notice-version bump locked people out).
    - `memory_interaction_event_body_bytes{module}` (histogram — watches for content leaking through `attributes`).
14. **MUST** be tenant-scoped on read via the existing `l1_audit_log` RLS path (FR-AUTH-003 pattern); interaction-events inherit it because they ARE `l1_audit_log` rows. There is no second table and therefore no second RLS policy to drift.
15. **MUST** publish the contract as a versioned, frozen artifact: `interaction-event.schema.json` carries `schema_version` const `1` and an explicit `additive-only` evolution rule (new optional fields and new `event_type` verbs are allowed without a bump; removing/retyping a field or changing the required set requires `schema_version: 2` and a migration note). Downstream emitter FRs (FR-MEMORY-122 and beyond) depend on this file, not on each other.
16. **MUST** provide a typed builder so emitters cannot construct an invalid event: `InteractionEvent::builder(module, event_type, event_class).subject(..).occurred_now().target(..).content(ContentRef::..).source(..).build()` validates §1 #9–#10 at `build()` and returns `Result`. The free `emit` re-validates (defence in depth) but the builder is the ergonomic, misuse-resistant front door for FR-MEMORY-122's emitters.
17. **MUST** make `event_id` idempotent across replay: re-emitting the same `event_id` is a no-op at ingest (FR-MEMORY-101 already UPSERTs on the chain row; the BRAIN layer-2 dedups on `event_id`). An emitter that retries after a transient `EmitError` reuses the same `event_id` so a retry never double-counts an interaction.
18. **MUST** document, in the schema and in §11, the closed `module`/`event_class`/`source_channel`/`target_ref.kind`/`content_ref.kind` enums as the cardinality-bounded vocabulary the evaluation engine (FR-EVAL-003) and the BRAIN ingestion (FR-MEMORY-123) build against, so those FRs index on stable, low-cardinality dimensions rather than free strings.

---

## §2 — Why this design (rationale for humans)

**Why one shape, not per-module payloads (DEC-2700)?** The goal is "record all interactions" and then evaluate a person across all of them. If chat, auth, and proj each shape their audit rows differently, the ingestion worker and the evaluation engine carry a growing pile of per-module adapters, and a new module means new special-casing everywhere downstream. One frozen shape means FR-MEMORY-123 ingests once and FR-EVAL-003 evaluates once; a new module just emits the same event.

**Why aux rows on `l1_audit_log`, not a new store (DEC-2703)?** The brain plan is explicit that the foundations exist and the work is additive. The hash-chained audit log is already the tamper-evident system of record; the layer-2 pipeline already tails it; AUTH, CHAT, and OBS already write to it through `cyberos-audit-chain`. Interaction-events are the same kind of fact, so they ride the same chain. A second capture store would fork the system of record, double the RLS surface, and break the single-reconcile invariant for no benefit.

**Why `content_ref` instead of raw bodies (DEC-2701)?** Two reasons. Privacy: the audit chain is retained for years; inlining message/document/email bodies there is exactly the data-maximisation the PDPD framing warns against, and it would duplicate content the owning store already holds under its own RLS. Integrity: the BRAIN should point at the canonical content (chat's message row, proj's document) so there is one source of truth, not a stale copy. The `hash` arm covers the case where the content lives nowhere durable but we still need "did it change / are these the same" — without keeping the content.

**Why a consent gate in the capture primitive (DEC-2702)?** Governance comes before capture in the plan's phase order for legal and trust reasons. Putting the gate in the one emit path means there is no way for a module to capture a person before that person has acknowledged the monitoring notice — the property is enforced at the primitive, not left to each emitter to remember. System actors are exempt because there is no person behind them to notify.

**Why version the schema and freeze the field set (DEC-2704)?** Six modules will depend on this shape. If the shape can change under them, the first change is a silent break. A `schema_version` const plus an additive-only rule lets the vocabulary grow (new verbs, new optional attributes) without breaking pinned emitters, and forces the breaking changes to be loud (a version bump + migration note).

**Why UUIDv7 for `event_id` (§1 #2)?** Time-sortable ids make the BRAIN's recency queries and replay dedup cheap, and they are collision-safe across the many emitters producing events concurrently. v4 would work for uniqueness but loses the free time ordering the ingestion and evaluation paths want.

**Why derive `op` from `event_class` (§1 #6)?** The chain's `op` enum (`put | move | delete | view`) already distinguishes mutations from reads. Recording "Alice opened this document" as `view` and "Alice edited it" as `put` keeps that distinction meaningful and lets read-versus-write analyses run off the existing column. Reads are interactions too — opening a channel is signal — but they must not look like mutations.

**Why best-effort emit (§1 #7)?** A sign-in or a message send must not fail because the audit pool is briefly unreachable. AUTH and CHAT already treat their audit writes as best-effort and let an OBS alarm catch a sustained drop. The interaction-event emit follows the same rule so capture is never on the critical path of the thing it is capturing.

**Why a typed builder plus re-validation (§1 #16)?** The builder makes the common case correct by construction (an emitter physically cannot set an unknown module or inline a raw body), which is what FR-MEMORY-122's many call sites need. The free `emit` re-validates anyway so a hand-built event or a future caller that bypasses the builder still cannot write a malformed row.

**Why bound size and watch body bytes (§1 #10, #13)?** The cheapest way to defeat `content_ref` discipline is to dump raw text into `attributes`. A 2 KiB attributes cap plus a body-bytes histogram makes that both impossible at the limit and visible below it — if one module's bodies suddenly grow, the metric shows it.

**Why cache the consent verdict (§1 #12)?** A signed-in person generates a stream of interactions; a consent-ledger query per event would put a DB round-trip on every keystroke-adjacent action. A 60 s in-process cache makes the gate effectively free while keeping revocation latency to a documented, short bound.

---

## §3 — API contract

### Migration

```sql
-- services/memory/migrations/0005_interaction_event.sql
--
-- No new table: interaction-events are aux rows on l1_audit_log (DEC-2703). This migration adds
-- generated columns + partial indexes so the BRAIN ingestion (FR-MEMORY-123) and the console viewer
-- (FR-APP-005) can scan interaction-events by subject/module/event_type without parsing JSON per row.
-- The audit-row `event_type` column (FR-OBS-008, migration 0004) already equals 'memory.interaction_event'
-- for these rows; these columns reach INTO the payload.

-- The interaction's own module, pulled from body.payload.module.
ALTER TABLE l1_audit_log
    ADD COLUMN iev_module TEXT
    GENERATED ALWAYS AS ((body::jsonb -> 'payload' ->> 'module')) STORED;

-- The interaction's subject, pulled from body.payload.subject_id (distinct from the row's subject_id,
-- which cyberos-audit-chain sets to the same value; this generated column is the typed, indexable view).
ALTER TABLE l1_audit_log
    ADD COLUMN iev_event_type TEXT
    GENERATED ALWAYS AS ((body::jsonb -> 'payload' ->> 'event_type')) STORED;

ALTER TABLE l1_audit_log
    ADD COLUMN iev_event_class TEXT
    GENERATED ALWAYS AS ((body::jsonb -> 'payload' ->> 'event_class')) STORED;

-- Partial indexes scoped to interaction-event rows only (keeps them small; other audit kinds are excluded).
CREATE INDEX l1_iev_subject_idx
    ON l1_audit_log (tenant_id, subject_id, ts_ns DESC)
    WHERE event_type = 'memory.interaction_event';

CREATE INDEX l1_iev_module_class_idx
    ON l1_audit_log (tenant_id, iev_module, iev_event_class, ts_ns DESC)
    WHERE event_type = 'memory.interaction_event';

-- Dedup guard for replay (§1 #17): the interaction's event_id is unique per tenant.
CREATE UNIQUE INDEX l1_iev_event_id_uq
    ON l1_audit_log (tenant_id, (body::jsonb -> 'payload' ->> 'event_id'))
    WHERE event_type = 'memory.interaction_event';
```

### Rust — the event type + enums

```rust
// services/memory/src/interaction/event.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Module {
    Auth, Chat, Proj, Email, App, Mcp, Memory, Ai, Obs, Cuo,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventClass {
    Auth, Presence, Content, Activity, Admin, Read,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SourceChannel {
    Web, Desktop, Mobile, Api, Cli, System, Import,
}

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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ContentRef {
    /// The raw content lives in the owning store under its own RLS; the BRAIN holds only this pointer.
    Pointer { store: String, id: String },
    /// No durable store; keep a digest only. `preview_len` is 0 unless a module opts into a short,
    /// non-sensitive prefix.
    Hash { sha256: String, bytes: u64, preview_len: u32 },
    /// The interaction carried no content (sign-in, presence, module-open).
    None,
}

/// The one work-interaction event every module emits. `schema_version` is frozen at 1 for FR-MEMORY-121.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InteractionEvent {
    pub schema_version: u16,          // const 1
    pub event_id: Uuid,               // UUIDv7
    pub tenant_id: Uuid,
    pub subject_id: Option<Uuid>,     // None only for system actors
    pub occurred_at_ns: i64,
    pub module: Module,
    pub event_type: String,           // "<module>.<verb>"
    pub event_class: EventClass,
    pub target_ref: TargetRef,
    pub content_ref: ContentRef,
    pub session_id: Option<Uuid>,
    pub trace_id: Option<String>,
    pub source_channel: SourceChannel,
    #[serde(default)]
    pub attributes: serde_json::Map<String, serde_json::Value>,
}

pub const SCHEMA_VERSION: u16 = 1;
pub const AUDIT_ROW_KIND: &str = "memory.interaction_event";
```

### Rust — the emit path + consent gate

```rust
// services/memory/src/interaction/emit.rs
use crate::interaction::event::{InteractionEvent, EventClass, AUDIT_ROW_KIND, SCHEMA_VERSION};
use sqlx::PgPool;

#[derive(Debug)]
pub enum EmitOutcome { Recorded { seq: i64 }, Skipped { reason: SkipReason } }
#[derive(Debug)] pub enum SkipReason { ConsentNotAcknowledged }
#[derive(Debug, thiserror::Error)]
pub enum EmitError {
    #[error("invalid interaction-event: {0}")] Invalid(String),
    #[error(transparent)] Db(#[from] sqlx::Error),
}

pub async fn emit(pool: &PgPool, ev: &InteractionEvent) -> Result<EmitOutcome, EmitError> {
    validate(ev)?;                                            // §1 #9, #10
    // §1 #8: consent gate (system actors exempt).
    if let Some(subject) = ev.subject_id {
        if !crate::interaction::consent_gate::has_acknowledged(pool, ev.tenant_id, subject).await? {
            metrics::counter!("memory_interaction_consent_skipped_total",
                "module" => ev.module_str()).increment(1);
            return Ok(EmitOutcome::Skipped { reason: SkipReason::ConsentNotAcknowledged });
        }
    }
    // §1 #5/#6: chain as an aux audit row; op from event_class.
    let op = if ev.event_class == EventClass::Read { "view" } else { "put" };
    let body = canonical_audit_body(AUDIT_ROW_KIND, ev);      // §1 #11; {"event_type":<kind>,"payload":<ev>}
    let path = format!("iev/{}/{}/{}/{}",
        ev.tenant_id, ev.module_str(), ev.subject_str(), ev.event_id);
    let subject = ev.subject_id.unwrap_or(uuid::Uuid::nil());
    let seq = cyberos_audit_chain::emit_genesis_with_op(
        pool, ev.tenant_id, subject, op, &path, &body,
    ).await?;
    metrics::counter!("memory_interaction_events_total",
        "module" => ev.module_str(), "event_class" => ev.event_class_str(),
        "outcome" => "recorded").increment(1);
    Ok(EmitOutcome::Recorded { seq })
}

fn validate(ev: &InteractionEvent) -> Result<(), EmitError> {
    if ev.schema_version != SCHEMA_VERSION {
        return Err(EmitError::Invalid(format!("schema_version {} != {}", ev.schema_version, SCHEMA_VERSION)));
    }
    if !ev.event_type.starts_with(&format!("{}.", ev.module_str())) {
        return Err(EmitError::Invalid(format!("event_type '{}' lacks module prefix", ev.event_type)));
    }
    let attrs_len = serde_json::to_vec(&ev.attributes).map(|v| v.len()).unwrap_or(usize::MAX);
    if attrs_len > 2 * 1024 { return Err(EmitError::Invalid("attributes > 2KiB".into())); }
    Ok(())
}
```

```rust
// services/memory/src/interaction/consent_gate.rs
use sqlx::PgPool;
use uuid::Uuid;

/// Reads the FR-EVAL-001 notice-acknowledgment ledger. A subject that has acknowledged the *current*
/// notice version returns true. Cached in-process for ≤ 60s (§1 #12) keyed by (tenant, subject).
pub async fn has_acknowledged(pool: &PgPool, tenant: Uuid, subject: Uuid) -> Result<bool, sqlx::Error> {
    if let Some(v) = cache::get(tenant, subject) { return Ok(v); }
    // FR-EVAL-001 owns `eval_monitoring_ack (tenant_id, subject_id, notice_version, acknowledged_at)`
    // and `eval_current_notice (tenant_id, notice_version)`. Until FR-EVAL-001 lands, this resolves
    // against a stub that returns the operator-seeded default (documented in §11).
    let ok: bool = sqlx::query_scalar(
        "SELECT EXISTS (
           SELECT 1 FROM eval_monitoring_ack a
           JOIN eval_current_notice n
             ON n.tenant_id = a.tenant_id AND n.notice_version = a.notice_version
          WHERE a.tenant_id = $1 AND a.subject_id = $2)",
    ).bind(tenant).bind(subject).fetch_one(pool).await.unwrap_or(false);
    cache::put(tenant, subject, ok);
    Ok(ok)
}
```

### Shared crate — op-aware genesis emit

```rust
// services/shared/cyberos-audit-chain/src/lib.rs  (added; emit_genesis becomes a 'put' shim over this)
pub async fn emit_genesis_with_op(
    pool: &PgPool, tenant_id: Uuid, subject_id: Uuid,
    op: &str, path: &str, body: &str,
) -> Result<i64, sqlx::Error> {
    debug_assert!(matches!(op, "put" | "move" | "delete" | "view"));
    let anchor = chain_anchor(None, body);
    let ts_ns = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let row: (i64,) = sqlx::query_as(
        "INSERT INTO l1_audit_log
            (tenant_id, subject_id, op, path, body, prev_hash_hex, chain_anchor_hex, ts_ns)
         VALUES ($1, $2, $3, $4, $5, NULL, $6, $7)
         RETURNING seq",
    )
    .bind(tenant_id).bind(subject_id).bind(op).bind(path).bind(body)
    .bind(&anchor).bind(ts_ns)
    .fetch_one(pool).await?;
    Ok(row.0)
}
```

### Contract artifact (excerpt)

```json
// services/memory/contracts/interaction-event.schema.json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://cyberos.cyberskill.world/contracts/interaction-event/v1",
  "title": "CyberOS interaction-event",
  "x-evolution": "additive-only: new optional fields and new `<module>.<verb>` event_type values are allowed without a version bump; removing or retyping a field, or changing `required`, requires schema_version 2 + a migration note",
  "type": "object",
  "required": ["schema_version","event_id","tenant_id","occurred_at_ns","module","event_type","event_class","target_ref","content_ref","source_channel"],
  "properties": {
    "schema_version": { "const": 1 },
    "event_id":       { "type": "string", "format": "uuid" },
    "tenant_id":      { "type": "string", "format": "uuid" },
    "subject_id":     { "type": ["string","null"], "format": "uuid" },
    "occurred_at_ns": { "type": "integer" },
    "module":         { "enum": ["auth","chat","proj","email","app","mcp","memory","ai","obs","cuo"] },
    "event_type":     { "type": "string", "pattern": "^[a-z]+\\.[a-z0-9_]+$" },
    "event_class":    { "enum": ["auth","presence","content","activity","admin","read"] },
    "source_channel": { "enum": ["web","desktop","mobile","api","cli","system","import"] },
    "content_ref":    { "oneOf": [
      { "type":"object","required":["kind","store","id"],"properties":{"kind":{"const":"pointer"}}},
      { "type":"object","required":["kind","sha256","bytes"],"properties":{"kind":{"const":"hash"}}},
      { "type":"object","required":["kind"],"properties":{"kind":{"const":"none"}}}
    ]}
  }
}
```

---

## §4 — Acceptance criteria

1. **Row kind enumerated** — `interaction-event.schema.json` lists `memory.interaction_event` and validates a well-formed event (AC for §1 #1).
2. **Frozen field set serialises** — an `InteractionEvent` round-trips through serde with all §1 #2 fields; `schema_version == 1` (AC for §1 #2).
3. **Raw content rejected** — building an event with a >2 KiB `attributes` blob → `EmitError::Invalid`; nothing written (AC for §1 #3, #10).
4. **content_ref union enforced** — a `Pointer{store:"chat_messages", id}` validates; a hash arm without `sha256` is rejected by the schema (AC for §1 #4).
5. **Chains into l1_audit_log** — `emit` of a valid event inserts one `l1_audit_log` row with `event_type='memory.interaction_event'` and a `chain_anchor_hex` that re-verifies under the memory reconcile (AC for §1 #5).
6. **op derived from class** — a `Read`-class event writes `op='view'`; a `Content`-class event writes `op='put'` (AC for §1 #6).
7. **emit is best-effort** — with the audit pool down, `emit` returns `EmitError::Db` and does NOT panic; the caller can swallow it (AC for §1 #7).
8. **Consent gate blocks** — for a subject with no acknowledgment row, `emit` returns `Skipped{ConsentNotAcknowledged}` and writes zero rows; `memory_interaction_consent_skipped_total` increments (AC for §1 #8).
9. **Consent gate passes** — after seeding an acknowledgment of the current notice, the same `emit` returns `Recorded{seq}` (AC for §1 #8).
10. **System actor exempt** — `subject_id = null` event skips the gate and records (AC for §1 #8).
11. **Validation rejects bad module/type** — an `event_type` without the module prefix → `EmitError::Invalid` (AC for §1 #9).
12. **Body size bound** — a 17 KiB body → `EmitError::Invalid` (AC for §1 #10).
13. **Canonical determinism** — the same logical event serialised twice yields byte-identical bodies and identical chain anchors (AC for §1 #11).
14. **Consent cache** — two `emit`s for the same `(tenant, subject)` within 60 s issue ≤ 1 consent-ledger query (AC for §1 #12).
15. **Metrics emit** — `memory_interaction_events_total{outcome="recorded"}` increments on a recorded event (AC for §1 #13).
16. **RLS isolation** — tenant A's interaction-event rows are invisible to a tenant-B read through the `l1_audit_log` RLS path (AC for §1 #14).
17. **Schema is frozen/versioned** — the contract test asserts `schema_version` const 1 and that every emitted body validates against the published schema (AC for §1 #15).
18. **Builder validates at build()** — `InteractionEvent::builder(..)` with an unknown-prefixed event_type returns `Err` before any emit (AC for §1 #16).
19. **Replay idempotent** — emitting the same `event_id` twice yields one row (unique index); the second is a no-op/`ON CONFLICT` (AC for §1 #17).
20. **Indexes used** — `EXPLAIN` of a per-subject interaction scan uses `l1_iev_subject_idx` (AC for §1 #2/#18 indexing).

---

## §5 — Verification

```rust
#[tokio::test]
async fn valid_event_chains_into_audit_log() {
    let env = TestEnv::new().await;
    env.ack_notice(env.tenant(), env.alice()).await;          // pass the gate
    let ev = InteractionEvent::builder(Module::Chat, "chat.message_created", EventClass::Content)
        .subject(env.alice()).occurred_now()
        .target(TargetRef::Message { id: "msg-1".into() })
        .content(ContentRef::Pointer { store: "chat_messages".into(), id: "msg-1".into() })
        .source(SourceChannel::Web).tenant(env.tenant()).build().unwrap();
    let out = emit(&env.audit_pool, &ev).await.unwrap();
    let seq = match out { EmitOutcome::Recorded { seq } => seq, _ => panic!("expected recorded") };

    let row: (String, String, String) = sqlx::query_as(
        "SELECT op, event_type, chain_anchor_hex FROM l1_audit_log WHERE seq = $1")
        .bind(seq).fetch_one(&env.audit_pool).await.unwrap();
    assert_eq!(row.1, "memory.interaction_event");
    assert_eq!(row.0, "put");                                  // Content class → put
    assert!(env.reconcile_verifies(seq).await);                // anchor verifies under memory reconcile
}

#[tokio::test]
async fn read_class_records_as_view() {
    let env = TestEnv::new().await;
    env.ack_notice(env.tenant(), env.alice()).await;
    let ev = InteractionEvent::builder(Module::Proj, "proj.document_opened", EventClass::Read)
        .subject(env.alice()).occurred_now()
        .target(TargetRef::Document { id: "doc-1".into() })
        .content(ContentRef::None).source(SourceChannel::Web).tenant(env.tenant()).build().unwrap();
    let EmitOutcome::Recorded { seq } = emit(&env.audit_pool, &ev).await.unwrap() else { panic!() };
    let op: String = sqlx::query_scalar("SELECT op FROM l1_audit_log WHERE seq = $1")
        .bind(seq).fetch_one(&env.audit_pool).await.unwrap();
    assert_eq!(op, "view");
}

#[tokio::test]
async fn consent_gate_blocks_unacknowledged_subject() {
    let env = TestEnv::new().await;                            // no ack seeded
    let ev = sample_event(env.tenant(), Some(env.bob()));
    let out = emit(&env.audit_pool, &ev).await.unwrap();
    assert!(matches!(out, EmitOutcome::Skipped { reason: SkipReason::ConsentNotAcknowledged }));
    let n: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM l1_audit_log WHERE event_type='memory.interaction_event'")
        .fetch_one(&env.audit_pool).await.unwrap();
    assert_eq!(n, 0, "no row written before consent");
}

#[tokio::test]
async fn system_actor_exempt_from_gate() {
    let env = TestEnv::new().await;
    let ev = sample_event(env.tenant(), None);                 // subject_id = null
    assert!(matches!(emit(&env.audit_pool, &ev).await.unwrap(), EmitOutcome::Recorded { .. }));
}

#[tokio::test]
async fn oversize_attributes_rejected() {
    let env = TestEnv::new().await;
    let mut ev = sample_event(env.tenant(), Some(env.alice()));
    ev.attributes.insert("blob".into(), serde_json::json!("x".repeat(3000)));
    assert!(matches!(emit(&env.audit_pool, &ev).await, Err(EmitError::Invalid(_))));
}

#[tokio::test]
async fn canonical_body_is_deterministic() {
    let ev = sample_event(uuid::Uuid::nil(), Some(uuid::Uuid::nil()));
    assert_eq!(canonical_audit_body(AUDIT_ROW_KIND, &ev), canonical_audit_body(AUDIT_ROW_KIND, &ev));
}

#[tokio::test]
async fn replay_same_event_id_is_idempotent() {
    let env = TestEnv::new().await;
    env.ack_notice(env.tenant(), env.alice()).await;
    let ev = sample_event(env.tenant(), Some(env.alice()));
    let _ = emit(&env.audit_pool, &ev).await.unwrap();
    let _ = emit(&env.audit_pool, &ev).await;                  // same event_id
    let n: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM l1_audit_log WHERE (body::jsonb->'payload'->>'event_id') = $1")
        .bind(ev.event_id.to_string()).fetch_one(&env.audit_pool).await.unwrap();
    assert_eq!(n, 1);
}

#[tokio::test]
async fn every_emitted_body_validates_against_schema() {
    let schema = load_contract_schema();                       // contracts/interaction-event.schema.json
    for class in [EventClass::Auth, EventClass::Presence, EventClass::Content, EventClass::Read] {
        let ev = sample_event_with_class(uuid::Uuid::now_v7(), class);
        let payload = serde_json::to_value(&ev).unwrap();
        assert!(schema.validate(&payload).is_ok(), "class {class:?} failed schema");
    }
}
```

---

## §6 — Implementation skeleton

See §3 (event + enums, emit + gate, op-aware genesis, contract). The builder (`InteractionEvent::builder`) lives in `interaction/event.rs`; `canonical_audit_body` is shared with the memory chain canonicaliser.

---

## §7 — Dependencies

- **FR-MEMORY-101** — the layer-1 audit chain + reconcile + layer-2 ingest these rows ride on; `cyberos-audit-chain` anchor compatibility.
- **FR-EVAL-001** — owns the monitoring-notice + acknowledgment ledger the consent gate reads (`eval_monitoring_ack`, `eval_current_notice`). Until it lands, the gate resolves against a documented operator-seeded stub (§11).
- **FR-AUTH-002 / FR-AUTH-004** — `subject_id` and `session_id` come from AUTH subjects + sessions (jti).
- **FR-OBS-003** — `trace_id` is the W3C traceparent the request middleware sets.
- **FR-OBS-008** — the `event_type` generated column on `l1_audit_log` these rows populate; FR-APP-005 viewer reads it.
- **Downstream:** FR-MEMORY-122 (emitters depend on this contract), FR-MEMORY-123 (ingestion reads this shape), FR-EVAL-003 (evaluation indexes these dimensions).
- Crates: `sqlx@0.7`, `serde`, `serde_json`, `uuid` (v7 feature), `chrono`, `thiserror`, `jsonschema` (contract test), `sha2` (content_ref hash).

---

## §8 — Example payloads

### Sign-in (auth, no content)

```json
{
  "event_type": "memory.interaction_event",
  "payload": {
    "schema_version": 1,
    "event_id": "018f9c2a-7e10-7c3b-9a44-6b1d2e3f4a55",
    "tenant_id": "cyberskill-tenant-uuid",
    "subject_id": "stephen-subject-uuid",
    "occurred_at_ns": 1782950400000000000,
    "module": "auth",
    "event_type": "auth.signed_in",
    "event_class": "auth",
    "target_ref": { "kind": "session", "id": "jti-abc123" },
    "content_ref": { "kind": "none" },
    "session_id": "jti-abc123",
    "trace_id": "0af7651916cd43dd8448eb211c80319c",
    "source_channel": "web",
    "attributes": { "source_ip_hash16": "9f86d081884c7d65" }
  }
}
```

### Chat message created (content as pointer, never raw body)

```json
{
  "event_type": "memory.interaction_event",
  "payload": {
    "schema_version": 1,
    "event_id": "018f9c2a-9b22-7e88-8c01-11aa22bb33cc",
    "tenant_id": "cyberskill-tenant-uuid",
    "subject_id": "daria-subject-uuid",
    "occurred_at_ns": 1782950460000000000,
    "module": "chat",
    "event_type": "chat.message_created",
    "event_class": "content",
    "target_ref": { "kind": "channel", "id": "general-channel-uuid" },
    "content_ref": { "kind": "pointer", "store": "chat_messages", "id": "msg-7e57c0de" },
    "session_id": "jti-def456",
    "trace_id": null,
    "source_channel": "web",
    "attributes": { "channel_kind": "channel", "has_attachment": false }
  }
}
```

### Document opened (proj, read → op=view)

```json
{
  "event_type": "memory.interaction_event",
  "payload": {
    "schema_version": 1,
    "event_id": "018f9c2b-0140-7a51-9d77-44ee55ff6600",
    "tenant_id": "cyberskill-tenant-uuid",
    "subject_id": "stephen-subject-uuid",
    "occurred_at_ns": 1782950520000000000,
    "module": "proj",
    "event_type": "proj.document_opened",
    "event_class": "read",
    "target_ref": { "kind": "document", "id": "doc-ssl-l4-spec" },
    "content_ref": { "kind": "none" },
    "session_id": "jti-abc123",
    "source_channel": "web",
    "attributes": {}
  }
}
```

---

## §9 — Open questions

All resolved. Deferred:
- Per-`event_type` retention classes (e.g. presence pruned sooner than IP-activity) — owned by FR-EVAL-001 governance/retention, applied at ingest in FR-MEMORY-123.
- A streaming emit (batched, async channel) for very high-frequency emitters — slice 2; the synchronous best-effort `emit` is sufficient for AUTH + CHAT volumes at P0 scale.
- Cross-tenant aggregate views for CyberSkill-wide analytics — out of scope; RLS is per-tenant by design.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Audit pool unreachable at emit | sqlx error | `EmitError::Db`; caller logs + swallows; interaction itself unaffected | Self-heals; OBS alarm on sustained drop |
| Subject has not acknowledged notice | consent gate false | `Skipped{ConsentNotAcknowledged}`; no row | By design; person acknowledges, then capture begins |
| Notice version bumped (mass re-ack needed) | consent_skipped spike | Everyone suppressed until re-ack | By design; operator re-circulates notice (FR-EVAL-001) |
| Raw content smuggled in `attributes` | 2 KiB cap + body-bytes histogram | `EmitError::Invalid` at the limit; metric flags growth below it | Author fixes emitter to use content_ref |
| Unknown `module` value | serde enum + validate | `EmitError::Invalid`; no row | Author uses the closed enum |
| `event_type` lacks module prefix | validate | `EmitError::Invalid`; no row | Author namespaces the verb |
| Malformed `content_ref` arm | JSON Schema oneOf | rejected at contract test + validate | Author fixes the union arm |
| Replay / CDC re-delivers same event | unique index on event_id | second insert ON CONFLICT no-op | By design (§1 #17) |
| Consent cache serves stale `true` after revoke | 60 s TTL | up to 60 s of extra capture | By design; documented bound |
| Consent cache serves stale `false` after ack | 60 s TTL | up to 60 s of dropped capture | By design; documented bound |
| FR-EVAL-001 ledger absent (pre-landing) | stub resolves default | operator-seeded default applies | Replace stub when FR-EVAL-001 lands |
| Body > 16 KiB | size bound | `EmitError::Invalid`; no row | Author trims attributes / uses pointer |
| Schema bumped to v2 while emitter pins v1 | version check | v1 emitter still validates against v1 schema; v2-only fields ignored | Additive rule; bump is loud |
| Clock skew makes occurred_at_ns < a prior event | none (events independent) | ordering by event_id (v7) is monotonic enough | By design |
| RLS bypass attempt on read | l1_audit_log RLS policy | 0 rows | By design |
| `op='view'` row deleted by privileged role | chain reconcile (FR-MEMORY-101) | detected as a chain gap | Incident response |
| Module emits before AUTH session exists | session_id null | recorded with null session | By design; not all interactions have a session |
| OTel exporter down | buffered then dropped | metrics gap; rows still written | Restore FR-OBS-001 |
| Two emitters race the same event_id (bug) | unique index | one wins; SEV-3 warn | Author fixes id generation |
| Generated column NULL (payload missing module) | partial index excludes | row still chained; just unindexed by module | Validate guarantees module present for valid rows |
| Very high emit rate from one subject | consent cache + best-effort | DB write pressure on l1_audit_log | Slice-2 batched emit |

---

## §11 — Implementation notes

- Interaction-events are NOT a new store. They are `memory.interaction_event` aux rows on `l1_audit_log`, written through `cyberos-audit-chain` exactly like `auth.token_issued` and the obs rows, so memory's reconcile and the FR-MEMORY-101 layer-2 ingest accept them with no special path. This is the whole point of DEC-2703.
- The audit-row `event_type` (FR-OBS-008's generated column) is the constant `memory.interaction_event`; the interaction's own verb is `payload.event_type`, surfaced by the new `iev_event_type` generated column so it is indexable without per-row JSON parsing. The FR-APP-005 audit viewer keeps working because it reads the row-level `event_type`.
- `content_ref` is the privacy spine. Chat message bodies stay in chat's own DB under chat's RLS; the BRAIN holds `pointer{store:"chat_messages", id}`. The evaluation engine resolves the pointer only when it needs the content and only for a viewer who already has access to it — the BRAIN never becomes a second, looser copy of sensitive content.
- The consent gate reads FR-EVAL-001's acknowledgment ledger. FR-EVAL-001 is Phase 0 and lands first; this FR depends on it. Until it ships, `consent_gate` resolves against a stub table the operator seeds (default: capture OFF for every subject, so the safe default is "no capture without explicit acknowledgment"). The stub is replaced, not the call site, when FR-EVAL-001 arrives.
- `subject_id = null` is reserved for genuine system actors (scheduled jobs, the dream loop). It MUST NOT be used to anonymise a real person's action — that would defeat both the evaluation purpose and the audit trail. Validation cannot enforce intent here, so it is a code-review rule, called out in the schema description.
- `op` derivation: read-class interactions chain as `view`, mutations as `put`. The chain's `op` enum already had `view`; this FR is the first heavy producer of `view` rows, which is why `emit_genesis_with_op` is added (the old `emit_genesis` becomes a thin `'put'` shim so AUTH/OBS callers are untouched).
- UUIDv7 for `event_id` buys time-ordering for free (the BRAIN's recency-decay recall, FR-MEMORY-113, and replay dedup both want it) and keeps the unique index selective.
- The 2 KiB `attributes` cap and the body-bytes histogram are the anti-leak pair: the cap makes the worst case impossible, the histogram makes drift visible. If a module's body bytes climb, that is the signal someone started inlining content instead of referencing it.
- The contract file is the dependency surface for every emitter FR. FR-MEMORY-122 (AUTH + CHAT emitters) and every later module emitter depend on `interaction-event.schema.json` at `schema_version: 1`, not on each other. The additive-only rule (new optional fields, new verbs) lets the vocabulary grow without a flag day; a real breaking change is a `schema_version: 2` with its own migration note.
- The builder is the misuse-resistant front door, but `emit` re-validates so a hand-constructed event or a future non-builder caller still cannot write a malformed row. Defence in depth, cheap.
- This FR is `eu_ai_act_risk_class: limited` because the events it captures feed a downstream system (the BRAIN + the evaluation engine, FR-EVAL-003) that informs human decisions about people. This FR itself runs no model and makes no decision — it is the capture contract — but the limited classification is carried here so the whole capture-to-evaluation chain is consistently flagged. See the AI Risk Assessment below.

---

## AI Risk Assessment

- **Why limited, not minimal/not_ai:** this FR defines the capture primitive whose output feeds the BRAIN ingestion (FR-MEMORY-123) and, ultimately, an AI-assisted evaluation of employees against the three signed documents (FR-EVAL-003/004). The capture is not itself AI, but it is the data foundation of a limited-risk AI use (workplace evaluation support), so it is classified and governed as part of that chain.
- **Personal data captured:** identity and activity of CyberSkill employees on the platform — who signed in, who messaged in which channel, who opened or changed which document/issue. It is platform work-interaction metadata, plus pointers/hashes to content; it is NOT keystroke logging, screen capture, or anything from private life (DEC-2701). Content references prefer pointers/hashes over raw bodies, so the captured row carries who/what/when, not the sensitive payload.
- **Lawful basis + notice (human governance):** capture for any subject is hard-gated on that subject's acknowledgment of the FR-EVAL-001 monitoring notice (DEC-2702). The gate lives in the one emit path, so there is no code path that captures a person before notice is acknowledged — disclosed monitoring, never covert. The Vietnamese PDPD framing (lawful basis, notice, purpose limitation, data-subject rights) and the Labor Code context are owned by FR-EVAL-001; this FR enforces the technical gate that makes the notice meaningful.
- **Purpose limitation + minimisation:** the closed `module`/`event_class`/`target_ref`/`content_ref` vocabulary bounds what can be recorded; the 2 KiB attributes cap and the content_ref-over-raw-body rule enforce minimisation; the per-`event_type` retention classes are applied downstream (FR-EVAL-001 / FR-MEMORY-123). The point is to record the work, not the person's life.
- **Access + tenancy:** interaction-events inherit the `l1_audit_log` per-tenant RLS, so a tenant's records never cross to another tenant. Who-can-read (manager + HR + the employee for their own record) is enforced by the consuming surfaces (FR-EVAL-005 views), not loosened here.
- **No autonomous decision:** this FR records; it never scores, ranks, or decides. Every consequential use of these events runs through the human-in-the-loop evaluation workflow (FR-EVAL-004). The capture primitive deliberately has no opinion about the events it stores.

---

*End of FR-MEMORY-121.*
