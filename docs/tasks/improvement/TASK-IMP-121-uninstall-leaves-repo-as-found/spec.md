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
effort_hours: 8
service: tools/install
new_files:
  - (none)
modified_files:
  - tools/install/uninstall.sh
  - tools/install/tests/test_install_hygiene.sh
source_pages:
  - "2026-07-18 byte-level install/uninstall harness (/tmp/cyberos-harness), fresh-git case: 26 paths -> 2139 installed -> 95 after uninstall; 69 leftovers; 5 dangling symlinks; 2 dead MCP registrations. The 0-casualty / 0-mutation census is per-case over the six cases AS RUN (single cycle, no pre-existing .gitignore) - the two leak findings below sit outside it."
  - "tools/install/uninstall.sh:151 `rm -rf \"$CY\"` then :155-158 `mkdir -p \"$root/.cyberos/memory\"` + `mv \"$KEEP_BRAIN_STASH\" \"$root/.cyberos/memory/store\"` - `.cyberos/` SURVIVES by default; only CYBEROS_UNINSTALL_KEEP_BRAIN=0 removes it outright. Re-verified 2026-07-18."
  - "tools/install/uninstall.sh:94 (loop over ship-tasks task-author task-audit); :98-99 (the readlink ownership test - matches the DIRECT `*.cyberos/plugin/skills/$_sc` and the CHAINED `*.claude/skills/$_sc` forms); :112 (the `[ \"$_sc\" != \"ship-tasks\" ]` guard); :113-114 (the readlink test :112 exempts ship-tasks FROM). Two lines, not one."
  - "tools/install/uninstall.sh:68 (the hook block sed; :70 is its echo); :80 (the .gitignore block sed; :81 is its echo). Both ranges start AT the `# >>>` marker, so the separator install wrote ABOVE it survives."
  - "tools/install/uninstall.sh:118-119 `rmdir .agents/skills` + `rmdir .agents`, `2>/dev/null || true` - the prune precedent. It carries NO install-created check; rmdir's failure on a non-empty dir is the only guard. (:105-107 is the kept-unmarked-dir echo branch, not a prune.)"
  - "tools/install/uninstall.sh:162 `# 6. skill symlinks into .cyberos (dangling) - leave dirs; operator cleans` - the dangling is a RECORDED DECISION, not an oversight."
  - "tools/install/uninstall.sh: `grep -ci` returns 0 for each of mcp, codex, grok, commandcode, opencode, cursor. Four channels and both MCP files are outside the code path entirely. Re-verified 2026-07-18."
  - "tools/install/install.sh:623-627 (.claude/skills .grok/skills .commandcode/skills .codex/skills .opencode/skill - note the SINGULAR 'skill') + :631-632 (.claude/skills task-author + task-audit) + :650-679 (.agents/skills) = six channels"
  - "tools/install/install.sh:613 `ln -s \"$(relup \"$1\").cyberos/plugin/skills/$skill\"` (the direct form); :662 `ln -s \"$(relup \".agents/skills\").claude/skills/$_sc\"` (the chained form)"
  - "tools/install/install.sh:672-675 - `.cyberos-owned` is written ONLY into the .agents/skills copy-fallback. The five native channels' copy fallback (:613's `|| cp -R`) writes NO marker."
  - "tools/install/install.sh:685-688 - mcp_json() emits `node .cyberos/mcp/cyberos-mcp.mjs`; both .mcp.json and .cursor/mcp.json are create-if-absent (`[ -e ... ] ||`), so install NEVER adds our key to an operator's existing file"
  - "tools/install/install.sh:479, :580, :621, :686 - AGENT_FILES / SKILL_DIRS / wrote are SHELL VARIABLES that die with the process. Nothing on disk records what install created. Re-verified 2026-07-18."
  - "tools/install/install.sh:706 `[ -f \"$gi\" ] || : > \"$gi\"` (creates .gitignore when absent); :747 `{ cat tmp; [ -s tmp ] && printf '\\n'; gi_block; } > \"$gi\"` - the `[ -s ]` guard is why the CREATED case leaks no separator and the PRE-EXISTING case leaks one"
  - "tools/install/install.sh:851-853 and :913-915 - `cat >> \"$hk\" <<'HOOK'` whose heredoc FIRST line is blank, then `# >>> cyberos-status-hook v2`. That blank is the separator uninstall.sh:68 leaves behind."
  - "measured 2026-07-18 by this author, install.sh:733-747 + uninstall.sh:80 replayed verbatim on a PRE-EXISTING .gitignore: 14 -> 15 bytes, then 15/15/15 over three cycles (install.sh:745's awk trims the trailing blank run, so this leak is one-time, NOT accumulating). A NEW finding; the draft named only the hook."
  - "measured 2026-07-18 by this author, install.sh:851-886 + uninstall.sh:68 replayed verbatim on a pre-existing hook: 41 -> 42 -> 43 -> 44 bytes over three cycles, final tail `exit 0\\n\\n\\n\\n`. Same +1/cycle invariant and same tail shape the harness proved at 29 -> 30 -> 31 -> 32; the absolute bytes differ only by fixture."
  - "measured 2026-07-18 by this author: a strip that also consumes exactly ONE blank line immediately above the marker is byte-exact for a hook ending in \\n and for one ending in \\n\\n\\n, and is NOT recoverable for a hook with NO trailing newline (16 -> 17 bytes). See §3."
