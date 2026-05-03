---
title: "PROJ — notifications, @mentions, daily standup auto-summary in CHAT, TIME-tracking seam"
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

Close the PROJ surface with the notification fabric, the standup integration, and the TIME-tracking seam: **per-Member notification preferences** (in-app Genie-panel cards, CHAT bot DMs, optional email digests); **@mention semantics** in issue descriptions, comments, and PR-link metadata; **daily standup auto-summary** posted to the per-project CHAT channel by a CUO/COO bot at the team's configured standup time; **PROJ ↔ TIME seam stub** that wires PROJ issue completions to TIME entries (real TIME ships in batch-05); **digest cadences** (daily / weekly / cycle-close); and **per-channel CHAT ↔ Project mapping** so PROJ events can fan out to the right CHAT channel without spam. This FR is the connective tissue between PROJ and the rest of the platform's communication + time-tracking surfaces.

## Problem

Three failure modes the team will hit without this FR:

- **Important changes get lost.** Without a notification fabric, a Member assigning an issue at 17:30 ICT relies on the assignee opening PROJ before the next morning. If the assignee was on PTO, the issue languishes until cycle-close noticing.
- **Standups become manual labour.** The team's daily 09:00 ICT standup today is verbal status updates; the founder summarises into Slack manually. Auto-summary saves 30 minutes per day across the team.
- **Time-tracking decoupled from work.** The team's existing time logging is detached from the issues; logging time on Asana issues is voluntary and inconsistent. The TIME seam is the architectural link.

## Proposed Solution

The shape of the answer is a `cyberos-proj-notifier` service + per-Member preferences + the CHAT bot + the TIME seam stub.

**Notification triggers.**

The notifier subscribes to NATS subjects:
- `cyberos.{tenant}.proj.issue.created`
- `cyberos.{tenant}.proj.issue.updated`
- `cyberos.{tenant}.proj.issue.transitioned`
- `cyberos.{tenant}.proj.issue.assigned`
- `cyberos.{tenant}.proj.issue.commented`
- `cyberos.{tenant}.proj.issue.mentioned`
- `cyberos.{tenant}.proj.cycle.created`
- `cyberos.{tenant}.proj.cycle.closed`
- `cyberos.{tenant}.proj.engagement.health_changed`
- `cyberos.{tenant}.proj.blocker.detected`         (FR-PROJ-006)

For each event, the notifier:

1. Resolves recipients per the *event-relevance graph*:
   - **Direct** — assignee, reporter, mentioned-Members, watchers (Members who clicked "watch this issue").
   - **Indirect** — Project Lead (always for cycle events; conditional on event severity for issue events), Engagement Owner.
   - **Channel** — the project's mapped CHAT channel (FR-CHAT-001), if configured.
2. Applies the recipient's per-event preference (in-app / CHAT-DM / email-digest / off).
3. Applies the global "do-not-disturb" window (per-Member time + day-of-week range, default 22:00-07:00 ICT and weekends).
4. Dispatches: in-app Notify card via Genie panel (FR-GENIE-001); CHAT bot DM via the CHAT module's `cyberos.chat.post_message` (with the user's `client_confirmed` from preference); email digest entry into the next scheduled digest.

**Per-Member preferences.**

```sql
CREATE TABLE proj.notification_preference (
  tenant_id UUID NOT NULL,
  member_id UUID NOT NULL,
  event_kind TEXT NOT NULL,                    -- "issue.assigned" | "issue.mentioned" | etc. | "*"
  surface TEXT NOT NULL,                       -- "panel" | "chat_dm" | "email_digest" | "off"
  PRIMARY KEY (tenant_id, member_id, event_kind)
);

CREATE TABLE proj.notification_dnd (
  tenant_id UUID NOT NULL,
  member_id UUID NOT NULL,
  start_local TIME NOT NULL,
  end_local TIME NOT NULL,
  weekday_mask TEXT NOT NULL,                  -- "Mon,Tue,Wed,Thu,Fri" or "Sat,Sun" etc.
  active BOOLEAN NOT NULL DEFAULT true,
  PRIMARY KEY (tenant_id, member_id)
);
```

