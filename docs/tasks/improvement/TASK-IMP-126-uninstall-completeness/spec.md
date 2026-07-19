---
id: TASK-IMP-126
title: Uninstall completeness - MCP, dangling skill links, hook newline
template: task@1
type: improvement
module: improvement
status: done
priority: p2
author: "@stephencheng"
department: engineering
created_at: 2026-07-19T00:00:00Z
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-IMP-083, TASK-IMP-094]
routed_back_count: 0
awh: N/A
verify: T
phase: "post-1.0.0"
owner: Stephen Cheng (CTO)
created: 2026-07-19
memory_chain_hash: null
effort_hours: 5
service: tools/install
new_files:
  - (none)
modified_files:
  - tools/install/uninstall.sh
  - tools/install/tests/test_install_hygiene.sh
source_pages:
  - "tools/install/install.sh:694-697 (writes .mcp.json + .cursor/mcp.json - the MCP registration)"
  - "tools/install/uninstall.sh (zero mcp lines - grep -ci mcp = 0; no unregistration exists)"
  - "tools/install/install.sh:632-637 (installs skills for grok/command-code/codex/opencode families) + :644 ($HOME global installs behind CYBEROS_GLOBAL_SKILLS)"
  - "tools/install/uninstall.sh:98-127 (removes only .agents/skills/{ship-tasks,task-author,task-audit} + .claude/skills/{task-author,task-audit}; :125 explicitly leaves .claude/skills/ship-tasks)"
  - "tools/install/install.sh:860-861 (heredoc append with a leading blank separator before the >>> marker) vs tools/install/uninstall.sh:78 (sed range-strip from >>> to <<< inclusive - the leading blank is outside the range)"
source_decisions:
  - "2026-07-19 Stephen: PLAN gate - option 1 (author the three uninstall findings as one combined task with a single uninstall.sh cone; treat rules_sha #9 as accepted by-design)."
---

# TASK-IMP-126: Uninstall completeness - MCP, dangling skill links, hook newline

## Summary

`uninstall.sh` is not the inverse of `install.sh` in three places. It never removes the
MCP registration files install writes, it cleans only two of the agent families' skill
entries and leaves the rest pointing at the deleted machine, and its hook-strip leaves a
blank line behind that accumulates on every install/uninstall cycle. Make uninstall the
exact inverse of install for these three artifacts, proven by a fixture that installs,
uninstalls, and asserts the tree is clean.

## Problem

Three verified gaps, each a place where install writes and uninstall does not remove.

**Dead MCP registrations.** `install.sh:694-697` writes `.mcp.json` (and `.cursor/mcp.json`
when the cursor agent is selected), each pointing at `.cyberos/mcp/cyberos-mcp.mjs`.
`uninstall.sh` contains zero MCP handling (`grep -ci mcp` = 0). After uninstall removes
`.cyberos/`, those files survive and register an MCP server whose entry point no longer
exists - a broken registration every MCP-capable agent reads.

**Dangling skill links.** `install.sh:632-637` installs the workflow skills for the
claude-code, grok, command-code, codex, and opencode families, each a symlink (or
copy-fallback) into `.cyberos/plugin/skills/`. `uninstall.sh:98-127` removes only the
`.agents/skills` trio and the `.claude/skills` create-tasks pair, and `:125` explicitly
leaves `.claude/skills/ship-tasks`. The grok, command-code, codex, and opencode entries
are never touched. After `.cyberos/` is removed, every untouched managed link dangles.

**Hook-strip newline leak.** `install.sh:860-861` appends the managed block to a foreign
pre-commit hook via a heredoc whose first line is a blank separator, writing
`\n# >>> cyberos-status-hook v2 ...>>>\n...\n# <<< cyberos-status-hook <<<\n`.
`uninstall.sh:78` strips the marked range with `sed '/# >>> .../,/# <<< ...<<</d'` -
inclusive of both markers but not the leading blank. The separator survives, so each
install/uninstall cycle on a foreign hook accumulates one blank line at the append point.
The v1->v2 upgrade path (`install.sh:856`) shares the pattern.

## Proposed Solution

Make uninstall the inverse of install for all three, keeping the whole change in
`uninstall.sh` plus arms in the existing `test_install_hygiene.sh`:

- Remove `.mcp.json` and `.cursor/mcp.json` when they are the cyberos-written form, never
  when they are an operator's own file (same ownership discipline the skill and hook
  sections already use - match the exact content install writes, or a marker).
- Extend the skill-cleanup loop to cover every family install writes (grok, command-code,
  codex, opencode) plus `.claude/skills/ship-tasks`, removing an entry only when it is ours
  by construction: a symlink whose `readlink` target resolves into `.cyberos/plugin/skills`,
  or a copy carrying the `.cyberos-owned` marker.
- Widen the hook-strip so it consumes the leading blank separator, leaving a foreign hook
  byte-identical to its pre-install content across any number of cycles.

## Alternatives Considered

- Fix the newline on the install side (drop the leading blank from the heredoc). Rejected:
  the blank is a deliberate readability separator in a hook an operator reads, and the
  defect is that uninstall does not undo what install does - the inverse belongs in
  uninstall, next to the strip that already exists.
- Three separate tasks, one per finding. Rejected at the PLAN gate: all three live in
  `uninstall.sh` and would serialise on one file with no benefit (recorded 2026-07-19).