source_decisions:
  - "2026-07-18 Stephen: PLAN gate - fold all four uninstall-reversibility findings into one task (only IMP-115 avoids tools/install, so each task costs a whole batch); template override to task@1 (120/120 corpus)."
  - "2026-07-18 Stephen: uninstall.sh:112's ship-tasks exemption is REFINED, not kept - remove the entry when readlink PROVES it points into the machine we deleted. :113-114 already uses that exact test for task-author/task-audit."
  - "2026-07-18 audit FAIL 4/10 -> whole-document rewrite by a fresh author. The draft was written against a removed .cyberos/ that survives, tested a different predicate than it specified (the TASK-IMP-118 class), conditioned two clauses on a fact nothing records, and mis-cited six lines."
  - "2026-07-18 author (this rewrite): the install-writes-a-receipt idea, floated by the operator and not decided, is REJECTED. Rationale in Alternatives. No clause conditions on 'if install created it'; ownership is proven from the artefact itself at uninstall time, which is what makes §1.1-§1.6 decidable without touching install.sh."
  - "2026-07-18 author (this rewrite): effort 5 -> 8. The draft's 5 predated the .gitignore separator finding, the MCP byte-identical/edited partition, and the t22 rename. Breakdown in Alternatives."
---

# TASK-IMP-121: Uninstall must leave the repo as it found it

## Summary

Uninstall deletes the machine and leaves things pointing at it. A byte-level harness on a fresh git repo found five dangling skill symlinks and two MCP registrations naming a server that is gone. Two further defects are byte-level: stripping our managed block from an operator's pre-commit hook leaves one newline behind every cycle, and doing the same to a pre-existing `.gitignore` leaves one behind once. Nothing here destroys operator content - the harness recorded zero casualties across its six cases - but the repo is left broken rather than merely untidy, and two files an operator owns do not come back byte-identical.

## Problem

Measured 2026-07-18. `uninstall.sh:151` runs `rm -rf "$CY"`; `:155-158` then recreates `.cyberos/memory/` and moves the stashed BRAIN back. **`.cyberos/` survives by default** - what is gone in both modes is `plugin/`, `mcp/`, `cuo/`, `lib/`, `docs-tools/` and the root scripts. Every pointer install writes targets `.cyberos/plugin/skills/...` or `.cyberos/mcp/cyberos-mcp.mjs`, and both live in the part that is removed under `CYBEROS_UNINSTALL_KEEP_BRAIN=1` and `=0` alike. So the question is never "does `.cyberos/` still exist" - it is "does this pointer name a path we deleted", and that is answered by the pointer's own text.

