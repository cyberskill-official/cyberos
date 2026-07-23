---
id: TASK-MEMORY-122
title: "capture emitters — wire AUTH (sign-in, presence) + CHAT (message create/edit/delete, channel/DM activity) to emit TASK-MEMORY-121 interaction-events, turn ON the chat→brain audit link (CHAT_AUDIT_DATABASE_URL), define the emitter contract for PROJ/EMAIL/APP/MCP, gated by the consent check; day-1 wide capture"
client_visible: false
type: feature
created_at: 2026-06-29T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: memory
priority: p0
status: draft
verify: T
phase: P1
milestone: P1 · BRAIN capture · slice 2
slice: 2
owner: Stephen Cheng (CDO)
created: 2026-06-29
shipped: null
memory_chain_hash: null
related_tasks: [TASK-MEMORY-121, TASK-MEMORY-101, TASK-MEMORY-123, TASK-EVAL-001, TASK-AUTH-002, TASK-AUTH-004, TASK-CHAT-101, TASK-OBS-003, TASK-APP-005]
depends_on: [TASK-MEMORY-121, TASK-AUTH-002, TASK-CHAT-101]
blocks: [TASK-MEMORY-123]

source_pages:
  - docs/strategy/cyberos-brain-evaluation-plan.md#phase-1-capture
source_decisions:
  - DEC-2710 (WIDE, day-1 capture: every platform work-interaction emits the shared interaction-event from the moment a person logs in — sign-in/presence, chat message, module open/use, task/doc/IP activity; Stephen 2026-06-29)
  - DEC-2711 (platform work-interactions ONLY — no keystroke logging, no screen capture, no private life; emitters reference content via content_ref pointers/hashes, never inline raw bodies)
  - DEC-2712 (every emitter goes through TASK-MEMORY-121 emit(), which is consent-gated on the TASK-EVAL-001 notice acknowledgment — no emitter captures a subject who has not acknowledged)
  - DEC-2713 (turn ON the chat→brain audit link: set CHAT_AUDIT_DATABASE_URL so chat activity chains into MEMORY's l1_audit_log; P0 deliberately left it OFF, so chat content is in chat's DB but not yet mirrored to the brain)
  - DEC-2714 (define the emitter contract once — a CaptureEmitter trait + a thin per-module call-site convention — so PROJ/EMAIL/APP/MCP add emitters as they come online without re-deciding the shape)

ai_authorship: assisted
eu_ai_act_risk_class: limited
language: rust 1.81
service: cyberos/services/
new_files:
  - services/shared/cyberos-capture/Cargo.toml
  - services/shared/cyberos-capture/src/lib.rs
  - services/shared/cyberos-capture/src/emitter.rs
  - services/auth/src/capture.rs
  - services/chat/src/capture.rs
  - services/memory/src/interaction/backfill.rs
  - services/auth/tests/capture_signin_test.rs
  - services/chat/tests/smoke_capture.py
  - services/memory/tests/interaction_backfill_test.rs
modified_files:
  # emit auth.signed_in / auth.sign_in_failed next to emit_token_issued/_failed
  - services/auth/src/handlers.rs
  # hold the capture audit pool in AppState
  - services/auth/src/lib.rs
  # emit chat.message_created/_edited/_deleted via capture
  - services/chat/src/messages.rs
  # emit chat.channel_created/_joined/_left
  - services/chat/src/channels.rs
  # emit chat.presence_changed on join/leave
  - services/chat/src/realtime.rs
  # route interaction-events through cyberos-capture (keep best-effort)
  - services/chat/src/audit.rs
  # CHAT_AUDIT_DATABASE_URL is now the brain link (DEC-2713) — warn->info, document required-in-prod
  - services/chat/src/main.rs
  # set CHAT_AUDIT_DATABASE_URL on the chat service (Supabase audit DB)
  - deploy/vps/docker-compose.yml
  # document the chat→brain link is ON + how to verify
  - docs/deploy/p0-google-chat-runbook.md
