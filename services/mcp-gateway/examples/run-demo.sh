#!/usr/bin/env bash
# One-command CyberOS MCP demo: start the gateway, wait until it is healthy, then start
# the reference module so its self-registration never races the gateway boot. Runs both in
# the foreground of a single terminal; Ctrl-C tears both down.
#
#   bash run-demo.sh
#
# Override ports/hosts with env vars if needed:
#   GATEWAY_ADDR=127.0.0.1:8090  MODULE_ADDR=127.0.0.1:8099  bash run-demo.sh
#
# Once it prints "READY", trigger tools from the desktop app's Tools tab, or headlessly:
#   bash call.sh cyberos.demo.echo '{"message":"hello"}'

set -euo pipefail

GATEWAY_ADDR="${GATEWAY_ADDR:-127.0.0.1:8090}"
MODULE_ADDR="${MODULE_ADDR:-127.0.0.1:8099}"
OBS_MODULE_ADDR="${OBS_MODULE_ADDR:-127.0.0.1:8101}"

# Resolve paths relative to this script so it runs from anywhere.
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SERVICES_DIR="$(cd "$HERE/../.." && pwd)"   # .../cyberos/services
REPO_ROOT="$(cd "$SERVICES_DIR/.." && pwd)" # .../cyberos
LOG_DIR="$(mktemp -d "${TMPDIR:-/tmp}/cyberos-mcp-demo.XXXXXX")"
GATEWAY_LOG="$LOG_DIR/gateway.log"
MODULE_LOG="$LOG_DIR/module.log"
OBS_MODULE_LOG="$LOG_DIR/obs-module.log"

GATEWAY_PID=""
MODULE_PID=""
OBS_MODULE_PID=""

cleanup() {
  echo
  echo "[run-demo] shutting down..."
  [ -n "$OBS_MODULE_PID" ] && kill "$OBS_MODULE_PID" 2>/dev/null || true
  [ -n "$MODULE_PID" ] && kill "$MODULE_PID" 2>/dev/null || true
  [ -n "$GATEWAY_PID" ] && kill "$GATEWAY_PID" 2>/dev/null || true
  wait 2>/dev/null || true
  echo "[run-demo] logs kept in $LOG_DIR"
}
trap cleanup INT TERM EXIT

wait_health() {
  # wait_health <url> <label> <max_seconds>
  local url="$1" label="$2" max="$3" i=0
  printf "[run-demo] waiting for %s " "$label"
  while [ "$i" -lt "$max" ]; do
    if curl -fsS "$url" >/dev/null 2>&1; then
      echo " up."
      return 0
    fi
    printf "."
    sleep 1
    i=$((i + 1))
  done
  echo " TIMEOUT after ${max}s."
  return 1
}

echo "[run-demo] gateway -> http://$GATEWAY_ADDR   module -> http://$MODULE_ADDR/mcp"
echo "[run-demo] logs: $LOG_DIR"

# 1) Gateway, with the dev registration route enabled. First run may compile (slower).
# `exec` so GATEWAY_PID is cargo itself (clean kill on teardown).
( cd "$SERVICES_DIR" && exec env MCP_DEV_REGISTRATION=1 \
    cargo run -p cyberos-mcp-gateway --bin cyberos-mcp -- --listen "$GATEWAY_ADDR" ) \
    >"$GATEWAY_LOG" 2>&1 &
GATEWAY_PID=$!

if ! wait_health "http://$GATEWAY_ADDR/mcp/healthz" "gateway (building/booting)" 240; then
  echo "[run-demo] gateway did not become healthy; last log lines:"
  tail -n 20 "$GATEWAY_LOG" || true
  exit 1
fi

# 2) Reference module: serves /mcp and self-registers with the now-healthy gateway.
python3 "$HERE/reference_module.py" \
    --gateway "http://$GATEWAY_ADDR" \
    --listen "$MODULE_ADDR" \
    >"$MODULE_LOG" 2>&1 &
MODULE_PID=$!

if ! wait_health "http://$MODULE_ADDR/healthz" "reference module" 20; then
  echo "[run-demo] module did not become healthy; last log lines:"
  tail -n 20 "$MODULE_LOG" || true
  exit 1
fi

# 3) Obs triage module: serves /mcp and self-registers cyberos.obs.execute_triage. Run from modules/cuo so
# the `cuo` package imports; CYBEROS_ROOT lets it resolve the skill root. With no LLM invoker on the
# host it still serves (safe-degrade verdicts), so the demo never needs an API key.
( cd "$REPO_ROOT/modules/cuo" && exec env CYBEROS_ROOT="$REPO_ROOT" \
    python3 -m cuo.triage_mcp_module \
      --gateway "http://$GATEWAY_ADDR" \
      --listen "$OBS_MODULE_ADDR" ) \
    >"$OBS_MODULE_LOG" 2>&1 &
OBS_MODULE_PID=$!

if ! wait_health "http://$OBS_MODULE_ADDR/healthz" "obs triage module" 20; then
  echo "[run-demo] obs triage module did not become healthy; last log lines:"
  tail -n 20 "$OBS_MODULE_LOG" || true
  exit 1
fi

# Show how registration went and what is now listed.
sleep 1
echo
echo "[run-demo] module log:"
sed 's/^/    /' "$MODULE_LOG" | tail -n 4
echo
echo "[run-demo] obs triage module log:"
sed 's/^/    /' "$OBS_MODULE_LOG" | tail -n 4
echo
echo "[run-demo] tools the gateway now lists:"
curl -fsS -X POST "http://$GATEWAY_ADDR/mcp" \
  -H 'content-type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' \
  | python3 -c 'import sys,json; [print("    -",t["name"]) for t in json.load(sys.stdin)["result"]["tools"]]' \
  2>/dev/null || echo "    (could not list; see $GATEWAY_LOG)"

cat <<EOF

[run-demo] READY. Trigger a tool:
    - Desktop app: open the Tools tab, press Refresh, pick a tool, Run.
    - Headless, from the repo root:
          bash scripts/mcp_call.sh cyberos.demo.now
          bash scripts/mcp_call.sh cyberos.demo.echo '{"message":"hello"}'
          bash scripts/mcp_call.sh cyberos.obs.execute_triage '{"alert":{"name":"HighErrorRate","severity":"sev2","summary":"5xx above 2%"}}'

Press Ctrl-C to stop the gateway and the modules.
EOF

# Stay in the foreground until interrupted; surface a crash of any child. (Portable to
# macOS bash 3.2, which lacks `wait -n`.)
while kill -0 "$GATEWAY_PID" 2>/dev/null \
   && kill -0 "$MODULE_PID" 2>/dev/null \
   && kill -0 "$OBS_MODULE_PID" 2>/dev/null; do
  sleep 1
done
echo "[run-demo] a process exited; shutting down (see logs in $LOG_DIR)."
