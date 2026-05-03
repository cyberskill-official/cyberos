---
title: "HR — 1:1 templates with Genie prep brief, employee directory, organisation chart, frontend remote at /hr"
author: "@stephen-cheng"
department: human_resources
status: ready_for_review
priority: p2
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: user_facing
eu_ai_act_risk_class: limited
target_release: "P2 / 2027-Q1"
client_visible: false
template: feature_request@1
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Ship the HR-facing UX layer: **1:1 templates** (manager-direct-report; peer; founder-skip-level) with structured agendas; **Genie 1:1 prep brief** — when a manager has a 1:1 scheduled (FR-TIME-002 calendar import), CUO/CHRO drafts a one-page brief covering the report's recent work (PROJ), open blockers, recent CHAT activity in their channels, prior 1:1 takeaways, and suggested talking points; **1:1 follow-through tracking** — action items captured during the meeting, surfaced as Notify cards until completed; **employee directory** at `/hr/directory` with search, filter, role/team browsing; **organisation chart** at `/hr/org` rendering reports-to relationships from `hr.role_history` as a navigable tree; **per-Member self-service profile** at `/auth/account` extended with HR-aware fields (preferred name, pronouns, working pattern, photo); the **frontend remote at `/hr`** that ties together onboarding (FR-HR-002), directory, org chart, and 1:1s; and per-role **HR/Ops Lead admin views** for the underlying data. PRD §14.3.1 explicitly scopes "1:1 templates, Genie 1:1 prep brief".

## Problem

The team's current 1:1s are scheduled in Google Calendar with no agenda; a Slack message captures the takeaways, often imperfectly; follow-ups disappear. Three failure modes:

- **Unprepared 1:1s.** A manager walking into a 1:1 without context produces a transactional check-in instead of a useful conversation. The PRD §4.1 G8 founder-cognitive-load goal includes the founder's 1:1 cadence; without prep, the cost is the founder's time.
- **Dropped action items.** "I'll look into that" said in a 1:1 disappears unless captured. The team's prior tracker had no place for these.
- **Org-chart opacity.** "Who reports to whom?" is answerable today by asking the founder. As the team grows past 10, this becomes a bottleneck.

## Proposed Solution

The shape of the answer is `hr.one_on_one_*` schema, the CUO/CHRO prep-brief generator, the directory + org-chart views, and the `/hr` frontend remote.

**Schema.**

```sql
CREATE TABLE hr.one_on_one_template (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  template_kind TEXT NOT NULL,                          -- "manager_direct_report" | "peer" | "founder_skip_level"
                                                        -- | "buddy" | "ad_hoc"
  display_name TEXT NOT NULL,
  default_duration_minutes INT NOT NULL,
  default_cadence_days INT,                              -- e.g. 7 for weekly; null for ad-hoc
  agenda_md TEXT NOT NULL,                               -- the structured agenda Markdown
  ai_assist_enabled BOOLEAN NOT NULL DEFAULT true,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  is_active BOOLEAN NOT NULL DEFAULT true
);

CREATE TABLE hr.one_on_one (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  template_id UUID REFERENCES hr.one_on_one_template(id),
  scheduled_for_at TIMESTAMPTZ NOT NULL,
  duration_minutes INT NOT NULL,
  participant_member_ids UUID[] NOT NULL,                -- typically two; supports group 1:1s
  primary_owner_member_id UUID NOT NULL,                  -- the convener (typically the manager)
  agenda_md TEXT,                                         -- starts as template.agenda_md; drift over time
  prep_brief_md TEXT,                                     -- the AI-generated prep-brief
  prep_brief_persona_version TEXT,
  prep_brief_generated_at TIMESTAMPTZ,
  notes_md TEXT,                                          -- captured during/after the meeting
  status TEXT NOT NULL DEFAULT 'scheduled',               -- "scheduled" | "completed" | "cancelled" | "rescheduled"
  completed_at TIMESTAMPTZ,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX one_on_one_participant_idx ON hr.one_on_one USING gin (participant_member_ids);
CREATE INDEX one_on_one_owner_idx       ON hr.one_on_one (tenant_id, primary_owner_member_id, scheduled_for_at);

CREATE TABLE hr.one_on_one_action (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  one_on_one_id UUID NOT NULL REFERENCES hr.one_on_one(id) ON DELETE CASCADE,
  description_md TEXT NOT NULL,
  owner_member_id UUID NOT NULL,
  due_at DATE,
  status TEXT NOT NULL DEFAULT 'open',                    -- "open" | "in_progress" | "completed" | "cancelled"
  completed_at TIMESTAMPTZ,
  linked_proj_issue_id UUID,                              -- when the action becomes a PROJ issue
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Member self-declared preferences and profile fields exposed in the directory.
CREATE TABLE hr.profile_self (
  tenant_id UUID NOT NULL,
  employee_id UUID NOT NULL UNIQUE REFERENCES hr.employee(id) ON DELETE CASCADE,
  pronouns TEXT,
  bio_md TEXT,                                            -- short bio surfaced in the directory
  photo_url TEXT,                                         -- references the content-addressed blob store
  working_pattern_md TEXT,                                 -- e.g. "Mon-Fri, 09:00-18:00 ICT, async-friendly"
  social_links JSONB NOT NULL DEFAULT '[]'::jsonb,        -- [ { kind: "linkedin", url: ... } ]
  expertise_tags TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],  -- "rust", "design-systems", "vietnamese-tax"
  preferred_communication TEXT,                            -- "async-first" | "sync-first" | "mixed"
  current_focus_md TEXT,                                   -- short note on what they're working on
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (tenant_id, employee_id)
);
```

