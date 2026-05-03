---
title: "PROJ — MCP mutation surface (agent parity for create / update / transition / assign / move / close cycle)"
author: "@stephen-cheng"
department: engineering
status: ready_for_review
priority: p1
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: infrastructure
eu_ai_act_risk_class: limited
target_release: "P1 / 2026-Q4"
client_visible: false
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Ship the full PROJ MCP mutation tool surface so agents (Claude.ai, Cursor, Claude Desktop, Raycast, the embedded CUO client) can perform every PROJ action a human can perform — under the same RBAC, audit, persona-scope, destructive-confirmation, and step-up auth. This FR is the architectural delivery of the Bet 1 (agent parity is the moat) for PROJ. Tools land per-mutation: `create_issue`, `update_issue`, `transition_issue`, `assign_issue`, `move_to_cycle`, `add_comment`, `link_to_pr`, `add_blocker`, `remove_blocker`, `create_cycle`, `close_cycle`, `create_project`, `update_engagement`, etc. Every destructive mutation (`destructive: true`) requires `client_confirmed: true` per FR-MCP-001's annotation enforcement; high-sensitivity mutations (`close_cycle`, `update_engagement.budget`, mass bulk operations) additionally require step-up auth (FR-AUTH-003). The CUO/COO + CUO/CTO + CUO/CPO scope contracts allow `read` and `propose` (NLCRUD-style draft tools) but **never `commit`** — `nlcrud_propose` returns a confirmation token; `nlcrud_commit` is a human-only call.

## Problem

The PRD's Bet 1 collapses if PROJ exposes a different doorway for humans (GraphQL UI) versus agents (MCP). Three architectural failures the platform must avoid:

- **Agent shadow-permissions.** An agent that can do *more* than the human (because the MCP path lacks a check that the GraphQL path enforces) is a security hole.
- **Agent auto-mutation.** An agent that can mutate without explicit human-in-the-loop confirmation produces unrecoverable mistakes; the destructive-confirmation pattern is the architectural floor.
- **Tool naming inconsistency.** Tools that don't follow `cyberos.{module}.{verb}_{noun}` produce ambiguous prompts; the convention is locked at FR-MCP-001.

This FR is the inverse of FR-PROJ-005: that ships the human UX; this ships the equivalent agent surface.

## Proposed Solution

The shape of the answer is a complete mutation tool catalogue, registered with the MCP gateway, annotated correctly, scope-contract-aware, and audit-integrated.

**Tool catalogue.**

Engagement-level:
- `cyberos.proj.create_engagement(input)` — `destructive: true; requires_confirmation: true; sensitivity: high; step_up_required: true`.
- `cyberos.proj.update_engagement(id, patch)` — `destructive: true; requires_confirmation: true`. If `patch` includes `budget_*`, `rate_card`, or `client_account_id`, additionally `sensitivity: high; step_up_required: true`.
- `cyberos.proj.archive_engagement(id, reason)` — `destructive: true; requires_confirmation: true; sensitivity: high; step_up_required: true`.
- `cyberos.proj.set_engagement_visibility(id, default)` — `destructive: true; requires_confirmation: true; sensitivity: medium`.

Project-level:
- `cyberos.proj.create_project(input)` — `destructive: true; requires_confirmation: true`.
- `cyberos.proj.update_project(id, patch)` — `destructive: true; requires_confirmation: true`.
- `cyberos.proj.archive_project(id, reason)` — `destructive: true; requires_confirmation: true; sensitivity: high; step_up_required: true`.
- `cyberos.proj.add_project_member(project_id, member_id, role)` — `destructive: true; requires_confirmation: true`.
- `cyberos.proj.remove_project_member(project_id, member_id, reason)` — `destructive: true; requires_confirmation: true`.
- `cyberos.proj.create_workflow_state(project_id, key, label, category, position)` — `destructive: true; requires_confirmation: true`.
- `cyberos.proj.update_workflow_state(state_id, patch)` — `destructive: true; requires_confirmation: true`.
- `cyberos.proj.set_wip_limit(project_id, scope, ...)` — `destructive: true; requires_confirmation: true`.

