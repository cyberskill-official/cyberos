---
title: "EMAIL — promote a thread to a PROJ task with the email linked, summary auto-drafted, recipient mapped to assignee suggestion"
author: "@stephen-cheng"
department: product
status: ready_for_review
priority: p1
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: user_facing
eu_ai_act_risk_class: limited
target_release: "P1 / 2026-Q4"
client_visible: false
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Wire the EMAIL ↔ PROJ seam. From any thread, a Member can promote it to a PROJ task with one action: the task title is auto-drafted from the thread subject; the description is auto-drafted as a CUO-summary of the thread; the source email is linked back as a structured reference (clickable from the task; clickable from the email back to the task); the assignee is suggested from the team's recent activity on similar threads; the project is suggested by the linked Engagement → Project mapping; the task is created in the PROJ module's `Issue` primitive (FR-PROJ-001 in batch-04). Outbound: when a PROJ task is closed and the closing comment is "send update to client", a CUO Review-mode draft email is staged in EMAIL with the task as context. Both directions respect persona-scope contracts and human-in-the-loop confirmation.

## Problem

The team's daily workflow oscillates between EMAIL and PROJ: a customer email becomes work; a piece of work becomes a customer update. Without a structured promotion path, the workflow goes through a Slack message, two screens, and three copy-pastes — and the customer email's reference is lost. The PRD §9.4.3 names "thread 'promote to project task' — turns an email into a PROJ task with the thread linked" as the canonical AI-native feature for EMAIL.

PROJ ships in batch-04; this FR ships the EMAIL side of the seam with stubs against PROJ's interfaces, and a small bidirectional reference.

## Proposed Solution

The shape of the answer is a "Promote to task" action in the EMAIL UX, the auto-draft pipeline through CUO, the per-task email-source link, and the reverse path from task-close to email-draft.

**Promote-to-task flow.**