**1. Five dangling skill symlinks.** Install writes six channels: `.agents/skills/`, `.claude/skills/`, `.grok/skills/`, `.commandcode/skills/`, `.codex/skills/`, `.opencode/skill/` (`:623-627`, `:631-632`, `:650-679`; note OpenCode's singular `skill`). Uninstall's loop (`:94`) looks at two. `.agents/skills/ship-tasks` is removed by `:98-99`; `.claude/skills/{task-author,task-audit}` by `:113-114`. `.claude/skills/ship-tasks` is exempted by name at `:112` (`[ "$_sc" != "ship-tasks" ]`), and `.grok`, `.commandcode`, `.codex`, `.opencode` are never looked at - `grep -ci` returns **0** for each of them in `uninstall.sh`. Five survivors, all `-> ../../.cyberos/plugin/skills/ship-tasks`, all confirmed dangling.

**2. Two dead MCP registrations.** `.mcp.json` and `.cursor/mcp.json` both register `node .cyberos/mcp/cyberos-mcp.mjs` (`:685-688`). Uninstall never unregisters - `grep -ci mcp` returns 0. Every agent reading those configs launches a missing server on every startup, in a repo the operator just cleaned.

**3. The hook gains a newline every cycle.** Install appends its block with `cat >> "$hk" <<'HOOK'` whose heredoc's **first line is blank** (`:851-853`), so the file gains `\n# >>> cyberos-status-hook v2`. `uninstall.sh:68`'s sed range starts **at** the marker, so the blank above it survives. Proven by the harness at 29 -> 30 -> 31 -> 32 bytes over three cycles, final tail `exit 0\n\n\n\n`; reproduced independently today at 41 -> 42 -> 43 -> 44 with the same tail. Unbounded: nothing ever trims it.

**4. The same defect in `.gitignore`, which the draft missed.** `install.sh:747` writes `[ -s "$gi.cyberos.tmp" ] && printf '\n'` before the block, and `uninstall.sh:80`'s sed range also starts at the marker. Measured today by replaying both verbatim: a pre-existing `.gitignore` goes **14 -> 15 bytes** and stays at 15 across three cycles - `install.sh:745`'s awk trims the trailing blank run on re-install, so this leak is one-time rather than accumulating. It is still a mutation of an operator's file, and it is why the fix belongs to the strip, not to the hook.

The `[ -s ]` guard at `:747` also explains the separate `.gitignore` symptom the harness saw: where install **created** the file (`:706`), the tmp is empty, no separator is written, and the strip leaves a 0-byte file.

**Against section 6.** `uninstall.sh:162` reads `# 6. skill symlinks into .cyberos (dangling) - leave dirs; operator cleans`. The dangling is **known and deliberate**, and `:111`'s comment records the same posture for `.claude/skills/ship-tasks`. This is a decision to overturn, not a gap to fill. It is overturned because its premise does not survive contact with `:113-114`. The restraint section 6 encodes is the right instinct about *operator files* - and a symlink whose target text names our own machine is not an operator file. `:113-114` already trusts exactly that proof to remove `task-author` and `task-audit`, in the same run that leaves `ship-tasks` dangling. Section 6 is not restraint applied consistently; it is one channel's entry escaping a test its two siblings pass. Leaving a pointer to a file we deleted protects nobody.

**What ownership can and cannot be proven from.** The discipline `:98-99` and `:113-114` use is: read the artefact, decide from what it says. It generalises to every artefact here - a marker-delimited block, an MCP command naming our path, a `.cyberos-owned` file - and it stops exactly at the container. Whether install *created* `.gitignore` or `.mcp.json` is not recorded anywhere: `install.sh`'s `wrote`, `AGENT_FILES` and `SKILL_DIRS` (`:479`, `:580`, `:621`, `:686`) are shell variables that die with the process, and the only on-disk marker, `.cyberos-owned`, is written solely into `.agents/skills` copy-fallback dirs (`:672-675`). An operator who ran `touch .gitignore` before install is byte-indistinguishable from install's `: > "$gi"`. So no clause below conditions on it.

## Proposed Solution

Extend the readlink ownership test to every channel install writes, and apply the same read-the-artefact discipline to the MCP registrations and to both managed blocks. Four proofs, each evaluated at uninstall time from the artefact itself:

| artefact | proof it is ours |
|---|---|
| skill symlink | `readlink` target text naming `.cyberos/plugin/skills/<entry>`, or the chained `.claude/skills/<entry>` form (`:98`) |
| `.agents/skills` copy dir | the `.cyberos-owned` file it carries (`:101`) |
| `.gitignore` / hook block | the `# >>>` / `# <<<` marker pair delimiting it |
| MCP registration | the command naming `.cyberos/mcp/cyberos-mcp.mjs` |

**The predicate is textual, never resolution.** `readlink` returns a string; a target naming our machine proves ownership whether or not it still resolves. This is what makes the rule identical under `CYBEROS_UNINSTALL_KEEP_BRAIN=1` (where `.cyberos/` survives with BRAIN inside) and `=0` (where it does not), and it is why an operator's own broken symlink pointing elsewhere is untouched: unresolvable is not the same as unowned, in either direction.

**What is provably ours is removed; the container is not.** The block comes out of `.gitignore`, the file stays. The registration comes out of `.mcp.json`, the file stays with an empty registry. The symlink goes, the channel dir stays. Removing a container would require knowing install created it, which nothing records - so the line is drawn where the proof runs out. An emptied container is inert; a pointer to a deleted file is not.

**Strip the separator with the block.** Both sed ranges must also consume the single blank line install writes immediately above the `# >>>` marker - exactly one, because install writes exactly one. Verified today: byte-exact for a hook ending in one newline and for one ending in three.

## Alternatives Considered

- **Have install write a receipt of what it created, so uninstall can know rather than guess.** The operator's instinct, floated for this task and not decided; adopted in spirit for TASK-IMP-122, where the underivable fact (what `install.sh` vendors) is load-bearing and has no safe fallback. It is rejected here because the fact it would record is not load-bearing and the fallback is safe. The only clause a receipt changes is "delete the container": everything else is already decided from the artefact. And a receipt never decides it alone - an operator who adds `node_modules/` outside our markers owns that file whatever the receipt says, so the rule is always "receipt says created **and** the file is now empty", and the emptiness test does the work in every case. So the receipt buys the deletion of a 0-byte file. It costs a new cross-script artefact, a format, staleness semantics across re-installs, `install.sh` in `modified_files` - and a new casualty vector: a receipt that wrongly claims creation deletes an operator's file, which is the one outcome §1.8 forbids and which no mechanism can do today. Note also what a receipt does **not** rescue: it says nothing about how to remove one key from JSON an operator has since edited (§1.4), and nothing about a hook that arrived without a trailing newline (§3) - both survive it untouched. Trading a guardrail for tidiness is the wrong direction.
- **Delete an emptied container when it is byte-identical to install's exact output.** Rejected: that is the TASK-IMP-118 defect in miniature. Install's `.mcp.json` is a fixed 5-line string and `install.sh:684`'s own summary points operators at `.cyberos/mcp/README.md` to hand-register, so an operator-authored file can be byte-identical to ours. Byte-identity proves the *content* is ours; it cannot prove the *file* is.
- **Trim trailing blank lines after the strip instead of consuming the separator.** Rejected: an operator hook whose pre-install content legitimately ends in three newlines would come back with one. Measured today - the correct rule removes exactly one blank line immediately above the marker, and that is byte-exact for both shapes.
- **Remove the whole agent surface on uninstall.** Rejected: `.cursorrules`, `GEMINI.md` and the rules pointers may predate install or carry operator edits; unprovable ownership is not a licence to delete. TASK-IMP-094's recorded reasoning, and it still holds.
- **Leave the dangling links and let TASK-IMP-106's summary name them.** Rejected: 106 §1.5 forbids changing behaviour, so it can only describe the breakage honestly. A truthful report of a broken repo is not a fixed repo.
- **Effort 5 -> 8 hours.** The 5 predated three of this task's obligations. Breakdown: channel loop rewrite 1h; MCP partition 1.5h; separator-exact strip 1.5h; `t22` rename + re-point 0.5h; suite 3.5h.

## Success Metrics

- Primary: zero surviving pointers naming a path under `.cyberos/`, asserted by reading every surviving symlink target and every MCP command after uninstall. Baseline: 5 dangling symlinks + 2 dead registrations, measured 2026-07-18.
- Primary: an operator's pre-existing hook and `.gitignore` are byte-identical to their pre-install bytes after 1 and after 3 cycles, asserted with `cmp`. Baseline: hook +1 per cycle (29 -> 30 -> 31 -> 32, harness); `.gitignore` +1 once (14 -> 15, measured today).
- Guardrail: zero casualties - no path present before install may be missing or mutated after uninstall. Baseline: 0 across the harness's six cases; it must stay 0, and the two leak fixes must not create one.
- Guardrail: every container install writes into still exists after uninstall. Baseline: all present today (the defect is what is left *in* them, not that they go missing) - this metric exists to keep the fix from over-reaching.

## Scope

In scope: `uninstall.sh`'s handling of the six skill channels, the two MCP registrations, and the separator on both managed-block strips; the suite arms for each; the rename and re-point of TASK-IMP-106's `t22`.

### Out of scope / Non-Goals

- `install.sh`. This task changes nothing on the install side - see Alternatives on the receipt, and §3 on the no-trailing-newline hook, which is the one defect here that would need it.
- What uninstall KEEPS by design (`docs/tasks/`, `docs/status/`, `CHANGELOG.md`, `AGENTS.md`/pointer files, BRAIN). Correct, and untouched.
- The rules pointers (`.cursorrules`, `.windsurfrules`, `GEMINI.md`, `.github/copilot-instructions.md`, `.agents/rules/`, `.devin/rules/`, `.cursor/rules/`). Agent surface, kept for the same reason as `CLAUDE.md`. (`uninstall.sh:92-93`'s comment records only three of these; the rest are covered by the same reasoning, not by that citation.)
- `uninstall.sh:118-119`'s existing `rmdir` of `.agents/skills` and `.agents`. Existing shipped behaviour, not a defect this task found. It sits in mild tension with §1.5's container rule; §3 records that rather than smuggling a change in.
- Payload staleness detection - TASK-IMP-122.
- The summary's wording - TASK-IMP-106 owns it.

## Dependencies

depends_on TASK-IMP-106: both edit `uninstall.sh` and `test_install_hygiene.sh`, so they serialise. 106 lands first, creating `t22_uninstall_behavior_unchanged`, whose AC 3 pins "the set of files present after uninstall is byte-identical to today's behavior" - a test that freezes every defect above as correct. This task MUST re-point and rename it (§1.9); shipping 121 without doing so turns 106's green suite red. 106 §1.5 keeps its verification: the re-pointed test still fails any summary change that alters what uninstall removes or keeps, which is the whole of what §1.5 asks. Per TASK-IMP-101's evidence gate, 106's coverage-gate artefact is the evidence.

## AI Authorship Disclosure

- **Tools used:** Claude (Opus 4.8) running the CyberOS task-author skill inside Cowork.
- **Scope:** rewritten in full 2026-07-18 by a different author after an independent audit failed the prior draft at 4/10 for writing against a `.cyberos/` that survives, asserting a weaker predicate in AC 1 than §1.1 specified, conditioning §1.3/§1.4 on a fact nothing records, and mis-citing six lines. This is a whole-document rewrite, not a patch. Every line number and every number above was re-verified against source at HEAD by this author during this rewrite; the six mis-cited lines are corrected in `source_pages` with the true mechanism named beside each. Two claims are new measurements of my own (the `.gitignore` separator leak; the byte-exactness of the candidate strip), and one clause was narrowed because measurement showed it could not be honoured as written (§1.6, §3). Claims that could not be measured are recorded as gaps in §3 rather than assigned to a clause.
- **Human review:** scope and the `:112` refinement approved at the 2026-07-18 PLAN gate; both HITL gates are recorded human verdicts. The receipt decision is this author's, with rationale in Alternatives; the operator floated it and left it open.

## 1. Description (normative)

- 1.1 After uninstall, no symlink under any channel install writes (`.agents/skills/`, `.claude/skills/`, `.grok/skills/`, `.commandcode/skills/`, `.codex/skills/`, `.opencode/skill/`) may remain whose `readlink` target names `.cyberos/plugin/skills/<entry>` or the chained `.claude/skills/<entry>` form (`:98`). Ownership MUST be decided from the target TEXT, never from whether the target resolves and never from the entry's name; no entry may be exempt by name, which removes `:112`'s `ship-tasks` guard.
- 1.2 An entry under those channels whose ownership is proven by neither §1.1's target test nor a `.cyberos-owned` marker it carries (`:101`) MUST be left byte-identical and reported as kept - including a real directory without our marker, a symlink pointing outside `.cyberos/`, and a BROKEN symlink pointing outside `.cyberos/`.
- 1.3 Where `.mcp.json` or `.cursor/mcp.json` is byte-identical to the form `install.sh:685` generates - our registration and nothing else - the registration naming `.cyberos/mcp/` MUST be removed, leaving the file present and carrying a registry with no entries.
- 1.4 Where `.mcp.json` or `.cursor/mcp.json` carries our registration but is NOT byte-identical to that form, the file has content install did not write; uninstall MUST NOT edit it, and MUST report the dead registration and the file's path so the operator can remove it by hand.
- 1.5 A container install writes into MUST survive uninstall even when emptied of our content: `.gitignore` (including one install created at `:706`), `.mcp.json`, `.cursor/mcp.json`, and each of §1.1's six channel directories. No container may be removed on the grounds that it is now empty.
- 1.6 Stripping a managed block from a file that predates install MUST restore that file byte-identical to its pre-install content, consuming the single blank separator install writes immediately above the `# >>>` marker (`install.sh:747` for `.gitignore`, `:851-853`/`:913-915` for the hook) and consuming exactly one, so content that legitimately ended in blank lines survives. This clause binds where the pre-install content ends with a newline; §3 records why the remaining case is not recoverable from `uninstall.sh`.
- 1.7 Uninstall MUST be idempotent over cycles: for any N >= 1, N install/uninstall cycles MUST leave an operator's pre-existing hook and `.gitignore` byte-identical to their pre-install content.
- 1.8 No path present before install may be missing or mutated after uninstall.
- 1.9 The suite MUST NOT assert that the pre-121 leftovers are correct. TASK-IMP-106's `t22_uninstall_behavior_unchanged` MUST be re-pointed at the set §1.1-§1.6 define and renamed, because a test whose baseline has deliberately moved cannot honestly be named `_unchanged`.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1) - after an install writing all six channels, every surviving symlink under them is read with `readlink` and NONE has a target naming `.cyberos/plugin/skills/<entry>` or `.claude/skills/<entry>`; the test MUST assert all six channels separately and MUST FAIL if any entry is exempt by name, which today's `:112` guard makes true of `ship-tasks` - test: `tools/install/tests/test_install_hygiene.sh::t23_no_managed_skill_links_survive`
- [ ] AC 2 (traces_to: #1.2) - an operator's unmarked skill dir, an operator symlink resolving outside `.cyberos/`, AND an operator's BROKEN symlink pointing outside `.cyberos/` all survive byte-identical and are named as kept; the test MUST FAIL if an unresolvable target alone causes removal - test: `tools/install/tests/test_install_hygiene.sh::t24_unprovable_entries_kept_and_named`
- [ ] AC 3 (traces_to: #1.3) - where `.mcp.json` and `.cursor/mcp.json` are byte-identical to install's generated form, after uninstall each file EXISTS, parses, carries zero registry entries, and contains no string naming `.cyberos/` - test: `tools/install/tests/test_install_hygiene.sh::t25_sole_cyberos_registration_removed`
- [ ] AC 4 (traces_to: #1.4) - an `.mcp.json` carrying our registration plus an operator's own is byte-identical after uninstall AND the dead registration is reported with the file's path; the test MUST FAIL if the file is edited at all - test: `tools/install/tests/test_install_hygiene.sh::t26_operator_edited_mcp_kept_and_named`
- [ ] AC 5 (traces_to: #1.5) - a `.gitignore` install created, an `.mcp.json` reduced to an empty registry, and each of the six channel dirs all still EXIST after uninstall; the test MUST FAIL if any is removed for being empty - test: `tools/install/tests/test_install_hygiene.sh::t27_emptied_containers_survive`
- [ ] AC 6 (traces_to: #1.6) - a pre-existing newline-terminated pre-commit hook AND a pre-existing newline-terminated `.gitignore` are byte-identical to their pre-install bytes after one cycle, asserted with `cmp`; the test MUST include a hook whose content already ends in three newlines and MUST FAIL on today's leak (hook 29 -> 30, `.gitignore` 14 -> 15) - test: `tools/install/tests/test_install_hygiene.sh::t28_block_strip_restores_byte_exact`
- [ ] AC 7 (traces_to: #1.7) - after 3 install/uninstall cycles the hook and the `.gitignore` are still byte-identical to their pre-install bytes; the test MUST compare bytes and not line counts, and MUST FAIL on today's 29 -> 30 -> 31 -> 32 hook accumulation - test: `tools/install/tests/test_install_hygiene.sh::t29_strip_idempotent_over_cycles`
- [ ] AC 8 (traces_to: #1.8) - no path present before install is missing or mutated after uninstall across a fresh-git repo, a repo with a foreign hook, a repo with `core.hooksPath` set, and a repo uninstalled with `CYBEROS_UNINSTALL_KEEP_BRAIN=0` - test: `tools/install/tests/test_install_hygiene.sh::t30_no_casualties`
- [ ] AC 9 (traces_to: #1.9) - `t22` is renamed `t22_uninstall_removal_set_pinned`, pins the set §1.1-§1.6 define, and passes; the test MUST FAIL if any assertion in the suite still pins a pre-121 leftover as correct - test: `tools/install/tests/test_install_hygiene.sh::t22_uninstall_removal_set_pinned`

## 3. Edge cases

- `CYBEROS_UNINSTALL_KEEP_BRAIN=0` vs the default: in the default path `.cyberos/` SURVIVES (`:151` removes it, `:155-158` recreates it with BRAIN inside); under `=0` it does not. §1.1's predicate is textual, so it is the same rule in both, and both are covered by AC 8. This is the premise the prior draft inverted; the fix is not to pick the other mode but to stop conditioning on the mode at all.
- A symlink into `.cyberos/` whose target was ALREADY absent before uninstall (a prior partial install): still provably ours by target text, still removed. Unresolvable is not unowned.
- The CHAINED form's proof is weaker than the direct form's: `.agents/skills/<cmd> -> ../../.claude/skills/<cmd>` names no `.cyberos/` path, so it is ours by construction (install writes exactly that shape at `:662`) rather than by target. An operator symlink of that exact shape would be removed. This is TASK-IMP-094's existing shipped behaviour at `:98`, carried forward unchanged and named here rather than discovered later.
- `CYBEROS_COPY_SKILLS=1`: the five native channels' copy fallback (`:613`'s `|| cp -R`) writes NO `.cyberos-owned` marker - only `.agents/skills` gets one (`:672-675`). So a copied `ship-tasks` under `.claude/skills/` is not provably ours and §1.2 keeps and reports it. It is untidy, not broken: a copy is self-contained and dangles at nothing. Closing it means marking the copy at install time, which is an `install.sh` change and out of scope here.
- **An operator hook with NO trailing newline: byte-exact restore is IMPOSSIBLE from `uninstall.sh` and §1.6 does not claim it.** Install's `cat >>` appends `\n# >>> ...`; against `exit 0` (no newline) that `\n` becomes the terminator of the operator's last line, and the result `exit 0\n# >>> ...` is indistinguishable from what a newline-terminated file with no separator would produce. The information is destroyed at append time, so no uninstall-side rule can invert it - measured today at 16 -> 17 bytes with the candidate strip. The residue is a +1-byte gain on a file that is not a POSIX text file, and closing it means changing install's append. Recorded as an uncovered gap, not handed to a clause that cannot honour it.
- `uninstall.sh:118-119` already `rmdir`s `.agents/skills` and `.agents` when they empty, with no ownership check - `rmdir`'s failure on a non-empty dir is the only guard. That is in mild tension with §1.5, which keeps the other five channel dirs. It is existing shipped behaviour, this task does not extend it to the five, and it is named here rather than quietly widened or quietly reversed.
- `.mcp.json` containing both our registration and an operator's: §1.4, not §1.3 - the file is not byte-identical to install's form, so it is left alone and the dead registration is reported. Surgically removing one key from arbitrary JSON needs a parser `uninstall.sh` does not have and `node` it must not require; reporting is what it can honestly do, and TASK-IMP-106's kept-list is the idiom for saying it.
- An operator hook whose pre-install content already ended in multiple newlines: restore is byte-exact to THAT content - §1.6 consumes exactly one blank line, never normalises. Measured byte-exact today for a hook ending in three newlines.
- `CYBEROS_GLOBAL_SKILLS=1` copies `ship-tasks` into `$HOME/.claude/skills` and three siblings (`:634-637`); uninstall never touches them. Outside the repo is outside this task's contract ("leave the repo as it found it"); named so it is not mistaken for coverage.
- Uninstall run twice: the second run finds nothing provably ours and removes nothing; §1.7's byte-identity still holds.
- Security-class: ownership is decided by `readlink` output, our own marker file, our marker pair, and a command string naming our path - never by a path pattern an operator could forge by naming a directory `ship-tasks`. No user-supplied string is interpolated into a removal command; nothing printed is executed.
