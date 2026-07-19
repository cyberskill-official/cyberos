---
id: TASK-EMAIL-003
title: "EMAIL Missive-style team UX — shared inbox, thread assignment, internal comments, Genie actions panel, keyboard shortcuts"
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
eu_ai_act_risk_class: not_ai
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-17T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: EMAIL
priority: p0
status: draft
verify: T
phase: P2
milestone: P2 · slice 2
slice: 2
owner: Stephen Cheng (CDO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-EMAIL-001, TASK-EMAIL-008, TASK-EMAIL-009, TASK-EMAIL-007, TASK-EMAIL-006, TASK-CHAT-005, TASK-CUO-101, TASK-MEMORY-111]
depends_on: [TASK-EMAIL-001, TASK-EMAIL-009]
blocks: []

source_pages:
  - website/docs/modules/email.html#missive-ux
  # Missive — collaborative email reference
  - https://missiveapp.com/

source_decisions:
  - DEC-1610 2026-05-17 — Shared inbox model: tenant has one or more channels (e.g. support@, sales@); threads visible to all members of channel
  - DEC-1611 2026-05-17 — Thread assignment: one assignee at a time, internal-note-only handoff (no email-to-customer for assignment changes)
  - DEC-1612 2026-05-17 — Internal comments NEVER sent to customer; rendered inline in thread view but excluded from reply Quote/Forward
  - DEC-1613 2026-05-17 — Closed enum `thread_state` = {open, assigned, snoozed, closed, archived}; cardinality 5
  - DEC-1614 2026-05-17 — Genie actions panel (right pane): live action proposals from TASK-EMAIL-008 + manual quick-actions
  - DEC-1615 2026-05-17 — Keyboard shortcuts: navigation (j/k), reply (r), forward (f), assign (a), snooze (z), close (e), Genie (g)
  - DEC-1616 2026-05-17 — memory audit kinds: email.thread_assigned, email.thread_state_changed, email.internal_comment_added, email.thread_snoozed, email.thread_closed

language: typescript / react
service: cyberos/services/portal-web/
new_files:
  - services/email/migrations/0012_thread_state.sql
  - services/email/src/threads/state.rs
  - services/email/src/threads/internal_comments.rs
  - services/email/src/handlers/thread_state_routes.rs
  - services/email/src/handlers/comment_routes.rs
  - services/email/src/audit/thread_events.rs
  - services/portal-web/src/email/InboxView.tsx
  - services/portal-web/src/email/ThreadView.tsx
  - services/portal-web/src/email/AssignmentPicker.tsx
  - services/portal-web/src/email/SnoozePicker.tsx
  - services/portal-web/src/email/InternalCommentEditor.tsx
  - services/portal-web/src/email/GenieActionsPanel.tsx
  - services/portal-web/src/email/keyboard_shortcuts.ts
  - services/email/tests/thread_assign_test.rs
  - services/email/tests/thread_state_enum_cardinality_test.rs
  - services/email/tests/internal_comment_not_in_reply_test.rs
  - services/email/tests/thread_snooze_wake_test.rs
  - services/email/tests/thread_audit_emission_test.rs
  - services/portal-web/tests/inbox-keyboard.spec.ts
  - services/portal-web/tests/thread-genie-panel.spec.ts

modified_files:
  - services/portal-web/src/app/email/page.tsx

