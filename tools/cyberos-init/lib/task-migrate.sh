#!/usr/bin/env bash
# fr-migrate.sh — FR layout + status page (sourced by init.sh; not a standalone entry).
# Combined into init so one entry point owns vendoring + migrate + page regen.
# shellcheck shell=bash
#
# Requires (set by caller):
#   root  — absolute target repo root
#   kit   — absolute dir with docs-tools/ (payload or .cyberos)
# Optional:
#   PAGE_ONLY=1  — status page only (step 3)

_cyberos_fr_migrate() {
  local PAGE_ONLY="${PAGE_ONLY:-0}"
  local root="${1:?root required}"
  local kit="${2:?kit dir with docs-tools required}"

  root="$(cd "$root" 2>/dev/null && pwd)" || { echo "cyberos migrate: no such root: $1" >&2; return 2; }
  kit="$(cd "$kit" 2>/dev/null && pwd)" || { echo "cyberos migrate: no kit at $2" >&2; return 2; }

  if [ ! -d "$root/docs/tasks" ]; then
    mkdir -p "$root/docs/tasks"
  fi
  command -v python3 >/dev/null || { echo "cyberos migrate: python3 required" >&2; return 2; }

  if [ "$PAGE_ONLY" = 0 ]; then
    echo "== migrate 0/4 adopt stray inputs =="
    if [ -f "$root/docs/BACKLOG.md" ] && [ ! -f "$root/docs/tasks/BACKLOG.md" ]; then
      mv "$root/docs/BACKLOG.md" "$root/docs/tasks/BACKLOG.md"
      echo "cyberos migrate: adopted docs/BACKLOG.md -> docs/tasks/BACKLOG.md"
    fi
    if [ -f "$root/docs/CHANGELOG.md" ] && [ ! -f "$root/CHANGELOG.md" ]; then
      mv "$root/docs/CHANGELOG.md" "$root/CHANGELOG.md"
      echo "cyberos migrate: adopted docs/CHANGELOG.md -> CHANGELOG.md"
    fi

    echo "== migrate 1/4 frontmatter repair =="
    if [ -f "$kit/docs-tools/repair_fr_yaml.py" ]; then
      python3 "$kit/docs-tools/repair_fr_yaml.py" --root "$root" || true
    fi

    echo "== migrate 2/4 folder-per-FR layout =="
    if [ -f "$kit/docs-tools/migrate_fr_layout.py" ]; then
      python3 "$kit/docs-tools/migrate_fr_layout.py" --root "$root"
    else
      echo "cyberos migrate: WARN migrate_fr_layout.py missing — skip layout"
    fi
  fi

  echo "== migrate 3/4 status page (docs/status/) =="
  local page_done=0
  # Cleanup orphaned pre-folder / intermediate paths every run
  rm -f "$root/.cyberos/status.html" "$root/docs/status.html" 2>/dev/null || true
  rm -rf "$root/.cyberos/status-site" 2>/dev/null || true

  if command -v node >/dev/null 2>&1 && [ -f "$kit/docs-tools/render-status-hub.mjs" ]; then
    mkdir -p "$root/.cyberos/status-site" "$root/docs/status"
    CYBEROS_HUB_LENIENT=1 CYBEROS_PAGE_ASSETS=1 CYBEROS_PROJECT="$(basename "$root")" \
      CYBEROS_FR_BASE="../tasks/" \
      CYBEROS_STATUS_SPECS="${CYBEROS_STATUS_SPECS:-1}" \
      CYBEROS_TEMPLATES="$kit/docs-tools/templates" \
      node "$kit/docs-tools/render-status-hub.mjs" "$root" "$root/.cyberos/status-site"
    # rebuild published folder from scratch (no stale chunks)
    rm -rf "$root/docs/status/assets" "$root/docs/status/data"
    mkdir -p "$root/docs/status/assets"
    if [ -f "$root/.cyberos/status-site/reference/status.html" ]; then
      cp "$root/.cyberos/status-site/reference/status.html" "$root/docs/status/index.html"
      cp -R "$root/.cyberos/status-site/reference/assets/." "$root/docs/status/assets/" 2>/dev/null || true
      if [ -d "$root/.cyberos/status-site/reference/data" ]; then
        cp -R "$root/.cyberos/status-site/reference/data" "$root/docs/status/data"
      fi
      page_done=1
      echo "cyberos migrate: open $root/docs/status/index.html"
    else
      echo "cyberos migrate: WARN status renderer produced no reference/status.html"
    fi
    # Drop intermediate render tree (not for commit)
    rm -rf "$root/.cyberos/status-site" 2>/dev/null || true
  else
    echo "cyberos migrate: WARN node or render-status-hub.mjs missing — skipped status page"
  fi

  if [ "$PAGE_ONLY" = 0 ] && [ "$page_done" = 1 ]; then
    for f in ROADMAP.md BACKLOG.md CHANGELOG.md; do
      if [ -f "$root/docs/$f" ]; then
        rm -f "$root/docs/$f"
        echo "cyberos migrate: removed docs/$f (replaced by docs/status/)"
      fi
    done
  fi

  # Orphan cleanup under docs/status (empty junk, legacy)
  if [ -d "$root/docs/status" ]; then
    find "$root/docs/status" -name '.DS_Store' -delete 2>/dev/null || true
  fi

  echo "== migrate 4/4 verify =="
  local flat flat_n nospec nospec_n page specs_n deep_n
  flat="$(find "$root/docs/tasks" -mindepth 1 -maxdepth 2 -type f -name 'FR-*.md' \
          -not -path '*/_*' -not -path '*/.*' 2>/dev/null | sort || true)"
  flat_n=0; [ -n "$flat" ] && flat_n="$(printf '%s\n' "$flat" | wc -l | tr -d ' ')"
  nospec="$(find "$root/docs/tasks" -mindepth 2 -maxdepth 2 -type d -name 'FR-*' \
          -not -path '*/_*' -not -path '*/.*' 2>/dev/null | sort | while IFS= read -r d; do
            [ -f "$d/spec.md" ] || echo "$d"; done || true)"
  nospec_n=0; [ -n "$nospec" ] && nospec_n="$(printf '%s\n' "$nospec" | wc -l | tr -d ' ')"
  page="absent"
  [ -f "$root/docs/status/index.html" ] && [ -f "$root/docs/status/assets/status.css" ] && page="present"
  specs_n="$(find "$root/docs/tasks" -mindepth 3 -maxdepth 3 -type f -name 'spec.md' \
          -not -path '*/_*' -not -path '*/.*' 2>/dev/null | wc -l | tr -d ' ')"
  deep_n="$(find "$root/docs/tasks" -mindepth 3 -type f -name 'FR-*.md' \
          -not -path '*/_*' -not -path '*/.*' 2>/dev/null | wc -l | tr -d ' ')"
  echo "cyberos-migrate verify: fr_specs=$specs_n flat_fr_files_remaining=$flat_n fr_folders_missing_spec=$nospec_n deep_fr_files=$deep_n status_page=$page"
  if [ "${deep_n:-0}" -gt 0 ] 2>/dev/null; then
    echo "cyberos migrate: note $deep_n FR-named .md below module depth (left untouched)"
  fi
  if [ "${flat_n:-0}" -gt 0 ] 2>/dev/null; then
    echo "cyberos migrate: WARN un-migrated flat FR files:"
    printf '%s\n' "$flat" | sed 's/^/  /'
  fi
  if [ "${nospec_n:-0}" -gt 0 ] 2>/dev/null; then
    echo "cyberos migrate: WARN FR folders without spec.md:"
    printf '%s\n' "$nospec" | sed 's/^/  /'
  fi
  echo "cyberos migrate: done. FR frontmatter is the record of truth."
  return 0
}
