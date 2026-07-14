---
# ── Identity ─────────────────────────────────────────────────────────
name: edge-case-matrix-audit
description: >-
  Audit an edge-case-matrix@1 against edge_case_matrix_rubric@1.0: enforces ≥1 row per category, SECURITY rows pointing at real test paths, DEGRADATION rows specifying detection + recovery, and `total_rows ≥ 8` for MUST-priority FRs. Emits a `score / 10` verdict + an itemised findings list; refuses to pass the chain on <10/10. Use when user asks to "audit this edge case matrix" or "check the edge case matrix". Do NOT use for "draft a new edge case matrix" (use edge-case-matrix-author instead).
license: Apache-2.0
metadata:
  version: 1.0.0
  module: skill
  stage: e
  cyberos-template: edge-case-matrix-audit@1
  cyberos-rubric-target: edge_case_matrix_rubric@1.0

allowed_memory_scopes:
  read:
    - project:*
  write:
    - project:fr/{task_id}/edge-case-matrix.audit

audit:
  row_kind: edge_case_matrix_audited
  required_fields: [task_id, score, issues_open, issues_resolved]

inputs:
  - { name: matrix, format: edge-case-matrix@1, required: true }
outputs:
  - { name: audit_report, format: edge-case-matrix-audit@1 }
---

# edge-case-matrix-audit

## 1. Rubric (edge_case_matrix_rubric@1.0)

| Rule ID | Check | Weight | Severity if failed |
|---|---|---|---|
| EC-001 | Every category (NULL_INPUT, BOUNDARY, MALFORMED, CONCURRENT, SECURITY, DEGRADATION) has ≥1 row | 30% | error |
| EC-002 | Every SECURITY row has a `planned_test` pointing at a real file | 25% | error |
| EC-003 | Every DEGRADATION row specifies both detection AND recovery | 15% | error |
| EC-004 | For MUST-priority FRs: `total_rows ≥ 8` | 10% | error |
| EC-005 | No row has a vacuous `expected` ("works correctly" etc.) | 10% | warning |
| EC-006 | Severity field is one of `critical|high|medium|low` | 10% | error |

## 2. Pass criterion

10/10 only. Any error-class miss → fail (return matrix to author with
fix list).

---

*End of edge-case-matrix-audit SKILL.md.*

## Contract files (TASK-SKILL-118)

This pair is at full contract parity: `RUBRIC.md` (versioned rules + prose->rule map), `AUDIT_LOOP.md` (canonical-loop binding), `REPORT_FORMAT.md`, `envelopes/` (I/O schemas), `acceptance/README.md`. SKILL.md remains the normative prose; the files encode it.
