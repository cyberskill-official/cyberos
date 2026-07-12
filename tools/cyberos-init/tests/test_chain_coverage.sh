#!/usr/bin/env bash
# test_chain_coverage.sh - FR-SKILL-116 §5 suite (t01-t06 -> AC 1-6).
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
CHECK="$repo/tools/cyberos-init/check-chain-coverage.sh"
BUILD="$repo/tools/cyberos-init/build.sh"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

echo "building scratch payload..."
bash "$BUILD" "$TMP/payload" >/dev/null 2>&1 || { echo "FATAL: scratch build failed"; exit 1; }

t01_pair_vendored() {                                                # AC 1
  local all=1
  for t in cuo plugin; do for s in debugging-cycle-author debugging-cycle-audit; do
    [ -f "$TMP/payload/$t/skills/$s/SKILL.md" ] || { fail t01 "missing $t/skills/$s"; all=0; }
  done; done
  [ "$all" -eq 1 ] && ok t01
}

t02_parses_chain_not_list() {                                        # AC 2
  local P="$TMP/t02"; rm -rf "$P"; cp -r "$TMP/payload" "$P"
  printf '  - { step: 99, skill: nonexistent-author, inputs_from: x, outputs_to: y }\n' >> "$P/cuo/ship-feature-requests.md"
  out="$(bash "$CHECK" "$P" 2>&1)" && rc=0 || rc=$?
  [ "$rc" -eq 10 ] && echo "$out" | grep -q 'MISSING nonexistent-author (referenced by cuo/ship-feature-requests.md)' \
    && ok t02 || fail t02 "rc=$rc out=$out"
  # edge row 2: zero references -> exit 2, never a vacuous pass
  P="$TMP/t02b"; rm -rf "$P"; cp -r "$TMP/payload" "$P"
  sed -i 's/skill:/skil_:/g' "$P/cuo/ship-feature-requests.md"
  find "$P/plugin/commands" -name '*.md' -delete
  bash "$CHECK" "$P" >/dev/null 2>&1; rc=$?
  [ "$rc" -eq 2 ] || fail "t02b" "rc=$rc, want 2 on zero extraction"
}

t03_dropped_pair_fails_build() {                                     # AC 3
  # part 1: a payload lacking the pair fails the check with the two MISSING lines
  local P="$TMP/t03"; rm -rf "$P"; cp -r "$TMP/payload" "$P"
  rm -rf "$P/cuo/skills/debugging-cycle-author" "$P/cuo/skills/debugging-cycle-audit" \
         "$P/plugin/skills/debugging-cycle-author" "$P/plugin/skills/debugging-cycle-audit"
  out="$(bash "$CHECK" "$P" 2>&1)" && rc=0 || rc=$?
  [ "$rc" -eq 10 ] && echo "$out" | grep -q 'MISSING debugging-cycle-author' \
                   && echo "$out" | grep -q 'MISSING debugging-cycle-audit' \
    || { fail t03 "rc=$rc out=$out"; return; }
  # part 2: build.sh runs the check as its final step, so the same state fails the BUILD
  grep -q 'check-chain-coverage.sh" "\$out"' "$BUILD" && ok t03 || fail t03 "build.sh does not invoke the check"
}

t04_allowlist_both_ways() {                                          # AC 4
  # real allowlist: awh-gate/caf-gate referenced by the chain, no failure (t01 build already proved exit 0)
  out="$(bash "$CHECK" "$TMP/payload" 2>/dev/null)" || { fail t04 "clean payload failed"; return; }
  # rot warning: an entry nothing references and nothing vendors warns on stderr, exit stays 0
  local A="$TMP/allow.txt"; cp "$repo/tools/cyberos-init/chain-allowlist.txt" "$A"
  echo "zombie-skill   # stale test entry" >> "$A"
  errout="$(CYBEROS_CHAIN_ALLOWLIST="$A" bash "$CHECK" "$TMP/payload" 2>&1 >/dev/null)"; rc=$?
  [ "$rc" -eq 0 ] && echo "$errout" | grep -q "zombie-skill" && ok t04 || fail t04 "rc=$rc err=$errout"
  # edge row 7: an allowlist TYPO cannot silently skip a real miss
  local P="$TMP/t04b"; rm -rf "$P"; cp -r "$TMP/payload" "$P"
  rm -rf "$P/cuo/skills/debugging-cycle-author" "$P/plugin/skills/debugging-cycle-author"
  cp "$repo/tools/cyberos-init/chain-allowlist.txt" "$A"
  echo "debugging-cycleauthor  # typo" >> "$A"
  CYBEROS_CHAIN_ALLOWLIST="$A" bash "$CHECK" "$P" >/dev/null 2>&1; rc=$?
  [ "$rc" -eq 10 ] || fail "t04b" "typo allowlist suppressed a real MISSING (rc=$rc)"
}

t05_unpaired_detected() {                                            # AC 5
  local P="$TMP/t05"; rm -rf "$P"; cp -r "$TMP/payload" "$P"
  rm -rf "$P/cuo/skills/repo-context-map-audit"
  out="$(bash "$CHECK" "$P" 2>&1)" && rc=0 || rc=$?
  [ "$rc" -eq 10 ] && echo "$out" | grep -q 'UNPAIRED repo-context-map-author' \
    && ok t05 || fail t05 "rc=$rc out=$out"
}

t06_readonly_check() {                                               # AC 6
  local P="$TMP/t06"; rm -rf "$P"; cp -r "$TMP/payload" "$P"
  before="$(cd "$P" && find . -type f -exec sha256sum {} + | sort | sha256sum)"
  (cd "$TMP" && bash "$CHECK" "$P" >/dev/null 2>&1) || { fail t06 "check failed on copy"; return; }
  after="$(cd "$P" && find . -type f -exec sha256sum {} + | sort | sha256sum)"
  [ "$before" = "$after" ] && ok t06 || fail t06 "payload mutated by the check"
  # edge rows 1/4: missing payload / missing doc -> exit 2
  bash "$CHECK" "$TMP/nonexistent" >/dev/null 2>&1; [ $? -eq 2 ] || fail "t06b" "missing payload not exit 2"
  rm "$P/cuo/ship-feature-requests.md"
  bash "$CHECK" "$P" >/dev/null 2>&1; [ $? -eq 2 ] || fail "t06c" "missing doc not exit 2"
}

t01_pair_vendored; t02_parses_chain_not_list; t03_dropped_pair_fails_build
t04_allowlist_both_ways; t05_unpaired_detected; t06_readonly_check

t07_reduced_profile_skips() {                                        # FR-SKILL-116 §1 #5 amendment
  local P="$TMP/t07"; rm -rf "$P"; cp -r "$TMP/payload" "$P"
  rm -rf "$P"/cuo/skills/* 
  out="$(bash "$CHECK" "$P" 2>/dev/null)"; rc=$?
  [ "$rc" -eq 0 ] && echo "$out" | grep -q "chain SKIP: reduced profile" && ok t07 || fail t07 "rc=$rc out=$out"
}
t07_reduced_profile_skips
echo "----"; echo "pass=$PASS fail=$FAIL"
[ "$FAIL" -eq 0 ]