allowed_tools:
  - file_read: services/{auth,chat,memory,shared}/**
  - file_write: services/shared/cyberos-capture/**
  - file_write: services/auth/{src,tests}/**
  - file_write: services/chat/{src,tests}/**
  - file_write: services/memory/{src,tests}/**
  - file_write: deploy/vps/docker-compose.yml
  - file_write: docs/deploy/p0-google-chat-runbook.md
  - bash: cd services/auth && cargo test capture
  - bash: cd services/chat && cargo test && python tests/smoke_capture.py
  - bash: cd services/memory && cargo test interaction_backfill
disallowed_tools:
  - emit an interaction-event by hand-building an l1_audit_log row instead of calling TASK-MEMORY-121 emit() (bypasses the consent gate + validation + schema)
  - inline a raw chat message body / document text in any emitter payload (per DEC-2711 — use content_ref::pointer to the owning store)
  - capture a subject who has not acknowledged the notice (per DEC-2712 — guaranteed by routing through emit(), never around it)
  #7)
  - block or fail the underlying interaction (sign-in, message send) on a capture error (capture is best-effort, per TASK-MEMORY-121 §1

effort_hours: 10
subtasks:
  - "1.0h: cyberos-capture crate — CaptureEmitter trait + Capturer{pool} wrapper that builds the TASK-MEMORY-121 event + calls emit(); shared by auth + chat (and future modules)"
  - "1.5h: auth/capture.rs + handlers wiring — emit auth.signed_in (on token issue) + auth.sign_in_failed (on token_failed), with source_ip_hash16 in attributes; reuse the existing audit pool"
  - "1.5h: chat/capture.rs + messages/channels/realtime wiring — chat.message_created/_edited/_deleted (content_ref::pointer{chat_messages,id}), chat.channel_created/_joined/_left, chat.dm_opened, chat.presence_changed"
  - "0.5h: turn ON the chat→brain link — chat already has CHAT_AUDIT_DATABASE_URL + audit_pool; point it at the brain DB, flip the 'unset' warn to an info, make it required-in-prod; set it in deploy compose"
  - "1.0h: presence emit dedup — only emit presence_changed on the 0↔1 connection-count edge (join that brings a subject online / leave that takes them offline), not on every websocket"
  - "1.5h: interaction/backfill.rs — replay recent chat history into interaction-events for subjects who have since acknowledged (bounded window, idempotent on event_id, dry-run default)"
  - "0.5h: emitter metrics — reuse TASK-MEMORY-121 counters; add memory_capture_emitter_calls_total{module,event_type,outcome}"
  - "0.5h: capture_signin_test.rs — sign-in emits one auth.signed_in interaction-event chained into the audit DB (gated subject) / none for an unacknowledged subject"
  - "1.0h: smoke_capture.py — create/edit/delete a message + join a channel over the live chat service with the brain link ON; assert the matching interaction-event rows appear in l1_audit_log"
  - "0.5h: interaction_backfill_test.rs — backfill replays N messages into events idempotently; re-run is a no-op"
  - "0.5h: docs/runbook — p0 runbook says the chat→brain link is ON, with the verify query; note the consent prerequisite"
risk_if_skipped: "TASK-MEMORY-121 defines the event shape but nothing emits it, so the BRAIN stays empty and the whole evaluation plan has no data — the schema is a contract with no traffic. Concretely: chat content is already live in production (P0) but the audit link is OFF, so none of it reaches the brain; without this task that gap persists and the first thing the evaluation engine would look for (what people actually did) isn't there. Without the shared emitter contract, AUTH and CHAT each wire capture differently and PROJ/EMAIL/APP/MCP re-litigate the shape when they arrive. Without routing every emitter through emit(), one emitter can forget the consent gate and capture someone who never acknowledged — the exact governance failure the gate exists to prevent."
---

## §1 — Description (BCP-14 normative)

This task makes day-1 wide capture real: it wires the first two live modules (AUTH and CHAT) to emit TASK-MEMORY-121 interaction-events for every platform work-interaction, turns on the chat→brain audit link so chat activity chains into MEMORY, and defines the emitter contract so the remaining modules add emitters the same way. It introduces no new event shape — it produces events of the TASK-MEMORY-121 shape. Each piece:

1. **MUST** provide a shared emitter in a new crate `cyberos-capture`: a `Capturer` that holds the brain audit pool and a `capture()` method which builds an TASK-MEMORY-121 `InteractionEvent` (via that task's typed builder) and calls TASK-MEMORY-121 `emit()`. Every module emits through this one path; no module constructs an `l1_audit_log` row directly. The `Capturer::capture` signature is best-effort: it returns the TASK-MEMORY-121 `EmitOutcome`/`EmitError`, and callers log + swallow.
2. **MUST** define a `CaptureEmitter` convention (DEC-2714): each module owns a thin `capture.rs` that exposes typed helpers (e.g. `emit_signed_in(...)`, `emit_message_created(...)`) translating that module's domain event into a built `InteractionEvent`. The helper set is the per-module surface; the crate is the shared mechanism. New modules add a `capture.rs`, not a new shape.
3. **MUST** wire AUTH to emit, next to the existing `emit_token_issued` / `emit_token_failed` in `handlers.rs`:
- `auth.signed_in` (`event_class: auth`, `source_channel` from the request, `target_ref: session{jti}`, `content_ref: none`, `attributes: {source_ip_hash16, method}` where method ∈ `password | oidc | passkey`) on a successful token issue.
- `auth.sign_in_failed` (`event_class: auth`, `content_ref: none`, `attributes: {reason, source_ip_hash16}`) on a failed attempt. `subject_id` is `null` when the email did not resolve (no person to attribute), mirroring `emit_token_failed`.
- These reuse AUTH's existing audit pool (the same one `memory_bridge` writes to); no new connection.
4. **MUST** wire CHAT to emit, in `messages.rs` / `channels.rs` / `realtime.rs`:
- `chat.message_created` (`event_class: content`, `target_ref: channel{id}` or `dm{id}`, `content_ref: pointer{store:"chat_messages", id:<message_id>}`, `attributes: {channel_kind, has_attachment}`).
- `chat.message_edited` (`content`, same target, `content_ref: pointer{chat_messages,id}`).
- `chat.message_deleted` (`content`, `content_ref: none` — the body is gone; the pointer would dangle).
- `chat.channel_created` (`admin`, `target_ref: channel{id}`).
- `chat.channel_joined` / `chat.channel_left` (`activity`, `target_ref: channel{id}`).
- `chat.dm_opened` (`activity`, `target_ref: dm{id}`).
- `chat.presence_changed` (`presence`, `target_ref: channel{id}`, `attributes: {state}` where state ∈ `online | offline`). Message bodies are NEVER inlined; the `content_ref` points at chat's own `chat_messages` row (migration 0005's attachment link is reused for the attachment flag only).
5. **MUST** turn ON the chat→brain audit link (DEC-2713). CHAT already reads `CHAT_AUDIT_DATABASE_URL` into `audit_pool` and logs `"unset; chat audit events are logged, not chained"` when absent (P0 left it unset). This task sets that variable to the brain's audit DB (the MEMORY `l1_audit_log` database, Supabase Postgres in prod) so chat interaction-events chain into MEMORY, changes the `unset` log from `warn` to `info`, and documents the variable as required in production. The chat capture path routes through `cyberos-capture` → TASK-MEMORY-121 `emit()` over that pool.
6. **MUST** route every emitter through TASK-MEMORY-121 `emit()` so the consent gate (TASK-MEMORY-121 §1 #8) and validation apply uniformly. An emitter physically cannot skip the gate, because it has no other way to write the row. A subject who has not acknowledged the TASK-EVAL-001 notice produces `Skipped` outcomes and zero rows, regardless of which module emitted.
7. **MUST** keep capture best-effort and off the critical path: a capture failure (audit pool down, validation error, `Skipped`) MUST NOT fail or delay the underlying interaction — the sign-in still issues its token, the message still sends. Emitters call `capture()`, match the outcome for metrics, and swallow errors with a `tracing::warn!`, exactly as AUTH's `emit_token_issued` and chat's `audit::emit` already do.
8. **MUST** dedup presence emits to the online/offline edges only: `chat.presence_changed{online}` is emitted when a subject's open-connection count for a channel goes 0→1, and `{offline}` when it goes 1→0 (the `Presence` map in `realtime.rs` already tracks the count). A subject opening a second tab does NOT emit another `online`. This keeps presence signal meaningful and the chain uncluttered.
9. **MUST** provide a bounded backfill/replay path (`services/memory/src/interaction/backfill.rs`) that, for a subject who has since acknowledged the notice, replays recent chat history (a bounded window, default 30 days, operator-set) into `chat.message_created` interaction-events. Backfill is idempotent on `event_id` (deterministically derived from the source message id so a re-run is a no-op via TASK-MEMORY-121 §1 #17), defaults to a dry-run that reports the count, and only writes on an explicit `--apply`. This recovers the pre-acknowledgment / pre-link history without double-counting.
10. **MUST** derive backfill `event_id`s deterministically from the source row (e.g. UUIDv5 over `chat_messages.id`) so replaying the same message always yields the same `event_id`, and `occurred_at_ns` from the message's original timestamp (not replay time) so the backfilled event sits at the correct point in the person's history.
11. **MUST** respect the consent gate in backfill too: a message is only backfilled for a subject who has acknowledged; messages authored by an unacknowledged subject are skipped (and counted), never written. Backfill calls the same `emit()` and so inherits the gate; it does not bypass it for historical data.
12. **MUST** emit OTel metrics (in addition to the TASK-MEMORY-121 counters): `memory_capture_emitter_calls_total{module, event_type, outcome}` (outcome ∈ `recorded | skipped_consent | invalid | emit_error`) and `memory_capture_backfill_events_total{outcome}`. These give a per-emitter view of live capture and of backfill progress.
13. **MUST** tag `source_channel` from the real entry point: AUTH reads it from the request (web console vs API vs CLI), CHAT from the client (web/desktop/mobile/api), backfill emits `import`. This is what makes "Daria messaged from mobile" vs "an API client posted" distinguishable downstream.
14. **MUST** set `trace_id` on emitted events when a request trace is in scope (TASK-OBS-003 traceparent), so a captured interaction correlates with its request span and log line. When no trace is in scope (e.g. a websocket presence edge), `trace_id` is `null` — never fabricated.
15. **MUST** document, in `docs/deploy/p0-google-chat-runbook.md`, that the chat→brain link is ON in production, the exact `CHAT_AUDIT_DATABASE_URL` it uses, the verify query (`SELECT count(*) FROM l1_audit_log WHERE event_type='memory.interaction_event' AND iev_module='chat'`), and the consent prerequisite (rows appear only for acknowledged subjects). Operators must be able to confirm capture is live and understand why an unacknowledged subject shows none.
16. **MUST** keep AUTH and CHAT independent of MEMORY's HTTP liveness: capture writes go directly to the shared audit DB via `cyberos-capture` (the same direct-write pattern AUTH `memory_bridge` and the obs services already use), so capture does not couple a sign-in or a message send to the memory service being up.

---

## §2 — Why this design (rationale for humans)

**Why a shared `cyberos-capture` crate (DEC-2714)?** AUTH and CHAT (and soon PROJ, EMAIL, APP, MCP) all need to turn a domain event into the one interaction-event shape and write it the same way. Without a shared mechanism, each module reinvents the building + the best-effort write + the metric, and they drift. The crate holds the mechanism once; each module's `capture.rs` holds only the thin domain translation. A new module adds a `capture.rs`, not a new pattern.

**Why wire AUTH and CHAT first?** They are the two live modules in production today (P0 is sign-in + team chat). Sign-in and presence are the "from the moment a person logs in" signal the plan calls for, and chat is where most of the work conversation happens. Starting here makes capture real on day one with the modules that already have traffic, instead of waiting for modules that aren't built.

**Why turn on the chat→brain link now (DEC-2713)?** The brain plan names this exact gap: chat content is in chat's DB but, because P0 left `CHAT_AUDIT_DATABASE_URL` off, none of it is mirrored to the brain. The infrastructure is already there — chat reads the variable, holds an `audit_pool`, and writes through `cyberos-audit-chain` when it is set. Turning it on is a configuration flip plus routing chat's capture through the TASK-MEMORY-121 emit path, not new plumbing. Until it is on, the brain has no chat signal at all.

**Why route everything through `emit()` and forbid hand-built rows (DEC-2712)?** The consent gate, the validation, and the schema only protect capture if there is no way around them. If an emitter could insert an `l1_audit_log` row directly, it could (by accident) capture an unacknowledged subject or write a malformed/raw-content row. Making `emit()` the only door means the governance property holds for every module by construction, not by everyone remembering.

**Why best-effort, never blocking (§1 #7)?** Capture must never be the reason a sign-in fails or a message is slow. The product comes first; the record is a side effect. AUTH and CHAT already treat their audit writes this way; this task keeps that contract so capture is invisible to the user whose work it records.

**Why dedup presence to edges (§1 #8)?** Presence is useful signal ("Stephen was online 09:00–18:00") but a naive emit on every websocket open/close produces noise — a person with three tabs would emit three `online`s. Emitting only on the 0↔1 connection-count edge captures the meaningful transition and keeps the chain readable. The count is already tracked in `realtime.rs`.

**Why backfill exists and why it is idempotent + consent-gated (§1 #9–#11)?** Two reasons drive replay: chat has live production history from before the link was on, and a person who acknowledges the notice later should have their prior platform activity captured from the acknowledged window. Deterministic `event_id`s (UUIDv5 over the source message id) make a re-run a no-op, so backfill is safe to run repeatedly. Routing backfill through `emit()` means historical capture obeys the same consent rule as live capture — you do not get to capture someone's history just because it is in the past.

**Why deterministic id + original timestamp in backfill (§1 #10)?** If backfill used fresh ids and replay-time timestamps, a re-run would double-count and the events would sit at the wrong point in the person's timeline. Deriving the id from the message and the time from the original message keeps the backfilled history both idempotent and chronologically honest.

**Why direct-write to the audit DB, not via the memory HTTP service (§1 #16)?** AUTH `memory_bridge` and the obs services already write to `l1_audit_log` directly because the alternative — coupling a sign-in or a message send to memory's HTTP liveness — is worse. Capture follows the same rule: it shares the Postgres deployment and writes directly, so a memory-service blip never touches the critical path.

**Why document the link + verify query in the runbook (§1 #15)?** "Is capture actually live?" must be answerable in one query, and "why does this person show no rows?" must have an obvious answer (they have not acknowledged). The runbook is where the operator looks; the brain plan's whole trust argument depends on capture being visible and explainable, not a black box.

---

## §3 — API contract

### Shared crate

```rust
// services/shared/cyberos-capture/src/emitter.rs
use cyberos_memory::interaction::{emit, event::*, EmitOutcome, EmitError};
use sqlx::PgPool;
use uuid::Uuid;

/// The shared capture mechanism. Holds the brain audit pool; turns a built InteractionEvent into a
/// chained l1_audit_log row via TASK-MEMORY-121 emit() (consent-gated, validated, best-effort).
#[derive(Clone)]
pub struct Capturer {
    pool: PgPool,
}

impl Capturer {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    /// Best-effort: returns the TASK-MEMORY-121 outcome. Callers log + swallow on error;
    /// the underlying interaction (sign-in, message send) MUST proceed regardless.
    pub async fn capture(&self, ev: &InteractionEvent) -> Result<EmitOutcome, EmitError> {
        emit(&self.pool, ev).await
    }
}

/// A module's capture.rs implements typed helpers that build the event, then call Capturer::capture.
pub trait CaptureEmitter {
    fn capturer(&self) -> &Capturer;
}
```

### AUTH emitter

```rust
// services/auth/src/capture.rs
use cyberos_capture::Capturer;
use cyberos_memory::interaction::event::*;
use uuid::Uuid;

pub async fn emit_signed_in(
    cap: &Capturer, tenant: Uuid, subject: Uuid, jti: &str,
    method: &str, source: SourceChannel, ip_hash16: &str, trace_id: Option<String>,
) {
    let ev = InteractionEvent::builder(Module::Auth, "auth.signed_in", EventClass::Auth)
        .tenant(tenant).subject(subject).occurred_now()
        .session(Uuid::nil())                              // jti string carried in target_ref/attributes
        .target(TargetRef::Session { id: jti.to_string() })
        .content(ContentRef::None)
        .source(source).trace(trace_id)
        .attr("method", method).attr("source_ip_hash16", ip_hash16)
        .build();
    match ev {
        Ok(ev) => { let _ = cap.capture(&ev).await; }       // best-effort
        Err(e) => tracing::warn!(error = %e, "auth.signed_in event build failed"),
    }
}

pub async fn emit_sign_in_failed(
    cap: &Capturer, tenant: Uuid, subject: Option<Uuid>,
    reason: &str, source: SourceChannel, ip_hash16: &str,
) {
    let mut b = InteractionEvent::builder(Module::Auth, "auth.sign_in_failed", EventClass::Auth)
        .tenant(tenant).occurred_now()
        .target(TargetRef::None).content(ContentRef::None)
        .source(source).attr("reason", reason).attr("source_ip_hash16", ip_hash16);
    if let Some(s) = subject { b = b.subject(s); }          // null when the email didn't resolve
    if let Ok(ev) = b.build() { let _ = cap.capture(&ev).await; }
}
```

```rust
// services/auth/src/handlers.rs  (added next to the existing emit_token_issued call, ~line 2126)
//
// TASK-MEMORY-122 §1 #3 — emit the auth.signed_in interaction-event alongside the audit token row.
// Same pool as memory_bridge; best-effort; does not gate token issuance.
crate::capture::emit_signed_in(
    &state.capturer, tenant_id, sub_id, &jti, method, source_channel,
    &source_ip_hash16, request_trace_id(),
).await;
```

### CHAT emitter (message create)

```rust
// services/chat/src/capture.rs
use cyberos_capture::Capturer;
use cyberos_memory::interaction::event::*;
use uuid::Uuid;

pub async fn emit_message_created(
    cap: &Capturer, tenant: Uuid, author: Uuid, channel_id: &str, channel_kind: &str,
    message_id: &str, has_attachment: bool, source: SourceChannel, trace_id: Option<String>,
) {
    let target = match channel_kind {
        "dm" => TargetRef::Dm { id: channel_id.to_string() },
        _    => TargetRef::Channel { id: channel_id.to_string() },
    };
    let ev = InteractionEvent::builder(Module::Chat, "chat.message_created", EventClass::Content)
        .tenant(tenant).subject(author).occurred_now()
        .target(target)
        // §1 #4: pointer to chat's own row, NEVER the raw body.
        .content(ContentRef::Pointer { store: "chat_messages".into(), id: message_id.to_string() })
        .source(source).trace(trace_id)
        .attr("channel_kind", channel_kind).attr("has_attachment", has_attachment)
        .build();
    if let Ok(ev) = ev { let _ = cap.capture(&ev).await; }   // best-effort; send already happened
}
```

```rust
// services/chat/src/realtime.rs  (presence edge — §1 #8: emit only on 0↔1)
// `presence.join` already returns whether this open brought the subject online (count 0→1).
if st.presence.join(channel, me) {                            // returns true only on the 0→1 edge
    chat_capture::emit_presence(&st.capturer, tenant, me, channel, "online", None).await;
    // ... existing ChatEvent::Presence broadcast ...
}
```

### Turning ON the chat→brain link

```rust
// services/chat/src/main.rs  (the audit_pool already exists; DEC-2713 makes it the brain link)
let audit_pool = match std::env::var("CHAT_AUDIT_DATABASE_URL") {
    Ok(url) => Some(
        sqlx::postgres::PgPoolOptions::new().max_connections(4).connect(&url).await?,
    ),
    Err(_) => {
        // Was warn! ("logged, not chained"); now info! — but REQUIRED in production (runbook §1 #15).
        tracing::info!("CHAT_AUDIT_DATABASE_URL unset; chat interaction-events are logged, not chained \
                        (dev/local only — production MUST set this to the brain audit DB)");
        None
    }
};
// AppState gains: capturer: audit_pool.clone().map(cyberos_capture::Capturer::new)
```

```yaml
# deploy/vps/docker-compose.yml  (chat service env — DEC-2713)
    environment:
      # ... existing ...
      # The brain link: chat interaction-events chain into MEMORY's l1_audit_log (Supabase audit DB).
      CHAT_AUDIT_DATABASE_URL: ${SUPABASE_AUDIT_DATABASE_URL}
```

### Backfill

```rust
// services/memory/src/interaction/backfill.rs
use cyberos_memory::interaction::{emit, event::*, EmitOutcome};
use sqlx::PgPool;
use uuid::Uuid;

/// Replays recent chat history into chat.message_created interaction-events for acknowledged subjects.
/// Idempotent (§1 #10: UUIDv5 event_id over the source message id); dry-run unless `apply`.
pub async fn backfill_chat(
    chat_pool: &PgPool, audit_pool: &PgPool, tenant: Uuid, window_days: u32, apply: bool,
) -> anyhow::Result<BackfillReport> {
    let rows = recent_messages(chat_pool, tenant, window_days).await?;   // (id, author, channel, kind, ts_ns, has_attachment)
    let mut report = BackfillReport::default();
    for m in rows {
        let event_id = Uuid::new_v5(&CAPTURE_NAMESPACE, m.id.as_bytes());  // deterministic
        let ev = InteractionEvent {
            schema_version: SCHEMA_VERSION, event_id, tenant_id: tenant,
            subject_id: Some(m.author), occurred_at_ns: m.ts_ns,           // original time, not now
            module: Module::Chat, event_type: "chat.message_created".into(),
            event_class: EventClass::Content,
            target_ref: TargetRef::Channel { id: m.channel.clone() },
            content_ref: ContentRef::Pointer { store: "chat_messages".into(), id: m.id.clone() },
            session_id: None, trace_id: None, source_channel: SourceChannel::Import,
            attributes: Default::default(),
        };
        report.seen += 1;
        if !apply { continue; }
        match emit(audit_pool, &ev).await {                  // consent-gated + idempotent (§1 #11)
            Ok(EmitOutcome::Recorded { .. }) => report.recorded += 1,
            Ok(EmitOutcome::Skipped { .. })  => report.skipped_consent += 1,
            Err(_)                           => report.errors += 1,
        }
    }
    Ok(report)
}
```

---

## §4 — Acceptance criteria

1. **Shared emitter chains via emit()** — `Capturer::capture` of a built event inserts one `l1_audit_log` interaction-event row; no module writes the row directly (AC for §1 #1).
2. **AUTH signed_in emitted** — a successful token issue for an acknowledged subject produces one `auth.signed_in` interaction-event with `event_class='auth'` and `source_ip_hash16` in attributes (AC for §1 #3).
3. **AUTH sign_in_failed with null subject** — a failed attempt for an unresolved email produces `auth.sign_in_failed` with `subject_id=null` (AC for §1 #3).
4. **CHAT message_created is a pointer, not raw** — sending a message produces `chat.message_created` whose `content_ref` is `pointer{store:"chat_messages", id}`; the row contains no message text (AC for §1 #4).
5. **CHAT edit/delete emitted** — editing emits `chat.message_edited` (pointer); deleting emits `chat.message_deleted` with `content_ref:none` (AC for §1 #4).
6. **Channel + DM activity emitted** — creating/joining/leaving a channel and opening a DM each emit the matching event with the right `target_ref` (AC for §1 #4).
7. **Chat→brain link ON** — with `CHAT_AUDIT_DATABASE_URL` set to the brain DB, chat interaction-events appear in `l1_audit_log`; with it unset, chat logs the info line and writes none (AC for §1 #5).
8. **Consent gate honored by emitters** — an unacknowledged subject sending a message produces a `Skipped` outcome and zero rows; an acknowledged subject produces a row (AC for §1 #6).
9. **Capture never blocks the interaction** — with the audit pool down, the sign-in still issues its token and the message still sends; capture logs a warn (AC for §1 #7).
10. **Presence dedup to edges** — opening two tabs emits one `chat.presence_changed{online}`; closing the last emits one `{offline}`; the middle close emits none (AC for §1 #8).
11. **Backfill is idempotent** — running backfill `--apply` twice over the same window yields one interaction-event per source message (AC for §1 #9/#10).
12. **Backfill uses original timestamp** — a backfilled event's `occurred_at_ns` equals the source message's timestamp, not replay time (AC for §1 #10).
13. **Backfill respects consent** — a message by an unacknowledged subject is counted but not written during backfill (AC for §1 #11).
14. **Emitter metrics** — `memory_capture_emitter_calls_total{module="chat", event_type="chat.message_created", outcome="recorded"}` increments on a recorded message (AC for §1 #12).
15. **source_channel reflects entry point** — a web-console sign-in tags `web`; a backfilled event tags `import` (AC for §1 #13).
16. **trace_id correlation** — a message sent within a traced request carries that `trace_id`; a presence edge carries `null` (AC for §1 #14).
17. **Runbook documents the link + verify query** — `p0-google-chat-runbook.md` states the link is ON, the variable, the verify query, and the consent prerequisite (AC for §1 #15).
18. **Direct-write decoupling** — capture writes succeed with the memory HTTP service stopped (it writes to the audit DB directly) (AC for §1 #16).

---

## §5 — Verification

```rust
// services/auth/tests/capture_signin_test.rs
#[tokio::test]
async fn signin_emits_interaction_event_for_acknowledged_subject() {
    let env = AuthTestEnv::new().await;
    env.ack_notice(env.tenant(), env.alice()).await;
    let _token = env.password_grant(env.alice(), "web").await.unwrap();   // issues + captures
    let row: (String, String) = sqlx::query_as(
        "SELECT iev_event_type, iev_event_class FROM l1_audit_log
         WHERE event_type='memory.interaction_event' AND subject_id=$1")
        .bind(env.alice()).fetch_one(&env.audit_pool).await.unwrap();
    assert_eq!(row.0, "auth.signed_in");
    assert_eq!(row.1, "auth");
}

#[tokio::test]
async fn signin_for_unacknowledged_subject_captures_nothing() {
    let env = AuthTestEnv::new().await;                                   // no ack
    let _token = env.password_grant(env.bob(), "web").await.unwrap();     // token still issues
    let n: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM l1_audit_log WHERE event_type='memory.interaction_event' AND subject_id=$1")
        .bind(env.bob()).fetch_one(&env.audit_pool).await.unwrap();
    assert_eq!(n, 0);                                                     // gate held
}

#[tokio::test]
async fn signin_succeeds_even_when_audit_pool_down() {
    let env = AuthTestEnv::new_with_dead_audit_pool().await;
    env.ack_notice(env.tenant(), env.alice()).await;
    let token = env.password_grant(env.alice(), "web").await;             // must still succeed
    assert!(token.is_ok(), "sign-in must not fail because capture failed");
}
```

```python
# services/chat/tests/smoke_capture.py  (live chat service, brain link ON)
def test_message_create_edit_delete_emit_interaction_events(chat, audit_db, alice_token):
    ack_notice(audit_db, TENANT, ALICE)                       # pass the gate
    ch = chat.create_channel(alice_token, "general")
    msg = chat.send_message(alice_token, ch, "hello team")    # body stays in chat's DB
    chat.edit_message(alice_token, msg, "hello everyone")
    chat.delete_message(alice_token, msg)

    rows = audit_db.query(
        "SELECT iev_event_type, body::jsonb->'payload'->'content_ref'->>'kind' AS cref "
        "FROM l1_audit_log WHERE event_type='memory.interaction_event' AND iev_module='chat' "
        "AND subject_id=%s ORDER BY seq", (ALICE,))
    kinds = [r[0] for r in rows]
    assert "chat.channel_created" in kinds
    assert "chat.message_created" in kinds
    assert "chat.message_edited"  in kinds
    assert "chat.message_deleted" in kinds
    # created/edited point at chat's row; deleted has no content_ref
    created = next(r for r in rows if r[0] == "chat.message_created")
    assert created[1] == "pointer"                            # never raw body
    deleted = next(r for r in rows if r[0] == "chat.message_deleted")
    assert deleted[1] == "none"

def test_presence_dedup_to_edges(chat, audit_db, alice_token):
    ack_notice(audit_db, TENANT, ALICE)
    ch = chat.create_channel(alice_token, "general")
    ws1 = chat.connect_ws(alice_token, ch)                    # 0→1: emit online
    ws2 = chat.connect_ws(alice_token, ch)                    # 1→2: emit nothing
    ws2.close()                                               # 2→1: emit nothing
    ws1.close()                                               # 1→0: emit offline
    states = audit_db.query(
        "SELECT body::jsonb->'payload'->'attributes'->>'state' FROM l1_audit_log "
        "WHERE iev_event_type='chat.presence_changed' AND subject_id=%s ORDER BY seq", (ALICE,))
    assert [s[0] for s in states] == ["online", "offline"]
```

```rust
// services/memory/tests/interaction_backfill_test.rs
#[tokio::test]
async fn backfill_is_idempotent_and_keeps_original_time() {
    let env = BackfillTestEnv::new().await;
    env.ack_notice(env.tenant(), env.alice()).await;
    env.seed_chat_messages(env.alice(), 5, /*at*/ DAYS_AGO_3).await;

    let r1 = backfill_chat(&env.chat_pool, &env.audit_pool, env.tenant(), 30, true).await.unwrap();
    let r2 = backfill_chat(&env.chat_pool, &env.audit_pool, env.tenant(), 30, true).await.unwrap();
    assert_eq!(r1.recorded, 5);
    assert_eq!(r2.recorded, 0, "re-run is a no-op (idempotent event_id)");

    let n: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM l1_audit_log WHERE iev_event_type='chat.message_created' AND subject_id=$1")
        .bind(env.alice()).fetch_one(&env.audit_pool).await.unwrap();
    assert_eq!(n, 5);
    let ts: i64 = sqlx::query_scalar(
        "SELECT (body::jsonb->'payload'->>'occurred_at_ns')::bigint FROM l1_audit_log
         WHERE iev_event_type='chat.message_created' AND subject_id=$1 ORDER BY seq LIMIT 1")
        .bind(env.alice()).fetch_one(&env.audit_pool).await.unwrap();
    assert!(ts < now_ns() - 2 * DAY_NS, "backfilled at original time, not replay time");
}

