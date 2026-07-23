---
id: TASK-CHAT-267
title: "In-app content reporting - report a message, an attachment, or a person"
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
eu_ai_act_risk_class: not_ai
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-07-11T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: chat
priority: p0
status: done
verify: T
phase: P0
milestone: P0 - store compliance (UGC controls)
slice: 1
owner: Stephen Cheng (CTO)
created: 2026-07-11
shipped: 2026-07-11
memory_chain_hash: null
related_tasks: [TASK-CHAT-268, TASK-CHAT-269, TASK-CHAT-101, TASK-CHAT-265]
depends_on: []
blocks: [TASK-CHAT-269]
source_pages:
  - docs/deploy/play-store-submission.md
  - https://support.google.com/googleplay/android-developer/answer/9876937
source_decisions:
  - "Play content rating declares user-to-user communication -> the app falls under Play's UGC policy, which requires an in-app report path."
language: Rust (axum + sqlx + tokio), TypeScript (React 19 + Vite)
service: cyberos/services/chat
new_files:
  - services/chat/migrations/0013_chat_reports.sql
  - services/chat/src/reports.rs
  - services/chat/tests/reports.rs
  - apps/web/src/components/ReportDialog.tsx
modified_files:
  - services/chat/src/lib.rs
  - apps/web/src/components/MessageRow.tsx
  - apps/web/src/components/MemberList.tsx
  - apps/web/src/i18n.ts
  - apps/web/src/styles.css
allowed_tools:
  - sqlx migrations against the chat schema
  - axum route registration inside services/chat
disallowed_tools:
  - any write to the auth schema
  - any client-side moderation decision
effort_hours: 16
subtasks:
  - "0013 migration: chat_reports + RLS + partial unique index (2h)"
  - "reports.rs: POST handler, reason enum, snapshot capture, rate limit (5h)"
  - "audit row chat.report_created + tests (2h)"
  - "ReportDialog + message overflow entry + member-list entry (4h)"
  - "i18n EN/VI + a11y pass (1h)"
  - "integration tests: dedup, rate limit, cross-tenant, deleted target (2h)"
risk_if_skipped: "Google Play requires an in-app report path for any app that carries user-generated content. CyberOS ships team chat, so the content-rating questionnaire must declare user-to-user communication, and declaring it without a report path is a policy violation that gets the submission rejected. Declining to declare it is worse: the app is pulled after it is live, with the developer account at risk. Without this task the Play submission cannot honestly proceed."
---

## §1 - Description (BCP-14 normative)

1. The chat service **MUST** expose `POST /v1/chat/reports`, authenticated by the same CyberOS token as every other chat route, which records a report against one of three target kinds: a message, an attachment, or a person (a subject).

2. A report **MUST** carry a `reason` drawn from a closed set: `spam | harassment | hate | sexual | violence | self_harm | illegal | other`. The set is closed because TASK-CHAT-269's moderation queue groups and prioritises by reason, and a free-text reason cannot be grouped. A caller-supplied `detail` free-text field **MAY** accompany it, capped at 1000 characters.

3. The reporter **MUST** be a member of the channel that contains the reported message or attachment. A person who cannot see the content cannot report it. Reporting a *subject* (a person) does not require co-membership of any channel, because harassment can arrive by DM from someone whose channels you do not share.

