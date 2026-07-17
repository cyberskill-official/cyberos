---
id: TASK-IMP-095
title: gates.env regeneration names its backup and the durable home
template: task@1
type: improvement
module: improvement
status: done
priority: p2
author: "@stephencheng"
department: engineering
created_at: 2026-07-17T08:05:00Z
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-CUO-207]
routed_back_count: 0
awh: N/A
verify: T
phase: "pre-1.0.0 release"
owner: Stephen Cheng (CTO)
created: 2026-07-17
shipped: 2026-07-17
memory_chain_hash: null
effort_hours: 1
service: tools/install
new_files: []
modified_files:
  - tools/install/install.sh
  - tools/install/tests/test_install_hygiene.sh
source_pages:
  - "tools/install/install.sh step 3 (lines ~155-157: gates.env regenerated every install; prior copy moved to gates.env.bak.<ts> silently)"
  - "IMPROVEMENT_HANDOFF.md IMP-09 + observation O1 (the .bak is the designed one-deep undo; nothing says so at the moment of clobber)"
source_decisions:
  - "2026-07-17 Stephen: batch 4 PLAN approved (§0a, all 7 items)."
---

# TASK-IMP-095: gates.env regeneration names its backup and the durable home

## Summary

Every install regenerates gates.env and silently moves an edited prior copy to a timestamped .bak. The design is fine - config.yaml is the durable override home - but the operator learns that nowhere near the moment their edit vanishes. When the regenerated file differs from the prior one, print one line naming the backup path and pointing at config.yaml.

## Problem

A silent clobber of an operator-edited file is a trust leak even when a backup exists: the sachviet run only found the .bak by listing the directory.

## Proposed Solution

In step 3, after writing the new gates.env: if a prior copy existed and differs (cmp) from the regenerated file, print `cyberos install: gates.env regenerated (previous kept at <bak>); durable overrides belong in .cyberos/config.yaml`. Identical regeneration stays silent, and the freshly-created case (no prior file) stays silent.

## Alternatives Considered

- Stop regenerating when edited. Rejected: gates.env is documented as machine-owned (TASK-CUO-207); config.yaml is the override home - changing ownership semantics is a bigger decision than a message.
- Prompt interactively. Rejected: install must stay non-interactive for CI and agents.

## Success Metrics

- Primary: an edited gates.env followed by re-install produces exactly the notice line naming an existing .bak file; unedited re-install produces no notice - suite-asserted every run. Baseline: silence in both cases. Deadline: final acceptance.
- Guardrail: fresh install (no prior file) stays silent (no false notice).

## Scope

In scope: the diff check, the one message line, hygiene scenarios.

### Out of scope / Non-Goals

- Changing regeneration or backup behavior itself.
- Merging operator edits into the regenerated file.

## Dependencies

- Shares install.sh with TASK-IMP-094/096 - one agent, serial.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS task-author skill inside Cowork.
- **Scope:** spec drafted from IMP-09; implementation under ship-tasks supervision.
- **Human review:** batch-4 PLAN approved 2026-07-17 (§0a); both HITL gates are recorded human verdicts.

## 1. Description (normative)

- 1.1 When a prior gates.env existed and the regenerated file differs from it, install MUST print one notice line naming the exact .bak path and `.cyberos/config.yaml` as the durable override home.
- 1.2 When the regenerated file is byte-identical to the prior one, and when no prior file existed, install MUST NOT print the notice.
- 1.3 Coverage MUST land as a hygiene scenario exercising both the edited and unedited paths.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1) - edited file + re-install -> notice with a real .bak path - test: `tools/install/tests/test_install_hygiene.sh::t08_gates_env_regen_notice`
- [ ] AC 2 (traces_to: #1.2) - unedited re-install and fresh install -> no notice - test: `tools/install/tests/test_install_hygiene.sh::t08_gates_env_regen_notice (silent arms)`
- [ ] AC 3 (traces_to: #1.3) - scenario runs inside the hygiene suite - verify: suite summary counts t08 (glob discovery is the runner's contract; recorded in the gate log).

## 3. Edge cases

- Two installs in the same second (same .bak timestamp): the message names whichever .bak the step just wrote; the pre-existing `rm -f gates.env.bak.*` churn guard at line 72 is unchanged.
- Operator edit that regeneration happens to reproduce byte-identically: silent by design (1.2) - nothing was lost.
- Security-class: none - one echo of local paths; no content of the edited file is printed.
