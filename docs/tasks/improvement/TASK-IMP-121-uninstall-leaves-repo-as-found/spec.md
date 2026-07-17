---
id: TASK-IMP-121
title: Uninstall must leave the repo as it found it
template: task@1
type: improvement
module: improvement
status: draft
priority: p1
author: "@stephencheng"
department: engineering
created_at: 2026-07-18T04:00:00Z
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: [TASK-IMP-106]
blocks: []
related_tasks: [TASK-IMP-094, TASK-IMP-083, TASK-IMP-103]
routed_back_count: 0
awh: N/A
verify: T
phase: "pre-1.0.0 release"
owner: Stephen Cheng (CTO)
created: 2026-07-18
memory_chain_hash: null
effort_hours: 5
service: tools/install
new_files:
  - (none)
modified_files:
  - tools/install/uninstall.sh
  - tools/install/tests/test_install_hygiene.sh
source_pages:
  - "2026-07-18 byte-level install/uninstall harness (/tmp/cyberos-harness): fresh-git case, 26 -> 2139 -> 95 paths, 69 leftovers, 0 casualties, 0 mutations"
  - "tools/install/uninstall.sh:111,:114 (the ship-tasks exemption and the readlink ownership test it exempts from)"
  - "tools/install/uninstall.sh:70 (block strip) - proven to leak one newline per cycle: 29 -> 30 -> 31 -> 32 bytes over 3 install/uninstall cycles"
source_decisions:
  - "2026-07-18 Stephen: PLAN gate - fold all four uninstall-reversibility findings into one task (only IMP-115 avoids tools/install, so each task costs a whole batch); template override to task@1 (120/120 corpus)."
  - "2026-07-18 Stephen: uninstall.sh:111's ship-tasks exemption is REFINED, not kept - remove the entry when readlink proves it points into the machine we deleted."
---

# TASK-IMP-121: Uninstall must leave the repo as it found it

## Summary

Uninstall removes the machine and leaves things pointing at it. A byte-level harness on a fresh git repo found five dangling symlinks, two MCP registrations naming a deleted server, an empty `.gitignore` install created, and a hook that gains one newline per install/uninstall cycle. None of it damages operator content - the harness recorded zero casualties and zero mutations across six cases - but the repo is left broken rather than merely untidy.

## Problem

`rm -rf "$CY"` (`:151`) deletes `.cyberos/`. Four things survive that shouldn't:

1. **Dangling skill symlinks.** `.claude/skills/ship-tasks`, `.codex/skills/ship-tasks`, `.grok/skills/ship-tasks`, `.commandcode/skills/ship-tasks`, `.opencode/skill/ship-tasks` all point at `../../.cyberos/plugin/skills/ship-tasks`. Install creates all five. Uninstall's loop considers only `.agents/skills/` and `.claude/skills/`, and `:114` exempts `ship-tasks` explicitly (`[ "$_sc" != "ship-tasks" ]`). So one is left by decision and four are never looked at.
2. **Dead MCP registrations.** `.mcp.json` and `.cursor/mcp.json` register `node .cyberos/mcp/cyberos-mcp.mjs`. Uninstall never unregisters. Every agent reading those configs tries to launch a missing server on every startup - in a repo the operator just cleaned.
3. **An empty `.gitignore`.** Install creates the file where none existed; uninstall strips the managed block (`:81`) and leaves a 0-line file behind.
4. **A hook that grows.** Install appends its block to an operator's pre-existing `pre-commit`; uninstall strips the block (`:70`, honestly reported) but leaves the separator newline. Proven to accumulate linearly: 29 -> 30 -> 31 -> 32 bytes over three cycles.

The ownership test this needs already exists and is already proven: `:114` matches `readlink` against `*".cyberos/plugin/skills/$_sc"` and used it to correctly remove `task-author` and `task-audit` in the same run that left `ship-tasks` dangling. The discipline is present; it is applied to two channels out of six and to no other artefact class.

