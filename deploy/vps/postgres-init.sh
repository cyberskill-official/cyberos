#!/bin/sh
set -eu

: "${AUTH_DB:=cyberos_auth}"
: "${MEMORY_DB:=cyberos_memory}"
: "${PROJ_DB:=cyberos_proj}"

psql_root() {
  psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" "$@"
}

db_exists() {
  psql_root -tAc "SELECT 1 FROM pg_database WHERE datname = '$1'" | grep -q 1
}

create_db() {
  name="$1"
  if db_exists "$name"; then
    echo "database $name already exists"
  else
    createdb --username "$POSTGRES_USER" "$name"
  fi
  psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$name" \
    -c "GRANT ALL PRIVILEGES ON DATABASE \"$name\" TO \"$POSTGRES_USER\";"
}

enable_common_extensions() {
  db="$1"
  psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$db" <<'SQL'
CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
SQL
}

enable_memory_extensions() {
  db="$1"
  psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$db" <<'SQL'
CREATE EXTENSION IF NOT EXISTS vector;
CREATE EXTENSION IF NOT EXISTS age;
LOAD 'age';
SET search_path = ag_catalog, "$user", public;
SQL
}

create_db "$AUTH_DB"
create_db "$MEMORY_DB"
create_db "$PROJ_DB"

enable_common_extensions "$POSTGRES_DB"
enable_memory_extensions "$POSTGRES_DB"

enable_common_extensions "$AUTH_DB"

enable_common_extensions "$MEMORY_DB"
enable_memory_extensions "$MEMORY_DB"

enable_common_extensions "$PROJ_DB"
