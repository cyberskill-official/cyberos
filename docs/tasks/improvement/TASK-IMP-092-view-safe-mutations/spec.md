---
id: TASK-IMP-092
title: Lost-update hardening, retally headers and committed-object evidence
template: task@1
type: improvement
module: improvement
status: implementing
priority: p0
author: "@stephencheng"
department: engineering
created_at: 2026-07-16T17:25:00Z
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-IMP-085, TASK-IMP-086]
routed_back_count: 0
awh: N/A
verify: T
phase: "pre-1.0.0 release"
owner: Stephen Cheng (CTO)
created: 2026-07-16
shipped: null
memory_chain_hash: null
effort_hours: 4
service: tools/install/docs-tools
new_files: []
modified_files:
  - tools/install/docs-tools/backlog-mutate.mjs
  - tools/install/tests/test_workflow_helpers.sh
  - modules/cuo/chief-technology-officer/workflows/ship-tasks.md
source_pages:
  - "TASK-IMP-086 gate-log CORRECTIVE ADDENDUM (the lost-update incident: host-view writes vs sandbox-view commits, header masking via inherited counts)"
  - "tools/install/docs-tools/backlog-mutate.mjs (incremental header count adjust - inherited the lying baseline, 34 vs true 20)"
  - "IMPROVEMENT_HANDOFF.md IMP-18 (adopted rules: one writer one view; committed-object evidence)"
source_decisions:
  - "2026-07-16 Stephen: batch 3 PLAN approved with this item at p0."
---

# TASK-IMP-092: Lost-update hardening, retally headers and committed-object evidence

## Summary

The 086 incident had two mechanical enablers: backlog-mutate adjusts header counts incrementally (so a wrong baseline propagates forever) and nothing in doctrine required acceptance evidence to be measured on a committed object rather than a working view. Fix both: every mutation retallies the target section's header from its actual rows, and ship-tasks doctrine (v2.6.3) gains the two rules adopted from the incident - shared files get one writer through one filesystem view per run, and content-deliverable acceptance evidence is measured with git show against the commit, never a working tree.

## Problem

Post-acceptance verification (raised by the PR review bot) proved no commit ever carried 086's rows while every working-view read looked consistent; the header count 34 vs a true tally of 20 was the incremental adjust faithfully preserving a lie. Tools and doctrine must make this class structurally loud.

## Proposed Solution

backlog-mutate's flip and insert recompute the counted header from a full scan of the section's rows after the mutation (zero-count statuses omitted, lifecycle order, matching the file's own convention); a fixture whose header lies proves any mutation corrects it. ship-tasks gains two short normative passages: §11a swarm cone-independence explicitly includes view-independence with shared files owned by ONE writer through ONE view, and the testing-phase guidance requires committed-object evidence (`git show <commit>:<path>`) for content deliverables. workflow_version bumps to 2.6.3; payload re-vendored.

## Alternatives Considered

- File locking between views. Rejected: the two views are separate caches, not cooperating processes; a lock file is itself subject to the same divergence.
- Post-commit hook comparing BACKLOG to frontmatter. Deferred as a candidate guard (recorded in IMP-18); the retally plus evidence rule close the incident's actual enablers first.

## Success Metrics

- Primary: a mutation over a lying-header fixture emits the true tally (suite-asserted every run), and the vendored doctrine carries both rules. Baseline: incremental adjust preserved a 14-off header through six mutations in the incident. Deadline: final acceptance.
- Guardrail: existing t01-t09 helper scenarios stay green (retally must not break flip/insert semantics).

## Scope

In scope: the retally implementation, the lying-baseline fixture, the two doctrine passages, version bump and re-vendor.

### Out of scope / Non-Goals

- The post-commit parity guard (candidate follow-up in IMP-18).
- Changing the two-view filesystem itself (environment fact; doctrine routes around it).

## Dependencies

- None; cone is backlog-mutate + its suite + ship-tasks.md (disjoint from 088/089/090/091).

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS task-author skill inside Cowork.
- **Scope:** spec drafted from the 086 corrective record; implementation under ship-tasks supervision.
- **Human review:** batch-3 PLAN approved 2026-07-16; both HITL gates are recorded human verdicts.

## 1. Description (normative)

- 1.1 backlog-mutate's flip and insert MUST recompute the target section's counted header from a full tally of the section's rows after the mutation, replacing incremental adjustment; zero-count statuses are omitted and statuses print in lifecycle order per the file's own convention. Headers without counts stay untouched.
- 1.2 A fixture whose header disagrees with its rows MUST be corrected to the true tally by ANY mutation, asserted in the suite.
- 1.3 The whole-file discipline MUST hold: a mutation's diff stays one row plus at most one header line (the retally can only change the header it was already allowed to change).
- 1.4 ship-tasks.md MUST gain the one-writer-one-view rule in §11a (swarm cone-independence includes view-independence; shared files are owned by one writer through one filesystem view per run) and the committed-object evidence rule in the testing-phase guidance (content-deliverable acceptance evidence is measured via git show against the commit).
- 1.5 workflow_version MUST bump to 2.6.3 and the payload MUST re-vendor the updated workflow and tool (existing vendor gates cover carry; the suite asserts the doctrine text in the scratch payload).

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1, #1.2) - lying-header fixture corrected to true tally on flip and on insert - test: `tools/install/tests/test_workflow_helpers.sh::t10_retally_corrects_lying_header`
- [ ] AC 2 (traces_to: #1.3) - mutation diff footprint stays 1 row + <=1 header - test: `tools/install/tests/test_workflow_helpers.sh::t11_footprint_holds_with_retally`
- [ ] AC 3 (traces_to: #1.1) - existing t01-t09 stay green - test: `tools/install/tests/test_workflow_helpers.sh::t01_manifest_lifecycle` (representative; full suite runs as one)
- [ ] AC 4 (traces_to: #1.4, #1.5) - doctrine passages present in source and scratch payload at v2.6.3 - test: `tools/install/tests/test_workflow_helpers.sh::t12_doctrine_view_rules_vendored`

## 3. Edge cases

- Section with rows in statuses the header never listed (the incident's shape): retally introduces them in lifecycle order (AC 1).
- Empty section after a hypothetical removal: counts drop to the sole remaining status or the header keeps its bare form - never a negative or a zero entry (covered inside t10 fixtures).
- Uncounted (bare) headers: untouched, existing behavior preserved (t06 regression stays green, AC 3).
- Security-class: none - tool and prose changes gated by the suite; no new execution surface.
