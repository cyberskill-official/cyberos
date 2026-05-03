---
title: "RES — schema (allocation, capacity forecast, staffing scenarios) + Apollo subgraph + cross-module signal aggregation"
author: "@stephen-cheng"
department: operations
status: ready_for_review
priority: p2
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: infrastructure
eu_ai_act_risk_class: not_ai
target_release: "P2 / 2027-Q3"
client_visible: false
template: feature_request@1
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Stand up the RES (Resource Planning) module's schema and Apollo Federation subgraph. Four primitives: **Allocation** (the per-Member-per-Engagement-per-period planned-vs-actual capacity), **CapacityForecast** (per-Member rolling-90-day projected availability accounting for FR-TIME-002 leave + FR-PROJ-004 cycle WIP + FR-LEARN-003 level + skill tags), **StaffingScenario** (a what-if simulation: "if we win the Beta engagement at 60% capacity for 3 months, who do we staff?"), and **SkillTag** (the per-Member competency tags from FR-LEARN-003 + Member self-declared expertise from `hr.profile_self.expertise_tags`). Cross-module signal aggregation: pulls TIME-002 leave + working pattern, PROJ-004 cycle capacity, LEARN-003 level + competencies, INV-001 Engagement-budget burn signals, CRM-001 deal pipeline (for forward-looking demand). Lives in a new `res` schema (non-secret). Subsequent batch-08 FRs ship the rebalancer + simulator (FR-RES-002) and the frontend (FR-RES-003).

## Problem

CyberSkill's two long-term engagements + the founder's mental model of "who has bandwidth" works at 10 employees; at 15-30 it breaks. PRD §9.18 names "Capacity vs forecast; allocation Gantt; CUO/COO-skill rebalancing suggestions"; PRD §14.3.1 P2 scope: "RES module — resource planning, capacity-vs-forecast rebalancer, project staffing simulator." Three failure modes the platform must structurally avoid:

- **Capacity over-commitment.** A new Engagement is signed without a structured availability check; the Member assigned discovers two weeks in that they're double-booked. Calibration drift compounds.
- **Skill-mismatch staffing.** A Member assigned to an Engagement they don't have the right level / skill for; rework + re-staffing cost.
- **Pipeline blindness.** A CRM deal at "negotiation" stage with high probability is invisible to capacity planning until it closes; by then the team has filled the slot with other work.

## Proposed Solution

The shape of the answer is `res.*` schema + Apollo subgraph + cross-module signal aggregation pipeline.

**Schema (`res` — non-secret).**

```sql
CREATE SCHEMA res;

-- Per-Member-per-Engagement-per-period allocation.
CREATE TABLE res.allocation (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  employee_id UUID NOT NULL REFERENCES hr.employee(id) ON DELETE RESTRICT,
  engagement_id UUID NOT NULL REFERENCES proj.engagement(id) ON DELETE RESTRICT,
  starts_at_date DATE NOT NULL,
  ends_at_date DATE NOT NULL,
  planned_pct REAL NOT NULL,                                          -- 0.0 to 1.0; the planned capacity allocation
                                                                      -- e.g. 0.5 = half-time
  planned_role_kind TEXT,                                              -- "engineering" | "design" | etc. — the role planned for
  planned_level TEXT,                                                  -- "L2" | "Senior" | etc.
  status TEXT NOT NULL DEFAULT 'planned',                              -- "planned" | "active" | "completed" | "cancelled"
  signed_off_by_engagement_owner_at TIMESTAMPTZ,
  signed_off_by_member_at TIMESTAMPTZ,                                  -- Member acceptance is the floor
  signed_off_by_hr_ops_at TIMESTAMPTZ,                                  -- HR/Ops Lead final sign for cross-engagement view
  notes_md TEXT,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX allocation_employee_period_idx ON res.allocation (tenant_id, employee_id, starts_at_date, ends_at_date);
CREATE INDEX allocation_engagement_idx       ON res.allocation (tenant_id, engagement_id, status);

-- Per-Member rolling capacity snapshot (recomputed nightly).
CREATE TABLE res.capacity_snapshot (
  tenant_id UUID NOT NULL,
  employee_id UUID NOT NULL REFERENCES hr.employee(id) ON DELETE CASCADE,
  date DATE NOT NULL,
  baseline_pct REAL NOT NULL,                                           -- working pattern × non-leave-day factor
  allocated_pct REAL NOT NULL,                                          -- sum of res.allocation.planned_pct for this date
  proj_wip_load REAL,                                                   -- 0.0 to 1.0; estimated PROJ load from FR-PROJ-003 WIP
  on_leave BOOLEAN NOT NULL DEFAULT false,
  available_pct REAL NOT NULL,                                          -- max(0, baseline - allocated - proj_wip_load)
                                                                      -- (or 0 if on_leave)
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  PRIMARY KEY (tenant_id, employee_id, date)
);

CREATE INDEX capacity_snapshot_date_idx ON res.capacity_snapshot (tenant_id, date);

-- Staffing scenario.
CREATE TABLE res.staffing_scenario (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  name TEXT NOT NULL,                                                  -- "If we win the Acme Q4 expansion"
  description_md TEXT,
  hypothetical_engagement_input JSONB NOT NULL,                        -- structured: roles needed × periods × pcts
  status TEXT NOT NULL DEFAULT 'draft',                                 -- "draft" | "evaluating" | "modelled" | "archived"
  modelled_allocations JSONB,                                           -- the simulator's suggested per-Member assignment
  feasibility_summary_md TEXT,                                          -- "feasible at 80% with Member X taking lead"
  created_by UUID NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  modelled_at TIMESTAMPTZ,
  archived_at TIMESTAMPTZ,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);

-- Skill tags (catalog + per-Member assignment).
CREATE TABLE res.skill_tag (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  tag TEXT NOT NULL,                                                   -- "rust", "design-systems", "vietnamese-tax", "client-comms"
  category TEXT NOT NULL,                                              -- "technical" | "domain" | "soft_skill" | "language"
  description_md TEXT,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  UNIQUE (tenant_id, tag)
);

CREATE TABLE res.member_skill (
  tenant_id UUID NOT NULL,
  employee_id UUID NOT NULL REFERENCES hr.employee(id) ON DELETE CASCADE,
  skill_tag_id UUID NOT NULL REFERENCES res.skill_tag(id) ON DELETE CASCADE,
  proficiency TEXT NOT NULL,                                           -- "exposure" | "competent" | "proficient" | "expert"
  source TEXT NOT NULL,                                                -- "self_declared" | "peer_endorsed" | "council_assessed"
  endorsed_by UUID,
  endorsed_at TIMESTAMPTZ,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  PRIMARY KEY (tenant_id, employee_id, skill_tag_id)
);
```