#[tokio::test]
async fn backfill_skips_unacknowledged_authors() {
    let env = BackfillTestEnv::new().await;                   // bob never acknowledged
    env.seed_chat_messages(env.bob(), 3, DAYS_AGO_3).await;
    let r = backfill_chat(&env.chat_pool, &env.audit_pool, env.tenant(), 30, true).await.unwrap();
    assert_eq!(r.seen, 3);
    assert_eq!(r.recorded, 0);
    assert_eq!(r.skipped_consent, 3);
}
```

---

## §6 — Implementation skeleton

See §3 (shared crate, AUTH + CHAT emitters, the link flip, backfill). Each module's `capture.rs` is the per-module surface; `cyberos-capture` is the shared mechanism; everything routes through TASK-MEMORY-121 `emit()`.

---

## §7 — Dependencies

- **TASK-MEMORY-121** — the event shape, the typed builder, `emit()`, the consent gate, and the contract these emitters produce/obey. This task adds no shape; it produces TASK-MEMORY-121 events.
- **TASK-AUTH-002 / TASK-AUTH-004** — the subjects + sessions AUTH attributes events to; the existing audit pool + `emit_token_issued`/`_failed` call sites the new emitters sit next to.
- **TASK-CHAT-101** — the cyberos-chat service (channels/DMs/messages/attachments/presence) the chat emitters hook into; the existing `CHAT_AUDIT_DATABASE_URL` / `audit_pool` plumbing DEC-2713 turns on.
- **TASK-EVAL-001** — owns the acknowledgment ledger the consent gate reads (via TASK-MEMORY-121); capture for a subject begins only after acknowledgment.
- **TASK-OBS-003** — the traceparent `trace_id` emitters attach when a request trace is in scope.
- **Downstream:** TASK-MEMORY-123 (ingestion consumes the events these emitters produce); TASK-APP-005 (the audit viewer surfaces them).
- Crates: `sqlx@0.7`, `serde`, `uuid` (v5 + v7), `chrono`, `tracing`, `cyberos-memory` (interaction module), `cyberos-capture` (new).

---

## §8 — Example payloads

### auth.signed_in (emitted by AUTH next to the token row)

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
    "session_id": null,
    "trace_id": "0af7651916cd43dd8448eb211c80319c",
    "source_channel": "web",
    "attributes": { "method": "oidc", "source_ip_hash16": "9f86d081884c7d65" }
  }
}
```

