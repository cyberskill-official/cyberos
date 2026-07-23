---
id: TASK-CHAT-268
title: "User blocking - stop seeing a person's content, and stop receiving their messages"
eu_ai_act_risk_class: not_ai
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
related_tasks: [TASK-CHAT-267, TASK-CHAT-269, TASK-CHAT-101]
depends_on: []
blocks: []
source_pages:
  - docs/deploy/play-store-submission.md
  - https://support.google.com/googleplay/android-developer/answer/9876937
source_decisions:
  - "Play's UGC policy requires an in-app way to block another user, alongside the report path (TASK-CHAT-267)."
language: Rust (axum + sqlx + tokio), TypeScript (React 19 + Vite)
service: cyberos/services/chat
new_files:
  - services/chat/migrations/0014_chat_blocks.sql
  - services/chat/src/blocks.rs
  - services/chat/tests/blocks.rs
  - apps/web/src/components/BlockedMessage.tsx
modified_files:
  - services/chat/src/lib.rs
  - services/chat/src/messages.rs
  - services/chat/src/realtime.rs
  - services/chat/src/notify.rs
  - services/chat/src/channels.rs
  - apps/web/src/components/MessageRow.tsx
  - apps/web/src/components/MemberList.tsx
  - apps/web/src/i18n.ts
allowed_tools:
  - sqlx migrations against the chat schema
  - axum route registration inside services/chat
disallowed_tools:
  - any client-only enforcement of a block
  - any notification to the blocked person
effort_hours: 20
subtasks:
  - "0014 migration: chat_blocks + RLS (1h)"
  - "blocks.rs: POST / DELETE / GET, self-block guard (3h)"
  - "enforcement: message list annotation (3h)"
  - "enforcement: realtime fan-out suppression (3h)"
  - "enforcement: notification + push fan-out suppression (3h)"
  - "enforcement: DM list hiding (2h)"
  - "client: collapsed message row with reveal, block/unblock entry points (3h)"
  - "integration tests incl. the four enforcement points (2h)"
risk_if_skipped: "Google Play requires an in-app block mechanism for any app carrying user-generated content. Reporting alone (TASK-CHAT-267) does not satisfy the policy: a report is a request to a third party, whereas a block is the control the affected person holds themselves. Without it the Play submission cannot honestly declare user-to-user communication, and the app is rejected."
---

## §1 - Description (BCP-14 normative)

