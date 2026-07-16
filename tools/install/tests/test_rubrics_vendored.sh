#!/usr/bin/env bash
# test_rubrics_vendored.sh - per-type RULE families ship with the payload and land in installs:
#   t01  the assembled payload carries cuo/rubrics/ with every rubric family the task-audit
#        RUBRIC.md dispatches to (bug.md today, common.md alongside), each non-empty.
#   t02  install.sh lays cuo/rubrics/ into a target repo's .cyberos/cuo/rubrics/ verbatim.
#
# Origin: 2026-07-16 sachviet consumer-repo audit. The vendored RUBRIC.md cites
# contracts/task/rubrics/bug.md (FM-108 routes type: bug to the BUG-*/REGRESSION-* family;
# FM-114 severity depends on BUG-010), but no rubrics/ shipped in the payload - the exact
# incident class of the 2026-07-15 missing per-type templates. A rule that is correct in
# modules/ and absent from dist/ is correct nowhere that matters.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

echo "building scratch payload..."
bash "$repo/tools/install/build.sh" "$TMP/payload" >/dev/null 2>&1 || { echo FATAL build; exit 1; }

t01_payload_carries_rubrics() {
  local missing=""
  for f in bug.md common.md; do
    [ -s "$TMP/payload/cuo/rubrics/$f" ] || missing="$missing $f"
  done
  if [ -n "$missing" ]; then fail t01 "payload cuo/rubrics missing:$missing"; return; fi
  # the source of truth and the payload copy must not drift
  for f in bug.md common.md; do
    if ! cmp -s "$repo/modules/skill/contracts/task/rubrics/$f" "$TMP/payload/cuo/rubrics/$f"; then
      fail t01 "payload cuo/rubrics/$f differs from modules/skill/contracts/task/rubrics/$f"; return
    fi
  done
  ok t01
}

t02_install_lays_rubrics() {
  local d="$TMP/target"; mkdir -p "$d"; (cd "$d" && git init -q . 2>/dev/null || true)
  (cd "$d" && CYBEROS_NO_MIGRATE=1 CYBEROS_NO_HOOK=1 bash "$TMP/payload/install.sh" "$d" >/dev/null 2>&1) \
    || { fail t02 "install.sh failed"; return; }
  for f in bug.md common.md; do
    [ -s "$d/.cyberos/cuo/rubrics/$f" ] || { fail t02 ".cyberos/cuo/rubrics/$f missing after install"; return; }
  done
  ok t02
}

t01_payload_carries_rubrics
t02_install_lays_rubrics

echo "test_rubrics_vendored: pass=$PASS fail=$FAIL"
[ "$FAIL" -eq 0 ]
