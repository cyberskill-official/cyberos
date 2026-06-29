#!/usr/bin/env bash
# Optional: bring up the AI gateway that backs the dashboard's Assistant and AI Ops tiles (FR-APP-001 /
# FR-APP-003). Dev only. Stop with dev-down.sh. Serves /v1/chat, /v1/status, /healthz on :8080.
# /v1/status reads the tenant policy from the config dir; /v1/chat needs a local model (LM Studio / Ollama)
# to actually answer, but AI Ops works with no model running.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
SERVICES="$ROOT/services"
PIDS="$HERE/.pids"
BIND="${AI_GATEWAY_BIND:-127.0.0.1:8080}"
CONFIG_DIR="${AI_GATEWAY_CONFIG_DIR:-$SERVICES/ai-gateway/config/tenants}"

[ -f "$PIDS" ] && { sed -i '' '/^aigw /d' "$PIDS" 2>/dev/null || true; } || : > "$PIDS"

echo "==> building ai gateway"
( cd "$SERVICES" && cargo build -q -p cyberos-ai-gateway --bin cyberos-gateway )

echo "==> starting ai gateway on ${BIND} (config: ${CONFIG_DIR})"
( cd "$SERVICES"
  AI_GATEWAY_BIND="$BIND" AI_GATEWAY_DEV_CORS=1 AI_GATEWAY_CONFIG_DIR="$CONFIG_DIR" \
    nohup target/debug/cyberos-gateway > /tmp/cyberos-aigw.log 2>&1 &
  echo "aigw $!" >> "$PIDS" )
for _ in $(seq 1 30); do curl -sf "http://${BIND}/healthz" >/dev/null 2>&1 && break; sleep 1; done
curl -sf "http://${BIND}/healthz" >/dev/null 2>&1 || { echo "gateway did not become healthy (see /tmp/cyberos-aigw.log)" >&2; exit 1; }

echo
echo "AI gateway up: http://${BIND}  (tiles: Assistant, AI Ops)"
echo "Tenants with a loaded policy:"
ls "$CONFIG_DIR"/*.yaml 2>/dev/null | sed 's#.*/##;s/\.yaml$//;s/^/  - /' | grep -v EXAMPLE || true
echo "Stop with scripts/dev/dev-down.sh"
