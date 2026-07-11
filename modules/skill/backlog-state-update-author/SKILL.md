---
# ── Identity ─────────────────────────────────────────────────────────
name: backlog-state-update-author
description: >-
  Compute the new status cell for an FR in `docs/feature-requests/BACKLOG.md` and rewrite that single cell atomically (one tracked mutation; the same write that emits the `workflow_complete` memory audit row). Status derives from the `ship-feature-requests` step that just completed, constrained to the 10-value lifecycle enum in `docs/feature-requests/STATUS-REFERENCE.md` §1. Failures and blockers route the FR back to `ready_to_implement` (§1.3), incrementing `routed_back_count`. Used by `chief-technology-officer/ship-feature-requests` between every workflow phase. Use when user asks to "draft a backlog state update" or "create the backlog state update". Do NOT use for "audit existing backlog state update" (use backlog-state-update-audit instead). HITL note — operators can override any cell to any other cell at any time; this skill only writes the default workflow-driven transition (§1.4).
license: Apache-2.0
metadata:
  version: 2.0.0
  module: skill
  stage: e
  cyberos-template: backlog-state-update@1
  cyberos-rubric-target: backlog_state_update_rubric@1.0

# ── Scope contract (memory/AGENTS.md §15) ────────────────────────────
allowed_memory_scopes:
  read:
    - project:*
  write:
    - project:fr/{fr_id}/backlog-state-update
    - project:backlog/{fr_id}
audit:
  row_kind: backlog_state_update_authored
  required_fields: [fr_id, prior_status, new_status, line_number, evidence_artefact_ids]

# ── Inputs / outputs ─────────────────────────────────────────────────
inputs:
  - { name: fr,      format: feature-request@1,                 required: true }
  - { name: outcome, format: workflow-step-outcome-bundle@1,    required: true }
outputs:
  - { name: backlog_mutation, format: backlog-state-update@1 }

# ── Triggers / blockers ──────────────────────────────────────────────
triggers:
  - workflow `chief-technology-officer/ship-feature-requests` — invoked between every phase transition
blockers:
  - "BACKLOG.md is locked by another concurrent workflow — wait for lock"
  - "BACKLOG.md has divergent uncommitted changes — escalate to operator"
---

# backlog-state-update-author

## 1. Purpose

The BACKLOG is the **single source of truth** for FR state. This skill is the only authorised writer; every other step in the workflow emits artefacts, but this is the step that flips the status cell. The mutation is atomic with the `workflow_complete` memory row, so the chain and the state file can never disagree.

This skill is called between **every phase transition** of `ship-feature-requests`, not just on terminal outcomes:

- `ready_to_implement → implementing` (workflow start)
- `implementing → ready_to_review` (build complete)
- `ready_to_review → reviewing` (reviewer claims)
- `reviewing → ready_to_test` (review approved)
- `ready_to_test → testing` (tester claims)
- `testing → done` (coverage-gate-audit passes)
- any-stage → `ready_to_implement` (failure/blocker rework path, increments `routed_back_count`)

## 2. Output schema

```yaml
# backlog-state-update@1
fr_id: FR-<MODULE>-<NNN>
generated_at: <ISO-8601>
backlog_path: docs/feature-requests/BACKLOG.md
prior_status: <one of the 10 enum values from STATUS-REFERENCE.md §1>
new_status: <one of the 10 enum values from STATUS-REFERENCE.md §1>
transition_kind: forward | rework | off_ramp
routed_back_count_delta: 0 | 1   # 1 only when transition_kind == "rework"
line_number: <int — the BACKLOG.md line being mutated>
old_line: "<full text of the line being replaced>"
new_line: "<full text of the replacement line>"
evidence_artefact_ids:
  # references vary by transition_kind; populated from the workflow step's outcome bundle
  context_map: "<artefact id | null>"
  adr: "<artefact id | null>"
  edge_case_matrix: "<artefact id | null>"
  mock_contract: "<artefact id | null>"
  impl_plan: "<artefact id | null>"
  obs_injection: "<artefact id | null>"
  coverage_report: "<artefact id | null>"
  debug_trace: "<artefact id | null>"
  feature_request_audit: "<artefact id | null>"   # populated on draft → ready_to_implement
  coverage_gate_audit: "<artefact id | null>"     # populated on testing → done
rework_reason: "<one-sentence reason | null>"      # required when transition_kind == "rework"
mutation_kind: status-cell-only   # never multi-line; one cell per FR
memory_emit:
  row_kind: workflow_complete | workflow_phase_complete | fr_routed_back
  fr_id: FR-<MODULE>-<NNN>
  outcome_summary: "<one-paragraph human-readable summary>"
```

## 3. Quality gates

- `new_status` is one of the 10 enum values listed in `docs/feature-requests/STATUS-REFERENCE.md` §1 — never freeform, never with embedded modifiers like `+ strict-audited`.
- `transition_kind` MUST match the direction of the status change:
  - `forward` — moving down the §1.1 lifecycle (e.g. `implementing → ready_to_review`)
  - `rework` — moving back to `ready_to_implement` from any downstream state; increments `routed_back_count`; `rework_reason` is required
  - `off_ramp` — moving to `on_hold` or `closed` from any state
- `line_number` resolves to a real BACKLOG row whose `fr_id` matches.
- `old_line` matches the current contents of that line byte-for-byte (optimistic concurrency check; if the file shifted underneath us, refuse and re-enter the queue).
- `evidence_artefact_ids` references real memory audit rows from the same workflow run (cross-reference check). Which fields are required depends on the transition — e.g. `coverage_gate_audit` is required for the `testing → done` transition; `feature_request_audit` is required for `draft → ready_to_implement`.
- `mutation_kind == status-cell-only` — this skill never moves rows, reorders the queue, or deletes FRs.
- The `memory_emit.row_kind` MUST be one of: `workflow_phase_complete` (intra-lifecycle forward transition), `workflow_complete` (only when `new_status == "done"`), or `fr_routed_back` (when `transition_kind == "rework"`).

## 4. HITL bypass

This skill writes ONLY the default workflow-driven transition. Per STATUS-REFERENCE.md §1.4, an operator can override any cell to any other cell at any time — those overrides do NOT route through this skill. They are recorded by a separate `memory.status_overridden` aux row emitted directly by the BACKLOG editor (CLI or web UI), bypassing the workflow entirely.

If the operator's override leaves the BACKLOG in a state that the next workflow phase doesn't expect (e.g. `done` reset to `ready_to_review`), the workflow detects the mismatch on resume and either picks up from the operator-set status or halts with a clear "operator override detected — resume?" prompt. There is no machine-enforced transition restriction.

## 5. Chains to

`backlog-state-update-audit` — the only successor. The audit emits the `workflow_complete` / `workflow_phase_complete` / `fr_routed_back` memory row as a side effect of passing.

---

*End of backlog-state-update-author SKILL.md.*
