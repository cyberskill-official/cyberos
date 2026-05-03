---
title: "PROJ — three primitives + Engagement schema (Issue, Cycle, Project, Engagement); Apollo subgraph; RLS; audit"
author: "@stephen-cheng"
department: engineering
status: ready_for_review
priority: p1
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: infrastructure
eu_ai_act_risk_class: not_ai
target_release: "P1 / 2026-Q4"
client_visible: false
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Stand up the canonical schema and GraphQL subgraph for the PROJ module's four primitives: **Issue** (the unit of work), **Cycle** (the time-boxed iteration), **Project** (the durable container that aggregates issues across cycles), and **Engagement** (the Vietnamese-consultancy-specific overlay — the contract under which a Project is delivered for a Client). The schema is the floor on which every other PROJ FR is built; subsequent batch-04 FRs add sync engine (FR-PROJ-002), issue lifecycle (FR-PROJ-003), cycle planning (FR-PROJ-004), the frontend (FR-PROJ-005), AI (FR-PROJ-006), Engagements + CRM seam (FR-PROJ-007), MCP (FR-PROJ-008), migration (FR-PROJ-009), and notifications (FR-PROJ-010). This FR ships data + GraphQL only; no UI, no AI, no sync — those layer on top.

## Problem

PROJ is the second-most-load-bearing module in P1 after EMAIL — every Member uses it daily; every Engagement's status, every Cycle's progress, every Issue's blocker traces through here. Without a clean primitives schema, every later FR re-invents data model semantics, and the dogfooding bet (Bet 4) collapses because the team will not adopt PROJ if the basic shape disagrees with how they think about work.

The PRD §9.5.1 makes the four-primitive choice explicit: "Linear pioneered (and Height, Cycle, and Plane have validated) the three-primitive model that consultancies need: Issues (the unit of work), Cycles (the time-boxed iteration), and Projects (the durable container)." The Vietnamese-consultancy-specific Engagement overlay maps a contract to a Project — a property no Linear-clone has out of the box, but one CyberSkill needs to reconcile time, invoicing (FR-INV in P2), revenue sharing (FR-RES in P2), and client-portal visibility (FR-PORTAL in P4).

## Proposed Solution

The shape of the answer is a Postgres schema in the `proj` namespace, an Apollo Federation v2 subgraph, RLS policies that respect tenant + assignment + privacy invariants, and audit-row coverage on every state transition.

**Schema.**