`:111`'s exemption exists to avoid clobbering operator files - the right instinct, recorded in section 6. But a symlink whose target is inside our own machine is not an operator file: `readlink` proves ownership, which is exactly why `:114` trusts that test for its siblings. Leaving a pointer to a file we deleted is not restraint.

## Proposed Solution

Extend the existing readlink ownership test to every channel install writes, and apply the same provable-ownership discipline to the MCP registrations and the `.gitignore`. Strip the hook block with its separator so the restore is byte-exact. Ownership must be PROVEN per artefact (readlink target inside `.cyberos/`, or our marker) - never inferred from a path's existence. Anything not provably ours stays, and is named as kept per TASK-IMP-106.

## Alternatives Considered

- Remove the whole agent surface on uninstall. Rejected: `.cursorrules`, `GEMINI.md` and the rules pointers may predate install or carry operator edits; unprovable ownership is not a licence to delete. This is TASK-IMP-094's recorded reasoning and it still holds.
- Leave the dangling links and let TASK-IMP-106's summary name them. Rejected: 106 §1.5 forbids changing behaviour, so it can only describe the breakage honestly. A truthful report of a broken repo is not a fixed repo.
- Track install's creations in a manifest and reverse it exactly. Rejected as scope creep here - it is the right shape for TASK-IMP-122's content fingerprint, and this task must not pre-empt that design.

## Success Metrics

- Primary: zero dangling symlinks and zero registrations naming a removed path, asserted by resolving every surviving pointer after uninstall. Baseline: 5 and 2.
- Guardrail: zero casualties - no path present before install may be missing or mutated after uninstall. Baseline: 0, and it must stay 0.
- Guardrail: byte-count of an operator's hook is constant across N cycles. Baseline: +1 per cycle.

## Scope

In scope: `uninstall.sh` cleanup of the six skill channels, the two MCP registrations, the install-created `.gitignore`, and the hook block separator. Suite arms for each.

### Out of scope / Non-Goals

- What uninstall KEEPS by design (`docs/tasks/`, `docs/status/`, `CHANGELOG.md`, `AGENTS.md`/pointer files, BRAIN). Correct, and untouched.
- The rules pointers (`.cursorrules`, `.windsurfrules`, `GEMINI.md`, `.github/copilot-instructions.md`, `.agents/rules/`, `.devin/rules/`, `.cursor/rules/`). Agent surface, kept per `:92-93`.
- Payload staleness detection - TASK-IMP-122.
- The summary's wording - TASK-IMP-106 owns it.

## Dependencies

depends_on TASK-IMP-106: both edit `uninstall.sh` and `test_install_hygiene.sh`, so they serialise. More importantly 106's AC 3 pins "the set of files present after uninstall is byte-identical to today's behavior" (`t22_uninstall_behavior_unchanged`) - a test that freezes the defects above as correct. This task MUST re-point `t22` at the corrected set; shipping 121 without doing so turns 106's green suite red. Per TASK-IMP-101's evidence gate, 106's coverage-gate artefact is the evidence.

## AI Authorship Disclosure

- **Tools used:** Claude (Opus 4.8) running the CyberOS task-author skill inside Cowork.
- **Scope:** spec drafted from a byte-level harness run on host macOS against `dist/cyberos` 1.0.0; every claim above is a recorded measurement, not a reading of the script. Implementation under ship-tasks supervision.
- **Human review:** scope and the `:111` refinement approved at the 2026-07-18 PLAN gate; both HITL gates are recorded human verdicts.

## 1. Description (normative)

