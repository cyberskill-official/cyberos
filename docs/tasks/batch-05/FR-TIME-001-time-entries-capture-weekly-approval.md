---
title: "TIME — time entries schema + capture flow (manual + PROJ candidate consumption + calendar import) + weekly approval"
author: "@stephen-cheng"
department: engineering
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

Stand up the TIME module's core: a Postgres schema for time entries + approvals + categories + adjustments; **manual time entry** with a fast keyboard-first form; **PROJ time-candidate consumption** (the candidates emitted by FR-PROJ-010 are the largest source of entries — Members confirm + adjust rather than enter from scratch); **calendar import** from Google Calendar / Microsoft 365 with auto-categorisation; **CUO/COO auto-categorisation** suggestions; **weekly approval flow** (Member submits Friday; HR/Ops Lead approves Monday; rejection routes back with comment); **billable / non-billable / internal** classification at entry-level; **Engagement → Project → Cycle → Issue** linking inherited from PROJ; and the **INV feed** stub (P2 INV consumes approved entries for invoicing). The module is the source of truth for "who worked on what for how long" and the substrate for capacity planning (FR-PROJ-004), revenue sharing (RES P2), and invoicing (INV P2).

## Problem

The team's current time-tracking is voluntary, inconsistent, and detached from the work. Three failure modes:

- **Friction-driven decay.** Members forget to log; by Friday the previous week is a blur. The PRD §9.10 explicitly addresses this: "TIME entries by Member by Engagement / Task; expense capture; weekly approval flow; feeds INV."
- **Disconnected from work.** Without PROJ candidate-record consumption (FR-PROJ-010), every time entry is re-entered manually. The candidate-record pattern reduces friction by 70%+ for engineering work.
- **Capacity and billing depend on it.** Without TIME data, capacity planning (FR-PROJ-004) uses point-velocity proxies instead of hours, and invoicing (INV P2) cannot bill T&M Engagements accurately.

## Proposed Solution

The shape of the answer is a `time` schema, an Apollo subgraph, the entry surfaces (manual + candidate + calendar), the weekly approval flow, and the INV feed stub.

**Schema.**

```sql
CREATE SCHEMA time;

CREATE TABLE time.entry (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  member_id UUID NOT NULL,
  engagement_id UUID,                          -- references proj.engagement
  project_id UUID,                              -- references proj.project
  cycle_id UUID,                                -- references proj.cycle
  issue_id UUID,                                -- references proj.issue
  category TEXT NOT NULL,                       -- "billable_dev" | "billable_design" | "billable_meeting"
                                                -- | "internal_admin" | "internal_learning"
                                                -- | "internal_company" | "non_billable_pto"
  description TEXT NOT NULL,
  occurred_on DATE NOT NULL,
  started_at TIMESTAMPTZ,                       -- nullable for "block of time" entries
  ended_at TIMESTAMPTZ,
  duration_minutes INT NOT NULL,
  is_billable BOOLEAN NOT NULL,
  rate_card_role TEXT,                          -- mapped to engagement.rate_card.rates[].role
  source TEXT NOT NULL,                         -- "manual" | "proj_candidate" | "calendar" | "imported"
  source_ref TEXT,                              -- the originating proj.time_candidate id, calendar event id, or import-batch id
  status TEXT NOT NULL DEFAULT 'draft',          -- "draft" | "submitted" | "approved" | "rejected" | "auto_approved"
  submitted_at TIMESTAMPTZ,
  approved_by UUID,
  approved_at TIMESTAMPTZ,
  rejection_reason TEXT,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX time_entry_member_date_idx ON time.entry (tenant_id, member_id, occurred_on);
CREATE INDEX time_entry_engagement_idx  ON time.entry (tenant_id, engagement_id, occurred_on);
CREATE INDEX time_entry_status_idx      ON time.entry (tenant_id, status, occurred_on);

-- Weekly summary roll-up; recomputed nightly + on-mutation.
CREATE TABLE time.week_summary (
  tenant_id UUID NOT NULL,
  member_id UUID NOT NULL,
  week_start DATE NOT NULL,                     -- ISO week start, Monday
  total_minutes INT NOT NULL,
  billable_minutes INT NOT NULL,
  per_engagement JSONB NOT NULL,                -- { engagement_id -> minutes }
  per_category JSONB NOT NULL,                  -- { category -> minutes }
  status TEXT NOT NULL,                         -- "open" | "submitted" | "approved" | "partially_approved"
  submitted_at TIMESTAMPTZ,
  approved_at TIMESTAMPTZ,
  PRIMARY KEY (tenant_id, member_id, week_start)
);

-- Adjustments after approval (rare; audited).
CREATE TABLE time.adjustment (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  entry_id UUID NOT NULL REFERENCES time.entry(id),
  adjusted_by UUID NOT NULL,
  adjusted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  field TEXT NOT NULL,                          -- "duration_minutes" | "category" | "engagement_id" | "is_billable"
  old_value JSONB,
  new_value JSONB,
  reason_md TEXT NOT NULL                        -- mandatory
);
```

