#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

if [[ ! -f auth/tokens.live || ! -f auth/collector.token.live ]]; then
  ./scripts/rotate_tokens.sh
fi

token="$(awk '$1 == "ai-gateway" { print $2 }' auth/tokens.live)"
batch_id="restart_test_$(uuidgen | tr -d '-' | tr '[:upper:]' '[:lower:]')"

for i in $(seq 1 100); do
  trace_id="$(printf "%032x" "$i")"
  curl -fsS -X POST http://localhost:4318/v1/traces \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $token" \
    -d "{\"resourceSpans\":[{\"resource\":{\"attributes\":[{\"key\":\"service.name\",\"value\":{\"stringValue\":\"ai-gateway\"}},{\"key\":\"tenant_id\",\"value\":{\"stringValue\":\"00000000-0000-0000-0000-000000000001\"}},{\"key\":\"test_batch\",\"value\":{\"stringValue\":\"$batch_id\"}}]},\"scopeSpans\":[{\"spans\":[{\"traceId\":\"$trace_id\",\"spanId\":\"0011223344556677\",\"name\":\"fr_obs_001_restart_$i\",\"kind\":1,\"startTimeUnixNano\":\"1747526400000000000\",\"endTimeUnixNano\":\"1747526401000000000\"}]}]}]}" >/dev/null &
done

sleep 1
docker compose kill collector >/dev/null
sleep 5
docker compose up -d collector >/dev/null
sleep 15

found="$(curl -fsS "http://localhost:3200/api/search?tags=test_batch:$batch_id" | jq '.traces | length')"
if (( found < 95 )); then
  echo "FAIL: only $found/100 spans recovered"
  exit 1
fi

echo "buffer_survives_restart_test passed ($found/100)"
