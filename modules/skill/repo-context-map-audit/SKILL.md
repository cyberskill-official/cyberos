---
# ── Identity ─────────────────────────────────────────────────────────
name: repo-context-map-audit
description: >-
  Audit a repo-context-map@1 against repo_context_map_rubric@1.0: enforces presence of the three baseline patterns (error_type, logging, test_framework), `pinned_in` references that resolve to real files, schemas present when the task declares migrations, and the module-placement warning either null or escalated. Emits a `score / 10` verdict; refuses to pass on <10/10. Use when user asks to "audit this repo context map" or "check the repo context map". Do NOT use for "draft a new repo context map" (use repo-context-map-author instead).
license: Apache-2.0
metadata:
  version: 1.0.0
  module: skill
  stage: e
  cyberos-template: repo-context-map-audit@1
  cyberos-rubric-target: repo_context_map_rubric@1.0

allowed_memory_scopes:
  read:
    - project:*
  write:
    - project:task/{task_id}/repo-context-map.audit

audit:
  row_kind: repo_context_map_audited
  required_fields: [task_id, score, issues_open, issues_resolved]

inputs:
  - { name: context_map, format: repo-context-map@1, required: true }
outputs:
  - { name: audit_report, format: repo-context-map-audit@1 }
---

# repo-context-map-audit

## 1. Rubric (repo_context_map_rubric@1.0)

| Rule ID | Check | Weight | Severity if failed |
|---|---|---|---|
| RCM-001 | `existing_patterns` includes all of: error_type, logging, test_framework | 25% | error |
| RCM-002 | Every `pinned_in` reference resolves to a real file (no dangling pointers) | 20% | error |
| RCM-003 | `schemas` is non-empty when the task declares migrations or a `data:` frontmatter block | 15% | error |
| RCM-004 | `files_outside_immediate_domain` entries each have a non-vacuous `reason` | 10% | warning |
| RCM-005 | `blast_radius.score` is a number in 0–100 and consistent with the file counts | 10% | warning |
| RCM-006 | If `files_outside_immediate_domain.length > 3`, the workflow ADR-branch flag is set | 10% | error |
| RCM-007 | `module_placement_warning` is null OR an escalation row was emitted | 10% | error |

## 2. Pass criterion

10/10 only. Any error-class miss returns the map to the author with a
fix list. The workflow only proceeds to step 3 (ADR branch) after this
audit passes.

---

*End of repo-context-map-audit SKILL.md.*

## Contract files (TASK-SKILL-118)

This pair is at full contract parity: `RUBRIC.md` (versioned rules + prose->rule map), `AUDIT_LOOP.md` (canonical-loop binding), `REPORT_FORMAT.md`, `envelopes/` (I/O schemas), `acceptance/README.md`. SKILL.md remains the normative prose; the files encode it.
