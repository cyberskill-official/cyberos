# TASK-IMP-094 repo context map

## Cone
- `tools/install/install.sh` (step 5b agent surface: pointer table, install_skill, new shared-dir block; step 6 managed gitignore block)
- `tools/install/uninstall.sh` (new section 2b: shared-entry + counterpart strip)
- `tools/install/tests/test_channels.sh` (three new scenarios + a scratch build/install harness)
- `tools/install/tests/test_install_hygiene.sh` (t01 extended: gitignore round-trip over the new paths + uninstall arm)

## Patterns the change must follow
- **Create-if-absent everywhere**: step 5b never clobbers an operator file. `pointer()` returns on `[ -e ]`; the shared-dir block skips any existing entry; both reused as-is.
- **Relative symlinks with copy fallback**: `install_skill` is the house pattern (`ln -s "$(relup ...)"... || cp -R`); generalized to a `$3 = skill` parameter instead of duplicating it. `relup()` computes the `../` chain, so links stay repo-relative.
- **want_agent filtering**: every new path sits behind a family key (`devin`, `windsurf`, `agents`, `claude-code`), same `CYBEROS_AGENTS` semantics as the existing table.
- **ONE managed gitignore block, regenerated in place**: new paths are lines INSIDE the markers, so the existing shape-match strip round-trips them for free (the 21-repo duplicate-block lesson).
- **Ownership tests before removal**: uninstall removes only what it can prove it wrote (readlink target match, or a fallback copy under the exact command name carrying SKILL.md) - same doctrine as the line-2 hook ownership test.

## Blast radius
- Files: 4 modified, 0 new. Modules: 1 (tools/install) + its two suites.
- New consumer surface: `.agents/skills/{ship-tasks,task-author,task-audit}`, `.devin/rules/cyberos.md`, `.windsurf/rules/cyberos.md`, and the `/create-tasks` pair landing beside ship-tasks under `.claude/skills/` (the counterparts the shared entries chain through). Legacy `.windsurfrules` untouched.
- Cross-module edges: install.sh + both suites are shared with TASK-IMP-095/096 - one agent, serial, per the batch plan cone rule. `build.sh` writes the manifest's `native_skill_dirs` line and is sibling-owned this round - deliberately NOT extended (noted for the C1 pre-tag matrix re-verify).

## Module placement
Correct. `improvement` is the cross-cutting hardening module; this is install-channel freshness, not a product surface.
