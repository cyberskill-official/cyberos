---
title: "OKR — schema (Objective + Key Result + cycle + cascade hierarchy) + Apollo subgraph + cross-module linkage"
author: "@stephen-cheng"
department: product
status: ready_for_review
priority: p2
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: infrastructure
eu_ai_act_risk_class: not_ai
target_release: "P2 / 2027-Q2"
client_visible: false
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Stand up the OKR module's schema and Apollo Federation subgraph. Five primitives: **Cycle** (the quarterly OKR period), **Objective** (the qualitative aspiration scoped to a Member, Team, or company-level), **KeyResult** (the measurable + time-bound outcome attached to an Objective), **CheckIn** (per-week or per-month progress update with confidence + commentary), and **Alignment** (the parent-child relationship between objectives — a team objective aligns to a company objective; a Member's objective aligns to a team objective). The schema supports the **OKR cascade pattern** (PRD §9.19): company-level → team-level → individual-level. Cross-module linkage: KeyResults can reference PROJ Issues + Cycles (FR-PROJ-001); KeyResults can reference Engagement metrics; KeyResults can reference KB pages as artefacts. Subsequent batch-07 FRs ship the cycle workflow + CUO/CSO review (FR-OKR-002) and the frontend at `/okr` (FR-OKR-003).

## Problem

PRD §9.19 names OKR as P2 with: "Quarterly OKR cycle; cascade through teams; CUO/CEO + CSO-skill review." Three failure modes the platform must prevent:

- **OKR drift.** Without a queryable cascade structure, "what's our company OKR for Q3?" varies by who you ask. The PRD's strategic alignment depends on this being one source.
- **OKR-disconnected work.** PROJ issues that don't trace back to a KeyResult are work without strategic justification; the cross-module linkage is the visibility floor.
- **Cycle-close opacity.** Without structured CheckIns + cycle-close review, the company learns from each quarter only by founder retrospection.

## Proposed Solution

The shape of the answer is `okr.*` schema (not in `hr_secure` — OKR data is not compensation-secret) + an Apollo Federation v2 subgraph + cross-module linkage primitives.

**Schema.**

