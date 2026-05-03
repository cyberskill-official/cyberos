---
title: "TIME — leave + sabbatical accrual (Total Rewards Appendix scaffold), capacity heat map, holiday calendar"
author: "@stephen-cheng"
department: human_resources
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

Ship the leave + sabbatical + capacity surfaces of TIME: **leave types** (PTO, sick, parental, bereavement, public holiday, sabbatical, unpaid) with per-tenant rules; **leave request flow** (Member submits → HR/Ops Lead approves → calendar updates → PROJ capacity reduces for affected cycles → other Members are notified for blocking dependencies); **sabbatical accrual scaffold** anchored to CyberSkill's Total Rewards Appendix (sabbaticals every five continuous years; the LEARN module in P2 owns the eligibility-tracking; this FR ships the data plumbing); **capacity heat map** showing per-Member availability across the next 90 days with leave + holiday + WIP overlays; **Vietnamese public-holiday calendar** preloaded (Tết / Reunification Day / Independence Day / Lunar New Year / etc.) with per-tenant override; **per-region holiday calendars** for international Members in P3+. The module captures the "who's available when" data that capacity planning (FR-PROJ-004), staffing simulation (FR-RES-001 in P2), and customer-commitment-confidence rely on.

## Problem

The team's current leave management is a Google Calendar shared event + a Slack message + an Excel timesheet for HR records. Three failure modes:

- **Leave invisible to PROJ.** A Member takes 5 days off; PROJ capacity (FR-PROJ-004) doesn't know; the cycle plan accidentally commits to work the Member cannot deliver.
- **Sabbatical accrual untracked.** CyberSkill's Total Rewards Appendix (PRD §1.1, §2.3 Bet 5) commits to sabbaticals every five continuous years. Without structured tracking, the obligation drifts; eligibility surprises both the company and the Member.
- **Holiday calendar silent.** Members spread across Vietnamese + (eventually) international locations; without a shared holiday calendar, cross-team scheduling fails on Tết and Lunar New Year.

The PRD §9.10 commits: "Time entries by Member by Engagement / Task; expense capture; weekly approval flow; feeds INV." Leave is the missing piece — it's the *non-billable, non-work* time that capacity must account for.

## Proposed Solution

The shape of the answer is `time` schema extensions, the leave-request workflow, the sabbatical-accrual scaffold consumed by LEARN P2, the capacity heat map view, and the holiday-calendar primitive.

**Schema extensions.**

```sql
-- Leave types per tenant (configurable; seeded with defaults).
CREATE TABLE time.leave_type (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  code TEXT NOT NULL,                          -- "pto" | "sick" | "parental" | "bereavement"
                                               -- | "public_holiday" | "sabbatical" | "unpaid"
                                               -- | "compassionate" | "training_external"
  display_name TEXT NOT NULL,
  is_paid BOOLEAN NOT NULL,
  counts_against_pto_balance BOOLEAN NOT NULL,
  default_annual_balance_days REAL,            -- e.g. 12 for PTO; 0 for unpaid
  requires_advance_notice_days INT,            -- e.g. 14 for PTO planned; 0 for sick
  requires_doctor_note_after_days INT,
  consumes_sabbatical_accrual BOOLEAN NOT NULL DEFAULT false,
  approval_required_above_days REAL,           -- e.g. 0.5; below threshold can auto-approve
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  archived_at TIMESTAMPTZ,
  UNIQUE (tenant_id, code)
);

-- Per-Member balance per leave-type per year.
CREATE TABLE time.leave_balance (
  tenant_id UUID NOT NULL,
  member_id UUID NOT NULL,
  leave_type_id UUID NOT NULL REFERENCES time.leave_type(id),
  year INT NOT NULL,
  starting_balance_days REAL NOT NULL,
  accrued_days REAL NOT NULL DEFAULT 0,
  used_days REAL NOT NULL DEFAULT 0,
  carryover_in REAL NOT NULL DEFAULT 0,        -- carried in from previous year
  carryover_out REAL,                           -- carried out at year end (computed)
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  PRIMARY KEY (tenant_id, member_id, leave_type_id, year)
);

-- Leave requests.
CREATE TABLE time.leave_request (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  member_id UUID NOT NULL,
  leave_type_id UUID NOT NULL REFERENCES time.leave_type(id),
  starts_at_date DATE NOT NULL,
  starts_at_half_day_kind TEXT,                -- null | "morning" | "afternoon"
  ends_at_date DATE NOT NULL,
  ends_at_half_day_kind TEXT,                  -- null | "morning" | "afternoon"
  total_days REAL NOT NULL,                     -- computed accounting for half-days + weekends + public holidays
  reason_md TEXT,
  status TEXT NOT NULL DEFAULT 'submitted',     -- "submitted" | "approved" | "rejected" | "cancelled"
  submitted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  approved_by UUID,
  approved_at TIMESTAMPTZ,
  rejection_reason TEXT,
  doctor_note_attachment_id UUID,               -- references the EMAIL/KB blob store when applicable
  affected_cycles UUID[],                       -- proj.cycle IDs whose capacity reduces
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX leave_request_member_idx ON time.leave_request (tenant_id, member_id, starts_at_date);
CREATE INDEX leave_request_status_idx ON time.leave_request (tenant_id, status);

-- Holiday calendar (per-tenant; per-region in P3+).
CREATE TABLE time.holiday (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  region TEXT NOT NULL DEFAULT 'VN',           -- "VN" | "US-CA" | "EU-DE" | etc.
  name TEXT NOT NULL,
  observed_on DATE NOT NULL,
  is_paid BOOLEAN NOT NULL,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  UNIQUE (tenant_id, region, observed_on, name)
);

-- Sabbatical accrual ledger (consumed by LEARN P2 for eligibility tracking).
CREATE TABLE time.sabbatical_accrual (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  member_id UUID NOT NULL,
  accrual_period_start DATE NOT NULL,           -- typically Member's start date or last sabbatical end
  accrual_period_end DATE,                       -- null while active; populated when sabbatical taken
  accrued_continuous_years REAL NOT NULL,       -- as-of-now value
  next_eligibility_at DATE,                      -- when the next sabbatical accrues
  last_recalculated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  PRIMARY KEY (tenant_id, member_id, accrual_period_start)
);
```

