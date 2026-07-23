# CyberOS remote MCP connector (TASK-IMP-076)

The CyberOS MCP server (`mcp/cyberos-mcp.mjs` in every payload / installed `.cyberos/`) now runs in two transports:

- **stdio** (default) - Claude Code, Cursor, any local MCP agent: `node .cyberos/mcp/cyberos-mcp.mjs`
- **remote connector** - `node .cyberos/mcp/cyberos-mcp.mjs --http 8799` - a zero-dependency MCP streamable-HTTP endpoint (POST JSON-RPC, `GET /healthz` for probes) that agent UIs' "custom connector" dialogs can point at. Tools exposed: `task_install`, `task_gates`, `task_status`, `ship_task`.

## Hooking it into agent UIs

- **Claude (web/desktop):** Settings → Connectors → Add custom connector → Name: `CyberOS`, Remote MCP server URL: `https://<your-host>/mcp` (OAuth fields optional/blank for a self-hosted deployment you control).
- **Grok:** Skills and Connectors → New Connector → Custom Connector → Name + Server URL. Grok's dialog suggests an `/sse` URL; its support for streamable HTTP vs legacy SSE should be confirmed at hookup time - if it requires the legacy SSE transport specifically, that transport is the recorded follow-up (spec §9), not silently claimed.

## Production checklist (operator - Stephen)

1. Pick the public URL (e.g. `https://os.cyberskill.world/mcp`) and route it on the VPS reverse proxy to `localhost:8799` - TLS terminates at the proxy; the node process stays loopback-only.
2. Run the server under a supervisor (systemd unit / docker compose alongside the existing services).
3. Auth: `--http` binds `127.0.0.1` by default (loopback-only). Set `CYBEROS_MCP_TOKEN` non-empty to require `Authorization: Bearer <token>` on every `POST /mcp` (`GET /healthz` stays open). Binding beyond loopback without a token warns at startup (`--host <addr>`). Prefer the token (or proxy-level auth) before any public exposure — the ship_task/task_install tools execute repo workflows; treat the endpoint like CI credentials.
4. Add the connector in each agent UI per above, then verify `tools/list` returns the 4 workflow tools.

## Local verification (no deploy needed)

```bash
node .cyberos/mcp/cyberos-mcp.mjs --http 8799 &
curl -s localhost:8799/healthz
curl -s -X POST localhost:8799 -d '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | head -c 300
```