```sql
CREATE SCHEMA proj;

-- Engagement: the contract under which a Project is delivered for a Client.
-- Engagements live above Projects; one Engagement can contain N Projects.
CREATE TABLE proj.engagement (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  name TEXT NOT NULL,                          -- "Acme Corp – Long-term retainer 2024-2026"
  slug TEXT NOT NULL,                          -- "acme-retainer-2024-2026"
  client_account_id UUID,                      -- FK to crm.account when CRM ships (FR-CRM-001 in batch-05)
  contract_kind TEXT NOT NULL,                 -- "fixed_price" | "time_and_materials" | "retainer" | "internal"
  contract_signed_at DATE,
  start_date DATE,
  end_date DATE,                               -- nullable for open-ended retainers
  rate_card JSONB,                             -- per-role rates; consumed by INV/RES in P2
  budget_hours INT,
  budget_amount_minor INT,                     -- amount in tenant currency minor units
  budget_currency TEXT,                        -- ISO 4217
  status TEXT NOT NULL DEFAULT 'active',       -- "active" | "paused" | "closed" | "draft"
  primary_owner_member_id UUID NOT NULL,
  client_visibility_default TEXT NOT NULL DEFAULT 'internal_only',
                                               -- "internal_only" | "client_visible_summary" | "client_visible_full"
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  archived_at TIMESTAMPTZ
);

-- Project: the durable container that aggregates issues across cycles.
CREATE TABLE proj.project (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  engagement_id UUID NOT NULL REFERENCES proj.engagement(id),
  key TEXT NOT NULL,                           -- e.g. "ALPHA"; derives Issue codes "ALPHA-1234"
  name TEXT NOT NULL,
  description_md TEXT,
  status TEXT NOT NULL DEFAULT 'planning',     -- "planning" | "active" | "paused" | "completed" | "cancelled"
  lead_member_id UUID,
  start_date DATE,
  target_date DATE,
  completed_at TIMESTAMPTZ,
  default_cycle_length_days INT NOT NULL DEFAULT 14,
  custom_states JSONB,                          -- override default issue states (FR-PROJ-003)
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  archived_at TIMESTAMPTZ,
  UNIQUE (tenant_id, key)
);

-- Cycle: the time-boxed iteration the issue is assigned to.
CREATE TABLE proj.cycle (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  project_id UUID NOT NULL REFERENCES proj.project(id),
  number INT NOT NULL,                          -- monotonic per project (Cycle 14 of Acme)
  name TEXT,                                    -- optional override ("Sprint 14: Acme Q3 launch")
  starts_at TIMESTAMPTZ NOT NULL,
  ends_at TIMESTAMPTZ NOT NULL,
  goal_md TEXT,                                 -- one-paragraph cycle goal
  status TEXT NOT NULL DEFAULT 'planned',       -- "planned" | "active" | "completed" | "cancelled"
  completed_at TIMESTAMPTZ,
  cycle_review_md TEXT,                         -- the cycle-close review (FR-PROJ-004 + FR-PROJ-006)
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (tenant_id, project_id, number)
);

-- Issue: the unit of work.
CREATE TABLE proj.issue (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  project_id UUID NOT NULL REFERENCES proj.project(id),
  cycle_id UUID REFERENCES proj.cycle(id),      -- nullable: an issue can be unscheduled
  number INT NOT NULL,                          -- monotonic per project
  title TEXT NOT NULL,
  description_md TEXT,
  state TEXT NOT NULL DEFAULT 'todo',
                                               -- default catalog: "todo" | "in_progress" | "blocked" |
                                               -- "in_review" | "done" | "cancelled" | "duplicate"
                                               -- (FR-PROJ-003 lifecycle; project can override)
  priority TEXT NOT NULL DEFAULT 'medium',     -- "urgent" | "high" | "medium" | "low" | "none"
  assignee_member_id UUID,
  reporter_member_id UUID NOT NULL,
  estimate_points INT,                          -- t-shirt sizing; convertible per project
  due_date DATE,
  labels TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
  parent_issue_id UUID REFERENCES proj.issue(id),
  blocked_by_issue_ids UUID[] NOT NULL DEFAULT ARRAY[]::UUID[],
  blocks_issue_ids UUID[] NOT NULL DEFAULT ARRAY[]::UUID[],
  external_refs JSONB NOT NULL DEFAULT '[]'::jsonb,
                                               -- e.g. [{kind: "email_thread", id: ..., url: ...},
                                               --       {kind: "github_pr",   id: ..., url: ...}]
  client_visible BOOLEAN NOT NULL DEFAULT false,
  ai_assisted_fields JSONB NOT NULL DEFAULT '{}'::jsonb,
                                               -- which fields are AI-generated; consumed by Article 50 chips
  body_pgrn TSVECTOR_TYPE NOT NULL,
  completed_at TIMESTAMPTZ,
  cancelled_at TIMESTAMPTZ,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (tenant_id, project_id, number)
);

CREATE INDEX issue_tenant_cycle_idx       ON proj.issue (tenant_id, cycle_id, state);
CREATE INDEX issue_tenant_assignee_idx    ON proj.issue (tenant_id, assignee_member_id, state);
CREATE INDEX issue_tenant_pgrn_idx        ON proj.issue USING pgroonga (body_pgrn);
CREATE INDEX issue_tenant_labels_idx      ON proj.issue USING gin (labels);
CREATE INDEX issue_blocked_by_idx         ON proj.issue USING gin (blocked_by_issue_ids);
CREATE INDEX issue_external_refs_idx      ON proj.issue USING gin (external_refs jsonb_path_ops);

-- Comment thread on an issue (separate table for fan-out + soft-delete).
CREATE TABLE proj.issue_comment (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  issue_id UUID NOT NULL REFERENCES proj.issue(id) ON DELETE CASCADE,
  author_member_id UUID NOT NULL,
  body_md TEXT NOT NULL,
  mentions UUID[],
  internal_only BOOLEAN NOT NULL DEFAULT false,  -- true: never visible to client_visible_full clients
  edited_at TIMESTAMPTZ,
  deleted_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- State transition history (separate table; the audit log is the canonical source but this is queryable per-issue).
CREATE TABLE proj.issue_state_transition (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  issue_id UUID NOT NULL REFERENCES proj.issue(id) ON DELETE CASCADE,
  from_state TEXT,
  to_state TEXT NOT NULL,
  actor_member_id UUID,
  reason_md TEXT,
  occurred_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Membership in a Project (who has access; what role).
CREATE TABLE proj.project_membership (
  tenant_id UUID NOT NULL,
  project_id UUID NOT NULL REFERENCES proj.project(id),
  member_id UUID NOT NULL,
  role TEXT NOT NULL,                           -- "lead" | "contributor" | "viewer"
  added_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (tenant_id, project_id, member_id)
);
```

