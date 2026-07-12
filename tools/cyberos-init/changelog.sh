#!/usr/bin/env bash
# changelog.sh - root CLI entry (FR-IMP-076): show what is installed, directly from the shell.
# Mirrors the plugin's /cyberos:changelog. Reads the manifest beside this script.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
m="$here/manifest.yaml"
[ -f "$m" ] || { echo "cyberos: no manifest.yaml beside this script (run from a payload or installed .cyberos/)" >&2; exit 2; }
ver="$(grep -E '^cyberos_version:' "$m" | awk '{print $2}')"
rsha="$(grep -E '^rules_sha:' "$m" | awk '{print $2}')"
built="$(grep -E '^built_at:' "$m" | awk '{print $2}')"
echo "CyberOS $ver (built $built)"
echo "rules_sha: ${rsha:-<pre-FR-IMP-074 payload>}   # compare across installs to detect rule drift"
if [ -f "$here/GUIDE.md" ]; then echo "guide:     $here/GUIDE.md"; fi
echo "changelog: https://github.com/cyberskill-official/cyberos/blob/main/CHANGELOG.md"