**Default seed (CyberSkill tenant).**

Leave types pre-populated:
- `pto` — 12 days/year; advance notice 14 days; approval above 0.5 days.
- `sick` — 60 days/year (Vietnamese-labour-law floor; capped at country-statute); doctor note after 3 days.
- `parental` — 180 days for primary caregiver; 14 days for secondary (Vietnamese-labour-law).
- `bereavement` — 5 days/year.
- `public_holiday` — auto-populated from holiday calendar; not deducted from balance.
- `sabbatical` — accrued every 5 continuous years per Total Rewards Appendix; 30 days; consumes_sabbatical_accrual: true.
- `unpaid` — no balance cap.
- `compassionate` — 5 days/year for unforeseen family circumstances.

Vietnamese public holidays seeded from a curated list (the official 8 + Tết multi-day window): New Year's Day, Tết Holiday (4–5 days depending on year), Hùng Kings' Festival, Reunification Day, Labour Day, Independence Day, plus the floating Lunar New Year days computed via the lunar calendar library.

**Leave request flow.**

1. Member opens `/time/leave/new`; picks leave type, dates, half-day options, reason.
2. Server-side: computes `total_days` accounting for weekends + public holidays + half-day kinds; checks balance + advance-notice + doctor-note-required rules.
3. If validation passes and `total_days <= leave_type.approval_required_above_days`, the request **auto-approves** (e.g. half-day PTO).
4. Otherwise: `status: 'submitted'`; HR/Ops Lead receives a Notify card.
5. HR/Ops Lead approves / rejects with comment. On approval:
   - Member's `time.leave_balance` updates.
   - `affected_cycles[]` populated by walking the Member's PROJ cycles overlapping the leave window.
   - For each affected cycle, the capacity (FR-PROJ-004) reduces; the capacity recompute job runs.
   - The Member's calendar (Google / Microsoft 365 if integrated) gets a "Out of office" event.
   - Other Members + Project Leads on affected projects receive a Notify card explaining the capacity change.
6. Approved + cancellation: a Member can cancel an approved leave with HR/Ops Lead acknowledgement; balance restores.

**Sabbatical accrual scaffold.**

- A nightly job recomputes `time.sabbatical_accrual.accrued_continuous_years` per Member using their start date + any prior approved sabbaticals + any periods that interrupt continuity (extended unpaid leave > 90 days breaks continuity; configurable).
- When a Member crosses 5.0 continuous years, the LEARN module's promotion / next-step engine (P2 FR-LEARN-001) surfaces a "Sabbatical eligible" Notify to the Member and HR/Ops Lead.
- The actual sabbatical request goes through the leave-request flow with `leave_type: sabbatical`.

This FR ships the *plumbing* (the accrual ledger + the recompute job + the data feed). The *eligibility surface + UX* live in LEARN P2.

**Capacity heat map.**

A `/time/capacity` view (HR/Ops Lead, Project Lead, Founder/CEO):

- Rows: Members (or per-engagement filter).
- Columns: next 90 days (per-day cells).
- Cell colour: green (full availability) → yellow (50% — partial leave / heavy WIP) → red (unavailable / on leave).
- Overlays: PROJ WIP indicator (FR-PROJ-003); approved leave; pending leave (faint); public holidays; sabbatical projection.
- Hover: tooltip with details (which engagement / which leave / which holiday).

The heat map is read-only; Member-level editing of capacity (planned working days) is a per-Member preference set in `/auth/account/working-pattern`.

**Per-Member working pattern.**

A simple primitive: which weekdays the Member typically works (default Mon-Fri); standard daily hours (default 8); overrides for irregular schedules (e.g. a Member working 4 long days). This feeds the capacity heat map and PROJ capacity.

