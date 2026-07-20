#!/usr/bin/env bash
# Rebuild and publish latest.json for the Tauri auto-updater, covering EVERY platform
# on the release.
#
# WHY THIS EXISTS
# ---------------
# tauri-action generates a latest.json containing ONLY the platform of the matrix leg
# that produced it, and uploads it to the release with clobber semantics. In a 3-OS
# matrix (macos/windows/ubuntu) all three legs race to write the same asset, so the LAST
# leg to finish silently overwrites the other two.
#
# v1.0.0 shipped exactly this way: the Windows leg finished last, so the published
# latest.json listed only windows-x86_64 / -msi / -nsis. Every macOS and Linux install of
# 1.0.0 was stranded - the updater endpoint returned no entry for their platform, so they
# could never be offered 1.0.1 - and the run was green end to end while it happened.
#
# The fix is ordering, not retry: build the manifest ONCE, after the whole matrix, from the
# .sig files actually attached to the release. A partial manifest is a hard failure here,
# because a partial manifest is invisible in the app and permanent on the release.
#
# Required env:
#   TAG          - release tag (e.g. v1.0.0)
#   REPO         - owner/name on GitHub
#   GITHUB_TOKEN - release write scope (for `gh release`)
# Optional env:
#   VERSION      - bare version; defaults to TAG with a leading "v" stripped
set -euo pipefail

: "${TAG:?TAG required (e.g. v1.0.0)}"
: "${REPO:?REPO required (e.g. cyberskill-official/cyberos)}"
: "${GITHUB_TOKEN:?GITHUB_TOKEN required}"
VERSION="${VERSION:-${TAG#v}}"

WORKDIR="$(mktemp -d)"
trap 'rm -rf "$WORKDIR"' EXIT

echo "[updater] Fetching asset list for $REPO $TAG"
gh release view "$TAG" --repo "$REPO" --json assets --jq '.assets[].name' \
  > "$WORKDIR/asset-names.txt"

asset_url() {
  printf 'https://github.com/%s/releases/download/%s/%s\n' "$REPO" "$TAG" "$1"
}

# Echo the single asset matching an extended regex, or empty when none.
find_asset() {
  local pattern="$1" matches count
  matches=$(grep -E "$pattern" "$WORKDIR/asset-names.txt" || true)
  count=$(printf '%s\n' "$matches" | grep -c . || true)
  [ "$count" = "0" ] && return 0
  if [ "$count" -gt 1 ]; then
    echo "[updater] WARN: pattern '$pattern' matched $count assets:" >&2
    printf '  %s\n' "$matches" >&2
    echo "[updater] WARN: using the first match" >&2
  fi
  printf '%s\n' "$matches" | head -1
}

# A Tauri .sig is a base64 minisign detached signature; Tauri expects the file verbatim.
read_sig() {
  local name="$1" sig_name="${1}.sig" path
  if ! grep -Fxq "$sig_name" "$WORKDIR/asset-names.txt"; then
    echo "[updater] ERROR: signature asset '$sig_name' not on the release - did createUpdaterArtifacts run?" >&2
    return 1
  fi
  gh release download "$TAG" --repo "$REPO" --pattern "$sig_name" --dir "$WORKDIR" --clobber >&2
  path="$WORKDIR/$sig_name"
  [ -s "$path" ] || { echo "[updater] ERROR: downloaded sig is empty: $path" >&2; return 1; }
  cat "$path"
}

# Bundle resolution. macOS ships ONE universal bundle (arm64 + x86_64 lipo'd together),
# so all three darwin keys point at the same artifact and the same signature - correct for
# either arch, and it means the lookup succeeds whichever key the plugin resolves to.
# Linux self-update is AppImage-only: tauri replaces the running AppImage in place and has
# no deb/rpm path, so deb/rpm users update through apt/dnf and get no key here.
MAC_UNIVERSAL=$(find_asset '^CyberOS.*\.app\.tar\.gz$')
LIN_APPIMAGE=$(find_asset  '^CyberOS.*amd64\.AppImage$')
WIN_NSIS=$(find_asset      '^CyberOS.*x64-setup\.exe$')
WIN_MSI=$(find_asset       '^CyberOS.*x64.*\.msi$')

echo "[updater] Resolved updater bundles:"
echo "  macOS universal = ${MAC_UNIVERSAL:-<missing>}"
echo "  linux AppImage  = ${LIN_APPIMAGE:-<missing>}"
echo "  windows nsis    = ${WIN_NSIS:-<missing>}"
echo "  windows msi     = ${WIN_MSI:-<missing>}"

PUB_DATE=$(date -u +"%Y-%m-%dT%H:%M:%S.000Z")
MANIFEST="$WORKDIR/latest.json"
jq -n \
  --arg version "$VERSION" \
  --arg pub_date "$PUB_DATE" \
  --arg notes "See https://github.com/$REPO/releases/tag/$TAG" \
  '{version: $version, notes: $notes, pub_date: $pub_date, platforms: {}}' > "$MANIFEST"

add_platform() {
  local key="$1" name="$2" sig url
  [ -z "$name" ] && return 0
  sig=$(read_sig "$name")
  url=$(asset_url "$name")
  jq --arg key "$key" --arg sig "$sig" --arg url "$url" \
    '.platforms[$key] = {signature: $sig, url: $url}' "$MANIFEST" > "$MANIFEST.tmp"
  mv "$MANIFEST.tmp" "$MANIFEST"
  echo "[updater] + $key -> $name"
}

add_platform "darwin-universal"      "$MAC_UNIVERSAL"
add_platform "darwin-aarch64"        "$MAC_UNIVERSAL"
add_platform "darwin-x86_64"         "$MAC_UNIVERSAL"
add_platform "linux-x86_64"          "$LIN_APPIMAGE"
add_platform "linux-x86_64-appimage" "$LIN_APPIMAGE"
add_platform "windows-x86_64"        "$WIN_MSI"
add_platform "windows-x86_64-msi"    "$WIN_MSI"
add_platform "windows-x86_64-nsis"   "$WIN_NSIS"

# Hard gate. Every platform we publish an installer for MUST resolve, or we do not publish
# a manifest at all. Shipping a partial one is worse than shipping none: the release looks
# complete, the run stays green, and the gap only surfaces when the NEXT version fails to
# reach half the install base.
missing=$(jq -r '
  ["darwin-universal","darwin-aarch64","darwin-x86_64",
   "linux-x86_64","windows-x86_64","windows-x86_64-nsis"]
  - (.platforms | keys) | join(", ")
' "$MANIFEST")
if [ -n "$missing" ]; then
  echo "[updater] ERROR: missing required latest.json platform(s): $missing" >&2
  echo "[updater] Refusing to publish a partial updater manifest." >&2
  echo "[updater] Assets present on $TAG:" >&2
  sed 's/^/  /' "$WORKDIR/asset-names.txt" >&2
  exit 1
fi

echo "[updater] Final manifest:"
cat "$MANIFEST"

gh release upload "$TAG" "$MANIFEST" --repo "$REPO" --clobber
echo "[updater] Uploaded latest.json to $TAG ($(jq -r '.platforms | keys | length' "$MANIFEST") platforms)"
