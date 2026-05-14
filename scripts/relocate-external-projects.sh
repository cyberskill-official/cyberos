#!/usr/bin/env bash
#
# relocate-external-projects.sh
#
# Earlier consolidation accidentally absorbed `/design-system/` and
# `/landing-page/` (each its own git repo) INTO `cyberos/`. This script
# undoes that — syncs the Liquid Glass v1.1.0 changes back to the
# external `design-system/` repo, removes the duplicates from inside
# `cyberos/`, and leaves the umbrella matching the README.md layout.
#
# Run from the host shell (not from inside Cowork/Claude Code, which
# can't unlink files outside cyberos/).
#
# Usage:
#     bash cyberos/scripts/relocate-external-projects.sh
#
# Idempotent — safe to re-run.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
CYBEROS="$ROOT/cyberos"
DS_EXT="$ROOT/design-system"
LP_EXT="$ROOT/landing-page"
DS_INNER="$CYBEROS/design-system"
LP_INNER="$CYBEROS/website/landing"

echo "=== Relocate external projects ==="
echo "Root: $ROOT"
echo

# --------------------------------------------------------------------- 1. Sync
# Three files were updated inside cyberos/design-system/ during the Liquid
# Glass work (v1.1.0 — Part 21 added). Sync them back to the external repo.

if [[ -d "$DS_INNER" && -d "$DS_EXT" ]]; then
    echo "→ Step 1: Sync v1.1.0 changes from cyberos/design-system/ to ../design-system/"
    for f in DESIGN.md README.md CHANGELOG.md; do
        if [[ -f "$DS_INNER/$f" ]]; then
            cp "$DS_INNER/$f" "$DS_EXT/$f"
            echo "  copied: design-system/$f"
        fi
    done
    echo
else
    echo "→ Step 1: SKIPPED (either source or target missing — already cleaned?)"
    echo
fi

# --------------------------------------------------------------------- 2. Delete

if [[ -d "$DS_INNER" ]]; then
    echo "→ Step 2: Remove duplicate cyberos/design-system/"
    rm -rf "$DS_INNER"
    echo "  removed: cyberos/design-system/"
else
    echo "→ Step 2: SKIPPED (cyberos/design-system/ already gone)"
fi

if [[ -d "$LP_INNER" ]]; then
    echo "→ Step 3: Remove duplicate cyberos/website/landing/"
    rm -rf "$LP_INNER"
    echo "  removed: cyberos/website/landing/"
else
    echo "→ Step 3: SKIPPED (cyberos/website/landing/ already gone)"
fi

echo

# --------------------------------------------------------------------- 4. Verify

echo "→ Step 4: Verify final state"

if [[ -d "$DS_EXT" ]]; then
    DS_VER=$(grep -m1 "^| Version" "$DS_EXT/README.md" | sed -E 's/.*\*\*([^*]+)\*\*.*/\1/' || echo "unknown")
    echo "  ../design-system/ exists; README version: $DS_VER"
else
    echo "  ../design-system/ MISSING — investigate"
fi

if [[ -d "$LP_EXT" ]]; then
    echo "  ../landing-page/ exists"
else
    echo "  ../landing-page/ MISSING — investigate"
fi

if [[ -d "$DS_INNER" ]]; then
    echo "  cyberos/design-system/ STILL PRESENT — investigate"
else
    echo "  cyberos/design-system/ removed ✓"
fi

if [[ -d "$LP_INNER" ]]; then
    echo "  cyberos/website/landing/ STILL PRESENT — investigate"
else
    echo "  cyberos/website/landing/ removed ✓"
fi

echo

# --------------------------------------------------------------------- 5. Git state

cd "$CYBEROS"
if git rev-parse --is-inside-work-tree > /dev/null 2>&1; then
    echo "→ Step 5: cyberos/ git status"
    git status --short | head -10
    echo
fi

cd "$DS_EXT" 2>/dev/null && {
    echo "→ design-system/ git status"
    git status --short | head -10
    echo
    echo "  When ready, commit the Liquid Glass v1.1.0 changes:"
    echo "    cd $DS_EXT"
    echo "    git add DESIGN.md README.md CHANGELOG.md"
    echo "    git commit -m 'feat: v1.1.0 — Part 21 Liquid Glass default'"
    echo "    git tag v1.1.0"
    echo
}

echo "=== Done ==="
