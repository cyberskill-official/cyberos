#!/bin/sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
DEPLOY_DIR="$ROOT_DIR/deploy/vps"
ENV_FILE="${1:-$DEPLOY_DIR/.env.local}"

if ! docker version >/dev/null 2>&1; then
  echo "Docker daemon is not reachable. Start Docker Desktop, then rerun this script." >&2
  exit 1
fi

if [ ! -f "$ENV_FILE" ]; then
  cp "$DEPLOY_DIR/env.production.example" "$ENV_FILE"
  python3 - "$ENV_FILE" <<'PY'
from pathlib import Path
path = Path(__import__("sys").argv[1])
text = path.read_text()
text = text.replace("change-me-use-a-long-random-password", "cyberos-local-password")
text = text.replace("change-me-use-a-different-long-random-password", "mattermost-local-password")
text = text.replace("0000000000000000000000000000000000000000000000000000000000000000", "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
text = text.replace("CYBEROS_MEMORY_ROOT=/opt/cyberos/data/memory-root", f"CYBEROS_MEMORY_ROOT={path.parent / 'data' / 'memory-root'}")
path.write_text(text)
PY
  echo "Created local env file: $ENV_FILE"
fi

cd "$ROOT_DIR"

RESET="${CYBEROS_LOCAL_SMOKE_RESET:-auto}"
if [ "$RESET" = "1" ] || { [ "$RESET" = "auto" ] && [ "$(basename "$ENV_FILE")" = ".env.local" ]; }; then
  docker compose --env-file "$ENV_FILE" -f deploy/vps/compose.prod.yml down -v --remove-orphans >/dev/null 2>&1 || true
fi

docker compose --env-file "$ENV_FILE" -f deploy/vps/compose.prod.yml build postgres cyberos-auth cyberos-memory
docker compose --env-file "$ENV_FILE" -f deploy/vps/compose.prod.yml up -d postgres redis migrator cyberos-auth cyberos-memory caddy

"$DEPLOY_DIR/scripts/healthcheck.sh" "http://127.0.0.1:8080"

echo "Local VPS-profile Docker smoke passed"