Default preferences: assignment / mention → panel + chat-dm; updates → panel-only; cycle events → panel-only; blocker-detected → panel + chat-dm. The HR/Ops Lead can author tenant defaults; Members override per-event.

**@mention semantics.**

Mentions in issue descriptions or comments use `@member` (auto-completion in the editor); the mention resolves to a Member ID via the federation. Mentioned Members receive the `proj.issue.mentioned` event; default preference is panel + chat-dm. Mentions render with the Design System's `<MemberAvatar>` component + a hover card showing role + recent activity.

For backwards-compat with imported issues, `@email-address` and `@github-handle` formats are auto-resolved to the Member ID via `auth.member` lookups.

**Daily standup auto-summary.**

Per Project (or Engagement, configurable), a 09:00 ICT (or per-project-configured) cron runs a CUO/COO summary:

- Inputs: yesterday's `proj.issue_state_transition` events for the Project, comments (CaMeL-sanitised), blocker-Notify events from FR-PROJ-006.
- Output: a 4–8-sentence summary citing: what closed, what moved, what's blocked, what changes since yesterday, who's on call for what.
- Post: via the CHAT bot to the Project's mapped CHAT channel as a thread; the assignee + Project Lead are mentioned where relevant.

The summary's persona-version is stamped; acceptance metrics are tracked (FR-GENIE-002).

**Per-channel mapping.**

```sql
CREATE TABLE proj.chat_channel_link (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  project_id UUID REFERENCES proj.project(id) ON DELETE CASCADE,
  engagement_id UUID REFERENCES proj.engagement(id) ON DELETE CASCADE,
  chat_channel_id UUID NOT NULL,
  link_kind TEXT NOT NULL,                     -- "primary" | "alerts" | "standup_only"
  CHECK ((project_id IS NOT NULL) OR (engagement_id IS NOT NULL))
);
```

A Project can have a primary channel (general fan-out), an alerts channel (sev-1+ events only), and a standup-only channel (the daily auto-summary post). HR/Ops Lead configures these.

**TIME-tracking seam stub.**

The seam (full implementation in batch-05's FR-TIME-001):

- Every `proj.issue.transitioned` event carries the previous state's start time + the transition time. A small consumer creates a candidate `time.entry` record (in TIME's schema, when the module ships) for the assignee with `kind: "issue_work"`, `issue_id: <id>`, `started_at: <prev>`, `ended_at: <now>`.
- The TIME module's UI (batch-05) lets the Member confirm or adjust the candidate.
- Until TIME ships, the candidate records land in `proj.time_candidate{...}` for the future migration.

**Digest cadences.**

- **Daily digest:** per-Member; surfaces in the Founder Daily Flow (FR-GENIE-003) for the founder; a Genie-panel digest card for others; configurable to email at user's request. Lists: my open issues, today's @mentions, my carryover, my completed-yesterday.
- **Weekly digest:** Friday 16:00 ICT; project-summary aggregations for Project Leads.
- **Cycle-close digest:** automatic on cycle close; the cycle review (FR-PROJ-006) is the canonical artefact.

**MCP tool surface.**

- `cyberos.proj.list_notifications(member_id?, since)` (read).
- `cyberos.proj.mark_notifications_read(notification_ids)` (`destructive: false`).
- `cyberos.proj.update_notification_preference(event_kind, surface)` (`destructive: false`).
- `cyberos.proj.set_dnd(start, end, weekday_mask)` (`destructive: false`).
- `cyberos.proj.link_chat_channel(project_id|engagement_id, channel_id, link_kind)` (`destructive: true; requires_confirmation: true`).
- `cyberos.proj.daily_standup_summary(project_id, date?)` (read; runs the summary on demand).

## Alternatives Considered

