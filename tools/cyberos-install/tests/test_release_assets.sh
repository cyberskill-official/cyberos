#!/usr/bin/env bash
# test_release_assets.sh - TASK-IMP-069 §5 suite (t01-t09 -> AC 1-9). file:// fixtures, no network.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
RA="$repo/tools/cyberos-install/release-assets.sh"
BOOT="$repo/tools/cyberos-install/bootstrap.sh"
ROLL="$repo/tools/cyberos-install/rollout.sh"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }
gnu_tar() { tar --version 2>/dev/null | grep -q "GNU tar"; }

# SKIP the whole suite without GNU tar, BEFORE the setup that needs it.
#
# t01 already guarded itself with `gnu_tar || SKIP`, but the setup below calls release-
# assets.sh unguarded at file scope — and that tool hard-errors ("GNU tar required for
# deterministic assets (run on ubuntu/CI)") and exits 2 on BSD tar. So on macOS the file
# died at line ~17 and the guard on line 21 was never reached. A guard downstream of the
# thing that kills you is decoration.
#
# This suite is genuinely CI-only: determinism here MEANS GNU tar's --sort/--owner/--mtime,
# which BSD tar has no equivalent for. That is not a portability bug to fix, it is a real
# platform requirement — so exit 0 (skip), never fail. A suite that cannot run on the
# developer's machine must not block their commits; CI on ubuntu is where it has teeth.
if ! gnu_tar; then
  echo "  SKIP test_release_assets.sh — needs GNU tar (BSD/macOS host); runs on ubuntu/CI"
  exit 0
fi

echo "building scratch payload + assets..."
bash "$repo/tools/cyberos-install/build.sh" "$TMP/payload" >/dev/null 2>&1 || { echo FATAL build; exit 1; }
bash "$RA" "$TMP/payload" "$TMP/assets" >/dev/null || { echo FATAL assets; exit 1; }
ver="$(cat "$TMP/payload/VERSION")"

