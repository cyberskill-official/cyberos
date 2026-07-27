---
id: TASK-IMP-147
title: "Ingest inspect-harden — four skills, /inspect+/harden, conductor clarity"
template: task@1
type: improvement
module: improvement
status: done
priority: p1
author: "@stephencheng"
department: engineering
created_at: 2026-07-27T00:00:00+00:00
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-IMP-110, TASK-IMP-111, TASK-SKILL-118]
routed_back_count: 0
awh: N/A
verify: T
phase: "inspect-harden-ingest"
owner: Stephen Cheng (CTO)
created: 2026-07-27
effort_hours: 12
service: modules/skill
new_files:
  - modules/skill/inspection-report-author/
  - modules/skill/inspection-report-audit/
  - modules/skill/harden-record-author/
  - modules/skill/harden-record-audit/
  - modules/skill/INSPECT-HARDEN.md
  - tools/install/plugin/commands/inspect.md
  - tools/install/plugin/commands/harden.md
modified_files:
  - tools/install/build.sh
  - tools/install/check-pair-parity.sh
  - tools/install/install.sh
  - tools/install/tests/test_full_sdp_payload.sh
  - tools/install/tests/test_channels.sh
  - tools/install/tests/test_cli_cuo_verb.sh
  - tools/install/cli/bin/cli.mjs
  - tools/install/plugin/commands/help.md
  - tools/install/plugin/skills/ship-tasks/SKILL.md
  - modules/cuo/chief-technology-officer/workflows/ship-tasks.md
  - docs/tasks/BACKLOG.md
source_pages:
  - "/Users/stephencheng/Downloads/inspect-harden/GUIDELINE.md §7 pick-up items"
  - "tools/install/build.sh VENDORED_SKILLS"
  - "tools/install/check-pair-parity.sh SCOPE"
source_decisions:
  - "2026-07-27 approved plan ingest_inspect_harden_f0cd28fa — four skills + distribution + conductor clarity; /harden stays distinct from ship-tasks class:improvement"
---

# TASK-IMP-147: Ingest inspect-harden

## Summary

Bring the inspect-harden package into CyberOS as four first-class skills
(`inspection-report-author/audit`, `harden-record-author/audit`), fix the known
`harden-plan.mjs` `operator_prerequisites` defect, wire `/inspect` and `/harden`
for Claude plugin and native shared skills, and clarify that ship-tasks
"harden a task" (`class: improvement`) is not `/harden` (inspection remediation).

## Problem

Inspection → remediation lived as an external package. CyberOS had no vendored
skills, no slash commands, and conductor language that overloaded "harden" with
improvement-class shipping. The planner also misclassified findings that declare
`operator_prerequisites` (shopass INS-F-0002).

## Proposed Solution

1. Ingest four skill trees under `modules/skill/` to the author/audit floor.
2. Align ledger contract for spec ≥1.2 to 75 disciplines; keep 1.0 goldens valid.
3. Fix `harden-plan.mjs` + extend `harden-plan-check.sh`.
4. Scaffold `harden-record-audit` (RUBRIC / AUDIT_LOOP / REPORT_FORMAT / triggers).
5. Vendor + pair-parity SCOPE + census 52→56; plugin commands; SHARED_CMDS; CLI stubs.
6. Conductor docs: ship-tasks §1a "Not `/harden`" note; plugin skill + help.

## Non-goals

- Applying shopass patch to a live target.
- MCP tools for inspect/harden.
- Merging `/harden` into `/ship-tasks`.
- Committing package-root `reports/` / `evidence/` as runtime.

## Success Metrics

- `inspect-lint --selftest` green; all acceptance goldens lint clean.
- `harden-plan-check.sh` PASS including shopass INS-F-0002 non-agent.
- `build.sh` + pair-parity + channel tests green; census 56.
- Task remains non-`done` until human HITL.