**Manual entry.**

A fast entry form in `/time` (FR-TIME's frontend remote, shipped alongside in this FR's "frontend" sub-scope):

- One row per entry: date / duration / category / engagement / project / issue (auto-populated from issue) / description / billable toggle.
- Keyboard shortcuts: `n` new entry; `Tab` advances field; `Enter` saves; `Cmd-Enter` saves and creates the next.
- Auto-complete on engagement / project / issue from the recently-touched list.
- Description can be a single line; longer narrative optional.
- Time formats: `2h 30m`, `2.5h`, `150m`, `9:00–11:30`, `now-30m` (last 30 minutes from current time).

**PROJ candidate consumption.**

The `proj.time_candidate` records emitted by FR-PROJ-010 are surfaced in a "Pending candidates" inbox at the top of the daily TIME view:

- Each candidate shows: issue title + key, suggested duration (from state-transition timestamps), suggested category (engineering work for engineers; design for designers — derived from Member role).
- One-click accept → creates a `time.entry` with `source: 'proj_candidate'`, `source_ref: <candidate_id>`, fields populated from the candidate.
- Adjust + accept → opens the manual entry form pre-populated; user adjusts; saves.
- Dismiss → removes the candidate without creating an entry; an audit row records the dismissal.

The PROJ candidate path is expected to be the largest source of entries (≥ 60% by volume).

**Calendar import.**

A per-Member opt-in OAuth flow (Google Calendar / Microsoft 365 Calendar):

- Daily 06:00 ICT pull of yesterday's events.
- Each event becomes a candidate entry: duration from event length; description from event title; category suggested by CUO/COO ("Acme weekly sync" → `billable_meeting` + engagement: Acme; "internal team standup" → `internal_company`).
- Surfaced in the same "Pending candidates" inbox.
- Per-Member preference: events with specific calendars / patterns can be auto-rejected (e.g. private appointments) or auto-accepted (e.g. recurring sync with a specific client).

**CUO/COO auto-categorisation.**

For both manual and candidate entries, CUO/COO suggests:
- The Engagement (from the description + recent activity).
- The Project + Issue (from PROJ context).
- The category (from past patterns).
- The billable flag (from category + engagement contract).

Suggestions are suggestions; the Member confirms.

**Weekly approval flow.**

The flow:

1. **Friday 17:00 ICT** — `cyberos-time-weekly-reminder` Notify card lands in each Member's panel: "Submit this week's time?".
2. **Member submits** — clicks "Submit week". Server-side: every `draft` entry for the week becomes `submitted`; the `time.week_summary` row updates.
3. **HR/Ops Lead reviews** — a "Time approvals" view shows submitted weeks per Member. Lead can: approve all, reject specific entries with comment, request adjustment.
4. **Rejection routes back** — entries reverting to `draft` with `rejection_reason`; Member sees the rejection and the comment; resubmits.
5. **Approval** — `approved`; the entry feeds INV (P2; today, the data is materialised as an aggregate `time.week_summary` for INV to consume later).
6. **Auto-approval** — entries from PROJ candidates that exactly match the Member's last-30-day pattern + are < 8 hours / day can opt into auto-approval (per-Member preference, defaults off).

**Adjustment after approval.**

Rare but supported: HR/Ops Lead can adjust an approved entry with `time.adjustment` row. The adjustment is fully audited; INV (P2) handles re-billing logic.

**Frontend.**

A small Module-Federation remote at `/time` with three views:
- **Day view** — today's entries + pending candidates + manual-entry row.
- **Week view** — calendar-style grid; entries shown as blocks; click to edit.
- **Approval view** — HR/Ops Lead only; weekly submissions per Member.

Initial bundle ≤ 50 KB gzipped.

**MCP tool surface.**

- `cyberos.time.list_entries(member_id?, since, until, status?)` — read.
- `cyberos.time.get_entry(id)` — read.
- `cyberos.time.create_entry(input)` — `destructive: true; requires_confirmation: true`.
- `cyberos.time.update_entry(id, patch)` — `destructive: true; requires_confirmation: true`.
- `cyberos.time.delete_entry(id, reason)` — `destructive: true; requires_confirmation: true`.
- `cyberos.time.accept_candidate(candidate_id, override_patch?)` — `destructive: true; requires_confirmation: true`.
- `cyberos.time.dismiss_candidate(candidate_id, reason)` — `destructive: false`.
- `cyberos.time.submit_week(member_id, week_start)` — `destructive: true; requires_confirmation: true; sensitivity: medium`.
- `cyberos.time.approve_entries(entry_ids)` — `destructive: true; requires_confirmation: true; sensitivity: high; step_up_required: true`.
- `cyberos.time.reject_entries(entry_ids, reason)` — `destructive: true; requires_confirmation: true; sensitivity: medium`.
- `cyberos.time.adjust_entry(id, field, new_value, reason)` — `destructive: true; requires_confirmation: true; sensitivity: high; step_up_required: true`.
- `cyberos.time.suggest_categorisation(description, engagement_id?)` — `destructive: false`; CUO suggestion path.
- `cyberos.time.weekly_summary(member_id, week_start)` — read.

