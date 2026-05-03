---
title: "RES — CUO/COO rebalancing suggestions, project staffing simulator (read-only AI), capacity-vs-forecast alerts"
author: "@stephen-cheng"
department: operations
status: ready_for_review
priority: p2
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: user_facing
eu_ai_act_risk_class: limited
target_release: "P2 / 2027-Q3"
client_visible: false
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Layer the AI-native RES features on top of FR-RES-001. **CUO/COO rebalancing suggestions** — a weekly Notify cadence surfacing under-allocated + over-allocated Members + recommended rebalances ("Khoa is at 130% next week; consider shifting Alpha-1234 to Trang who has 20% capacity"); **project staffing simulator** — given a hypothetical engagement input (roles × pcts × periods), CUO/COO scores potential staffing combinations against capacity + skill-match + cost (rate-card) + LEARN-level appropriateness, returns a structured ranking with rationale + the `feasibility_summary_md` that populates `res.staffing_scenario.modelled_allocations`; **capacity-vs-forecast alerts** when the team's aggregate available capacity in the next 60 days drops below pipeline-weighted demand; **skill-gap radar** identifying patterns where multiple deals require skills the team lacks. All AI surfaces are **read-only suggestions** — never auto-assign; the human commits via the FR-RES-001 sign chain. Persona-scope contract forbids any allocation mutation.

## Problem

The schema + aggregation from FR-RES-001 produce data; without AI surfaces, the founder + HR/Ops Lead read the heatmap but the leverage point — turning observation into rebalanced action — requires manual cognition. Three failure modes:

- **Imbalanced workload silent.** A Member at 130% allocated for 4 weeks burns out before the manager notices; capacity-snapshot trends visible but un-surfaced.
- **Slow-staffing on new wins.** A CRM deal closes; the founder spends a day asking "who can take this on"; the AI surface answers in seconds with a structured ranking.
- **Skill-gap blind spot.** The pipeline shifts toward more design-heavy work but the team's design capacity hasn't grown; without a radar surface, the gap surfaces only when a deal falls through.

PRD §9.18: "CUO/COO-skill rebalancing suggestions" + PRD §14.3.1 P2 scope: "project staffing simulator" — this FR ships both.

## Proposed Solution

The shape of the answer is four AI surfaces — rebalancer, simulator, capacity-vs-forecast alert, skill-gap radar — all running through the AI Gateway via CUO/COO with persona-scope-contract-enforced retrieval against RES + cross-module data.

**1. Weekly rebalancing suggestions.**

A scheduled job at Sunday 18:00 ICT runs CUO/COO over the next 4 weeks of capacity:

- **Inputs.** `res.capacity_snapshot` for the next 28 days, `res.allocation`, `time.leave_request`, PROJ WIP signals, recent CRM deal-pipeline shifts.
- **Output.** A structured rebalancing report:
  - **Over-allocated Members.** Members at > 100% allocated for ≥ 5 of the next 28 days; rationale; suggested rebalances.
  - **Under-allocated Members.** Members at < 50% allocated for ≥ 7 of the next 28 days; suggestion options (other Engagements that could absorb them; KB authorship; learning-time; sabbatical-eligibility-trigger).
  - **Cross-Engagement adjustments.** Where Member X on Engagement A is over-allocated AND Member Y has skills A needs and is under-allocated, suggest a swap.
