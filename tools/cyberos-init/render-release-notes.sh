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
  echo "cyberos-init: wrote release notes → $out" >&2
else
  cat "$tmp/out.md"
fi