**RLS.** Every `proj.*` table has `FORCE ROW LEVEL SECURITY` with a tenant predicate. `proj.issue` adds an additional ACL: a Member sees an issue if any of (a) the Member is in the project's `project_membership`, (b) the issue is `client_visible: true` and the Member has `crm.read` on the linked Engagement's account, (c) the Member has the `proj.read.*` predicate (Founder, Engineering Lead, Auditor, DPO).

**GraphQL subgraph.**

```graphql
type Query {
  projEngagements(status: String, ownerId: ID): [ProjEngagement!]!
  projEngagement(id: ID!): ProjEngagement
  projProjects(engagementId: ID, status: String, scope: ProjScope = MEMBER): [ProjProject!]!
  projProject(id: ID, key: String): ProjProject
  projCycles(projectId: ID!, status: String): [ProjCycle!]!
  projCycle(id: ID!): ProjCycle
  projCurrentCycle(projectId: ID!): ProjCycle
  projIssues(filter: ProjIssueFilter!, sort: ProjIssueSort, first: Int = 50, after: String): ProjIssueConnection!
  projIssue(id: ID, key: String): ProjIssue
  projSearch(query: String!, projectIds: [ID!], states: [String!], assigneeIds: [ID!], first: Int = 50): [ProjIssue!]!
}

type Mutation {
  projCreateEngagement(input: ProjEngagementInput!): ProjEngagement!
  projUpdateEngagement(id: ID!, patch: ProjEngagementPatch!): ProjEngagement!
  projArchiveEngagement(id: ID!): ProjEngagement!

  projCreateProject(input: ProjProjectInput!): ProjProject!
  projUpdateProject(id: ID!, patch: ProjProjectPatch!): ProjProject!
  projArchiveProject(id: ID!): ProjProject!

  projCreateCycle(input: ProjCycleInput!): ProjCycle!
  projUpdateCycle(id: ID!, patch: ProjCyclePatch!): ProjCycle!
  projCloseCycle(id: ID!, reviewMd: String, carryoverIssueIds: [ID!]): ProjCycle!

  projCreateIssue(input: ProjIssueInput!): ProjIssue!
  projUpdateIssue(id: ID!, patch: ProjIssuePatch!): ProjIssue!
  projTransitionIssue(id: ID!, toState: String!, reasonMd: String): ProjIssue!
  projAssignIssue(id: ID!, assigneeMemberId: ID): ProjIssue!
  projMoveIssueToCycle(id: ID!, cycleId: ID): ProjIssue!
  projAddBlockedBy(id: ID!, blockedById: ID!): ProjIssue!
  projRemoveBlockedBy(id: ID!, blockedById: ID!): ProjIssue!
  projAddIssueComment(issueId: ID!, body: String!, internalOnly: Boolean = false): ProjIssueComment!
  projEditIssueComment(commentId: ID!, body: String!): ProjIssueComment!
  projDeleteIssueComment(commentId: ID!): Boolean!
  projAddProjectMember(projectId: ID!, memberId: ID!, role: String!): Boolean!
  projRemoveProjectMember(projectId: ID!, memberId: ID!): Boolean!
}

type Subscription {
  projIssueStream(projectId: ID!): ProjIssueEvent!
  projCycleStream(projectId: ID!): ProjCycleEvent!
}
```

Persisted-queries discipline applies (FR-INFRA-001).

**Federation directives.**
- `ProjEngagement @key(fields: "id")` — exposes a stub for CRM (`extend type ProjEngagement { account: CrmAccount @requires }` lives in the CRM subgraph in batch-05).
- `ProjIssue @key(fields: "id")` — exposes a stub for EMAIL (so FR-EMAIL-007 can `extend type ProjIssue { sourceEmailThread: EmailThread @requires }`).
- `Member @key(fields: "id") @external` — references the AUTH subgraph's Member type for `assigneeMemberId`, `reporterMemberId`, `leadMemberId`.

**Audit integration.** Every mutation writes an audit row in scope `proj.{tenant}` with the issue/cycle/project ID, before/after state for transitions, and the calling subject. The state-transition table is the queryable mirror for in-app UX; the audit log is the canonical source for compliance evidence.

**Slug + Key generation.** `proj.engagement.slug` is auto-derived from `name` (kebab-case, ASCII-fold Vietnamese diacritics, 2-30 chars) with collision-suffix; `proj.project.key` is uppercase 2-6 chars derived from name's initials (Member can override).