Cycle-level:
- `cyberos.proj.create_cycle(input)` — `destructive: true; requires_confirmation: true`.
- `cyberos.proj.update_cycle(id, patch)` — `destructive: true; requires_confirmation: true`.
- `cyberos.proj.close_cycle(id, review_md, carryover_ids, cancel_ids, unschedule_ids)` — `destructive: true; requires_confirmation: true; sensitivity: high; step_up_required: true`.
- `cyberos.proj.set_cycle_capacity(cycle_id, member_id, capacity_points)` — `destructive: true; requires_confirmation: true`.

Issue-level:
- `cyberos.proj.create_issue(input)` — `destructive: true; requires_confirmation: true`.
- `cyberos.proj.update_issue(id, patch)` — `destructive: true; requires_confirmation: true`.
- `cyberos.proj.transition_issue(id, to_state, reason_md)` — `destructive: true; requires_confirmation: true`.
- `cyberos.proj.assign_issue(id, assignee_member_id)` — `destructive: true; requires_confirmation: true`.
- `cyberos.proj.move_to_cycle(id, cycle_id_or_null)` — `destructive: true; requires_confirmation: true`.
- `cyberos.proj.add_blocker(id, blocked_by_id)` — `destructive: true; requires_confirmation: true`.
- `cyberos.proj.remove_blocker(id, blocked_by_id)` — `destructive: true; requires_confirmation: true`.
- `cyberos.proj.add_comment(issue_id, body, internal_only)` — `destructive: false; sensitivity: low` (writes a comment but doesn't transition state).
- `cyberos.proj.edit_comment(comment_id, body)` — `destructive: true; requires_confirmation: true`.
- `cyberos.proj.delete_comment(comment_id, reason)` — `destructive: true; requires_confirmation: true`.

Bulk:
- `cyberos.proj.bulk_transition(issue_ids, to_state, reason_md)` — `destructive: true; requires_confirmation: true; sensitivity: high; step_up_required: true` (bulk is high-sensitivity by definition).
- `cyberos.proj.bulk_assign(issue_ids, assignee_member_id)` — `destructive: true; requires_confirmation: true; sensitivity: high; step_up_required: true`.
- `cyberos.proj.bulk_move_to_cycle(issue_ids, cycle_id)` — `destructive: true; requires_confirmation: true; sensitivity: high; step_up_required: true`.

Drafts (no commit):
- `cyberos.proj.nlcrud_propose_issue(utterance, project_id?)` — `destructive: false; idempotent: true`. Returns proposed input + a `confirmation_token` valid for 5 minutes; the agent UI must collect human confirmation before calling commit.
- `cyberos.proj.nlcrud_commit_issue(confirmation_token)` — `destructive: true; requires_confirmation: true`. Calls `create_issue` server-side using the proposed input.

The propose-then-commit flow is the architectural floor for AI-driven mutations (matches FR-BRAIN-NLCRUD-001's pattern for memory). It enforces the property that even with a CUO scope contract that doesn't include `create_issue`, an agent can still draft an issue *for the human to commit* via the propose pathway.

**Annotation enforcement.**

Every annotation enforced by FR-MCP-001's gateway:

- `read_only: false` — applies to all mutation tools.
- `destructive: true` — applies to every mutation in the catalogue except `add_comment` and `nlcrud_propose_*`. Requires `client_confirmed: true`.
- `idempotent: true` on `add_blocker`, `remove_blocker`, `nlcrud_propose_*`; idempotency keys deduplicated for 24 h.
- `sensitivity: high` on Engagement-budget changes, project archive, cycle close, bulk operations, member-role changes.
- `step_up_required: true` cascades from `sensitivity: high`; FR-AUTH-003 mints the step-up token bound to the operation.
- `irreversible: true` is **not** set on any PROJ tool; the closest tools (`archive_*`) are recoverable by un-archive (administrative path).

**Persona scope contracts.**

CUO/COO scope (default for PROJ):
```
tools_allowed:
  - cyberos.proj.list_*
  - cyberos.proj.get_*
  - cyberos.proj.search
  - cyberos.proj.engagement_dashboard
  - cyberos.proj.daily_triage
  - cyberos.proj.draft_cycle_review
  - cyberos.proj.draft_status_update
  - cyberos.proj.list_calibration_drift
  - cyberos.proj.cross_project_insight
  - cyberos.proj.list_blocker_signals
  - cyberos.proj.add_comment           # CUO can post a Notify-like comment
  - cyberos.proj.nlcrud_propose_issue  # CUO can DRAFT an issue
  - cyberos.proj.nlcrud_propose_*      # all draft tools
tools_forbidden_explicit:
  - cyberos.proj.create_*
  - cyberos.proj.update_*
  - cyberos.proj.transition_*
  - cyberos.proj.assign_*
  - cyberos.proj.move_*
  - cyberos.proj.close_*
  - cyberos.proj.archive_*
  - cyberos.proj.bulk_*
  - cyberos.proj.nlcrud_commit_*
  - cyberos.proj.set_*
```

CUO/CTO and CUO/CPO have similar shapes (with read/draft access, no mutation). Future skills (CUO/CRO when CRM lands; CUO/CHRO for HR) extend per-skill.

**Audit + telemetry.** Every mutation tool call writes:
- The canonical audit row in scope `proj.{tenant}` via `audit.write` (FR-AUTH-002).
- A row in `proj.mutation_log` (FR-PROJ-002) so it goes through the same sync-engine path as a UI-driven mutation; subscribers receive the WebSocket fan-out identically. Agent-driven and human-driven mutations are visually indistinguishable to other clients (which is the agent-parity invariant in action) but are distinguishable in the audit log via `actor_kind: 'agent'`.
- A Prometheus counter `cyberos_mcp_proj_calls_total{tool, module, status, actor_kind}` for OBS dashboards.

**Tool descriptions for LLM consumption.**

Each tool ships with a structured description optimised for LLM tool-selection:
```yaml
- name: cyberos.proj.transition_issue
  description: |
    Move a PROJ issue to a new workflow state (e.g. "todo" -> "in_progress",
    "in_review" -> "done"). Respects per-project workflow rules (FR-PROJ-003);
    transitions like "done -> todo" require a reason and lead-role permission.
  parameters:
    id: {type: string, format: uuid, description: "issue ID or key (e.g. ALPHA-1234)"}
    to_state: {type: string, description: "target state key from the project workflow"}
    reason_md: {type: string, description: "reason for the transition; required when the workflow rule demands"}
  returns: {type: object, description: "the updated issue"}
  destructive: true
  requires_confirmation: true
```

The descriptions are loaded by FR-MCP-001's tool-registry endpoint and surfaced to MCP clients for tool discovery.

**Latency NFR.** Mutation tool call p99 ≤ 250 ms over a typical operation; bulk operations p95 ≤ 2 s for 100 issues.

## Alternatives Considered

- **Auto-generate tools from GraphQL mutations.** Rejected (also at FR-MCP-001): tool naming, annotation, and description quality matter; auto-generation produces noisy tools.
- **Skip the propose-then-commit pattern; let CUO call mutation tools directly.** Rejected: human-in-the-loop is the floor; the architectural property is enforced via the persona scope contract excluding mutation tools.
- **Single `cyberos.proj.execute(action, params)` tool.** Rejected: tool-name precision is what makes LLM tool-selection accurate.
- **Send unconfirmed mutations through with `destructive: true` only at the UI surface.** Rejected: the destructive flag is the architectural floor at the gateway; any UI bypass is a hole.
- **Skip step-up on bulk operations** (defer to client UI to confirm). Rejected: bulk is high-leverage and the step-up gate is the only architectural protection against runaway scripts.

## Success Metrics

- **Primary metric.** P1 sprint demo passes: (1) an MCP client (Claude.ai) calls `cyberos.proj.create_issue` and is rejected without `client_confirmed: true`; retries with confirmation and succeeds; (2) the same client calls `cyberos.proj.close_cycle` and is rejected without a step-up token; the human completes step-up; the call succeeds; (3) a CUO persona's `tools_forbidden_explicit` list rejects an attempted `cyberos.proj.transition_issue` call from the persona; (4) the `nlcrud_propose_issue` flow round-trips end-to-end.
- **Coverage metric.** 100% of GraphQL mutations in FR-PROJ-001..007 have an MCP tool counterpart with correct annotations.
- **Latency NFR.** Mutation call p99 ≤ 250 ms on the canonical synthetic workload.

## Scope

**In-scope.**
- The full mutation tool catalogue listed above.
- Annotation correctness + gateway enforcement.
- Propose-then-commit pair for issue creation.
- Persona scope contract updates for CUO/COO/CTO/CPO.
- Tool descriptions optimised for LLM consumption.
- Audit + telemetry integration.
- The eval suite for tool annotations: every tool has a regression case verifying the annotations are enforced at the gateway.

**Out-of-scope (deferred).**
- NLCRUD propose pairs for cycles, projects, engagements (P2; P1 ships only the issue propose pair as the canonical pattern).
- Tool versioning + deprecation flow (P2; for now `since_version` annotation is informational).
- Per-tool fine-grained per-Member permissions beyond RBAC role (P3).
- Customer-facing MCP for external integrations (P4 PORTAL).

## Dependencies

- FR-PROJ-001..007.
- FR-MCP-001 (gateway + annotation enforcement + persona-scope contracts).
- FR-AUTH-001 / FR-AUTH-003 (step-up auth).
- FR-AI-001 (the AI Gateway is the path through which CUO calls the propose tools).
- FR-OBS-001 / FR-OBS-002 (telemetry + dashboards).
- Compliance: EU AI Act Article 14 (human-in-the-loop on every destructive); GDPR Article 22 (no fully automated decisions on natural persons — agent-driven assignment is reversible by the human).
- Locked decisions referenced: DEC-119 (every PROJ mutation has an MCP equivalent), DEC-120 (CUO scope contract excludes commit; propose pattern is the bridge).

## AI Risk Assessment

The PROJ MCP surface is the substrate for agent-parity in PROJ. EU AI Act risk class: `limited`.

### Data Sources

The mutation surface itself does not consume training data; it routes structured calls to the GraphQL layer. Agents calling these tools are authenticated as Members with their own scopes; per-tenant residency.

### Human Oversight

- Every destructive mutation requires the human-confirmed token.
- High-sensitivity mutations require step-up auth (fresh passkey ceremony).
- CUO scope contracts exclude mutation tools by default; CUO must propose; the human commits.
- The audit log captures `actor_kind: 'agent'` and `actor_agent_client` so a forensic query distinguishes AI-driven mutations.

### Failure Modes

- **Agent client bypasses confirmation UI.** Mitigation: the gateway requires the explicit `client_confirmed` flag; bypass attempts are audit-logged; persistent abuse triggers manual review of the agent client's authorisation (FR-AUTH-003).
- **Agent calls forbidden tool.** Gateway rejects with `code: PERSONA_SCOPE_VIOLATION`; persona is auto-quarantined if violation rate exceeds threshold (FR-GENIE-002).
- **Step-up timeout.** The 5-minute token expires; the agent must request a fresh token; bulk operations with multiple step-ups require batching.
- **Bulk mutation creates inconsistent state.** Server-side transaction ensures atomicity; partial failure surfaces to the agent as a structured error.

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6.
- **Scope:** Drafted full mutation catalogue, annotation rules, persona scope contracts, propose-then-commit pattern, failure modes.
- **Human review:** `@stephen-cheng` reviewed; the destructive / sensitivity / step-up matrix to be re-verified by the Engineering Lead at PR-review.
