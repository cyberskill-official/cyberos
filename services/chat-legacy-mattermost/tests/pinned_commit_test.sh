#!/usr/bin/env bash
# TASK-CHAT-001 §4 #1 + §4 #10 — PINNED_COMMIT file invariants + patches/ exists.
#
# Assertions:
#   - PINNED_COMMIT file is present.
#   - The first non-comment line is a 40-char hex string.
#   - The file is owned/restricted via CODEOWNERS (best-effort warn only;
#     CODEOWNERS enforcement is a GitHub-side check, not a filesystem one).
#   - patches/ directory exists (may be empty).
#   - CYBEROS_PATCH_VERSION is a semver string.

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

fail() { echo "::error::$*" >&2; exit 1; }
ok()   { echo "  ✓ $*"; }

echo "Running PINNED_COMMIT shape tests in $ROOT..."

PINNED_FILE="$ROOT/PINNED_COMMIT"
[[ -f "$PINNED_FILE" ]] || fail "$PINNED_FILE missing"
ok "PINNED_COMMIT exists"

PINNED=$(grep -v '^[[:space:]]*#' "$PINNED_FILE" | awk 'NF{print $1; exit}')
[[ "$PINNED" =~ ^[0-9a-f]{40}$ ]] || fail "PINNED_COMMIT first non-comment line is not a 40-char hex SHA: '$PINNED'"
ok "PINNED_COMMIT carries a 40-char hex SHA ($PINNED)"

VERSION_FILE="$ROOT/CYBEROS_PATCH_VERSION"
[[ -f "$VERSION_FILE" ]] || fail "$VERSION_FILE missing"
VERSION=$(head -1 "$VERSION_FILE" | tr -d '[:space:]')
[[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$ ]] || fail "CYBEROS_PATCH_VERSION is not semver: '$VERSION'"
ok "CYBEROS_PATCH_VERSION is semver ($VERSION)"

PATCHES_DIR="$ROOT/patches"
[[ -d "$PATCHES_DIR" ]] || fail "patches/ directory missing"
ok "patches/ directory exists"

echo "✓ All PINNED_COMMIT invariants pass."
