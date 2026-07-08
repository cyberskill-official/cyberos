#!/usr/bin/env bash
# bootstrap.sh - one-liner installer. Fetch the FR pack and init the current repo:
#   curl -fsSL <raw-url>/bootstrap.sh | bash
# Requires a published pack tarball. Set FRPACK_URL to point at your release.
# If you already have the pack folder locally, skip this and run init.sh directly.
set -euo pipefail

FRPACK_URL="${FRPACK_URL:-}"
target="${1:-$(pwd)}"

if [ -z "$FRPACK_URL" ]; then
  cat >&2 <<'EOF'
fr-pack bootstrap: no FRPACK_URL set.
Publish a pack tarball (built with build-pack.sh, then tar -czf fr-pack.tar.gz -C dist fr-pack)
and re-run:  FRPACK_URL=<url-to-fr-pack.tar.gz> curl -fsSL <bootstrap-url> | bash
Or use a channel that needs no hosting: copy the folder, add it as a git submodule, or
load plugin/ as a Claude plugin. See the pack README.
EOF
  exit 1
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
echo "fr-pack bootstrap: downloading $FRPACK_URL"
curl -fsSL "$FRPACK_URL" -o "$tmp/fr-pack.tar.gz"
mkdir -p "$tmp/fr-pack"
tar -xzf "$tmp/fr-pack.tar.gz" -C "$tmp/fr-pack" --strip-components=1
bash "$tmp/fr-pack/init.sh" "$target"
