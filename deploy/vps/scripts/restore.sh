#!/bin/sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
DEPLOY_DIR="$ROOT_DIR/deploy/vps"
ENV_FILE="${1:?usage: restore.sh deploy/vps/.env /path/to/backup-dir}"
BACKUP_DIR="${2:?usage: restore.sh deploy/vps/.env /path/to/backup-dir}"

if [ ! -d "$BACKUP_DIR" ]; then
  echo "missing backup dir: $BACKUP_DIR" >&2
  exit 1
fi

set -a
. "$ENV_FILE"
set +a

compose() {
  docker compose --env-file "$ENV_FILE" -f "$DEPLOY_DIR/compose.prod.yml" "$@"
}

for db in "${POSTGRES_DB:-cyberos}" "${AUTH_DB:-cyberos_auth}" "${MEMORY_DB:-cyberos_memory}" "${PROJ_DB:-cyberos_proj}"; do
  dump="$BACKUP_DIR/$db.dump"
  if [ -f "$dump" ]; then
    echo "restoring $db"
    compose exec -T postgres dropdb -U "${POSTGRES_USER:-cyberos}" --if-exists "$db"
    compose exec -T postgres createdb -U "${POSTGRES_USER:-cyberos}" "$db"
    compose exec -T postgres pg_restore -U "${POSTGRES_USER:-cyberos}" -d "$db" --clean --if-exists < "$dump"
  fi
done

if [ -n "${CYBEROS_MEMORY_ROOT:-}" ] && [ -f "$BACKUP_DIR/memory-root.tgz" ]; then
  mkdir -p "$CYBEROS_MEMORY_ROOT"
  tar -xzf "$BACKUP_DIR/memory-root.tgz" -C "$CYBEROS_MEMORY_ROOT"
fi

echo "restore complete"
