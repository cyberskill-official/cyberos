# mcp-gateway examples

## reference_module.py - the smallest module that federates into the gateway

`reference_module.py` is a stdlib-only Python MCP server that shows the two things any CyberOS module does to join the gateway (TASK-MCP-002):

1. serves JSON-RPC 2.0 over `POST /mcp` - the `initialize`, `tools/list`, and `tools/call` methods the gateway forwards;
2. self-registers its tool catalogue to the gateway's `POST /v1/mcp/register` at startup, so its tools appear in `tools/list` and in the desktop Tools tab, and the gateway forwards `tools/call` for them back to this server.

It exposes two read-only demo tools: `cyberos.demo.echo` (returns your arguments) and `cyberos.demo.now` (returns the current UTC time). The bodies are trivial on purpose - real modules (cuo, obs, memory) keep this exact contract and put real work in `run_tool`.

### Quickest path: from the repo root

Two repo-root wrappers so you do not have to cd anywhere or remember paths:

```
bash scripts/mcp_demo.sh        # starts gateway + module, health-gated, one terminal
```

It starts the gateway, waits until it is healthy, then starts this module so its self-registration never races the gateway boot (the "connection refused" you get if you start the module first). When it prints `READY`, trigger a tool from another terminal:

```
bash scripts/mcp_call.sh cyberos.demo.now
bash scripts/mcp_call.sh cyberos.demo.echo '{"message":"hello"}'

# or the desktop app: open the Tools tab, Refresh, pick a tool, Run.
```

`Ctrl-C` in the demo terminal stops both. (`scripts/mcp_demo.sh` and `scripts/mcp_call.sh` are thin wrappers over `run-demo.sh` and `call.sh` here; run those directly from this dir if you prefer.)

Stop the module (Ctrl-C in run-demo.sh) and trigger again to see the honest `module_unreachable` - the gateway really tries to reach the endpoint now.

### Manual, three terminals

If you would rather run the pieces yourself:

```
# 1) the gateway, with the dev registration route enabled
cd services
MCP_DEV_REGISTRATION=1 cargo run -p cyberos-mcp-gateway --bin cyberos-mcp -- --listen 127.0.0.1:8090

# 2) this reference module - it serves /mcp and self-registers with the gateway
cd services/mcp-gateway/examples
python3 reference_module.py --gateway http://127.0.0.1:8090 --listen 127.0.0.1:8099

# 3) confirm the gateway now lists the tools
curl -sS -X POST http://127.0.0.1:8090/mcp \
  -H 'content-type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list"}'
```

### Health and heartbeats (TASK-MCP-002)

Once registered, the module heartbeats the gateway every 10s. The gateway tracks each module's health and stops offering a module's tools when it goes silent:

- healthy: heartbeat within 10s.
- degraded: 10-30s since the last beat (still listed and callable).
- unhealthy: more than 30s (3 missed beats) - the module's tools drop out of `tools/list` and `tools/call` returns `skill_unavailable` (-32006) before any network attempt.
- deregistered: the module said goodbye (the reference module does this on Ctrl-C).

See it: with the demo running, `curl -sS http://127.0.0.1:8090/mcp/healthz` shows a `servers` array with each module's status. Stop the module with Ctrl-C and it deregisters immediately, so its tools vanish from the Tools tab on the next Refresh. `kill -9` it instead (no goodbye) and it flips to unhealthy about 30s later.

### Run the contract tests

```
cd services/mcp-gateway/examples
python3 -m pytest test_reference_module.py -q
```

These check the `tools/call` envelope shape the gateway expects, with no gateway running.

### Notes

- The registration route is gated behind `MCP_DEV_REGISTRATION=1` on the gateway because registration decides where the gateway forwards calls - a trust boundary. Production replaces the dev gate with authenticated registration (TASK-MCP-004) plus an endpoint allowlist.
- `--public-host` lets you register a different host:port than you bind locally (useful when the gateway reaches the module across a container boundary). It defaults to `--listen`.
- The heartbeat/health lifecycle (a module that goes silent getting marked unhealthy, with `skill_unavailable` propagated to `tools/list`) is the next TASK-MCP-002 slice; this module registers once and stays listed until the gateway restarts.
