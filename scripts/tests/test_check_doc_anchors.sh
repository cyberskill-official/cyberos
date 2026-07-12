#!/usr/bin/env bash
# test_check_doc_anchors.sh - FR-SKILL-119 §5 suite.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../.." && pwd)"
CHECK="$repo/scripts/check_doc_anchors.sh"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

mkfix() { # scratch repo with modules/skill + modules/cuo trees
  d="$1"; mkdir -p "$d/modules/skill/x" "$d/modules/cuo/y" "$d/scripts"
  ( cd "$d" && git init -q . )
  printf '# Target\n\n## Real Heading\nbody\n' > "$d/modules/skill/x/target.md"
}

t01_repo_clean() {                                                    # AC 1 (live tree)
  out="$(bash "$CHECK" 2>/dev/null)"; rc=$?
  [ "$rc" -eq 0 ] && grep -q "anchors OK" <<<"$out" && ok t01 || fail t01 "rc=$rc $out"
}
t02_dead_anchor_detected() {                                          # AC 4
  mkfix "$TMP/a"
  printf 'see [x](modules/skill/x/target.md#no-such-heading)\n' > "$TMP/a/modules/cuo/y/doc.md"
  out="$(cd "$TMP/a" && bash "$CHECK" 2>/dev/null)"; rc=$?
  [ "$rc" -eq 10 ] && grep -q "DEAD modules/cuo/y/doc.md:1 -> modules/skill/x/target.md#no-such-heading" <<<"$out" \
    && ok t02 || fail t02 "rc=$rc $out"
}
t03_valid_anchor_passes() {                                           # AC 4
  mkfix "$TMP/b"
  printf 'see [x](modules/skill/x/target.md#real-heading)\n' > "$TMP/b/modules/cuo/y/doc.md"
  ( cd "$TMP/b" && bash "$CHECK" >/dev/null 2>&1 ) && ok t03 || fail t03 "valid anchor flagged"
}
t04_external_skipped_list_mode() {                                    # AC 5
  mkfix "$TMP/c"
  printf '[ext](https://example.com/x.md#y) and [dead](modules/skill/x/nope.md)\n' > "$TMP/c/modules/cuo/y/doc.md"
  out="$(cd "$TMP/c" && bash "$CHECK" --list 2>/dev/null)"; rc=$?
  [ "$rc" -eq 0 ] && grep -q "dead-file" <<<"$out" && ! grep -q "example.com" <<<"$out" \
    && ok t04 || fail t04 "rc=$rc $out"
}
t05_exemptions_and_warn() {                                           # allowlist discipline
  mkfix "$TMP/d"
  printf 'see [dead](modules/skill/x/nope.md)\n' > "$TMP/d/modules/cuo/y/hist.md"
  printf 'modules/cuo/y/hist.md  # archive\nmodules/skill/ghost.md  # matches nothing\n' > "$TMP/d/scripts/doc-anchor-exemptions.txt"
  err="$( (cd "$TMP/d" && bash "$CHECK" >/dev/null) 2>&1 )"; rc=$?
  [ "$rc" -eq 0 ] && grep -q "WARN unused exemption: modules/skill/ghost.md" <<<"$err" \
    && ok t05 || fail t05 "rc=$rc err=$err"
}
t06_ci_wired() {                                                      # AC 6
  grep -q "check_doc_anchors.sh" "$repo/.github/workflows/payload-gate.yml" \
    && grep -q "modules/skill/\*\*" "$repo/.github/workflows/payload-gate.yml" \
    && ok t06 || fail t06 "workflow step or path filter missing"
}

t01_repo_clean; t02_dead_anchor_detected; t03_valid_anchor_passes
t04_external_skipped_list_mode; t05_exemptions_and_warn; t06_ci_wired
echo "----"; echo "pass=$PASS fail=$FAIL"; [ "$FAIL" -eq 0 ]
