#!/usr/bin/env bash
#
# scripts/release.sh — bump CyberOS version across the monorepo.
#
# Single source of truth: VERSION file at repo root.
# Propagates to all pyproject.toml + __init__.py files that declare a version.
#
# Usage:
#     scripts/release.sh major          # 0.1.0 → 1.0.0
#     scripts/release.sh minor          # 0.1.0 → 0.2.0
#     scripts/release.sh patch          # 0.1.0 → 0.1.1
#     scripts/release.sh 2.5.0          # explicit version
#
#     # Dry run (show what would change, don't write):
#     scripts/release.sh --dry-run minor
#
#     # Skip git commit + tag:
#     scripts/release.sh --no-commit patch

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VERSION_FILE="$REPO_ROOT/VERSION"
DRY_RUN=0
NO_COMMIT=0
BUMP=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --dry-run)    DRY_RUN=1; shift ;;
        --no-commit)  NO_COMMIT=1; shift ;;
        -h|--help)
            grep '^#' "$0" | sed 's/^# \{0,1\}//'
            exit 0 ;;
        *)
            if [[ -z "$BUMP" ]]; then BUMP="$1"
            else echo "error: unexpected arg: $1" >&2; exit 2; fi
            shift ;;
    esac
done

if [[ -z "$BUMP" ]]; then
    echo "error: specify bump type: major | minor | patch | X.Y.Z" >&2
    exit 2
fi

# --- read current version --------------------------------------------------

if [[ ! -f "$VERSION_FILE" ]]; then
    echo "error: $VERSION_FILE not found" >&2
    exit 1
fi

CURRENT=$(cat "$VERSION_FILE" | tr -d '[:space:]')
IFS='.' read -r MAJOR MINOR PATCH <<< "${CURRENT%%[-+]*}"

# --- compute new version ---------------------------------------------------

if [[ "$BUMP" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$ ]]; then
    NEW="$BUMP"
elif [[ "$BUMP" == "major" ]]; then
    NEW="$((MAJOR + 1)).0.0"
elif [[ "$BUMP" == "minor" ]]; then
    NEW="${MAJOR}.$((MINOR + 1)).0"
elif [[ "$BUMP" == "patch" ]]; then
    NEW="${MAJOR}.${MINOR}.$((PATCH + 1))"
else
    echo "error: invalid bump: $BUMP (use major|minor|patch|X.Y.Z)" >&2
    exit 2
fi

echo "=== cyberos release ==="
echo "  current : $CURRENT"
echo "  new     : $NEW"
echo

# --- discover targets ------------------------------------------------------

TARGETS=()

# All pyproject.toml with a version = "..." line
while IFS= read -r f; do
    if grep -q '^version\s*=' "$f" 2>/dev/null; then
        TARGETS+=("$f")
    fi
done < <(find "$REPO_ROOT/modules" -name "pyproject.toml" 2>/dev/null)

# All __init__.py with __version__ = "..."
while IFS= read -r f; do
    if grep -q '__version__' "$f" 2>/dev/null; then
        TARGETS+=("$f")
    fi
done < <(find "$REPO_ROOT/modules" -path "*/cyberos*/__init__.py" -o -path "*/cuo*/__init__.py" 2>/dev/null | sort -u)

echo "→ targets (${#TARGETS[@]} files):"
for f in "${TARGETS[@]}"; do
    rel="${f#$REPO_ROOT/}"
    if [[ "$f" == *pyproject.toml ]]; then
        old=$(grep '^version' "$f" | head -1 | sed 's/.*"\(.*\)".*/\1/')
    else
        old=$(grep '__version__' "$f" | sed 's/.*"\(.*\)".*/\1/')
    fi
    echo "  $rel  ($old → $NEW)"
done
echo

# --- apply ------------------------------------------------------------------

if [[ "$DRY_RUN" == "1" ]]; then
    echo "  (dry run — no changes written)"
    exit 0
fi

# Write VERSION
echo "$NEW" > "$VERSION_FILE"
echo "  ✓ VERSION"

# Update pyproject.toml files
for f in "${TARGETS[@]}"; do
    if [[ "$f" == *pyproject.toml ]]; then
        # Match version = "X.Y.Z" or version = "X.Y.Zalpha1" etc.
        sed -i '' "s/^version\s*=.*$/version = \"$NEW\"/" "$f"
        echo "  ✓ ${f#$REPO_ROOT/}"
    fi
done

# Update __init__.py files
for f in "${TARGETS[@]}"; do
    if [[ "$f" == *__init__.py ]]; then
        sed -i '' "s/__version__ = \".*\"/__version__ = \"$NEW\"/" "$f"
        echo "  ✓ ${f#$REPO_ROOT/}"
    fi
done
echo

# --- git commit + tag ------------------------------------------------------

if [[ "$NO_COMMIT" == "1" ]]; then
    echo "  (skipping git commit + tag)"
    exit 0
fi

cd "$REPO_ROOT"
if git rev-parse --git-dir >/dev/null 2>&1; then
    # Stage only the files we touched
    git add VERSION
    for f in "${TARGETS[@]}"; do
        git add "$f"
    done

    if git diff --cached --quiet; then
        echo "  no changes to commit"
    else
        git commit -m "release: v$NEW"
        git tag -a "v$NEW" -m "v$NEW"
        echo "  ✓ committed + tagged v$NEW"
    fi
else
    echo "  (not a git repo — skipping commit)"
fi

echo
echo "=== done: v$NEW ==="
