# cyberos-mcp - the MCP channel

A zero-dependency Node stdio MCP server that exposes the CyberOS `ship-tasks`
workflow as tools, so any MCP-capable agent triggers it with no files. Requires `node >= 18`.

Tools:

- `task_install {repo?}` - vendor the CyberOS machine into a repo (needs the payload reachable; set `CYBEROS_PAYLOAD` if the server was vendored away from `install.sh`).
- `task_gates {repo?}` - run the machine gates (the repo's own build/lint/test + coverage, plus caf/awh if present).
- `task_status {repo?}` - summarize the task backlog (counts by status, next eligible task) and installed version.
- `ship_task {repo?, task_id?}` - return the canonical, HITL-gated trigger for the next (or a named) task. It never drives or accepts a task itself - the human still holds the two acceptance gates.

`repo` defaults to the current working directory, walked up to the repo root. After `install.sh`
runs, the server is vendored at `.cyberos/mcp/cyberos-mcp.mjs`; `task_gates` / `task_status` /
`ship_task` need only that repo's `.cyberos/`.

## Register it (pick your agent)

Claude Code, Cursor, Windsurf and other `.mcp.json` readers - `install.sh` already writes this
(and `.cursor/mcp.json` for Cursor) when absent. Manual form:

```json
{
  "mcpServers": {
    "cyberos": { "command": "node", "args": [".cyberos/mcp/cyberos-mcp.mjs"] }
  }
}
```

Claude Code (CLI):

```bash
claude mcp add cyberos -- node .cyberos/mcp/cyberos-mcp.mjs
```

Codex CLI - add to `~/.codex/config.toml` (or the project `.codex/config.toml`):

```toml
[mcp_servers.cyberos]
command = "node"
args = [".cyberos/mcp/cyberos-mcp.mjs"]
```

Antigravity / zcode / Command Code / any MCP client - point a stdio server at
`node .cyberos/mcp/cyberos-mcp.mjs` (use the client's "add MCP server" UI or its
`mcp.json`/config, e.g. Command Code `/mcp add`).

Payload-hosted (before a repo is inited) - run from the pack so `task_install` can bootstrap new
repos, or set `CYBEROS_PAYLOAD`:

```bash
node dist/cyberos/mcp/cyberos-mcp.mjs           # task_install resolves ../install.sh
CYBEROS_PAYLOAD=/path/to/dist/cyberos node .cyberos/mcp/cyberos-mcp.mjs
```

## Smoke test (no client needed)

```bash
printf '%s\n' \
 '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' \
 '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' \
 '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"task_status","arguments":{}}}' \
 | node cyberos-mcp.mjs
```

`CYBEROS_MCP_TIMEOUT_MS` caps how long `task_gates` / `task_install` may run (default 30 min).
