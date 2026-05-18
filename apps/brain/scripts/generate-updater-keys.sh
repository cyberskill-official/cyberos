#!/usr/bin/env bash
# Generate the Tauri 2 updater signing keypair.
#
# Run ONCE per release line, then commit the PUBLIC key into tauri.conf.json
# (`plugins.updater.pubkey`) and keep the PRIVATE key in an offline vault
# (e.g. 1Password / Bitwarden / HSM). Releases sign their update artifacts
# with the private key; clients verify with the public key.
#
# Usage:
#   ./generate-updater-keys.sh ./out
#   # → ./out/tauri-updater.key       (private — DO NOT COMMIT)
#   # → ./out/tauri-updater.key.pub   (public — paste into tauri.conf.json)

set -euo pipefail
DEST="${1:-./out}"
mkdir -p "$DEST"

if ! command -v tauri >/dev/null 2>&1; then
  echo "[generate-updater-keys] installing @tauri-apps/cli via pnpm…"
  pnpm add -D @tauri-apps/cli@^2 >/dev/null
fi

pnpm tauri signer generate -w "$DEST/tauri-updater.key" -p ""

echo
echo "===================================================================="
echo "  Public key (paste into apps/brain/src-tauri/tauri.conf.json under"
echo "  plugins.updater.pubkey):"
echo "===================================================================="
cat "$DEST/tauri-updater.key.pub"
echo
echo "===================================================================="
echo "  Private key path: $DEST/tauri-updater.key"
echo "  PROTECT IT. Put it in your secrets manager. NEVER commit."
echo "===================================================================="
