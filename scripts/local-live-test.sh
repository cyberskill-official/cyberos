#!/usr/bin/env bash
set -Eeuo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEV_COMPOSE="$ROOT/services/dev/docker-compose.yml"
CHAT_COMPOSE="$ROOT/services/chat/compose.yml"

AUTH_DATABASE_URL="${AUTH_DATABASE_URL:-postgres://cyberos:cyberos@localhost:5432/cyberos_auth}"
MEMORY_DATABASE_URL="${MEMORY_DATABASE_URL:-postgres://cyberos:cyberos@localhost:5432/cyberos_memory}"
PROJ_DATABASE_URL="${PROJ_DATABASE_URL:-postgres://cyberos:cyberos@localhost:5432/cyberos_proj}"

usage() {
  cat <<'EOF'
CyberOS local live-test helper.

Usage:
  scripts/local-live-test.sh doctor
  scripts/local-live-test.sh infra-up
  scripts/local-live-test.sh infra-down
  scripts/local-live-test.sh migrate
  scripts/local-live-test.sh test
  scripts/local-live-test.sh live-db
  scripts/local-live-test.sh chat-up
  scripts/local-live-test.sh chat-down

Commands:
  doctor      Check local tools and Docker daemon reachability.
  infra-up    Start Postgres/Redis from services/dev/docker-compose.yml.
  infra-down  Stop Postgres/Redis.
  migrate     Apply AUTH, MEMORY, and PROJ migrations to separate local DBs.
  test        Run non-live verification for SKILL, MEMORY, CUO, AUTH, CHAT, PROJ.
  live-db     Run ignored Postgres-gated AUTH/MEMORY tests after migrate.
  chat-up     Build/start the CHAT compose stack on http://localhost:8065.
  chat-down   Stop the CHAT compose stack.

Notes:
  - PROJ currently ships as a Rust library/test surface, not an HTTP daemon.
  - SKILL and CUO do not need Docker for their local verification path.
  - chat-up is network-heavy because it builds Mattermost from the pinned SHA.
EOF
}

section() {
  printf '\n== %s ==\n' "$1"
}

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    printf 'missing required command: %s\n' "$1" >&2
    return 1
  fi
}

docker_ready() {
  docker version >/dev/null 2>&1
}

doctor() {
  local failed=0
  section "tooling"
  for cmd in cargo python docker; do
    if need_cmd "$cmd"; then
      "$cmd" --version 2>/dev/null | head -1 || true
    else
      failed=1
    fi
  done

  if command -v sqlx >/dev/null 2>&1; then
    sqlx --version
  else
    printf 'sqlx: missing (optional; Docker migrator is used by default)\n'
  fi

  if command -v psql >/dev/null 2>&1; then
    psql --version | head -1
  else
    printf 'psql: missing (optional, useful for manual DB inspection)\n'
  fi

  section "docker"
  if docker_ready; then
    docker version --format 'server {{.Server.Version}}'
    docker compose version
  else
    printf 'Docker daemon is not reachable. Start Docker Desktop, then rerun infra-up.\n' >&2
    failed=1
  fi

  return "$failed"
}

infra_up() {
  need_cmd docker
  section "starting Postgres/Redis"
  docker compose -f "$DEV_COMPOSE" up -d
  docker compose -f "$DEV_COMPOSE" ps
}

infra_down() {
  need_cmd docker
  section "stopping Postgres/Redis"
  docker compose -f "$DEV_COMPOSE" down
}

migrate_all() {
  need_cmd docker
  section "migrate AUTH/MEMORY/PROJ"
  docker compose -f "$DEV_COMPOSE" --profile tools run --rm migrator
}

test_all() {
  section "MEMORY Python"
  (cd "$ROOT/modules/memory" && python -m pytest tests runtime/tests)

  section "MEMORY Rust service"
  (cd "$ROOT/services" && cargo test -p cyberos-memory)

  section "CUO"
  (cd "$ROOT/modules/cuo" && python -m pytest tests)

  section "SKILL Rust workspace"
  (cd "$ROOT/modules/skill" && cargo test --workspace)

  section "SKILL VN parity"
  (cd "$ROOT/modules/skill" && python tests/parity/run_parity.py)

  section "SKILL corpus smoke"
  (cd "$ROOT/modules/skill" && python tests/run_corpus.py fr-with-tasks --no-llm)

  section "AUTH and PROJ"
  (cd "$ROOT/services" && cargo test -p cyberos-auth -p cyberos-proj)

  section "CHAT"
  (cd "$ROOT/services/chat" && bash tests/run_all_tests.sh)
}

live_db() {
  section "AUTH Postgres-gated tests"
  (cd "$ROOT/services" && DATABASE_URL="$AUTH_DATABASE_URL" cargo test -p cyberos-auth -- --ignored)

  section "MEMORY Postgres-gated tests"
  (cd "$ROOT/services" && DATABASE_URL="$MEMORY_DATABASE_URL" cargo test -p cyberos-memory -- --ignored)
}

chat_up() {
  need_cmd docker
  local pinned
  local patch_version
  pinned="$(grep -E '^[0-9a-f]{40}' "$ROOT/services/chat/PINNED_COMMIT" | head -1 | cut -d' ' -f1)"
  patch_version="$(tr -d '[:space:]' < "$ROOT/services/chat/CYBEROS_PATCH_VERSION" | head -c 32)"
  if [[ -z "$pinned" ]]; then
    printf 'could not read pinned CHAT commit\n' >&2
    return 2
  fi
  section "starting CHAT"
  PINNED_COMMIT="$pinned" CYBEROS_PATCH_VERSION="$patch_version" \
    docker compose -f "$CHAT_COMPOSE" up -d --build
  docker compose -f "$CHAT_COMPOSE" ps
}

chat_down() {
  need_cmd docker
  section "stopping CHAT"
  docker compose -f "$CHAT_COMPOSE" down
}

cmd="${1:-help}"
case "$cmd" in
  doctor) doctor ;;
  infra-up) infra_up ;;
  infra-down) infra_down ;;
  migrate) migrate_all ;;
  test) test_all ;;
  live-db) live_db ;;
  chat-up) chat_up ;;
  chat-down) chat_down ;;
  help|-h|--help) usage ;;
  *)
    usage >&2
    exit 2
    ;;
esac
