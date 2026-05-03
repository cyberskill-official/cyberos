---
title: "PROJ — issue lifecycle, custom states per project, WIP limits, transition rules"
author: "@stephen-cheng"
department: product
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

Define and enforce the **issue lifecycle**: the default seven-state catalogue (`todo` / `in_progress` / `blocked` / `in_review` / `done` / `cancelled` / `duplicate`); per-project **custom states** that override the default while preserving the canonical "completion" semantics; **transition rules** that enforce which states can move to which (e.g. `done` → `todo` requires explicit reopen); **work-in-progress (WIP) limits** per state per assignee per project (the kanban discipline); **auto-transitions** on linked-PR merge / linked-PR closed / blocking-issue resolved; and the **completion semantics** that downstream modules (TIME, INV, OKR, RES) depend on. The lifecycle is the contract that makes the kanban view (FR-PROJ-005) meaningful and that makes the cycle-close numbers (FR-PROJ-004) auditable.

## Problem

A project tracker without lifecycle rules is a glorified todo list. Three failures the team will hit immediately without enforcement:

- **State drift.** A Member moves an issue to `done`, then back to `in_progress`, then to `blocked`, then to `done` again with no reopen reason — the cycle-close numbers (issues completed, velocity, carryover) become unreliable.
- **Hidden WIP.** A Member silently has 12 issues `in_progress` simultaneously; the cycle plan said capacity was 6; the cycle slips and no one notices until close.
- **Manual reverse-link maintenance.** A linked GitHub PR is merged; the issue stays `in_progress` because no one updated PROJ; status reports show wrong work-in-flight counts.

The PRD §9.5 commits to issue lifecycle as part of the Linear-style three primitives; this FR is the rule layer.

## Proposed Solution

The shape of the answer is a `proj.workflow` table per project + transition rules + WIP-limit enforcement at the mutation layer + auto-transition consumers.

**Default state catalogue.**

| State | Category | Description |
|---|---|---|
| `todo` | unstarted | The issue is scheduled but not yet picked up. |
| `in_progress` | started | A Member is actively working on it. |
| `blocked` | started | Work is paused on an external dependency. |
| `in_review` | started | Code or design is in review. |
| `done` | completed | Work is delivered + accepted. |
| `cancelled` | completed | Work will not be done; preserved for history. |
| `duplicate` | completed | Closed in favour of another issue (`metadata.duplicate_of`). |

**Categories** (the underlying semantic that downstream modules read):
- `unstarted`: no work has begun; counts in cycle backlog.
- `started`: work is in flight; counts in active WIP.
- `completed`: work is done; counts in cycle completion + velocity.

Custom states must declare their category so downstream calculations remain correct.

**Custom states per project.**

```sql
CREATE TABLE proj.workflow (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  project_id UUID NOT NULL REFERENCES proj.project(id) ON DELETE CASCADE,
  state_key TEXT NOT NULL,                     -- e.g. "qa_pending"
  state_label TEXT NOT NULL,                    -- "QA Pending"
  category TEXT NOT NULL,                       -- "unstarted" | "started" | "completed"
  position INT NOT NULL,                        -- ordering on the board view
  is_default BOOLEAN NOT NULL DEFAULT false,    -- new issues default here for this category
  is_archived BOOLEAN NOT NULL DEFAULT false,   -- soft-delete with backfill
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (tenant_id, project_id, state_key)
);
```

A new project is seeded with the default seven states. Adding a state is one click; renaming is one click; archiving requires that no `active` issue is in that state (issues are bulk-moved first).

**Transition rules.**

```sql
CREATE TABLE proj.workflow_transition (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  project_id UUID NOT NULL REFERENCES proj.project(id) ON DELETE CASCADE,
  from_state_key TEXT NOT NULL,
  to_state_key TEXT NOT NULL,
  requires_reason BOOLEAN NOT NULL DEFAULT false,
  requires_role TEXT,                           -- e.g. "lead" — only Project Lead can perform
  auto_apply_on_event TEXT,                     -- "linked_pr_merged" | "linked_pr_closed" | "blocker_resolved"
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  UNIQUE (tenant_id, project_id, from_state_key, to_state_key)
);
```

