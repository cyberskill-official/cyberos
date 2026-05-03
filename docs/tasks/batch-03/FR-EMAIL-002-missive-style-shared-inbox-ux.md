---
title: "EMAIL — Missive-style shared inbox UX (queues, internal comments, assignment, statuses, snooze)"
author: "@stephen-cheng"
department: product
status: ready_for_review
priority: p1
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: user_facing
eu_ai_act_risk_class: not_ai
target_release: "P1 / 2026-Q4"
client_visible: false
template: feature_request@1
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Ship the Missive-style shared inbox UX layered on top of the Stalwart core (FR-EMAIL-001). Shared inboxes (`info@cyberskill.world`, `sales@cyberskill.world`, `support@cyberskill.world`) appear as collaborative queues where multiple Members triage threads with **assignment**, **statuses** (open / waiting-on-customer / waiting-on-internal / resolved / spam), **internal comments** that are visible to teammates but never sent to the customer, **read-state synchronisation** across the team (a thread marked read by Account Manager A is visible-as-read to Account Manager B), and **snooze-until** with on-time CUO reminders. The personal inboxes use the same primitives minus the team-collaboration surface. The whole UX renders as a Module-Federation remote at `/email` consuming the GraphQL subgraph from FR-EMAIL-001.

## Problem

A standard mail client (Apple Mail, Outlook, Gmail) handles personal inboxes well but degrades at shared inbox triage in three ways the team hits daily:

- **Two Members reply to the same customer simultaneously.** Without assignment + read-sync, both Account Managers reach into `sales@` and write conflicting replies.
- **Discussion about a customer email leaks back to the customer.** Replying to an external email with internal context appended is the most common email blooper in a small team. Internal comments solve this by being structurally distinct from replies.
- **Status disappears.** A thread that's "waiting on customer for a quote response" is forgotten until the customer chases. Statuses + snooze surface the right state at the right time.

The PRD §9.4.1 names Missive as the UX inspiration; the strategy is "Stalwart core + Missive-style UX". This FR ships the UX.

## Proposed Solution

The shape of the answer is a Module-Federation remote `cyberos-email-ux` mounted at `/email` plus GraphQL extensions on the `email` subgraph plus three primitives in the Postgres mirror.

**Three-pane layout.**

```
┌──────────────┬──────────────────────┬──────────────────────────────────┐
│ Mailboxes    │ Threads              │ Thread detail                    │
│              │                      │                                  │
│ ▾ Personal   │ [Thread row 1]       │ [Customer message]               │
│   Inbox  (3) │ [Thread row 2]       │ [Customer message reply]         │
│   Sent       │ [Thread row 3]       │ ─── INTERNAL COMMENTS ────       │
│   Drafts     │                      │ [Internal comment from team]     │
│              │                      │                                  │
│ ▾ Shared     │                      │ [Reply composer]                 │
│   sales@ (8) │                      │ [Internal comment composer]      │
│   support@   │                      │ ── Assignment / status / snooze ─│
│   info@      │                      │                                  │
└──────────────┴──────────────────────┴──────────────────────────────────┘
```

The thread row shows: from + subject + preview + assignee avatar + status chip + flags + snooze indicator + last-activity-at. Sorting modes: chronological, status-priority, assignee-mine-first, unread-first.

**Shared inbox primitives.**

- **Assignment.** Each thread has an assignee (a Member or null). Assignment is mutually exclusive with the "claim" pattern; assigning yourself is a one-click action; reassignment requires the new assignee to accept (panel notification) unless the assigner is the Founder or HR/Ops Lead.
- **Statuses.** Per-tenant configurable status enum. Default: `open` | `waiting-customer` | `waiting-internal` | `resolved` | `spam`. The status is a property of the thread within the shared inbox; a thread's status in `sales@` is independent of its status in `support@` if it surfaces in both (rare).
- **Internal comments.** Inline in the thread detail, visually distinct (yellow background, "internal" badge, mention syntax `@member`). Stored in `email.internal_comment` separately from messages so accidental include-in-reply is structurally impossible.
- **Read-state sync.** Per Member per thread per shared-inbox; a Member opens the thread, the read-state is synced via WebSocket subscription to all teammates in the same shared inbox. A small avatar stack on the thread row shows "currently viewing" indicators.
- **Concurrent-edit awareness.** When two Members open the reply composer for the same thread, the second sees a banner "Stephen is also drafting a reply"; both can save drafts to their own personal drafts but only one can attach a draft to the thread as the canonical pending reply.