4. On accepting a report the service **MUST** persist an immutable **snapshot** of the reported content: the message body (or the attachment's filename, content type, and size) exactly as it stood at report time. It **MUST NOT** rely on reading the content back at review time. The sender can edit or soft-delete a message after it is reported, and a moderation queue that renders "(deleted)" for every report it receives is not a moderation queue.

5. A report **MUST NOT** notify, or in any way become visible to, the reported person. The response body **MUST NOT** disclose whether the same target has been reported before, and **MUST NOT** carry any reporter-identifying field.

6. The service **MUST** deduplicate open reports: at most one `open` report may exist per `(tenant_id, reporter_subject_id, target)` triple. A second submission against the same target by the same reporter while a report is still `open` **MUST** return `200 OK` with the existing report's id, not `409`. From the reporter's point of view pressing Report twice is not an error, and a 409 leaks that a prior report exists.

7. The service **MUST** rate-limit report creation to 20 reports per subject per rolling hour, returning `429 Too Many Requests` beyond that. The limit exists to make report-spam as a harassment vector uneconomic, not to ration legitimate use; 20/hour is far above any honest rate.

8. Every accepted report **MUST** emit exactly one `chat.report_created` audit row via `audit::emit`, carrying `report_id`, `target_kind`, `reason`, `channel_id` (nullable), and the reporter's `subject_id` as the actor. It **MUST NOT** carry the snapshot body: the audit chain is replicated to the memory module, and copying reported content into it doubles the blast radius of the very content someone asked us to remove.

9. Reports **MUST** be tenant-scoped and protected by row-level security with both `USING` and `WITH CHECK`, in line with every other chat table. A report raised in one workspace **MUST NOT** be readable from another under any query.

10. The web client **MUST** offer a report entry point in two places: the message overflow menu (for a message, and for its attachment if it has one), and the member list / profile popover (for a person). Both **MUST** open the same dialog.

11. The report dialog **MUST** be reachable by keyboard alone, **MUST** trap focus while open, and **MUST** announce its result to assistive technology. A moderation control that only a mouse user can reach is not a moderation control.

12. Every string the dialog renders **MUST** ship in both `en` and `vi`. A Vietnamese-speaking employee reporting harassment in English is a failure of the product, not of the employee.

13. The service **MUST NOT** take any automated action on the reported content. It does not hide, delete, or flag the message. It records. Deciding what happens is TASK-CHAT-269's job and a human's decision.

14. The client **SHOULD** confirm submission with a non-blocking toast and close the dialog. It **MUST NOT** render the report's id, status, or any downstream state to the reporter; there is no user-facing report history in this slice.

## §2 - Why this design (rationale for humans)

**Why a closed reason set (§1 #2)?** Google's UGC policy is satisfied by "a mechanism to report", but the mechanism has to be usable by whoever reviews the report. TASK-CHAT-269 sorts an admin's queue by severity, and severity is a function of reason. Free text cannot be sorted. The set chosen mirrors the categories every major platform converged on, so it maps cleanly onto the content-rating questionnaire's own vocabulary.

**Why snapshot the content at report time (§1 #4)?** This is the clause the whole task turns on. `chat_messages` supports edit (`edited_at`) and soft delete (`deleted_at`), both available to the sender. Without a snapshot, the obvious abuse is: post something abusive, wait for the report, delete it, and the moderation queue shows an empty row while the recipient has already read it. The snapshot is the evidence. It is written once and never updated.

**Why does a second report return 200, not 409 (§1 #6)?** Two reasons, one usability and one security. Usability: a user who is not sure their tap registered will tap again, and greeting that with an error teaches them the feature is broken. Security: a distinct response for "already reported" is an oracle. Anyone could probe whether a given message has an outstanding report by reporting it and reading the status code. Returning the same shape either way closes that.

**Why can you report a person without sharing a channel (§1 #3)?** Because the DM path exists. `chat_channels.kind = 'direct'` lets any workspace member open a DM with any other. If reporting required co-membership of a *group* channel, the one place harassment is most likely - a DM from someone you do not work with - would be the one place you could not report it.

**Why exclude the snapshot from the audit row (§1 #8)?** The audit chain is hash-chained and replicated into the memory module, where it is designed to be durable and hard to rewrite. That is exactly right for "who did what when", and exactly wrong for a copy of content someone has asked us to consider removing. The audit row records that a report happened; the report row holds the evidence, under RLS, deletable when the report is resolved.

**Why no automated action (§1 #13)?** Auto-hiding on report is a self-service censorship button: one person can silence another with a tap. In an invite-only workspace of colleagues the correct arbiter is the workspace administrator, who knows the people involved. This also keeps CyberSkill out of the position of adjudicating a customer's internal dispute, which is what the published privacy policy already promises ("your organisation is the controller").

**Why rate-limit at 20/hour (§1 #7)?** The report table is writable by any authenticated member, so it is an amplification surface: a malicious member could bury an admin's queue under thousands of rows. 20/hour is roughly two orders of magnitude above the observed rate of any honest reporter and still bounds the queue.

## §3 - API contract

### Migration

```sql
-- services/chat/migrations/0013_chat_reports.sql
-- TASK-CHAT-267: in-app content reporting. One row per report. The snapshot columns are written once at
-- INSERT and never updated: the reported message can be edited or soft-deleted by its sender afterwards,
-- and a moderation queue that renders "(deleted)" for every row is not a moderation queue.

CREATE TABLE IF NOT EXISTS chat_reports (
    id                    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id             UUID NOT NULL,
    reporter_subject_id   UUID NOT NULL,

    target_kind           TEXT NOT NULL,
    target_message_id     UUID NULL REFERENCES chat_messages(id)    ON DELETE SET NULL,
    target_attachment_id  UUID NULL REFERENCES chat_attachments(id) ON DELETE SET NULL,
    target_subject_id     UUID NULL,
    channel_id            UUID NULL REFERENCES chat_channels(id)    ON DELETE SET NULL,

    reason                TEXT NOT NULL,
    detail                TEXT NULL,

    -- Evidence. Written at INSERT, never updated. See §1 #4.
    snapshot_body         TEXT NULL,
    snapshot_filename     TEXT NULL,
    snapshot_content_type TEXT NULL,
    snapshot_size_bytes   BIGINT NULL,
    snapshot_sender_id    UUID NULL,
    snapshot_taken_at     TIMESTAMPTZ NOT NULL DEFAULT now(),

    status                TEXT NOT NULL DEFAULT 'open',
    resolution            TEXT NULL,
    resolved_at           TIMESTAMPTZ NULL,
    resolved_by_subject_id UUID NULL,

    created_at            TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chat_reports_target_kind_enum
        CHECK (target_kind IN ('message', 'attachment', 'subject')),
    CONSTRAINT chat_reports_reason_enum
        CHECK (reason IN ('spam','harassment','hate','sexual','violence','self_harm','illegal','other')),
    CONSTRAINT chat_reports_status_enum
        CHECK (status IN ('open', 'actioned', 'dismissed')),
    CONSTRAINT chat_reports_detail_len
        CHECK (detail IS NULL OR char_length(detail) <= 1000),
    -- Exactly one target column is populated, and it matches target_kind.
    CONSTRAINT chat_reports_target_shape CHECK (
        (target_kind = 'message'    AND target_message_id    IS NOT NULL
                                    AND target_attachment_id IS NULL AND target_subject_id IS NULL) OR
        (target_kind = 'attachment' AND target_attachment_id IS NOT NULL
                                    AND target_message_id    IS NULL AND target_subject_id IS NULL) OR
        (target_kind = 'subject'    AND target_subject_id    IS NOT NULL
                                    AND target_message_id    IS NULL AND target_attachment_id IS NULL)
    ),
    -- A reporter cannot report themselves. Cheap guard against a confused client.
    CONSTRAINT chat_reports_not_self
        CHECK (target_subject_id IS NULL OR target_subject_id <> reporter_subject_id)
);

-- §1 #6: at most one OPEN report per (tenant, reporter, target). Resolved reports do not block a
-- new one - the same person can misbehave twice.
CREATE UNIQUE INDEX IF NOT EXISTS chat_reports_open_uniq
    ON chat_reports (tenant_id, reporter_subject_id, target_kind,
                     COALESCE(target_message_id,    '00000000-0000-0000-0000-000000000000'::uuid),
                     COALESCE(target_attachment_id, '00000000-0000-0000-0000-000000000000'::uuid),
                     COALESCE(target_subject_id,    '00000000-0000-0000-0000-000000000000'::uuid))
    WHERE status = 'open';

CREATE INDEX IF NOT EXISTS chat_reports_queue_idx
    ON chat_reports (tenant_id, status, created_at DESC);
CREATE INDEX IF NOT EXISTS chat_reports_rate_idx
    ON chat_reports (reporter_subject_id, created_at DESC);

ALTER TABLE chat_reports ENABLE ROW LEVEL SECURITY;
ALTER TABLE chat_reports FORCE  ROW LEVEL SECURITY;
CREATE POLICY chat_reports_tenant_scoped ON chat_reports
    FOR ALL
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    );

GRANT SELECT, INSERT, UPDATE ON chat_reports TO cyberos_app;
GRANT SELECT ON chat_reports TO cyberos_ro;
```

### Types

```rust
// services/chat/src/reports.rs

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::{audit, auth, db, AppState};

/// Reports are created at 20/subject/hour (§1 #7). Above that the endpoint 429s.
const REPORT_RATE_LIMIT_PER_HOUR: i64 = 20;
const DETAIL_MAX_CHARS: usize = 1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetKind {
    Message,
    Attachment,
    Subject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    Spam,
    Harassment,
    Hate,
    Sexual,
    Violence,
    SelfHarm,
    Illegal,
    Other,
}

impl Reason {
    /// Explicit metric/audit label. NEVER Debug-format an enum into a label
    /// (discipline §8.6a): Debug output is not a stable wire format.
    pub fn as_label(&self) -> &'static str {
        match self {
            Reason::Spam => "spam",
            Reason::Harassment => "harassment",
            Reason::Hate => "hate",
            Reason::Sexual => "sexual",
            Reason::Violence => "violence",
            Reason::SelfHarm => "self_harm",
            Reason::Illegal => "illegal",
            Reason::Other => "other",
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateReport {
    pub target_kind: TargetKind,
    #[serde(default)]
    pub target_message_id: Option<Uuid>,
    #[serde(default)]
    pub target_attachment_id: Option<Uuid>,
    #[serde(default)]
    pub target_subject_id: Option<Uuid>,
    pub reason: Reason,
    #[serde(default)]
    pub detail: Option<String>,
}

/// The ONLY thing the reporter gets back. No status, no history, no count.
/// §1 #5: the response must not become an oracle for prior reports.
#[derive(Debug, Serialize)]
pub struct ReportAccepted {
    pub id: Uuid,
}
```

### Endpoint

```
POST /v1/chat/reports
Authorization: Bearer <cyberos token>
Content-Type: application/json

201 Created  { "id": "<uuid>" }   -- new report
200 OK       { "id": "<uuid>" }   -- an open report by this reporter against this target already exists
400          "unknown reason" | "detail is too long" | "target shape does not match target_kind"
401          token invalid
403          "not a channel member"   -- message/attachment targets only
404          "no such message" | "no such attachment"
429          "too many reports"
```

### Handler skeleton

```rust
pub async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateReport>,
) -> Result<(StatusCode, Json<ReportAccepted>), (StatusCode, String)> {
    let claims = auth::verify(&state, &headers)
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let tenant = claims.tenant_id;
    let reporter = claims.subject_id;

    if let Some(d) = req.detail.as_deref() {
        if d.chars().count() > DETAIL_MAX_CHARS {
            return Err((StatusCode::BAD_REQUEST, "detail is too long".into()));
        }
    }

    let mut tx = db::begin_tenant(&state.pool, tenant).await?;

    // §1 #7 - rate limit BEFORE any target lookup, so a rate-limited caller cannot use the
    // endpoint's 403/404 responses to probe which message ids exist.
    let recent: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM chat_reports
          WHERE reporter_subject_id = $1 AND created_at > now() - interval '1 hour'",
    )
    .bind(reporter)
    .fetch_one(&mut *tx)
    .await
    .map_err(db::internal)?;
    if recent >= REPORT_RATE_LIMIT_PER_HOUR {
        return Err((StatusCode::TOO_MANY_REQUESTS, "too many reports".into()));
    }

    // Resolve the target, enforce membership (§1 #3), and take the snapshot (§1 #4) in the
    // SAME transaction that inserts, so the snapshot cannot race an edit/delete.
    let snap = snapshot_target(&mut tx, tenant, reporter, &req).await?;

    let row: Option<(Uuid,)> = sqlx::query_as(
        "INSERT INTO chat_reports
            (tenant_id, reporter_subject_id, target_kind, target_message_id, target_attachment_id,
             target_subject_id, channel_id, reason, detail, snapshot_body, snapshot_filename,
             snapshot_content_type, snapshot_size_bytes, snapshot_sender_id)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)
         ON CONFLICT DO NOTHING
         RETURNING id",
    )
    .bind(tenant).bind(reporter)
    .bind(kind_str(req.target_kind))
    .bind(req.target_message_id).bind(req.target_attachment_id).bind(req.target_subject_id)
    .bind(snap.channel_id)
    .bind(req.reason.as_label()).bind(req.detail.as_deref())
    .bind(snap.body.as_deref()).bind(snap.filename.as_deref())
    .bind(snap.content_type.as_deref()).bind(snap.size_bytes)
    .bind(snap.sender_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db::internal)?;

    // §1 #6 - the partial unique index fired: an open report already exists. Return it with 200,
    // NOT 409. Same shape either way; the caller learns nothing it did not already know.
    let (id, code) = match row {
        Some((id,)) => (id, StatusCode::CREATED),
        None => (existing_open_report(&mut *tx, tenant, reporter, &req).await?, StatusCode::OK),
    };

    tx.commit().await.map_err(db::internal)?;

    // §1 #8 - audit AFTER commit, and WITHOUT the snapshot body.
    audit::emit(
        &state, tenant, reporter, "chat.report_created",
        json!({
            "report_id":   id,
            "target_kind": kind_str(req.target_kind),
            "reason":      req.reason.as_label(),
            "channel_id":  snap.channel_id,
        }),
    ).await;

    Ok((code, Json(ReportAccepted { id })))
}
```

### Route registration

```rust
// services/chat/src/lib.rs
    .route("/v1/chat/reports", post(reports::create))
```

## §4 - Acceptance criteria

1. **A message report is stored with its snapshot** - posting a report against a message persists `snapshot_body` equal to the message body at that instant, and `snapshot_sender_id` equal to its sender.
2. **The snapshot survives a delete** - soft-deleting the reported message afterwards leaves `chat_reports.snapshot_body` unchanged and non-null.
3. **The snapshot survives an edit** - editing the reported message afterwards leaves `chat_reports.snapshot_body` holding the *original* text.
4. **Non-members are refused** - a subject who is not a member of the message's channel receives `403`, and no row is written.
5. **A subject report needs no shared channel** - reporting a person with whom the reporter shares no channel succeeds with `201`.
6. **Self-report is refused** - `target_kind: subject` with `target_subject_id == reporter` is rejected by the DB constraint and surfaces as `400`.
7. **Duplicate reports return 200, not 409** - a second report by the same reporter against the same target while the first is `open` returns `200` with the first report's id, and creates no second row.
8. **A resolved report does not block a new one** - after the first report is set to `dismissed`, the same reporter can raise a new `open` report against the same target.
9. **Rate limit fires at 20/hour** - the 21st report by one subject inside an hour returns `429`, and the check runs before any target lookup.
10. **Exactly one audit row per accepted report** - `chat.report_created` is emitted once, carries `report_id`, `target_kind`, `reason`, and does **not** carry `snapshot_body` or `detail`.
11. **No audit row on a rejected report** - a 400/403/404/429 emits nothing.
12. **Cross-tenant isolation holds** - a report created in tenant A is invisible to any query executed with tenant B's GUC set, including a direct `SELECT *`.
13. **An unknown reason is rejected** - a body carrying `reason: "because"` returns `400` and writes nothing.
14. **Detail is capped** - a 1001-character `detail` returns `400`.
15. **The dialog is keyboard-operable** - the report dialog can be opened, filled, submitted, and dismissed without a pointer, and focus returns to the invoking control on close.
16. **Both locales render** - every string in the dialog resolves in `en` and in `vi`; no key falls back to its own name.

## §5 - Verification

```rust
// services/chat/tests/reports.rs

#[tokio::test]
async fn snapshot_survives_delete_and_edit() {                      // AC 1, 2, 3
    let app = harness().await;
    let ch = app.channel("general", &[ALICE, BOB]).await;
    let msg = app.post_as(BOB, ch, "something abusive").await;

    let r = app.report_as(ALICE, json!({
        "target_kind": "message", "target_message_id": msg, "reason": "harassment"
    })).await;
    assert_eq!(r.status(), StatusCode::CREATED);

    app.edit_as(BOB, msg, "actually something nice").await;
    app.delete_as(BOB, msg).await;

    let (body, sender): (String, Uuid) = sqlx::query_as(
        "SELECT snapshot_body, snapshot_sender_id FROM chat_reports WHERE target_message_id = $1")
        .bind(msg).fetch_one(app.pool()).await.unwrap();
    assert_eq!(body, "something abusive");   // NOT the edited text, NOT null
    assert_eq!(sender, BOB);
}

#[tokio::test]
async fn non_member_cannot_report_a_message() {                     // AC 4
    let app = harness().await;
    let ch = app.channel("private", &[BOB]).await;
    let msg = app.post_as(BOB, ch, "hello").await;

    let r = app.report_as(ALICE, json!({
        "target_kind": "message", "target_message_id": msg, "reason": "spam"
    })).await;
    assert_eq!(r.status(), StatusCode::FORBIDDEN);
    assert_eq!(app.count_reports().await, 0);
}

#[tokio::test]
async fn subject_report_needs_no_shared_channel() {                 // AC 5
    let app = harness().await;                                      // ALICE and BOB share nothing
    let r = app.report_as(ALICE, json!({
        "target_kind": "subject", "target_subject_id": BOB, "reason": "harassment"
    })).await;
    assert_eq!(r.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn duplicate_report_is_idempotent_not_conflict() {            // AC 7
    let app = harness().await;
    let ch = app.channel("general", &[ALICE, BOB]).await;
    let msg = app.post_as(BOB, ch, "spam spam spam").await;
    let body = json!({"target_kind":"message","target_message_id":msg,"reason":"spam"});

    let first  = app.report_as(ALICE, body.clone()).await;
    let second = app.report_as(ALICE, body.clone()).await;

    assert_eq!(first.status(),  StatusCode::CREATED);
    assert_eq!(second.status(), StatusCode::OK);          // NOT 409 - see §2
    assert_eq!(first.json::<Value>().await["id"], second.json::<Value>().await["id"]);
    assert_eq!(app.count_reports().await, 1);
}

#[tokio::test]
async fn resolved_report_does_not_block_a_new_one() {               // AC 8
    let app = harness().await;
    let ch  = app.channel("general", &[ALICE, BOB]).await;
    let msg = app.post_as(BOB, ch, "again").await;
    let body = json!({"target_kind":"message","target_message_id":msg,"reason":"spam"});

    app.report_as(ALICE, body.clone()).await;
    app.set_report_status(msg, "dismissed").await;

    let again = app.report_as(ALICE, body).await;
    assert_eq!(again.status(), StatusCode::CREATED);
    assert_eq!(app.count_reports().await, 2);
}

#[tokio::test]
async fn rate_limit_fires_before_target_lookup() {                  // AC 9
    let app = harness().await;
    for i in 0..20 {
        let r = app.report_as(ALICE, json!({
            "target_kind":"subject","target_subject_id": app.filler_subject(i),"reason":"spam"
        })).await;
        assert_eq!(r.status(), StatusCode::CREATED);
    }
    // 21st, against a message id that does NOT exist. If the limit were checked after the
    // lookup this would 404 and leak that the id is unknown.
    let r = app.report_as(ALICE, json!({
        "target_kind":"message","target_message_id": Uuid::new_v4(),"reason":"spam"
    })).await;
    assert_eq!(r.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn audit_row_is_emitted_once_and_carries_no_content() {       // AC 10, 11
    let app = harness().await;
    let ch  = app.channel("general", &[ALICE, BOB]).await;
    let msg = app.post_as(BOB, ch, "abusive text here").await;

    app.report_as(ALICE, json!({
        "target_kind":"message","target_message_id":msg,"reason":"hate","detail":"private note"
    })).await;

    let rows = app.audit_rows("chat.report_created").await;
    assert_eq!(rows.len(), 1);
    let p = &rows[0]["payload"];
    assert_eq!(p["reason"], "hate");
    assert_eq!(p["target_kind"], "message");
    assert!(p.get("snapshot_body").is_none());
    assert!(p.get("detail").is_none());
    assert!(!rows[0].to_string().contains("abusive text here"));
    assert!(!rows[0].to_string().contains("private note"));

    // Rejected reports emit nothing.
    app.report_as(ALICE, json!({"target_kind":"message",
        "target_message_id": Uuid::new_v4(), "reason":"spam"})).await;
    assert_eq!(app.audit_rows("chat.report_created").await.len(), 1);
}

#[tokio::test]
async fn reports_are_tenant_isolated() {                            // AC 12
    let app = harness().await;
    let id  = app.seed_report_in(TENANT_A).await;
    let leaked: Option<(Uuid,)> = app.as_tenant(TENANT_B, |tx| {
        sqlx::query_as("SELECT id FROM chat_reports WHERE id = $1").bind(id).fetch_optional(tx)
    }).await.unwrap();
    assert!(leaked.is_none());
}

#[tokio::test]
async fn closed_enums_and_caps_are_enforced() {                     // AC 6, 13, 14
    let app = harness().await;
    assert_eq!(app.report_raw(ALICE, r#"{"target_kind":"message","target_message_id":"…","reason":"because"}"#)
        .await.status(), StatusCode::BAD_REQUEST);
    assert_eq!(app.report_as(ALICE, json!({
        "target_kind":"subject","target_subject_id": ALICE, "reason":"spam"})).await.status(),
        StatusCode::BAD_REQUEST);
    assert_eq!(app.report_as(ALICE, json!({
        "target_kind":"subject","target_subject_id": BOB, "reason":"spam",
        "detail": "x".repeat(1001)})).await.status(), StatusCode::BAD_REQUEST);
}
```

```tsx
// apps/web/src/components/__tests__/ReportDialog.test.tsx          // AC 15, 16
test("dialog is operable by keyboard alone and returns focus", async () => {
  const { getByRole } = render(<MessageRow message={msg} />);
  const trigger = getByRole("button", { name: /more actions/i });
  trigger.focus();
  await userEvent.keyboard("{Enter}");
  await userEvent.keyboard("{ArrowDown}{Enter}");         // Report
  expect(getByRole("dialog", { name: /report/i })).toHaveFocus();
  await userEvent.keyboard("{Escape}");
  expect(trigger).toHaveFocus();
});

test("every dialog string resolves in en and vi", () => {
  for (const locale of ["en", "vi"] as const) {
    for (const key of REPORT_DIALOG_KEYS) {
      expect(t(locale, key)).not.toEqual(key);            // no key-as-fallback
    }
  }
});
```

## §6 - Implementation skeleton

(The API contract in §3 is the skeleton. `snapshot_target` is the only helper worth spelling out, because the membership check and the snapshot must happen in the same transaction as the insert - see §10 row 3.)

```rust
struct Snapshot {
    channel_id:   Option<Uuid>,
    body:         Option<String>,
    filename:     Option<String>,
    content_type: Option<String>,
    size_bytes:   Option<i64>,
    sender_id:    Option<Uuid>,
}

async fn snapshot_target(
    tx: &mut sqlx::PgConnection,
    tenant: Uuid,
    reporter: Uuid,
    req: &CreateReport,
) -> Result<Snapshot, (StatusCode, String)> {
    match req.target_kind {
        TargetKind::Message => {
            let id = req.target_message_id
                .ok_or((StatusCode::BAD_REQUEST, "target shape does not match target_kind".to_string()))?;
            // FOR SHARE: hold the message row for the life of the tx so a concurrent edit cannot
            // land between the read and the insert (§10 row 3).
            let row: Option<(Uuid, Uuid, String)> = sqlx::query_as(
                "SELECT channel_id, sender_subject_id, body FROM chat_messages
                  WHERE id = $1 AND tenant_id = $2 FOR SHARE")
                .bind(id).bind(tenant).fetch_optional(&mut *tx).await.map_err(db::internal)?;
            let (channel_id, sender, body) = row
                .ok_or((StatusCode::NOT_FOUND, "no such message".to_string()))?;
            require_member(&mut *tx, channel_id, reporter).await?;   // §1 #3 -> 403
            Ok(Snapshot { channel_id: Some(channel_id), body: Some(body),
                          sender_id: Some(sender), filename: None,
                          content_type: None, size_bytes: None })
        }
        TargetKind::Attachment => { /* same shape against chat_attachments */ }
        TargetKind::Subject => {
            let id = req.target_subject_id
                .ok_or((StatusCode::BAD_REQUEST, "target shape does not match target_kind".to_string()))?;
            // No membership check by design (§1 #3). No snapshot: the target IS the person.
            let _ = id;
            Ok(Snapshot { channel_id: None, body: None, filename: None,
                          content_type: None, size_bytes: None, sender_id: None })
        }
    }
}
```

## §7 - Dependencies

- **Upstream:** none. `chat_messages`, `chat_attachments`, `chat_channel_members` and the tenant-GUC helper (`db::begin_tenant`) all exist from TASK-CHAT-101 and its successors.
- **Downstream:** TASK-CHAT-269 (moderation queue) reads `chat_reports` and is the only writer of `status`, `resolution`, `resolved_at`, `resolved_by_subject_id`. This task creates those columns but never transitions them.
- **Sibling:** TASK-CHAT-268 (blocking) is independent - a person can be blocked without being reported, and reported without being blocked. The two share only the member-list entry point in the client.
- **Cross-module:** `audit::emit` writes into the memory module's `l1_audit_log` when an audit pool is configured. No schema change is required there: `chat.report_created` is a new `event_type` value, not a new column.

## §8 - Example payloads

Report a message:

```json
POST /v1/chat/reports
{
  "target_kind": "message",
  "target_message_id": "6b1f4e0a-7c2d-4d19-9b0e-1f2a3c4d5e6f",
  "reason": "harassment",
  "detail": "Third time this week after I asked him to stop."
}

201 Created
{ "id": "9d2c8f31-0b44-4a6e-8f77-2c1e5a9b3d40" }
```

Report a person, no shared channel:

```json
POST /v1/chat/reports
{ "target_kind": "subject",
  "target_subject_id": "a0b1c2d3-e4f5-4607-8899-aabbccddeeff",
  "reason": "spam" }

201 Created
{ "id": "1a2b3c4d-5e6f-4708-99aa-bbccddeeff00" }
```

The audit row (note: no `snapshot_body`, no `detail`):

```json
{
  "event_type": "chat.report_created",
  "payload": {
    "report_id":   "9d2c8f31-0b44-4a6e-8f77-2c1e5a9b3d40",
    "target_kind": "message",
    "reason":      "harassment",
    "channel_id":  "3f5a1b2c-9d8e-4c7b-a6f5-e4d3c2b1a098"
  }
}
```

The stored row, after the reported message has been edited and deleted by its sender:

```json
{
  "id": "9d2c8f31-0b44-4a6e-8f77-2c1e5a9b3d40",
  "status": "open",
  "reason": "harassment",
  "snapshot_body": "the original abusive text",
  "snapshot_sender_id": "b7c8d9e0-1f2a-4b3c-8d4e-5f60718293a4",
  "snapshot_taken_at": "2026-07-11T05:14:22Z"
}
```

## §9 - Open questions

**Deferred:**

- *Reporter-visible report history* - "you reported this, here is what happened" is a real product need but it re-introduces the oracle problem (§2) and needs a notification surface. Deferred to a later slice; the columns needed already exist.
- *Report an entire channel* - useful for a workspace where an entire channel has gone bad. Out of scope: the target-shape constraint would need a fourth arm, and no store policy requires it.
- *Escalation to CyberSkill* - the current design has reports terminate with the workspace administrator, matching what the published privacy policy says about who controls workspace content. If we ever need cross-tenant abuse handling (a customer using CyberOS to harass another customer's members), that is a new task and a change to the privacy policy, in that order.

## §10 - Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Reporter edits/deletes the message between the read and the insert | `FOR SHARE` lock on the message row for the life of the tx | Snapshot is consistent with the row the reporter saw | None needed; the lock is the fix |
| Sender deletes the message after the report lands | `snapshot_body` is never updated | Moderation queue still shows the evidence | None needed; the snapshot is the record |
| Reported message is hard-deleted (channel dropped) | `ON DELETE SET NULL` on `target_message_id` | Report survives with a null target but a live snapshot | Queue renders from the snapshot, flags "original removed" |
| Reporter double-taps Report | Partial unique index on open reports | `200` with the first report's id; one row | None needed |
| Report-spam as a harassment vector | Rate limit counted per reporter per hour | `429` after 20/hour | Admin sees the reporter's volume in the queue and can act |
| Rate-limit check placed after target lookup | AC 9 asserts a 429, not a 404, on an unknown id | Would leak message-id existence to a rate-limited caller | Check is first statement in the tx |
| Report used as an existence oracle for messages | Both 403 (not a member) and 404 (no such id) exist and are distinguishable | A non-member could distinguish "exists but private" from "does not exist" | **Accepted risk, bounded:** membership is workspace-wide and the id space is a v4 UUID; brute-forcing it is not a practical attack. Revisit if channels ever become cross-tenant. |
| Reported person learns they were reported | No notification path is wired; response carries no reporter field | Reported person learns nothing | Enforced by AC 10 (audit payload) and by there being no fan-out |
| Snapshot content copied into the hash-chained audit log | AC 10 asserts the payload has no `snapshot_body` / `detail` | Reported content stays in one deletable place | Fails the test if a future edit adds it |
| Audit emit fails (memory pool down) | `audit::emit` logs a warning and returns | Report is still created; audit row is lost | Best-effort by existing module convention; the report row is the durable record |
| Cross-tenant read of a report | RLS `USING` + `WITH CHECK`; AC 12 | Zero rows | Policy is `FORCE`d, so even the table owner is subject to it |
| Client sends `target_kind: message` with a subject id | `chat_reports_target_shape` CHECK | `400` | Constraint is in the DB, not just the handler |
| A confused client reports the reporter | `chat_reports_not_self` CHECK | `400` | Constraint is in the DB |
| `detail` used to smuggle a payload into the admin UI | Cap at 1000 chars; React escapes on render | No injection | Queue renders `detail` as text, never as HTML (TASK-CHAT-269 AC) |
| Reports pile up unresolved | `chat_reports_queue_idx` on (tenant, status, created_at) | Queue query stays fast | TASK-CHAT-269 surfaces an open-count badge |

## §11 - Implementation notes

- **Why `FOR SHARE` and not `FOR UPDATE`.** We are not writing to `chat_messages`, only reading it consistently. `FOR SHARE` blocks a concurrent `UPDATE`/`DELETE` of that row for the life of our transaction without blocking other readers, which is exactly the guarantee the snapshot needs and nothing more. Reaching for `FOR UPDATE` here would serialise unrelated readers of a hot message row.

- **Why the rate-limit query is a `count(*)` and not a token bucket.** The chat service is stateless and horizontally scaled; an in-process bucket would be per-replica and therefore N times more permissive than advertised. A count against `chat_reports_rate_idx` is one index scan over at most 20 rows and is correct across replicas. If report volume ever justifies it, move to Redis - but not before.

- **`ON CONFLICT DO NOTHING` plus a follow-up SELECT, rather than `ON CONFLICT ... DO UPDATE ... RETURNING`.** The upsert form would touch `updated_at`-style columns and, more importantly, would let a duplicate submission overwrite the original snapshot with a fresh one - re-opening exactly the edit-then-report race the snapshot exists to close. Do-nothing preserves the first snapshot, which is the one that matters.

- **The nil-UUID `COALESCE` in the partial unique index.** Postgres treats `NULL` as distinct from `NULL` in a unique index, so an index over three nullable target columns would never fire. Coalescing the unused columns to `Uuid::nil()` (never a real subject id, per the module-wide convention) makes the triple comparable. This is the same nil-UUID convention the auth module uses for the root tenant.

- **Deliberately no `report_count` on the message.** A visible count would turn reporting into a voting mechanism and let a group pile onto one person. The queue aggregates by target internally; the client never sees it.

- **The client dialog does not pre-select a reason.** A pre-selected default is what a user submits when they are upset and clicking fast, which poisons the queue's prioritisation. Submit stays disabled until a reason is chosen.

*End of TASK-CHAT-267.*
