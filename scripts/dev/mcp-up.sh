#!/usr/bin/env bash
# Optional: bring up the MCP gateway + reference module that back the dashboard's MCP Registry tile
# (TASK-APP-004). Separate from dev-up.sh because it is heavier and optional. Stop with dev-down.sh.
# Dev only. The gateway runs on 7730 (not 8090) so it does not collide with the console.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
SERVICES="$ROOT/services"
PIDS="$HERE/.pids"
GW_ADDR="${MCP_GATEWAY_ADDR:-127.0.0.1:7730}"
MOD_ADDR="${MCP_MODULE_ADDR:-127.0.0.1:7731}"

# drop any previous mcp entries so a re-run does not double up
[ -f "$PIDS" ] && { sed -i '' '/^mcpgw /d;/^mcpmod /d' "$PIDS" 2>/dev/null || true; } || : > "$PIDS"

echo "==> building mcp gateway"
( cd "$SERVICES" && cargo build -q -p cyberos-mcp-gateway --bin cyberos-mcp )

echo "==> starting gateway on ${GW_ADDR} (dev CORS + dev registration)"
( cd "$SERVICES"
  MCP_DEV_CORS=1 MCP_DEV_REGISTRATION=1 nohup target/debug/cyberos-mcp --listen "$GW_ADDR" > /tmp/cyberos-mcp-gw.log 2>&1 &
  echo "mcpgw $!" >> "$PIDS" )
for _ in $(seq 1 40); do curl -sf "http://${GW_ADDR}/mcp/healthz" >/dev/null 2>&1 && break; sleep 1; done
curl -sf "http://${GW_ADDR}/mcp/healthz" >/dev/null 2>&1 || { echo "gateway did not become healthy (see /tmp/cyberos-mcp-gw.log)" >&2; exit 1; }

echo "==> starting reference module on ${MOD_ADDR} (self-registers its tools)"
( cd "$SERVICES"
  nohup python3 mcp-gateway/examples/reference_module.py --gateway "http://${GW_ADDR}" --listen "$MOD_ADDR" > /tmp/cyberos-mcp-mod.log 2>&1 &
  echo "mcpmod $!" >> "$PIDS" )
for _ in $(seq 1 20); do curl -sf "http://${MOD_ADDR}/healthz" >/dev/null 2>&1 && break; sleep 1; done

echo
echo "MCP gateway up: http://${GW_ADDR}"
echo "Open the MCP Registry tile (app.html) or mcp.html. Tools now listed:"
curl -fsS -X POST "http://${GW_ADDR}/mcp" -H 'content-type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' \
  | python3 -c 'import sys,json;[print("  -",t["name"]) for t in json.load(sys.stdin)["result"]["tools"]]' 2>/dev/null || echo "  (could not list; see /tmp/cyberos-mcp-gw.log)"
echo "Stop with scripts/dev/dev-down.sh"
