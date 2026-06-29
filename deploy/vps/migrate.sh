#!/usr/bin/env bash
# Uniform deploy-step migration. Applies the migrations of the DEPLOYED services (MIGRATE_SERVICES) to the
# CyberOS database, tracked PER SERVICE so version numbers across services never collide on the shared
# Supabase database. Idempotent: a file already recorded is skipped. Run from deploy/vps by deploy.sh,
# before the stack starts.
#
# Scope: only the services in MIGRATE_SERVICES are touched (default: the deployed P0 set "auth chat eval").
# Modules that are not deployed yet (memory, obs, proj, ...) are ignored, so their schema is never applied
# to the production DB prematurely - add a service here when it actually deploys.
#
# Baseline: auth and chat were applied by hand before this step existed. The first time this runs it
# RECORDS their current migration files as applied WITHOUT executing them (so a non-idempotent migration is
# never re-run on the live schema). New files added to a baselined service later are applied normally.
# Override the sets with MIGRATE_SERVICES / MIGRATE_BASELINE_SERVICES (space-separated) if needed.
#
# Connection: MIGRATIONS_DATABASE_URL (falls back to AUTH_DATABASE_URL - same Supabase DB for P0). psql runs
# inside a throwaway postgres image so the VPS needs nothing installed but Docker.
set -euo pipefail

DB_URL="${MIGRATIONS_DATABASE_URL:-${AUTH_DATABASE_URL:-}}"
if [ -z "$DB_URL" ]; then
  echo "migrate: set MIGRATIONS_DATABASE_URL (or AUTH_DATABASE_URL) in .env.p0" >&2
  exit 1
fi

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
MIGRATE_SERVICES="${MIGRATE_SERVICES:-auth chat eval}"
BASELINE_SERVICES="${MIGRATE_BASELINE_SERVICES:-auth chat}"
PG_IMAGE="${MIGRATE_PG_IMAGE:-postgres:16}"

# psql inside docker, repo mounted read-only at /repo, ON_ERROR_STOP so a bad statement aborts the file.
dpsql() { docker run --rm -i -v "$REPO_ROOT":/repo:ro "$PG_IMAGE" psql "$DB_URL" -v ON_ERROR_STOP=1 "$@"; }

echo "==> ensuring migration tracking table"
dpsql -q >/dev/null <<'SQL'
CREATE TABLE IF NOT EXISTS _cyberos_migrations (
  service    text        NOT NULL,
  filename   text        NOT NULL,
  applied_at timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (service, filename)
);
SQL

# What is already recorded (one round trip).
APPLIED="$(dpsql -tAc "SELECT service || '/' || filename FROM _cyberos_migrations")"

pending_values=""
applied_count=0
baseline_count=0

for svc in $MIGRATE_SERVICES; do
  mdir="$REPO_ROOT/services/$svc/migrations"
  [ -d "$mdir" ] || { echo "==> $svc: no migrations dir, skipping"; continue; }

  is_baseline=false
  for b in $BASELINE_SERVICES; do
    [ "$b" = "$svc" ] && is_baseline=true
  done

  # Sorted so 0001 < 0002 < ... applies in order.
  for f in $(ls "$mdir"/*.sql 2>/dev/null | sort); do
    fn="$(basename "$f")"
    key="$svc/$fn"
    if grep -qxF "$key" <<<"$APPLIED"; then
      continue
    fi
    if $is_baseline; then
      echo "==> baseline (record-only): $key"
      baseline_count=$((baseline_count + 1))
    else
      echo "==> applying: $key"
      dpsql -1 -f "/repo/services/$svc/migrations/$fn"
      applied_count=$((applied_count + 1))
    fi
    pending_values="${pending_values}('${svc}','${fn}'),"
  done
done

if [ -n "$pending_values" ]; then
  dpsql -q >/dev/null -c \
    "INSERT INTO _cyberos_migrations (service, filename) VALUES ${pending_values%,} ON CONFLICT DO NOTHING"
fi

echo "==> migrations complete: applied ${applied_count} new file(s), baselined ${baseline_count}"