**Default 1:1 templates seeded.**

| Kind | Cadence | Duration | Sample agenda sections |
|---|---|---|---|
| manager_direct_report | weekly | 30 min | "Wins this week" / "Blockers" / "Career growth" / "Feedback" / "Action items" |
| founder_skip_level | quarterly | 30 min | "How are you?" / "What's going well?" / "What's frustrating?" / "What should we change?" / "What would you build if you could?" |
| peer | bi-weekly | 30 min | "What you're working on" / "What I'm working on" / "How can we help each other?" / "Action items" |
| buddy | week-1 then ad-hoc | 60 min then 30 min | "Onboarding progress" / "Stuck areas" / "Questions about the team" / "Action items" |
| ad_hoc | one-off | 30-60 min | "Topic" / "Discussion" / "Action items" |

The HR/Ops Lead can clone + edit templates; managers can author personal templates per direct report.

**Genie 1:1 prep brief.**

When a 1:1 is scheduled (typically via calendar — FR-TIME-002 calendar import detects it; or via direct creation in HR), CUO/CHRO drafts the prep-brief 1 hour before the meeting:

1. Inputs: the participants' recent PROJ activity (last 7 days of issue mutations); recent CHAT activity in their shared channels; prior 1:1 notes + open actions; recent feedback signals (LEARN P2 carries this); recent BRAIN facts about each participant.
2. Output: a structured prep-brief:
   - **Recap of last 1:1** — open actions, decisions, follow-up items.
   - **What's been happening** — 3-5 sentences summarising recent work.
   - **Open blockers** — issues from PROJ in `blocked` state assigned to the report.
   - **Suggested talking points** — 3-5 points based on patterns (e.g. "you mentioned wanting more design feedback last 1:1; the design review process changed two weeks ago — discuss?").
   - **Career-growth notes** — when the meeting is `manager_direct_report` and a recurring growth-conversation pattern is detected.
3. The brief is rendered in the `/hr/one-on-ones/<id>` page; the manager opens it 5-15 minutes before the 1:1.

Latency: pre-generated 1 hour before the meeting; on-demand recompute ≤ 6 s p95.

**1:1 notes during the meeting.**

A simple Markdown editor pinned to the bottom of the `/hr/one-on-ones/<id>` page. Members can co-edit (Yjs CRDT reused from FR-BRAIN-001 / FR-KB-001). Action items are extracted via a slash command `/action @member due:friday "follow up on Acme rate-card change"` — creates a `hr.one_on_one_action` row.

**Action-item lifecycle.**

