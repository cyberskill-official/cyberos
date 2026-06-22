#!/usr/bin/env bash
# Start the CyberOS MCP demo (gateway + reference module) from the repo root, in one
# terminal. Waits for the gateway to be healthy before starting the module so registration
# never races. Ctrl-C stops both.
#
#   bash scripts/mcp_demo.sh
#
# Then, in another terminal:  bash scripts/mcp_call.sh cyberos.demo.echo '{"message":"hi"}'
#
# Thin shim over services/mcp-gateway/examples/run-demo.sh (see examples/README.md). Override
# ports with GATEWAY_ADDR / MODULE_ADDR.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
exec bash "$ROOT/services/mcp-gateway/examples/run-demo.sh" "$@"
