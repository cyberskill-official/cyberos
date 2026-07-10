# cyberos-mcp - the MCP channel

A zero-dependency Node stdio MCP server that exposes the CyberOS `ship-feature-requests`
workflow as tools, so any MCP-capable agent triggers it with no files. Requires `node >= 18`.

Tools:

- `fr_init {repo?}` - vendor the CyberOS machine into a repo (needs the payload reachable; set `CYBEROS_PAYLOAD` if the server was vendored away from `init.sh`).
- `fr_gates {repo?}` - run the machine gates (the repo's own build/lint/test + coverage, plus caf/awh if present).
- `fr_status {repo?}` - summarize the FR backlog (counts by status, next eligible FR) and installed version.
- `ship_fr {repo?, fr_id?}` - return the canonical, HITL-gated trigger for the next (or a named) FR. It never drives or accepts an FR itself - the human still holds the two acceptance gates.

`repo` defaults to the current working directory, walked up to the repo root. After `init.sh`
runs, the server is vendored at `.cyberos/mcp/cyberos-mcp.mjs`; `fr_gates` / `fr_status` /
`ship_fr` need only that repo's `.cyberos/`.

## Register it (pick your agent)

Claude Code, Cursor, Windsurf and other `.mcp.json` readers - `init.sh` already writes this
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

Payload-hosted (before a repo is inited) - run from the pack so `fr_init` can bootstrap new
repos, or set `CYBEROS_PAYLOAD`:

```bash
node dist/cyberos/mcp/cyberos-mcp.mjs           # fr_init resolves ../init.sh
CYBEROS_PAYLOAD=/path/to/dist/cyberos node .cyberos/mcp/cyberos-mcp.mjs
```

## Smoke test (no client needed)

```bash
printf '%s\n' \
 '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' \
 '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' \
 '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"fr_status","arguments":{}}}' \
 | node cyberos-mcp.mjs
```

`CYBEROS_MCP_TIMEOUT_MS` caps how long `fr_gates` / `fr_init` may run (default 30 min).
