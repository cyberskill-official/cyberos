---
# ── Identity ─────────────────────────────────────────────────────────
name: backlog-state-update-audit
description: >-
  Audit a backlog-state-update@2 (accepting @1 for one release window) against backlog_state_update_rubric@2.0: enforces new_status in the 10-value enum, line_number/old_line optimistic concurrency for status-cell mutations, the BSU-INS family for insert-row mutations (uniqueness pre-image gate, regenerator-identical grammar and placement, whole-file discipline, status equals FR frontmatter), evidence cross-references, and the closed mutation_kind enum. On 10/10, emits the `workflow_complete` memory audit row as the workflow's terminal artefact. Use when user asks to "audit this backlog state update" or "check the backlog state update". Do NOT use for "draft a new backlog state update" (use backlog-state-update-author instead).
license: Apache-2.0
metadata:
  version: 2.0.0
  module: skill
  stage: e
  cyberos-template: backlog-state-update-audit@2
  cyberos-rubric-target: backlog_state_update_rubric@2.0

allowed_memory_scopes:
  read:
    - project:*
  write:
    - project:fr/{fr_id}/backlog-state-update.audit
    - project:workflow/{run_id}/complete

audit:
  row_kind: backlog_state_update_audited
  required_fields: [fr_id, score, issues_open, issues_resolved, workflow_complete_emitted]

inputs:
  - { name: backlog_mutation, format: backlog-state-update@2 (or @1, transition window), required: true }
outputs:
  - { name: audit_report,       format: backlog-state-update-audit@1 }
  - { name: workflow_complete,  format: memory-audit-row@1 }
---

# backlog-state-update-audit

## 1. Rubric (backlog_state_update_rubric@2.0)

Common rules (every mutation):

| Rule ID | Check | Severity |
|---|---|---|
| BSU-001 | `new_status` (or `insert.status`) ∈ the 10-value lifecycle enum of STATUS-REFERENCE.md §1 (draft, ready_to_implement, implementing, ready_to_review, reviewing, ready_to_test, testing, done, on_hold, closed). Fixed in @2.0: the @1.0 table still cited the RETIRED pre-enum vocabulary. | error |
| BSU-004 | Every `evidence_artefact_ids` entry cross-references a real memory audit row from the same run | error |
| BSU-005 | `mutation_kind` ∈ {status-cell-only, insert-row}; never row reorder / FR delete / multi-line edit | error |
| BSU-006 | The matching memory row (`workflow_complete` / `workflow_phase_complete` / `fr_routed_back`) emitted as a side effect of passing | error |

status-cell-only rules:

| BSU-002 | `line_number` resolves to a real BACKLOG row whose fr_id matches | error |
| BSU-003 | `old_line` matches the current file contents byte-for-byte (optimistic concurrency) | error |

insert-row rules (FR-CUO-205):

| BSU-INS-001 | No row for `fr_id` in the pre-image; exactly one in the post-image | error |
| BSU-INS-002 | Row grammar exact: `- [<status>] <FR-ID-slug> - <title>` + ` (improvement)` suffix iff class: improvement | error |
| BSU-INS-003 | Placed in the correct `## <module>` section (created per regenerator conventions when absent), sort order kept | error |
| BSU-INS-004 | No other line of the file changed, except that section's header counts when present | error |
| BSU-INS-005 | `insert.status` equals the FR file's frontmatter status at write time | error |

Transition window: a @1 artefact (no mutation_kind) audits as status-cell-only with a
transition note, not a failure; the window closes one release after FR-CUO-205 ships.

## 2. Pass criterion

10/10. The `workflow_complete` memory row is the workflow's terminal
artefact — supervisors that watch the memory chain use this row to
detect "FR drained from queue, move on".

---

*End of backlog-state-update-audit SKILL.md.*

## Contract files (FR-SKILL-118)

This pair is at full contract parity: `RUBRIC.md` (versioned rules + prose->rule map), `AUDIT_LOOP.md` (canonical-loop binding), `REPORT_FORMAT.md`, `envelopes/` (I/O schemas), `acceptance/README.md`. SKILL.md remains the normative prose; the files encode it.