Default transitions seeded per project:

| From | To | Rules |
|---|---|---|
| `todo` | `in_progress` | open |
| `todo` | `cancelled` | open |
| `in_progress` | `blocked` | requires_reason: true |
| `in_progress` | `in_review` | open |
| `in_progress` | `done` | requires_reason: false (but cycle close prefers via `in_review`) |
| `blocked` | `in_progress` | open |
| `blocked` | `cancelled` | requires_reason: true |
| `in_review` | `in_progress` | open (review-rejected) |
| `in_review` | `done` | open |
| `done` | `todo` | requires_reason: true; reopen requires `lead` role |
| `cancelled` | `todo` | requires_reason: true; reopen requires `lead` role |
| `duplicate` | `todo` | requires_reason: true; rare |

Any transition not in the table is rejected at the GraphQL mutation layer with `code: TRANSITION_NOT_ALLOWED`. A project can add custom transitions or relax the default set; the audit log captures every workflow change.

**Reason capture.** When `requires_reason: true`, the mutation requires `reasonMd` in the input; the reason is stored in `proj.issue_state_transition.reason_md` and surfaces in the issue history.

**WIP limits.**

```sql
CREATE TABLE proj.wip_limit (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  project_id UUID NOT NULL REFERENCES proj.project(id) ON DELETE CASCADE,
  scope TEXT NOT NULL,                          -- "per_assignee" | "per_state" | "per_project"
  state_key TEXT,                               -- nullable for "per_project"
  member_id UUID,                               -- nullable for non-per-Member scopes
  cap INT NOT NULL,
  enforcement TEXT NOT NULL,                    -- "warn" | "block"
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

Defaults seeded per project on creation:
- `per_assignee + state=in_progress + cap=4 + enforcement=warn`.
- `per_assignee + state=in_review + cap=3 + enforcement=warn`.
- `per_project + state=blocked + cap=10 + enforcement=warn`.

Project Lead can change limits; HR/Ops Lead can set tenant-wide defaults. `block` mode prevents the mutation; the user sees "WIP limit reached for in_progress (4); finish or reassign one issue first". `warn` mode allows the mutation but surfaces a Notify card to the assignee + Project Lead.

**Auto-transitions.**

Three triggers ship in P1:

1. **`linked_pr_merged`.** A GitHub webhook (configured per Engagement at FR-PROJ-007 in this batch) fires; if the PR's commit message contains `Closes #ALPHA-1234` or `Fixes #ALPHA-1234`, the issue auto-transitions to the target state. The default rule is `in_review → done`; configurable per project. The transition writes an audit row with `actor_kind: 'system'`, `actor_origin: 'github_webhook'`, plus a Genie panel Notify to the assignee.
2. **`linked_pr_closed`.** Symmetric: if the PR is closed without merge, the issue surfaces a Notify "PR closed; what next?" with options to transition to `cancelled`, `todo`, or no-change.
3. **`blocker_resolved`.** When an issue's `blocked_by_issue_ids[]` has its last unresolved entry transition to `done`, the blocked issue surfaces a Notify "blocker resolved; ready to resume?" with a one-click `blocked → in_progress` transition.

**State-aware rendering.**

Issue cards in the board view show:
- Category-coloured chip (unstarted = grey; started = blue; completed = green).
- WIP-limit ribbon if the assignee is over the cap.
- Auto-transition source if the most recent transition was system-driven.

**Bulk transitions.**

A multi-select bulk action: "move 12 issues to `cancelled` (cycle close cleanup)". Bulk transitions atomic-batch through the sync engine; the mutation envelope contains an array of issue IDs and the target state; rebases and conflicts apply per-issue.