**Cross-module signal aggregation.**

The `res-capacity-aggregator` service runs nightly + on-demand:

1. **TIME-002 leave** — reads `time.leave_request{status: approved}`; on-leave-day Members have `on_leave: true` in their snapshot.
2. **TIME-002 working pattern** — reads per-Member working-pattern (e.g. Mon-Fri 8h vs. 4-day-week); produces baseline_pct per date.
3. **PROJ-003 WIP** — reads `proj.issue` count by assignee + state; estimates per-Member PROJ load (heuristic: WIP cap reached → proj_wip_load = 1.0; below cap → linear).
4. **PROJ-007 Engagement → res.allocation** — when a Member is allocated to an Engagement at planned_pct, that pct counts toward `allocated_pct`.
5. **LEARN-003 level + skills** — joined on Member-detail queries for matching to Engagement role-needs.
6. **CRM-001 deal pipeline** — pipeline-position deals with high probability surface as `forecast_demand` (informational; FR-RES-002 simulator consumes).

**RLS + ACL.**

- `res.allocation`: HR/Ops Lead + Founder + Engagement primary owner read; the assigned Member sees their own; sign chain enforced at mutation level.
- `res.capacity_snapshot`: HR/Ops + Founder + manager + the Member sees their own.
- `res.staffing_scenario`: HR/Ops + Founder + Account Manager (the typical scenario authors).
- `res.skill_tag` / `res.member_skill`: tenant-readable; mutations restricted to HR/Ops + Founder + the Member self-declaring (with peer-endorsement audit trail).

**GraphQL subgraph.**

```graphql
type Query {
  resAllocations(employeeId: ID, engagementId: ID, since: Date, until: Date): [ResAllocation!]!
  resAllocation(id: ID!): ResAllocation
  resCapacitySnapshot(employeeId: ID!, since: Date!, until: Date!): [ResCapacitySnapshot!]!
  resTeamCapacityHeatMap(since: Date!, until: Date!, memberIds: [ID!]): ResCapacityHeatMap!
  resStaffingScenarios(status: String): [ResStaffingScenario!]!
  resStaffingScenario(id: ID!): ResStaffingScenario
  resSkillTags(category: String): [ResSkillTag!]!
  resMembersBySkill(skillTag: String!, minProficiency: String): [HrEmployee!]!
  resMyCapacity(since: Date, until: Date): [ResCapacitySnapshot!]!
}

type Mutation {
  resCreateAllocation(input: ResAllocationInput!): ResAllocation!
  resUpdateAllocation(id: ID!, patch: ResAllocationPatch!): ResAllocation!
  resSignOffAllocation(id: ID!, signerKind: String!): ResAllocation!  # "engagement_owner" | "member" | "hr_ops"
  resCancelAllocation(id: ID!, reason: String!): ResAllocation!
  resDraftStaffingScenario(input: ResStaffingScenarioInput!): ResStaffingScenario!
  resModelStaffingScenario(id: ID!): ResStaffingScenario!  # FR-RES-002 simulator
  resCreateSkillTag(input: ResSkillTagInput!): ResSkillTag!
  resAssignMemberSkill(input: ResMemberSkillInput!): ResMemberSkill!
}
```

Persisted-queries discipline applies.

**MCP tool surface.**

Read tools (everyone with appropriate ACL):

