---
# ── Identity ─────────────────────────────────────────────────────────
name: backlog-state-update-author
description: |
  Compute the new status row for an FR in `docs/feature-requests/BACKLOG.md`
  and physically rewrite that single row atomically (one tracked mutation;
  same write that emits the workflow_complete BRAIN audit row). Status is
  derived from steps 1–16 outcome: `shipped + strict-audited`,
  `shipped + mocked-dependency`, `[FAILED: UNRESOLVABLE ERROR]`, or
  `[BLOCKED: {reason}]`. Used by chief-technology-officer/implement-backlog-frs
  as step 17.
license: Apache-2.0
metadata:
  version: 1.0.0
  module: skill
  stage: e
  cyberos-template: backlog-state-update@1
  cyberos-rubric-target: backlog_state_update_rubric@1.0

# ── Scope contract (memory/AGENTS.md §15) ────────────────────────────
allowed_brain_scopes:
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
  - workflow `chief-technology-officer/implement-backlog-frs` step 17
blockers:
  - "BACKLOG.md is locked by another concurrent workflow — wait for lock"
  - "BACKLOG.md has divergent uncommitted changes — escalate to operator"
---

# backlog-state-update-author

## 1. Purpose

The BACKLOG is the **single source of truth** for FR state. This skill
is the only authorised writer; every other step in the workflow emits
artefacts, but this is the step that flips the status cell. The mutation
is atomic with the workflow_complete BRAIN row, so the chain and the
state file can never disagree.

## 2. Output schema

```yaml
# backlog-state-update@1
fr_id: FR-<MODULE>-<NNN>
generated_at: <ISO-8601>
backlog_path: docs/feature-requests/BACKLOG.md
prior_status: "accepted | building | ..."
new_status: |
  "shipped + strict-audited"
  | "shipped + mocked-dependency"
  | "[FAILED: UNRESOLVABLE ERROR]"
  | "[BLOCKED: <one-sentence reason>]"
line_number: <int — the BACKLOG.md line being mutated>
old_line: "<full text of the line being replaced>"
new_line: "<full text of the replacement line>"
evidence_artefact_ids:
  context_map: "<artefact id>"
  adr: "<artefact id | null>"
  edge_case_matrix: "<artefact id>"
  mock_contract: "<artefact id | null>"
  impl_plan: "<artefact id>"
  obs_injection: "<artefact id>"
  coverage_report: "<artefact id>"
  debug_trace: "<artefact id | null>"
mutation_kind: status-cell-only   # never multi-line; one cell per FR
brain_emit:
  row_kind: workflow_complete
  fr_id: FR-<MODULE>-<NNN>
  outcome_summary: "<one-paragraph human-readable summary>"
```

## 3. Quality gates

- `new_status` is one of the four enum values — never freeform.
- `line_number` resolves to a real row whose `fr_id` matches.
- `old_line` matches the current contents of that line byte-for-byte
  (optimistic concurrency check; if the file shifted underneath us,
  refuse and re-enter the queue).
- `evidence_artefact_ids` references real BRAIN audit rows from the
  same workflow run (cross-reference check).
- `mutation_kind == status-cell-only` — this skill never moves rows,
  reorders the queue, or deletes FRs.

## 4. Chains to

`backlog-state-update-audit` — the only successor. The audit emits the
`workflow_complete` BRAIN row as a side effect of passing.

---

*End of backlog-state-update-author SKILL.md.*
