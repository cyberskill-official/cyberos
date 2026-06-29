#!/usr/bin/env bash
# Optional: bring up the CUO status server that backs the dashboard's CUO Workflows & GENIE tile
# (FR-APP-006). Read-only; no LLM call. Dev only. Stop with dev-down.sh.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
PIDS="$HERE/.pids"
LISTEN="${CUO_STATUS_LISTEN:-127.0.0.1:7740}"

[ -f "$PIDS" ] && { sed -i '' '/^cuo /d' "$PIDS" 2>/dev/null || true; } || : > "$PIDS"

echo "==> starting CUO status server on ${LISTEN}"
( cd "$ROOT/modules/cuo"
  CYBEROS_ROOT="$ROOT" CUO_DEV_CORS=1 nohup python3 -m cuo.status_server --listen "$LISTEN" > /tmp/cyberos-cuo.log 2>&1 &
  echo "cuo $!" >> "$PIDS" )
for _ in $(seq 1 15); do curl -sf "http://${LISTEN}/healthz" >/dev/null 2>&1 && break; sleep 1; done
curl -sf "http://${LISTEN}/healthz" >/dev/null 2>&1 || { echo "CUO status did not become healthy (see /tmp/cyberos-cuo.log)" >&2; exit 1; }

echo "CUO status up: http://${LISTEN}  (tile: CUO Workflows & GENIE)"
echo "Stop with scripts/dev/dev-down.sh"
