---
id: TASK-IMP-096
title: Non-git installs state the ship-tasks git requirement
template: task@1
type: improvement
module: improvement
status: ready_to_implement
priority: p3
author: "@stephencheng"
department: engineering
created_at: 2026-07-17T08:05:00Z
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-IMP-083]
routed_back_count: 0
awh: N/A
verify: T
phase: "pre-1.0.0 release"
owner: Stephen Cheng (CTO)
created: 2026-07-17
shipped: null
memory_chain_hash: null
effort_hours: 1
service: tools/install
new_files: []
modified_files:
  - tools/install/install.sh
  - tools/install/tests/test_install_hygiene.sh
source_pages:
  - "tools/install/install.sh summary block (today only the hook line hints: 'skipped - not a git checkout')"
  - "IMPROVEMENT_HANDOFF.md IMP-12 + observation O3: ship-tasks needs commits, diff-scoped coverage, and route-back restores - none exist without git"
source_decisions:
  - "2026-07-17 Stephen: batch 4 PLAN approved (§0a, all 7 items)."
---

# TASK-IMP-096: Non-git installs state the ship-tasks git requirement

## Summary

Install succeeds on a plain directory, and the only clue that the governance loop cannot run there is a hook line's aside. Say the quiet part in the summary: one line, printed only when `.git` is absent, naming what ship-tasks needs and the exact command to get it.

## Problem

A consumer who installs into a non-git folder discovers the requirement at their first phase commit - the most expensive place to learn it.

## Proposed Solution

In the summary block, when the repo has no `.git`: print `cyberos install: this repo is not a git checkout - ship-tasks needs one; run: git init -b main && git add -A && git commit -m init`. Git installs print nothing new.

## Alternatives Considered

- Refuse to install without git. Rejected: install legitimately serves doc-only and evaluation uses; the loop is optional until tasks ship.
- Auto-run git init. Rejected: creating a repository is an operator decision (identity, default branch, what to commit), not an installer side effect.

## Success Metrics

- Primary: a scratch non-git install prints exactly one such line and a git install prints none - suite-asserted every run. Baseline: only the hook aside exists. Deadline: final acceptance.
- Guardrail: the line names a command that works verbatim on a fresh directory.

## Scope

In scope: the summary line, the hygiene scenario.

### Out of scope / Non-Goals

- Any behavior change for git repos.
- Running git commands on the consumer's behalf.

## Dependencies

- Shares install.sh with TASK-IMP-094/095 - one agent, serial.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS task-author skill inside Cowork.
- **Scope:** spec drafted from IMP-12; implementation under ship-tasks supervision.
- **Human review:** batch-4 PLAN approved 2026-07-17 (§0a); both HITL gates are recorded human verdicts.

## 1. Description (normative)

- 1.1 When the target repo has no `.git`, the install summary MUST include one line stating ship-tasks requires a git checkout and naming the verbatim remedy (`git init -b main && git add -A && git commit -m init`).
- 1.2 When the target is a git checkout, the summary MUST NOT include the line.
- 1.3 Coverage MUST land as a hygiene scenario running install against a non-git scratch directory (CYBEROS_NO_MIGRATE path acceptable) and a git one.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1, #1.3) - non-git scratch install shows the line once - test: `tools/install/tests/test_install_hygiene.sh::t09_nongit_summary_line`
- [ ] AC 2 (traces_to: #1.2) - git scratch install shows no such line - test: `tools/install/tests/test_install_hygiene.sh::t09_nongit_summary_line (git arm)`

## 3. Edge cases

- A git worktree or submodule dir where `.git` is a FILE, not a directory: counts as a git checkout (test uses `git rev-parse` semantics, not a `-d .git` probe - matching install.sh's own root detection).
- Bare-ish folders with a stale `.git` remnant: whatever `git rev-parse --show-toplevel` says is the truth the installer already uses; the line follows it.
- Security-class: none - one echo; no commands executed on the consumer's behalf.
