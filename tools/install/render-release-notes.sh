#!/usr/bin/env bash
# render-release-notes.sh — build GitHub release body from release-notes.md + CHANGELOG.md.
# usage: render-release-notes.sh [version] [out-file]
#   version default: contents of repo VERSION
#   out-file default: stdout
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
ver="${1:-$(tr -d ' \n\r' < "$repo/VERSION")}"
out="${2:-}"
tpl="$here/release-notes.md"
cl="$repo/CHANGELOG.md"
[ -f "$tpl" ] || { echo "missing $tpl" >&2; exit 2; }
[ -f "$cl" ] || { echo "missing $cl" >&2; exit 2; }

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

# FIRST RELEASE: the changelog block on a release page means "what changed since the
# previous release". When no previous PUBLISHED release exists, there is nothing to diff
# against and splicing the version's CHANGELOG section renders the whole pre-release
# history as if it were a delta — misleading on the very first release page. Detection is
# against published releases (gh), not git tags: pre-1.0 tags exist here (v0.1.0..v0.4.0)
# that never had a release, and "previous release" is what a release page reader means.
# Fail-open: when gh is absent or unauthenticated (local render, forks), keep the old
# behaviour and splice the section — wrong only in the rare first-release case, and never
# silently hides a changelog on release N>1.
is_first_release=0
if command -v gh >/dev/null 2>&1; then
  if rel_tags="$(gh release list --limit 100 --json tagName --jq '.[].tagName' 2>/dev/null)"; then
    prev="$(printf '%s\n' "$rel_tags" | grep -vx "v$ver" | grep -v '^[[:space:]]*$' | head -1 || true)"
    [ -z "$prev" ] && is_first_release=1
  fi
fi

if [ "$is_first_release" = 1 ]; then
  printf '%s\n' "_First release — no previous release to compare against. The full road to $ver is recorded in the CHANGELOG below._" > "$tmp/section.md"
else
  # Extract ## [ver] section body (until next ## [ or EOF)
  awk -v v="$ver" '
    BEGIN { p=0 }
    $0 ~ "^## \\[" v "\\]" { p=1; next }
    p && /^## \[/ { exit }
    p { print }
  ' "$cl" > "$tmp/section.md"

  if [ ! -s "$tmp/section.md" ] || [ -z "$(tr -d '[:space:]' < "$tmp/section.md")" ]; then
    printf '%s\n' "_(No CHANGELOG.md section for $ver — add ## [$ver] - YYYY-MM-DD before releasing.)_" > "$tmp/section.md"
  fi
fi

# Substitute version tokens, then splice changelog section at the marker
sed -e "s/{{VERSION}}/$ver/g" "$tpl" > "$tmp/body.md"
# Replace the single-line marker with the multi-line section file
{
  while IFS= read -r line || [ -n "$line" ]; do
    if [ "$line" = "{{CHANGELOG_SECTION}}" ]; then
      cat "$tmp/section.md"
    else
      printf '%s\n' "$line"
    fi
  done < "$tmp/body.md"
} > "$tmp/out.md"

if [ -n "$out" ]; then
  cp "$tmp/out.md" "$out"
  echo "cyberos: wrote release notes → $out" >&2
else
  cat "$tmp/out.md"
fi
