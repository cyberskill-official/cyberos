# {{PROJECT}}

This project runs **CyberOS** - the governed, HITL-gated `ship-tasks` workflow, wired for every popular coding agent (Claude Code, Codex, Cursor, Gemini, Antigravity, Grok CLI, zcode, Command Code, Copilot, Windsurf, ...).

## Start here

1. Your agent reads `AGENTS.md` (the cross-agent spine) or `.cyberos/AGENT-ENTRY.md`.
2. Write a task:
   ```bash
   cp .cyberos/cuo/templates/task-TEMPLATE.md docs/tasks/TASK-001-<slug>.md
   # fill section 1, set status: ready_to_implement, add the row to docs/tasks/BACKLOG.md
   ```
3. Trigger it: tell your agent to "follow `.cyberos/cuo/ship-tasks.md` and drive the next eligible task" - or, with an MCP client, call the `ship_task` tool.
4. Gates any time: `bash .cyberos/cuo/gates/run-gates.sh`.

HITL is required: the agent halts at review acceptance and final acceptance for your verdict, and never sets `done`, pushes, deploys, or merges on its own.

Re-run `bash <payload>/install.sh .` after pulling a newer CyberOS to refresh the machine (your `BACKLOG.md`, tasks, and BRAIN are never clobbered).
