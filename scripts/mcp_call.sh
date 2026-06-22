#!/usr/bin/env bash
# Trigger an MCP tool through the gateway, runnable from the repo root.
#
#   bash scripts/mcp_call.sh cyberos.demo.now
#   bash scripts/mcp_call.sh cyberos.demo.echo '{"message":"hello"}'
#
# Thin shim over services/mcp-gateway/examples/call.sh so you do not need to cd into the
# examples dir. Point at a different gateway with MCP_GATEWAY (default http://127.0.0.1:8090).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
exec bash "$ROOT/services/mcp-gateway/examples/call.sh" "$@"