CUO scope contracts: read + suggest allowed; commit-mutations forbidden.

## Alternatives Considered

- **Use Toggl / Harvest / Clockify via API.** Rejected: residency + the Engagement primitive linkage + CUO/persona consistency cannot be enforced via a hosted provider.
- **Auto-track every minute via desktop app.** Rejected (also Microsoft Recall pattern from PRD §6.5–6.7): event-driven > screen-observed. Calendar import + PROJ candidates are the principled source of "what happened".
- **Single submit / approve at month end.** Rejected: PRD §9.10 specifies weekly approval; monthly is too long a cycle for catching errors.
- **No auto-approval ever.** Rejected: too friction-heavy for the highest-frequency low-risk entries; the opt-in pattern with per-Member control is the floor.
- **No adjustment-after-approval path.** Rejected: rare-but-real edge cases (e.g. a billing dispute) need it; the audit trail makes it safe.

## Success Metrics

- **Primary metric.** P1 → P2 exit-gate progress: ≥ 80% of work hours captured weekly across the 10-employee team for 14 consecutive days; weekly submission rate ≥ 90%; HR/Ops Lead approval median latency ≤ 1 business day.
- **Friction metric.** Average time to log an entry ≤ 15 seconds via candidate-accept; ≤ 30 seconds via manual entry.
- **Latency NFR.** Day view p95 ≤ 600 ms; week view p95 ≤ 1.2 s.

## Scope

**In-scope.**
- The `time` schema with `entry`, `week_summary`, `adjustment`.
- Manual entry form + keyboard shortcuts + time-format parsing.
- PROJ-candidate consumption surface.
- Calendar import (Google Calendar + Microsoft 365 Calendar).
- CUO/COO categorisation suggestions.
- Weekly approval flow with submit / approve / reject / adjust.
- Per-Member auto-approval preference (default off).
- The Module-Federation remote at `/time` with day / week / approval views.
- The 13 MCP tools.
- INV-feed stub (the `time.week_summary` is what INV will consume in P2).
- Audit integration in scope `time.{tenant}`.

**Out-of-scope (deferred).**
- Leave / sabbatical workflows (FR-TIME-002).
- Expense capture (FR-TIME-003).
- Per-Member billable-rate overrides (P2 — INV-001 owns the rate logic).
- Mobile native (P3).
- Stopwatch-style start/stop timer (P2 — current entry pattern is duration-based, not stopwatch).
- Idle detection (P3).

## Dependencies

- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-002 / FR-AUTH-003 / FR-MCP-001 / FR-AI-001.
- FR-PROJ-001 (Engagement / Project / Cycle / Issue linkage).
- FR-PROJ-010 (`proj.time_candidate` source records).
- FR-DESIGN-001.
- FR-GENIE-001 / FR-GENIE-002 (weekly-reminder Notify cards; persona-scope; CUO categorisation).
- Google Calendar API + Microsoft 365 Graph API credentials.
- Compliance: PDPL Decree 13 (work-time data is personal data); EU AI Act Article 22 (the auto-approval path is automated decision-making; the per-Member opt-in + reversibility is the structural mitigation); Vietnamese labour law (working-hour caps + overtime recording — FR-HR-001 in P2 owns the labour-law surface; TIME provides the data feed).
- Locked decisions referenced: DEC-136 (weekly cadence), DEC-137 (PROJ candidate is the primary source for engineering work), DEC-138 (auto-approval is opt-in per Member).

## AI Risk Assessment

The categorisation suggestion is an AI surface; auto-approval is an automated decision over personal data (work hours). EU AI Act risk class: `limited`.

### Data Sources

Per-tenant only: TIME data + PROJ data + calendar event metadata + BRAIN context (recent Member patterns). No third-party.

### Human Oversight

- Categorisation is suggested; the Member confirms before commit.
- Auto-approval is per-Member opt-in; reversible by HR/Ops Lead via adjustment.
- Approval / rejection requires step-up auth on the high-sensitivity path.
- The weekly review surface is itself the human-oversight cadence.

### Failure Modes

- **Calendar event mis-categorised.** Mitigation: per-Member learning; the Member's correction feeds the next suggestion.
- **PROJ candidate over-counts.** State-transition-based duration may exceed actual work (a Member left an issue `in_progress` overnight). Mitigation: candidates are *suggestions*; the Member adjusts before accepting; CUO surfaces "this candidate is suspiciously long" warnings (> 8 hours single-stretch).
- **Auto-approval false-pass.** A wrongly-categorised entry slips through. Mitigation: opt-in default off; HR/Ops Lead post-hoc adjustment available; INV adjustment cycle handles billing corrections (P2).
- **Calendar API outage.** Manual entry continues; calendar import resumes on outage end.

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6.
- **Scope:** Drafted schema, capture flow, weekly approval, MCP tool surface, failure modes.
- **Human review:** `@stephen-cheng` reviewed; HR/Ops Lead role description for approval responsibility re-aligned at PR-review.
