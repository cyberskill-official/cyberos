---
title: "RES — frontend remote at /res (allocation Gantt, capacity heatmap, scenario simulator UI, Member workload)"
author: "@stephen-cheng"
department: design
status: ready_for_review
priority: p2
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: user_facing
eu_ai_act_risk_class: limited
target_release: "P2 / 2027-Q3"
client_visible: false
template: feature_request@1
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Ship the RES Module-Federation remote at `/res` consuming FR-RES-001 + FR-RES-002. Four primary surfaces: **allocation Gantt** (`/res/gantt`) showing Member rows × time columns with Engagement-coloured allocation bars + leave overlays + WIP indicators; **capacity heatmap** (`/res/heatmap`) — a forward-90-day team-wide availability view; **per-Member workload** (`/res/my`) showing the calling Member's current + upcoming allocations + leave + skill profile + suggested upskilling; **scenario simulator UI** (`/res/scenarios`) for HR/Ops Lead + Founder to draft + model staffing scenarios via FR-RES-002's simulator. Plus an admin **rebalancing inbox** (`/res/admin/rebalances`) for the weekly CUO/COO suggestions. PRD §9.18 names "allocation Gantt" explicitly; this FR ships it alongside the heatmap (which extends FR-TIME-002's primitive).

## Problem

The RES schema + AI suggestions are unusable without the Gantt + heatmap visualisation. Three failure modes:

- **Capacity invisibility.** A team-wide forward 90-day view in tabular form is unreadable; the heatmap + Gantt are the visualisation that makes the data scannable.
- **Scenario friction.** Authoring a hypothetical staffing scenario in raw GraphQL is over-engineered for HR/Ops Lead; the scenario simulator UI surfaces the form + the AI-modeled ranking.
- **Member opacity.** A Member without a workload view doesn't see why they're over-allocated until they're burned out.

## Proposed Solution

The shape of the answer is a Vite + React 19 Module-Federation remote at `/res`.

**Allocation Gantt (`/res/gantt`).**

For HR/Ops Lead + Founder + Engagement primary owner.

- **Layout.** Horizontal Gantt:
  - Rows: Members (filterable by team, role, skill).
  - Columns: weeks across a configurable horizon (default: next 12 weeks).
  - Cells: per-week allocation bars colour-coded by Engagement; bar height = allocated_pct.
- **Overlays.** Leave days (FR-TIME-002) shown as faded grey blocks; PROJ WIP load (FR-PROJ-003) shown as dot-density indicator.
- **Click an allocation.** Opens FR-RES-001's allocation detail in a side-drawer.
- **Drag-to-resize.** A planned allocation's date range can be dragged; the change drafts an `update` mutation pending sign-off.
- **"+ New allocation" CTA per Member-week.** Click an empty cell → opens the create-allocation form pre-populated.
- **Filters.** Members (multi-select), Engagements (multi-select), period range, allocation status.
- **Aggregate row.** Top of the Gantt shows "team-aggregate available capacity per week" + the pipeline-weighted demand from FR-RES-002's forecast.

**Capacity heatmap (`/res/heatmap`).**

For HR/Ops Lead + Founder + manager.

- **Layout.** Matrix:
  - Rows: Members.
  - Columns: days across a 90-day window.
  - Cell: colour-coded green (under-allocated) / yellow (near-capacity) / red (over-allocated) / grey (on leave).
- **Hover cell.** Tooltip with: baseline %, allocated %, PROJ WIP, available %, on-leave reason if applicable.
- **Click.** Opens the Member's workload view in a side-drawer.
- **Aggregate column.** Right side shows per-Member 90-day-average available %.
- **Aggregate row.** Bottom shows per-day team-wide available capacity.
- **Toggle.** "Show pipeline-weighted demand overlay" surfaces FR-RES-002's forecast as a dashed line on the aggregate row.

**Per-Member workload (`/res/my`).**

For every Member.

- **Header.** "Hi <preferred_name>. Your workload."
- **Current week summary.** Current allocations + leave + WIP indicators.
- **Forward 4-week timeline.** Personal mini-Gantt of the calling Member's allocations.
- **Skill profile card.**
  - Self-declared skills.
  - Peer-endorsed skills.
  - Council-assessed proficiency where applicable.
  - "+ Add skill" / "+ Endorse a teammate" CTAs.
- **Sabbatical accrual.** Read from FR-TIME-002 + FR-LEARN's accrual.
- **Upskilling suggestions.** When the skill-gap radar (FR-RES-002) identifies a gap that this Member could fill via training, a Notify card surfaces here with action options ("attend course X", "shadow Khoa on Acme architecture"). Informational — not a gate.
- **Pending allocation sign-offs.** Allocations awaiting the Member's countersign with full context + the engagement primary owner's notes.

**Scenario simulator UI (`/res/scenarios`).**

For HR/Ops Lead + Founder + Account Manager.

- **Scenarios list.** Active drafts + modelled + archived.
- **"+ Draft new scenario" CTA.**
  - Scenario name (e.g. "If we win Acme Q4 expansion").
  - Description.
  - Hypothetical roles needed: form rows of (role_kind × planned_pct × period). Add multiple rows.
  - Estimated start + end dates.
  - "Model this" action triggers FR-RES-002's `simulate_staffing` MCP tool.
- **Modelled scenario detail.**
  - The top-3 staffing combinations with rationale.
  - Per-Member per-role match score breakdown.
  - "Promote this scenario to actual allocations" CTA → opens a multi-step flow: each suggested allocation goes through FR-RES-001's sign chain.
- **Comparison view.** Multi-scenario side-by-side comparison: which scenario uses fewest over-allocated Members?

**Admin rebalancing inbox (`/res/admin/rebalances`).**

For HR/Ops Lead + Founder.

- Weekly CUO/COO rebalancing suggestions (FR-RES-002).
- Per-suggestion: over-allocated Member, options, suggested swaps with rationale.
- Actions: "Open Gantt to investigate", "Draft a rebalance allocation", "Dismiss with reason".
- History of past rebalances + acceptance metrics.

**Skill-search panel.**

A small search modal accessible from any RES surface: "Find Members with skill X at proficiency ≥ Y". Returns a ranked list with each Member's current allocation context.

**Vietnamese-locale rendering.**

- vi-VN default; column headers + tooltips localised.
- Date formatting per Vietnamese convention.

**Performance.**

- Initial JS bundle ≤ 50 KB gzipped.
- Gantt p95 ≤ 1 s for 10 Members × 12 weeks.
- Heatmap p95 ≤ 1.2 s for 10 Members × 90 days.
- Simulator-modelled scenario detail p95 ≤ 15 s (the underlying FR-RES-002 simulator is the dominant cost).

**MCP tool surface (read-only).**

- `cyberos.res.gantt_payload(since, until, member_ids?, engagement_ids?)` — read.
- `cyberos.res.my_workload_payload` — read; calling Member.
- `cyberos.res.scenario_payload(scenario_id?)` — read.

## Alternatives Considered

- **Skip the Gantt; show table only.** Rejected: visual time-series is the floor.
- **Use a hosted resource-planning UI (Float, Productive embed).** Rejected: residency + cross-module data integration.
- **Skip per-Member workload view.** Rejected: Members' self-direction relies on visibility.
- **AI-suggest skill endorsements.** Rejected: peer-endorsement is the trust mechanism; AI-suggested would erode it.

## Success Metrics

- **Primary metric.** P2 sprint demo passes: (1) Gantt renders for the 10-employee team across 12 weeks with all 2 Engagement allocations + leave overlays; (2) heatmap renders 10 × 90 days; (3) HR/Ops Lead authors a synthetic staffing scenario; the simulator returns the top-3 staffing options; (4) every Member has a `/res/my` view with their workload + skill profile.
- **Adoption metric.** Founder uses the heatmap weekly; HR/Ops Lead authors ≥ 2 scenarios/quarter; ≥ 80% of Members open `/res/my` monthly.
- **Latency NFR.** Per the budgets above; bundle ≤ 50 KB.

## Scope

**In-scope.**
- The Module-Federation remote at `/res` with the 4 primary surfaces + admin rebalancing inbox + skill-search panel.
- Drag-to-resize on Gantt allocations.
- Vietnamese-locale rendering.
- The 3 read-only MCP tools.
- Audit integration in scope `res.ui.{tenant}`.
- Mobile-responsive Gantt (compact view on small screens).

**Out-of-scope (deferred).**
- Mobile native (P3).
- Real-time pipeline-demand overlay update (P3 — daily refresh in P2).
- Drag-and-drop allocation creation (P3 — current pattern is form-based).
- Cross-team comparison dashboards (P3).

## Dependencies

- FR-RES-001 / FR-RES-002.
- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-002 / FR-MCP-001 / FR-AI-001.
- FR-DESIGN-001.
- FR-PROJ-005 / FR-CRM-002 (cross-module side-drawer integration).
- FR-TIME-002 (leave overlays).
- FR-LEARN-003 (skill profile).
- FR-GENIE-001 / FR-GENIE-002 (Notify cards).
- FR-OBS-001 / FR-OBS-002.
- Compliance: PDPL Decree 13; EU AI Act Article 50 (simulator + rebalancer surfaces inherit FR-RES-002's classification).
- Locked decisions referenced: DEC-244 (4-surface frontend layout), DEC-245 (allocation Gantt as the canonical visualisation), DEC-246 (drag-to-resize triggers a draft mutation, not auto-commit).

## AI Risk Assessment

The simulator + rebalancing surfaces inherit FR-RES-002's classification. Frontend itself is deterministic UI. EU AI Act risk class: `limited`.

### Data Sources

ACL-scoped GraphQL data; per-tenant residency.

### Human Oversight

All allocation mutations go through FR-RES-001's sign chain. AI surfaces (rebalancer, simulator) are Notify-mode + descriptive; the human commits.

### Failure Modes

- **Drag-to-resize race.** Mitigated by optimistic-mutation pattern reused from FR-PROJ-002.
- **Heatmap shows stale data.** Mitigated by capacity-snapshot recomputation on-demand from the UI.
- **Simulator output not actionable.** Mitigated by the human-feedback loop on suggestion acceptance.

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6.
- **Scope:** Drafted 4-surface frontend layout, Gantt + heatmap UX, scenario simulator UI, failure modes.
- **Human review:** `@stephen-cheng` reviewed; founder + HR/Ops Lead will validate the heatmap + Gantt during a P2 sprint walkthrough.
