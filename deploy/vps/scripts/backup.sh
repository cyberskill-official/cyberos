#!/bin/sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
DEPLOY_DIR="$ROOT_DIR/deploy/vps"
ENV_FILE="${1:-$DEPLOY_DIR/.env}"
BACKUP_DIR="${CYBEROS_BACKUP_DIR:-$ROOT_DIR/backups}"
TS="$(date -u +%Y%m%dT%H%M%SZ)"

if [ ! -f "$ENV_FILE" ]; then
  echo "missing env file: $ENV_FILE" >&2
  exit 1
fi

set -a
. "$ENV_FILE"
set +a

mkdir -p "$BACKUP_DIR/$TS"

compose() {
  docker compose --env-file "$ENV_FILE" -f "$DEPLOY_DIR/compose.prod.yml" "$@"
}

for db in "${POSTGRES_DB:-cyberos}" "${AUTH_DB:-cyberos_auth}" "${MEMORY_DB:-cyberos_memory}" "${PROJ_DB:-cyberos_proj}"; do
  echo "dumping $db"
  compose exec -T postgres pg_dump -U "${POSTGRES_USER:-cyberos}" -d "$db" -Fc > "$BACKUP_DIR/$TS/$db.dump"
done

if [ -n "${CYBEROS_MEMORY_ROOT:-}" ] && [ -d "$CYBEROS_MEMORY_ROOT" ]; then
  tar -czf "$BACKUP_DIR/$TS/memory-root.tgz" -C "$CYBEROS_MEMORY_ROOT" .
fi

echo "backup written: $BACKUP_DIR/$TS"
