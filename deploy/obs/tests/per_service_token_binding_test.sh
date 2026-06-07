#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

if [[ ! -f auth/tokens.live || ! -f auth/collector.token.live ]]; then
  ./scripts/rotate_tokens.sh
fi

ai_token="$(awk '$1 == "ai-gateway" { print $2 }' auth/tokens.live)"
auth_token="$(awk '$1 == "auth-service" { print $2 }' auth/tokens.live)"
trace_id="$(uuidgen | tr -d '-' | tr '[:upper:]' '[:lower:]')"
payload="{\"resourceSpans\":[{\"resource\":{\"attributes\":[{\"key\":\"service.name\",\"value\":{\"stringValue\":\"auth-service\"}},{\"key\":\"tenant_id\",\"value\":{\"stringValue\":\"00000000-0000-0000-0000-000000000001\"}}]},\"scopeSpans\":[{\"spans\":[{\"traceId\":\"$trace_id\",\"spanId\":\"0011223344556677\",\"name\":\"fr_obs_001_authz\",\"kind\":1,\"startTimeUnixNano\":\"1747526400000000000\",\"endTimeUnixNano\":\"1747526401000000000\"}]}]}]}"

status="$(curl -s -o /dev/null -w "%{http_code}" -X POST http://localhost:4318/v1/traces \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $ai_token" \
  -d "$payload")"
if [[ "$status" != "403" ]]; then
  echo "FAIL: ai-gateway token was accepted for auth-service telemetry (HTTP $status)"
  exit 1
fi

status="$(curl -s -o /dev/null -w "%{http_code}" -X POST http://localhost:4318/v1/traces \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $auth_token" \
  -d "$payload")"
if [[ "$status" != "200" ]]; then
  echo "FAIL: auth-service token was rejected for auth-service telemetry (HTTP $status)"
  exit 1
fi

echo "per_service_token_binding_test passed"
