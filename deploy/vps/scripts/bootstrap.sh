#!/bin/sh
set -eu

if [ "$(id -u)" -ne 0 ]; then
  echo "Run this on the VPS as root: sudo deploy/vps/scripts/bootstrap.sh" >&2
  exit 1
fi

REPO_DIR="${REPO_DIR:-/opt/cyberos/repo}"

if ! command -v docker >/dev/null 2>&1; then
  apt-get update
  apt-get install -y ca-certificates curl gnupg
  install -m 0755 -d /etc/apt/keyrings
  curl -fsSL https://download.docker.com/linux/ubuntu/gpg -o /etc/apt/keyrings/docker.asc
  chmod a+r /etc/apt/keyrings/docker.asc
  . /etc/os-release
  echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/ubuntu ${VERSION_CODENAME} stable" \
    > /etc/apt/sources.list.d/docker.list
  apt-get update
  apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
fi

mkdir -p /opt/cyberos/data/memory-root /opt/cyberos/backups

if [ ! -f "$REPO_DIR/deploy/vps/.env" ]; then
  cp "$REPO_DIR/deploy/vps/env.production.example" "$REPO_DIR/deploy/vps/.env"
  chmod 600 "$REPO_DIR/deploy/vps/.env"
  echo "Created $REPO_DIR/deploy/vps/.env. Edit secrets before starting CyberOS."
fi

cp "$REPO_DIR/deploy/vps/cyberos.service" /etc/systemd/system/cyberos.service
systemctl daemon-reload
systemctl enable cyberos.service

echo "Bootstrap complete."
echo "Next:"
echo "  1. Edit $REPO_DIR/deploy/vps/.env"
echo "  2. systemctl start cyberos"
echo "  3. $REPO_DIR/deploy/vps/scripts/healthcheck.sh http://127.0.0.1:8080"