t01_deterministic_tarball() {                                        # AC 1
  gnu_tar || { echo "  SKIP t01 (non-GNU tar host)"; return; }
  bash "$RA" "$TMP/payload" "$TMP/assets2" >/dev/null || { fail t01 "second run failed"; return; }
  a="$(sha256sum "$TMP/assets/cyberos-payload-$ver.tar.gz" | cut -d' ' -f1)"
  b="$(sha256sum "$TMP/assets2/cyberos-payload-$ver.tar.gz" | cut -d' ' -f1)"
  [ "$a" = "$b" ] && ok t01 || fail t01 "tarballs differ"
}
t02_five_files_two_name_forms() {                                    # AC 2
  local all=1
  for f in "cyberos-payload-$ver.tar.gz" cyberos-payload.tar.gz "cyberos-$ver.plugin" cyberos.plugin SHA256SUMS; do
    [ -f "$TMP/assets/$f" ] || { fail t02 "missing $f"; all=0; }
  done
  cmp -s "$TMP/assets/cyberos-payload-$ver.tar.gz" "$TMP/assets/cyberos-payload.tar.gz" || { fail t02 "tarball twins differ"; all=0; }
  cmp -s "$TMP/assets/cyberos-$ver.plugin" "$TMP/assets/cyberos.plugin" || { fail t02 "plugin twins differ"; all=0; }
  [ "$all" -eq 1 ] && ok t02
}
t03_sha256sums_roundtrip() {                                         # AC 3
  (cd "$TMP/assets" && sha256sum -c SHA256SUMS >/dev/null 2>&1) || { fail t03 "clean verify failed"; return; }
  cp -r "$TMP/assets" "$TMP/assets3"
  printf 'x' >> "$TMP/assets3/cyberos-payload.tar.gz"
  (cd "$TMP/assets3" && sha256sum -c SHA256SUMS >/dev/null 2>&1) && fail t03 "corruption not detected" || ok t03
}
t04_version_triple_check() {                                         # AC 4
  GITHUB_REF_NAME="v0.0.9" bash "$RA" "$TMP/payload" "$TMP/assets4" >/dev/null 2>&1; rc=$?
  [ "$rc" -eq 10 ] && [ ! -d "$TMP/assets4" ] || { fail t04 "rc=$rc dirExists=$([ -d "$TMP/assets4" ] && echo y)"; return; }
  # dispatch case: GITHUB_REF_NAME is the branch; TAG (from inputs) takes precedence and passes
  TAG="v$ver" GITHUB_REF_NAME="main" bash "$RA" "$TMP/payload" "$TMP/assets4b" >/dev/null 2>&1; rc=$?
  [ "$rc" -eq 0 ] || { fail "t04b" "TAG precedence failed (rc=$rc) - dispatch regression"; return; }
  # and a WRONG TAG still refuses even when GITHUB_REF_NAME looks right
  TAG="v0.0.9" GITHUB_REF_NAME="v$ver" bash "$RA" "$TMP/payload" "$TMP/assets4c" >/dev/null 2>&1; rc=$?
  [ "$rc" -eq 10 ] && ok t04 || fail "t04c" "wrong TAG not refused (rc=$rc)"
}
t05_workflow_shape() {                                               # AC 5
  local W="$repo/.github/workflows/release.yml" all=1
  for pat in '  payload:' 'test "v\$(cat VERSION)" = "\$TAG"' 'release-assets.sh' 'check-version-sync.sh' 'gh release upload' -- ; do
    [ "$pat" = "--" ] && break
    grep -q "$pat" "$W" || { fail t05 "missing: $pat"; all=0; }
  done
  grep -q -- '--clobber' "$W" || { fail t05 "missing --clobber"; all=0; }
  python3 -c "import yaml,sys; yaml.safe_load(open('$W'))" 2>/dev/null || { fail t05 "yaml parse"; all=0; }
  [ "$all" -eq 1 ] && ok t05
}
t06_bootstrap_url_happy_path() {                                     # AC 6
  local T="$TMP/target6"; mkdir -p "$T"; (cd "$T" && git init -q)
  (cd "$T" && CYBEROS_PAYLOAD_URL="file://$TMP/assets/cyberos-payload.tar.gz" bash "$BOOT" "$T" >/dev/null 2>&1) || { fail t06 "bootstrap failed"; return; }
  [ -f "$T/.cyberos/VERSION" ] && [ "$(cat "$T/.cyberos/VERSION")" = "$ver" ] && ok t06 || fail t06 ".cyberos/VERSION missing or wrong"
}
t07_bootstrap_bad_checksum() {                                       # AC 7
  local A="$TMP/badassets"; cp -r "$TMP/assets" "$A"; printf 'x' >> "$A/cyberos-payload.tar.gz"
  local T="$TMP/target7"; mkdir -p "$T"; (cd "$T" && git init -q)
  (cd "$T" && CYBEROS_PAYLOAD_URL="file://$A/cyberos-payload.tar.gz" bash "$BOOT" "$T" >/dev/null 2>&1) && { fail t07 "bootstrap passed on tampered tarball"; return; }
  [ ! -d "$T/.cyberos" ] && ok t07 || fail t07 ".cyberos created despite checksum failure"
}
t08_rollout_from_release() {                                         # AC 8
  local R1="$TMP/r1" R2="$TMP/r2"
  for r in "$R1" "$R2"; do mkdir -p "$r"; (cd "$r" && git init -q && git config user.email t@t && git config user.name t && echo hi > README.md && git add -A && git commit -qm init); done
  CYBEROS_PAYLOAD_URL="file://$TMP/assets/cyberos-payload.tar.gz" bash "$ROLL" --from-release "$R1" "$R2" >/dev/null 2>&1 || { fail t08 "rollout failed"; return; }
  [ -f "$R1/.cyberos/VERSION" ] && [ -f "$R2/.cyberos/VERSION" ] && ok t08 || fail t08 "targets not initialized"
}
t09_docs_real_urls() {                                               # AC 9
  grep -q 'releases/latest/download' "$repo/tools/cyberos-install/README.md" \
    && grep -q 'releases/latest/download' "$repo/docs/deploy/RELEASE.md" \
    && ! grep -q 'available once you host a tarball' "$repo/tools/cyberos-install/README.md" \
    && ok t09 || fail t09 "docs missing real URLs or placeholder remains"
}

t01_deterministic_tarball; t02_five_files_two_name_forms; t03_sha256sums_roundtrip
t04_version_triple_check; t05_workflow_shape; t06_bootstrap_url_happy_path
t07_bootstrap_bad_checksum; t08_rollout_from_release; t09_docs_real_urls
echo "----"; echo "pass=$PASS fail=$FAIL"
[ "$FAIL" -eq 0 ]
