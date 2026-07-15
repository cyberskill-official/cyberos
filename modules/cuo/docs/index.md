---
title: cuo - the CyberOS workflow engine · CyberOS
migrated: TASK-DOCS-002
---

cuo is the workflow engine: it drives every unit of work - a task - through one governed lifecycle, with humans holding the two acceptance gates. There is exactly one implementation workflow; improvement and hardening work runs the same machinery as net-new features.

## The single workflow

`ship-tasks` (v2.3.x) picks the first eligible task from `docs/tasks/BACKLOG.md` and drives it end to end:

```
draft -> ready_to_implement -> implementing -> ready_to_review -> reviewing
      -> ready_to_test -> testing -> done        (off-ramps: on_hold, closed)
```

- Task frontmatter `status` is the record of truth; BACKLOG.md is the index kept in lockstep.
- One backlog for both classes: `class: improvement` rows carry an `(improvement)` tag, product rows are untagged. There is never a second backlog file.
- Failures route back to `ready_to_implement` with `routed_back_count += 1` - there are no terminal failure states.

## Human-in-the-loop is required

Two transitions are human-acceptance gates the agent must never cross by itself:

1. Review acceptance (`reviewing -> ready_to_test`): a human accepts the code-review packet.
2. Final acceptance (`testing -> done`): a human accepts the shipped task after every machine gate is green.

Between the gates the agent runs continuously and self-resolves everything it can verify. The doctrine lives in `modules/cuo/EXECUTION-DISCIPLINE.md`; the status contract in `modules/skill/contracts/task/STATUS-REFERENCE.md`.

## Gates

Machine gates derive from the touched module's `audit-profile.yaml`: build, lint, tests, coverage on touched files, the caf audit gate, and - where a module has a sealed goldenset (`modules/<m>/.awh/`) - an independent awh rerun against the baseline. Green machine gates are necessary, never sufficient: the two human verdicts still decide.

## Where things live

- Workflow: `modules/cuo/chief-technology-officer/workflows/ship-tasks.md`
- tasks: `docs/tasks/<module>/task-<MOD>-NNN-slug.md`; cross-cutting hardening under `docs/tasks/improvement/`
- In any CyberOS-initialised repo: `.cyberos/cuo/` carries the same workflow, doctrine, status contract, skills, and gate runner - trigger it with the prompt in `tools/install/docs/index.md` or the `/ship-tasks` plugin skill.

## Guides

- [Ship your first task](./guides/ship-your-first-task.html) - the day-one, step-by-step walkthrough for employees: install, write a task, trigger the agent, hold the two gates, land the change.

## Changelog

History (including the collapse of the old separate improvement track into this single workflow) lives in the [changelog](./changelog.html); this page describes only the current state.
