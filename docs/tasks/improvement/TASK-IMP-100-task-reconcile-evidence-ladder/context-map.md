# TASK-IMP-100 repo context map

## Cone
- `tools/install/docs-tools/task-reconcile.mjs` (new), `tools/install/tests/test_task_reconcile.sh` (new)
- `modules/skill/task-reconcile/SKILL.md` (new), `tools/install/build.sh` (vendor copy + VENDORED_SKILLS entry)

## Patterns the change must follow
- **Node stdlib only** in docs-tools (task-lint, ship-manifest, backlog-mutate, coverage-scope) - no deps reach consumers.
- **Skill frontmatter is a schema**: `name` must equal the directory name and `description` <= 1024 chars; build.sh fails closed on both (it caught my first draft's invented frontmatter).
- **A chain skill must be vendored**: `check-chain-coverage.sh` requires every `skill:` named in ship-tasks.md to exist in BOTH payload trees (`cuo/skills/` and `plugin/skills/`); build.sh's `VENDORED_SKILLS` list is the source.
- **Composition over reimplementation**: R1 shells out to task-lint, R3 to ship-manifest verify - the rungs reuse shipped tools rather than re-deriving their logic.
- **Read-only by contract**: the 086 lesson - a measuring tool that writes is a tool whose measurements you cannot trust.

## Blast radius
- Files: 3 new + 1 modified. Modules: 2 (docs-tools, skill). Cross-module edge: build.sh (shared with nothing else this batch).
- Consumer impact: one more vendored tool + skill; payload 9.79 MB, plugin zip 1.11 MB (both inside budget).

## Module placement
Correct - cross-cutting governance hardening.
