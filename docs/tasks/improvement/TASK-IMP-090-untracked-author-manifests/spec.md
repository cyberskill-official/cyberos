---
id: TASK-IMP-090
title: task-author manifests default to untracked .workflow session state
template: task@1
type: improvement
module: improvement
status: reviewing
priority: p2
author: "@stephencheng"
department: engineering
created_at: 2026-07-16T17:25:00Z
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-CUO-206, TASK-IMP-088]
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
new_files:
  - docs/tasks/_audits/IMPROVEMENT-BATCHES-2026-07-16.md
modified_files:
  - modules/skill/task-author/SKILL.md
  - tools/install/install.sh
  - tools/install/tests/test_install_hygiene.sh
  - docs/tasks/.workflow/.gitignore
source_pages:
  - "modules/skill/task-author/SKILL.md CONTRACT_ECHO (manifest_path default: <output_dir>/manifest.json - session state inside the tracked task tree)"
  - "tools/install/install.sh .workflow/.gitignore seed (covers *.ship.json only)"
  - "operator decision 2026-07-16 (batch-2 PLAN gate, IMP-11): untracked session state; the _audits batch summary is the tracked approval record"
source_decisions:
  - "2026-07-16 Stephen: IMP-11 untracked chosen; batch 3 PLAN approved."
---

# TASK-IMP-090: task-author manifests default to untracked .workflow session state

## Summary

Author-run manifests are session state, but the skill defaults them into the tracked task tree and the .workflow gitignore seed only covers ship manifests - so this run's three batch manifests landed tracked. Per the recorded IMP-11 decision: the skill default moves to `docs/tasks/.workflow/`, the gitignore seed grows `*.manifest.json`, the three tracked manifests leave the index, and a tracked batch summary in `_audits/` becomes the durable approval record.

## Problem

Session caches in tracked space churn history and get mistaken for records; the actual record the operator needs (what was planned, who approved, what landed) belongs in `_audits/` like the sachviet run's batch summary.

## Proposed Solution

Four small moves: (1) SKILL.md CONTRACT_ECHO default becomes `docs/tasks/.workflow/task-author.<slug>.manifest.json`; (2) install.sh's .workflow/.gitignore seed writes both `*.ship.json` and `*.manifest.json`, and appends the manifest pattern once to an existing seed that lacks it; (3) `git rm --cached` the three batch manifests in this repo and update its .workflow/.gitignore; (4) write `docs/tasks/_audits/IMPROVEMENT-BATCHES-2026-07-16.md` summarizing batches 1-3 (members, verdicts, evidence commits) as the tracked approval record.

## Alternatives Considered

- Keep manifests tracked as the audit trail. Rejected by the decision: they churn per-step and duplicate what _audits records once.
- Ignore-only without the _audits summary. Rejected: removing tracking without a durable record would orphan the PLAN approvals.

## Success Metrics

- Primary: a fresh install's .workflow/.gitignore covers both patterns and a new author run leaves zero tracked manifest files (hygiene scenario on every run). Baseline: three tracked manifests today. Deadline: final acceptance.
- Guardrail: the _audits summary exists and names every batch member with its evidence commit.

## Scope

In scope: the skill default line, the seed patterns plus append-once migration, this repo's index cleanup, the _audits summary.

### Out of scope / Non-Goals

- Ship-manifest handling (already untracked by the existing pattern).
- Rewriting git history (the files leave the index going forward; history keeps them).

## Dependencies

- Shares the install.sh cone with TASK-IMP-088 - same agent, serial order per the batch plan. Disjoint from 089/091/092.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS task-author skill inside Cowork.
- **Scope:** spec drafted from the recorded IMP-11 decision; implementation under ship-tasks supervision.
- **Human review:** batch-3 PLAN approved 2026-07-16; both HITL gates are recorded human verdicts.

## 1. Description (normative)

- 1.1 task-author SKILL.md's CONTRACT_ECHO default manifest_path MUST become `docs/tasks/.workflow/task-author.<slug>.manifest.json`, with the caller override unchanged.
- 1.2 install.sh's .workflow/.gitignore seed MUST cover `*.ship.json` and `*.manifest.json`; when the file exists carrying only the ship pattern, the manifest pattern MUST be appended exactly once (idempotent on re-install).
- 1.3 This repo's three tracked batch manifests MUST leave the git index (`git rm --cached`), with the local .workflow/.gitignore updated so they stay on disk untracked.
- 1.4 A tracked approval record MUST land at `docs/tasks/_audits/IMPROVEMENT-BATCHES-2026-07-16.md` naming each batch's members, PLAN approval, HITL verdicts, and evidence commits.
- 1.5 Hygiene coverage MUST land as t07 scenarios in test_install_hygiene.sh for 1.2's fresh-seed and append-once paths.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1) - skill default names .workflow - verify: recorded grep of SKILL.md CONTRACT_ECHO in the gate log (prose contract; the skill is not executable in isolation).
- [ ] AC 2 (traces_to: #1.2, #1.5) - fresh seed carries both patterns; existing seed gains the pattern exactly once across two installs - test: `tools/install/tests/test_install_hygiene.sh::t07_workflow_gitignore_patterns`
- [ ] AC 3 (traces_to: #1.3) - `git ls-files docs/tasks/.workflow` returns empty - verify: recorded command output in the gate log (repo-state chore).
- [ ] AC 4 (traces_to: #1.4) - the _audits record names all 11 shipped members with commits - verify: recorded grep set in the gate log (single tracked document, same rationale as TASK-IMP-087 ACs).

## 3. Edge cases

- Operator-customized .workflow/.gitignore with extra lines: append-once adds the one pattern and touches nothing else (AC 2's second-install assertion).
- The removed-from-index manifests reappearing dirty: the updated local gitignore keeps them invisible (AC 3).
- Security-class: none - gitignore and prose edits plus an index operation with recorded evidence.
