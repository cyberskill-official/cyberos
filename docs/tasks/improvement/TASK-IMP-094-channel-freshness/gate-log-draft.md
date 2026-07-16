# TASK-IMP-094 gate-log evidence (implementing -> ready_to_review)

E1 - scratch install surface (AC 1, AC 4; payload built from this tree, consumer scratch repo):
  $ ls -la <repo>/.agents/skills/
  ship-tasks  -> ../../.claude/skills/ship-tasks     (SKILL.md: yes)
  task-author -> ../../.claude/skills/task-author    (SKILL.md: yes)
  task-audit  -> ../../.claude/skills/task-audit     (SKILL.md: yes)
  $ head -3 .devin/rules/cyberos.md
  # cyberos
  This repo runs **CyberOS**. Canonical agent instructions: `AGENTS.md` (root) and `.cyberos/AGENT-ENTRY.md`.
  (.windsurf/rules/cyberos.md same body; legacy .windsurfrules present)
  managed .gitignore block gained, inside the markers:
  .claude/skills/task-author  .claude/skills/task-audit
  # shared .agents/skills entries (Agent Skills open standard) chain via .claude/skills (TASK-IMP-094)
  .agents/skills/ship-tasks  .agents/skills/task-author  .agents/skills/task-audit
  summary lines:
  pointer files: ... .windsurfrules .devin/rules/cyberos.md .windsurf/rules/cyberos.md
  native skills: ... .claude/skills/task-author .claude/skills/task-audit .agents/skills/ship-tasks .agents/skills/task-author .agents/skills/task-audit

E2 - channels suite (AC 1, 2, 4) verbatim tail:
  ok   t_shared_skills_and_devin_rules: 3 shared entries + both rules pointers, legacy kept, filter honored
  ok   t_channel_idempotence: second install byte-identical on the new paths (gitignore entry x1)
  ok   t_shared_skills_resolve: every entry resolves in-repo (symlink or copy; no dangling links)
  ...
  channels: pass=24 fail=0        (was 21 at baseline; +3 = these scenarios)

E3 - hygiene suite (AC 3) verbatim tail:
  ok   t01
  ...
  ok   t08_gates_env_regen_notice
  ok   t09_nongit_summary_line
  install-hygiene: 19 passed, 0 failed   (was 17 at baseline; t01 now carries the new-path
                                          round-trip + uninstall assertions)

E4 - idempotence spot proof: t_channel_idempotence's normalized snapshot (link targets,
  pointer/gitignore bytes, copy manifests) is cmp-identical across installs #1/#2 and
  `.agents/skills/ship-tasks` appears exactly once in .gitignore.

E5 - uninstall strip (AC 3), live scratch run:
  removed .agents/skills/ship-tasks (managed entry)
  removed .agents/skills/task-author (managed entry)
  removed .claude/skills/task-author (managed entry)
  removed .agents/skills/task-audit (managed entry)
  removed .claude/skills/task-audit (managed entry)
  post-state: .agents/skills/ pruned (dir gone), .devin/rules/cyberos.md KEPT (tracked
  pointer surface), zero '>>> cyberos' blocks left in .gitignore, .claude/skills/ship-tasks
  untouched per today's documented section-6 behavior.

## PR-review addendum (2026-07-17, Devin Review x2)

F4 (doc staleness, fixed): the managed-gitignore policy comment's TRACKED enumeration now
names the new pointer files (.devin/rules/cyberos.md, .windsurf/rules/cyberos.md).
F5 (behavioral nuance, documented in place): a shared-skills entry that landed as a COPY
(counterpart filtered off at first install) stays a copy on later installs - deliberate
create-if-absent idempotence; the block now carries the comment stating it and the
re-vendor path for anyone wanting the symlink form. Hygiene 19/19 after both.
