# `backlog-state-update-author` - invariants

Lifted from SKILL.md's normative prose (FR-SKILL-118 AC 2 discipline: no invariant without a prose source).

1. FR frontmatter is the record of truth; BACKLOG.md is the index - the mutation never inverts that.
2. Exactly one tracked mutation per artefact; whole-file rewrites are the regenerator's job, never this skill's.
3. Operators may override any cell at any time; the skill writes only the default workflow-driven transition.
4. @1 artefacts (no mutation_kind) audit as status-cell-only during the transition window, with a note.

Enforced at audit time by `backlog-state-update-audit` per RUBRIC.md (backlog_state_update_rubric@2.0).
