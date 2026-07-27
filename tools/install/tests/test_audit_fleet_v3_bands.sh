#!/usr/bin/env bash
# test_audit_fleet_v3_bands.sh — TASK-DOCS-029
# Status-hub@3 builds band ids in assets/status.js, not static HTML. audit-fleet must
# not false-fail healthy v3 pages on band:pulse etc.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
AUDIT="$repo/tools/install/audit-fleet.sh"
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

# --- t01: source contract — v3 path must prefer status.js, not require HTML ids ---
t01_audit_source_prefers_js() {
  grep -q 'status-hub@3' "$AUDIT" || { fail t01 "no v3 branch"; return; }
  # Must not require static HTML id= before checking assets (the 1.12.0 false-negative).
  # Acceptable shape: band src = assets/status.js OR inlined page with sv3-data.
  grep -q 'docs/status/assets/status.js' "$AUDIT" \
    && grep -q 'sv3-data\|status-feed' "$AUDIT" \
    && ok t01 || fail t01 "audit-fleet missing v3 band-src via status.js / feed"
}

# --- t02: mothership page would fail the OLD html-only check, pass the NEW one ---
t02_mothership_bands_in_js_not_html() {
  local page="$repo/docs/status/index.html"
  local js="$repo/docs/status/assets/status.js"
  [ -f "$page" ] && [ -f "$js" ] || { fail t02 "missing mothership status page/assets"; return; }
  grep -q 'data-template-id="status-hub@3"' "$page" || { fail t02 "not status-hub@3"; return; }

  local html_miss=0 js_miss=0 band
  for band in pulse roadmap sysmap flowband ledger indexband; do
    grep -q "id=\"$band\"" "$page" || html_miss=$((html_miss + 1))
    grep -q "id=\"$band\"" "$js" || js_miss=$((js_miss + 1))
  done
  # Precondition: at least some bands absent from static HTML (JS-built canvas).
  [ "$html_miss" -gt 0 ] || { fail t02 "expected static HTML to lack band ids (html_miss=$html_miss)"; return; }
  [ "$js_miss" -eq 0 ] || { fail t02 "status.js missing bands (js_miss=$js_miss)"; return; }
  ok t02
}

# --- t03: simulate fixed band check against mothership tree ---
t03_fixed_check_passes_mothership() {
  local page="$repo/docs/status/index.html"
  local js="$repo/docs/status/assets/status.js"
  local bad="" band _band_src=""
  if [ -f "$js" ]; then
    _band_src="$js"
  elif grep -q 'id="sv3-data"\|status-feed' "$page"; then
    _band_src="$page"
  fi
  [ -n "$_band_src" ] || { fail t03 "no band src"; return; }
  for band in pulse roadmap sysmap flowband ledger indexband; do
    grep -q "id=\"$band\"" "$_band_src" || bad="$bad band:$band"
  done
  [ -z "$bad" ] && ok t03 || fail t03 "fixed check failed:$bad"
}

t01_audit_source_prefers_js
t02_mothership_bands_in_js_not_html
t03_fixed_check_passes_mothership
echo "----"; echo "pass=$PASS fail=$FAIL"; [ "$FAIL" -eq 0 ]