**Holiday calendar UX.**

`/time/holidays` shows the full calendar; HR/Ops Lead can add custom tenant-specific holidays (e.g. company-wide closures); per-region overlays for international Members; iCal export available (so Members can subscribe in their personal calendar app).

**MCP tool surface.**

- `cyberos.time.list_leave_balances(member_id?, year?)` — read.
- `cyberos.time.list_leave_requests(member_id?, status?, since)` — read.
- `cyberos.time.create_leave_request(input)` — `destructive: true; requires_confirmation: true`.
- `cyberos.time.approve_leave_request(id, comment?)` — `destructive: true; requires_confirmation: true; sensitivity: high; step_up_required: true`.
- `cyberos.time.reject_leave_request(id, reason)` — `destructive: true; requires_confirmation: true; sensitivity: medium`.
- `cyberos.time.cancel_leave_request(id, reason)` — `destructive: true; requires_confirmation: true`.
- `cyberos.time.list_holidays(region?, year?)` — read.
- `cyberos.time.create_holiday(region, name, observed_on)` — `destructive: true; requires_confirmation: true`.
- `cyberos.time.capacity_heat_map(start_date, end_date, member_ids?)` — read.
- `cyberos.time.sabbatical_eligibility(member_id?)` — read.

CUO scope contracts: read all; commit-mutations on leave (approve/reject) restricted to HR/Ops Lead's role + step-up.

## Alternatives Considered

- **Use Google Workspace's built-in leave / calendar.** Rejected: Google has no policy primitives (per-leave-type rules, accrual ledgers, sabbatical tracking); the Total Rewards Appendix is not modelled.
- **Skip sabbatical scaffold; build later.** Rejected: the accrual ledger needs continuous data from day one; bolting on later means reconstructing history.
- **Auto-approve all leave under N days.** Rejected: Vietnamese-labour-law treats leave types differently; auto-approval needs per-leave-type rule (the `approval_required_above_days` field).
- **Single calendar for all Members regardless of region.** Rejected: at P1 only Vietnam matters but the per-region table is forward-compatible for P3+ international hires.

## Success Metrics

- **Primary metric.** P1 → P2 exit-gate: 100% of team leave + holidays captured in TIME for 21 consecutive days; capacity heat map renders correctly for every active Member; the affected-cycles wiring produces a capacity adjustment in PROJ within 5 s of leave approval.
- **Sabbatical scaffold.** The accrual ledger is correct for every Member as-of-now (verified manually against employee records by HR/Ops Lead).
- **Latency NFR.** Capacity heat map p95 ≤ 1 s for a 10-Member × 90-day window.

## Scope

**In-scope.**
- `time.leave_type`, `time.leave_balance`, `time.leave_request`, `time.holiday`, `time.sabbatical_accrual` tables.
- Default seed (8 leave types + Vietnamese public holidays).
- Leave-request flow with auto-approval rules.
- Sabbatical accrual nightly recompute.
- Capacity heat map view.
- Per-Member working pattern.
- Holiday calendar UX + iCal export.
- The 10 MCP tools.
- Audit integration in scope `time.leave.{tenant}`.
- Cross-module integration: PROJ capacity recompute trigger; Member's external calendar update on approval.

**Out-of-scope (deferred).**
- Per-Member statutory deduction calculations (P2 — REW-001 owns Vietnamese SI/PIT mechanics).
- Leave forecasting (predict who's likely to request leave) — P3 if useful.
- Employee handbook integration (P2 — HR-001 module).
- Maternity / paternity claim filing automation with Vietnamese authorities — P3.

## Dependencies

- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-002 / FR-AUTH-003 / FR-MCP-001.
- FR-TIME-001 (the parent module).
- FR-PROJ-004 (capacity recompute trigger).
- FR-DESIGN-001.
- FR-GENIE-001 / FR-GENIE-002 (Notify cards; HR/Ops Lead approval surface).
- A Vietnamese lunar-calendar library (`lunar-vietnam` or equivalent).
- Compliance: Vietnamese Labour Code (Articles on PTO, sick, parental); the `time.leave_type` rules encode the statutory floor; per-tenant overrides cannot reduce below floor.
- Locked decisions referenced: DEC-139 (8 default leave types), DEC-140 (sabbatical accrual scaffold lives in TIME; surface lives in LEARN P2), DEC-141 (auto-approval rule per leave-type via `approval_required_above_days`).

## AI Risk Assessment

Not applicable. `eu_ai_act_risk_class: not_ai`. Leave management is deterministic policy; no AI inference in the path. (CUO-driven leave-pattern insight is a P3 surface and would be classified `limited` at that point.)

## AI Authorship Disclosure

- **Tools used:** Claude Cowork (Anthropic).
- **Scope:** drafted the FR end-to-end against the PRD + SRS; founder reviews and edits before status changes from `ready_for_review`.
- **Human review:** founder (`@stephen-cheng`) — final wording is the founder's responsibility.
