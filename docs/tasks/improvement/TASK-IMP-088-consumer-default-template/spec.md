---
id: TASK-IMP-088
title: install scaffolds task_template task@1 in consumer config.yaml
template: task@1
type: improvement
module: improvement
status: testing
priority: p1
author: "@stephencheng"
department: engineering
created_at: 2026-07-16T17:25:00Z
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-CUO-207, TASK-CUO-208]
routed_back_count: 0
awh: N/A
verify: T
phase: "pre-1.0.0 release"
owner: Stephen Cheng (CTO)
created: 2026-07-16
shipped: null
memory_chain_hash: null
effort_hours: 2
service: tools/install
new_files: []
modified_files:
  - tools/install/install.sh
  - tools/install/tests/test_install_hygiene.sh
source_pages:
  - "tools/install/install.sh step 3b (config.yaml scaffold, commented task_template: engineering-spec@1)"
  - "operator decision 2026-07-16 (batch-2 PLAN gate, IMP-06): scaffold task@1 uncommented on consumer installs; platform repo untouched; resolution chain itself unchanged"
  - "IMPROVEMENT_HANDOFF.md IMP-06 (sachviet run: fresh repo followed the printed task@1 instructions under an engineering-spec default; overridden at the PLAN gate)"
source_decisions:
  - "2026-07-16 Stephen: IMP-06 option (a) chosen; batch 3 PLAN approved."
---

# TASK-IMP-088: install scaffolds task_template task@1 in consumer config.yaml

## Summary

Everything the installer vendors into a consumer repo is task@1 (templates, rubric families, the printed next steps), yet the task-author resolution chain defaults to engineering-spec@1 when config.yaml is silent. Per the recorded IMP-06 decision, the config.yaml scaffold on consumer installs now writes `task_template: task@1` uncommented, so a fresh repo's first authoring run resolves the profile its vendored materials assume - with the chain itself and the platform repo untouched.

## Problem

The sachviet run hit this live: the PLAN gate had to override the default and record the choice by hand. A fresh operator following install's own printed instructions writes task@1 files while the chain silently prefers the heavy profile - a mismatch every consumer meets on day one.

## Proposed Solution

In install.sh step 3b, when creating config.yaml (create-once, unchanged), write `task_template: task@1` as a live line on consumer installs; on the platform repo (detected via the existing `is_platform_repo()` guard) keep today's commented default so the corpus profile stays operator-chosen. Existing config.yaml files are never touched.

## Alternatives Considered

- Flip the chain default in task-author when .cyberos exists. Rejected by the operator decision: a config line is inspectable and overridable in place; a conditional chain default is invisible.
- Keep as is and document. Rejected: the mismatch already cost a live gate override on the first consumer run.

## Success Metrics

- Primary: a fresh consumer scratch install resolves task@1 with source `config.yaml` and zero operator intervention (asserted by the new hygiene scenario on every suite run). Baseline: resolution requires a PLAN-gate override (sachviet evidence). Deadline: final acceptance.
- Guardrail: re-install on a repo with an existing config.yaml leaves it byte-identical (create-once regression stays covered).

## Scope

In scope: the step 3b scaffold line, the platform-repo guard, hygiene scenarios.

### Out of scope / Non-Goals

- The resolution chain, task-author prose, and the engineering-spec@1 profile itself.
- Migrating existing consumer repos (their config.yaml is theirs).

## Dependencies

- None; cone is install.sh step 3b (serial with TASK-IMP-090 in the same agent; disjoint from 089/091/092).

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS task-author skill inside Cowork.
- **Scope:** spec drafted from the recorded IMP-06 decision; implementation under ship-tasks supervision.
- **Human review:** batch-3 PLAN approved 2026-07-16; both HITL gates are recorded human verdicts.

## 1. Description (normative)

- 1.1 On a consumer install that creates config.yaml, the scaffold MUST include the live line `task_template: task@1` (uncommented), keeping every other scaffold line as today.
- 1.2 On the platform repo (existing `is_platform_repo()` detection), the scaffold MUST keep today's commented `# task_template: engineering-spec@1` form.
- 1.3 An existing config.yaml MUST NOT be modified on re-install (create-once regression).
- 1.4 Hygiene coverage MUST land as t06 scenarios in test_install_hygiene.sh covering 1.1, 1.2, and 1.3.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1) - fresh consumer install yields live task@1 line - test: `tools/install/tests/test_install_hygiene.sh::t06_consumer_template_default`
- [ ] AC 2 (traces_to: #1.2) - platform-shaped repo keeps the commented default - test: `tools/install/tests/test_install_hygiene.sh::t06_platform_keeps_comment`
- [ ] AC 3 (traces_to: #1.3) - re-install leaves an existing config.yaml byte-identical - test: `tools/install/tests/test_install_hygiene.sh::t06_existing_config_untouched`
- [ ] AC 4 (traces_to: #1.4) - scenarios run inside the hygiene suite - verify: suite summary counts t06 (ops check in the gate log; glob discovery is the runner's contract).

## 3. Edge cases

- Consumer repo that happens to contain modules/memory/memory.schema.json (platform detector's marker): treated as platform - documented, the guard is the existing one (AC 2 exercises the marker directly).
- Operator who wants engineering-spec@1 in a consumer repo: edits the live line; create-once means the choice then sticks forever (AC 3).
- Security-class: none - one scaffold line in a gitignored local config; no execution surface.
