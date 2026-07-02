#!/usr/bin/env bash
# Roll the CyberOS P0 stack to the latest images + repo state. Run by the GitHub Actions auto-deploy
# (.github/workflows/deploy.yml) over SSH on every push to main, and usable by hand for a manual deploy:
#
#   bash ~/cyberos/deploy/vps/deploy.sh
#
# It pulls the latest main (for the Caddyfile, console, and compose), pulls the freshly built auth + chat
# images from GHCR, and restarts the stack. The VPS never compiles anything.
set -euo pipefail

REPO_DIR="${CYBEROS_REPO_DIR:-$HOME/cyberos}"
cd "$REPO_DIR"

echo "==> pulling latest main"
git pull --ff-only origin main

cd deploy/vps
COMPOSE=(docker compose --env-file .env.p0 -f docker-compose.p0.images.yml)

# Core services the team depends on (Google login + chat + the single-origin router). eval (BRAIN/EVAL) is
# best-effort - still stabilising, nothing depends on it, and it must never block the core stack.
CORE=(auth chat caddy)

# AI group (gateway + bge-m3 embeddings) - best-effort like eval: chat degrades gracefully when the gateway
# is absent, so an AI failure must never block the core deploy. The ollama chat LLM sits behind the compose
# profile "llm": enable it by setting COMPOSE_PROFILES=llm in .env.p0 (after the VPS resize - see
# docs/deploy/ai-gateway-and-embeddings.md), and every later deploy keeps it automatically.
AI=(ai-gateway embed)
LLM_ON=0
if grep -qE '^COMPOSE_PROFILES=.*llm' .env.p0 2>/dev/null; then
  LLM_ON=1
  AI+=(ollama)
fi

echo "==> pulling new images from GHCR"
"${COMPOSE[@]}" pull "${CORE[@]}"
"${COMPOSE[@]}" pull "${AI[@]}" || echo "==> AI images not all available yet; continuing (best-effort group)"
# eval (BRAIN/EVAL) is OFF by default: it opens its own Supabase pooler connections and the small pooler
# tier cannot spare them next to auth + chat. Turn it on with DEPLOY_EVAL=1 once the pooler limit is raised.
if [ "${DEPLOY_EVAL:-0}" = "1" ]; then
  "${COMPOSE[@]}" pull eval || echo "==> eval image not available yet; continuing without it"
fi

# Apply DB migrations for every service before starting (uniform, per-service tracked, idempotent). The DB
# URL is read straight out of .env.p0 (not sourced as shell). Best-effort: auth/chat are baselined so their
# live schema is untouched, and a failure can only be in a new/eval/memory migration - which must not block
# the team's core deploy.
MIGRATIONS_DATABASE_URL="$(grep -E '^MIGRATIONS_DATABASE_URL=' .env.p0 2>/dev/null | cut -d= -f2- || true)"
[ -n "$MIGRATIONS_DATABASE_URL" ] || MIGRATIONS_DATABASE_URL="$(grep -E '^AUTH_DATABASE_URL=' .env.p0 2>/dev/null | cut -d= -f2- || true)"
export MIGRATIONS_DATABASE_URL
echo "==> applying migrations"
bash ./migrate.sh || echo "==> migrate.sh reported an error; deploying the core stack anyway"

echo "==> rolling the core stack"
"${COMPOSE[@]}" up -d --remove-orphans "${CORE[@]}"

echo "==> rolling the AI group (best-effort)"
"${COMPOSE[@]}" up -d "${AI[@]}" || echo "==> AI group not (fully) started; core stack is unaffected"
if [ "$LLM_ON" = "1" ]; then
  # Make sure the chat model behind chat.smart/chat.fast exists in the ollama store. Idempotent: a pull of
  # a present model is a fast no-op. Model id must match deploy/vps/ai/tenants/org-cyberskill.yaml.
  OLLAMA_CHAT_MODEL="${OLLAMA_CHAT_MODEL:-qwen2.5:3b-instruct}"
  echo "==> ensuring ollama model ${OLLAMA_CHAT_MODEL} is present"
  "${COMPOSE[@]}" exec -T ollama ollama pull "${OLLAMA_CHAT_MODEL}" \
    || echo "==> ollama model pull failed; translation stays degraded until it succeeds"
fi

if [ "${DEPLOY_EVAL:-0}" = "1" ]; then
  echo "==> rolling eval (DEPLOY_EVAL=1)"
  "${COMPOSE[@]}" up -d eval || echo "==> eval not started (build/image pending); core stack is up"
else
  echo "==> eval NOT deployed (DEPLOY_EVAL!=1); stopping any running eval to free DB connections"
  "${COMPOSE[@]}" stop eval 2>/dev/null || true
fi

# Caddy serves the console + config from the git checkout. Static console files are served live, but a
# changed Caddyfile needs a reload to take effect. The reload used to swallow its own errors, which let a
# stale config survive deploys unnoticed (found 2026-07-02: /status/ai 404 in prod while present in git) -
# so now a failed reload falls back to a container restart (seconds of downtime, guaranteed-fresh config).
if ! "${COMPOSE[@]}" exec -T caddy caddy reload --config /etc/caddy/Caddyfile; then
  echo "==> caddy reload failed; restarting caddy to force the fresh config"
  "${COMPOSE[@]}" restart caddy || echo "==> caddy restart failed too - check caddy logs"
fi

echo "==> pruning dangling images"
docker image prune -f >/dev/null || true

echo "==> deploy complete"