1. Member opens a thread; clicks **Promote to task** (keyboard shortcut `t`; also surfaced on the assignment + status bar).
2. A modal opens with auto-drafted fields:
   - **Title** — the email subject, stripped of `Re: ` / `Fwd: ` prefixes; ≤ 96 chars.
   - **Description** — a Markdown summary of the thread's CaMeL-sanitised facts + the most recent inbound message, ≤ 500 words. The description includes a structured back-link block: `> Source email: thread <link to /email/thread/{id}>; latest message <date> from <sender>; participants: <list>`.
   - **Project** — suggested via Engagement → Project → Contact mapping (the contact is in CRM via FR-EMAIL-006; the contact's Engagement maps to a Project in PROJ). If multiple Projects map, the modal shows a chooser. Default: the most recently active.
   - **Cycle** — suggested as the Project's current Cycle.
   - **Assignee** — suggested from: (1) explicit `@member` mention in the email body; (2) the Member who handled prior tasks for this Engagement; (3) the Member who previously replied in this thread. The picker shows confidence + reason. Default: empty (force a choice for the assignment).
   - **Priority** — defaulted to `medium`; CUO can suggest `high` if the email's CaMeL `sentiment` is `negative` or the body matches escalation patterns.
   - **Due date** — suggested if the email mentions a date; parsed by `chrono` Rust crate; surfaced as a chip the Member confirms.
   - **Labels** — pre-selected from CUO classification (e.g. `bug`, `feature-request`, `customer-question`).
3. Member edits any field, clicks **Create task**. The PROJ task is created via `cyberos.proj.create_issue(...)` (FR-PROJ-001 in batch-04; stub here).
4. The thread's state is updated: a "linked task: PROJ-1234" chip appears in the thread row + the thread state (FR-EMAIL-002) gets a `linked_task_id`.
5. The reverse link is written to PROJ: the Issue's `external_refs` array gets `{kind: "email_thread", id: <thread-id>, url: <deep link to email>}`.
6. The thread can have multiple linked tasks; tasks can be linked to multiple threads (many-to-many via `email.thread_task_link`).

**Schema.**

```sql
CREATE TABLE email.thread_task_link (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  thread_id TEXT NOT NULL,
  mailbox_id UUID NOT NULL,
  task_id UUID NOT NULL,
  created_by UUID NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  unlinked_at TIMESTAMPTZ
);

CREATE INDEX thread_task_link_thread_idx ON email.thread_task_link (tenant_id, thread_id) WHERE unlinked_at IS NULL;
CREATE INDEX thread_task_link_task_idx   ON email.thread_task_link (tenant_id, task_id) WHERE unlinked_at IS NULL;
```

**Auto-draft pipeline.** When the modal opens, the auto-draft starts asynchronously and fills the fields as they arrive:

1. Gather thread context: CaMeL facts (FR-EMAIL-003), CRM context (FR-EMAIL-006), Engagement mapping.
2. Single AI Gateway call with the CUO/COO skill: "draft a PROJ task from this thread. Output JSON of {title, description_md, suggested_assignee, suggested_priority, suggested_due, labels}".
3. Stream the JSON into the modal as fields arrive (the title appears first; the description streams).
4. Latency budget: title ≤ 1.5 s p95; full draft ≤ 5 s p95 (streamed).

**Reverse path: task-close → email draft.**

1. PROJ emits `cyberos.{tenant}.proj.issue.closed` with the closing comment.
2. The reverse-path consumer in EMAIL inspects: if the issue has at least one linked email thread *and* the closing comment matches a "notify customer" pattern (regex + CUO classifier with `kind: "send-customer-update"`), CUO/COO drafts a Review-mode email reply on the original thread.
3. The Review card lands in the assignee's panel: title "Reply to thread re: <subject>"; the draft body summarises the work + closing comment in a customer-facing register.
4. The Member edits + approves; send + step-up + audit applies as normal.

**Bidirectional reference UI.**

- **In EMAIL:** the thread row shows a small "→ PROJ-1234" chip; clicking opens the task in a side-drawer (the PROJ Module-Federation remote loads the task in a panel without leaving EMAIL).
- **In PROJ:** the task header shows "← Email thread: <subject>"; clicking opens the EMAIL thread in a side-drawer.

**Persona-scope.** CUO/COO declares `cyberos.proj.create_issue` in `tools_allowed` for the auto-draft path but only as a *suggestion*; the actual task creation happens via `cyberos.proj.create_issue` invoked by the human (or by the agent under destructive-confirmation in FR-MCP-001). CUO never auto-creates tasks.

**MCP tool surface (extends FR-EMAIL series).**

- `cyberos.email.draft_task_from_thread(thread_id)` — `destructive: false`; returns the auto-drafted task fields without creating.
- `cyberos.email.link_thread_to_task(thread_id, task_id)` — `destructive: false`; links without creating a new task (when the task already exists).
- `cyberos.email.unlink_thread_from_task(thread_id, task_id)` — `destructive: false`.
- `cyberos.email.draft_reply_from_task_close(task_id)` — `destructive: false`; reverse-path suggestion.

## Alternatives Considered

- **A general "convert to anything" surface that handles email → task / KB page / CRM activity / decision log.** Rejected for P1: each conversion has different field mappings; one-size-fits-all is too generic. We will reuse the auto-draft pipeline pattern for KB and decision-log conversions in P2 batch-08+ via separate FRs.
- **Auto-create a task on every inbound email matching a "support" classification.** Rejected: noise + duplicate tasks; the human-confirm step is the floor.
- **Defer the entire seam to PROJ batch-04.** Rejected: the EMAIL side hooks need to exist when PROJ ships; a stubbed promote-flow gives the team practice surface.
- **Use only structural references (no auto-drafted task body).** Rejected: the auto-draft is where most of the time-saving lives; without it the action is "open a new browser tab".

## Success Metrics

- **Primary metric.** Once PROJ ships in batch-04: ≥ 60% of customer-question / bug-report inbound emails get promoted to a PROJ task within 24 hours; the auto-draft is accepted-with-edit ≥ 70% of the time (i.e. the auto-draft is useful, not noise).
- **Latency metric.** Modal-open to title-rendered ≤ 1.5 s p95; full draft streamed ≤ 5 s p95.
- **Reverse-path metric.** ≥ 30% of task-closes with a linked email yield a Review-mode reply draft that the Member approves with ≤ 1 minute of editing.

## Scope

**In-scope.**
- The "Promote to task" UX in EMAIL.
- The auto-draft pipeline through CUO/COO.
- The `email.thread_task_link` table + bidirectional reference UI.
- The reverse-path consumer + Review-mode email-draft generator.
- Suggested-assignee + suggested-project + suggested-priority + suggested-due-date logic.
- The four new MCP tools.
- Stubs against PROJ's interfaces (real in batch-04).

**Out-of-scope (deferred).**
- Auto-promote heuristics (P3 if acceptance metrics support it).
- Conversion to KB / CRM / decision-log (P2 batch-08+ via separate FRs).
- Multi-language reverse-path drafts beyond what FR-EMAIL-005 already enables.
- Threaded sub-tasks for very-long threads (P2; for now one task per promotion).

## Dependencies

- FR-EMAIL-001 / FR-EMAIL-002 / FR-EMAIL-003 / FR-EMAIL-004 / FR-EMAIL-005 / FR-EMAIL-006.
- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-003 / FR-MCP-001 / FR-AI-001.
- FR-BRAIN-001 / FR-BRAIN-002.
- FR-CRM-001 + FR-CRM-002 (batch-05) for Engagement/Contact mapping; this FR ships against stubs.
- FR-PROJ-001 + FR-PROJ-002 (batch-04) for the canonical task creation surface.
- Compliance: EU AI Act Article 50 (auto-drafted task descriptions are AI-generated content; the task description renders the disclosure chip until human-edited and approved).
- Locked decisions referenced: DEC-090 (promote is human-initiated, never auto), DEC-091 (many-to-many thread ↔ task linking).

## AI Risk Assessment

This FR materially shapes how AI-derived content flows from email into tasks (which then flow into work plans, customer updates, etc.). EU AI Act risk class: `limited`.

### Data Sources

Per-tenant only: thread + CRM + BRAIN. No third-party. Auto-draft runs through the AI Gateway with persona-stamping.

### Human Oversight

- Promotion is human-initiated.
- The auto-draft populates the modal; the human edits and clicks Create.
- The reverse path produces a Review-mode draft only; no auto-send.

### Failure Modes

- **Wrong assignee suggested.** The Member picks the right one before creating; the per-Engagement assignment-pattern data is updated.
- **Duplicate task created.** Mitigation: the modal checks for existing linked tasks and shows them as "an existing task is already linked — link to it instead?" before creation.
- **Task description leaks confidential context.** Mitigation: CaMeL-sanitised inputs only; the task description is reviewed by the human before save.
- **Reverse-path drafts a customer update referencing internal-only context.** Mitigation: the persona's voice rules separate internal vs. external register; the Review surface shows the draft to the human before send; the destructive + step-up gate is the floor.

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6.
- **Scope:** Drafted promote-flow + auto-draft pipeline + reverse-path + failure modes.
- **Human review:** `@stephen-cheng` reviewed; PROJ-side details to be re-aligned with batch-04's FR-PROJ-001 at PR-review.
