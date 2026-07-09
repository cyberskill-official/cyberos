#!/usr/bin/env bash
# Run the Postgres-gated (#[ignore]) integration tests against a REAL pgvector Postgres.
#
# The DB integration tests across services/ (memory brain + interaction, auth admin + capture, eval, ...)
# are #[ignore] on purpose: a plain `cargo test` stays hermetic and fast, so they show as "ignored,
# requires Postgres". This script provides the missing half - it boots the dev Postgres, applies every
# crate's migrations, and runs the ignored tests, so they actually execute.
#
# Usage (from anywhere):
#   services/dev/test-db.sh                     # all workspace crates
#   services/dev/test-db.sh -p cyberos-memory   # one crate (extra args pass through to `cargo test`)
#   services/dev/test-db.sh -p cyberos-auth --test capture_signin_test
#
# Requires: Docker (daemon up) + the Rust toolchain. Leaves the containers running; stop with:
#   docker compose -f services/dev/docker-compose.yml down        # keep data
#   docker compose -f services/dev/docker-compose.yml down -v     # wipe data
set -euo pipefail

DEV="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$DEV/../.." && pwd)"
export DATABASE_URL="${DATABASE_URL:-postgres://cyberos:cyberos@localhost:5432/cyberos}"

echo "==> booting Postgres + Redis (pgvector image)"
docker compose -f "$DEV/docker-compose.yml" up -d >/dev/null

echo "==> waiting for Postgres to accept connections"
for _ in $(seq 1 30); do
  docker exec cyberos-postgres pg_isready -U cyberos -d cyberos >/dev/null 2>&1 && break
  sleep 1
done

echo "==> ensuring extensions (idempotent)"
docker exec cyberos-postgres psql -U cyberos -d cyberos -qc \
  'CREATE EXTENSION IF NOT EXISTS vector; CREATE EXTENSION IF NOT EXISTS pgcrypto; CREATE EXTENSION IF NOT EXISTS "uuid-ossp";' >/dev/null

echo "==> applying migrations (idempotent; auth roles first, then memory, eval, chat)"
# Order matters: auth creates the cyberos_app role + tenants that memory/eval rows reference. Applied
# leniently (ON_ERROR_STOP=0 + the migrations' own IF NOT EXISTS) so a pre-migrated DB is a clean no-op.
for crate in auth memory eval chat; do
  d="$ROOT/services/$crate/migrations"
  [ -d "$d" ] || continue
  for f in "$d"/*.sql; do
    [ -e "$f" ] || continue
    docker exec -i cyberos-postgres psql -U cyberos -d cyberos -v ON_ERROR_STOP=0 -q < "$f" >/dev/null 2>&1 || true
  done
done

echo "==> cargo test -- --ignored  (single-threaded: parallel DDL races Postgres catalog updates)"
cd "$ROOT/services"
# --no-fail-fast is a `cargo test` flag (before --); --ignored/--test-threads are libtest (after --).
if [ "$#" -gt 0 ]; then
  exec cargo test "$@" --no-fail-fast -- --ignored --test-threads=1
else
  exec cargo test --workspace --no-fail-fast -- --ignored --test-threads=1
fi
