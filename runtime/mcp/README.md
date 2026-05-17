# `runtime/mcp/` — MCP server for the BRAIN

Read-only Model Context Protocol server that exposes the `.cyberos-memory/` BRAIN to Claude (and other MCP clients) over stdio.

## What it does

Lets Claude query the BRAIN as if it were any other MCP-connected datasource. Read-only by default — Claude cannot write back through this server (writes go through `cyberos add <TYPE>` or `runtime/lib/brain_writer.py` with full audit).

Surfaced tools (per Aspect 12.7):

| Tool | Returns |
| --- | --- |
| `brain.search <query>` | Memories matching the query (semantic + lexical hybrid) |
| `brain.read_memory <memory_id>` | Full memory record + audit trail |
| `brain.list_decisions [filter]` | All `DEC-NNN` memories with optional filter |
| `brain.list_facts` | All `FACT-NNN` memories |
| `brain.list_preferences` | All `PREF-NNN` memories |
| `brain.list_people` | All `PERSON-NNN` profiles |
| `brain.audit <subject> <date_range>` | Audit ledger entries for a memory or persona |

## Running

```shell
python3 runtime/mcp/server.py
```

Connect in Claude Desktop / Claude Code by adding to `~/.config/claude/mcp.json`:
```json
{
  "mcpServers": {
    "cyberos-brain": {
      "command": "python3",
      "args": ["/abs/path/to/cyberos/runtime/mcp/server.py"]
    }
  }
}
```

## Why read-only?

The protocol's audit-ledger invariants (AGENTS §0.6, §5.3, §8.7) require every BRAIN mutation to flow through a single writer (`brain_writer.py`) so that audit rows are correctly appended, source-tier policy enforced, and tombstone consistency preserved. An MCP write path would bypass those checks. Writes stay in the CLI surface (`cyberos add`, `cyberos edit`, etc.).

## Related

- BRAIN spec: [`../../memory/docs/AGENTS.md`](../../memory/docs/AGENTS.md)
- Canonical writer: [`../lib/brain_writer.py`](../lib/brain_writer.py)
- Aspect 12.7 in the operator manual: [`../../memory/docs/README.md`](../../memory/docs/README.md) Part 26.12.7
