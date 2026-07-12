#!/usr/bin/env bash
# release-assets.sh - produce the GitHub Release asset set from a built payload. FR-IMP-069.
# usage: release-assets.sh <payload-dir> <out-dir>
# exit 0  wrote 5 files (versioned + stable tarball, versioned + stable plugin, SHA256SUMS)
# exit 10 version disagreement (payload VERSION vs root VERSION vs $GITHUB_REF_NAME) - writes nothing
# exit 2  payload missing/incomplete or GNU tar unavailable (determinism is non-negotiable)
set -uo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
payload="${1:?usage: release-assets.sh <payload-dir> <out-dir>}"
out="${2:?usage: release-assets.sh <payload-dir> <out-dir>}"

err2()  { echo "cyberos-init: ERROR: $*" >&2; exit 2; }
err10() { echo "cyberos-init: ERROR: $*" >&2; exit 10; }

tar --version 2>/dev/null | grep -q "GNU tar" || err2 "GNU tar required for deterministic assets (run on ubuntu/CI)"
[ -d "$payload" ]                 || err2 "payload dir missing: $payload"
[ -f "$payload/VERSION" ]         || err2 "payload incomplete: no VERSION"
[ -f "$payload/cyberos.plugin" ]  || err2 "payload incomplete: no cyberos.plugin"

ver="$(tr -d ' \n\r' < "$payload/VERSION")"
root_ver="$(tr -d ' \n\r' < "$repo/VERSION" 2>/dev/null || echo MISSING)"
[ "$ver" = "$root_ver" ] || err10 "payload VERSION ($ver) != root VERSION ($root_ver)"
# Tag agreement: TAG (set by the release workflow for both tag-push AND workflow_dispatch,
# where GITHUB_REF_NAME is the branch, not the tag) takes precedence over GITHUB_REF_NAME.
ref="${TAG:-${GITHUB_REF_NAME:-}}"
if [ -n "$ref" ]; then
  [ "v$ver" = "$ref" ] || err10 "tag $ref != v$ver"
fi

mkdir -p "$out"
tar --sort=name --owner=0 --group=0 --numeric-owner --mtime='2000-01-01 00:00:00Z' \
    -cf - -C "$payload" . | gzip -n > "$out/cyberos-payload-$ver.tar.gz"
cp "$out/cyberos-payload-$ver.tar.gz" "$out/cyberos-payload.tar.gz"    # stable alias for latest/download
cp "$payload/cyberos.plugin" "$out/cyberos-$ver.plugin"
cp "$payload/cyberos.plugin" "$out/cyberos.plugin"                     # stable alias
(cd "$out" && sha256sum "cyberos-payload-$ver.tar.gz" cyberos-payload.tar.gz "cyberos-$ver.plugin" cyberos.plugin > SHA256SUMS)
echo "cyberos-init: release assets ready in $out (version $ver, 5 files)"