**MCP tool surface (extends FR-PROJ-001's read-only set).** Mutations land in FR-PROJ-008; this FR ships the read tools related to lifecycle:
- `cyberos.proj.list_workflow_states(project_id)` (read).
- `cyberos.proj.list_workflow_transitions(project_id)` (read).
- `cyberos.proj.list_wip_limits(project_id, member_id?)` (read).
- `cyberos.proj.check_wip(project_id, member_id, target_state)` (read; returns `{ over_cap: bool, current: int, cap: int }`).

**Audit integration.** State transitions write to `proj.issue_state_transition` (queryable per-issue) + the canonical `audit.entry` in scope `proj.{tenant}` (chain-protected per FR-AUTH-002). Workflow + WIP-limit changes are themselves audit-logged.

## Alternatives Considered

- **No transition rules; let any state move to any state.** Rejected: cycle-close numbers degrade silently; PRD §14.2.3 P1 → P2 gate ("PROJ has fully replaced the prior project tracker for at least 21 consecutive days") relies on those numbers being trustworthy.
- **Per-issue workflow (each issue declares its own).** Rejected: per-project is the right granularity; per-issue is operational chaos.
- **Hard WIP limits everywhere by default.** Rejected: too disruptive in the first weeks of adoption; warn mode is the floor, projects can opt into block mode.
- **Auto-transition driven by status keywords in commit messages alone.** Considered; current solution combines GitHub webhook + commit message regex; arguably the same thing. We use GitHub's `Closes #` convention as canonical (the prior art Linear and Asana both use this).
- **Treat custom states as an entirely separate table per project.** Rejected: the workflow table with a project FK is the simpler shape.

## Success Metrics

- **Primary metric.** P1 sprint demo passes: (1) every default transition allowed; the disallowed set rejected; (2) a `done → todo` transition without `reasonMd` rejected; (3) a per-assignee WIP limit triggers a warn Notify on the 5th in-progress issue; (4) a synthetic GitHub webhook with `Closes #ALPHA-1234` auto-transitions the issue to `done` and writes the audit row.
- **Adoption metric.** 0 unaudited workflow drift events at P1 → P2 exit (every transition has a row).
- **Quality metric.** Reopen rate (`done → todo`) ≤ 8% on a 30-day rolling window — proves cycle-close completion data is trustworthy.

## Scope

**In-scope.**
- The default seven-state catalogue + categories.
- `proj.workflow`, `proj.workflow_transition`, `proj.wip_limit` tables.
- Default seed for each new project.
- Transition-rule enforcement at mutation layer.
- WIP-limit warn/block enforcement at mutation + UI surface.
- Three auto-transitions (`linked_pr_merged`, `linked_pr_closed`, `blocker_resolved`).
- Bulk-transition support.
- The four read MCP tools.
- Audit integration.

**Out-of-scope (deferred).**
- Per-Member custom workflows (P3 if the team needs them).
- SLA-driven transitions (P2 — issue must move to `in_review` within X hours of `in_progress`).
- Workflow-driven approvals (a transition requires explicit second-Member approval) — P3.
- Auto-transition from JIRA / Linear / Asana — only GitHub in P1; others P2.

## Dependencies

- FR-PROJ-001 (schema).
- FR-PROJ-002 (sync engine; transitions ride the optimistic mutation contract).
- FR-AUTH-001 / FR-AUTH-002 (RBAC + audit).
- FR-MCP-001 (read MCP tools).
- FR-INFRA-001 (NATS for webhook ingestion).
- A GitHub webhook receiver (`cyberos-proj-github-receiver`) with HMAC verification.
- Compliance: SOC 2 CC8 (change management — workflow + transition rules are themselves a change-control surface).
- Locked decisions referenced: DEC-105 (default seven-state catalogue), DEC-106 (custom states must declare a category), DEC-107 (WIP limits warn-by-default), DEC-108 (`done → todo` requires lead role + reason).

## AI Risk Assessment

Not applicable. `eu_ai_act_risk_class: not_ai`. Lifecycle rules are deterministic; the auto-blocker-resolved Notify card inherits FR-GENIE-001's risk classification.

## AI Authorship Disclosure

- **Tools used:** Claude Cowork (Anthropic).
- **Scope:** drafted the FR end-to-end against the PRD + SRS; founder reviews and edits before status changes from `ready_for_review`.
- **Human review:** founder (`@stephen-cheng`) — final wording is the founder's responsibility.
