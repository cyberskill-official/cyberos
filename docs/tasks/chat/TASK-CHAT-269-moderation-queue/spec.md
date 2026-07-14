---
id: TASK-CHAT-269
title: "Workspace moderation queue - an administrator reviews reports, acts, and is audited"
module: CHAT
priority: MUST
status: done
class: product
verify: T
phase: P0
milestone: P0 - store compliance (UGC controls)
slice: 1
owner: Stephen Cheng (CTO)
created: 2026-07-11
shipped: 2026-07-12
memory_chain_hash: null
related_tasks: [TASK-CHAT-267, TASK-CHAT-268, TASK-CHAT-265, TASK-AUTH-101]
depends_on: [TASK-CHAT-267]
blocks: []
source_pages:
  - docs/deploy/play-store-submission.md
  - https://cyberskill.world/en/cyberos/privacy
source_decisions:
  - "Reports terminate with the workspace administrator, not with CyberSkill. This matches the published privacy policy, which names the customer organisation as the controller of its workspace content and CyberSkill as its processor."
language: Rust (axum + sqlx + tokio), TypeScript (React 19 + Vite)
service: cyberos/services/chat
new_files:
  - services/chat/src/moderation.rs
  - services/chat/tests/moderation.rs
  - services/chat/migrations/0015_chat_reports_retention.sql
  - apps/web/src/routes/Moderation.tsx
  - apps/web/src/components/ReportCard.tsx
modified_files:
  - services/chat/src/lib.rs
  - services/chat/src/auth.rs
  - apps/web/src/App.tsx
  - apps/web/src/components/SettingsPanel.tsx
  - apps/web/src/i18n.ts
allowed_tools:
  - axum route registration inside services/chat
  - reads of chat_reports, chat_messages, chat_attachments, chat_channel_members
disallowed_tools:
  - any automated moderation decision
  - any read of a DM thread beyond the reported message itself
effort_hours: 24
subtasks:
  - "role gate: tenant-admin / root-admin from the token's roles claim, fail-closed (2h)"
  - "GET queue: grouping by target, severity ordering, cursor pagination (5h)"
  - "GET detail: snapshot + conditional surrounding context (4h)"
  - "POST resolve: CAS on open, sibling auto-resolve, three actions (5h)"
  - "0015 migration: retention purge of resolved reports (1h)"
  - "client: Moderation route, ReportCard, content-policy link in Settings (5h)"
  - "integration tests incl. the role gate and the DM-context carve-out (2h)"
risk_if_skipped: "TASK-CHAT-267 gives users a button that files a report into a table nobody can read. A report path with no review path is worse than none: it tells the person being harassed that something will happen, and nothing does. Google Play's UGC policy requires that reported content is actually reviewed, and the app must link to a published content policy. Without this FR the store submission is dishonest even if it passes."
---

## §1 - Description (BCP-14 normative)

1. The chat service **MUST** expose three administrator routes: `GET /v1/chat/admin/reports` (the queue), `GET /v1/chat/admin/reports/:id` (one report in context), and `POST /v1/chat/admin/reports/:id/resolve` (act on it).

2. All three **MUST** be gated on the caller holding `tenant-admin` or `root-admin` in the token's `roles` claim, and the gate **MUST** fail closed: a token with no `roles` claim at all is refused with `403`, never granted. This is chat's first consumer of the TASK-AUTH-101 roles claim; there is no legacy path to preserve and no reason to be lenient.

3. Channel-level roles (`owner`, `manager`, `member`) **MUST NOT** grant access to the queue. A channel owner is not a workspace moderator, and a report raised in a channel they own may well be *about* them.

4. The queue **MUST** group reports by target: three people reporting the same message is one queue entry with `report_count: 3`, not three entries. Reviewing the same message three times is how a moderation queue trains its reviewer to stop reading it.

5. The queue **MUST** order by severity first and age second. Severity is derived from `reason` in a fixed rank: `self_harm > illegal > violence > sexual > hate > harassment > spam > other`. A `self_harm` report filed a minute ago outranks a `spam` report filed last week, and no amount of spam volume may push it down.

6. The queue **MUST** be filterable by `status` and by `reason`, and **MUST** paginate by an opaque cursor. It **MUST NOT** offer an unpaginated "all" mode.

7. The report detail **MUST** render the immutable snapshot captured by TASK-CHAT-267 §1 #4, and **MUST** mark plainly whether the original content still exists (`original_present: true | false`). A reviewer must be able to tell the difference between "this is what they said" and "this is what they said, and they have since deleted it" - the second is itself evidence.

