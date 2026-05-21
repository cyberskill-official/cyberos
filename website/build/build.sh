#!/usr/bin/env bash
# website/build/build.sh — regenerate website data from source files.
#
# Usage:
#   website/build/build.sh          # full build (FR + NFR catalogs + changelog)
#   website/build/build.sh --fr     # FR catalog only
#   website/build/build.sh --nfr    # NFR catalog only
#   website/build/build.sh --changelog  # changelog only
#
# The build is deterministic — same input ⇒ byte-identical output (FR-DOCS-001 §1 #3).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
MODE="${1:-full}"

cd "$REPO_ROOT"

# ── FR catalog ──────────────────────────────────────────────────────────────
if [[ "$MODE" == "full" || "$MODE" == "--fr" ]]; then
  echo "→ FR catalog"
  node website/build/data-extract.mjs
  node website/build/render-fr-catalog.mjs
fi

# ── NFR catalog ─────────────────────────────────────────────────────────────
if [[ "$MODE" == "full" || "$MODE" == "--nfr" ]]; then
  echo "→ NFR catalog"
  node website/build/nfr-extract.mjs
  node website/build/render-nfr-catalog.mjs
fi

# ── Changelog ──────────────────────────────────────────────────────────────
if [[ "$MODE" == "full" || "$MODE" == "--changelog" ]]; then
  echo "→ Changelog (consolidated)"
  node website/build/render-changelog.mjs
  echo "→ Changelog (per-module)"
  node website/build/render-module-changelog.mjs
fi

echo "✓ build complete"
