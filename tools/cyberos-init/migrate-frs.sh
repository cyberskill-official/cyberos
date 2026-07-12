#!/usr/bin/env bash
# migrate-frs.sh - bring ANY repo's docs/feature-requests to CyberOS 1.0 rules (FR-DOCS-004 lineage):
#   1. frontmatter repaired to strict YAML (minimal quoting, semantics untouched)
#   2. folder-per-FR layout: <module>/<STEM>/{spec.md, audit.md, assets/-on-demand}
#   3. a self-contained CDS status page generated at .cyberos/status.html
# Idempotent; run it from anywhere inside the target repo (or pass the root as $1).
# Ships in the payload; lives at .cyberos/migrate-frs.sh after /init.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="${1:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"

[ -d "$root/docs/feature-requests" ] || { echo "migrate-frs: no docs/feature-requests under $root - run init.sh first"; exit 2; }
command -v python3 >/dev/null || { echo "migrate-frs: python3 required"; exit 2; }

echo "== 1/3 frontmatter repair (strict YAML, formatting-only) =="
python3 "$here/docs-tools/repair_fr_yaml.py" --root "$root" || true

echo "== 2/3 folder-per-FR layout =="
python3 "$here/docs-tools/migrate_fr_layout.py" --root "$root"

echo "== 3/3 status page (.cyberos/status.html) =="
if command -v node >/dev/null 2>&1; then
  mkdir -p "$root/.cyberos/status-site"
  CYBEROS_HUB_LENIENT=1 CYBEROS_TEMPLATES="$here/docs-tools/templates" \
    node "$here/docs-tools/render-status-hub.mjs" "$root" "$root/.cyberos/status-site"
  cp "$root/.cyberos/status-site/reference/status.html" "$root/.cyberos/status.html"
  echo "migrate-frs: open $root/.cyberos/status.html (regenerate by re-running this script)"
else
  echo "migrate-frs: WARN node not found - skipped the status page (steps 1-2 done)"
fi
echo "migrate-frs: done. FR frontmatter is the record of truth; commit the moved specs."
