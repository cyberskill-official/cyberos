---
id: TASK-IMP-097
title: GUIDE gains the sandboxed-agent runbook section
template: task@1
type: improvement
module: improvement
status: reviewing
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
shipped: null
memory_chain_hash: null
effort_hours: 2
service: tools/install/docs
new_files: []
modified_files:
  - tools/install/docs/index.md
  - modules/cuo/chief-technology-officer/workflows/ship-tasks.md
  - tools/install/tests/test_full_sdp_payload.sh
source_pages:
  - "IMPROVEMENT_HANDOFF.md IMP-13 + observations O2/O7: 45 s command caps kill hook chains and npm installs; background processes die with the call; package-manager churn over synced mounts is impractical; the working pattern is a local clone for build/test with a local-ref push back to the mounted repo"
  - "TASK-IMP-092's one-writer-one-view doctrine (v2.6.3) - the sibling environment rule this section complements"
  - "tools/install/docs/index.md (GUIDE.md source; ships via build.sh)"
source_decisions:
  - "2026-07-17 Stephen: batch 4 PLAN approved (§0a, all 7 items)."
---

# TASK-IMP-097: GUIDE gains the sandboxed-agent runbook section

## Summary

Three governed runs re-learned the same sandbox facts by hitting them. Write them down where consumers read: a short "Running CyberOS under sandboxed agents" section in the GUIDE source - command caps and background-process death, the local-clone pattern for build/test with a local-ref push back to the mounted repo, manual hook-obligation replay with `--no-verify` plus recorded evidence, and mount unlink/permission quirks - with a one-line cross-reference from ship-tasks.

## Problem

Every fact in that list cost real time to discover and none of it is discoverable today except by reading incident addenda in task folders.

## Proposed Solution

One section in tools/install/docs/index.md (which builds into the payload's GUIDE.md), written as a runbook: symptom, cause, working pattern. Explicit that the local-clone push is a local ref move - the no-push policy stays intact. ship-tasks.md gains one cross-reference line pointing constrained environments at the GUIDE section (prose pointer, no version bump - TASK-IMP-099 carries this round's bump). test_full_sdp_payload.sh gains a grep gate that the built GUIDE carries the section.

## Alternatives Considered

- A separate RUNBOOK.md. Rejected: the GUIDE is the one consumer document install ships and the checklist's D2 pass already reads it end-to-end; a second doc is a second place to go stale.
- Doctrine passages in ship-tasks.md instead. Rejected: these are environment facts, not workflow rules; the workflow already carries the two normative rules (v2.6.3) and links out for the rest.

## Success Metrics

- Primary: the built payload's GUIDE.md carries the section with the local-clone pattern and the hook-replay rule, gated by the suite on every run. Baseline: zero consumer-facing documentation of any of it. Deadline: final acceptance.
- Guardrail: ship-tasks.md carries exactly one cross-reference line (no duplicated rule text to drift).

## Scope

In scope: the GUIDE section, the ship-tasks cross-reference line, the payload grep gate.

### Out of scope / Non-Goals

- New workflow rules (v2.6.3 already carries the normative pair).
- Automation of the local-clone pattern.

## Dependencies

- Shares ship-tasks.md with TASK-IMP-099 - one agent, serial, per the batch plan.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS task-author skill inside Cowork.
- **Scope:** spec drafted from IMP-13 and the recorded run observations; implementation under ship-tasks supervision.
- **Human review:** batch-4 PLAN approved 2026-07-17 (§0a); both HITL gates are recorded human verdicts.

## 1. Description (normative)

- 1.1 tools/install/docs/index.md MUST gain a "Running CyberOS under sandboxed agents" section covering: per-command time caps and background-process death (hook chains, package installs), the local-clone build/test pattern with local-ref push back to the mounted repo (explicitly not a remote push), manual hook-obligation replay with `--no-verify` plus recorded evidence, and mount unlink/permission quirks.
- 1.2 ship-tasks.md MUST gain one cross-reference line pointing constrained environments at the GUIDE section, without duplicating its content and without a workflow_version bump.
- 1.3 The built payload's GUIDE.md MUST carry the section, gated by a grep in test_full_sdp_payload.sh against a scratch build.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1, #1.3) - scratch payload GUIDE carries the section incl. local-clone and hook-replay lines - test: `tools/install/tests/test_full_sdp_payload.sh (sandbox-runbook grep gate)`
- [ ] AC 2 (traces_to: #1.2) - ship-tasks.md carries exactly one cross-reference line - verify: recorded grep -c in the gate log (single prose line; a test for one line of prose is out of proportion, and t12's doctrine gate already pins the file's normative content).

## 3. Edge cases

- The GUIDE is consumer-facing: the section names no internal session paths or tool brands beyond what reproduces anywhere (generic "sandboxed agent" framing).
- Cross-reference placement: near the §11a/testing doctrine it complements, so readers meet rule and runbook together.
- Security-class: none - documentation; the no-push policy is restated, not weakened.
