#!/usr/bin/env bash
#
# scripts/install-pre-commit.sh — install the BRAIN pre-commit hook.
#
# Symlinks scripts/hooks/pre-commit into the target project's
# .git/hooks/. The hook refuses commits that would corrupt the BRAIN
# (doctor failure, schema-invalid memory file, schema drift).
#
# Usage:
#     ./scripts/install-pre-commit.sh                # current dir
#     ./scripts/install-pre-commit.sh ~/Projects/x   # specific project

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
TARGET="${1:-$(pwd)}"
TARGET="$(cd "$TARGET" && pwd)"

if [[ ! -d "$TARGET/.git" ]]; then
    echo "error: $TARGET has no .git/ — not a git repo" >&2
    exit 2
fi

HOOK="$TARGET/.git/hooks/pre-commit"

if [[ -e "$HOOK" && ! -L "$HOOK" ]]; then
    echo "warning: $HOOK already exists and is NOT a symlink" >&2
    echo "  back it up and re-run, or merge manually" >&2
    exit 2
fi

mkdir -p "$TARGET/.git/hooks"
ln -sf "$REPO_ROOT/memory/scripts/hooks/pre-commit" "$HOOK"
chmod +x "$REPO_ROOT/memory/scripts/hooks/pre-commit"

echo "✓ installed: $HOOK → $REPO_ROOT/memory/scripts/hooks/pre-commit"
echo
echo "to test:"
echo "  cd $TARGET && touch .cyberos-memory/memories/decisions/dummy.md && git add ."
echo "  git commit -m 'test'    # should fail validation"
echo "  git restore --staged .  # undo"
echo
echo "to uninstall:"
echo "  rm $HOOK"
