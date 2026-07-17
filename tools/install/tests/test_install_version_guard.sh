#!/usr/bin/env bash
# TASK-IMP-104 - refuse an older payload over a newer .cyberos. One arm per AC.
set -uo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; root="$(cd "$here/../../.." && pwd)"
pass=0; fail=0
ok(){ printf '  ok   %s\n' "$1"; pass=$((pass+1)); }
no(){ printf '  FAIL %s\n     %s\n' "$1" "${2:-}"; fail=$((fail+1)); }
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
SRC="$root/tools/install"
# A thin stub payload cannot complete an install (templates, gates, migration all live in the
# real one), so the arms that must SUCCEED need the built payload. Copy it once; each arm gets
# its own copy with only VERSION rewritten - the guard reads VERSION and nothing else.
DIST="$root/dist/cyberos"
[ -d "$DIST" ] || { echo "  SKIP: no dist/cyberos - run tools/install/build.sh first"; exit 0; }

# A scratch payload at a chosen version + a scratch repo with an installed VERSION.
mk(){ # mk <payload_ver> <installed_ver|"">
  local d="$1"; shift; local pv="$1" iv="${2:-}"
  mkdir -p "$d/repo/.cyberos"
  cp -R "$DIST" "$d/payload"
  printf '%s\n' "$pv" > "$d/payload/VERSION"
  [ -n "$iv" ] && printf '%s\n' "$iv" > "$d/repo/.cyberos/VERSION"
  ( cd "$d/repo" && git init -q -b main . && git -c user.email=t@t -c user.name=t commit -q --allow-empty -m i )
}
run(){ # run <dir> [env...] -> prints output, sets RC
  local d="$1"; shift
  out="$(env "$@" CYBEROS_NO_MIGRATE=1 bash "$d/payload/install.sh" "$d/repo" 2>&1)"; RC=$?
}

# --- AC 1 (#1.1,#1.3): older payload over newer installed -> refuse, name both, don't vendor
t01_downgrade_refused(){
  local d="$TMP/t01"; mk "$d" 1.0.0 2.0.0
  run "$d"
  [ "$RC" -ne 0 ] || { no t01_downgrade_refused "expected non-zero, got $RC"; return; }
  grep -q "is OLDER than the installed" <<<"$out" || { no t01_downgrade_refused "no refusal: $out"; return; }
  grep -q "1.0.0" <<<"$out" && grep -q "2.0.0" <<<"$out" || { no t01_downgrade_refused "both versions not named: $out"; return; }
  grep -q "CYBEROS_ALLOW_DOWNGRADE=1" <<<"$out" || { no t01_downgrade_refused "override not named: $out"; return; }
  [ -d "$d/repo/.cyberos/cuo" ] && { no t01_downgrade_refused "VENDORED under a refusal - violates 1.3"; return; }
  ok t01_downgrade_refused
}
# --- AC 2 (#1.4): override proceeds and the summary records both versions
t02_override_records_both(){
  local d="$TMP/t02"; mk "$d" 1.0.0 2.0.0
  run "$d" CYBEROS_ALLOW_DOWNGRADE=1
  [ "$RC" -eq 0 ] || { no t02_override_records_both "override did not proceed (rc=$RC): $out"; return; }
  grep -q "DOWNGRADE" <<<"$out" || { no t02_override_records_both "downgrade not recorded: $out"; return; }
  grep -q "2.0.0 -> 1.0.0" <<<"$out" || { no t02_override_records_both "both versions not in the record: $out"; return; }
  ok t02_override_records_both
}
# --- AC 3 (#1.5): equal version is silent (the documented idempotent path)
t03_equal_is_silent(){
  local d="$TMP/t03"; mk "$d" 1.0.0 1.0.0
  run "$d"
  [ "$RC" -eq 0 ] || { no t03_equal_is_silent "equal re-install failed (rc=$RC)"; return; }
  grep -qE "OLDER|DOWNGRADE|not comparable" <<<"$out" && { no t03_equal_is_silent "equal path emitted guard noise: $out"; return; }
  ok t03_equal_is_silent
}
# --- AC 4 (#1.6): absent and unparseable VERSION both proceed, condition named
t04_missing_version_proceeds(){
  local d="$TMP/t04a"; mk "$d" 1.0.0 ""          # absent -> first install, silent
  run "$d"
  [ "$RC" -eq 0 ] || { no t04_missing_version_proceeds "absent VERSION blocked install (rc=$RC): $out"; return; }
  grep -q "not comparable" <<<"$out" && { no t04_missing_version_proceeds "absent VERSION treated as incomparable, should be silent"; return; }
  local e="$TMP/t04b"; mk "$e" 1.0.0 "not-a-version"   # unparseable -> proceed, NAMED
  run "$e"
  [ "$RC" -eq 0 ] || { no t04_missing_version_proceeds "unparseable VERSION blocked install (rc=$RC): $out"; return; }
  grep -q "not comparable" <<<"$out" || { no t04_missing_version_proceeds "unparseable condition not named: $out"; return; }
  ok t04_missing_version_proceeds
}
# --- AC 5 (#1.2): ONE comparator. install defines no ver_lt/is_ver of its own.
t05_single_comparator(){
  grep -qE "^\s*(ver_lt|is_ver)\s*\(\)" "$SRC/install.sh" && { no t05_single_comparator "install.sh defines its own comparator - violates 1.2"; return; }
  grep -q "version-compare.sh" "$SRC/install.sh" || { no t05_single_comparator "install.sh does not source the shared comparator"; return; }
  local n; n=$(grep -rlE "^\s*ver_lt\s*\(\)" "$root/tools/install" | wc -l | tr -d ' ')
  [ "$n" -eq 1 ] || { no t05_single_comparator "ver_lt defined in $n files - must be exactly 1"; return; }
  ok t05_single_comparator
}
# --- edge (§3): newer payload proceeds (the normal upgrade)
t06_newer_proceeds(){
  local d="$TMP/t06"; mk "$d" 2.0.0 1.0.0
  run "$d"
  [ "$RC" -eq 0 ] || { no t06_newer_proceeds "upgrade blocked (rc=$RC): $out"; return; }
  grep -q "OLDER" <<<"$out" && { no t06_newer_proceeds "upgrade misread as downgrade"; return; }
  ok t06_newer_proceeds
}
echo "test_install_version_guard.sh (TASK-IMP-104)"
t01_downgrade_refused; t02_override_records_both; t03_equal_is_silent
t04_missing_version_proceeds; t05_single_comparator; t06_newer_proceeds
echo "  ---"; echo "  $pass passed, $fail failed"
[ "$fail" -eq 0 ]
