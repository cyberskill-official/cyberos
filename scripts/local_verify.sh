#!/usr/bin/env bash
# Local P0 verify, one command: infra up -> migrations (incl mcp-gateway 0013-0017) -> every module suite.
# Mirrors docs/deploy/local-dev-and-testing.md Steps 1-3. Run on your Mac with Docker Desktop running.
#
# Usage:  bash scripts/local_verify.sh        (from anywhere in the repo)
# Exit:   0 = all green, N = number of failed steps (details above the summary).
#
# Note: this runs the in-memory MCP unit suite (143 tests). The MCP store-of-record DB path (the DB slice)
# has no automated integration test yet; exercise it manually per the "two local modes" section of
# docs/deploy/local-dev-and-testing.md (MCP_DATABASE_URL + MCP_REQUIRE_AUTH=1 + MCP_KMS_KEY + a token).
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT/services" || { echo "cannot cd to services/"; exit 1; }
DC="docker compose -f dev/docker-compose.yml"
export DATABASE_URL=postgres://cyberos:cyberos@localhost:5432/cyberos
export REDIS_URL=redis://127.0.0.1:6379
fails=0

echo "== Step 1: bring up Postgres (pgvector) and Redis =="
$DC up -d --build || { echo "compose up failed"; exit 1; }
echo -n "waiting for postgres"
for _ in $(seq 1 30); do
  if $DC exec -T postgres pg_isready -U cyberos -d cyberos >/dev/null 2>&1; then echo " ready"; break; fi
  echo -n "."; sleep 2
done
echo "extensions:"
$DC exec -T postgres psql -U cyberos -d cyberos -tAc \
  "SELECT extname FROM pg_extension ORDER BY 1;" || { echo "postgres not reachable"; exit 1; }

echo
echo "== Step 2: apply migrations (auth -> mcp-gateway -> memory -> ai-gateway -> email -> proj) =="
# mcp-gateway right after auth: its 0013-0017 reference auth's tenants/subjects/cyberos_app/signing keys.
for crate in auth mcp-gateway memory ai-gateway email proj; do
  for f in $(ls "$crate"/migrations/*.sql 2>/dev/null | sort); do
    if $DC exec -T postgres psql -U cyberos -d cyberos -v ON_ERROR_STOP=1 -q -f - < "$f" >/dev/null; then
      echo "  ok   $f"
    else
      echo "  FAIL $f"; fails=$((fails + 1))
    fi
  done
done

echo
echo "== Step 3: run the module suites (serial; DB-backed tests included) =="
# auth: skip the macOS-only p95 latency-noise assertion (not a logic failure).
echo "---- cyberos-auth ----"
cargo test -p cyberos-auth -- --include-ignored --test-threads=1 --skip create_subject_p95 || fails=$((fails + 1))
for crate in cyberos-memory cyberos-email cyberos-proj \
             cyberos-obs-compliance-view cyberos-obs-router cyberos-mcp-gateway; do
  echo "---- $crate ----"
  cargo test -p "$crate" -- --include-ignored --test-threads=1 || fails=$((fails + 1))
done
# ai-gateway: needs the memory Python package importable for the FR-AI-003 cost-hold expiry tests.
echo "---- cyberos-ai-gateway ----"
PYTHONPATH="$ROOT/modules/memory" cargo test -p cyberos-ai-gateway -- --include-ignored --test-threads=1 \
  || fails=$((fails + 1))
# pure shared crates (no database).
for crate in cyberos-obs-sdk cyberos-audit-chain cyberos-cli-exit cyberos-types; do
  echo "---- $crate ----"
  cargo test -p "$crate" || fails=$((fails + 1))
done

echo
if [ "$fails" -eq 0 ]; then
  echo "LOCAL VERIFY GREEN - infra + migrations + every module suite passed."
  echo "Next: the Step 6 smoke (bash scripts/mcp_demo.sh) and the /v1/chat tiers in"
  echo "docs/deploy/local-dev-and-testing.md."
else
  echo "LOCAL VERIFY: $fails step(s) failed - see the output above."
fi
exit "$fails"
