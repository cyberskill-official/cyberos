---
id: TASK-DOCS-019
title: "Installer: checker in payload lib, config scaffold, CI template"
eu_ai_act_risk_class: not_ai
ai_authorship: generated_then_reviewed
client_visible: false
type: improvement
class: improvement
created_at: 2026-07-27T00:35:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: docs
priority: p0
status: done
# Operator closeout 2026-07-27 (Stephen): work already on main; HITL override → done
verify: T
phase: Status-v3 Phase 4
owner: Stephen Cheng (CTO)
created: 2026-07-27
shipped: null
memory_chain_hash: null
related_tasks: [TASK-DOCS-009, TASK-DOCS-022, TASK-DOCS-023, TASK-DOCS-028]
depends_on: [TASK-DOCS-009]
blocks: [TASK-DOCS-022, TASK-DOCS-023, TASK-DOCS-028]
source_pages:
  - docs/plans/PLAN-status-v3-platform-2026-07-27/plan.md
source_decisions:
  - "2026-07-27 operator instruction: execute PLAN-status-v3-platform-2026-07-27"
language: javascript + bash + yaml + html/css
service: tools/docs-site/
new_files:
  - (none)
modified_files:
  - (see phase plan)
effort_hours: 4
---

# TASK-DOCS-019: Installer: checker in payload lib, config scaffold, CI template

## Summary

Deliver plan item T12 (Phase 4) from `docs/plans/PLAN-status-v3-platform-2026-07-27/plan.md`: fleet-install-test green; fresh install carries hook+config+(optional) CI template.

## Problem

Status v3 is still a session preview and enforcement is not yet the platform standard. Without this task, the mothership and consumer fleet keep the v2 page and cannot enforce THE RULE consistently.

## Proposed Solution

Execute Phase 4 for T12 exactly as specified in the plan, preserving non-negotiables: THE RULE, no history rewrite, determinism, offline-first, consumer sovereignty. Every commit cites `TASK-DOCS-019`.

## Alternatives Considered

- Keep a second extractor beside render-status-hub — rejected by plan (fold into one generator).
- Rewrite git history to fix links — rejected; ledger-only retro fixes.
- Auto-set `done` on green gates — rejected; HITL remains mandatory.

## Success Metrics

- Primary: acceptance core is true — fleet-install-test green; fresh install carries hook+config+(optional) CI template.
- Guardrail: no hand-edited derived payload copies; pair/parity and payload hooks remain the mirror path.

## Scope

In: plan §7 T12 and its Phase 4 steps.
Out: docs/status-v2-preview; silently choosing `traceability.scaffold_ci` default (P0 open question); push/merge/tag without operator instruction.

## Dependencies

Upstream: TASK-DOCS-009.
Downstream: TASK-DOCS-022, TASK-DOCS-023, TASK-DOCS-028.
Source plan: docs/plans/PLAN-status-v3-platform-2026-07-27/plan.md.

## 1. Description

- 1.1 This task MUST deliver its plan §7 item while preserving THE RULE, no history rewrite, determinism, offline-first, and consumer sovereignty.
- 1.2 Acceptance core MUST be met as stated in Summary.
- 1.3 Every commit for this task MUST cite its canonical TASK id.
- 1.4 Green machine gates are necessary and never sufficient; HITL gates remain mandatory.
- 1.5 Derived payload copies MUST NOT be hand-edited.

## Acceptance Criteria

- [ ] AC 1 (traces_to: #1.1, #1.2) — plan item acceptance core is demonstrably met for this task's Phase gate - verify: plan Phase gate checklist + recorded evidence in the ship packet
- [ ] AC 2 (traces_to: #1.3) — every commit for this task cites its canonical TASK id in subject or body - test: `bash scripts/check_task_link.sh --range <base>..<head>`
- [ ] AC 3 (traces_to: #1.4) — review and final acceptance HITL gates are not self-crossed; agent halts with evidence - verify: STATUS-REFERENCE §1.4 / ship-tasks halt
- [ ] AC 4 (traces_to: #1.5) — no hand-edited derived payload copies land in the PR - test: `bash tools/install/check-pair-parity.sh` when generator/docs-tools touched

## Test plan

1. `bash scripts/tests/run_all.sh` when scripts/hooks touched
2. docs-site / DOM / fleet tests named by the phase plan when those surfaces are touched
3. `bash .cyberos/cuo/gates/run-gates.sh` before the review packet

## AI Authorship Disclosure

- **Tools used:** Cursor agent (GPT-5.6 Sol) drafting from the approved plan.
- **Scope:**
  **re-derived and CONFIRMED:** plan §7 item ids T1–T22 and Phase 0–8 sequencing are taken from `docs/plans/PLAN-status-v3-platform-2026-07-27/plan.md` as read in this worktree.
  **re-derived and CORRECTED:** none — no numeric or citation claim from an earlier draft was corrected in this rewrite.
  **measured and ADDED:** task ids TASK-DOCS-008..029 allocated via `backlog-mutate next-id docs` at mint time; audit figures were treated as claims to verify, not facts to copy.
- **Human review:** Operator approved the plan and reviews at CyberOS HITL gates.

*End of TASK-DOCS-019.*
