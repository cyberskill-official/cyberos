# `planning/` — Per-project work folders

Every active project gets a folder here named `YYYY-MM-DD-<slug>/`. Inside, the chain runner writes:

- `FR-NNN-*.md` — one Feature Request per user story (new shape: slim frontmatter + body H2 task sections)
- `chain-manifest.json` — chain run state (for `cyberos chain status` / `resume`)
- `project-index.md` — auto-generated project dashboard (Batch D)
- (optional) `prd.md`, `srs.md` — operator-supplied spec inputs if used

## Current projects

```shell
ls -1 planning/
```

To see all FRs across all projects:
```shell
cyberos fr list
```

## Starting a new project

```shell
cyberos chain run \
  --pitch "Short description of the idea" \
  --profile solo \
  [--prd path/to/prd.md] \
  [--srs path/to/srs.md] \
  [--with-llm]
```

This creates `planning/<auto-slug>/` populated with the chain artefacts.

## Conventions

- One folder per project; folder name is `YYYY-MM-DD-<slug>`.
- One FR per user story. Multiple FRs in the same project folder = multiple stories.
- Task IDs are `FR-NNN-T-MM`; subtask IDs are `FR-NNN-T-MM-ST-XX`.
- The `project-index.md` has a `<!-- BEGIN human-edited -->` block preserved across chain reruns — put milestones / vendor notes / risks there.

## When a project ships

After the FRs in a project folder reach `status: shipped`, move the folder under `.cyberos-memory/memories/projects/<slug>/` (the BRAIN's project archive) to keep planning/ focused on active work.

## Related

- FR shape: [`../docs/contracts/feature-request/`](../docs/contracts/feature-request/)
- Task shape: [`../docs/contracts/task/`](../docs/contracts/task/)
- Skills layer that produces FRs: [`../docs/skills/README.md`](../docs/skills/README.md)