- `cyberos.res.list_my_allocations(since?, until?)` — read; calling Member.
- `cyberos.res.my_capacity(since?, until?)` — read.
- `cyberos.res.team_heatmap(since, until, member_ids?)` — read; HR/Ops + Founder + manager.
- `cyberos.res.list_engagement_allocations(engagement_id)` — read; Engagement primary owner + HR/Ops + Founder.
- `cyberos.res.list_skill_tags(category?)` — read.
- `cyberos.res.find_members_by_skill(skill_tag, min_proficiency?)` — read.
- `cyberos.res.list_staffing_scenarios(status?)` — read.

Mutation tools:

- `cyberos.res.create_allocation(input)` — `destructive: true; requires_confirmation: true; sensitivity: medium` (commits Member time).
- `cyberos.res.sign_off_allocation(id, signer_kind)` — `destructive: false`.
- `cyberos.res.draft_scenario(input)` — `destructive: false; idempotent: true` (drafts only; not committed allocations).
- `cyberos.res.assign_self_skill(skill_tag, proficiency)` — `destructive: false`; per-Member self-service.
- `cyberos.res.endorse_member_skill(member_id, skill_tag, proficiency)` — `destructive: false`; peer-endorsement.
- `cyberos.res.cancel_allocation(id, reason)` — `destructive: true; requires_confirmation: true`.

CUO scope contract: read all + scenario-draft + draft suggestions allowed; allocation commits restricted (HR/Ops + Founder + Engagement-owner sign chain at mutation level).

**Audit integration.** `res.{tenant}` audit scope.

## Alternatives Considered

- **Use a hosted resource-planning tool (Float, Resource Guru, Productive).** Rejected: residency + cross-module integration with PROJ + TIME + CRM + LEARN cannot be enforced hosted.
- **Skip the staffing scenario primitive.** Rejected: PRD §14.3.1 explicitly names "project staffing simulator"; scenarios are the substrate.
- **Per-day allocation precision.** Considered. The schema supports per-day via `starts_at_date` + `ends_at_date`; the snapshot is per-day; finer-grained (per-hour) is over-engineered for our scale.
- **AI-driven auto-assignment.** Rejected: PRD §6.4 + §2.5 — AI suggests; humans assign. The persona-scope contract forbids `create_allocation` for CUO.

## Success Metrics

- **Primary metric.** P2 sprint demo passes: (1) HR/Ops Lead creates allocations for the 10 employees across the 2 active Engagements; (2) per-Member capacity snapshot computes correctly accounting for leave + WIP; (3) the team heatmap renders for a 90-day forward window; (4) skill-tag-based search ("find Members with `rust` ≥ proficient") returns expected results.
- **Adoption metric.** ≥ 90% of allocated Member-Engagement pairs have a signed allocation by P2 → P3 exit; ≥ 80% of Members have ≥ 5 self-declared skills.
- **Latency NFR.** Team heatmap p95 ≤ 800 ms over a 10-Member × 90-day window.

## Scope

**In-scope.**
- The 5 schema additions (`allocation`, `capacity_snapshot`, `staffing_scenario`, `skill_tag`, `member_skill`).
- Cross-module signal aggregation pipeline (TIME + PROJ + LEARN + CRM).
- Allocation sign-off chain (engagement_owner + member + hr_ops).
- The 7 read MCP tools + 6 mutation MCP tools.
- Apollo Federation v2 subgraph with all queries + mutations.
- Audit integration in scope `res.{tenant}`.

**Out-of-scope (deferred to FR-RES-002 / FR-RES-003).**
- CUO/COO rebalancing suggestions + simulator (FR-RES-002).
- Frontend remote at /res (FR-RES-003).
- Multi-tenant resource-pool federation (forbidden).
- Auto-assignment based on skill-match (P3 — informational only in P2).
- Hours-precision allocation (P3).

## Dependencies

- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-002 / FR-MCP-001.
- FR-HR-001 / FR-HR-003 (Member + skill-self-profile).
- FR-TIME-001 / FR-TIME-002 (capacity snapshot + leave signals).
- FR-PROJ-001 / FR-PROJ-003 / FR-PROJ-007 (Engagement + WIP signals).
- FR-LEARN-003 (level + competency framework).
- FR-CRM-001 (deal pipeline forecast signals).
- FR-INV-001 (Engagement-budget signals).
- Compliance: PDPL Decree 13 (Member capacity data is personal).
- Locked decisions referenced: DEC-238 (Engagement-owner + Member + HR/Ops sign chain on allocations), DEC-239 (skill-tag self-declaration with peer-endorsement; not council-only), DEC-240 (per-day capacity precision; not per-hour).

## AI Risk Assessment

Not applicable. `eu_ai_act_risk_class: not_ai`. The schema + aggregation are deterministic. AI surfaces (CUO/COO rebalancer + simulator) ship in FR-RES-002.

## AI Authorship Disclosure

- **Tools used:** Claude Cowork (Anthropic).
- **Scope:** drafted the FR end-to-end against the PRD + SRS; founder reviews and edits before status changes from `ready_for_review`.
- **Human review:** founder (`@stephen-cheng`) — final wording is the founder's responsibility.
