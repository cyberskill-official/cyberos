# TASK-IMP-094 code review

Reviewer: ship-tasks batch-4 install-trio agent. Diff: `tools/install/install.sh` (install_skill parameterized + 2 counterpart calls + shared-dir block + 2 pointer lines + gitignore lines + summary prose), `tools/install/uninstall.sh` (section 2b), `tools/install/tests/test_channels.sh` (harness + 3 scenarios), `tools/install/tests/test_install_hygiene.sh` (t01 extended).

## Disclosure: "the three commands" mapping

The spec (1.1) requires shared entries "for the three commands" pointing at `.claude/skills/` counterparts. The three user-facing commands are `/install`, `/create-tasks`, `/ship-tasks` (RELEASE-CHECKLIST.md B3); only ship-tasks exists as a skill dir, `/create-tasks` is implemented by the task-author + task-audit skill pair, and `/install` has no skill (it is the installer itself; inventing one is out of scope - "no new skill content"). Implemented set: **ship-tasks, task-author, task-audit** - the three existing payload skills behind the workflow commands. Checklist line E3 says `.agents/skills/ship-tasks` (singular); the spec broadened it, and this mapping is the only one satisfying 1.1 + AC 4 without new content. The `/create-tasks` pair therefore also lands under `.claude/skills/` (the counterparts the clause presupposes) - the one deliberate widening beyond E3's literal text, disclosed here.

## Clause -> proof

| Clause | Requirement | Proof |
|---|---|---|
| 1.1 | shared entries, relative links, copy fallback, CYBEROS_AGENTS applies | install.sh:516-531 (`want_agent agents`, `ln -s "$(relup ...)"` + `[ -e ]` post-check, `cp -R` fallback); scratch run: 3 relative links `../../.claude/skills/<cmd>`, all resolving (gate log E1); t_shared_skills_and_devin_rules + t_shared_skills_resolve ok |
| 1.2 | both rules pointers, legacy kept | install.sh:462-464; scratch run shows `.devin/rules/cyberos.md`, `.windsurf/rules/cyberos.md`, `.windsurfrules` (gate log E1); same `md` body as `.agents/rules/cyberos.md` |
| 1.3 | gitignore covers new paths; uninstall removes them, operator files untouched | install.sh:560-574 (lines inside the managed block); uninstall.sh:85-112 ours-only strip + dir prune; hygiene t01 extended (new-paths x1 + uninstall arm) ok; gate log E5 |
| 1.4 | re-install idempotent on new paths | t_channel_idempotence ok (normalized snapshot cmp + gitignore-entry count) |
| 1.5 | coverage in the two named suites | channels 24 pass / hygiene 19 pass, scenario names in the tails (gate log E2/E3) |

## Judgment

- **Correctness vs ticket**: every convention the 2026-07-16 research recorded now lands on install: the shared dir (5 agents), the Devin-preferred + Windsurf-fallback rules pair, legacy file kept. E3's implementation row can flip with this landing commit.
- **Blast radius**: install_skill's parameterization keeps all five existing call sites byte-compatible (default arg); the only behavior additions are create-if-absent paths. `build.sh`'s manifest `native_skill_dirs` line is sibling-owned this round and does not yet name `.agents/skills` - left for the C1 pre-tag matrix re-verify, noted in context-map.
- **Failure mode if wrong**: a dangling shared entry (guarded by the `[ -e ]` post-check + copy fallback, asserted by t_shared_skills_resolve) or gitignore churn (t01 blocks=1 + per-path count=1) or uninstall eating operator work (ours-only readlink/SKILL.md tests; foreign-lookalike case reviewed in the edge matrix).
- **Deliberate asymmetries, disclosed**: (a) uninstall keeps the two rules pointers - they are tracked agent surface, exactly like CLAUDE.md/.windsurfrules which uninstall has always kept; "remove them" in 1.3 is read as the gitignore-covered artifacts, and the t01 arm pins that reading (`.devin/rules/cyberos.md` asserted present post-uninstall). (b) `.claude/skills/ship-tasks` keeps today's leave-in-place behavior (uninstall section 6); only the paths NEW with this task are stripped - "no orphan under any new path" holds.
- **Security**: pointer bodies are repo-relative prose; links are relative and proven to resolve inside the repo (absolute-target + outside-repo checks in t_shared_skills_resolve).
- **Scenario naming**: AC names t_shared_skills_and_devin_rules / t_channel_idempotence / t_shared_skills_resolve and hygiene t01 - all landed under exactly those names, no remap.

Verdict: no open findings.

HALT: review acceptance (reviewing -> ready_to_test) is a human gate.
