---
id: TASK-IMP-099
title: Queue selection prose ranks p0-p3, MoSCoW wording retired
template: task@1
type: improvement
module: improvement
status: done
priority: p3
author: "@stephencheng"
department: engineering
created_at: 2026-07-17T08:05:00Z
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-IMP-092]
routed_back_count: 0
awh: N/A
verify: T
phase: "pre-1.0.0 release"
owner: Stephen Cheng (CTO)
created: 2026-07-17
shipped: 2026-07-17
memory_chain_hash: null
effort_hours: 1
service: modules/cuo
new_files: []
modified_files:
  - modules/cuo/chief-technology-officer/workflows/ship-tasks.md
  - tools/install/tests/test_workflow_helpers.sh
source_pages:
  - "modules/cuo/chief-technology-officer/workflows/ship-tasks.md:311 ('order by priority (MUST before SHOULD before COULD), then created ascending, then id ascending')"
  - "IMPROVEMENT_HANDOFF.md IMP-16: FM-105 made priority p0-p3 (MoSCoW retired 2026-07-14; rank code accepts both for migration) - the distributed prose still teaches the retired scale"
  - "tools/install/tests/test_workflow_helpers.sh t12 (pins workflow_version 2.6.3 exactly - must move with the bump)"
source_decisions:
  - "2026-07-17 Stephen: batch 4 PLAN approved (§0a, all 7 items)."
---

# TASK-IMP-099: Queue selection prose ranks p0-p3, MoSCoW wording retired

## Summary

The distributed workflow's queue-selection rule still teaches the retired MoSCoW scale while FM-105 and the rank code moved to p0-p3 months ago. Reword line 311 to `p0 before p1 before p2 before p3 (legacy MoSCoW values map per FM-105)`, bump workflow_version 2.6.3 to 2.6.4 (a normative selection rule changed wording), move t12's exact version pin with it, and gate the payload against any bare MoSCoW ordering rule.

## Problem

An agent reading the vendored workflow verbatim ranks by a scale the linter rejects in new specs - prose teaching what the machine floor forbids.

## Proposed Solution

One line reworded; version bump; t12 pin updated to 2.6.4 within this task (the exact-pin discipline that caught the batch-3 seed change is preserved, not loosened); a payload grep in the same suite asserting the distributed cuo/ship-tasks.md carries the p0-p3 rule and no bare MoSCoW ordering (the parenthetical legacy-mapping mention is the one allowed occurrence).

## Alternatives Considered

- Loosen t12 to a version-agnostic regex. Rejected: the exact pin is the feature - it forces every normative edit to be a deliberate, disclosed bump.
- Delete the MoSCoW mention entirely. Rejected: rank code still accepts legacy values for migration; the parenthetical tells a reader with an old corpus why.

## Success Metrics

- Primary: the scratch payload's workflow carries the p0-p3 ordering rule at 2.6.4 with no bare MoSCoW rule, suite-asserted every run. Baseline: line 311 teaches MoSCoW. Deadline: final acceptance.
- Guardrail: t01-t11 untouched and green (the reword must not disturb helper behavior gates).

## Scope

In scope: the prose line, the version bump, t12's pin, the payload grep.

### Out of scope / Non-Goals

- Rank-code changes (already p0-p3 with legacy mapping).
- Sweeping other historical docs for MoSCoW mentions (task specs are historical records).

## Dependencies

- Shares ship-tasks.md with TASK-IMP-097 - one agent, serial; this task carries the round's single version bump.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS task-author skill inside Cowork.
- **Scope:** spec drafted from IMP-16; implementation under ship-tasks supervision.
- **Human review:** batch-4 PLAN approved 2026-07-17 (§0a); both HITL gates are recorded human verdicts.

## 1. Description (normative)

- 1.1 The queue-selection prose MUST rank `p0 before p1 before p2 before p3`, retaining a parenthetical that legacy MoSCoW values map per FM-105; no other ordering rule wording survives.
- 1.2 workflow_version MUST bump to 2.6.4, and t12's exact pin MUST move to 2.6.4 in the same change (disclosed in the code review).
- 1.3 The suite MUST assert the scratch payload's distributed workflow carries the p0-p3 rule and no bare MoSCoW ordering rule (the legacy-mapping parenthetical exempted).

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1, #1.3) - source and scratch payload carry the p0-p3 rule; no bare MoSCoW ordering - test: `tools/install/tests/test_workflow_helpers.sh::t13_queue_rule_p0_p3`
- [ ] AC 2 (traces_to: #1.2) - version 2.6.4 pinned in source and payload - test: `tools/install/tests/test_workflow_helpers.sh::t12_doctrine_view_rules_vendored (pin moved)`
- [ ] AC 3 (guardrail) - t01-t11 green - test: `tools/install/tests/test_workflow_helpers.sh::t01_manifest_lifecycle (representative; suite runs as one)`

## 3. Edge cases

- Other MoSCoW mentions inside ship-tasks.md (e.g. rank-mapping notes): allowed where they describe the legacy mapping; only an ORDERING rule in MoSCoW terms is forbidden (t13's pattern targets the rule shape, not the word).
- Historical task specs using MUST/SHOULD priorities: out of scope by design - records, not rules.
- Security-class: none - prose and a test pin.
