#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

for endpoint in \
  "ingress:http://localhost:4318/ready" \
  "collector:http://localhost:13133" \
  "loki:http://localhost:3100/ready" \
  "prometheus:http://localhost:9090/-/ready" \
  "alertmanager:http://localhost:9093/-/ready" \
  "obs-router:http://localhost:7777/ready" \
  "tempo:http://localhost:3200/ready" \
  "grafana:http://localhost:3000/api/health"; do
  name="${endpoint%%:*}"
  url="${endpoint#*:}"
  status="$(curl -fsS -o /dev/null -w "%{http_code}" "$url" || true)"
  if [[ "$status" != "200" ]]; then
    echo "FAIL: $name health endpoint returned $status"
    exit 1
  fi
done

echo "healthcheck passed"
