#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

if [[ ! -f auth/tokens.live || ! -f auth/collector.token.live ]]; then
  ./scripts/rotate_tokens.sh
fi

export GRAFANA_ADMIN_PASSWORD="${GRAFANA_ADMIN_PASSWORD:-cyberos-local-dev}"
docker compose up -d

deadline=$((SECONDS + 60))
for svc in ingress collector loki prometheus tempo grafana; do
  while true; do
    health="$(docker compose ps --format json "$svc" | jq -r 'if type == "array" then (.[0].Health // "") else (.Health // "") end' 2>/dev/null || true)"
    [[ "$health" == "healthy" ]] && break
    if (( SECONDS > deadline )); then
      echo "FAIL: $svc did not become healthy; health=$health"
      docker compose ps
      exit 1
    fi
    sleep 2
  done
done

token="$(awk '$1 == "ai-gateway" { print $2 }' auth/tokens.live)"
trace_id="$(uuidgen | tr -d '-' | tr '[:upper:]' '[:lower:]')"

curl -fsS -X POST http://localhost:4318/v1/traces \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $token" \
  -d "{\"resourceSpans\":[{\"resource\":{\"attributes\":[{\"key\":\"service.name\",\"value\":{\"stringValue\":\"ai-gateway\"}},{\"key\":\"tenant_id\",\"value\":{\"stringValue\":\"00000000-0000-0000-0000-000000000001\"}}]},\"scopeSpans\":[{\"spans\":[{\"traceId\":\"$trace_id\",\"spanId\":\"0011223344556677\",\"name\":\"fr_obs_001_smoke\",\"kind\":1,\"startTimeUnixNano\":\"1747526400000000000\",\"endTimeUnixNano\":\"1747526401000000000\"}]}]}]}"

sleep 10
status="$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:3200/api/traces/$trace_id")"
if [[ "$status" != "200" ]]; then
  echo "FAIL: trace $trace_id not found in Tempo (HTTP $status)"
  exit 1
fi

echo "smoke_test passed"
