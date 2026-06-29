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

echo "==> pulling new images from GHCR"
"${COMPOSE[@]}" pull

echo "==> rolling the stack"
"${COMPOSE[@]}" up -d --remove-orphans

# Caddy serves the console + config from the git checkout. Static console files are served live, but a
# changed Caddyfile needs a reload to take effect; ignore the error if caddy is mid-restart.
"${COMPOSE[@]}" exec -T caddy caddy reload --config /etc/caddy/Caddyfile 2>/dev/null || true

echo "==> pruning dangling images"
docker image prune -f >/dev/null || true

echo "==> deploy complete"
