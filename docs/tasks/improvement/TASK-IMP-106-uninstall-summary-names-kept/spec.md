---
id: TASK-IMP-106
title: Uninstall summary names what it kept
template: task@1
type: improvement
module: improvement
status: ready_to_test
priority: p1
author: "@stephencheng"
department: engineering
created_at: 2026-07-17T14:00:00Z
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: [TASK-IMP-103]
blocks: []
related_tasks: [TASK-IMP-095, TASK-IMP-096]
routed_back_count: 0
awh: N/A
verify: T
phase: "pre-1.0.0 release"
owner: Stephen Cheng (CTO)
created: 2026-07-17
memory_chain_hash: null
effort_hours: 1
service: tools/install
new_files:
  - (none)
modified_files:
  - tools/install/uninstall.sh
  - tools/install/tests/test_install_hygiene.sh
source_pages:
  - "IMPROVEMENT_HANDOFF.md §10 IMP-27"
  - "tools/install/uninstall.sh (keeps docs/tasks by design; summary never says so - verified on main bb231900)"
source_decisions:
  - "2026-07-17 Stephen: PLAN gate - scope C (all 13 actionable handoff findings), template override to task@1 (recorded HITL answer)."
---

# TASK-IMP-106: Uninstall summary names what it kept

## Summary

`uninstall.sh` deliberately keeps the operator's corpus, status page, CHANGELOG, and BRAIN - the right default - and never says so. Someone who uninstalls expects the repo to be clean and finds task folders and memory still there, with no way to know it was intentional. Add a summary that names what was removed, what was kept and why, and how to remove the kept material by hand.

## Problem

`uninstall.sh` references `docs/tasks` twice and leaves it alone by design: the corpus is the operator's work, not the machine's. But the summary says nothing about it. This is the same failure shape as TASK-IMP-095 (gates.env clobbered silently) and TASK-IMP-096 (non-git install silently useless) - a correct default, undocumented at the exact moment it surprises someone - and both of those proved worth fixing.

Silence at that moment has a specific cost: the operator either assumes the uninstall failed and re-runs it, or deletes the corpus by hand to "finish the job" and loses the work the default existed to protect.

## Proposed Solution

Print a two-part summary on successful uninstall. Removed: the vendored machine and the agent surface, each named. Kept: `docs/tasks/` (your corpus), `docs/status/`, `CHANGELOG.md`, `.cyberos/memory` (BRAIN) - with a one-line reason and the verbatim command to remove them for an operator who means it. Names come from what the run actually did, not a hard-coded list, so the summary cannot drift from the behavior.

## Alternatives Considered

- Prompt to delete the corpus. Rejected: uninstall is frequently non-interactive, and a prompt that defaults to deleting an operator's work is a footgun with a confirmation step.
- A `--purge` flag that removes everything. Rejected here as scope creep - the defect is silence, not a missing capability. Worth its own task if anyone asks.
- Documentation only (GUIDE note). Rejected: the surprise happens at the terminal, and that is where the answer has to be.

## Success Metrics

- Primary: a successful uninstall names every kept path with a reason and the removal command - suite-asserted against the real run's output. Baseline: zero mentions today.
- Guardrail: the summary reflects what the run did (derived, not hard-coded) - a kept path absent from the repo is not claimed as kept.

## Scope

In scope: the uninstall summary block, suite arms.

### Out of scope / Non-Goals

- Changing what uninstall removes or keeps - the state machine is correct and this task must not touch it.
- A purge flag.
- The install-side summary (TASK-IMP-096 already covers its case).

## Dependencies

depends_on TASK-IMP-103: both edit `uninstall.sh`, and 103 adds the lock-removal branch whose outcome this summary must report. Serialised, not parallel. Per TASK-IMP-101's evidence gate, 103's coverage-gate artefact is the evidence.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS task-author skill inside Cowork.
- **Scope:** spec drafted from IMPROVEMENT_HANDOFF.md IMP-27, verified against uninstall.sh on merged main; implementation under ship-tasks supervision.
- **Human review:** scope approved at the 2026-07-17 PLAN gate; both HITL gates are recorded human verdicts.

## 1. Description (normative)

- 1.1 On successful uninstall, the summary MUST name what was removed (the vendored machine and the agent surface entries the run actually removed).
- 1.2 The summary MUST name each kept path with a one-line reason: `docs/tasks/` (the corpus), `docs/status/`, `CHANGELOG.md`, `.cyberos/memory` (BRAIN).
- 1.3 The summary MUST print the verbatim command an operator can run to remove the kept material themselves.
- 1.4 Kept paths MUST be derived from what exists after the run - a path that is not present MUST NOT be claimed as kept.
- 1.5 The summary MUST NOT change what uninstall removes or keeps.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1, #1.2, #1.3) - uninstall on a populated repo prints removed entries, all four kept paths with reasons, and the manual-removal command - test: `tools/install/tests/test_install_hygiene.sh::t20_uninstall_summary_names_kept`
- [ ] AC 2 (traces_to: #1.4) - uninstall on a repo with no `docs/status/` does not claim it as kept - test: `tools/install/tests/test_install_hygiene.sh::t21_uninstall_summary_derived_not_hardcoded`
- [ ] AC 3 (traces_to: #1.5) - the set of files present after uninstall is byte-identical to today's behavior - test: `tools/install/tests/test_install_hygiene.sh::t22_uninstall_behavior_unchanged`

## 3. Edge cases

- Uninstall on a repo that was never installed: no removal summary - it MUST NOT print a kept list for a machine that was not there (nothing was kept; nothing was removed).
- BRAIN present but empty: still named as kept - the directory is the operator's, and its emptiness is not the uninstaller's judgment to make.
- Corpus present but `docs/status/` absent (never rendered): per 1.4, status is omitted from the kept list rather than claimed.
- A partially-removed machine from an interrupted earlier uninstall: the summary reports what this run removed, not what it wished it had removed.
- Security-class: prints paths that already exist in the repo; interpolates no user-supplied string into a command. The printed removal command is documentation, never executed by the script.
