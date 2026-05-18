---
# ── Identity ─────────────────────────────────────────────────────────
name: backlog-state-update-audit
description: |
  Audit a backlog-state-update@1 against backlog_state_update_rubric@1.0:
  enforces new_status is in the closed enum, line_number resolves to a
  real FR row, old_line matches byte-for-byte (optimistic concurrency),
  evidence_artefact_ids cross-reference real BRAIN rows, and
  mutation_kind == status-cell-only. On 10/10, emits the
  `workflow_complete` BRAIN audit row as the workflow's terminal
  artefact.
license: Apache-2.0
metadata:
  version: 1.0.0
  module: skill
  stage: e
  cyberos-template: backlog-state-update-audit@1
  cyberos-rubric-target: backlog_state_update_rubric@1.0

allowed_brain_scopes:
  read:
    - project:*
  write:
    - project:fr/<fr_id>/backlog-state-update.audit
    - project:workflow/<run_id>/complete

audit:
  row_kind: backlog_state_update_audited
  required_fields: [fr_id, score, issues_open, issues_resolved, workflow_complete_emitted]

inputs:
  - { name: backlog_mutation, format: backlog-state-update@1, required: true }
outputs:
  - { name: audit_report,       format: backlog-state-update-audit@1 }
  - { name: workflow_complete,  format: brain-audit-row@1 }
---

# backlog-state-update-audit

## 1. Rubric (backlog_state_update_rubric@1.0)

| Rule ID | Check | Weight | Severity if failed |
|---|---|---|---|
| BSU-001 | `new_status` ∈ {shipped + strict-audited, shipped + mocked-dependency, [FAILED: UNRESOLVABLE ERROR], [BLOCKED: ...]} | 25% | error |
| BSU-002 | `line_number` resolves to a real BACKLOG row whose fr_id matches | 20% | error |
| BSU-003 | `old_line` matches the current file contents byte-for-byte (optimistic concurrency) | 20% | error |
| BSU-004 | Every `evidence_artefact_ids` entry cross-references a real BRAIN audit row from the same run | 15% | error |
| BSU-005 | `mutation_kind == status-cell-only` (no row reorder / no FR delete / no multi-line edit) | 10% | error |
| BSU-006 | `workflow_complete` BRAIN row was emitted as a side effect of passing this audit | 10% | error |

## 2. Pass criterion

10/10. The `workflow_complete` BRAIN row is the workflow's terminal
artefact — supervisors that watch the BRAIN chain use this row to
detect "FR drained from queue, move on".

---

*End of backlog-state-update-audit SKILL.md.*