### chat.message_created (emitted by CHAT; body stays in chat's DB)

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

### Backfill report (operator output)

```text
backfill_chat tenant=cyberskill window_days=30 apply=true
  seen=412  recorded=388  skipped_consent=24  errors=0
  (24 messages authored by subjects who have not acknowledged the notice — not captured)
```

---

## §9 — Open questions

All resolved. Deferred:
- PROJ / EMAIL / APP / MCP emitters — added as those modules come online, each via its own `capture.rs` against this task's contract (the emitter convention is defined here; the wiring lands per module).
- A batched/async emit channel for very high-frequency emitters — slice 3; the synchronous best-effort path is sufficient for AUTH + CHAT volumes at P0 scale.
- Backfilling AUTH sign-in history — not meaningful (sign-ins were not durably recorded pre-link beyond the token rows); chat is the history worth replaying.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Brain audit pool unreachable | sqlx error in capture | `EmitError`; emitter logs warn; sign-in/message proceed | Self-heals; OBS alarm on sustained drop |
| `CHAT_AUDIT_DATABASE_URL` unset in prod | startup info line + runbook check | chat captures nothing; logged not chained | Operator sets the variable (runbook §1 #15) |
| Subject not acknowledged | `Skipped` outcome | no row; counted in skipped_consent metric | By design; subject acknowledges, capture begins |
| Emitter inlines raw body (bug) | TASK-MEMORY-121 size cap + body-bytes histogram | `Invalid` at the limit; metric flags below | Author fixes emitter to use content_ref |
| Presence emit on every websocket (bug) | edge-dedup in realtime.rs | only 0↔1 edges emit | By design (§1 #8) |
| Backfill re-run | deterministic event_id unique index | second run no-op | By design (§1 #10) |
| Backfill with `--apply` on huge window | dry-run-first + bounded window | operator sees count before writing | By design; operator narrows window |
| Capture latency on hot path | best-effort, off critical path | interaction unaffected; capture may lag/drop | By design (§1 #7) |
| Message deleted then capture replays | message_deleted has content_ref:none | no dangling pointer | By design (§1 #4) |
| trace_id fabricated for ws events | null-when-absent rule | trace_id null, never fake | By design (§1 #14) |
| Two emitters for one domain event (bug) | TASK-MEMORY-121 event_id unique index | one wins; SEV-3 warn | Author fixes call site |
| AUTH audit pool and brain pool differ | both are the same l1_audit_log DB | rows land together | By design (shared deployment) |
| Chat DB and audit DB out of sync (pointer dangles) | resolve-time check in consumer | consumer treats missing target as redacted | TASK-MEMORY-123 handles gracefully |
| Source_channel unknown (legacy client) | default mapping | falls back to `api` | Client sends explicit channel |
| Backfill author no longer a subject | subject FK | row skipped + counted | By design |
| Notice version bumped mid-backfill | gate re-reads | newly-unacknowledged authors skipped | By design (governance) |
| OBS exporter down | buffered then dropped | metric gap; rows still written | Restore TASK-OBS-001 |
| Websocket churn floods presence edges | each edge is one row | bounded by real online/offline transitions | By design |
| Capture added to a module without capture.rs | compile (no helper) | module simply emits nothing yet | Add the module's capture.rs |

---

## §11 — Implementation notes

- This task produces TASK-MEMORY-121 events; it defines no new shape. The shared `cyberos-capture` crate is the mechanism (build → `emit()`); each module's `capture.rs` is the thin domain translation. New modules add a `capture.rs`, not a new pattern — that is the whole DEC-2714 point.
- AUTH reuses its existing audit pool (the one `memory_bridge` already writes to). The `auth.signed_in` / `auth.sign_in_failed` emits sit literally next to `emit_token_issued` / `emit_token_failed` so the same request data (jti, subject, ip hash, reason) is in hand; no new connection, no new request plumbing.
- CHAT already had the brain link plumbing from P0 — it reads `CHAT_AUDIT_DATABASE_URL`, holds an `audit_pool`, and writes through `cyberos-audit-chain`. P0 deliberately left the variable unset (the warn line). DEC-2713 is therefore a configuration flip plus routing chat's capture through `emit()`, not new infrastructure. The deploy compose sets it to the Supabase audit DB.
- Message bodies never leave chat's database. The emitter writes `content_ref: pointer{store:"chat_messages", id}`; the BRAIN holds the pointer, chat holds the content under chat's RLS. A consumer that needs the text resolves the pointer for a viewer who already has access. `message_deleted` uses `content_ref:none` so there is no dangling pointer to a removed row.
- Presence is deduped to the online/offline edges in `realtime.rs`, where the `Presence` connection-count map already lives. `presence.join` returns true only on the 0→1 edge; the offline emit hangs off the symmetric 1→0 edge. A person with three tabs produces one `online`, not three.
- Backfill exists because chat has live production history from before the link was on, and because a person who acknowledges later deserves their acknowledged-window activity captured. It is idempotent (UUIDv5 `event_id` over the source message id), defaults to a dry-run that reports the count, and only writes on `--apply`. It routes through `emit()`, so historical capture obeys the same consent gate as live capture — you cannot backfill someone who never acknowledged.
- Capture is best-effort everywhere. A failed capture logs a `tracing::warn!` and is swallowed; the sign-in still issues its token, the message still sends. This is the same contract AUTH and CHAT already hold for their audit writes, kept so capture is invisible to the user whose work it records.
- Direct-write to the audit DB (not via the memory HTTP service) follows the existing AUTH `memory_bridge` / obs-services pattern: the services share one Postgres deployment, so capture writes go straight to `l1_audit_log` and never couple a critical-path interaction to memory's HTTP liveness.
- The runbook update is load-bearing for trust: "is capture live?" is one query (`count(*) ... WHERE iev_module='chat'`), and "why does this person show nothing?" has one answer (they have not acknowledged). The brain plan's trust argument depends on capture being visible and explainable.
- `eu_ai_act_risk_class: limited` and the personal-data handling are carried in the AI Risk Assessment below: this task is where employee work-activity actually starts being recorded, so the privacy posture (platform-only, content_ref-not-raw, consent-gated, per-tenant RLS, no autonomous decision) is stated explicitly here, not only in the schema task.

---

## AI Risk Assessment

- **Why limited:** this task is the point where real employee work-activity begins flowing into the BRAIN — sign-ins, presence, and chat activity for CyberSkill staff — which is the data foundation for the later AI-assisted evaluation (TASK-EVAL-003/004). The emitters run no model and make no decision, but they feed a limited-risk AI use (workplace evaluation support), so they are classified and governed as part of that chain.
- **Personal data captured, and the hard limits:** who signed in (and from where, as a hashed IP prefix), who was online when, and who created/edited/deleted which message in which channel/DM. That is platform work-interaction metadata plus pointers to content — NOT keystroke logging, NOT screen capture, NOT anything from private life (DEC-2711). Message bodies stay in chat's own database; the captured row carries a `content_ref` pointer, never the text.
- **Consent before capture (enforced, not promised):** every emitter writes only through TASK-MEMORY-121 `emit()`, which is hard-gated on the TASK-EVAL-001 notice acknowledgment. There is no emit path that bypasses the gate (hand-built audit rows are forbidden by `disallowed_tools`), so no subject is captured before they have acknowledged the monitoring notice — live or in backfill. This is disclosed monitoring made structural.
- **Minimisation + purpose limitation:** emitters carry only the bounded attributes their event needs (e.g. `channel_kind`, `method`, `source_ip_hash16`), reference content rather than copying it, and inherit TASK-MEMORY-121's 2 KiB attributes cap and body-bytes watch. The purpose is to record the work, not to surveil the person; the closed event vocabulary bounds what can be recorded.
- **Access + tenancy:** captured events are `l1_audit_log` rows and inherit the per-tenant RLS, so a tenant's records never cross to another. Who may read them (manager + HR + the employee for their own record) is enforced by the consuming surfaces (TASK-EVAL-005), not loosened here.
- **Operator transparency:** the runbook documents that the chat→brain link is ON, the exact variable, the one-line verify query, and the consent prerequisite — so capture is auditable and explainable rather than a silent background process.
- **No autonomous decision:** these emitters record interactions; they never score, rank, or decide anything about a person. Every consequential use runs through the human-in-the-loop evaluation workflow (TASK-EVAL-004).

---

*End of TASK-MEMORY-122.*