- Created during 1:1 with optional due-date + owner.
- Surfaced as Notify card to the owner the next day.
- Can be promoted to a PROJ issue with one click (uses FR-PROJ-008's `nlcrud_propose_issue` pattern).
- On completion, the original 1:1 receives a "follow-up completed" link.
- Rolling open-action count surfaced in the directory + manager's daily flow.

**Employee directory (`/hr/directory`).**

A grid of cards (Member photo + preferred-name + title + team + working-pattern + expertise tags). Search across name / role / expertise / current focus. Filters: team, status, location/timezone, tenure-bucket. Click a card → `/hr/people/<member-id>` profile view.

**Profile view (`/hr/people/<member-id>`).**

- Header: name + photo + pronouns + title + team + reports-to.
- Tabs: Overview / Career history / 1:1 history (with-me) / Direct reports (if any) / Recent work (PROJ summary + KB authorship).
- Recent activity feed.
- A "Schedule 1:1" button that opens the 1:1-template chooser + calendar slot suggestion (uses FR-TIME-001's calendar integration).

The **Career history tab** renders `hr.role_history` as a chronological list (transparent organisational history is part of the platform's transparent-by-design posture).

**Org chart (`/hr/org`).**

A tree view of reports-to relationships:
- Root: Founder.
- Branches: direct reports → their reports → ...
- Each node is a Member card (clickable to profile).
- Toggle: by team / by Engagement / flat (all employees).
- Search highlights matching nodes.
- Export as PNG / PDF for board materials.

The org chart is the single shareable artefact for "who's who"; the founder uses it in fundraising materials at P3 (per PRD §14.4).

**Member self-service profile.**

`/auth/account/profile-public` lets a Member edit their `hr.profile_self` row: pronouns, bio, photo, expertise tags, current focus, social links. Some fields (title, team, reports-to) are read-only — they're managed by HR/Ops via `hr.role_history`.

**Frontend remote at `/hr`.**

Routes:
- `/hr` — the home; HR/Ops Lead view by default; redirects to a Member-friendly view if not HR/Ops.
- `/hr/onboarding` — FR-HR-002.
- `/hr/directory` — directory grid.
- `/hr/people/<id>` — profile view.
- `/hr/org` — org chart.
- `/hr/one-on-ones` — list of 1:1s (mine, my reports', all-tenant for HR/Ops).
- `/hr/one-on-ones/<id>` — single 1:1 view with prep brief, agenda, notes, actions.
- `/hr/templates` — 1:1 + onboarding template management (HR/Ops + Founder).

Bundle ≤ 50 KB gzipped; mobile-responsive.

**MCP tool surface.**

- `cyberos.hr.list_one_on_ones(member_id?, since)` — read.
- `cyberos.hr.get_one_on_one(id)` — read.
- `cyberos.hr.draft_prep_brief(one_on_one_id)` — read; on-demand recompute.
- `cyberos.hr.create_one_on_one(input)` — `destructive: true; requires_confirmation: true`.
- `cyberos.hr.add_action(one_on_one_id, description, owner, due)` — `destructive: false`.
- `cyberos.hr.complete_action(action_id)` — `destructive: false`.
- `cyberos.hr.list_directory(filters?)` — read.
- `cyberos.hr.get_org_chart(root_member_id?)` — read.
- `cyberos.hr.list_open_actions(member_id?)` — read.
- `cyberos.hr.update_self_profile(patch)` — `destructive: false`; per-Member self-service.

CUO scope contracts: read all + draft brief + add action allowed; commit-mutations on schedule/template restricted by HR/Ops role.

## Alternatives Considered

- **Use Lattice / 15Five / Culture Amp.** Rejected: residency + the integration with PROJ + BRAIN + the persona-stamping is not viable hosted.
- **Skip the org chart in P2.** Rejected: the org chart is the single most-asked artefact at any team-growth phase.
- **Auto-extract action items from notes via AI.** Considered. P2 ships explicit `/action` slash command (deterministic). P3 may add an "Extract action items" CUO action that proposes (review-mode) — but never auto-creates.
- **Skip per-Member self-service profile.** Rejected: pronouns + bio + photo + expertise are the cheapest possible introductions for new hires.

## Success Metrics

- **Primary metric.** P2 sprint demo passes: (1) the founder schedules a 1:1 with a direct report; (2) CUO/CHRO drafts the prep brief 1 hour before with citations to recent PROJ + CHAT activity; (3) action items captured during the meeting surface as Notify cards next day; (4) the org chart renders correctly for the 10-employee structure; (5) every Member has a self-edited public profile.
- **Adoption metric.** ≥ 80% of weekly 1:1s use the templated agenda; ≥ 60% of completed 1:1s have at least one action item recorded; action-item completion rate ≥ 70% within 14 days.
- **Latency NFR.** Prep-brief draft (on-demand) p95 ≤ 6 s; directory grid p95 ≤ 600 ms.

## Scope

**In-scope.**
- The 4 schema additions (`one_on_one_template`, `one_on_one`, `one_on_one_action`, `profile_self`).
- 5 default 1:1 templates seeded.
- CUO/CHRO prep-brief generator with the 5-section structure.
- Yjs-backed live notes editor in 1:1 view.
- `/action` slash command.
- Action-item Notify lifecycle.
- Action → PROJ-issue promotion.
- Employee directory + profile view.
- Org chart with tree rendering + filters + export.
- Member self-service profile UI.
- The 10 MCP tools.
- Audit integration in scope `hr.one_on_one.{tenant}`.

**Out-of-scope (deferred).**
- Auto-action-extraction from notes (P3 — Review mode).
- Performance reviews + Hội đồng Chuyên môn (LEARN P2 batch-07).
- 360 feedback (P2 LEARN cluster).
- Anonymous feedback channel (P3).
- Skip-level cadence enforcement (P3).

## Dependencies

- FR-HR-001 / FR-HR-002.
- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-002 / FR-MCP-001 / FR-AI-001.
- FR-DESIGN-001.
- FR-GENIE-001 / FR-GENIE-002 (Notify cards; CUO/CHRO).
- FR-BRAIN-001 (Yjs reused for collaborative notes).
- FR-PROJ-001 / FR-PROJ-008 (action → issue promotion).
- FR-TIME-001 / FR-TIME-002 (calendar import for 1:1 detection).
- FR-EMAIL-001..010 (calendar invite path).
- FR-OBS-001 / FR-OBS-002.
- Compliance: PDPL Decree 13 (1:1 notes + profile data is personal data; per-tenant residency); EU AI Act Article 50 (prep-brief is AI-generated content; disclosure chip).
- Locked decisions referenced: DEC-162 (5 default 1:1 templates), DEC-163 (action items captured via deterministic slash command in P2; AI extraction P3+), DEC-164 (org chart is internal-only by default; export-to-image is the share path).

## AI Risk Assessment

The CUO/CHRO prep-brief is an AI surface that influences how managers conduct 1:1s with direct reports. EU AI Act risk class: `limited`.

### Data Sources

Per-tenant only: PROJ + CHAT + BRAIN + prior 1:1 notes. No third-party. CUO/CHRO runs through the AI Gateway with persona-stamping.

### Human Oversight

- The prep brief is suggestion-only; the manager reads + interprets.
- Action items are captured via deterministic slash command; AI does not extract autonomously.
- 1:1 notes are co-edited; AI never edits notes.
- Profile data is self-edited; AI does not modify.

### Failure Modes

- **Prep brief surfaces wrong context** (e.g. recent activity is from a different report). Mitigation: the brief cites sources; the manager spot-checks; CUO acceptance metric tracks the suggestion's usefulness.
- **Action item dropped.** Mitigation: open-action Notify cadence; rolling count visible in the daily flow; HR/Ops Lead can audit completion rates.
- **Org chart drift.** Mitigation: the chart is derived from `hr.role_history`; any drift is itself a data-quality issue.
- **Self-profile abuse** (Member writes inappropriate content). Mitigation: HR/Ops Lead can flag + revert; the audit log captures every change.

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6.
- **Scope:** Drafted 1:1 schema, prep-brief structure, directory + org chart, MCP tools, failure modes.
- **Human review:** `@stephen-cheng` reviewed; the founder + first manager use the system live before declaring P2 → P3 readiness.
