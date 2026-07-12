#!/usr/bin/env bash
# tools/docs-site/build.sh — regenerate website data from source files.
#
# Usage:
#   tools/docs-site/build.sh          # full build (FR + NFR catalogs + changelog)
#   tools/docs-site/build.sh --fr     # FR catalog only
#   tools/docs-site/build.sh --nfr    # NFR catalog only
#   tools/docs-site/build.sh --changelog  # changelog only
#
# The build is deterministic — same input ⇒ byte-identical output (FR-DOCS-001 §1 #3).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
MODE="${1:-full}"

cd "$REPO_ROOT"

# ── Site skeleton: the WHOLE site is generated into gitignored dist/website ──
# (FR-DOCS-002: nothing generated is committed). Chrome = the shared css/js/nav
# + the home page, maintained at tools/docs-site/{chrome,index.html}.
# A full build starts from a CLEAN tree: without this, pages whose source was
# removed or relocated would linger from earlier builds, breaking determinism
# (same input must yield byte-identical output, orphans included). Partial modes
# (--fr/--nfr/--changelog/--docs) refresh in place by design.
[[ "$MODE" == "full" ]] && rm -rf dist/website
mkdir -p dist/website/reference dist/website/modules dist/website/architecture
cp -R "$SCRIPT_DIR/chrome/." dist/website/assets/
cp "$SCRIPT_DIR/index.html" dist/website/index.html

# ── FR catalog ──────────────────────────────────────────────────────────────
if [[ "$MODE" == "full" || "$MODE" == "--fr" ]]; then
  echo "→ FR catalog"
  node tools/docs-site/data-extract.mjs
  node tools/docs-site/render-fr-catalog.mjs
fi

# ── NFR catalog ─────────────────────────────────────────────────────────────
if [[ "$MODE" == "full" || "$MODE" == "--nfr" ]]; then
  echo "→ NFR catalog"
  node tools/docs-site/nfr-extract.mjs
  node tools/docs-site/render-nfr-catalog.mjs
fi

# ── Changelog ──────────────────────────────────────────────────────────────
if [[ "$MODE" == "full" || "$MODE" == "--changelog" ]]; then
  echo "→ Changelog (consolidated)"
  node tools/docs-site/render-changelog.mjs
  echo "→ Changelog (per-module)"
  node tools/docs-site/render-module-changelog.mjs
fi

echo "✓ build complete"

# ── Doctrine pages from the markdown single source of truth (FR-DOCS-002) ───
if [[ "$MODE" == "full" || "$MODE" == "--docs" ]]; then
  echo "→ Docs pages (markdown SSoT)"
  node tools/docs-site/render-fr-pages.mjs   # FR-DOCS-005: per-FR CDS pages
  node tools/docs-site/render-roadmap.mjs   # FR-DOCS-003: roadmap page (before nav generation)
  node tools/docs-site/render-docs.mjs
fi
