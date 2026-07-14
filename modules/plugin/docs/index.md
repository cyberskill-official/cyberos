---
title: PLUGIN - the CyberOS plugin and agent-independent distribution
source: website/docs/modules/plugin/index.html
migrated: TASK-DOCS-002
---

plugin is the CyberOS plugin for Claude Code and Cowork - name `cyberos`, version 1.0.0, authored by CyberSkill. It packages the task workflow as host commands and a bundled skill, so a user can install CyberOS into a repo and drive its backlog without leaving the agent. `tools/cyberos-init/build.sh` assembles it into `dist/cyberos/`, a plugin marketplace whose root carries `.claude-plugin/marketplace.json` and catalogs the plugin at `plugin/` (its own manifest at `plugin/.claude-plugin/plugin.json`). The same build emits `dist/cyberos/cyberos.plugin`, a one-file bundle for hosts whose Add picker wants a file rather than a folder.

## Install

- Claude Code: `/plugin marketplace add <path to dist/cyberos>`, then `/plugin install cyberos@cyberos`.
- Claude desktop and Cowork: add `dist/cyberos/cyberos.plugin` (the one-file bundle); the Add picker greys the Open button on a folder, so the file is the route here.

Both give the same commands and the `ship-tasks` skill.

## The commands and the skill

- `/init [repo]` - install CyberOS into the current repo, or update it: autodetect the build / lint / test commands, vendor the machine under a gitignored `.cyberos/`, scaffold the `docs/tasks/` backlog, and write the agent entry files. It prefers a full CyberOS payload when one is reachable and falls back to self-hosting from the plugin's own bundle. Idempotent - re-running it applies an update and never touches an existing `BACKLOG.md`, FRs, `AGENTS.md`, `gates.env`, or BRAIN.
- `/update [repo]` - compare the repo's installed version (`.cyberos/VERSION`) against an available payload and apply the update on request. Read-only until you confirm.
- `/changelog [repo]` - report the installed version and what changed recently.
- `/help` - what the plugin does: the commands, the FR lifecycle, the two human gates, and where things live in an initialised repo.
- `ship-tasks` skill - drive the next eligible FR from `docs/tasks/BACKLOG.md` through implement -> review -> test -> done. Type it as `/ship-tasks`, or just ask to ship an FR and the skill is used. It bundles its own copy of the workflow doctrine (`ship-tasks.md`, `EXECUTION-DISCIPLINE.md`, `STATUS-REFERENCE.md`) so it works standalone.

## Human-in-the-loop is required

The workflow halts at two human-acceptance gates: review acceptance (`reviewing -> ready_to_test`) and final acceptance (`testing -> done`). The agent runs everything it can verify between them, presents its review packet and test evidence, and stops for a recorded human verdict. It never sets `done`, and never pushes, merges, or deploys.

## Doc-driven and agent-independent

The Claude plugin is convenience, not a dependency. The core is doc-driven: `/init` writes the canonical `AGENTS.md` spine, `.cyberos/AGENT-ENTRY.md`, and per-agent pointer files (all create-if-absent), and installs the `ship-tasks` skill natively into every skill-aware agent's folder (`.claude/skills`, `.grok/skills`, `.commandcode/skills`, `.codex/skills`, `.opencode/skill`). Point any file-and-shell agent - Codex, Cursor, Gemini, Grok, and the rest - at `AGENTS.md` or `.cyberos/AGENT-ENTRY.md` and it drives the same workflow, the same gates, the same required human verdicts. The per-agent instruction files, native skill directories, and MCP registration are catalogued in the agent-support matrix in `tools/cyberos-init/README.md`.

## One channel of many

`dist/cyberos/` is delivered through several channels, of which the Claude plugin is one. The others include copying the folder, a git submodule or subtree, a `curl | sh` bootstrap, a GitHub Action that runs the machine gates in CI, a Docker image, a Makefile or just target, a Node stdio MCP server (tools `fr_init`, `fr_gates`, `fr_status`, `ship_fr`) for any MCP agent, an npx CLI (`cyberos-init`, `cyberos-gates`, `cyberos-mcp`), and a template-repo scaffolder (`create.sh`). The full catalog, with the trade-offs of each, is in `tools/cyberos-init/README.md`; the install-and-operate walkthrough is in `tools/cyberos-init/docs/index.md`.

## Changelog

History lives in the [changelog](./changelog.html); this page describes only the current state.
