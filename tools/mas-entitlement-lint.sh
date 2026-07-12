#!/usr/bin/env bash
# mas-entitlement-lint.sh - FR-APP-003 AC #2: entitlements are minimal-by-audit.
#
# Fails (exit 1) if Entitlements.mas.plist declares any entitlement key that has no
# justifying row in the "Sandbox surface audit" table of the submission answer sheet
# (docs/deploy/mac-app-store-submission.md). Keeps the MAS entitlement set an
# allowlist grounded in the audited source surface, never a speculative grab-bag -
# Apple review challenges scope-creep, and this lint makes creep a build failure.
#
# Usage: tools/mas-entitlement-lint.sh [plist] [answer-sheet]
# Defaults match the repo layout. Dependency-free: POSIX shell + grep/sed only
# (no plutil, so it runs identically on ubuntu CI and macOS laptops).
set -euo pipefail

plist="${1:-apps/desktop/src-tauri/Entitlements.mas.plist}"
sheet="${2:-docs/deploy/mac-app-store-submission.md}"

[ -f "$plist" ] || { echo "mas-entitlement-lint: plist not found: $plist" >&2; exit 2; }
[ -f "$sheet" ] || { echo "mas-entitlement-lint: answer sheet not found: $sheet" >&2; exit 2; }

# Extract every <key>com.apple.security.*</key> from the plist.
keys="$(sed -n 's/.*<key>\(com\.apple\.security\.[a-zA-Z0-9._-]*\)<\/key>.*/\1/p' "$plist")"
[ -n "$keys" ] || { echo "mas-entitlement-lint: no entitlement keys found in $plist (malformed?)" >&2; exit 2; }

# The audit table lives under the "Sandbox surface audit" heading; scope the search
# to that section so a key mentioned in unrelated prose does not count as justified.
section="$(awk '/^#+ .*Sandbox surface audit/{flag=1; next} /^#+ /{flag=0} flag' "$sheet")"
[ -n "$section" ] || { echo "mas-entitlement-lint: no 'Sandbox surface audit' section in $sheet" >&2; exit 2; }

fail=0
for key in $keys; do
  if ! printf '%s\n' "$section" | grep -qF "$key"; then
    echo "ERROR: $plist declares \"$key\""
    echo "  but no row in $sheet \"Sandbox surface audit\" justifies it."
    echo "  Add a row or remove the entitlement."
    fail=1
  fi
done

if [ "$fail" -eq 0 ]; then
  count="$(printf '%s\n' "$keys" | wc -l | tr -d ' ')"
  echo "mas-entitlement-lint: OK - all $count entitlement keys justified in the audit table."
fi
exit "$fail"