8. Surrounding context (the messages immediately before and after the reported one) **MUST** be returned **only** when both of these hold: the target is in a **group** channel, and the administrator is **already a member of that channel**. Otherwise the response carries the snapshot alone.

9. In particular, a report about a **direct message MUST NOT** cause any part of that DM thread to be disclosed to the administrator beyond the single reported message. The reporter consented to disclosing *that message* by reporting it. They did not consent to handing their private correspondence to their employer, and a moderation feature that quietly does so is a surveillance feature wearing a safety badge.

10. The blocked-set from TASK-CHAT-268 **MUST NOT** be applied to any of these three routes. An administrator who has blocked someone must still be able to adjudicate a report about them.

11. `POST .../resolve` **MUST** accept exactly one `action` from a closed set - `dismiss | delete_message | remove_member` - plus an optional `note` (≤ 1000 chars) recording the reviewer's reasoning.

12. `resolve` **MUST** be a compare-and-swap on `status = 'open'`. If two administrators act on the same report concurrently, exactly one **MUST** win; the loser **MUST** receive `200` carrying the winning resolution, **MUST NOT** re-apply the action, and **MUST NOT** emit a second audit row.

13. When a report is resolved with `delete_message` or `remove_member`, every **other** open report against the same target **MUST** be resolved with the same action in the same transaction. A message can only be deleted once, and leaving five siblings open against a message that no longer exists is how a queue fills with ghosts.

14. `remove_member` **MUST** remove the reported subject from the channel the report came from - not from the workspace. Removing a person from the organisation is an identity operation that belongs to the AUTH module and to a human in HR, not to a chat moderation button.

15. Every resolution **MUST** emit exactly one `chat.report_resolved` audit row carrying `report_id`, `action`, `sibling_report_ids`, and the administrator as actor. The underlying effect (`chat.message_deleted`, `chat.member_removed`) is emitted by the existing handler, so a resolution that deletes a message produces **two** rows, and that is correct: one records the decision, one records the effect.

16. The audit row **MUST NOT** carry the snapshot body or the reviewer's `note`. Same reasoning as TASK-CHAT-267 §1 #8: the audit chain is the wrong place for content someone asked us to remove.

17. Resolved reports, **including their snapshots**, **MUST** be purged 90 days after resolution. This is not optional tidiness: the snapshot is a copy of content the reporter wanted gone, and 90 days matches the retention window already published at `cyberskill.world/en/cyberos/delete-account`.

18. The client **MUST** expose the queue only to administrators, and **MUST NOT** render a Moderation entry in the navigation for anyone else. A visible-but-403 route teaches everyone in the workspace that a moderation surface exists and that they are not trusted with it.

19. The client **MUST** link to the published content policy (`https://cyberskill.world/{locale}/cyberos/content-policy`) from Settings, and the report dialog from TASK-CHAT-267 **MUST** carry the same link. Google Play requires the policy to exist and to be reachable; a policy nobody can find is not a policy.

20. `note` and `detail` **MUST** be rendered as text, never as HTML or markdown. They are attacker-controlled strings displayed in an administrator's browser.

21. Every string **MUST** ship in `en` and `vi`.

## §2 - Why this design (rationale for humans)

