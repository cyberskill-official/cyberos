#!/usr/bin/env bash
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }
EMIT="$repo/tools/install/emit-payload-sbom.sh"
DOC="$repo/docs/deploy/PAYLOAD-SUPPLY-CHAIN.md"
REL="$repo/.github/workflows/release.yml"
t_actions_sha_pinned() {
  local bad=0
  for f in "$repo/.github/workflows/suite-gate.yml" "$repo/.github/workflows/payload-gate.yml" "$REL"; do
    while IFS= read -r line; do
      case "$line" in
        *actions/checkout@v*|*actions/setup-node@v*|*actions/setup-java@v*|*actions/upload-artifact@v*) bad=1; echo "  note  floating: $line";;
      esac
    done < <(grep -E 'uses:[[:space:]]*actions/(checkout|setup-node|setup-java|upload-artifact)@' "$f" || true)
    grep -E 'actions/checkout@[0-9a-f]{40}' "$f" >/dev/null || bad=1
  done
  grep -E 'actions/setup-node@[0-9a-f]{40}' "$REL" >/dev/null || bad=1
  grep -E 'actions/upload-artifact@[0-9a-f]{40}' "$REL" >/dev/null || bad=1
  grep -E 'actions/setup-java@[0-9a-f]{40}' "$REL" >/dev/null || bad=1
  [ "$bad" -eq 0 ] && ok t_actions_sha_pinned || fail t_actions_sha_pinned "pins"
}
t_emit_script_present() { [ -f "$EMIT" ] && chmod +x "$EMIT" && ok t_emit_script_present || fail t_emit_script_present "missing"; }
t_release_uploads_sbom() {
  grep -q 'emit-payload-sbom' "$REL" && grep -q 'cdx.json' "$REL" && ok t_release_uploads_sbom || fail t_release_uploads_sbom "missing"
}
t_posture_doc() {
  [ -f "$DOC" ] && grep -qi 'SHA256SUMS' "$DOC" && grep -qi 'cosign' "$DOC" && ok t_posture_doc || fail t_posture_doc "doc"
}
t_sbom_dry_run() {
  local tmp out; tmp="$(mktemp -d "${TMPDIR:-/tmp}/sbom-fix.XXXXXX")"; out="$tmp/out.cdx.json"
  mkdir -p "$tmp/payload"; echo "1.2.3" > "$tmp/payload/VERSION"; echo "hello" > "$tmp/payload/hello.txt"
  if ! bash "$EMIT" "$tmp/payload" "$out" >/tmp/sbom-emit.$$.log 2>&1; then fail t_sbom_dry_run "$(cat /tmp/sbom-emit.$$.log)"; rm -rf "$tmp"; return; fi
  grep -q '"bomFormat": "CycloneDX"' "$out" && grep -q '"components"' "$out" && grep -q 'hello.txt' "$out" \
    && ok t_sbom_dry_run || fail t_sbom_dry_run "shape"
  rm -rf "$tmp"
}
t_actions_sha_pinned; t_emit_script_present; t_release_uploads_sbom; t_posture_doc; t_sbom_dry_run
echo "----"; echo "pass=$PASS fail=$FAIL"; [ "$FAIL" -eq 0 ]
