#!/usr/bin/env bash
# status.sh — manual report of what is installed (version, rules fingerprint, pointers).
# Not auto-run. Soft update-check still fires so you hear about newer CyberOS.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
if [ -f "$here/lib/update-check.sh" ]; then
  # shellcheck source=/dev/null
  source "$here/lib/update-check.sh"
  _cyberos_update_check || true
fi
m="$here/manifest.yaml"
[ -f "$m" ] || { echo "cyberos: no manifest.yaml beside this script (run from a payload or installed .cyberos/)" >&2; exit 2; }
ver="$(grep -E '^cyberos_version:' "$m" | awk '{print $2}')"
rsha="$(grep -E '^rules_sha:' "$m" | awk '{print $2}')"
built="$(grep -E '^built_at:' "$m" | awk '{print $2}')"
echo "CyberOS $ver (built $built)"
echo "rules_sha: ${rsha:-<pre-1.0.0 payload>}   # compare across installs to detect rule drift"
if [ -f "$here/VERSION" ]; then echo "VERSION:   $(tr -d ' \n\r' < "$here/VERSION")"; fi
if [ -f "$here/GUIDE.md" ]; then echo "guide:     $here/GUIDE.md"; fi
echo "changelog: https://github.com/cyberskill-official/cyberos/blob/main/CHANGELOG.md"
echo "docs:      https://cyberos.cyberskill.world/docs"
