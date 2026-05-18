#!/usr/bin/env bash
# Sign the Windows MSI/EXE produced by `pnpm tauri build` with an EV
# code-signing certificate.
#
# Required:
#   WIN_SIGN_CERT_PATH   — path to the .pfx file (EV cert exported from HSM/USB)
#   WIN_SIGN_CERT_PASS   — password protecting the .pfx
#   WIN_SIGN_TS_URL      — RFC 3161 timestamp server (default: http://timestamp.digicert.com)
#
# Inputs:
#   $1   path to the .msi produced under apps/brain/src-tauri/target/release/bundle/msi/
#
# What it does:
#   * signs the .msi with signtool (SHA-256 + RFC 3161 timestamp)
#   * verifies the signature with `signtool verify /pa`
#
# Note: EV certs are usually attached to a YubiKey or eToken physical device;
# on hosted CI you'll typically use Azure Code Signing or sign the artifact
# in a separate dedicated environment. This script is the local-dev path.

set -euo pipefail

MSI="${1:?usage: sign-windows.sh path/to/CyberOS-BRAIN.msi}"
[[ -f "$MSI" ]] || { echo "MSI not found: $MSI" >&2; exit 1; }
: "${WIN_SIGN_CERT_PATH:?WIN_SIGN_CERT_PATH must be set}"
: "${WIN_SIGN_CERT_PASS:?WIN_SIGN_CERT_PASS must be set}"
TS_URL="${WIN_SIGN_TS_URL:-http://timestamp.digicert.com}"

if ! command -v signtool.exe >/dev/null 2>&1 && ! command -v signtool >/dev/null 2>&1; then
  echo "signtool not on PATH. Install Windows 10/11 SDK and add to PATH." >&2
  exit 1
fi
SIGNTOOL="$(command -v signtool.exe || command -v signtool)"

echo "[win-sign] signing $MSI"
"$SIGNTOOL" sign \
  /fd SHA256 \
  /tr "$TS_URL" \
  /td SHA256 \
  /f "$WIN_SIGN_CERT_PATH" \
  /p "$WIN_SIGN_CERT_PASS" \
  "$MSI"

echo "[win-sign] verifying"
"$SIGNTOOL" verify /pa "$MSI"

echo "[win-sign] done — $MSI is signed."
