#!/usr/bin/env bash
#
# package.sh — Bundle a CyberOS skill directory into a portable .skill archive.
#
# Output:
#   <name>-<version>.skill.tar.gz     — the bundle
#   <name>-<version>.skill.tar.gz.sha256 — content hash for resolver
#
# Usage:
#   bash skill/tools/package.sh <skill-dir> [--out <dir>]
#
# Phase 5+ swap: replace the tarball + sha256 with cosign-signed OCI artifact
# pushed to ghcr.io or agentskills.io.

set -euo pipefail

SKILL_DIR="${1:?usage: package.sh <skill-dir> [--out <dir>]}"
shift || true

OUT_DIR="."
while [[ $# -gt 0 ]]; do
    case "$1" in
        --out) OUT_DIR="$2"; shift 2 ;;
        *) echo "unknown arg: $1" >&2; exit 2 ;;
    esac
done

if [[ ! -f "$SKILL_DIR/SKILL.md" ]]; then
    echo "error: $SKILL_DIR/SKILL.md not found" >&2
    exit 2
fi

NAME=$(awk '/^name:/ {print $2; exit}' "$SKILL_DIR/SKILL.md")
VERSION=$(awk '/^  version:/ {gsub(/"/, "", $2); print $2; exit}' "$SKILL_DIR/SKILL.md")
VERSION="${VERSION:-0.1.0}"

BASE=$(basename "$SKILL_DIR")
if [[ "$BASE" != "$NAME" ]]; then
    echo "error: directory name '$BASE' must match SKILL.md name '$NAME'" >&2
    exit 2
fi

OUT="$OUT_DIR/${NAME}-${VERSION}.skill.tar.gz"
mkdir -p "$OUT_DIR"

# Tar with a deterministic, sorted member order so the hash is stable.
( cd "$(dirname "$SKILL_DIR")" && \
  find "$NAME" -print | LC_ALL=C sort | \
  tar --numeric-owner --owner=0 --group=0 --mtime='2000-01-01T00:00:00Z' \
      -czf - --no-recursion --files-from=- ) > "$OUT"

SHA=$(shasum -a 256 "$OUT" | awk '{print $1}')
echo "$SHA" > "${OUT}.sha256"

echo "  packaged: $OUT"
echo "  sha256  : $SHA"
echo "  bytes   : $(wc -c < "$OUT")"
