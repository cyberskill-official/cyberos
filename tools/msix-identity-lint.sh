#!/usr/bin/env bash
# msix-identity-lint.sh - FR-APP-004 AC #3: the AppxManifest placeholder is a second,
# independent gate behind MSSTORE_RELEASE. While MSSTORE_RELEASE is unset/false this
# lint always passes (local verification and ordinary CI must not fail on scaffolding);
# with MSSTORE_RELEASE=true it hard-fails while the Partner Center identity is still
# the placeholder - a Store submission can never ship CHANGEME identity values.
set -euo pipefail

MANIFEST="${1:-apps/desktop/src-tauri/AppxManifest.xml}"
PLACEHOLDER="CHANGEME-PENDING-PARTNER-CENTER-RESERVATION"

[ -f "$MANIFEST" ] || { echo "msix-identity-lint: manifest not found: $MANIFEST" >&2; exit 2; }

# Inert outside a real Store release attempt (AC #3 scope).
if [[ "${MSSTORE_RELEASE:-false}" != "true" ]]; then
  echo "msix-identity-lint: MSSTORE_RELEASE not true - placeholder state not enforced (ok)"
  exit 0
fi

if grep -q "$PLACEHOLDER" "$MANIFEST"; then
  echo "ERROR: MSSTORE_RELEASE=true but AppxManifest.xml Identity Name/Publisher" >&2
  echo "  still contain the placeholder \"$PLACEHOLDER\"." >&2
  echo "  Update Identity values from the Partner Center app identity reservation" >&2
  echo "  before enabling MSSTORE_RELEASE." >&2
  exit 1
fi

echo "msix-identity-lint: OK - identity values are non-placeholder."
exit 0
