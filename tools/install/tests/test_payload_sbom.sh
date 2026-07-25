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
  # Enforce SHA pins for the first-party Actions this batch hardens.
  # Pre-existing third-party tags (tauri-action@v0, etc.) are out of IMP-043 scope.
  local bad=0 f line
  for f in "$repo/.github/workflows/suite-gate.yml" "$repo/.github/workflows/payload-gate.yml" "$REL"; do
    while IFS= read -r line; do
      if ! printf '%s' "$line" | grep -Eq 'uses:[[:space:]]*actions/(checkout|setup-node|setup-java|upload-artifact)@[0-9a-f]{40}'; then
        bad=1
        echo "  note  unpinned actions/*: $line"
      fi
    done < <(grep -E 'uses:[[:space:]]*actions/(checkout|setup-node|setup-java|upload-artifact)@' "$f" || true)
  done
  [ "$bad" -eq 0 ] && ok t_actions_sha_pinned || fail t_actions_sha_pinned "pins"
}
t_emit_script_present() { [ -f "$EMIT" ] && chmod +x "$EMIT" && ok t_emit_script_present || fail t_emit_script_present "missing"; }
t_release_uploads_sbom() {
  # Scope to the payload job: between "payload:" job header and the next top-level job key.
  local block
  block="$(awk '
    /^  payload:/ {grab=1}
    grab && /^  [a-zA-Z0-9_-]+:/ && !/^  payload:/ {exit}
    grab {print}
  ' "$REL")"
  printf '%s' "$block" | grep -q 'emit-payload-sbom' \
    && printf '%s' "$block" | grep -q 'cdx.json' \
    && printf '%s' "$block" | grep -Eq 'gh[[:space:]]+release[[:space:]]+upload' \
    && ok t_release_uploads_sbom || fail t_release_uploads_sbom "missing in payload job"
}
t_posture_doc() {
  [ -f "$DOC" ] && grep -qi 'SHA256SUMS' "$DOC" && grep -qi 'cosign' "$DOC" && ok t_posture_doc || fail t_posture_doc "doc"
}
t_sbom_dry_run() {
  local tmp out log
  tmp="$(mktemp -d "${TMPDIR:-/tmp}/sbom-fix.XXXXXX")"
  out="$tmp/out.cdx.json"
  log="$(mktemp "${TMPDIR:-/tmp}/sbom-emit.XXXXXX")"
  mkdir -p "$tmp/payload"; echo "1.2.3" > "$tmp/payload/VERSION"; echo "hello" > "$tmp/payload/hello.txt"
  if ! CYBEROS_SBOM_BACKEND=hermetic bash "$EMIT" "$tmp/payload" "$out" >"$log" 2>&1; then
    fail t_sbom_dry_run "$(cat "$log")"; rm -rf "$tmp"; rm -f "$log"; return
  fi
  grep -q '"bomFormat": "CycloneDX"' "$out" \
    && grep -q '"components"' "$out" \
    && grep -q 'hello.txt' "$out" \
    && grep -q '"alg": "SHA-256"' "$out" \
    && grep -Eq '"content": "[0-9a-f]{64}"' "$out" \
    && ok t_sbom_dry_run || fail t_sbom_dry_run "shape"
  rm -rf "$tmp"; rm -f "$log"
}
t_sbom_empty_payload_fails() {
  local tmp out log
  tmp="$(mktemp -d "${TMPDIR:-/tmp}/sbom-empty.XXXXXX")"
  out="$tmp/out.cdx.json"
  log="$(mktemp "${TMPDIR:-/tmp}/sbom-empty-log.XXXXXX")"
  mkdir -p "$tmp/payload"
  if CYBEROS_SBOM_BACKEND=hermetic bash "$EMIT" "$tmp/payload" "$out" >"$log" 2>&1; then
    fail t_sbom_empty_payload_fails "empty payload accepted"
  else
    ok t_sbom_empty_payload_fails
  fi
  rm -rf "$tmp"; rm -f "$log"
}
t_actions_sha_pinned; t_emit_script_present; t_release_uploads_sbom; t_posture_doc
t_sbom_dry_run; t_sbom_empty_payload_fails
echo "----"; echo "pass=$PASS fail=$FAIL"; [ "$FAIL" -eq 0 ]
