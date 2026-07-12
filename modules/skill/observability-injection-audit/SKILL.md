---
# ── Identity ─────────────────────────────────────────────────────────
name: observability-injection-audit
description: >-
  Audit an observability-injection@1 against observability_injection_rubric@1.0: enforces ≥1 log_point per state transition, ≥1 trace_span per external IO, ≥1 error_counter per error branch, branch_coverage ≥80%, redaction_policy present when PII is in scope. Emits a `score / 10` verdict; refuses to pass on <10/10. Use when user asks to "audit this observability injection" or "check the observability injection". Do NOT use for "draft a new observability injection" (use observability-injection-author instead).
license: Apache-2.0
metadata:
  version: 1.0.0
  module: skill
  stage: e
  cyberos-template: observability-injection-audit@1
  cyberos-rubric-target: observability_injection_rubric@1.0

allowed_memory_scopes:
  read:
    - project:*
  write:
    - project:fr/{fr_id}/observability-injection.audit

audit:
  row_kind: observability_injection_audited
  required_fields: [fr_id, score, issues_open, issues_resolved]

inputs:
  - { name: obs_injection, format: observability-injection@1, required: true }
outputs:
  - { name: audit_report, format: observability-injection-audit@1 }
---

# observability-injection-audit

## 1. Rubric (observability_injection_rubric@1.0)

| Rule ID | Check | Weight | Severity if failed |
|---|---|---|---|
| OBS-001 | Every state transition in impl_plan has ≥1 corresponding log_point | 25% | error |
| OBS-002 | Every external IO in impl_plan has ≥1 trace_span (wraps + attributes set) | 20% | error |
| OBS-003 | Every error branch in impl_plan has ≥1 error_counter | 20% | error |
| OBS-004 | `branch_coverage.coverage_pct ≥ 80` | 15% | error |
| OBS-005 | If FR touches PII, `redaction_policy` is non-empty + covers obvious fields | 10% | error |
| OBS-006 | `subscriber` matches the project's configured subscriber (no rogue libs) | 10% | warning |

## 2. Pass criterion

10/10. The workflow proceeds to step 13 (coverage-gate-author) once
this audit passes.

---

*End of observability-injection-audit SKILL.md.*

## Contract files (FR-SKILL-118)

This pair is at full contract parity: `RUBRIC.md` (versioned rules + prose->rule map), `AUDIT_LOOP.md` (canonical-loop binding), `REPORT_FORMAT.md`, `envelopes/` (I/O schemas), `acceptance/README.md`. SKILL.md remains the normative prose; the files encode it.
