#!/usr/bin/env bash
set -euo pipefail

status="$(curl -s -o /dev/null -w "%{http_code}" -X POST http://localhost:4318/v1/traces \
  -H "Content-Type: application/json" \
  -d '{"resourceSpans":[]}')"

if [[ "$status" != "401" ]]; then
  echo "FAIL: ingress accepted unauthenticated request (HTTP $status)"
  exit 1
fi

echo "auth_required_test passed"