allowed_tools:
  - file_read: services/{email,portal-web}/**
  - file_write: services/{email,portal-web}/{src,tests,migrations}/**
  - bash: cd services/email && cargo test thread; cd services/portal-web && pnpm test

disallowed_tools:
  - include internal comments in email reply quote (per DEC-1612)
  - email notification on assignment (per DEC-1611 — internal only)

effort_hours: 16
subtasks:
  - "0.4h: 0012_thread_state.sql"
  - "0.6h: threads/state.rs (assign + state machine)"
  - "0.5h: threads/internal_comments.rs"
  - "0.5h: handlers/thread_state_routes.rs"
  - "0.4h: handlers/comment_routes.rs"
  - "0.3h: audit/thread_events.rs"
  - "2.5h: InboxView.tsx (channel selector + thread list)"
  - "3.0h: ThreadView.tsx (rendering + reply composer)"
  - "1.0h: AssignmentPicker.tsx"
  - "0.8h: SnoozePicker.tsx (time chip selector)"
  - "1.5h: InternalCommentEditor.tsx + mention support"
  - "1.8h: GenieActionsPanel.tsx (live TASK-EMAIL-008 stream)"
  - "0.8h: keyboard_shortcuts.ts (focus mgmt + global handlers)"
  - "1.5h: Rust tests — 5 files"
  - "0.8h: TS Playwright tests — 2 files"
  - "0.6h: docs"

risk_if_skipped: "Without Missive-style UX, CDO/AM use Apple Mail / Gmail — no shared inbox, no assignment, no Genie integration — productivity loss. Without DEC-1612 internal-comment isolation, customer accidentally sees private notes (disaster). Without DEC-1615 keyboard shortcuts, power users disengage."
---

## §1 — Description (BCP-14 normative)

The EMAIL service + portal-web frontend **MUST** ship Missive-style UX — shared inbox, thread state + assignment, internal comments, Genie panel, keyboard shortcuts, 5 memory audit kinds.

1. **MUST** define `thread_state` per DEC-1610 + DEC-1613 — `open | assigned | snoozed | closed | archived`. Validated against closed enum (cardinality 5).

2. **MUST** expose thread-state APIs:
   ```text
   POST   /v1/email/threads/{id}/assign        (body: {user_id})
   POST   /v1/email/threads/{id}/snooze        (body: {wake_at})
   POST   /v1/email/threads/{id}/close
   POST   /v1/email/threads/{id}/reopen
   ```

3. **MUST** support internal comments at `threads/internal_comments.rs`:
   - `POST /v1/email/threads/{id}/comments` body `{ body, mentions[] }`
   - Renders in thread view inline (visual distinction)
   - NEVER included in email reply quote per DEC-1612

4. **MUST** define table extension at migration `0012`:
   ```sql
   ALTER TABLE threads ADD COLUMN state TEXT NOT NULL DEFAULT 'open'
     CHECK (state IN ('open','assigned','snoozed','closed','archived'));
   ALTER TABLE threads ADD COLUMN assigned_to UUID;
   ALTER TABLE threads ADD COLUMN snoozed_until TIMESTAMPTZ;
   ALTER TABLE threads ADD COLUMN closed_at TIMESTAMPTZ;
   ALTER TABLE threads ADD COLUMN closed_by UUID;
   CREATE INDEX threads_state_assigned_idx ON threads(tenant_id, state, assigned_to)
     WHERE state IN ('open','assigned');
   GRANT UPDATE (state, assigned_to, snoozed_until, closed_at, closed_by, updated_at) ON threads TO cyberos_app;

   CREATE TABLE thread_comments (
     comment_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     thread_id UUID NOT NULL,
     author_id UUID NOT NULL,
     body TEXT NOT NULL,
     mentions UUID[] NOT NULL DEFAULT '{}',
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   ALTER TABLE thread_comments ENABLE ROW LEVEL SECURITY;
   CREATE POLICY thread_comments_rls ON thread_comments
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON thread_comments FROM cyberos_app;
   ```

5. **MUST** wake snoozed threads at `snoozed_until` via TASK-MCP-007 cron — flip `state` to `open`, notify assignee via TASK-CHAT-005.

6. **MUST** render frontend at `services/portal-web/src/email/`:
   - `InboxView.tsx`: channel selector (left), thread list (middle)
   - `ThreadView.tsx`: thread rendering + reply composer (right or full)
   - `AssignmentPicker.tsx`: avatar grid + search
   - `SnoozePicker.tsx`: chips (1h, 4h, tomorrow, next week, custom)
   - `InternalCommentEditor.tsx`: separate composer, visual distinction
   - `GenieActionsPanel.tsx`: streams from TASK-EMAIL-008 + quick-actions

7. **MUST** wire keyboard shortcuts per DEC-1615 at `keyboard_shortcuts.ts`:
   - `j/k` navigate
   - `r` reply, `R` reply-all
   - `f` forward
   - `a` assign focus
   - `z` snooze focus
   - `e` archive/close
   - `g` Genie panel focus
   - `Escape` clears focus
   - All disabled when text input focused

8. **MUST** emit 5 memory audit kinds per DEC-1616. PII per TASK-MEMORY-111: comment body SHA-256 hashed; mentions (uuids) ok.

9. **MUST** thread trace_id from UI action → backend mutation → audit.

10. **MUST NOT** include internal comments in email reply quote per DEC-1612 — `Reply` composer pulls thread.messages only, not thread.comments.

11. **MUST NOT** send email notification on assignment per DEC-1611 — in-app + TASK-CHAT-005 mention only.

12. **MUST NOT** show closed/archived threads in default inbox view — separate filter chip.

---

## §2 — Why this design

**Why shared inbox (DEC-1610)?** Single-user email apps (Apple Mail) can't support team handoff; Missive's channel model is industry-validated.

**Why one assignee (DEC-1611)?** Multiple assignees → diffusion of responsibility; Missive's single-assignee model proves better SLA.

**Why no email notif on assignment (DEC-1611)?** Customer doesn't need to see "Stephen reassigned to Lisa" emails; internal-only via TASK-CHAT-005.

**Why never include internal comments in reply (DEC-1612)?** Single most catastrophic bug class in collab email tools; hard contract.

**Why keyboard shortcuts (DEC-1615)?** Power users 3-5x faster than mouse; Missive's shortcut grammar is well-known.

**Why Genie panel right-side (DEC-1614)?** Active context without disrupting reading flow; consistent with TASK-PORTAL-005 chat layout.

---

## §3 — API contract (see §1.2 + §1.3)

Sample thread state response:
```json
{
  "thread_id": "uuid",
  "state": "assigned",
  "assigned_to": "uuid",
  "assigned_to_name": "Lisa Nguyen",
  "snoozed_until": null,
  "message_count": 5,
  "internal_comment_count": 2,
  "last_message_at": "2026-05-17T10:00:00Z"
}
```

Sample comment add:
```json
POST /v1/email/threads/{id}/comments
{
  "body": "@Lisa can you handle this? Customer is asking about pricing",
  "mentions": ["uuid-lisa"]
}
```

---

## §4 — Acceptance criteria
1. **5 thread states + cardinality test**. 2. **One assignee at a time**. 3. **Assignment no customer-facing email**. 4. **Internal comments visible in thread view**. 5. **Internal comments NEVER in Reply quote**. 6. **Snooze wakes at `wake_at` via cron**. 7. **Snoozed thread invisible in default inbox**. 8. **5 memory audit kinds emitted**. 9. **PII scrubbed (comment body SHA256)**. 10. **RLS denies cross-tenant**. 11. **Mentions trigger TASK-CHAT-005 notification**. 12. **Trace_id preserved**. 13. **Keyboard shortcuts work (j/k/r/f/a/z/e/g)**. 14. **Shortcuts disabled in text inputs**. 15. **Genie panel streams TASK-EMAIL-008 proposals**. 16. **Channel selector lists tenant inboxes**. 17. **Reply composer pulls thread.messages only (no comments)**. 18. **Append-only thread_comments table**. 19. **Closed/archived hidden by default**. 20. **Wake from snooze sends TASK-CHAT-005 ping to assignee**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn assign_no_customer_email() {
    let ctx = TestContext::with_thread().await;
    ctx.assign_thread(ctx.thread_id, ctx.user_b).await;
    let sent_emails = ctx.outbound_send_count().await;
    assert_eq!(sent_emails, 0);
}

#[tokio::test]
async fn internal_comment_not_in_reply() {
    let ctx = TestContext::with_thread_and_comments().await;
    let reply_quote = ctx.compose_reply_quote(ctx.thread_id).await;
    assert!(!reply_quote.contains("INTERNAL_FLAG_XYZ"));
}

#[tokio::test]
async fn snooze_wakes_at_target() {
    let ctx = TestContext::with_thread().await;
    let wake = Utc::now() + Duration::seconds(2);
    ctx.snooze_thread(ctx.thread_id, wake).await;
    tokio::time::sleep(Duration::from_secs(3)).await;
    ctx.run_snooze_cron().await;
    let t: Thread = ctx.fetch_thread(ctx.thread_id).await;
    assert_eq!(t.state, "open");
}

// 5.4..5.10
```

```ts
test('keyboard shortcut r opens reply composer', async ({page}) => {
  await page.goto('/email/inbox');
  await page.keyboard.press('j');  // focus first thread
  await page.keyboard.press('r');
  await expect(page.locator('[data-testid=reply-composer]')).toBeVisible();
});
```

---

## §7 — Dependencies
**Upstream:** TASK-EMAIL-001, TASK-EMAIL-009.
**Cross-module:** TASK-EMAIL-008 (Genie panel), TASK-EMAIL-006 (CRM contact display), TASK-EMAIL-007 (convert button), TASK-CHAT-005 (mention notif), TASK-CUO-101 (panel embedding), TASK-MCP-007 (snooze cron), TASK-MEMORY-111 (PII).

## §8 — Sample payloads (see §3)

## §9 — Open questions
None blocking — Missive is the gold-standard reference.

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Concurrent assignment | optimistic lock on version | second 409 | UI refresh + retry |
| Snooze cron missed run | last_run check | wake on next boot | inherent |
| Mention user doesn't exist | validate | filter + sev-3 audit | data fix |
| Internal comment >10k chars | validate | 400 | use TASK-DOC-001 for long |
| Reply quote source includes comment (bug) | test guard | hard CI block | tests catch |
| TASK-CHAT-005 unreachable | mention notif retry | inherent | inherent |
| Genie panel stream disconnects | reconnect | retry | inherent |
| Snoozed past 1 year | warn at create | UI alert | manual confirm |
| Channel ACL mismatch | RLS | 403 | request access |
| Cross-tenant URL guess | RLS | 404 | inherent |

## §11 — Implementation notes
- §11.1 Thread state machine: open ↔ assigned, any → snoozed → wake → open/assigned, open/assigned → closed → reopen ↔ archived.
- §11.2 Internal comment markdown rendering via `marked` + DOMPurify sanitize.
- §11.3 Keyboard shortcuts: use `mousetrap` or custom dispatcher; respect `<input>`/`<textarea>` focus.
- §11.4 Genie panel uses SSE/WebSocket for live TASK-EMAIL-008 stream.
- §11.5 memory audit body: thread_id, state, assignee uuid; comment body SHA256.
- §11.6 Snooze cron via TASK-MCP-007 `kind: 'email.snooze_wake'`, runs every 5min.

---

*End of TASK-EMAIL-003 spec.*
