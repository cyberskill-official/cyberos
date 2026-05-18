#!/usr/bin/env bash
# Sign + notarise the macOS DMG produced by `pnpm tauri build`.
#
# Required env vars (typically loaded from a 1Password CLI session in CI):
#   APPLE_DEVELOPER_ID         — "Developer ID Application: CYBERSKILL ... (TEAMID)"
#   APPLE_NOTARIZE_APPLE_ID    — Apple ID with notarisation entitlement
#   APPLE_NOTARIZE_PASSWORD    — app-specific password for that Apple ID
#   APPLE_NOTARIZE_TEAM_ID     — Apple Developer Team ID (10 chars)
#
# Inputs:
#   $1   path to the .dmg produced under apps/brain/src-tauri/target/release/bundle/dmg/
#
# What it does:
#   1. codesign the inner .app with the Developer ID + hardened runtime + entitlements.
#   2. codesign the outer .dmg.
#   3. submit to Apple notarisation (xcrun notarytool submit ... --wait).
#   4. staple the notarisation ticket onto the .dmg.
#   5. spctl --assess to verify Gatekeeper accepts it.

set -euo pipefail

DMG="${1:?usage: sign-and-notarize-macos.sh path/to/CyberOS-BRAIN.dmg}"
[[ -f "$DMG" ]] || { echo "DMG not found: $DMG" >&2; exit 1; }
: "${APPLE_DEVELOPER_ID:?APPLE_DEVELOPER_ID must be set}"
: "${APPLE_NOTARIZE_APPLE_ID:?APPLE_NOTARIZE_APPLE_ID must be set}"
: "${APPLE_NOTARIZE_PASSWORD:?APPLE_NOTARIZE_PASSWORD must be set}"
: "${APPLE_NOTARIZE_TEAM_ID:?APPLE_NOTARIZE_TEAM_ID must be set}"

# 1. Sign the embedded .app bundle.
ENTITLEMENTS="$(dirname "$0")/../src-tauri/entitlements.plist"
if [[ ! -f "$ENTITLEMENTS" ]]; then
  cat > "$ENTITLEMENTS" <<'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>com.apple.security.cs.allow-jit</key><true/>
  <key>com.apple.security.cs.allow-unsigned-executable-memory</key><true/>
  <key>com.apple.security.network.client</key><true/>
  <key>com.apple.security.files.user-selected.read-write</key><true/>
  <!-- Required so the supervisor can read ~/.cyberos-memory/ -->
  <key>com.apple.security.files.bookmarks.app-scope</key><true/>
</dict>
</plist>
EOF
fi

MOUNT_POINT="$(mktemp -d)/CyberOS-BRAIN"
hdiutil attach "$DMG" -mountpoint "$MOUNT_POINT" -nobrowse -quiet
APP_PATH="$(find "$MOUNT_POINT" -maxdepth 2 -name '*.app' -print -quit)"
[[ -d "$APP_PATH" ]] || { echo "no .app inside DMG" >&2; hdiutil detach "$MOUNT_POINT" -quiet; exit 1; }

echo "[macos-sign] codesigning $APP_PATH"
codesign --force --options runtime --timestamp \
  --entitlements "$ENTITLEMENTS" \
  --sign "$APPLE_DEVELOPER_ID" \
  --deep "$APP_PATH"
hdiutil detach "$MOUNT_POINT" -quiet

# 2. Sign the outer DMG.
echo "[macos-sign] codesigning DMG"
codesign --force --sign "$APPLE_DEVELOPER_ID" --timestamp "$DMG"

# 3. Notarise.
echo "[macos-sign] submitting for notarisation (this can take 5-10 min)…"
xcrun notarytool submit "$DMG" \
  --apple-id "$APPLE_NOTARIZE_APPLE_ID" \
  --password "$APPLE_NOTARIZE_PASSWORD" \
  --team-id "$APPLE_NOTARIZE_TEAM_ID" \
  --wait

# 4. Staple the ticket onto the DMG.
echo "[macos-sign] stapling"
xcrun stapler staple "$DMG"

# 5. Verify.
echo "[macos-sign] verifying with spctl"
spctl --assess --type install -v "$DMG"

echo "[macos-sign] done — $DMG is signed, notarised, and stapled."