- **Notify everything by default.** Rejected: notification fatigue is the single largest driver of opt-out; the relevance graph is the floor.
- **Skip the standup auto-summary; let CUO/COO be invoked manually.** Rejected: the founder-cognitive-load goal (PRD §4.1 G8) depends on the daily automation.
- **Email-only notifications (no in-app).** Rejected: in-app is the primary surface; email is the spillover for important events.
- **Tight coupling with TIME so PROJ blocks if TIME is down.** Rejected: TIME is downstream; PROJ events are the source of truth; the candidate-record pattern decouples cleanly.
- **Centralised notification service across all modules** (one notifier for PROJ + EMAIL + CHAT + CRM …). Considered for P3; the per-module notifier is the floor in P1 because module-specific relevance graphs are too different.

## Success Metrics

- **Primary metric.** P1 → P2 exit-gate progress: ≥ 80% of @mentions reach the assignee within 60 s p95; daily standup auto-summary acceptance rate ≥ 50% (the team uses it as the canonical standup post); 0 "missed assignment" incidents (the team's prior tracker had a recurring issue).
- **Founder-cognitive-load metric.** Daily-standup-prep time reduced by ≥ 30 minutes/day across the team.
- **Latency NFR.** Notification dispatch p95 ≤ 5 s from event to surface (in-app or CHAT-DM).

## Scope

**In-scope.**
- `cyberos-proj-notifier` service.
- Per-Member preference table + DnD table + UI in `/auth/account/notifications`.
- @mention resolution + render.
- Daily standup auto-summary cron + post-via-CHAT-bot.
- Per-channel mapping (project/engagement → CHAT channel).
- TIME-seam stub (`proj.time_candidate`).
- Daily / weekly / cycle-close digest cadences.
- The six new MCP tools.
- Audit integration in scope `proj.notification.{tenant}`.

**Out-of-scope (deferred).**
- Cross-module unified notification surface (P3).
- Real TIME-module integration (batch-05 FR-TIME-001).
- Custom notification rules per Member (e.g. "notify me only when X engagement is mentioned") — P3.
- Slack-native integration (P4 if hybrid teams demand it; today CHAT is the canonical channel).

## Dependencies

- FR-PROJ-001..008.
- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-002 / FR-MCP-001 / FR-AI-001.
- FR-CHAT-001 (CHAT bot post path).
- FR-GENIE-001 / FR-GENIE-002 (Notify cards + persona).
- FR-DESIGN-001 (notification card components).
- FR-OBS-001 / FR-OBS-002.
- FR-TIME-001 (batch-05) for the real TIME consumer.
- Compliance: EU AI Act Article 50 (the standup summary renders the disclosure chip); GDPR Article 22 (notifications are reversible / configurable per Member).
- Locked decisions referenced: DEC-124 (per-Member preferences override tenant defaults), DEC-125 (DnD respects time-zone + weekday mask), DEC-126 (TIME-seam stub via candidate records).

## AI Risk Assessment

The standup auto-summary is the AI surface in this FR. EU AI Act risk class: `limited`.

### Data Sources

Per-tenant only: PROJ events + CHAT messages (CaMeL-sanitised) + BRAIN context. No third-party.

### Human Oversight

- The standup post is a Notify card by default (the team confirms before it posts); a per-project preference can flip to "auto-post if the founder is OK with it". The default preserves human-in-the-loop.
- Notification preferences are per-Member, per-event, configurable.
- DnD is enforced before any dispatch.
- Mentions are explicit user intent (@-typed); never auto-mentioned by AI.

### Failure Modes

- **Auto-standup posts wrong content.** Mitigation: persona regression suite covers cases; the founder retracts via CHAT bot's edit; the audit row records the retraction.
- **Notification storm on a high-velocity day.** Mitigation: per-event rate limit (≤ 20 dispatches/Member/hour for any single event_kind); over-limit collapse into a digest.
- **Mention resolves to wrong Member** (collision on `@khoa`). Mitigation: the autocompleter shows full name + role; the Member picks; the resolution is stored per-mention.
- **TIME candidate records bloat without consumption.** Mitigation: 90-day retention until TIME ships; metrics on candidate-record count drive the FR-TIME-001 priority.

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6.
- **Scope:** Drafted notification fabric, standup auto-summary, TIME seam stub, failure modes.
- **Human review:** `@stephen-cheng` reviewed; the daily-standup post template will be co-authored with a vi-native Account Manager at PR-review.
