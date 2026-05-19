#!/usr/bin/env bash
# FR-AUTH-106 — installs MaxMind GeoLite2-City + GeoIP2-Anonymous-IP
# into a vendored location the AUTH service reads at boot.
#
# Usage:
#   MAXMIND_LICENSE_KEY=xxxx ./install-geoip.sh
#
# Or, in CI (downloads cached from internal mirror):
#   GEOIP_MIRROR_URL=https://internal.cdn/geoip ./install-geoip.sh
#
# Writes:
#   $GEOIP_DEST/GeoLite2-City.mmdb        (kind-2 + kind-3 detectors)
#   $GEOIP_DEST/GeoIP2-Anonymous-IP.mmdb  (VPN/Tor flagging — Anonymous-IP DB)
#
# After install, set in env:
#   AUTH_GEOIP_DB=$GEOIP_DEST/GeoLite2-City.mmdb
#   AUTH_GEOIP_ANONYMOUS_DB=$GEOIP_DEST/GeoIP2-Anonymous-IP.mmdb
#
# CI variant: pin AUTH_GEOIP_REQUIRED=1 in production so the service refuses
# to start if either DB is missing (catches a forgotten install).

set -euo pipefail

DEST="${GEOIP_DEST:-/opt/cyberos/geoip}"
mkdir -p "$DEST"

CITY_EDITION="GeoLite2-City"
ANON_EDITION="GeoIP2-Anonymous-IP"

# --- Mirror path (preferred in CI to avoid burning MaxMind API quota) -------
if [[ -n "${GEOIP_MIRROR_URL:-}" ]]; then
  echo "[install-geoip] using mirror: $GEOIP_MIRROR_URL"
  for ed in "$CITY_EDITION" "$ANON_EDITION"; do
    curl -fsSL "$GEOIP_MIRROR_URL/${ed}.mmdb" -o "$DEST/${ed}.mmdb"
    echo "[install-geoip] installed $DEST/${ed}.mmdb"
  done
  exit 0
fi

# --- MaxMind direct download (license key required) -------------------------
if [[ -z "${MAXMIND_LICENSE_KEY:-}" ]]; then
  echo "[install-geoip] ERROR: set MAXMIND_LICENSE_KEY or GEOIP_MIRROR_URL" >&2
  exit 1
fi

download_edition() {
  local ed="$1"
  local url="https://download.maxmind.com/app/geoip_download"
  local tmp; tmp="$(mktemp -d)"
  local tarball="${tmp}/${ed}.tar.gz"
  echo "[install-geoip] downloading $ed"
  curl -fsSL \
    "$url?edition_id=${ed}&license_key=${MAXMIND_LICENSE_KEY}&suffix=tar.gz" \
    -o "$tarball"
  tar -C "$tmp" -xzf "$tarball"
  # MaxMind tarballs unpack to a versioned dir; find the .mmdb file
  local mmdb; mmdb="$(find "$tmp" -name "${ed}.mmdb" -print -quit)"
  if [[ -z "$mmdb" ]]; then
    echo "[install-geoip] ERROR: ${ed}.mmdb not found in tarball" >&2
    rm -rf "$tmp"; exit 1
  fi
  install -m 0644 "$mmdb" "$DEST/${ed}.mmdb"
  rm -rf "$tmp"
  echo "[install-geoip] installed $DEST/${ed}.mmdb"
}

download_edition "$CITY_EDITION"
download_edition "$ANON_EDITION"

cat <<EOF
[install-geoip] done. Set these env vars in your AUTH service:
  AUTH_GEOIP_DB=$DEST/${CITY_EDITION}.mmdb
  AUTH_GEOIP_ANONYMOUS_DB=$DEST/${ANON_EDITION}.mmdb
  AUTH_GEOIP_REQUIRED=1     # production guardrail
EOF