**Snooze-until.** A thread can be snoozed until: a date+time the user picks, "until customer replies", "until tomorrow morning at 09:00 ICT", "until next Monday 09:00 ICT". Snoozed threads are hidden from the default queue view and resurface at the snooze-until time as a Notify-mode card from CUO/COO ("Snoozed thread is back: re: Acme proposal"). The reminder lands in the Genie panel and a small unread-counter appears on the snooze-button in the thread row.

**Schema extensions.**

```sql
CREATE TABLE email.thread_state (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  thread_id TEXT NOT NULL,
  mailbox_id UUID NOT NULL,                  -- which inbox this state belongs to
  assignee_member_id UUID,
  status TEXT NOT NULL DEFAULT 'open',
  snooze_until TIMESTAMPTZ,
  flags TEXT[],
  last_activity_at TIMESTAMPTZ NOT NULL,
  UNIQUE (tenant_id, thread_id, mailbox_id)
);

CREATE TABLE email.thread_read_state (
  thread_id TEXT NOT NULL,
  mailbox_id UUID NOT NULL,
  member_id UUID NOT NULL,
  read_through_message_id TEXT,
  last_seen_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (thread_id, mailbox_id, member_id)
);

CREATE TABLE email.internal_comment (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  thread_id TEXT NOT NULL,
  mailbox_id UUID NOT NULL,
  author_member_id UUID NOT NULL,
  body_md TEXT NOT NULL,
  mentions UUID[],                            -- Member IDs mentioned via @member
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE email.thread_typing (
  thread_id TEXT NOT NULL,
  mailbox_id UUID NOT NULL,
  member_id UUID NOT NULL,
  composer_kind TEXT NOT NULL,                -- "reply" | "internal_comment"
  started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (thread_id, mailbox_id, member_id, composer_kind)
);
```

**GraphQL extensions.**

```graphql
extend type EmailThread {
  state(mailboxId: ID!): EmailThreadState!
  internalComments(mailboxId: ID!): [EmailInternalComment!]!
  readState(mailboxId: ID!, memberId: ID!): EmailReadState!
  typingMembers(mailboxId: ID!): [EmailTypingIndicator!]!
}

extend type Mutation {
  emailAssignThread(threadId: ID!, mailboxId: ID!, assigneeMemberId: ID): EmailThreadState!
  emailSetStatus(threadId: ID!, mailboxId: ID!, status: String!): EmailThreadState!
  emailSnoozeThread(threadId: ID!, mailboxId: ID!, until: DateTime!): EmailThreadState!
  emailUnsnoozeThread(threadId: ID!, mailboxId: ID!): EmailThreadState!
  emailPostInternalComment(threadId: ID!, mailboxId: ID!, body: String!): EmailInternalComment!
  emailEditInternalComment(commentId: ID!, body: String!): EmailInternalComment!
  emailDeleteInternalComment(commentId: ID!): Boolean!
  emailSetTyping(threadId: ID!, mailboxId: ID!, composerKind: String!, typing: Boolean!): Boolean!
}

extend type Subscription {
  emailThreadStateStream(mailboxId: ID!): EmailThreadStateEvent!
  emailInternalCommentStream(threadId: ID!, mailboxId: ID!): EmailInternalCommentEvent!
  emailTypingStream(threadId: ID!, mailboxId: ID!): EmailTypingEvent!
}
```

WebSocket subscriptions piggyback on the host shell's existing GraphQL-WS connection; per-mailbox subscription scope keeps per-tenant volume tractable.

**Status workflow automation (small set).** Status transitions are mostly user-driven, but a few automatic transitions are surfaced as Notify-mode suggestions (never auto-applied):

- A new outbound reply on a thread in `waiting-internal` → CUO suggests transitioning to `waiting-customer`.
- A new inbound message on a thread in `waiting-customer` → CUO suggests transitioning to `open`.
- A snoozed thread that resurfaces with no new inbound activity → CUO suggests transitioning to `resolved` or extending the snooze.

The Member confirms each transition; auto-application is a P3 feature once acceptance metrics show > 90% confirmation rate per pattern.

**Empty states.** Each shared inbox has an empty state explaining its purpose and the assignment/status workflow; new Members see a one-time tooltip walkthrough on first visit (driven by `auth.member.email_walkthrough_completed`).

**Mobile + accessibility.** The remote is responsive down to 320px viewport and meets WCAG 2.1 AAA contrast on default Design System tokens (FR-DESIGN-001). Keyboard navigation: `j` / `k` to move between threads (Gmail convention); `r` to reply; `c` to compose; `e` to assign-to-me; `s` to set status; `z` to snooze; `i` to internal-comment.

