#!/usr/bin/env bash
# test_sync_host_plugins.sh — gate for tools/install/sync-host-plugins.sh
# Standalone bash, no framework. Run: bash tools/install/tests/test_sync_host_plugins.sh
set -uo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
SYNC="$repo/tools/install/sync-host-plugins.sh"
BUILD="$repo/tools/install/build.sh"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

echo "building scratch payload..."
# Host sync must NOT run for a non-default out path (and we force-disable it anyway).
CYBEROS_SYNC_HOST_PLUGINS=0 bash "$BUILD" "$TMP/payload" >/dev/null 2>&1 \
  || { echo "FATAL: scratch build failed"; exit 1; }
pv="$(tr -d ' \n\r' < "$TMP/payload/VERSION")"

t01_missing_payload_exits_2() {
  out="$(bash "$SYNC" "$TMP/no-such-dir" 2>&1)" && rc=0 || rc=$?
  [ "$rc" -eq 2 ] && echo "$out" | grep -qi 'missing\|ERROR' && ok t01 || fail t01 "rc=$rc out=$out"
}

t02_non_marketplace_exits_2() {
  mkdir -p "$TMP/empty"
  out="$(bash "$SYNC" "$TMP/empty" 2>&1)" && rc=0 || rc=$?
  [ "$rc" -eq 2 ] && ok t02 || fail t02 "rc=$rc out=$out"
}

t03_dry_run_plans_both_hosts() {
  out="$(CYBEROS_SYNC_HOST_PLUGINS_DRY_RUN=1 bash "$SYNC" "$TMP/payload" 2>&1)" && rc=0 || rc=$?
  [ "$rc" -eq 0 ] \
    && echo "$out" | grep -q "version=$pv" \
    && echo "$out" | grep -q "dry-run:.*claude plugin install cyberos@cyberos" \
    && echo "$out" | grep -q "dry-run:.*grok plugin install" \
    && ok t03 || fail t03 "rc=$rc out=$out"
}

t04_offline_skips() {
  out="$(CYBEROS_OFFLINE=1 bash "$SYNC" "$TMP/payload" 2>&1)" && rc=0 || rc=$?
  [ "$rc" -eq 0 ] && echo "$out" | grep -qi 'skip.*(CYBEROS_OFFLINE=1)' && ok t04 || fail t04 "rc=$rc out=$out"
}

t05_per_host_disable() {
  # dry-run still prints host sections; with both disabled we should see skip lines and no dry-run installs.
  out="$(CYBEROS_SYNC_HOST_PLUGINS_DRY_RUN=1 CYBEROS_SYNC_CLAUDE=0 CYBEROS_SYNC_GROK=0 bash "$SYNC" "$TMP/payload" 2>&1)" && rc=0 || rc=$?
  [ "$rc" -eq 0 ] \
    && echo "$out" | grep -q 'claude: skip (CYBEROS_SYNC_CLAUDE=0)' \
    && echo "$out" | grep -q 'grok: skip (CYBEROS_SYNC_GROK=0)' \
    && ! echo "$out" | grep -q 'dry-run:.*plugin install' \
    && ok t05 || fail t05 "rc=$rc out=$out"
}

t06_build_default_path_invokes_sync_dry() {
  # Prove build.sh wires the hook: force-sync on a non-default out under DRY_RUN so no real host CLIs run.
  out="$(CYBEROS_SYNC_HOST_PLUGINS=1 CYBEROS_SYNC_HOST_PLUGINS_DRY_RUN=1 bash "$BUILD" "$TMP/payload2" 2>&1)" && rc=0 || rc=$?
  [ "$rc" -eq 0 ] \
    && echo "$out" | grep -q 'cyberos sync-host-plugins:' \
    && echo "$out" | grep -q 'dry-run:.*claude plugin install cyberos@cyberos' \
    && ok t06 || fail t06 "rc=$rc out=$(echo "$out" | tail -40)"
}

t07_build_scratch_auto_skips_sync() {
  # Default auto mode + non-canonical out => no sync-host-plugins banner.
  out="$(CYBEROS_SYNC_HOST_PLUGINS=auto bash "$BUILD" "$TMP/payload3" 2>&1)" && rc=0 || rc=$?
  [ "$rc" -eq 0 ] \
    && ! echo "$out" | grep -q 'cyberos sync-host-plugins:' \
    && ok t07 || fail t07 "rc=$rc out=$(echo "$out" | tail -40)"
}

t01_missing_payload_exits_2
t02_non_marketplace_exits_2
t03_dry_run_plans_both_hosts
t04_offline_skips
t05_per_host_disable
t06_build_default_path_invokes_sync_dry
t07_build_scratch_auto_skips_sync

echo
echo "PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ]
