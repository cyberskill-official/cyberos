---
title: Getting started · CyberOS
migrated: TASK-DOCS-002
---

# Getting started with CyberOS

CyberOS is CyberSkill's internal working platform: a set of modules (workflow engine, memory, skills, services) that agents and people operate together. Everything ships from this repo and carries one platform version (the root `VERSION` file); module versions are internal.

## Run CyberOS in your own project

CyberOS vendors itself into any repository under a single gitignored `.cyberos/` folder, organised by module.

1. Build the payload once, from a CyberOS checkout:

   ```
   bash tools/cyberos-init/build.sh        # writes dist/cyberos/
   ```

2. Initialise your repo:

   ```
   bash /path/to/dist/cyberos/init.sh /path/to/your/repo
   ```

   This vendors `.cyberos/cuo` (the task workflow engine), `.cyberos/memory` (the BRAIN protocol) and `.cyberos/plugin`, detects your build/lint/test gates into `.cyberos/gates.env`, scaffolds `docs/tasks/BACKLOG.md`, creates the local BRAIN store at `.cyberos/memory/store/`, and stamps `.cyberos/VERSION`.

3. Stay current:

   ```
   bash /path/to/dist/cyberos/init.sh --check /path/to/your/repo   # notify
   bash /path/to/dist/cyberos/init.sh /path/to/your/repo           # apply update
   ```

Desktop alternative: the CyberOS desktop app has a "CyberOS Ops" tab that runs the same build / check / init operations from the UI.

The full step-by-step runbook (first task, gates, human acceptance, multi-repo rollout) is `tools/cyberos-init/docs/index.md`.

## Ship work

All work is a task under `docs/tasks/` - net-new (`class: product`) and hardening (`class: improvement`) alike, indexed by ONE `BACKLOG.md`. The `ship-tasks` workflow drives each task through the lifecycle with two mandatory human-acceptance gates (review acceptance and final acceptance); an agent never sets `done` itself.

## Develop CyberOS itself

- Rust services: `cd services && cargo test`.
- The full DB-backed verification (what CI runs): `bash scripts/local_verify.sh` - boots the dev Postgres + Redis, applies every crate's migrations, runs each module suite. Wipe first (`docker compose -f services/dev/docker-compose.yml down -v`) to see exactly what CI sees.
- Docs: edit markdown under `docs/`, `modules/<m>/docs/`, or `services/<s>/docs/`, then `bash tools/docs-site/build.sh`. The website renders into gitignored `dist/website` - generated output is never committed or edited.

## Changelog

History lives in the [changelog](./changelog.html); this page describes only the current state.
