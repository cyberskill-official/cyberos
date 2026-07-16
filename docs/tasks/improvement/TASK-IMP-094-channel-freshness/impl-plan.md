# TASK-IMP-094 implementation plan

1. **Generalize install_skill** (enables 1.1) - `install.sh:473-489`: third parameter `$3 = skill` (default ship-tasks), so the existing five native-dir calls stay byte-identical in behavior while the `/create-tasks` pair can land beside ship-tasks. Refresh guard, COPY_SKILLS branch and relative-link mechanics unchanged, just parameterized.
2. **Counterparts** (1.1 precondition) - `install.sh:495-499`: `install_skill .claude/skills claude-code task-author|task-audit` so the shared entries have in-repo `.claude/skills/<cmd>` targets ("the three commands": ship-tasks for /ship-tasks, the author+audit pair for /create-tasks - see code-review disclosure).
3. **Shared dir** (clause 1.1) - `install.sh:506-531`: `.agents/skills/<cmd>` create-if-absent behind `want_agent agents`; relative symlink `../../.claude/skills/<cmd>` with `[ -e ]` resolution post-check; plain-copy fallback from the payload skill wherever a resolving link cannot exist.
4. **Rules pointers** (clause 1.2) - `install.sh:462-464`: `pointer devin .devin/rules/cyberos.md md` + `pointer windsurf .windsurf/rules/cyberos.md md`; the legacy `.windsurfrules` pointer line stays first. Same body pattern as `.agents/rules/cyberos.md` (style `md`).
5. **Gitignore block** (clause 1.3) - `install.sh:560-574`: the skill-symlink list gains `.claude/skills/{task-author,task-audit}` and a commented `.agents/skills/*` trio; lines live inside the markers so strip/regenerate round-trips.
6. **Uninstall strip** (clause 1.3) - `uninstall.sh:85-112` section 2b: remove ours-only (readlink target match, or fallback copy with SKILL.md), prune emptied dirs, keep rules pointers (tracked surface) and keep `.claude/skills/ship-tasks` (today's documented leave-in-place behavior).
7. **Coverage** (clauses 1.4, 1.5) - test_channels.sh: scratch build+install harness + t_shared_skills_and_devin_rules / t_channel_idempotence / t_shared_skills_resolve; hygiene t01 extended with the new-path round-trip + uninstall arm.
8. **Gates** - both suites green (channels 24, hygiene 19), scratch-run evidence in the gate log.

Order matters: 1 before 2 (parameter exists before the calls), 2 before 3 (counterparts before links), 3-6 in file order, tests last. TASK-IMP-095/096 edit the same file afterwards - serial in this agent.
