---
# ── Identity ─────────────────────────────────────────────────────────
name: coverage-gate-audit
description: >-
  Test coverage gate audit (testing → done) — audit a coverage-gate@1 artefact against coverage_gate_rubric@1.0: enforces tests_failed==0, files_below_90pct empty, ecm_rows_uncovered empty, raw_terminal present + non-truncated, AND every §1 clause's cited test from the FR is `passed` in the coverage report (TRACE-004 closure). Emits a `score / 10` verdict; refuses to pass on <10/10. The pass certifies the `testing → done` lifecycle transition per `modules/skill/contracts/feature-request/STATUS-REFERENCE.md` §1.1. Use when user asks to "audit this coverage gate" or "check the coverage gate". Do NOT use for "draft a new coverage gate" (use coverage-gate-author instead). Do NOT use for spec correctness (that is `feature-request-audit`'s job during the `draft → ready_to_implement` transition).
license: Apache-2.0
metadata:
  version: 1.0.0
  module: skill
  stage: e
  cyberos-template: coverage-gate-audit@1
  cyberos-rubric-target: coverage_gate_rubric@1.0

allowed_memory_scopes:
  read:
    - project:*
  write:
    - project:fr/{fr_id}/coverage-gate.audit

audit:
  row_kind: coverage_gate_audited
  required_fields: [fr_id, score, issues_open, issues_resolved]

inputs:
  - { name: report, format: coverage-gate@1, required: true }
outputs:
  - { name: audit_report, format: coverage-gate-audit@1 }
---

# coverage-gate-audit

## 1. Rubric (coverage_gate_rubric@1.0)

| Rule ID | Check | Weight | Severity if failed |
|---|---|---|---|
| CG-001 | `tests_failed == 0` | 30% | error |
| CG-002 | `files_below_90pct` is empty | 30% | error |
| CG-003 | `ecm_rows_uncovered` is empty | 20% | error |
| CG-004 | `raw_terminal` is present + > 100 chars | 10% | warning |
| CG-005 | `overall_coverage_pct ≥ 80` (project-level) | 10% | warning |

## 2. Pass criterion

10/10. Errors → return to coverage-gate-author with the failing rows;
the workflow then proceeds to the debugging-cycle skill (step 15).

---

*End of coverage-gate-audit SKILL.md.*

## Contract files (FR-SKILL-118)

This pair is at full contract parity: `RUBRIC.md` (versioned rules + prose->rule map), `AUDIT_LOOP.md` (canonical-loop binding), `REPORT_FORMAT.md`, `envelopes/` (I/O schemas), `acceptance/README.md`. SKILL.md remains the normative prose; the files encode it.
