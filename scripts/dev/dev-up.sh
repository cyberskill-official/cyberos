#!/usr/bin/env bash
# One-command local bring-up of auth + chat + the console, so you can sign in and test.
# Requires the dev Postgres container (cyberos-postgres) to be running. Dev only.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
SERVICES="$ROOT/services"
CONSOLE="$ROOT/apps/console"
PIDS="$HERE/.pids"
: > "$PIDS"

# Optional local overrides / secrets (gitignored).
[ -f "$HERE/dev.env" ] && { set -a; . "$HERE/dev.env"; set +a; }

PGC="${PGCONTAINER:-cyberos-postgres}"
DB_BASE="${DB_BASE:-postgres://cyberos:cyberos@localhost:5432}"
AUTH_ADDR="${AUTH_LISTEN_ADDR:-0.0.0.0:7700}"
CHAT_ADDR="${CHAT_LISTEN_ADDR:-127.0.0.1:7720}"
CONSOLE_PORT="${CONSOLE_PORT:-8090}"
DEMO_PASSWORD="${DEMO_PASSWORD:-CyberOS-Demo-2026!}"

docker ps --format '{{.Names}}' | grep -q "^${PGC}$" \
  || { echo "dev Postgres '${PGC}' is not running. Start it, then re-run." >&2; exit 1; }

echo "==> ensuring cyberos_chat database + migrations"
docker exec "$PGC" psql -U cyberos -d postgres -tAc "select 1 from pg_database where datname='cyberos_chat'" | grep -q 1 \
  || docker exec "$PGC" psql -U cyberos -d postgres -c "CREATE DATABASE cyberos_chat" >/dev/null
HAS="$(docker exec "$PGC" psql -U cyberos -d cyberos_chat -tAc "select to_regclass('public.chat_channels') is not null")"
if [ "$HAS" != "t" ]; then
  for f in "$SERVICES"/chat/migrations/*.sql; do
    echo "    applying $(basename "$f")"
    docker exec -i "$PGC" psql -U cyberos -d cyberos_chat -v ON_ERROR_STOP=1 -q -f - < "$f"
  done
fi

echo "==> building auth + chat (debug)"
( cd "$SERVICES" && cargo build -q -p cyberos-auth -p cyberos-chat --bins )

echo "==> starting auth on ${AUTH_ADDR}"
( cd "$SERVICES"
  AUTH_LISTEN_ADDR="$AUTH_ADDR" \
  AUTH_JWT_ISSUER="${AUTH_JWT_ISSUER:-http://localhost:7700}" \
  AUTH_CURSOR_SIGNING_SECRET="${AUTH_CURSOR_SIGNING_SECRET:-dev-only-cursor-secret-change-me}" \
  DATABASE_URL="${AUTH_DATABASE_URL:-$DB_BASE/cyberos}" \
  REDIS_URL="${REDIS_URL:-redis://localhost:6379}" \
  AUTH_DEV_CORS=1 \
  nohup target/debug/cyberos-auth > /tmp/cyberos-auth.log 2>&1 &
  echo "auth $!" >> "$PIDS" )
for _ in $(seq 1 40); do curl -sf http://127.0.0.1:7700/healthz >/dev/null 2>&1 && break; sleep 1; done
curl -sf http://127.0.0.1:7700/healthz >/dev/null 2>&1 || { echo "auth did not become healthy (see /tmp/cyberos-auth.log)" >&2; exit 1; }

echo "==> starting chat on ${CHAT_ADDR} (verifying tokens via auth JWKS URL)"
( cd "$SERVICES"
  CHAT_LISTEN_ADDR="$CHAT_ADDR" \
  DATABASE_URL="${CHAT_DATABASE_URL:-$DB_BASE/cyberos_chat}" \
  CHAT_AUDIT_DATABASE_URL="${CHAT_AUDIT_DATABASE_URL:-$DB_BASE/cyberos}" \
  CHAT_AUTH_JWKS_URL="${CHAT_AUTH_JWKS_URL:-http://127.0.0.1:7700/.well-known/jwks.json}" \
  CHAT_DEV_CORS=1 \
  nohup target/debug/cyberos-chat > /tmp/cyberos-chat.log 2>&1 &
  echo "chat $!" >> "$PIDS" )
for _ in $(seq 1 40); do curl -sf http://127.0.0.1:7720/healthz >/dev/null 2>&1 && break; sleep 1; done
curl -sf http://127.0.0.1:7720/healthz >/dev/null 2>&1 || { echo "chat did not become healthy (see /tmp/cyberos-chat.log)" >&2; exit 1; }

echo "==> seeding demo user"
DEMO_PASSWORD="$DEMO_PASSWORD" "$HERE/seed-demo-user.sh"

echo "==> serving console on ${CONSOLE_PORT}"
( cd "$CONSOLE" && nohup python3 -m http.server "$CONSOLE_PORT" --bind 127.0.0.1 > /tmp/cyberos-console.log 2>&1 &
  echo "console $!" >> "$PIDS" )

cat <<EOF

Ready.
  Open:     http://127.0.0.1:${CONSOLE_PORT}/app.html
  Sign in:  workspace = cyberskill   handle = @stephen   password = ${DEMO_PASSWORD}
  Logs:     /tmp/cyberos-auth.log  /tmp/cyberos-chat.log  /tmp/cyberos-console.log
  Stop:     ${HERE}/dev-down.sh
EOF
