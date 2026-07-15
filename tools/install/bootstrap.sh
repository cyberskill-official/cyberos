#!/usr/bin/env bash
# bootstrap.sh - one-liner installer (TASK-IMP-069). Fetch the released CyberOS payload, verify its
# checksum against the SHA256SUMS published beside it, and init the target repo:
#   curl -fsSL https://raw.githubusercontent.com/cyberskill-official/cyberos/main/tools/install/bootstrap.sh | bash
# env:
#   CYBEROS_PAYLOAD_URL  payload tarball URL (default: this repo's latest-release stable asset)
#   CYBEROS_PACK_URL     legacy alias, still honored
# arg: target repo (default: cwd). If you already have a payload folder locally, run install.sh directly.
set -euo pipefail

DEFAULT_URL="https://github.com/cyberskill-official/cyberos/releases/latest/download/cyberos-payload.tar.gz"
url="${CYBEROS_PAYLOAD_URL:-${CYBEROS_PACK_URL:-$DEFAULT_URL}}"
target="${1:-$(pwd)}"

tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
echo "cyberos bootstrap: downloading $url"
curl -fsSL "$url" -o "$tmp/cyberos-payload.tar.gz"

sums_url="$(dirname "$url")/SHA256SUMS"
echo "cyberos bootstrap: verifying checksum against $sums_url"
curl -fsSL "$sums_url" -o "$tmp/SHA256SUMS" \
  || { echo "cyberos bootstrap: ERROR: cannot fetch SHA256SUMS beside the tarball - refusing to install unverified bits" >&2; exit 1; }
(cd "$tmp" && grep " cyberos-payload.tar.gz\$" SHA256SUMS | sha256sum -c - >/dev/null) \
  || { echo "cyberos bootstrap: ERROR: checksum mismatch - aborting before touching $target" >&2; exit 1; }

mkdir -p "$tmp/payload"
tar -xzf "$tmp/cyberos-payload.tar.gz" -C "$tmp/payload"
if [ ! -f "$tmp/payload/install.sh" ]; then   # tarballs may carry one top-level dir
  sub="$(find "$tmp/payload" -mindepth 1 -maxdepth 1 -type d | head -1)"
  [ -n "$sub" ] && [ -f "$sub/install.sh" ] && mv "$sub"/* "$tmp/payload/" || { echo "cyberos bootstrap: ERROR: no install.sh in the payload" >&2; exit 1; }
fi
bash "$tmp/payload/install.sh" "$target"
