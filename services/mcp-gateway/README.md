# cyberos-mcp-gateway

**P0 module · external-agent door.**
Implements [`docs/tasks/mcp/TASK-MCP-001..008`](../../docs/tasks/mcp/) — Model Context Protocol 2025-11-25 federation gateway. External MCP clients (Claude Desktop, IDE plugins, third-party agents) connect here; the gateway holds the federated tool catalog and dispatches `tools/call` to the owning module's MCP server.

## Status (2026-05-19 wave)

| FR | Title | Status |
|---|---|---|
| **TASK-MCP-001** | MCP 2025-11-25 spec compliance — `initialize` + `tools/list` + `tools/call` + capabilities | **building** (slice-1 scaffold: JSON-RPC parser · closed error-code map · `initialize` handshake with capability advertisement · `tools/list` with cursor pagination · `tools/call` dispatch returning `-32004 module_unreachable` until TASK-MCP-002 wires registration · `ToolAnnotations` · `ToolRegistry` · `axum` router with `/mcp` + `/mcp/healthz`. Tests cover JSON-RPC parsing, error codes, initialize match/mismatch, pagination, tools/call permission gate. Remaining: JWT verification + tenant-aware rate-limit + audit emission + Streamable HTTP SSE transport — landing in follow-on slices) |
| TASK-MCP-002 | Per-module server registration + heartbeat lifecycle | pending |
| TASK-MCP-003 | SEP-986 naming convention validator | pending |
| TASK-MCP-004 | OAuth 2.1 PKCE authorization-code flow | pending |
| TASK-MCP-005 | Protected Resource Metadata at `/.well-known/oauth-protected-resource` | pending |
| TASK-MCP-006 | Tool-annotation gating (destructive requires explicit confirm) | pending |
| TASK-MCP-007 | Tasks primitive (long-running work with resume-on-reconnect) | pending |
| TASK-MCP-008 | Elicitation server-initiated request/response | pending |

## Layout

```
src/
├── lib.rs                       # crate-level docs + MCP_PROTOCOL_VERSION
├── annotations.rs               # ToolAnnotations (destructiveHint / readOnlyHint / …)
├── federation.rs                # federation module root
├── federation/
│   └── registry.rs              # in-memory ToolRegistry (TASK-MCP-002 ships the live handler)
├── protocol.rs                  # protocol module root
├── protocol/
│   ├── jsonrpc.rs               # JSON-RPC 2.0 wire types + Inbound::parse (single + batch)
│   ├── errors.rs                # closed error-code map (-32700..-32603 + -32001..-32005)
│   ├── capabilities.rs          # Capabilities + ServerInfo per DEC-266
│   ├── initialize.rs            # initialize handshake (protocol-version match/mismatch)
│   ├── tools_list.rs            # tools/list with cursor pagination
│   └── tools_call.rs            # tools/call dispatch (slice-1: returns module_unreachable)
├── router.rs                    # Axum router mounting /mcp + /mcp/healthz
└── bin/
    └── cyberos_mcp.rs           # binary entrypoint
```

## Local development

```bash
# Build + run all tests:
cargo build -p cyberos-mcp-gateway
cargo test  -p cyberos-mcp-gateway

# Start the gateway (slice-1: empty registry; tools land via TASK-MCP-002):
cargo run -p cyberos-mcp-gateway --bin cyberos-mcp -- --listen 0.0.0.0:8090

# Smoke-test the initialize handshake:
curl -s -X POST http://localhost:8090/mcp \
     -H 'content-type: application/json' \
     -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25"}}' \
     | jq .

# Healthz:
curl -s http://localhost:8090/mcp/healthz | jq .
```

## §14 protocol emission

This module participates in the `AGENTS.md §14.1` protocol. The slice-1 ship is recorded in `docs/tasks/BACKLOG.md §0.5` production-status table with the building state.
