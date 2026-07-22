---
description: What CyberOS does — commands, task lifecycle, human gates, where things live.
---
Orient the user. Present concisely:

1. CyberOS turns work into tasks through implement → review → test → done, with the human holding two acceptance gates.

2. Commands (slash + shell). The npm/npx channel is the `cs` CLI (`npx cs <command>`):
- `/install` — install or re-vendor (once / when updating)
- `/uninstall` — remove the machine
- `/version` — check for a newer CyberOS; on yes → install
- `/status` — open `docs/status/index.html` in the browser
- `/help` — this overview
- `/ship-tasks` — drive the next task (HITL)
- `/create-tasks` — draft tasks into the backlog

3. Soft update-check runs automatically on any `.cyberos` use. Day-to-day: install once, then forget.

4. Layout after install: `.cyberos/cuo/`, `.cyberos/AGENT-ENTRY.md`, `.cyberos/memory/`, `docs/tasks/`, `docs/status/`.

5. Docs: https://os.cyberskill.world/docs

If no `.cyberos/`, suggest `/install`.