**Why the DM carve-out (§1 #9).** This is the clause worth arguing about, so here is the argument. Alice reports a harassing DM from Bob. The administrator needs to see that message - obviously. Does the administrator need to see the rest of Alice and Bob's DM history? No. But it is the single easiest thing to build (fetch the channel, page the messages), and the most natural thing to want ("I need context"), and it would quietly convert a safety feature into an employer surveillance tool: file a report, and your private messages become readable by your boss. That is a betrayal of the person who came to us for help, and it is the reason people do not report. The snapshot is what they consented to disclose. That is all they get.

**Why the administrator must already be a channel member to see group context (§1 #8).** Same principle, weaker case. A private channel the admin is not in is not their business, and a report should not be a skeleton key to it. If they need the context, they can join the channel - visibly, as an act with its own audit row - rather than acquiring a silent read.

**Why grouping (§1 #4) and severity ordering (§1 #5) are normative and not "nice to have".** A moderation queue is only as good as the attention of the person reading it, and the failure mode is not that they reject a good report - it is that they stop opening the queue. Three duplicate entries for one message, and a `self_harm` report buried under forty spam reports, are the two mechanisms by which that happens. Both are structural and both are cheap to prevent, so both are MUSTs.

**Why `remove_member` removes from the channel, not the workspace (§1 #14).** Because firing someone is not a chat feature. A single click in a chat UI should not be able to sever a person's access to the organisation's systems. Removing them from the channel where the harm happened stops the harm; the rest is a conversation between humans.

**Why CAS on resolve (§1 #12), and why the loser gets a 200.** Two admins opening the queue at the same time is the normal case in a small workspace, not an edge case. Without CAS, `delete_message` runs twice - the second deletion is a no-op, but the *audit chain* now claims two people independently decided to delete the same message, which is false, and the audit chain is the artefact we ask people to trust. The loser gets `200` and the winning resolution, because from their point of view the outcome they wanted has happened; an error would be technically accurate and practically useless.

**Why resolved reports are purged after 90 days (§1 #17).** The snapshot exists because we could not trust the sender not to destroy the evidence. Once the report is resolved, that justification expires, and what remains is a durable copy of exactly the content someone asked us to remove, sitting in a table their employer can read. The published deletion page already commits to a 90-day window for security-relevant logs. This matches it, deliberately: one published number, one behaviour.

**Why the roles gate fails closed on a missing claim (§1 #2).** TASK-AUTH-101 allows a 30-day grace window where a token may carry no `rbac_v`. Chat has never read `roles` before, so it has no legacy tokens to be gentle with. A missing claim is not "unknown, therefore allow"; it is "unknown, therefore no". Being lenient here would mean every pre-TASK-AUTH-101 token in circulation is a moderator.

## §3 - API contract

### Retention migration

```sql
-- services/chat/migrations/0015_chat_reports_retention.sql
-- TASK-CHAT-269 §1 #17. The snapshot is a copy of content someone asked us to consider removing. It is
-- justified while the report is open (the sender can destroy the original) and unjustified once it is
-- resolved. 90 days matches the window published at cyberskill.world/en/cyberos/delete-account.

ALTER TABLE chat_reports
    ADD COLUMN IF NOT EXISTS purge_after TIMESTAMPTZ NULL;

CREATE INDEX IF NOT EXISTS chat_reports_purge_idx
    ON chat_reports (purge_after) WHERE purge_after IS NOT NULL;

-- The purge itself is a job, not a trigger: a trigger would delete rows mid-transaction while an admin
-- is reading them. Runs hourly; deletes the row outright (snapshot included), not just the snapshot,
-- because a resolved report with no evidence is not a record of anything.
```

### Routes

```
GET  /v1/chat/admin/reports?status=open&reason=harassment&cursor=<opaque>&limit=25
GET  /v1/chat/admin/reports/:id
POST /v1/chat/admin/reports/:id/resolve   { "action": "delete_message", "note": "…" }

403  "moderation requires tenant-admin"    -- including a token with NO roles claim
404  "no such report"                       -- and for a report in another tenant (never 403: do not
                                            --  confirm that an id exists in a workspace you cannot see)
200  on resolve, including the CAS loser (carries the winning resolution)
```

### The role gate

```rust
// services/chat/src/auth.rs

const MODERATOR_ROLES: [&str; 2] = ["tenant-admin", "root-admin"];

/// §1 #2 - fail closed. An absent `roles` claim is NOT a wildcard.
pub fn require_moderator(claims: &Claims) -> Result<(), (StatusCode, String)> {
    if claims.roles.iter().any(|r| MODERATOR_ROLES.contains(&r.as_str())) {
        return Ok(());
    }
    Err((StatusCode::FORBIDDEN, "moderation requires tenant-admin".to_string()))
}
```

### Severity

```rust
// services/chat/src/moderation.rs
// §1 #5. An explicit rank, not a Debug-ordered enum: the ordering is a product decision and must be
// legible to whoever changes it next.
fn severity_rank(reason: &str) -> i16 {
    match reason {
        "self_harm"  => 0,
        "illegal"    => 1,
        "violence"   => 2,
        "sexual"     => 3,
        "hate"       => 4,
        "harassment" => 5,
        "spam"       => 6,
        _            => 7,   // "other"
    }
}
```

### Queue query

```sql
-- §1 #4 grouping + §1 #5 ordering, in one pass. The GROUP BY is on the target triple, so three
-- reports against one message fold into one row carrying report_count = 3.
SELECT
    min(r.id)                      AS lead_report_id,
    r.target_kind,
    r.target_message_id,
    r.target_attachment_id,
    r.target_subject_id,
    count(*)                       AS report_count,
    min(severity_rank(r.reason))   AS severity,      -- worst reason wins the group
    max(r.created_at)              AS last_reported_at,
    array_agg(DISTINCT r.reason)   AS reasons
FROM chat_reports r
WHERE r.status = 'open'
  AND ($1::text IS NULL OR r.reason = $1)
GROUP BY r.target_kind, r.target_message_id, r.target_attachment_id, r.target_subject_id
ORDER BY severity ASC, last_reported_at DESC
LIMIT $2;
```

### Resolve

```rust
pub async fn resolve(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(req): Json<Resolve>,
) -> Result<(StatusCode, Json<Resolution>), (StatusCode, String)> {
    let claims = auth::verify(&state, &headers)
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    auth::require_moderator(&claims)?;                                    // §1 #2
    let (tenant, admin) = (claims.tenant_id, claims.subject_id);

    let mut tx = db::begin_tenant(&state.pool, tenant).await?;

    // §1 #12 - CAS. Exactly one concurrent resolver transitions the row.
    let won: Option<(String, Option<Uuid>, Option<Uuid>)> = sqlx::query_as(
        "UPDATE chat_reports
            SET status = $2, resolution = $3, resolved_at = now(),
                resolved_by_subject_id = $4, purge_after = now() + interval '90 days'
          WHERE id = $1 AND status = 'open'
        RETURNING target_kind, target_message_id, target_subject_id",
    )
    .bind(id)
    .bind(if req.action == Action::Dismiss { "dismissed" } else { "actioned" })
    .bind(req.action.as_label())
    .bind(admin)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db::internal)?;

    let Some((kind, msg_id, subj_id)) = won else {
        // Lost the race, or already resolved. Return the WINNING resolution with 200 - do not
        // re-apply the action, do not emit a second audit row (§1 #12).
        let existing = load_resolution(&mut *tx, id).await?
            .ok_or((StatusCode::NOT_FOUND, "no such report".to_string()))?;
        tx.commit().await.map_err(db::internal)?;
        return Ok((StatusCode::OK, Json(existing)));
    };

    // §1 #13 - close every sibling open report against the same target, in this transaction.
    let siblings: Vec<Uuid> = sqlx::query_scalar(
        "UPDATE chat_reports SET status = $2, resolution = $3, resolved_at = now(),
                resolved_by_subject_id = $4, purge_after = now() + interval '90 days'
          WHERE status = 'open' AND id <> $1
            AND target_kind = $5
            AND target_message_id IS NOT DISTINCT FROM $6
            AND target_subject_id IS NOT DISTINCT FROM $7
        RETURNING id",
    )
    .bind(id).bind("actioned").bind(req.action.as_label()).bind(admin)
    .bind(&kind).bind(msg_id).bind(subj_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(db::internal)?;

    match req.action {
        Action::Dismiss => {}
        Action::DeleteMessage => {
            let mid = msg_id.ok_or((StatusCode::BAD_REQUEST,
                "delete_message on a non-message report".to_string()))?;
            messages::soft_delete_as_moderator(&mut tx, tenant, admin, mid).await?;   // emits chat.message_deleted
        }
        Action::RemoveMember => {
            // §1 #14 - from the CHANNEL the report came from. Not from the workspace.
            members::remove_as_moderator(&mut tx, tenant, admin, report_channel(&mut *tx, id).await?,
                                         subj_id_or_sender(&mut *tx, id).await?).await?;
        }
    }

    tx.commit().await.map_err(db::internal)?;

    // §1 #15, #16 - one row for the DECISION. No snapshot, no note.
    audit::emit(&state, tenant, admin, "chat.report_resolved", json!({
        "report_id":          id,
        "action":             req.action.as_label(),
        "sibling_report_ids": siblings,
    })).await;

    Ok((StatusCode::OK, Json(Resolution { id, action: req.action, by: admin })))
}
```

### Detail response

```rust
#[derive(Serialize)]
pub struct ReportDetail {
    pub id: Uuid,
    pub reason: String,
    pub detail: Option<String>,
    pub reported_at: DateTime<Utc>,
    pub reporter_subject_id: Uuid,          // admins see who reported. The reported person never does.
    pub report_count: i64,

    /// The evidence (TASK-CHAT-267 §1 #4). Always present for message/attachment targets.
    pub snapshot_body: Option<String>,
    pub snapshot_sender_id: Option<Uuid>,
    pub snapshot_taken_at: DateTime<Utc>,

    /// §1 #7 - has the sender since removed it? That is itself evidence.
    pub original_present: bool,

    /// §1 #8, #9 - populated ONLY for a group-channel target whose channel the admin is already in.
    /// Always empty for a DM. There is no flag to override this.
    pub context: Vec<Message>,
}
```

## §4 - Acceptance criteria

1. **The role gate fails closed** - a token with `roles: []`, and a token with no `roles` claim at all, both receive `403` on all three routes.
2. **A channel owner is not a moderator** - a subject who is `owner` of the channel but holds no `tenant-admin` role receives `403`.
3. **Duplicate reports fold into one entry** - three reports against one message produce one queue row with `report_count: 3` and `reasons` listing all three.
4. **Severity outranks age** - a `self_harm` report filed one minute ago sorts above a `spam` report filed a week ago, and above forty of them.
5. **The queue is always paginated** - no parameter combination returns an unbounded list; `limit` above the cap is clamped.
6. **The snapshot renders even when the original is gone** - after the sender soft-deletes the reported message, the detail still returns `snapshot_body` in full and sets `original_present: false`.
7. **A DM report discloses only the reported message** - `context` is empty, and no other message from that DM appears anywhere in the response.
8. **Group context requires membership** - an admin who is not a member of the private channel gets `context: []`; an admin who is a member gets the surrounding messages.
9. **Blocks do not apply** - an admin who has blocked the reported person still sees the full snapshot and full context.
10. **CAS: exactly one winner** - two concurrent `resolve` calls on one report produce exactly one state change, one `chat.message_deleted`, and one `chat.report_resolved`; the loser receives `200` carrying the winner's action.
11. **Siblings are closed** - resolving one of five open reports against the same message with `delete_message` leaves zero open reports against that message and lists the four sibling ids in the audit row.
12. **`remove_member` removes from the channel only** - the reported subject loses that channel's membership and retains every other channel and their account.
13. **A resolution emits two audit rows, not one** - `chat.report_resolved` (the decision) and `chat.message_deleted` (the effect); a `dismiss` emits only the first.
14. **The audit row carries no content** - `chat.report_resolved` contains neither `snapshot_body` nor `note`.
15. **Resolved reports carry a purge date** - `purge_after` is set to resolution + 90 days on every resolution, including `dismiss`.
16. **Cross-tenant probing returns 404, not 403** - a valid admin of tenant A requesting a report id belonging to tenant B receives `404`.
17. **The Moderation route is invisible to non-admins** - the navigation entry is absent, not merely disabled, for a subject without the role.
18. **`note` and `detail` render as text** - a `note` containing `<img src=x onerror=alert(1)>` renders as literal characters.
19. **The content-policy link is present** - Settings and the report dialog both link to `/{locale}/cyberos/content-policy`.
20. **Both locales render** - every moderation string resolves in `en` and `vi`.

## §5 - Verification

```rust
// services/chat/tests/moderation.rs

#[tokio::test]
async fn role_gate_fails_closed() {                                  // AC 1, 2
    let app = harness().await;
    for tok in [app.token_with_roles(ALICE, &[]),
                app.token_without_roles_claim(ALICE),
                app.token_with_roles(ALICE, &["tenant-member"])] {
        assert_eq!(app.queue_with(tok).await.status(), StatusCode::FORBIDDEN);
    }
    // channel owner, no workspace role
    let ch = app.channel("general", &[ALICE]).await;                 // ALICE is owner
    assert_eq!(app.queue_with(app.token_with_roles(ALICE, &[])).await.status(),
               StatusCode::FORBIDDEN);
    let _ = ch;
    assert_eq!(app.queue_with(app.token_with_roles(ADMIN, &["tenant-admin"])).await.status(),
               StatusCode::OK);
}

#[tokio::test]
async fn severity_outranks_age_and_duplicates_fold() {               // AC 3, 4
    let app = harness().await;
    let ch = app.channel("general", &[ALICE, BOB, CAROL, DAVE]).await;
    let spam = app.post_as(BOB, ch, "buy now").await;
    for who in [ALICE, CAROL, DAVE] {                                 // 3 reports, 1 target
        app.report_as(who, json!({"target_kind":"message","target_message_id":spam,"reason":"spam"})).await;
    }
    app.backdate_reports(Duration::days(7)).await;

    let sh = app.post_as(BOB, ch, "…").await;
    app.report_as(ALICE, json!({"target_kind":"message","target_message_id":sh,"reason":"self_harm"})).await;

    let q = app.queue_as(ADMIN).await;
    assert_eq!(q.len(), 2, "three spam reports must fold into ONE entry");
    assert_eq!(q[0].target_message_id, Some(sh), "self_harm outranks week-old spam");
    assert_eq!(q[1].report_count, 3);
}

#[tokio::test]
async fn a_dm_report_discloses_only_the_reported_message() {         // AC 7 - the privacy property
    let app = harness().await;
    let dm = app.dm(ALICE, BOB).await;
    app.post_as(ALICE, dm, "private thing one").await;
    let bad = app.post_as(BOB, dm, "the abusive line").await;
    app.post_as(ALICE, dm, "private thing two").await;

    let r = app.report_as(ALICE, json!({
        "target_kind":"message","target_message_id":bad,"reason":"harassment"})).await.id();

    let detail = app.report_detail_as(ADMIN, r).await;
    assert_eq!(detail.snapshot_body.unwrap(), "the abusive line");
    assert!(detail.context.is_empty(), "a DM must disclose NO surrounding context");
    let body = detail.raw_json();
    assert!(!body.contains("private thing one"));
    assert!(!body.contains("private thing two"));
}

#[tokio::test]
async fn group_context_requires_membership() {                       // AC 8
    let app = harness().await;
    let ch  = app.channel("private-team", &[BOB, CAROL]).await;      // ADMIN is NOT a member
    let msg = app.post_as(BOB, ch, "reported").await;
    app.post_as(CAROL, ch, "surrounding chatter").await;
    let r = app.report_as(CAROL, json!({
        "target_kind":"message","target_message_id":msg,"reason":"hate"})).await.id();

    let d1 = app.report_detail_as(ADMIN, r).await;
    assert!(d1.context.is_empty());
    assert!(!d1.raw_json().contains("surrounding chatter"));

    app.join_channel(ADMIN, ch).await;                                // visible, audited act
    let d2 = app.report_detail_as(ADMIN, r).await;
    assert!(d2.context.iter().any(|m| m.body == "surrounding chatter"));
}

#[tokio::test]
async fn snapshot_renders_after_the_sender_deletes() {               // AC 6
    let app = harness().await;
    let ch  = app.channel("general", &[ALICE, BOB]).await;
    let msg = app.post_as(BOB, ch, "the evidence").await;
    let r = app.report_as(ALICE, json!({
        "target_kind":"message","target_message_id":msg,"reason":"harassment"})).await.id();

    app.delete_as(BOB, msg).await;

    let d = app.report_detail_as(ADMIN, r).await;
    assert_eq!(d.snapshot_body.unwrap(), "the evidence");
    assert!(!d.original_present);
}

#[tokio::test]
async fn concurrent_resolve_has_exactly_one_winner() {               // AC 10, 13
    let app = harness().await;
    let ch  = app.channel("general", &[ALICE, BOB]).await;
    let msg = app.post_as(BOB, ch, "delete me").await;
    let r = app.report_as(ALICE, json!({
        "target_kind":"message","target_message_id":msg,"reason":"hate"})).await.id();

    let body = json!({"action":"delete_message"});
    let (a, b) = tokio::join!(
        app.resolve_as(ADMIN,   r, body.clone()),
        app.resolve_as(ADMIN_2, r, body.clone()),
    );
    assert_eq!(a.status(), StatusCode::OK);
    assert_eq!(b.status(), StatusCode::OK);
    assert_eq!(app.audit_rows("chat.report_resolved").await.len(), 1);
    assert_eq!(app.audit_rows("chat.message_deleted").await.len(), 1);
}

#[tokio::test]
async fn siblings_close_and_the_audit_lists_them() {                 // AC 11, 14
    let app = harness().await;
    let ch  = app.channel("general", &[ALICE, BOB, CAROL, DAVE, ERIN]).await;
    let msg = app.post_as(BOB, ch, "abuse").await;
    let ids: Vec<_> = [ALICE, CAROL, DAVE, ERIN].iter()
        .map(|w| app.report_as(*w, json!({
            "target_kind":"message","target_message_id":msg,"reason":"hate"})))
        .collect::<FuturesOrdered<_>>().collect().await;

    app.resolve_as(ADMIN, ids[0], json!({"action":"delete_message","note":"clear breach"})).await;

    assert_eq!(app.open_reports_against(msg).await, 0);
    let row = &app.audit_rows("chat.report_resolved").await[0];
    assert_eq!(row["payload"]["sibling_report_ids"].as_array().unwrap().len(), 3);
    assert!(row.get("note").is_none());
    assert!(!row.to_string().contains("clear breach"));
    assert!(!row.to_string().contains("abuse"));
}

#[tokio::test]
async fn remove_member_touches_the_channel_only() {                  // AC 12
    let app = harness().await;
    let ch1 = app.channel("general", &[ALICE, BOB]).await;
    let ch2 = app.channel("random",  &[ALICE, BOB]).await;
    let msg = app.post_as(BOB, ch1, "abuse").await;
    let r = app.report_as(ALICE, json!({
        "target_kind":"message","target_message_id":msg,"reason":"harassment"})).await.id();

    app.resolve_as(ADMIN, r, json!({"action":"remove_member"})).await;

    assert!(!app.is_member(BOB, ch1).await);
    assert!( app.is_member(BOB, ch2).await, "removal is scoped to the channel");
    assert!( app.subject_exists(BOB).await, "removal is NOT an identity operation");
}

#[tokio::test]
async fn cross_tenant_probe_is_404_not_403() {                       // AC 16
    let app = harness().await;
    let r = app.seed_report_in(TENANT_B).await;
    let res = app.report_detail_raw(app.admin_of(TENANT_A), r).await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);   // NOT 403 - that would confirm the id exists
}

#[tokio::test]
async fn resolution_sets_the_purge_date() {                          // AC 15
    let app = harness().await;
    let r = app.seed_open_report().await;
    app.resolve_as(ADMIN, r, json!({"action":"dismiss"})).await;
    let purge: DateTime<Utc> = app.scalar("SELECT purge_after FROM chat_reports WHERE id=$1", r).await;
    assert!((purge - Utc::now() - Duration::days(90)).num_minutes().abs() < 5);
}
```

```tsx
// apps/web/src/routes/__tests__/Moderation.test.tsx                 // AC 17, 18, 19
test("the nav entry is absent, not disabled, for a non-admin", () => {
  const { queryByRole } = render(<App user={{ roles: ["tenant-member"] }} />);
  expect(queryByRole("link", { name: /moderation/i })).toBeNull();
});

test("a note is rendered as text, never as markup", () => {
  const { getByTestId } = render(
    <ReportCard report={{ note: '<img src=x onerror=alert(1)>' }} />);
  expect(getByTestId("note").textContent).toBe('<img src=x onerror=alert(1)>');
  expect(getByTestId("note").querySelector("img")).toBeNull();
});
```

## §6 - Implementation skeleton

(§3 is the skeleton. The purge job is the only piece not shown there.)

```rust
// Hourly. A job, not a trigger: a trigger would delete rows out from under an admin mid-read.
pub async fn purge_resolved_reports(pool: &PgPool) -> anyhow::Result<u64> {
    let n = sqlx::query("DELETE FROM chat_reports WHERE purge_after IS NOT NULL AND purge_after < now()")
        .execute(pool).await?.rows_affected();
    if n > 0 { tracing::info!(target: "cyberos_chat::moderation", purged = n, "resolved reports purged"); }
    Ok(n)
}
```

## §7 - Dependencies

- **Upstream:** TASK-CHAT-267 (`chat_reports` and its snapshot columns; this FR is the only writer of `status`, `resolution`, `resolved_at`, `resolved_by_subject_id`, `purge_after`).
- **Cross-module:** TASK-AUTH-101 supplies the `roles` claim. Chat is its first consumer; no auth-side change is required, but if the role catalogue ever renames `tenant-admin`, `MODERATOR_ROLES` moves with it.
- **Constrains:** TASK-CHAT-268 - §1 #10 here requires that the blocked-set is *not* applied to these routes. That is asserted from both sides (AC #9 here, AC #12 there).
- **External:** the content policy page at `cyberskill.world/{locale}/cyberos/content-policy` must exist before §1 #19's link is anything but a 404. It ships in the landing-page repo alongside the privacy and account-deletion pages, not here.

## §8 - Example payloads

Queue entry (three people reported one message):

```json
{
  "lead_report_id": "9d2c8f31-0b44-4a6e-8f77-2c1e5a9b3d40",
  "target_kind": "message",
  "target_message_id": "6b1f4e0a-7c2d-4d19-9b0e-1f2a3c4d5e6f",
  "report_count": 3,
  "severity": 4,
  "reasons": ["hate", "harassment"],
  "last_reported_at": "2026-07-11T05:31:02Z"
}
```

Detail for a DM report - note the empty context:

```json
{
  "id": "9d2c8f31-…",
  "reason": "harassment",
  "reporter_subject_id": "a0b1c2d3-…",
  "snapshot_body": "the abusive line",
  "snapshot_sender_id": "b7c8d9e0-…",
  "snapshot_taken_at": "2026-07-11T05:22:19Z",
  "original_present": false,
  "context": []
}
```

The two audit rows a `delete_message` resolution produces:

```json
{ "event_type": "chat.report_resolved",
  "payload": { "report_id": "9d2c8f31-…", "action": "delete_message",
               "sibling_report_ids": ["1a2b3c4d-…", "2b3c4d5e-…"] } }

{ "event_type": "chat.message_deleted",
  "payload": { "message_id": "6b1f4e0a-…", "by": "moderator" } }
```

## §9 - Open questions

**Deferred:**

- *Escalation to CyberSkill.* Today a report terminates with the workspace administrator, which is what the published privacy policy says ("your organisation is the controller"). If we ever need to handle abuse *by* an administrator, or abuse crossing tenants, that is a new FR **and** a privacy-policy change, in that order.
- *Administrator visibility of block counts.* TASK-CHAT-268 §9 raises it: "six people have blocked this person" is the strongest signal available in a small workspace, and surfacing it partially breaks the privacy of the block. It needs a decision from the operator, not a design. Not in this slice.
- *Reporter feedback ("your report was actioned").* Real need, but it re-introduces the oracle TASK-CHAT-267 §2 closed, and needs a notification surface. Deferred; the columns exist.
- *Moderation of AI-generated content.* CyberOS chat has AI features (`ai.rs`). A summary that reproduces abusive content is not currently reportable. Deferred; the target-shape constraint would need a fourth arm.

## §10 - Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Missing `roles` claim treated as "unknown, allow" | AC 1 tests a token with no claim at all | Every legacy token in circulation would be a moderator | `require_moderator` is a whitelist match; there is no else-allow branch |
| Channel owner assumed to be a moderator | AC 2 | The person a report is *about* could resolve it | Channel roles are never consulted by these routes |
| DM thread disclosed to the administrator | AC 7 asserts on the raw JSON, not just the field | A safety feature becomes employer surveillance; people stop reporting | `context` is unconditionally empty for `kind = 'direct'`; there is no override flag |
| Private-channel context disclosed to a non-member admin | AC 8 | A report becomes a skeleton key into channels the admin is not in | Context requires an existing membership row |
| Two admins resolve at once, action applied twice | AC 10 | Audit chain falsely records two independent decisions | CAS on `status = 'open'`; the loser returns the winner's resolution |
| Siblings left open against a deleted message | AC 11 | Queue fills with ghosts; reviewer stops trusting it | Siblings closed in the same transaction |
| `self_harm` buried under spam volume | AC 4 | The one report that mattered is never read | Severity is the primary sort key; volume cannot outrank it |
| Duplicate entries for one message | AC 3 | Reviewer fatigue - the real failure mode of every moderation queue | `GROUP BY` on the target triple |
| Snapshot preserved forever | AC 15 | A durable copy of the content someone wanted removed, readable by their employer | `purge_after` on every resolution; hourly purge job |
| Purge trigger deletes a row an admin is reading | Job, not trigger; runs hourly | No mid-read deletion | §6 |
| `remove_member` interpreted as "remove from workspace" | AC 12 asserts the subject still exists and keeps other channels | A chat button could sever someone's access to the organisation | `members::remove_as_moderator` is channel-scoped; identity lives in AUTH |
| Reviewer `note` used as an XSS vector | AC 18 | Script execution in the admin's browser | Rendered as text; React escapes by default and the test pins it |
| Cross-tenant report id probed via 403 vs 404 | AC 16 | Confirms a report id exists in a workspace the caller cannot see | RLS returns zero rows; the handler maps that to `404` |
| Blocked-set applied to the queue | AC 9 | Admin cannot read a report about someone they blocked | These routes never call `blocked_by` |
| Content-policy link 404s | AC 19 + the landing-page page must ship first | Play rejects: the policy must be reachable | Named as an external dependency in §7 |

## §11 - Implementation notes

- **Why the queue groups in SQL rather than in Rust.** The fold has to happen *before* the `LIMIT`, or page one shows three copies of one message and the reviewer never reaches page two. Grouping after pagination is the classic version of this bug and it is invisible in a test fixture with four rows.

- **Why severity is a function and not a column.** A column would have to be backfilled on every rank change, and the rank *is* a product decision that will change. `severity_rank` is one `match`, one place to edit, and the ordering is legible to whoever reads it next. If the queue ever grows past a size where sorting on a computed expression matters, the column is an additive change.

- **`IS NOT DISTINCT FROM` in the sibling update.** The target columns are nullable and two of the three are null on any given row. `=` yields NULL against NULL and the sibling update would silently match nothing - the same trap as the partial unique index in TASK-CHAT-267, in a different guise.

- **Why the admin sees the reporter's identity.** A moderation decision without knowing who is complaining is a decision made blind: three reports from one person who habitually reports their colleagues is a different situation from three reports by three people. The reporter is disclosed to the *administrator* and never to the reported person, which is the correct asymmetry.

- **The purge job deletes the row, not just the snapshot.** A resolved report stripped of its evidence is not a record of anything - it is a row asserting that something happened, with nothing to show for it. The audit chain already holds the decision (`chat.report_resolved`, which by design carries no content). That is the durable record; the report row is the working copy.

*End of TASK-CHAT-269.*
