#!/usr/bin/env bash
# uninstall.sh — remove the vendored CyberOS machine from a repo (once / on demand).
# Keeps operator work: docs/feature-requests/, docs/status/, CHANGELOG.md, agent files.
# BRAIN store kept by default (CYBEROS_UNINSTALL_KEEP_BRAIN=0 to drop it).
#
#   bash .cyberos/uninstall.sh [repo]
#   bash <payload>/uninstall.sh [repo]
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
target="${1:-$(pwd)}"
root="$(cd "$target" 2>/dev/null && git rev-parse --show-toplevel 2>/dev/null || cd "$target" && pwd)"
CY="$root/.cyberos"

echo "cyberos uninstall: target=$root"

# Soft update check is irrelevant when removing — skip

if [ ! -d "$CY" ]; then
  echo "cyberos uninstall: nothing to do (no .cyberos/)"
  exit 0
fi

# 1. pre-commit: strip cyberos blocks / managed hook
hk="$root/.git/hooks/pre-commit"
if [ -f "$hk" ]; then
  if head -5 "$hk" 2>/dev/null | grep -q "cyberos-status-hook"; then
    rm -f "$hk"
    echo "  removed managed pre-commit hook"
  elif grep -q "cyberos-status-hook" "$hk" 2>/dev/null; then
    tmp="$hk.cyberos.tmp"
    sed '/# >>> cyberos-status-hook/,/# <<< cyberos-status-hook <<</d' "$hk" > "$tmp" && mv "$tmp" "$hk"
    chmod +x "$hk"
    echo "  stripped cyberos block from pre-commit"
  fi
fi

# 2. managed .gitignore block
gi="$root/.gitignore"
if [ -f "$gi" ] && grep -q 'cyberos' "$gi" 2>/dev/null; then
  tmp="$gi.cyberos.tmp"
  # strip marked block if present
  if grep -q '>>> cyberos' "$gi" 2>/dev/null; then
    sed '/# >>> cyberos/,/# <<< cyberos <<</d' "$gi" > "$tmp" && mv "$tmp" "$gi"
    echo "  removed managed .gitignore block"
  fi
fi

# 3. BRAIN store
brain="$CY/memory/store"
if [ "${CYBEROS_UNINSTALL_KEEP_BRAIN:-1}" = "1" ] && [ -d "$brain" ]; then
  stash="$(mktemp -d "${TMPDIR:-/tmp}/cyberos-brain.XXXXXX")"
  mv "$brain" "$stash/store"
  echo "  BRAIN stashed at $stash/store (restore under .cyberos/memory/store/ if needed)"
  KEEP_BRAIN_STASH="$stash/store"
else
  KEEP_BRAIN_STASH=""
  echo "  dropping BRAIN store (CYBEROS_UNINSTALL_KEEP_BRAIN=0 or absent)"
fi

# 4. remove machine
rm -rf "$CY"
echo "  removed .cyberos/"

# 5. optional restore brain only (minimal rehydrate)
if [ -n "${KEEP_BRAIN_STASH:-}" ] && [ -d "$KEEP_BRAIN_STASH" ]; then
  mkdir -p "$root/.cyberos/memory"
  mv "$KEEP_BRAIN_STASH" "$root/.cyberos/memory/store"
  rmdir "$(dirname "$KEEP_BRAIN_STASH")" 2>/dev/null || true
  echo "  restored BRAIN at .cyberos/memory/store/ (machine removed; re-install to restore workflow)"
fi

# 6. skill symlinks into .cyberos (dangling) — leave dirs; operator cleans
echo "cyberos uninstall: done."
echo "  kept: docs/feature-requests/, docs/status/, CHANGELOG.md, AGENTS.md / pointer files"
echo "  re-install: bash <payload>/install.sh $root"