```sql
CREATE SCHEMA okr;

-- Quarterly cycle.
CREATE TABLE okr.cycle (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  quarter TEXT NOT NULL,                                          -- "2026-Q3"
  starts_at DATE NOT NULL,
  ends_at DATE NOT NULL,
  description_md TEXT,
  founder_kickoff_md TEXT,                                         -- the founder's paragraph framing the quarter's strategic focus
  status TEXT NOT NULL DEFAULT 'planning',                         -- "planning" | "active" | "review" | "closed"
  closed_at TIMESTAMPTZ,
  closed_by UUID,
  cycle_review_md TEXT,                                            -- the CUO/CEO + CSO-drafted cycle-close review
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (tenant_id, quarter)
);

-- Objective: the qualitative aspiration.
CREATE TABLE okr.objective (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  cycle_id UUID NOT NULL REFERENCES okr.cycle(id) ON DELETE CASCADE,
  scope TEXT NOT NULL,                                             -- "company" | "team" | "individual"
  scope_team_kind TEXT,                                             -- when scope="team": e.g. "engineering" | "product" | "client_alpha"
  owner_member_id UUID NOT NULL,                                    -- the accountable owner
  parent_objective_id UUID REFERENCES okr.objective(id),            -- the cascade parent (team → company; individual → team)
  title TEXT NOT NULL,
  description_md TEXT,
  position INT NOT NULL DEFAULT 0,                                   -- ordering within scope
  status TEXT NOT NULL DEFAULT 'draft',                              -- "draft" | "active" | "completed" | "cancelled"
  signed_off_by_owner_at TIMESTAMPTZ,                                -- owner accepts the objective
  signed_off_by_founder_at TIMESTAMPTZ,                              -- founder accepts (for company + team scope)
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX objective_cycle_scope_idx ON okr.objective (tenant_id, cycle_id, scope, status);
CREATE INDEX objective_parent_idx      ON okr.objective (tenant_id, parent_objective_id);
CREATE INDEX objective_owner_idx       ON okr.objective (tenant_id, owner_member_id, cycle_id);

-- KeyResult: the measurable outcome.
CREATE TABLE okr.key_result (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  objective_id UUID NOT NULL REFERENCES okr.objective(id) ON DELETE CASCADE,
  position INT NOT NULL DEFAULT 0,
  title TEXT NOT NULL,
  description_md TEXT,
  metric_kind TEXT NOT NULL,                                        -- "number" | "percentage" | "currency_minor" | "binary"
                                                                   -- | "milestone_count"
  baseline_value NUMERIC,                                            -- starting state
  target_value NUMERIC NOT NULL,                                     -- the target to hit
  current_value NUMERIC,                                             -- updated by check-ins
  unit_label TEXT,                                                   -- "qualified leads", "Acme NPS", "VND", etc.
  data_source TEXT,                                                  -- "manual" | "proj_issue_count" | "crm_deal_amount"
                                                                   -- | "obs_metric" | "external_link"
  data_source_query JSONB,                                           -- e.g. { proj_issue_filter: ..., crm_deal_filter: ... }
  weight REAL NOT NULL DEFAULT 1.0,                                  -- per-KR weight in the objective's score
  status TEXT NOT NULL DEFAULT 'on_track',                            -- "on_track" | "at_risk" | "off_track" | "achieved" | "missed"
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX key_result_objective_idx ON okr.key_result (tenant_id, objective_id);

-- CheckIn: per-period progress update.
CREATE TABLE okr.check_in (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  key_result_id UUID NOT NULL REFERENCES okr.key_result(id) ON DELETE CASCADE,
  checked_in_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  checked_in_by UUID NOT NULL,
  current_value NUMERIC,                                             -- snapshot at this check-in
  confidence TEXT NOT NULL,                                          -- "low" | "medium" | "high"
  status_at_check_in TEXT NOT NULL,                                   -- "on_track" | "at_risk" | "off_track" | "achieved" | "missed"
  commentary_md TEXT,
  blockers_md TEXT,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);

CREATE INDEX check_in_kr_idx ON okr.check_in (tenant_id, key_result_id, checked_in_at DESC);

-- Cross-module linkage primitives.
CREATE TABLE okr.linked_artefact (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  key_result_id UUID NOT NULL REFERENCES okr.key_result(id) ON DELETE CASCADE,
  artefact_kind TEXT NOT NULL,                                        -- "proj_issue" | "proj_engagement" | "kb_page"
                                                                    -- | "crm_account" | "crm_deal" | "obs_metric" | "external_url"
  artefact_id UUID,                                                   -- references the linked entity by ID
  artefact_external_url TEXT,                                         -- when artefact is external
  description TEXT,
  added_by UUID NOT NULL,
  added_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);

CREATE INDEX linked_artefact_kr_idx       ON okr.linked_artefact (tenant_id, key_result_id);
CREATE INDEX linked_artefact_artefact_idx ON okr.linked_artefact (tenant_id, artefact_kind, artefact_id);
```

**Cascade rules (validated server-side).**

- A `company`-scope Objective can have no parent.
- A `team`-scope Objective MUST have a `company`-scope parent (or a higher team-level parent for nested teams).
- An `individual`-scope Objective SHOULD have a `team`-scope parent (warning if missing; not blocking).
- The cascade depth is unlimited but typically 3 levels (company → team → individual).
- Cycle-bound: parent + child must share the same cycle. (A new cycle requires re-cascading; the patterns from prior cycles can be cloned.)

The cascade is the visualisation primitive (FR-OKR-003 renders it as a tree).

**KeyResult metric kinds + auto-computed current_value.**

For `data_source: "manual"`, the `current_value` is updated by check-ins.

