---
id: TASK-IMP-094
title: Channels, shared .agents/skills dir plus Devin and Windsurf rules
template: task@1
type: improvement
module: improvement
status: done
priority: p1
author: "@stephencheng"
department: engineering
created_at: 2026-07-17T08:05:00Z
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-IMP-087]
routed_back_count: 0
awh: N/A
verify: T
phase: "pre-1.0.0 release"
owner: Stephen Cheng (CTO)
created: 2026-07-17
shipped: 2026-07-17
memory_chain_hash: null
effort_hours: 4
service: tools/install
new_files: []
modified_files:
  - tools/install/install.sh
  - tools/install/uninstall.sh
  - tools/install/tests/test_channels.sh
  - tools/install/tests/test_install_hygiene.sh
source_pages:
  - "tools/install/install.sh step 5b (agent-surface block: .cursorrules, .cursor/rules/, .github/copilot-instructions.md, .agents/rules/, .windsurfrules, .claude/skills, ...)"
  - "IMPROVEMENT_HANDOFF.md IMP-08 + the 2026-07-16 channel research: .agents/skills/ is the Agent Skills open standard's shared project dir (Codex, Copilot, Cursor, Gemini CLI, OpenCode); Windsurf rebranded to Devin Desktop June 2026 - .devin/rules/ preferred, .windsurf/rules/ fallback, .windsurfrules still read"
  - "docs/release/RELEASE-CHECKLIST.md rows C1 (matrix re-verify) and E3 (this item's gate line)"
source_decisions:
  - "2026-07-17 Stephen: batch 4 PLAN approved (§0a, all 7 items)."
---

# TASK-IMP-094: Channels, shared .agents/skills dir plus Devin and Windsurf rules

## Summary

The install's agent-surface block predates two convention moves the 2026-07-16 research recorded: the Agent Skills open standard's shared `.agents/skills/` dir (one dir now read by Codex, Copilot, Cursor, Gemini CLI, and OpenCode) and the Windsurf-to-Devin rebrand (`.devin/rules/` preferred, `.windsurf/rules/` fallback, legacy `.windsurfrules` still read). Extend step 5b with the shared skills entries and the two rules pointers, keep the legacy file, and teach the gitignore block and uninstall the new paths.

## Problem

Every convention miss is an agent that cannot discover the workflow: a consumer opening the repo in Devin Desktop today finds only the legacy filename, and the five agents reading the shared skills dir find nothing at all.

## Proposed Solution

Step 5b additions, all create-if-absent and honoring CYBEROS_AGENTS filtering: (a) `.agents/skills/<cmd>` entries for the three commands, as relative symlinks to the existing `.claude/skills/<cmd>` copies with a plain-copy fallback where symlinks are unavailable; (b) `.devin/rules/cyberos.md` and `.windsurf/rules/cyberos.md` pointer files with the same body as the other rules pointers; (c) the legacy `.windsurfrules` stays. The managed-gitignore block gains the new paths; uninstall strips them with the rest of the surface. test_channels.sh and hygiene t01 learn the new expectations.

## Alternatives Considered

- Replace `.windsurfrules` with the new paths. Rejected: the research says the legacy file is still read; removal breaks users who have not updated, for zero gain.
- Copies instead of symlinks in `.agents/skills/`. Kept only as the fallback: symlinks keep the skills single-sourced under `.claude/skills/`; copies drift.
- Waiting for the C1 pre-tag re-verify to do it all at once. Rejected: E3 is a checklist gate line for exactly this implementation; C1 re-verifies freshness at tag time regardless.

## Success Metrics

- Primary: a scratch install shows the shared skills entries and both rules pointers; a second install changes nothing (idempotence) - suite-asserted every run. Baseline: none of the three paths exist. Deadline: final acceptance.
- Guardrail: uninstall leaves no orphan under any new path, and the managed gitignore block round-trips (strip + regenerate) cleanly.

## Scope

In scope: step 5b additions, gitignore block, uninstall strip, suite extensions.

### Out of scope / Non-Goals

- Re-verifying the whole channel matrix (release row C1, operator, at tag time).
- New skill content - the shared dir points at the existing skills.
- MCP channel changes.

## Dependencies

- Shares install.sh with TASK-IMP-095/096 - one agent, serial, per the batch plan.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS task-author skill inside Cowork.
- **Scope:** spec drafted from the recorded channel research and IMP-08; implementation under ship-tasks supervision.
- **Human review:** batch-4 PLAN approved 2026-07-17 (§0a); both HITL gates are recorded human verdicts.

## 1. Description (normative)

- 1.1 Install MUST create (if absent) `.agents/skills/` entries for the three commands as relative symlinks to their `.claude/skills/` counterparts, falling back to copies where symlink creation fails; CYBEROS_AGENTS filtering applies.
- 1.2 Install MUST create (if absent) `.devin/rules/cyberos.md` and `.windsurf/rules/cyberos.md` pointer files, keeping `.windsurfrules`.
- 1.3 The managed gitignore block MUST cover the new paths, and uninstall MUST remove them (including the `.agents/skills` entries) without touching operator files.
- 1.4 Re-install MUST be idempotent across all new paths (no duplicates, no churn).
- 1.5 Coverage MUST land as extensions to test_channels.sh and test_install_hygiene.sh t01.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1, #1.2) - scratch install shows shared skills entries + both pointers, legacy kept - test: `tools/install/tests/test_channels.sh::t_shared_skills_and_devin_rules`
- [ ] AC 2 (traces_to: #1.4) - second install byte-idempotent on the new paths - test: `tools/install/tests/test_channels.sh::t_channel_idempotence`
- [ ] AC 3 (traces_to: #1.3, #1.5) - gitignore block covers new paths; uninstall strips them; coverage lands in the two named suites - test: `tools/install/tests/test_install_hygiene.sh::t01 (extended assertions)`
- [ ] AC 4 (traces_to: #1.1) - symlink targets resolve inside the repo (no dangling links) - test: `tools/install/tests/test_channels.sh::t_shared_skills_resolve`

## 3. Edge cases

- Filesystem without symlink support (or Windows checkout): copy fallback engages; t_shared_skills_resolve accepts either form but never a dangling link.
- Operator already created `.devin/rules/` with own files: create-if-absent adds only cyberos.md; nothing else touched (idempotence arm).
- CYBEROS_AGENTS excludes an agent family: its paths are not created (existing filter semantics, asserted for one exclusion).
- Security-class: pointer files carry repo-relative prose only; symlinks are relative and inside the repo - no absolute-path leakage.
