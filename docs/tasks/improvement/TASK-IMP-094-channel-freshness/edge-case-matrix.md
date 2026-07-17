# TASK-IMP-094 edge-case matrix

| # | Category | Trigger | Expected | Covered by |
|---|---|---|---|---|
| 1 | NULL/EMPTY | fresh repo, none of the new paths exist | 3 shared entries + both rules pointers created; legacy `.windsurfrules` also created (its own pre-existing pointer line) | t_shared_skills_and_devin_rules |
| 2 | IDEMPOTENCE | second install over the same repo | byte-identical on every new path (link targets, pointer bytes, gitignore); gitignore entries exactly once | t_channel_idempotence |
| 3 | NO SYMLINKS | filesystem without symlink support (or Windows checkout): `ln -s` fails | plain copy of the payload skill lands; never a dangling link | t_shared_skills_resolve (accepts symlink or copy; install.sh `|| cp -R` + `[ -e ]` post-check) |
| 4 | FILTERED FAMILY | `CYBEROS_AGENTS=claude-code` (excludes agents/devin/windsurf) | none of the new paths created; `.claude/skills` family still lands | t_shared_skills_and_devin_rules (exclusion arm) |
| 5 | FILTER SKEW | `agents` wanted but `claude-code` excluded: no `.claude/skills/<cmd>` counterpart to point at | copy fallback engages (`[ -e .claude/skills/<cmd> ]` pre-check), entry still resolves | install.sh:517-529 branch; resolution class asserted by t_shared_skills_resolve |
| 6 | OPERATOR STATE | operator already has own files under `.devin/rules/` or an own `.agents/skills/<cmd>` | create-if-absent adds only ours; existing entries untouched | pointer() `[ -e ] && return`; shared block `continue` on existing; idempotence arm re-proves no churn |
| 7 | UNINSTALL | uninstall after install | shared entries + `.claude/skills/{task-author,task-audit}` removed (no orphans), dirs pruned only when emptied, rules pointers KEPT (tracked agent surface, like CLAUDE.md) | test_install_hygiene.sh t01 (uninstall arm) |
| 8 | FOREIGN LOOKALIKE | operator's own symlink at `.agents/skills/ship-tasks` pointing at their own skill | uninstall readlink-match fails -> left alone | uninstall.sh 2b case-guard; reviewed (construction: only our two target shapes match) |
| 9 | GITIGNORE ROUND-TRIP | pre-existing managed block with OLD marker wording + two installs | new paths inside the regenerated block exactly once; operator lines survive | t01 (extended assertions) |
| 10 | SECURITY | pointer files / symlinks could leak paths | pointers carry repo-relative prose only; links are relative (`../../...`) and resolve inside the repo | t_shared_skills_resolve (absolute-target + resolves-outside-repo checks) |
| 11 | COPY_SKILLS MODE | `CYBEROS_COPY_SKILLS=1` | shared entries are copies (committable, no symlinks anywhere), gitignore skill section suppressed as today | install.sh:521 condition; existing COPY_SKILLS branch semantics, reviewed |