**MCP tool surface (read-mostly).** Mutations land in FR-PROJ-008; this FR ships the read tools:
- `cyberos.proj.list_engagements(status?, owner_id?)`
- `cyberos.proj.get_engagement(id)`
- `cyberos.proj.list_projects(engagement_id?, status?, scope?)`
- `cyberos.proj.get_project(id_or_key)`
- `cyberos.proj.list_cycles(project_id, status?)`
- `cyberos.proj.get_current_cycle(project_id)`
- `cyberos.proj.list_issues(filter, sort?, first?, after?)`
- `cyberos.proj.get_issue(id_or_key)`
- `cyberos.proj.search(query, ...)`

All `read_only: true`.

**Seed data.** P1 deploy seeds two Engagements (the two long-term CyberSkill projects from PRD §1.1's Origin), each with one Project; the team's prior project-tracker data is migrated by FR-PROJ-009.

## Alternatives Considered

- **Skip Engagement; the contract metadata lives outside PROJ.** Rejected: revenue-sharing (RES P2) and invoicing (INV P2) require the Engagement abstraction; building it later forces schema migration.
- **Use Linear's exact schema (Team + Cycle + Issue + Project).** Rejected: Linear's "Team" maps poorly onto a 10-person Vietnamese consultancy; we collapse Team into Project and add Engagement above to handle contract-level concerns.
- **JSON-blob storage for everything; let the application enforce shape.** Rejected: cross-module joins (CRM, INV, EMAIL, RES) require typed relations; JSON-blob would defeat federation.
- **Store comments as Markdown only without a separate table.** Rejected: per-comment editing, soft-delete, and mention indexing are needed.
- **Issue states as a separate table per project.** Considered for FR-PROJ-003; this FR keeps states as a string with `proj.project.custom_states JSONB` overrides — simpler at this scale.

## Success Metrics

- **Primary metric.** P1 sprint demo passes: (1) the founder creates an Engagement + Project + Cycle + Issue end-to-end via GraphQL; (2) the issue's key (e.g. `ALPHA-1` for project key `ALPHA`) renders correctly; (3) RLS denies a non-member's read of a private project's issue; (4) the search query returns the issue with PGroonga Vietnamese tokenisation.
- **Coverage metric.** 100% of mutations write an audit row + an `issue_state_transition` row where applicable.
- **Latency metric.** `projIssues` query with `cycle_id` filter p95 ≤ 120 ms over a synthetic 50K-issue corpus.

## Scope

**In-scope (P1 sprint cluster S1-3 to S1-4).**
- The `proj` schema with all six tables.
- RLS policies + Member-ACL composition for `client_visible` issues.
- Apollo Federation v2 subgraph with all queries + mutations + subscriptions.
- Audit integration in scope `proj.{tenant}`.
- Read-only MCP tools.
- Seed migration for the two Engagements.
- PGroonga search with Vietnamese tokenisation.

**Out-of-scope (deferred).**
- Sync engine + offline behaviour (FR-PROJ-002).
- Issue-state lifecycle rules + custom workflows (FR-PROJ-003).
- Cycle planning UX + capacity allocation (FR-PROJ-004).
- Frontend remote (FR-PROJ-005).
- AI features (FR-PROJ-006).
- CRM linkage at the Engagement level (FR-PROJ-007 + batch-05).
- Mutation MCP tools with destructive-confirmation gates (FR-PROJ-008).
- Migration from Asana/etc. (FR-PROJ-009).
- Notifications + standup integration (FR-PROJ-010).

## Dependencies

- FR-INFRA-001 (Postgres + NATS + federation).
- FR-AUTH-001 / FR-AUTH-002 (Member federation + audit log).
- FR-MCP-001 (read-only tool registration).
- FR-DESIGN-001 (no UI in this FR; subsequent FRs consume tokens).
- Compliance: PDPL Decree 13 (issue + comment text is personal-data-eligible; the audit + denylist controls apply at the BRAIN ingestion path when issues are surfaced as facts).
- Locked decisions referenced: DEC-099 (four primitives: Issue, Cycle, Project, Engagement), DEC-100 (Engagement above Project for contract-level concerns), DEC-101 (custom-state override at Project level via JSONB).

## AI Risk Assessment

Not applicable. `eu_ai_act_risk_class: not_ai`. The schema + GraphQL are deterministic. The `ai_assisted_fields` JSONB is a forward-compatible marker for FR-PROJ-006's AI surfaces but emits nothing in this FR.
