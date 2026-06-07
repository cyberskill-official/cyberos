#!/bin/sh
set -eu

: "${POSTGRES_HOST:=postgres}"
: "${POSTGRES_PORT:=5432}"
: "${POSTGRES_USER:=cyberos}"
: "${AUTH_DB:=cyberos_auth}"
: "${MEMORY_DB:=cyberos_memory}"
: "${PROJ_DB:=cyberos_proj}"
: "${MIGRATION_ROOT:=/workspace/services}"

export PGPASSWORD="${POSTGRES_PASSWORD:?POSTGRES_PASSWORD is required}"

psql_db() {
  db="$1"
  shift
  psql -v ON_ERROR_STOP=1 \
    -h "$POSTGRES_HOST" \
    -p "$POSTGRES_PORT" \
    -U "$POSTGRES_USER" \
    -d "$db" \
    "$@"
}

ensure_tracking_table() {
  db="$1"
  psql_db "$db" <<'SQL'
CREATE TABLE IF NOT EXISTS cyberos_schema_migrations (
    service     TEXT NOT NULL,
    version     TEXT NOT NULL,
    checksum    TEXT NOT NULL,
    applied_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (service, version)
);
SQL
}

applied_checksum() {
  db="$1"
  service="$2"
  version="$3"
  psql_db "$db" -tAc \
    "SELECT checksum FROM cyberos_schema_migrations WHERE service = '$service' AND version = '$version'"
}

record_migration() {
  db="$1"
  service="$2"
  version="$3"
  checksum="$4"
  psql_db "$db" -c \
    "INSERT INTO cyberos_schema_migrations(service, version, checksum)
     VALUES ('$service', '$version', '$checksum')"
}

apply_dir() {
  db="$1"
  service="$2"
  dir="$3"

  if [ ! -d "$dir" ]; then
    echo "missing migration directory: $dir" >&2
    exit 1
  fi

  ensure_tracking_table "$db"

  find "$dir" -maxdepth 1 -type f -name '*.sql' | sort | while read -r file; do
    version="$(basename "$file" .sql)"
    checksum="$(sha256sum "$file" | awk '{print $1}')"
    existing="$(applied_checksum "$db" "$service" "$version" | tr -d '[:space:]')"

    if [ -n "$existing" ]; then
      if [ "$existing" != "$checksum" ]; then
        echo "checksum mismatch for $service/$version in $db" >&2
        echo "  applied: $existing" >&2
        echo "  current: $checksum" >&2
        exit 1
      fi
      echo "skip $service/$version on $db"
      continue
    fi

    echo "apply $service/$version on $db"
    psql_db "$db" -f "$file"
    record_migration "$db" "$service" "$version" "$checksum"
  done
}

apply_dir "$AUTH_DB" auth "$MIGRATION_ROOT/auth/migrations"
apply_dir "$MEMORY_DB" memory "$MIGRATION_ROOT/memory/migrations"
apply_dir "$PROJ_DB" proj "$MIGRATION_ROOT/proj/migrations"

echo "CyberOS migrations complete"
