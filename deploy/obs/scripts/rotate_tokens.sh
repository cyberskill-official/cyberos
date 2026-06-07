#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TOKENS_FILE="${TOKENS_FILE:-$ROOT_DIR/auth/tokens.live}"
COLLECTOR_TOKEN_FILE="${COLLECTOR_TOKEN_FILE:-$ROOT_DIR/auth/collector.token.live}"
SERVICES=(ai-gateway auth-service chat-service memory-writer mcp-router)

tmp_tokens="$(mktemp)"
cleanup() {
  rm -f "$tmp_tokens"
}
trap cleanup EXIT

for service in "${SERVICES[@]}"; do
  token="$(openssl rand -hex 32)"
  printf "%s %s\n" "$service" "$token" >> "$tmp_tokens"
done

install -m 0600 "$tmp_tokens" "$TOKENS_FILE"

if [[ ! -f "$COLLECTOR_TOKEN_FILE" || "${ROTATE_COLLECTOR_TOKEN:-0}" == "1" ]]; then
  tmp_collector="$(mktemp)"
  openssl rand -hex 32 > "$tmp_collector"
  install -m 0600 "$tmp_collector" "$COLLECTOR_TOKEN_FILE"
  rm -f "$tmp_collector"
fi

if docker compose -f "$ROOT_DIR/docker-compose.yml" ps ingress >/dev/null 2>&1; then
  docker compose -f "$ROOT_DIR/docker-compose.yml" restart ingress >/dev/null 2>&1 || true
fi

if [[ "${ROTATE_COLLECTOR_TOKEN:-0}" == "1" ]] \
  && docker compose -f "$ROOT_DIR/docker-compose.yml" ps collector >/dev/null 2>&1; then
  docker compose -f "$ROOT_DIR/docker-compose.yml" restart collector >/dev/null 2>&1 || true
fi

echo "Tokens rotated: $TOKENS_FILE"
