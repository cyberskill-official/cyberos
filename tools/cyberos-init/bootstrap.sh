#!/usr/bin/env bash
# bootstrap.sh - one-liner installer. Fetch the CyberOS payload and init the current repo:
#   curl -fsSL <raw-url>/bootstrap.sh | bash
# Requires a published pack tarball. Set CYBEROS_PACK_URL to point at your release.
# If you already have the pack folder locally, skip this and run init.sh directly.
set -euo pipefail

CYBEROS_PACK_URL="${CYBEROS_PACK_URL:-}"
target="${1:-$(pwd)}"

if [ -z "$CYBEROS_PACK_URL" ]; then
  cat >&2 <<'EOF'
cyberos bootstrap: no CYBEROS_PACK_URL set.
Publish a pack tarball (built with build.sh, then tar -czf cyberos.tar.gz -C dist cyberos)
and re-run:  CYBEROS_PACK_URL=<url-to-cyberos.tar.gz> curl -fsSL <bootstrap-url> | bash
Or use a channel that needs no hosting: copy the folder, add it as a git submodule, or
load plugin/ as a Claude plugin. See the pack README.
EOF
  exit 1
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
echo "cyberos bootstrap: downloading $CYBEROS_PACK_URL"
curl -fsSL "$CYBEROS_PACK_URL" -o "$tmp/cyberos.tar.gz"
mkdir -p "$tmp/cyberos"
tar -xzf "$tmp/cyberos.tar.gz" -C "$tmp/cyberos" --strip-components=1
bash "$tmp/cyberos/init.sh" "$target"
