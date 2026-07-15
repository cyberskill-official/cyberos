---
title: skill - verifiable agent capabilities · CyberOS
migrated: TASK-DOCS-002
---

skill packages what an agent can do into versioned, auditable capabilities. A skill is a folder with a `SKILL.md` contract (trigger, inputs, outputs, verification) plus optional scripts and assets; agents load skills by name and every consequential run leaves evidence.

## What lives here

- The skill library: author/audit pairs for the task workflow (context maps, edge-case matrices, implementation plans, code review, coverage gate, observability injection, backlog state updates) - the working parts of `ship-tasks`.
- Contracts: `modules/skill/contracts/` holds normative references, including the task STATUS-REFERENCE (the 10-state lifecycle and the two required human-acceptance gates).
- The Rust host (`modules/skill/crates/host`): loads, validates, and executes skill definitions.
- Golden sets: `modules/skill/.awh/` seals a baseline of the skill suite; the awh gate reruns it out-of-band so a regression can never self-certify.

## How skills reach other repos

`npx cyberos install` vendors the author/audit skills into every initialised project under `.cyberos/cuo/skills/`, and the Claude plugin (payload marketplace) exposes `/init`, `/update`, `/changelog`, `/help`, and the `ship-tasks` skill. The same skill bodies drive the workflow everywhere - there is no repo-specific fork.

## Changelog

History lives in the [changelog](./changelog.html); this page describes only the current state.
