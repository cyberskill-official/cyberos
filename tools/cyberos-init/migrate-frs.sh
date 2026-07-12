#!/usr/bin/env bash
# migrate-frs.sh - bring ANY repo's docs/feature-requests to CyberOS 1.0 rules (FR-DOCS-004 lineage):
#   0. adopt stray inputs: docs/BACKLOG.md -> docs/feature-requests/BACKLOG.md and
#      docs/CHANGELOG.md -> CHANGELOG.md (root), each only when the canonical home is empty
#   1. frontmatter repaired to strict YAML (minimal quoting, semantics untouched)
#   2. folder-per-FR layout: <module>/<STEM>/{spec.md, audit.md, assets/-on-demand}
#      (root-level flat FRs relocate into a module folder: frontmatter `module:` > id segment > misc)
#   3. the CDS status page (Roadmap | Backlog | Changelog tabs) at docs/status/ - a folder:
#      index.html + assets/ (status.css, favicon). Titled after THIS repo. Then REMOVE any
#      remaining docs/ROADMAP.md, docs/BACKLOG.md, docs/CHANGELOG.md - the page replaces them
#      outright (previous content stays in git history; inputs were adopted in step 0).
#   4. verify: machine-readable coverage line + WARNs for anything the migration could not place
# `--page` runs step 3 only - the fast path the pre-commit hook and run-gates.sh use to keep
# the page synced with the markdown it renders (FR frontmatter, CHANGELOG.md, VERSION).
# Idempotent; run it from anywhere inside the target repo (or pass the root as $1).
# Ships in the payload; lives at .cyberos/migrate-frs.sh after /init (init runs it automatically;
# skip that with CYBEROS_NO_MIGRATE=1).
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
PAGE_ONLY=0
if [ "${1:-}" = "--page" ]; then PAGE_ONLY=1; shift; fi
root="${1:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"

[ -d "$root/docs/feature-requests" ] || { echo "migrate-frs: no docs/feature-requests under $root - run init.sh first"; exit 2; }
command -v python3 >/dev/null || { echo "migrate-frs: python3 required"; exit 2; }

if [ "$PAGE_ONLY" = 0 ]; then
echo "== 0/4 adopt stray inputs =="
if [ -f "$root/docs/BACKLOG.md" ] && [ ! -f "$root/docs/feature-requests/BACKLOG.md" ]; then
  mv "$root/docs/BACKLOG.md" "$root/docs/feature-requests/BACKLOG.md"
  echo "migrate-frs: adopted docs/BACKLOG.md -> docs/feature-requests/BACKLOG.md (the FR index's canonical home)"
fi
if [ -f "$root/docs/CHANGELOG.md" ] && [ ! -f "$root/CHANGELOG.md" ]; then
  mv "$root/docs/CHANGELOG.md" "$root/CHANGELOG.md"
  echo "migrate-frs: adopted docs/CHANGELOG.md -> CHANGELOG.md (root; the status page reads it)"
fi

echo "== 1/4 frontmatter repair (strict YAML, formatting-only) =="
python3 "$here/docs-tools/repair_fr_yaml.py" --root "$root" || true

echo "== 2/4 folder-per-FR layout =="
python3 "$here/docs-tools/migrate_fr_layout.py" --root "$root"
fi

echo "== 3/4 status page (docs/status/) =="
page_done=0
if command -v node >/dev/null 2>&1; then
  mkdir -p "$root/.cyberos/status-site" "$root/docs/status/assets"
  CYBEROS_HUB_LENIENT=1 CYBEROS_PAGE_ASSETS=1 CYBEROS_PROJECT="$(basename "$root")" \
    CYBEROS_TEMPLATES="$here/docs-tools/templates" \
    node "$here/docs-tools/render-status-hub.mjs" "$root" "$root/.cyberos/status-site"
  cp "$root/.cyberos/status-site/reference/status.html" "$root/docs/status/index.html"
  cp -R "$root/.cyberos/status-site/reference/assets/." "$root/docs/status/assets/"
  rm -f "$root/.cyberos/status.html" "$root/docs/status.html"   # pre-folder locations
  page_done=1
  echo "migrate-frs: open $root/docs/status/index.html (tracked folder: index.html + assets/)"
else
  echo "migrate-frs: WARN node not found - skipped the status page"
fi

# the one page IS the roadmap, the backlog view, and the changelog view - remove the old
# standalone documents outright once the page rendered. Inputs were adopted in step 0
# (docs/BACKLOG.md, docs/CHANGELOG.md move to their canonical homes when those are empty);
# whatever remains here is duplicate or stale, and git history keeps it.
if [ "$PAGE_ONLY" = 0 ] && [ "$page_done" = 1 ]; then
  for f in ROADMAP.md BACKLOG.md CHANGELOG.md; do
    if [ -f "$root/docs/$f" ]; then
      rm -f "$root/docs/$f"
      echo "migrate-frs: removed docs/$f (replaced by docs/status/; previous content in git history)"
    fi
  done
fi

echo "== 4/4 verify =="
# Any file still NAMED FR-*.md at the FR root or module level is un-migrated (migrated
# files are spec.md/audit.md inside their FR folder). _*/.* trees stay out of scope.
flat="$(find "$root/docs/feature-requests" -mindepth 1 -maxdepth 2 -type f -name 'FR-*.md' \
        -not -path '*/_*' -not -path '*/.*' 2>/dev/null | sort)"
flat_n=0; [ -n "$flat" ] && flat_n="$(printf '%s\n' "$flat" | wc -l | tr -d ' ')"
nospec="$(find "$root/docs/feature-requests" -mindepth 2 -maxdepth 2 -type d -name 'FR-*' \
        -not -path '*/_*' -not -path '*/.*' 2>/dev/null | sort | while IFS= read -r d; do
          [ -f "$d/spec.md" ] || echo "$d"; done)"
nospec_n=0; [ -n "$nospec" ] && nospec_n="$(printf '%s\n' "$nospec" | wc -l | tr -d ' ')"
page="absent"; [ -f "$root/docs/status/index.html" ] && [ -f "$root/docs/status/assets/status.css" ] && page="present"
specs_n="$(find "$root/docs/feature-requests" -mindepth 3 -maxdepth 3 -type f -name 'spec.md' \
        -not -path '*/_*' -not -path '*/.*' 2>/dev/null | wc -l | tr -d ' ')"
# FR-named files BELOW module depth live in repo-specific trees (pipeline outputs, reports...);
# the protocol defines only root + <module> flat layouts, so these are reported, never moved.
deep_n="$(find "$root/docs/feature-requests" -mindepth 3 -type f -name 'FR-*.md' \
        -not -path '*/_*' -not -path '*/.*' 2>/dev/null | wc -l | tr -d ' ')"
echo "migrate-frs verify: fr_specs=$specs_n flat_fr_files_remaining=$flat_n fr_folders_missing_spec=$nospec_n deep_fr_files=$deep_n status_page=$page"
if [ "$deep_n" -gt 0 ]; then
  echo "migrate-frs: note $deep_n FR-named .md file(s) below module depth (repo-specific trees - out of protocol scope, left untouched)"
fi
if [ "$flat_n" -gt 0 ]; then
  echo "migrate-frs: WARN un-migrated flat FR files (fix or re-run):"
  printf '%s\n' "$flat" | sed 's/^/  /'
fi
if [ "$nospec_n" -gt 0 ]; then
  echo "migrate-frs: WARN FR folders without spec.md (empty/partial - fill or remove):"
  printf '%s\n' "$nospec" | sed 's/^/  /'
fi
echo "migrate-frs: done. FR frontmatter is the record of truth; commit the moved specs."
