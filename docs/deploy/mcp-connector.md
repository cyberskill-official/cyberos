# CyberOS remote MCP connector (FR-IMP-076)

The CyberOS MCP server (`mcp/cyberos-mcp.mjs` in every payload / installed `.cyberos/`) now runs in two transports:

- **stdio** (default) - Claude Code, Cursor, any local MCP agent: `node .cyberos/mcp/cyberos-mcp.mjs`
- **remote connector** - `node .cyberos/mcp/cyberos-mcp.mjs --http 8799` - a zero-dependency MCP streamable-HTTP endpoint (POST JSON-RPC, `GET /healthz` for probes) that agent UIs' "custom connector" dialogs can point at. Tools exposed: `fr_init`, `fr_gates`, `fr_status`, `ship_fr`.

## Hooking it into agent UIs

- **Claude (web/desktop):** Settings → Connectors → Add custom connector → Name: `CyberOS`, Remote MCP server URL: `https://<your-host>/mcp` (OAuth fields optional/blank for a self-hosted deployment you control).
- **Grok:** Skills and Connectors → New Connector → Custom Connector → Name + Server URL. Grok's dialog suggests an `/sse` URL; its support for streamable HTTP vs legacy SSE should be confirmed at hookup time - if it requires the legacy SSE transport specifically, that transport is the recorded follow-up (spec §9), not silently claimed.

## Production checklist (operator - Stephen)

1. Pick the public URL (e.g. `https://os.cyberskill.world/mcp`) and route it on the VPS reverse proxy to `localhost:8799` - TLS terminates at the proxy; the node process stays loopback-only.
2. Run the server under a supervisor (systemd unit / docker compose alongside the existing services).
3. Auth: the transport ships UNAUTHENTICATED - do not expose it publicly without either proxy-level auth (basic/bearer at nginx) or an allowlist. The ship_fr/fr_init tools execute repo workflows; treat the endpoint like CI credentials.
4. Add the connector in each agent UI per above, then verify `tools/list` returns the 4 workflow tools.

## Local verification (no deploy needed)

```bash
node .cyberos/mcp/cyberos-mcp.mjs --http 8799 &
curl -s localhost:8799/healthz
curl -s -X POST localhost:8799 -d '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | head -c 300
```