- 1.1 After uninstall, NO symlink under any installed skill channel (`.agents/skills/`, `.claude/skills/`, `.codex/skills/`, `.grok/skills/`, `.commandcode/skills/`, `.opencode/skill/`) may remain whose `readlink` target resolves inside the removed `.cyberos/`. Ownership MUST be established by the readlink target, not by the entry's name.
- 1.2 An entry under those channels whose ownership is NOT provable (a real directory without our marker, or a symlink pointing outside `.cyberos/`) MUST be left in place and reported as kept.
- 1.3 After uninstall, NO MCP registration written by install (`.mcp.json`, `.cursor/mcp.json`) may name a path inside the removed `.cyberos/`. A registration whose command does not reference `.cyberos/` is operator work and MUST be left untouched; a file that becomes empty of registrations as a result MUST NOT survive if install created it.
- 1.4 A `.gitignore` that install created MUST NOT survive uninstall as an empty file. A `.gitignore` that predates install MUST survive byte-identical apart from the removal of the managed block.
- 1.5 Stripping the managed block from an operator's pre-existing hook MUST restore that hook byte-identical to its pre-install content, including trailing bytes.
- 1.6 Uninstall MUST be idempotent over cycles: for any N >= 1, N install/uninstall cycles MUST leave an operator's pre-existing hook byte-identical to its pre-install content.
- 1.7 No path present before install may be missing or mutated after uninstall.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1) - after uninstall on a repo where install wrote all six channels, resolving every surviving entry under those channels yields zero unresolvable targets; the test MUST FAIL if any one channel is left unhandled - test: `tools/install/tests/test_install_hygiene.sh::t23_no_dangling_skill_links`
- [ ] AC 2 (traces_to: #1.2) - an operator-owned unmarked skill dir and a symlink pointing outside `.cyberos/` both survive byte-identical - test: `tools/install/tests/test_install_hygiene.sh::t24_unprovable_ownership_kept`
- [ ] AC 3 (traces_to: #1.3) - after uninstall, no registration in `.mcp.json` or `.cursor/mcp.json` names a path under `.cyberos/`, and a pre-existing registration naming something else survives byte-identical - test: `tools/install/tests/test_install_hygiene.sh::t25_no_dead_mcp_registrations`
- [ ] AC 4 (traces_to: #1.4) - a `.gitignore` created by install does not survive; a `.gitignore` that predated install survives with only the managed block removed - test: `tools/install/tests/test_install_hygiene.sh::t26_gitignore_created_vs_preexisting`
- [ ] AC 5 (traces_to: #1.5, #1.6) - an operator's hook is byte-identical to its pre-install bytes after 1 cycle AND after 3 cycles; the test MUST compare bytes, not line count, and MUST FAIL on the current +1-newline-per-cycle behaviour - test: `tools/install/tests/test_install_hygiene.sh::t27_hook_restore_byte_exact_and_idempotent`
- [ ] AC 6 (traces_to: #1.7) - no path present before install is missing or mutated after uninstall, across a fresh-git repo, a repo with a foreign hook, and a repo with `core.hooksPath` set - test: `tools/install/tests/test_install_hygiene.sh::t28_no_casualties`
- [ ] AC 7 (traces_to: #1.1, #1.3, #1.4) - TASK-IMP-106's `t22_uninstall_behavior_unchanged` is re-pointed at the corrected set and passes; the suite MUST NOT contain an assertion that the pre-121 leftovers are correct - test: `tools/install/tests/test_install_hygiene.sh::t22_uninstall_behavior_unchanged`

## 3. Edge cases

- A channel dir that is empty after its managed entry is removed: pruned only if install created it, by the same rule `:105-107` already applies to `.agents/skills`; an operator's own empty dir is not ours to remove.
- A symlink into `.cyberos/` whose target was ALREADY absent before uninstall (a prior partial install): still provably ours by readlink, still removed. Unresolvable is not the same as unowned.
- `.mcp.json` containing both our registration and an operator's: our key is removed, theirs survives, the file survives. Only a file install created AND that is now empty of registrations is removed.
- An operator hook whose pre-install content already ended in multiple newlines: restore is byte-exact to THAT content - the rule is "identical to what was there", never "normalised".
- `CYBEROS_UNINSTALL_KEEP_BRAIN=0`: `.cyberos/` is fully removed, so every pointer into it dangles. This is the strictest case for 1.1/1.3 and MUST be covered, not the default-BRAIN case alone.
- Uninstall run twice: the second run finds nothing of ours and removes nothing; 1.6's byte-identity still holds.
- Security-class: ownership is decided by `readlink` output and our own marker file, never by a path pattern an operator could forge by naming a directory `ship-tasks`. No user-supplied string is interpolated into a removal command; nothing printed is executed.