For automated sources:
- `proj_issue_count`: a periodic job reads the `data_source_query` filter against PROJ (FR-PROJ-001) and counts issues matching; updates `current_value`.
- `crm_deal_amount`: same pattern reading from CRM (FR-CRM-001) — sum of deal amounts matching the filter.
- `obs_metric`: queries Prometheus + the canonical metric expression; updates `current_value` daily.
- `external_link`: the `current_value` is the most-recent manual check-in (the link is for context, not auto-fetched).

The auto-computed values run nightly + on-demand on the OKR detail surface.

**Federation directives.**

- `OkrObjective @key(fields: "id")`.
- `OkrKeyResult @key(fields: "id")`.
- `OkrCycle @key(fields: "id")`.
- Cross-module: `extend type ProjIssue { okrLinkedKrs: [OkrKeyResult!] @requires("id") }` (PROJ-side extension lives in PROJ's federated subgraph; the resolution is via `okr.linked_artefact`).
- `Member @key(fields: "id") @external` references AUTH.

**GraphQL subgraph.**

```graphql
type Query {
  okrCycles(status: String): [OkrCycle!]!
  okrCycle(id: ID, quarter: String): OkrCycle
  okrCurrentCycle: OkrCycle
  okrObjectives(cycleId: ID!, scope: String, ownerId: ID, parentId: ID): [OkrObjective!]!
  okrObjective(id: ID!): OkrObjective
  okrKeyResults(objectiveId: ID!): [OkrKeyResult!]!
  okrKeyResult(id: ID!): OkrKeyResult
  okrCheckIns(keyResultId: ID!): [OkrCheckIn!]!
  okrLinkedArtefacts(keyResultId: ID!): [OkrLinkedArtefact!]!
  okrCascadeTree(cycleId: ID!): OkrCascadeTree!
  okrAlignmentReport(cycleId: ID!): OkrAlignmentReport!  # FR-OKR-002 deeper version
}

type Mutation {
  okrCreateCycle(input: OkrCycleInput!): OkrCycle!
  okrCloseCycle(id: ID!, reviewMd: String!): OkrCycle!
  okrCreateObjective(input: OkrObjectiveInput!): OkrObjective!
  okrUpdateObjective(id: ID!, patch: OkrObjectivePatch!): OkrObjective!
  okrSignOffObjective(id: ID!): OkrObjective!  # the owner / founder accepts
  okrArchiveObjective(id: ID!): OkrObjective!
  okrCreateKeyResult(input: OkrKeyResultInput!): OkrKeyResult!
  okrUpdateKeyResult(id: ID!, patch: OkrKeyResultPatch!): OkrKeyResult!
  okrSubmitCheckIn(input: OkrCheckInInput!): OkrCheckIn!
  okrLinkArtefact(input: OkrLinkedArtefactInput!): OkrLinkedArtefact!
  okrUnlinkArtefact(id: ID!): Boolean!
}
```

Persisted-queries discipline applies.

**RLS + ACL.**

- `okr.cycle`: tenant-readable; mutations restricted to Founder + Engineering Lead.
- `okr.objective`:
  - `scope: "company"`: all Members read; only Founder writes.
  - `scope: "team"`: team members + Founder read; team lead + Founder write.
  - `scope: "individual"`: the owner + their manager + Founder read + write.
- `okr.key_result`: same as parent objective.
- `okr.check_in`: same as KR; the checking-in Member writes their own entries.
- `okr.linked_artefact`: same as KR.

**Audit integration.** `okr.{tenant}` audit scope. Every mutation audit-logged.

**MCP tool surface (read-only).**

- `cyberos.okr.list_cycles(status?)` — read.
- `cyberos.okr.current_cycle` — read.
- `cyberos.okr.list_objectives(cycle_id, scope?, owner_id?)` — read.
- `cyberos.okr.get_objective(id)` — read.
- `cyberos.okr.list_key_results(objective_id)` — read.
- `cyberos.okr.list_check_ins(key_result_id)` — read.
- `cyberos.okr.cascade_tree(cycle_id)` — read; for the visualisation in FR-OKR-003.
- `cyberos.okr.search(query)` — read.

Mutation MCP tools (with destructive-confirmation gates) follow PROJ-008's pattern but ship in a smaller surface — most OKR mutations are signed UI flows. Specifically:

- `cyberos.okr.submit_check_in(key_result_id, ...)` — `destructive: false; idempotent: true` (check-ins are explicitly non-destructive logs).
- `cyberos.okr.link_artefact(key_result_id, ...)` — `destructive: false; idempotent: true`.
- `cyberos.okr.unlink_artefact(id)` — `destructive: true; requires_confirmation: true`.
- `cyberos.okr.nlcrud_propose_objective(utterance, cycle_id, scope?)` — propose-then-commit pattern from PROJ.
- `cyberos.okr.nlcrud_commit_objective(token)` — `destructive: true; requires_confirmation: true`.

CUO scope contract: read all + propose + check-in + link allowed; mutation-commits restricted to UI flow.

## Alternatives Considered

- **Use a hosted OKR tool (Lattice OKRs, Mooncamp, WorkBoard).** Rejected: residency + cross-module linkage with PROJ + CRM + KB cannot be enforced hosted.
- **Skip cascade rules; flat OKR list.** Rejected: PRD §9.19 explicitly names "cascade through teams"; visibility into alignment is the floor.
- **Per-Member personalised OKR templates.** Rejected for P2; founder + team lead auth is the floor.
- **No automated current_value updates.** Rejected: manual-only check-ins decay; the auto-source pattern keeps OKRs honest.

## Success Metrics

- **Primary metric.** P2 sprint demo passes: (1) the founder creates the first OKR cycle (Q3); (2) authors 3-5 company-level objectives with 3-5 KRs each; (3) team leads cascade team-level objectives; (4) Members cascade individual-level objectives; (5) auto-computed KR values from PROJ + CRM update correctly on the nightly job.
- **Adoption metric.** 100% of active employees have at least one signed-off objective in the active cycle by Q3 mid-cycle.
- **Latency NFR.** Cascade-tree query p95 ≤ 400 ms over a 100-objective tenant.

## Scope

**In-scope.**
- The 5 schema additions (`cycle`, `objective`, `key_result`, `check_in`, `linked_artefact`).
- Apollo Federation v2 subgraph with queries + mutations + subscriptions.
- Cascade rules + RLS + ACL.
- Auto-computed current_value pipeline (PROJ + CRM + OBS sources).
- Federation directives for cross-module linkage.
- The 11 MCP tools (read + check-in + propose-then-commit).
- Audit integration in scope `okr.{tenant}`.

**Out-of-scope (deferred to FR-OKR-002 / FR-OKR-003).**
- Quarterly cycle workflow + CUO/CEO + CSO-skill review (FR-OKR-002).
- Frontend remote at /okr (FR-OKR-003).
- Multi-cycle trend analysis (P3).
- OKR scoring rubric beyond simple % achievement (P3 — Andy Grove vs. Doerr models).
- Public OKR surface for client portal (P4).

## Dependencies

- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-002 / FR-MCP-001.
- FR-PROJ-001 / FR-PROJ-007 (proj_issue_count + proj_engagement metric sources).
- FR-CRM-001 (crm_deal_amount metric source).
- FR-OBS-001 / FR-OBS-002 (obs_metric source).
- FR-DESIGN-001.
- Compliance: PDPL Decree 13 (OKR data may include personal targets); EU AI Act Article 22 (auto-computed values are deterministic; not automated decisions).
- Locked decisions referenced: DEC-218 (cascade pattern: company → team → individual), DEC-219 (auto-computed KR values from PROJ + CRM + OBS), DEC-220 (per-scope ACL: company-readable; team team-readable; individual owner+manager+founder).

## AI Risk Assessment

Not applicable. `eu_ai_act_risk_class: not_ai`. The schema + GraphQL + auto-compute are deterministic. AI surfaces (CUO/CEO + CSO review) ship in FR-OKR-002 with their own classification.