1. The chat service **MUST** expose `POST /v1/chat/blocks` (block a person), `DELETE /v1/chat/blocks/:subject_id` (unblock), and `GET /v1/chat/blocks` (the caller's own block list). All three are scoped to the calling subject: a block belongs to the person who created it and nobody else can read, create, or remove it.

2. A block **MUST** be one-directional and private. It records "A no longer wishes to receive B's content". It **MUST NOT** notify B, appear in any list B can read, or change anything B observes about their own account. B continues to see the channel, continues to see their own messages in it, and is told nothing.

3. A subject **MUST NOT** be able to block themselves. The service returns `400`.

4. Blocking **MUST** be enforced on the server, at every one of these four fan-out points, and **MUST NOT** rely on the client filtering anything:
- the message-list query,
- the realtime (WebSocket) fan-out,
- the notification and push fan-out,
- the direct-message list. A block implemented only in the client is not a block; it is a CSS rule that anyone can open the network tab and defeat, and it fails the store policy it exists to satisfy.

5. In a **shared group channel**, a message from a blocked person **MUST** be returned to the blocker with `body`, `attachments`, and `reactions` withheld, and a `blocked_sender: true` flag set. The message's position in the channel and its id **MUST** be preserved. The client renders a collapsed placeholder with an explicit "show anyway" affordance.

6. In a **direct message channel** between the blocker and the blocked person, messages from the blocked person **MUST NOT** be returned to the blocker at all - not collapsed, not flagged, not present. The DM **MUST** disappear from the blocker's DM list while the block stands.

7. The blocked person **MUST** still be able to post. Their message is persisted normally and they see it in their own client. It is simply never delivered to the blocker: no row in the blocker's message list, no WebSocket frame, no notification, no push. The service **MUST NOT** return an error to the blocked sender.

8. Consequently, the service **MUST NOT** disclose the existence of a block to the blocked person through *any* channel: not a status code, not an error string, not a missing-read-receipt, not a delivery indicator. See §2 for why this is the single most important security property of this task.

9. Reactions and mentions authored by a blocked person **MUST** be suppressed for the blocker: their reaction is not counted in the folded reaction set the blocker receives, and their `@mention` of the blocker raises no notification.

10. Blocking **MUST NOT** remove either party from any channel, and **MUST NOT** be visible to other members of that channel. Channel membership and moderation are the administrator's business (TASK-CHAT-269); blocking is the individual's.

11. Unblocking **MUST** restore everything immediately and completely. Messages the blocked person sent during the block become visible in place, in their original position, on the blocker's next fetch. Nothing is lost and nothing is back-dated.

12. Blocks **MUST NOT** apply to the moderation queue. When an administrator reviews a report in TASK-CHAT-269, they see the reported content in full regardless of any block either party holds. An administrator who has blocked someone must still be able to adjudicate a report about them.

13. Blocks **MUST** be tenant-scoped under row-level security with both `USING` and `WITH CHECK`, like every other chat table.

14. Every block and unblock **MUST** emit exactly one audit row - `chat.subject_blocked` or `chat.subject_unblocked` - carrying the blocker as actor and the blocked subject id. The row **MUST NOT** be readable by the blocked person through any surface exposed to them.

15. The web client **MUST** offer Block / Unblock in the member list and in the profile popover, and **MUST** confirm before applying it. Every string **MUST** ship in `en` and `vi`.

## §2 - Why this design (rationale for humans)

**Why the blocked person is never told (§1 #7, #8).** This is the clause the task turns on, and it is counter-intuitive, so it is worth being blunt. The obvious design is to refuse the blocked person's message with a `403` - it is honest, it is simple, and it is dangerous. Telling a harasser "you have been blocked" is an escalation trigger: it converts a person who was being ignored into a person who knows they were rejected, and the documented pattern is that they escalate through another channel. Every mature messaging product - Signal, WhatsApp, iMessage - lets the blocked sender believe the message went out. So do we. The message is persisted, the sender sees it in their own client, and it simply never arrives. The blocker is protected, and the blocked person has nothing to react to.

**Why group-channel messages are collapsed rather than deleted (§1 #5).** Removing a blocked person's messages outright silently rewrites the channel's history for one participant: replies to a vanished message become nonsense, thread counts stop matching, and the blocker ends up more confused than protected. A collapsed placeholder preserves the shape of the conversation, tells the truth ("someone you blocked said something here"), and leaves the choice to reveal in the hands of the person who made the block. It is *their* block; they are allowed to un-hide their own view.

**Why DM messages are removed entirely rather than collapsed (§1 #6).** In a group channel, a collapsed row is context - it explains a gap in a conversation you are still part of. In a DM there is no conversation left to contextualise: a column of "blocked message" placeholders is not information, it is a drip-feed of the harassment you asked to stop. The whole DM leaves the list.

**Why enforcement is at four points and not one (§1 #4).** Because there are four ways content reaches a person, and filtering three of them is filtering none. It is entirely possible to hide the messages from the list and still push a phone notification carrying the blocked person's name and the first 80 characters of their message onto the blocker's lock screen. That is not a hypothetical - it is the default behaviour of the existing `notify.rs` fan-out, which selects channel members and does not know blocks exist. All four are named in §1 #4 and each has its own acceptance criterion, so a future contributor who adds a fifth fan-out path has a checklist telling them what they just broke.

**Why blocks do not apply to the moderation queue (§1 #12).** The likeliest reporter of a person is the same person who blocked them. If the block also hid the content from the moderation queue, the administrator - who may well be that same person in a small workspace - would be adjudicating a report they cannot read. The queue is a distinct surface with a distinct purpose and it renders raw.

**Why blocking does not remove anyone from a channel (§1 #10).** Because that is a moderation action with consequences for everyone else in the channel, and one member should not be able to trigger it unilaterally. Blocking changes what *you* see. Removal changes what *everyone* sees, and belongs to the administrator.

## §3 - API contract

### Migration

```sql
-- services/chat/migrations/0014_chat_blocks.sql
-- TASK-CHAT-268: one row per (blocker, blocked) pair. Directional: A blocking B says nothing about
-- whether B blocks A. Private: only the blocker can ever read their own rows.

CREATE TABLE IF NOT EXISTS chat_blocks (
    tenant_id           UUID NOT NULL,
    blocker_subject_id  UUID NOT NULL,
    blocked_subject_id  UUID NOT NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    PRIMARY KEY (blocker_subject_id, blocked_subject_id),
    CONSTRAINT chat_blocks_not_self CHECK (blocker_subject_id <> blocked_subject_id)
);

-- The hot path is "give me every subject THIS caller has blocked", run once per message-list,
-- realtime fan-out, and notification fan-out. The PK already serves it (blocker is the leading
-- column); this index serves the reverse question the notification fan-out asks: "of the members
-- about to be notified, which of them have blocked the sender?"
CREATE INDEX IF NOT EXISTS chat_blocks_blocked_idx
    ON chat_blocks (blocked_subject_id, blocker_subject_id);

ALTER TABLE chat_blocks ENABLE ROW LEVEL SECURITY;
ALTER TABLE chat_blocks FORCE  ROW LEVEL SECURITY;
CREATE POLICY chat_blocks_tenant_scoped ON chat_blocks
    FOR ALL
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    );

GRANT SELECT, INSERT, DELETE ON chat_blocks TO cyberos_app;
GRANT SELECT ON chat_blocks TO cyberos_ro;
```

### Endpoints

```
POST   /v1/chat/blocks               { "subject_id": "<uuid>" }   -> 204 (idempotent)
DELETE /v1/chat/blocks/:subject_id                                -> 204 (idempotent)
GET    /v1/chat/blocks                                            -> 200 [ { subject_id, created_at } ]

400  "cannot block yourself"
401  token invalid
```

Both mutations are idempotent by design: blocking someone twice is a no-op, and unblocking someone you never blocked is a no-op. Neither returns `404` or `409`, because a distinguishable response would let a caller enumerate their own block state through side effects - and, worse, invites a client to render an error for a state the user does not care about.

### The blocked-set, and where it is applied

```rust
// services/chat/src/blocks.rs

/// Every subject the caller has blocked. Read ONCE per request and threaded through the four
/// enforcement points (§1 #4). A HashSet, not a per-message query: the message list is the hot
/// path and N+1 here would be the most expensive thing in the service.
pub async fn blocked_by(
    tx: &mut sqlx::PgConnection,
    blocker: Uuid,
) -> Result<HashSet<Uuid>, (StatusCode, String)> {
    let rows: Vec<(Uuid,)> = sqlx::query_as(
        "SELECT blocked_subject_id FROM chat_blocks WHERE blocker_subject_id = $1")
        .bind(blocker).fetch_all(&mut *tx).await.map_err(db::internal)?;
    Ok(rows.into_iter().map(|(id,)| id).collect())
}

/// The reverse question, asked by the notification fan-out: of these recipients, which have
/// blocked this sender? Returns the recipients to SKIP.
pub async fn blockers_of(
    tx: &mut sqlx::PgConnection,
    sender: Uuid,
    candidates: &[Uuid],
) -> Result<HashSet<Uuid>, (StatusCode, String)> {
    let rows: Vec<(Uuid,)> = sqlx::query_as(
        "SELECT blocker_subject_id FROM chat_blocks
          WHERE blocked_subject_id = $1 AND blocker_subject_id = ANY($2)")
        .bind(sender).bind(candidates).fetch_all(&mut *tx).await.map_err(db::internal)?;
    Ok(rows.into_iter().map(|(id,)| id).collect())
}
```

### Enforcement point 1 - the message list

```rust
// services/chat/src/messages.rs :: list

let blocked = blocks::blocked_by(&mut tx, caller).await?;
let is_dm = channel.kind == "direct";

let messages: Vec<Message> = rows
    .into_iter()
    // §1 #6 - in a DM, a blocked sender's messages are not returned at all.
    .filter(|m| !(is_dm && blocked.contains(&m.sender_subject_id)))
    .map(|mut m| {
        // §1 #5 - in a group channel, the row survives; its content does not.
        if blocked.contains(&m.sender_subject_id) {
            m.body = String::new();
            m.attachments = vec![];
            m.reactions = vec![];
            m.blocked_sender = true;
        }
        // §1 #9 - a blocked person's reaction is not counted for the blocker.
        m.reactions.retain(|r| !r.reactor_ids.iter().any(|id| blocked.contains(id)));
        m
    })
    .collect();
```

### Enforcement point 2 - the realtime fan-out

```rust
// services/chat/src/realtime.rs
// The per-channel broadcast is one-to-many, so the block check CANNOT live at the send site: it
// has to live at each subscriber. Each socket carries its own blocked-set, refreshed on block /
// unblock (the mutation publishes an invalidation on the subscriber's own control channel).

if sock.blocked.contains(&frame.sender_subject_id) {
    if sock.channel_kind == ChannelKind::Direct { continue; }        // §1 #6 - drop entirely
    frame = frame.redacted();                                        // §1 #5 - collapse
}
```

### Enforcement point 3 - the notification and push fan-out

```rust
// services/chat/src/notify.rs :: fan_out
// This is the point that was silently broken before this task: the fan-out selects channel members
// and pushes to their devices. It did not know blocks existed, so a blocked person's message would
// still have arrived on the blocker's lock screen carrying their name and the first line of text.

let mut recipients = channel_members_except(&mut tx, channel_id, sender).await?;
let blockers = blocks::blockers_of(&mut tx, sender, &recipients).await?;   // §1 #4, #9
recipients.retain(|r| !blockers.contains(r));
```

### Enforcement point 4 - the DM list

```rust
// services/chat/src/channels.rs :: list
// §1 #6 - a DM with a blocked person leaves the blocker's list entirely while the block stands.

let blocked = blocks::blocked_by(&mut tx, caller).await?;
dms.retain(|dm| !blocked.contains(&dm.partner_subject_id));
```

### Wire shape

```rust
#[derive(Serialize)]
pub struct Message {
    // ... existing fields ...
    /// True when the caller has blocked this message's sender AND this is a group channel.
    /// The client renders a collapsed row with a "show anyway" affordance. Never true in a DM,
    /// because a blocked sender's DM messages are not returned at all.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub blocked_sender: bool,
}
```

## §4 - Acceptance criteria

1. **Block is directional and private** - after A blocks B, `GET /v1/chat/blocks` as B returns an empty list, and nothing B can fetch differs from before the block.
2. **Self-block is refused** - blocking your own subject id returns `400` and writes no row.
3. **Both mutations are idempotent** - blocking twice returns `204` twice and leaves one row; unblocking someone never blocked returns `204` and changes nothing.
4. **Group-channel messages are collapsed, not removed** - B's message in a shared channel is returned to A with `blocked_sender: true`, an empty `body`, no attachments, no reactions, and its original id and position.
5. **DM messages are removed entirely** - B's messages in the A-B DM are absent from A's message list: not collapsed, not flagged, not present.
6. **The DM leaves A's list** - the A-B DM does not appear in A's `GET /v1/chat/channels` while the block stands.
7. **The blocked sender is never told** - B's `POST` of a message to the DM returns `201` exactly as before; B's own message list still contains it; no response field, status code, or timing differs from the unblocked case.
8. **The realtime socket drops it** - with A's socket open on the shared channel, a message posted by B produces a redacted frame for A in a group channel and **no frame at all** in a DM.
9. **No notification, no push** - a message from B, including one that `@mentions` A, produces zero notification rows and zero push sends targeting A's devices.
10. **Reactions from B are not counted** - B reacting to a message A can see leaves A's folded reaction count unchanged.
11. **Unblock restores everything in place** - after `DELETE`, A's next fetch returns B's messages from *during* the block with full bodies, in their original positions, and the DM reappears in A's list.
12. **The moderation queue is unaffected** - an administrator who has blocked B still sees B's reported content in full in the TASK-CHAT-269 queue.
13. **Neither party leaves the channel** - after A blocks B, both remain members and every other member's view is byte-identical to before.
14. **Cross-tenant isolation holds** - a block row in tenant A is invisible to any query run with tenant B's GUC.
15. **Exactly one audit row per mutation** - `chat.subject_blocked` / `chat.subject_unblocked`, actor = blocker; the idempotent no-op emits nothing.
16. **Both locales render** - every block/unblock string resolves in `en` and `vi`.

## §5 - Verification

```rust
// services/chat/tests/blocks.rs

#[tokio::test]
async fn group_message_is_collapsed_dm_message_is_gone() {          // AC 4, 5, 6
    let app = harness().await;
    let ch = app.channel("general", &[ALICE, BOB]).await;
    let dm = app.dm(ALICE, BOB).await;
    let gm = app.post_as(BOB, ch, "group text").await;
    let dmm = app.post_as(BOB, dm, "dm text").await;

    app.block_as(ALICE, BOB).await;

    let group = app.list_as(ALICE, ch).await;
    let m = group.iter().find(|m| m.id == gm).expect("row must survive");
    assert!(m.blocked_sender);
    assert_eq!(m.body, "");
    assert!(m.attachments.is_empty());

    let direct = app.list_as(ALICE, dm).await;
    assert!(direct.iter().all(|m| m.id != dmm), "DM message must be absent, not collapsed");

    let channels = app.channels_as(ALICE).await;
    assert!(!channels.iter().any(|c| c.id == dm), "the DM must leave the list");
}

#[tokio::test]
async fn the_blocked_sender_observes_nothing() {                    // AC 7 - the security property
    let app = harness().await;
    let dm = app.dm(ALICE, BOB).await;

    let before = app.post_as_full(BOB, dm, "one").await;
    app.block_as(ALICE, BOB).await;
    let after = app.post_as_full(BOB, dm, "two").await;

    assert_eq!(before.status(), after.status());                    // both 201
    assert_eq!(before.json_shape(), after.json_shape());            // same fields, same types
    // B still sees both of their own messages.
    let bobs_view = app.list_as(BOB, dm).await;
    assert_eq!(bobs_view.len(), 2);
    // And nothing anywhere in B's world names the block.
    assert!(!app.everything_visible_to(BOB).await.contains("block"));
}

#[tokio::test]
async fn realtime_drops_in_dm_and_redacts_in_group() {              // AC 8
    let app = harness().await;
    let ch = app.channel("general", &[ALICE, BOB]).await;
    let dm = app.dm(ALICE, BOB).await;
    app.block_as(ALICE, BOB).await;

    let mut sock = app.socket_as(ALICE).await;
    app.post_as(BOB, ch, "group").await;
    let frame = sock.next_frame().await.expect("a redacted frame arrives");
    assert!(frame.blocked_sender && frame.body.is_empty());

    app.post_as(BOB, dm, "dm").await;
    assert!(sock.next_frame_timeout(Duration::from_millis(500)).await.is_none(),
            "a DM from a blocked sender must produce NO frame at all");
}

#[tokio::test]
async fn no_notification_and_no_push_not_even_for_a_mention() {     // AC 9 - the point that was broken
    let app = harness().await;
    let ch = app.channel("general", &[ALICE, BOB]).await;
    app.register_device(ALICE, "android", "tok-a").await;
    app.block_as(ALICE, BOB).await;

    app.post_as(BOB, ch, "hey @alice look at this").await;

    assert_eq!(app.notifications_for(ALICE).await.len(), 0);
    assert_eq!(app.pushes_to("tok-a").await.len(), 0);
}

#[tokio::test]
async fn blocked_reactions_are_not_counted() {                      // AC 10
    let app = harness().await;
    let ch  = app.channel("general", &[ALICE, BOB, CAROL]).await;
    let msg = app.post_as(CAROL, ch, "hello").await;
    app.react_as(BOB, msg, "+1").await;
    app.block_as(ALICE, BOB).await;

    let seen = app.list_as(ALICE, ch).await;
    let m = seen.iter().find(|m| m.id == msg).unwrap();
    assert!(m.reactions.is_empty(), "a blocked person's reaction must not be counted");
}

#[tokio::test]
async fn unblock_restores_in_place() {                              // AC 11
    let app = harness().await;
    let ch = app.channel("general", &[ALICE, BOB]).await;
    let a1 = app.post_as(ALICE, ch, "first").await;
    app.block_as(ALICE, BOB).await;
    let b1 = app.post_as(BOB, ch, "during the block").await;
    let a2 = app.post_as(ALICE, ch, "third").await;
    app.unblock_as(ALICE, BOB).await;

    let seen = app.list_as(ALICE, ch).await;
    assert_eq!(seen.iter().map(|m| m.id).collect::<Vec<_>>(), vec![a1, b1, a2]);   // in place
    let m = seen.iter().find(|m| m.id == b1).unwrap();
    assert_eq!(m.body, "during the block");                                        // in full
    assert!(!m.blocked_sender);
}

#[tokio::test]
async fn moderation_queue_ignores_blocks() {                        // AC 12
    let app = harness().await;
    let ch  = app.channel("general", &[ADMIN, BOB]).await;
    let msg = app.post_as(BOB, ch, "reported text").await;
    app.report_as(ADMIN, json!({"target_kind":"message","target_message_id":msg,"reason":"hate"})).await;
    app.block_as(ADMIN, BOB).await;

    let queue = app.moderation_queue_as(ADMIN).await;               // TASK-CHAT-269
    assert_eq!(queue[0].snapshot_body, "reported text");            // full content, block or not
}

#[tokio::test]
async fn idempotent_mutations_and_self_block_guard() {              // AC 2, 3, 15
    let app = harness().await;
    assert_eq!(app.block_as_raw(ALICE, ALICE).await.status(), StatusCode::BAD_REQUEST);

    assert_eq!(app.block_as_raw(ALICE, BOB).await.status(), StatusCode::NO_CONTENT);
    assert_eq!(app.block_as_raw(ALICE, BOB).await.status(), StatusCode::NO_CONTENT);
    assert_eq!(app.block_rows(ALICE).await.len(), 1);
    assert_eq!(app.audit_rows("chat.subject_blocked").await.len(), 1);   // no-op emits nothing

    assert_eq!(app.unblock_as_raw(ALICE, CAROL).await.status(), StatusCode::NO_CONTENT);
    assert_eq!(app.audit_rows("chat.subject_unblocked").await.len(), 0);
}
```

## §6 - Implementation skeleton

(The four enforcement points in §3 are the skeleton. What remains is socket-state invalidation, which is the only genuinely stateful part and the easiest thing to get wrong - see §10 row 5.)

```rust
// services/chat/src/blocks.rs :: block
// After committing the block, publish an invalidation to the BLOCKER's own control topic so every
// socket they hold refreshes its blocked-set. Without this, an open tab keeps receiving the blocked
// person's frames until it reconnects - which is a live, observable failure of §1 #4.
state.realtime.invalidate_blocks(blocker).await;
```

## §7 - Dependencies

- **Upstream:** none. `chat_channels.kind`, `chat_channel_members`, `chat_devices`, the realtime broadcast, and `notify.rs` all exist.
- **Downstream:** TASK-CHAT-269 must explicitly *not* apply `blocked_by` when rendering the moderation queue (§1 #12, AC #12).
- **Sibling:** TASK-CHAT-267 (reporting) is independent. They share only the member-list entry point.
- **Cross-module:** two new audit `event_type` values (`chat.subject_blocked`, `chat.subject_unblocked`). No schema change in the memory module.

## §8 - Example payloads

```json
POST /v1/chat/blocks
{ "subject_id": "b7c8d9e0-1f2a-4b3c-8d4e-5f60718293a4" }
204 No Content
```

A group-channel message from a blocked sender, as returned to the blocker:

```json
{
  "id": "6b1f4e0a-7c2d-4d19-9b0e-1f2a3c4d5e6f",
  "channel_id": "3f5a1b2c-9d8e-4c7b-a6f5-e4d3c2b1a098",
  "sender_subject_id": "b7c8d9e0-1f2a-4b3c-8d4e-5f60718293a4",
  "body": "",
  "attachments": [],
  "reactions": [],
  "blocked_sender": true,
  "created_at": "2026-07-11T05:22:19Z"
}
```

The same message, as returned to *everyone else* in the channel - unchanged:

```json
{
  "id": "6b1f4e0a-7c2d-4d19-9b0e-1f2a3c4d5e6f",
  "body": "the actual text",
  "attachments": [ { "id": "…", "filename": "shot.png" } ],
  "reactions": [ { "emoji": "+1", "count": 2 } ],
  "created_at": "2026-07-11T05:22:19Z"
}
```

The audit row:

```json
{
  "event_type": "chat.subject_blocked",
  "payload": { "blocked_subject_id": "b7c8d9e0-1f2a-4b3c-8d4e-5f60718293a4" }
}
```

## §9 - Open questions

**Deferred:**

- *Workspace-wide block ("mute across every surface")* - once CyberOS grows modules beyond chat, a block should plausibly hide the person from mentions in PROJ, from the memory search index, and so on. That is a cross-module contract and belongs in its own task. Chat-scoped is what the store policy requires and what ships here.
- *Auto-expiring blocks* - "block for 7 days" is a real de-escalation tool. Deferred: `created_at` is present, an `expires_at` is additive.
- *Administrator visibility of blocks* - an admin arguably wants to know that six people have blocked the same person, because that is the strongest possible signal in a small workspace. Deliberately deferred, and it needs a decision, not a design: it partially breaks §1 #2 (privacy of the block), and the trade is real. Raised as a decision for TASK-CHAT-269's follow-up.

## §10 - Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Block implemented only in the client | AC 4, 5, 8, 9 all assert on the *server* response | Content would still cross the wire and be visible in the network tab | Every enforcement point is server-side; the client never filters |
| Notification fan-out ignores blocks | AC 9 asserts zero pushes, including for an `@mention` | Blocked person's name and text on the blocker's lock screen | `blockers_of` is applied in `notify.rs` before recipient retention |
| Blocked sender receives a 403 and learns they are blocked | AC 7 asserts identical status and body shape before and after | Escalation to another channel; the exact harm the block exists to prevent | Silent-drop delivery model (§1 #7); no error path exists |
| Timing side-channel discloses the block | Delivery path length is the same either way (the message is written normally) | Blocked person cannot infer the block from latency | The block is applied at *read* fan-out, not at write |
| Open WebSocket keeps delivering after the block lands | AC 8 with a socket opened *before* the block | Blocker keeps seeing frames until they reconnect | `invalidate_blocks` publishes to the blocker's control topic on mutation (§6) |
| Blocked-set fetched per message (N+1) | Message list is the hot path; one `HashSet` per request | Latency regression on every channel open | `blocked_by` is read once per request and threaded through |
| Reaction from a blocked person leaks their existence via a count | AC 10 | Blocker sees "2 reactions" where they should see 1 | Reactions are filtered by reactor id, not just message sender |
| Blocker blocks themselves | `chat_blocks_not_self` CHECK | `400` | Constraint is in the DB, not only the handler |
| Unblock loses messages sent during the block | AC 11 asserts exact ordering and full bodies | Silent data loss from the blocker's view | Nothing is deleted; the block filters reads only |
| Moderation queue applies the block and hides the evidence | AC 12 | Admin cannot adjudicate a report about someone they blocked | TASK-CHAT-269 does not call `blocked_by` |
| Block leaks to other channel members | AC 13 asserts byte-identical views for third parties | Social fallout inside a small workspace | The block is applied per-caller, never at write |
| Cross-tenant block row read | RLS `USING` + `WITH CHECK`, `FORCE`d; AC 14 | Zero rows | Policy mirrors every other chat table |
| A fifth fan-out path is added later and forgets blocks | §1 #4 enumerates the four; each has an AC | A new leak | The enumeration is the checklist; a new path needs a new AC |
| DM channel accumulates undelivered messages during a long block | Storage only; the blocker never reads them | Unbounded-ish growth in a pathological case | Bounded by the existing per-message size cap and by retention (TASK-CHAT-240) |

## §11 - Implementation notes

- **Why `blocked_sender` is `skip_serializing_if` false rather than always present.** The flag is absent from every message a normal reader sees, so the wire shape for the 99.99% case is unchanged and no existing client parser has to learn a new field. A client that does not know about blocking sees exactly what it saw before - an empty body - and degrades to rendering an empty row rather than crashing.

- **Why the block set is threaded rather than queried at each enforcement point.** Three of the four points (list, realtime, notify) sit on the hot path. Reading the set once per request and passing a `HashSet<Uuid>` costs one index scan and a few hundred bytes. Querying per message would be an N+1 on the single most-called endpoint in the service.

- **The realtime invalidation is the one piece of genuinely mutable state.** Each socket caches its owner's blocked-set at connect. A block placed in tab A must invalidate tab B's socket, or tab B keeps delivering. The mutation publishes to the *blocker's* control topic - not the channel's - because a block is nobody else's business.

- **Why `blockers_of` takes a candidate list rather than scanning.** In a large channel the fan-out already knows exactly who it is about to notify. Asking "of these 200 people, which have blocked this sender" is one indexed query against `chat_blocks_blocked_idx`; asking "who has blocked this sender" globally would return a set we would then have to intersect anyway.

- **No `blocked_at` on the message.** It is tempting to record when a message was hidden from whom, and it would be a small privacy disaster: it is a per-message log of one person's block state, sitting in a table the administrator can read.

*End of TASK-CHAT-268.*
