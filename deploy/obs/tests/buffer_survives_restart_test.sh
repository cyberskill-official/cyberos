#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

if [[ ! -f auth/tokens.live || ! -f auth/collector.token.live ]]; then
  ./scripts/rotate_tokens.sh
fi

export GRAFANA_ADMIN_PASSWORD="${GRAFANA_ADMIN_PASSWORD:-cyberos-local-dev}"

token="$(awk '$1 == "ai-gateway" { print $2 }' auth/tokens.live)"
batch_id="restart_test_$(uuidgen | tr -d '-' | tr '[:upper:]' '[:lower:]')"
trace_prefix="$(uuidgen | tr -d '-' | tr '[:upper:]' '[:lower:]' | cut -c1-24)"
trace_ids=()

batch_timeout_count() {
  curl -fsS http://localhost:8888/metrics \
    | awk '$1 ~ /^otelcol_processor_batch_timeout_trigger_send/ { print int($NF) }'
}

batch_send_sum() {
  curl -fsS http://localhost:8888/metrics \
    | awk '$1 ~ /^otelcol_processor_batch_batch_send_size_sum/ { print int($NF) }'
}

for i in $(seq 1 100); do
  trace_id="$(printf "%s%08x" "$trace_prefix" "$i")"
  trace_ids+=("$trace_id")
done

docker compose stop tempo >/dev/null
sleep 2
baseline_flushes="$(batch_timeout_count)"
baseline_flushes="${baseline_flushes:-0}"
baseline_sent="$(batch_send_sum)"
baseline_sent="${baseline_sent:-0}"

for i in $(seq 1 100); do
  trace_id="${trace_ids[$((i - 1))]}"
  curl -fsS -X POST http://localhost:4318/v1/traces \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $token" \
    -d "{\"resourceSpans\":[{\"resource\":{\"attributes\":[{\"key\":\"service.name\",\"value\":{\"stringValue\":\"ai-gateway\"}},{\"key\":\"tenant_id\",\"value\":{\"stringValue\":\"00000000-0000-0000-0000-000000000001\"}},{\"key\":\"test_batch\",\"value\":{\"stringValue\":\"$batch_id\"}}]},\"scopeSpans\":[{\"spans\":[{\"traceId\":\"$trace_id\",\"spanId\":\"0011223344556677\",\"name\":\"fr_obs_001_restart_$i\",\"kind\":1,\"startTimeUnixNano\":\"1747526400000000000\",\"endTimeUnixNano\":\"1747526401000000000\"}]}]}]}" >/dev/null
done

deadline=$((SECONDS + 45))
while true; do
  flushed="$(batch_timeout_count)"
  flushed="${flushed:-0}"
  sent="$(batch_send_sum)"
  sent="${sent:-0}"
  if (( flushed > baseline_flushes && sent >= baseline_sent + 100 )); then
    break
  fi
  if (( SECONDS > deadline )); then
    echo "FAIL: trace batch did not fully flush toward the exporter"
    exit 1
  fi
  sleep 1
done

docker compose kill collector >/dev/null
docker compose up -d tempo collector >/dev/null

deadline=$((SECONDS + 60))
for svc in tempo collector; do
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

deadline=$((SECONDS + 90))
found=0
while true; do
  found=0
  for trace_id in "${trace_ids[@]}"; do
    status="$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:3200/api/traces/$trace_id" || true)"
    if [[ "$status" == "200" ]]; then
      found=$((found + 1))
    fi
  done
  if (( found >= 95 )); then
    break
  fi
  if (( SECONDS > deadline )); then
    echo "FAIL: only $found/100 spans recovered"
    exit 1
  fi
  sleep 3
done

echo "buffer_survives_restart_test passed ($found/100)"