**Audit integration.** Assignments, status changes, snooze, internal-comment posts/edits/deletes write rows in `email.{tenant}` audit scope; internal comments preserve content in the audit row (subject to denylist) for forensic recoverability.

**MCP tool surface (extends FR-EMAIL-001).**

- `cyberos.email.assign_thread(thread_id, mailbox_id, assignee_id)` — `destructive: false`.
- `cyberos.email.set_status(thread_id, mailbox_id, status)` — `destructive: false`.
- `cyberos.email.snooze_thread(thread_id, mailbox_id, until)` — `destructive: false`.
- `cyberos.email.post_internal_comment(thread_id, mailbox_id, body)` — `destructive: false; sensitivity: medium` (visible to teammates).
- `cyberos.email.list_my_assigned_threads` — read.
- `cyberos.email.list_threads_by_status(mailbox_id, status)` — read.

## Alternatives Considered

- **Use Front / Missive directly via API.** Rejected: residency story breaks; the panel-adjacent Genie surface cannot be embedded; pricing scales by user.
- **Shared label in Gmail / Outlook.** Rejected: that's the status quo we're replacing; the status + assignment + snooze + internal-comment primitives don't exist in Gmail labels.
- **Build the UX into Stalwart's webmail directly via a fork.** Rejected: Stalwart's webmail is a thin reference UI; forking it and tracking upstream is a recurring tax. We use Stalwart's JMAP and own the UX.
- **No internal comments; use CHAT for thread discussion.** Rejected: a CHAT thread separate from the email thread loses the lineage; internal comments inline in the thread keep the discussion adjacent to the artefact.

## Success Metrics

- **Primary metric.** P1 mid-sprint demo passes: (1) the founder + Account Manager triage 20 synthetic shared-inbox emails: assignment, status set, internal comments, snooze; (2) read-state sync between two browsers updates within 1 s p95; (3) the snooze-until reminder fires within ± 60 s of the configured time and lands as a Notify card.
- **P1 → P2 gate.** "EMAIL has fully replaced Gmail for at least 21 consecutive days." (PRD §14.2.3.)
- **Adoption metric.** Internal comments per thread ≥ 0.3 average across shared-inbox threads in the 14-day pre-exit window — proves the team uses the primitive rather than reverting to CHAT side-channels.
- **Latency metric.** Thread-list query p95 ≤ 400 ms; thread-detail open p95 ≤ 600 ms.

## Scope

**In-scope.**
- The three-pane Module-Federation remote at `/email`.
- The four schema extensions and the GraphQL surface.
- Assignment, statuses, internal comments, read-state sync, snooze-until, typing indicators.
- Per-tenant configurable status enum with default values.
- The Notify-mode status-transition suggestions.
- Empty-state walkthrough + keyboard shortcuts.
- WebSocket subscriptions over the host shell's GraphQL-WS connection.
- Audit integration.
- The MCP tools listed above.

**Out-of-scope (deferred).**
- Customer-portal-style external view of a shared thread (P4 PORTAL).
- SLA tracking per shared inbox (P2 — falls into customer-success workflows; FR-INV / FR-CRM cluster).
- Auto-categorisation by sales / support / personal / spam (FR-EMAIL-004).
- Mobile native client (P3).
- Per-Member custom keyboard shortcuts (P3 ergonomics).

## Dependencies

- FR-EMAIL-001 (Stalwart core + GraphQL subgraph).
- FR-INFRA-001 (host shell + Postgres + NATS).
- FR-AUTH-001 / FR-AUTH-003 (identity + session sync).
- FR-MCP-001 (MCP tool registration).
- FR-DESIGN-001 (component library + tokens).
- FR-GENIE-001 / FR-GENIE-002 (Notify-card surface for snooze reminders + transition suggestions).
- Compliance: PDPL Decree 13 (internal comments are personal data when they reference Members or external contacts; the audit-row + denylist controls apply).
- Locked decisions referenced: DEC-080 (assignment is single-Member; multi-assignee is forbidden in P1 to prevent the diffusion-of-responsibility failure mode).

## AI Risk Assessment

Not applicable. `eu_ai_act_risk_class: not_ai`. The shared-inbox UX is deterministic; AI-driven status-transition suggestions inherit FR-GENIE-001's classification.

## AI Authorship Disclosure

- **Tools used:** Claude Cowork (Anthropic).
- **Scope:** drafted the FR end-to-end against the PRD + SRS; founder reviews and edits before status changes from `ready_for_review`.
- **Human review:** founder (`@stephen-cheng`) — final wording is the founder's responsibility.
