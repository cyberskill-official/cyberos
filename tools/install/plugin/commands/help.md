---
description: What CyberOS does — commands, task lifecycle, human gates, where things live.
---
Orient the user. Present concisely:

1. CyberOS turns work into tasks through implement → review → test → done, with the human holding two acceptance gates.

2. Commands (slash + shell). The npm/npx channel is the `cs` CLI (`npx cs <command>`):
- `/install` — install or re-vendor (once / when updating)
- `/uninstall` — remove the machine
- `/version` — check for a newer CyberOS; on yes → install
- `/status` — open `docs/status/index.html` (tabless status-hub@3 canvas) in the browser
- `/help` — this overview
- `/ship-tasks` — drive the next task (HITL); "harden a task" = `class: improvement`
- `/create-tasks` — draft tasks into the backlog
- `/inspect` — read-only full-repo inspection → `inspection-report@1` (never remediates)
- `/harden` — remediate a lint-clean inspection report → `hardening-record@1` (not ship-tasks)
- CLI-only: `npx cs gates`, `npx cs mcp`, `npx cs memory` (local cyberos-memory), `npx cs cuo <name>` (slash redirect stub; includes inspect/harden)

3. Soft update-check runs automatically on any `.cyberos` use. Day-to-day: install once, then forget.

4. Layout after install: `.cyberos/cuo/`, `.cyberos/AGENT-ENTRY.md`, `.cyberos/memory/`, `docs/tasks/`, `docs/status/`.

5. Docs: https://os.cyberskill.world/docs

If no `.cyberos/`, suggest `/install`.