- **Surface.** Notify card to Founder + HR/Ops Lead + each affected Engagement primary owner. The card has actions: "Open RES heatmap", "Draft a swap proposal" (which opens FR-RES-001's create-allocation form pre-populated; the human decides + signs).
- **Latency.** Pre-computed; ≤ 12 s p95 on demand.

**2. Project staffing simulator.**

When a `res.staffing_scenario` is created (HR/Ops Lead or Founder authors a hypothetical: "Beta engagement, 60% capacity for 3 months, requires 1 Senior Engineer + 1 Mid-level Designer + 0.3 Account Manager"):

- **Inputs.** The hypothetical input + current capacity snapshot for the period + skill catalogue + LEARN levels + Engagement rate card + active allocations.
- **Compute.**
  - For each role-need: enumerate Members matching level + skill_tag minimum proficiency.
  - Score each candidate against: capacity availability in the period (higher = better), skill-match strength (more required tags + higher proficiency = better), cost relative to rate card (cheaper-than-rate = better, expensive-than-rate = worse).
  - Generate top-3 staffing combinations with feasibility summary.
- **Output.** Populates `res.staffing_scenario.modelled_allocations` with structured per-Member-per-role suggestions + rationale.
- **Narrative.** A 4-8 sentence Vietnamese-or-English summary: "Option 1: Khoa as Senior Engineer (90% available + 'rust' expert + currently at 30% allocation), Trang as Mid Designer (full-time available + 'design-systems' proficient), Stephen as Account Manager (founder discretionary). Feasibility: high; cost vs. rate-card: 5% under; risk: Khoa's other engagement Alpha extends 2 weeks into this period."
- **Latency.** ≤ 15 s p95.

**3. Capacity-vs-forecast alert.**

Daily 06:30 ICT job:

- **Inputs.** Aggregate available capacity in the next 60 days; pipeline-weighted demand from CRM deals (deal amount × deal stage probability × estimated capacity-need from the Engagement's expected scope).
- **Compute.** When forecast demand exceeds available capacity for ≥ 3 consecutive future weeks, raise an alert.
- **Surface.** Notify card to Founder ("Forecast: weeks 38-41 are oversold by 25% based on pipeline; consider hiring or scope-trimming open deals").
- **Severity.** sev-2 (informational); the founder decides hire-vs-scope-down.

**4. Skill-gap radar.**

Weekly job:

- **Inputs.** CRM deals (closed-won + in-pipeline) over the last 6 months; the Engagements' skill profile (extracted from FR-PROJ-007 contract metadata or Engagement description); team's current skill-tag distribution.
- **Compute.** Identify skills appearing in ≥ 30% of recent + pipeline Engagements where the team has 0 or only-1 Member at proficient+ level.
- **Surface.** Notify card to Founder + HR/Ops Lead: "Skill gap: 4 of last 6 deals reference 'AWS architecture'; team has only Khoa at proficient. Consider: hire / training-investment / formal certification for an existing Member."

**Persona scope contract.**

CUO/COO declares for the RES path:
- `tools_allowed`: `cyberos.res.list_*` (read), `cyberos.res.team_heatmap` (read), `cyberos.res.find_members_by_skill` (read), `cyberos.proj.list_engagements` (read), `cyberos.proj.engagement_dashboard` (read), `cyberos.crm.list_deals` (read), `cyberos.crm.list_accounts` (read), `cyberos.time.list_leave_requests` (read), `cyberos.learn.list_levels` (read), `cyberos.genie.notify` (notify), `cyberos.genie.draft_review` (review).
- `tools_forbidden_explicit`: `cyberos.res.create_allocation`, `cyberos.res.update_allocation`, `cyberos.res.cancel_allocation` — every allocation mutation goes through the human via FR-RES-001's sign chain.

**MCP tool surface (read-only).**

- `cyberos.res.weekly_rebalancing_suggestions` — read; HR/Ops + Founder.
- `cyberos.res.simulate_staffing(scenario_id)` — read; on-demand simulator.
- `cyberos.res.list_at_risk_capacity(weeks_ahead?)` — read; oversold-capacity alerts.
- `cyberos.res.skill_gap_report` — read.
- `cyberos.res.suggest_rebalances_for_member(member_id)` — read; per-Member targeted suggestions.

CUO uses these internally; the human commits any state change via FR-RES-001's mutation MCP tools.

**Audit + observability.**

- `res.ai.{tenant}` audit scope.
- OBS dashboard: rebalancing acceptance rate, simulator-recommendation acceptance, skill-gap-flag accuracy.
- Per-suggestion Member feedback ("useful" / "partially useful" / "not useful") feeds persona-quality dashboard (FR-GENIE-002).

## Alternatives Considered

- **Auto-allocate based on simulator output.** Rejected: PRD §6.4 + §2.5 — human-in-the-loop floor on resource decisions affecting work + compensation-adjacent outcomes.
- **Skip the simulator; let the founder decide manually.** Rejected: the simulator is the leverage point for fast deal-acceptance.
- **Skill-gap radar based purely on Member self-declared tags.** Rejected: peer-endorsement + council-assessed proficiency is the higher-signal source; the skill-gap radar uses both.

## Success Metrics

- **Primary metric.** P2 → P3 exit-gate progress: weekly rebalancing suggestion acceptance ≥ 35% (founder or HR/Ops Lead actions on the card); simulator-recommendation acceptance ≥ 50% on the first staffing scenario for a closed-won deal; skill-gap-radar flag accuracy ≥ 70% (the flagged gap really exists, validated post-hoc).
- **Founder-cognitive-load metric.** Time-to-staffing-decision on a closed-won deal reduced by ≥ 70% vs. pre-simulator baseline.
- **Latency NFR.** Per the budgets above.

## Scope

**In-scope.**
- The 4 AI surfaces.
- Persona-scope contract for CUO/COO with explicit forbid on allocation mutations.
- Member-feedback loop on suggestions.
- The 5 read MCP tools.
- Cross-module retrieval against RES + PROJ + CRM + TIME + LEARN.
- Audit integration in scope `res.ai.{tenant}`.
- OBS dashboard panels.

**Out-of-scope (deferred to FR-RES-003).**
- Frontend remote at /res (FR-RES-003).
- Auto-rebalance suggestion application (P3 — even with confirmation; the per-allocation sign chain remains the floor).
- Cross-tenant resource pool sharing (forbidden).
- Real-time pipeline-weighted demand (P3 — daily refresh is the floor).

## Dependencies

- FR-RES-001.
- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-002 / FR-MCP-001 / FR-AI-001.
- FR-PROJ-001 / FR-PROJ-007.
- FR-CRM-001 / FR-CRM-003.
- FR-TIME-002.
- FR-LEARN-003 (skill catalogue + level definitions).
- FR-INV-001 (Engagement rate-card).
- FR-GENIE-001 / FR-GENIE-002 (CUO/COO persona; Notify mode; persona-scope).
- FR-OBS-001 / FR-OBS-002.
- Compliance: EU AI Act Article 22 (no automated decisions on allocation; AI suggests, human commits); Article 50 (transparency disclosure on suggestions); GDPR.
- Locked decisions referenced: DEC-241 (4 AI surfaces; rebalancer + simulator + capacity-forecast + skill-gap), DEC-242 (allocation mutations forbidden in CUO scope), DEC-243 (weekly rebalancing cadence Sunday 18:00 ICT).

## AI Risk Assessment

The 4 AI surfaces shape staffing + hiring + capacity decisions. EU AI Act risk class: `limited` (informational; no automated allocation; human commits via sign chain).

### Data Sources

Per-tenant only: RES + PROJ + CRM + TIME + LEARN + INV. CUO/COO runs through the AI Gateway with persona-stamping. No third-party data; no compensation values in scope (rate-card pulls + Engagement budget but never per-Member salary).

### Human Oversight

- All suggestions are Notify-mode; human dismisses or actions.
- Allocation mutations go through FR-RES-001's sign chain (engagement-owner + member + hr-ops).
- Simulator-recommended staffings populate the scenario but don't create allocations.
- The kill-switch from FR-GENIE-002 silences all RES AI surfaces.

### Failure Modes

- **Wrong rebalancing suggestion** (Member not actually over-allocated; capacity-snapshot stale). Mitigation: snapshot recomputed nightly; on-demand recompute available; the human verifies before acting.
- **Simulator picks Member with skill-mismatch.** Mitigation: skill-tag scoring + LEARN-level scoring; the human reviews the rationale + can re-prioritise.
- **Skill-gap radar false-positive on rare-but-real skill.** Mitigation: 30%-of-deals threshold; manual review.
- **Forecast over-counts pipeline.** Pipeline-stage probability bands (low/medium/high from CRM-003) feed weighted demand; the founder reviews assumptions.

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6.
- **Scope:** Drafted 4-surface architecture, persona scope contract, MCP tool surface, failure modes.
- **Human review:** `@stephen-cheng` reviewed; the founder + HR/Ops Lead will validate the first 4 weeks of rebalancing suggestions before declaring P2 → P3 readiness.
