#!/usr/bin/env bash
# test_check_latest.sh - TASK-IMP-070 §5 suite (t01-t08 -> AC 1-8).
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
CL="$repo/tools/cyberos-init/check-latest.sh"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

echo "building scratch payload..."
bash "$repo/tools/cyberos-init/build.sh" "$TMP/payload" >/dev/null 2>&1 || { echo FATAL build; exit 1; }
pv="$(cat "$TMP/payload/VERSION")"
mkrepo() { local r="$1" v="$2"; rm -rf "$r"; mkdir -p "$r/.cyberos"; (cd "$r" && git init -q); [ "$v" != none ] && echo "$v" > "$r/.cyberos/VERSION"; true; }
chk() { CYBEROS_RELEASE_ENDPOINT="${2:-}" ${3:+CYBEROS_OFFLINE=1} CYBEROS_NONINTERACTIVE=1 bash "$TMP/payload/version.sh" "$1"; }

t01_bare_version_endpoint() {                                        # AC 1
  echo "1.8.0" > "$TMP/bare"
  out="$(CYBEROS_RELEASE_ENDPOINT="$TMP/bare" bash "$CL")"
  [ "$out" = "latest=1.8.0 source=$TMP/bare" ] && ok t01 || fail t01 "$out"
}
t02_github_json_endpoint() {                                         # AC 2
  printf '{"url": "x", "tag_name": "v1.8.0", "name": "CyberOS v1.8.0"}' > "$TMP/json"
  out="$(CYBEROS_RELEASE_ENDPOINT="$TMP/json" bash "$CL")"
  [ "$out" = "latest=1.8.0 source=$TMP/json" ] && ok t02 || fail t02 "$out"
}
t03_unreachable_degrades() {                                         # AC 3
  out="$(CYBEROS_RELEASE_ENDPOINT="$TMP/nonexistent-endpoint" bash "$CL")"; rc=$?
  [ "$rc" -eq 0 ] && [ "$out" = "latest=unknown source=offline" ] || { fail t03 "rc=$rc out=$out"; return; }
  mkrepo "$TMP/r3" "$pv"
  out="$(CYBEROS_RELEASE_ENDPOINT="$TMP/nonexistent-endpoint" CYBEROS_NONINTERACTIVE=1 bash "$TMP/payload/version.sh" "$TMP/r3")"
  echo "$out" | grep -q "verdict=" && ok t03 || fail t03 "no verdict on degraded check"
}
t04_verdict_matrix() {                                               # AC 4
  local all=1
  echo "$pv" > "$TMP/eq"                       # A: inst==payload==latest -> up_to_date
  mkrepo "$TMP/rA" "$pv"
  out="$(CYBEROS_RELEASE_ENDPOINT="$TMP/eq" CYBEROS_NONINTERACTIVE=1 bash "$TMP/payload/version.sh" "$TMP/rA")"
  echo "$out" | grep -q "^verdict=up_to_date$" || { fail "t04A" "$out"; all=0; }
  PB="$TMP/payloadB"; rm -rf "$PB"; cp -r "$TMP/payload" "$PB"; echo "2.0.0" > "$PB/VERSION"
  mkrepo "$TMP/rB" "1.0.0"                     # B: inst<payload (pinned 2.0.0), latest unknown -> repo_stale + next
  out="$(CYBEROS_OFFLINE=1 CYBEROS_NONINTERACTIVE=1 bash "$PB/version.sh" "$TMP/rB")"
  echo "$out" | grep -q "^verdict=repo_stale$" && echo "$out" | grep -qE "^next: bash .*(install|update)\.sh" || { fail "t04B" "$out"; all=0; }
  echo "9.9.9" > "$TMP/newer"                  # C: payload<latest -> payload_stale + fetch line
  mkrepo "$TMP/rC" "$pv"
  out="$(CYBEROS_RELEASE_ENDPOINT="$TMP/newer" CYBEROS_NONINTERACTIVE=1 bash "$TMP/payload/version.sh" "$TMP/rC")"
  echo "$out" | grep -q "^verdict=payload_stale$" && echo "$out" | grep -q "releases/latest/download" || { fail "t04C" "$out"; all=0; }
  mkrepo "$TMP/rD" "$pv"                       # D: latest unknown + inst==payload -> up_to_date + note
  out="$(CYBEROS_OFFLINE=1 CYBEROS_NONINTERACTIVE=1 bash "$TMP/payload/version.sh" "$TMP/rD")"
  echo "$out" | grep -q "^verdict=up_to_date$" && echo "$out" | grep -qi "note:" || { fail "t04D" "$out"; all=0; }
  echo "$out" | grep -c "^verdict=" | grep -q "^1$" || { fail "t04-single" "multiple verdict lines"; all=0; }
  [ "$all" -eq 1 ] && ok t04
}
t05_numeric_semver() {                                               # AC 5
  echo "1.10.0" > "$TMP/ten"
  mkrepo "$TMP/r5" "1.9.0"
  P5="$TMP/payload5"; rm -rf "$P5"; cp -r "$TMP/payload" "$P5"; echo "1.10.0" > "$P5/VERSION"
  out="$(CYBEROS_RELEASE_ENDPOINT="$TMP/ten" CYBEROS_NONINTERACTIVE=1 bash "$P5/version.sh" "$TMP/r5")"
  echo "$out" | grep -q "^verdict=repo_stale$" && ok t05 || fail t05 "string-compare inversion? $out"
}
t06_offline_env() {                                                  # AC 6
  echo "9.9.9" > "$TMP/trap"
  out="$(CYBEROS_OFFLINE=1 CYBEROS_RELEASE_ENDPOINT="$TMP/trap" bash "$CL")"
  [ "$out" = "latest=unknown source=offline" ] && ok t06 || fail t06 "offline env ignored: $out"
}
t07_version_doc_contract() {                                         # AC 7
  local D="$repo/tools/cyberos-init/plugin/commands/version.md" all=1
  for pat in 'installed=' 'payload=' 'latest=' 'verdict=repo_stale' 'verdict=payload_stale' 'NEVER claim "up to date" from the local-payload comparison alone'; do
    grep -q "$pat" "$D" || { fail t07 "missing: $pat"; all=0; }
  done
  [ "$all" -eq 1 ] && ok t07
}
t08_version_release_pointer() {                                      # AC 8
  grep -q 'github.com/cyberskill-official/cyberos/releases' "$repo/tools/cyberos-init/plugin/commands/version.md" \
    && grep -q 'check-latest.sh' "$repo/tools/cyberos-init/plugin/commands/version.md" \
    && ok t08 || fail t08 "release / check-latest pointer missing from version.md"
}

t01_bare_version_endpoint; t02_github_json_endpoint; t03_unreachable_degrades; t04_verdict_matrix
t05_numeric_semver; t06_offline_env; t07_version_doc_contract; t08_version_release_pointer
echo "----"; echo "pass=$PASS fail=$FAIL"
[ "$FAIL" -eq 0 ]