- Have uninstall shell out to a manifest of what install wrote. Rejected: no such manifest
  exists, and ownership-by-construction (readlink target / marker / exact content) is the
  discipline the file already uses and is enough.

## Success Metrics

- Primary: a fixture install then uninstall leaves zero cyberos-owned artifacts - no
  `.mcp.json`/`.cursor/mcp.json` we wrote, no skill link resolving into the removed machine,
  and a foreign hook byte-identical to before install - suite-asserted. Baseline: all three
  survive today.
- Guardrail: no operator file is removed - an operator's own `.mcp.json`, an unmarked
  `.agents/skills/<cmd>` dir, and the foreign hook's own lines all stay (the existing
  spec 1.3 "never touch operator files" promise is not weakened).

## Scope

In scope: `uninstall.sh`'s MCP-file removal, the skill-cleanup coverage, the hook-strip
separator, and arms in `test_install_hygiene.sh`.

### Out of scope / Non-Goals

- Any change to `install.sh` (what install writes is correct; uninstall is what is missing).
- The `$HOME` global installs behind `CYBEROS_GLOBAL_SKILLS=1` (opt-in, default off) - a
  follow-up may cover them, but the default-path fleet does not create them and this task
  stays on the always-created artifacts.
- The `rules_sha` fingerprint (TASK covered separately; judged by-design at the PLAN gate).
- Re-installing the fleet (that is Goal 4, an operator-gated action).

## Dependencies

None blocking. Touches `uninstall.sh` (shared surface with TASK-IMP-083 hook logic and
TASK-IMP-094 skill logic - both already landed; this extends their inverse coverage).

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS task-author skill inside Cowork.
- **Scope:** spec drafted from the 2026-07-19 fleet-health pass; each of the three gaps was
  verified against the live `install.sh` and `uninstall.sh` at HEAD with the exact line
  numbers cited in source_pages. The handoff's "5 dangling symlinks" count was deliberately
  not adopted as a normative number - it cannot be verified without a live install, so 1.2
  specifies the invariant (zero dangling links) and the test proves it against a fixture.
- **Human review:** scope and granularity approved at the 2026-07-19 PLAN gate; both HITL
  gates are recorded human verdicts.

## 1. Description (normative)

- 1.1 `uninstall.sh` MUST remove `.mcp.json` and, when present, `.cursor/mcp.json` when they
  are the cyberos-written form (their content matches what `install.sh` writes, or they
  carry a cyberos marker), and MUST NOT remove an operator's own file of either name.
- 1.2 `uninstall.sh` MUST remove every managed skill entry install created that resolves
  into the removed machine - across all agent families install writes (claude-code, grok,
  command-code, codex, opencode) plus `.claude/skills/ship-tasks` - leaving zero symlinks
  that point into `.cyberos/plugin/skills` after the machine is removed. Ownership MUST be
  proven by construction (readlink target resolving into `.cyberos`, or the `.cyberos-owned`
  marker); an unmarked operator dir MUST be left in place.
- 1.3 The hook-strip MUST be the exact inverse of install's append: it MUST also remove the
  leading blank separator install writes before the `>>>` marker, so a foreign pre-commit
  hook is byte-identical to its pre-install content after uninstall, and stays byte-identical
  across any number of install/uninstall cycles.
- 1.4 None of 1.1-1.3 may remove an operator file: an operator-authored `.mcp.json`, an
  `.agents/skills/<cmd>` dir without the `.cyberos-owned` marker, or any line of a foreign
  hook outside the managed block MUST survive uninstall.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1) - a fixture install writes `.mcp.json`; uninstall removes it; a pre-existing operator `.mcp.json` is left untouched - test: `tools/install/tests/test_install_hygiene.sh::t_mcp_registration_removed`
- [ ] AC 2 (traces_to: #1.2) - after a fixture install then uninstall, zero skill links resolve into the removed `.cyberos/plugin/skills` (checked across every family install writes) - test: `tools/install/tests/test_install_hygiene.sh::t_no_dangling_skill_links`
- [ ] AC 3 (traces_to: #1.3) - install then uninstall then install then uninstall on a foreign pre-commit hook leaves it byte-identical to the original (no accumulated blank line) - test: `tools/install/tests/test_install_hygiene.sh::t_hook_strip_byte_identical_across_cycles`
- [ ] AC 4 (traces_to: #1.4) - an operator `.mcp.json`, an unmarked `.agents/skills/<cmd>` dir, and a foreign hook's own lines all survive uninstall - test: `tools/install/tests/test_install_hygiene.sh::t_operator_files_preserved`

## 3. Edge cases

- The v1->v2 hook upgrade path (`install.sh:856`) shares the leading-blank append shape;
  the widened strip MUST heal a v1 leftover too, not only a v2 block.
- `.cursor/mcp.json` exists only when the cursor agent was selected at install; uninstall
  MUST remove it when present and MUST NOT fail when absent.
- Skill entries that landed as copy-fallbacks (no symlink support, or `CYBEROS_COPY_SKILLS=1`)
  are removed only when they carry the `.cyberos-owned` marker - a copy an operator made is
  byte-indistinguishable without it and MUST stay.
- A `hooksPath` repo (core.hooksPath set) writes the hook outside `.git/hooks`; the strip
  MUST operate on the effective hooks dir the same way install and the existing strip do.
- Security-class: uninstall reads paths and file content and executes nothing. Paths are
  confined under the repo root on the same `relUnderRoot` rule the other helpers use; a
  crafted target cannot walk out.
