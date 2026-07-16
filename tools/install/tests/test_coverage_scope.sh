#!/usr/bin/env bash
# test_coverage_scope.sh - coverage-scope.mjs, the task-diff -> per-file-coverage
# skeleton emitter (TASK-IMP-098).
#
#   t01  base resolution: --base wins over an existing entry-flip commit; the
#        subject scan resolves the EARLIEST of two commits naming the task id +
#        "implementing" and notes the ambiguity in the range note; no match
#        fails loudly (exit 3) demanding --base - never a guessed range.
#   t02  fixture repo + BOTH report shapes (c8/istanbul coverage-summary.json
#        with absolute path keys, lcov.info with mixed relative/absolute SF
#        paths) emit the EXPECTED BYTES: a below-90 file, an exactly-90 file
#        NOT below (strict <90), a no-coverage-data row for a touched file
#        absent from the report, and the deletion named in the notes;
#        --out writes the same bytes and keeps stdout clean.
#   t03  any other report is refused BY NAME (exit 4) naming the two supported
#        shapes; nothing is written; a coverage-summary.json without 'total'
#        is refused on content (exit 2).
#   t04  the assembled payload carries the tool byte-identically and the
#        vendored copy runs (--help exits 0, documents both shapes).
#
# Fixture repos are scratch git repos built under mktemp inside each scenario
# (git init -b main + commits following the corpus entry-flip subject
# convention) - this suite never touches the enclosing repo's git state.
#
# Origin: IMPROVEMENT_HANDOFF.md IMP-14 - every recorded gate mapped diff to
# coverage by hand; this suite gates the tool that owns that mechanical walk.
#
# Usage: bash test_coverage_scope.sh [t01 t02 ...]   (no args = all scenarios)
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
CS="$repo/tools/install/docs-tools/coverage-scope.mjs"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }
only="$*"
want() { [ -z "$only" ] && return 0; case " $only " in *" $1 "*) return 0;; *) return 1;; esac; }

# scratch-repo git: isolated from user/system config, deterministic identity
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null
g() { git -C "$1" -c user.email=t@t -c user.name=t -c commit.gpgsign=false "${@:2}"; }

cs() { node "$CS" "$@" > "$TMP/out" 2> "$TMP/err"; }

# The t02 fixture: seed -> entry-flip -> work (modify low+exact, add doc, delete gone).
# Sets: FIX (repo dir), BASE_SHA (flip commit), HEAD_SHA.
emit_fixture() { # $1 = dir
  local d="$1"; mkdir -p "$d"; g "$d" init -q -b main .
  mkdir -p "$d/src" "$d/docs" "$d/coverage"
  printf 'v1\n' > "$d/src/low.js"; printf 'v1\n' > "$d/src/exact.js"; printf 'v1\n' > "$d/src/gone.js"
  g "$d" add -A; g "$d" commit -qm "seed corpus"
  g "$d" tag v-seed
  g "$d" commit -qm "TASK-W-001: enter implementing" --allow-empty
  BASE_SHA="$(g "$d" rev-parse HEAD)"
  printf 'v2\n' > "$d/src/low.js"; printf 'v2\n' > "$d/src/exact.js"; printf 'notes\n' > "$d/docs/notes.md"
  rm "$d/src/gone.js"
  g "$d" add -A; g "$d" commit -qm "TASK-W-001: implement the thing"
  HEAD_SHA="$(g "$d" rev-parse HEAD)"
  # c8/istanbul summary: ABSOLUTE path keys (what c8 emits), 85.71 below / 90 exactly-at
  cat > "$d/coverage/coverage-summary.json" <<EOF
{"total":{"lines":{"total":24,"covered":21,"skipped":0,"pct":87.5}},
 "$d/src/low.js":{"lines":{"total":14,"covered":12,"skipped":0,"pct":85.71}},
 "$d/src/exact.js":{"lines":{"total":10,"covered":9,"skipped":0,"pct":90}}}
EOF
  # lcov: same numbers, MIXED relative + absolute SF paths (both must normalize)
  printf 'TN:\nSF:src/low.js\nLF:14\nLH:12\nend_of_record\nSF:%s/src/exact.js\nLF:10\nLH:9\nend_of_record\n' "$d" > "$d/coverage/lcov.info"
}

# expected skeleton with placeholders (quoted heredoc keeps backticks literal)
emit_expected() { # $1 = out-file, then sed fills @BASE@ @HEAD@ @REPORT@
  cat > "$1" <<'EXP'
---
artefact: coverage-gate@1
task: TASK-W-001
phase: testing
tests_failed: TODO
files_below_90pct: [src/low.js]
ecm_rows_uncovered: TODO
---
# Coverage scope skeleton - TASK-W-001

Range: `@BASE@...@HEAD@` (base via subject-scan (entry-flip commit: "TASK-W-001: enter implementing"))
Report: @REPORT@

| file | lines.pct | status |
|---|---|---|
| docs/notes.md | no-coverage-data | no-coverage-data |
| src/exact.js | 90 | ok |
| src/low.js | 85.71 | below-90 |

Notes:
- deleted in range (excluded from the table per #1.2): src/gone.js
- 1 touched file(s) carry no coverage data - visible above, never silently dropped.
- TODO (coverage-gate-author): tests_failed from the suite run; ecm_rows_uncovered from the edge-case-matrix cross-walk; raw_terminal attached at authoring.
EXP
}

t01_base_resolution() {
  local d="$TMP/t01/repo"; mkdir -p "$d"; g "$d" init -q -b main .
  mkdir -p "$d/src" "$d/coverage"
  printf '{"total":{"lines":{"total":0,"covered":0,"skipped":0,"pct":100}}}\n' > "$d/coverage/coverage-summary.json"
  printf 'v1\n' > "$d/src/a.js"; g "$d" add -A; g "$d" commit -qm "seed"; g "$d" tag v-seed
  local seed_sha; seed_sha="$(g "$d" rev-parse HEAD)"
  g "$d" commit -qm "TASK-B-002: enter implementing" --allow-empty
  local flip1_sha; flip1_sha="$(g "$d" rev-parse HEAD)"
  printf 'v2\n' > "$d/src/a.js"; g "$d" add -A; g "$d" commit -qm "TASK-B-002: midway work"
  g "$d" commit -qm "TASK-B-002: back to implementing after route-back" --allow-empty
  printf 'v3\n' > "$d/src/a.js"; g "$d" add -A; g "$d" commit -qm "TASK-B-002: more work"
  # arm 1: --base WINS even though entry-flip commits exist
  cs TASK-B-002 --repo "$d" --base v-seed || { fail t01 "--base arm rc!=0: $(cat "$TMP/err")"; return; }
  grep -q "Range: \`$seed_sha\.\.\." "$TMP/out" || { fail t01 "--base did not win: $(grep Range "$TMP/out")"; return; }
  grep -q "base via --base 'v-seed'" "$TMP/out" || { fail t01 "--base provenance missing"; return; }
  grep -q "Note: .* commit subjects" "$TMP/out" && { fail t01 "--base arm carries a scan note"; return; }
  # arm 2: subject scan resolves the EARLIEST of the two matches + notes ambiguity
  cs TASK-B-002 --repo "$d" || { fail t01 "scan arm rc!=0: $(cat "$TMP/err")"; return; }
  grep -q "Range: \`$flip1_sha\.\.\." "$TMP/out" || { fail t01 "scan did not pick the earliest flip: $(grep Range "$TMP/out")"; return; }
  grep -q 'base via subject-scan (entry-flip commit: "TASK-B-002: enter implementing")' "$TMP/out" \
    || { fail t01 "scan provenance wrong: $(grep 'base via' "$TMP/out")"; return; }
  grep -q 'Note: 2 commit subjects name TASK-B-002 + "implementing"; the EARLIEST was used as base - pass --base to override.' "$TMP/out" \
    || { fail t01 "ambiguity not noted in the range note: $(cat "$TMP/out")"; return; }
  # arm 3: no match -> loud fail demanding --base (exit 3), no skeleton emitted
  cs TASK-NOPE --repo "$d"; local rc=$?
  { [ "$rc" -eq 3 ] && grep -q "pass --base" "$TMP/err" && grep -q "never guesses" "$TMP/err"; } \
    || { fail t01 "no-match rc=$rc err=$(cat "$TMP/err")"; return; }
  [ -s "$TMP/out" ] && { fail t01 "no-match arm still emitted a skeleton"; return; }
  ok t01
}

t02_skeleton_from_fixture() {
  local d="$TMP/t02/repo"; BASE_SHA=""; HEAD_SHA=""; emit_fixture "$d"
  emit_expected "$TMP/t02/tpl.md"
  # arm 1: c8/istanbul coverage-summary.json (default discovery, no --coverage flag)
  sed -e "s|@BASE@|$BASE_SHA|" -e "s|@HEAD@|$HEAD_SHA|" \
      -e "s|@REPORT@|coverage/coverage-summary.json (coverage-summary.json shape, lines.pct)|" \
      "$TMP/t02/tpl.md" > "$TMP/t02/want-json.md"
  cs TASK-W-001 --repo "$d" || { fail t02 "istanbul arm rc!=0: $(cat "$TMP/err")"; return; }
  cmp -s "$TMP/out" "$TMP/t02/want-json.md" \
    || { fail t02 "istanbul skeleton differs from expected bytes: $(diff "$TMP/t02/want-json.md" "$TMP/out" | head -6)"; return; }
  # arm 2: lcov.info -> identical table, report line names the lcov shape
  sed -e "s|@BASE@|$BASE_SHA|" -e "s|@HEAD@|$HEAD_SHA|" \
      -e "s|@REPORT@|coverage/lcov.info (lcov.info shape, LF/LH per SF record)|" \
      "$TMP/t02/tpl.md" > "$TMP/t02/want-lcov.md"
  cs TASK-W-001 --repo "$d" --coverage coverage/lcov.info || { fail t02 "lcov arm rc!=0: $(cat "$TMP/err")"; return; }
  cmp -s "$TMP/out" "$TMP/t02/want-lcov.md" \
    || { fail t02 "lcov skeleton differs from expected bytes: $(diff "$TMP/t02/want-lcov.md" "$TMP/out" | head -6)"; return; }
  # arm 3: --out writes the same bytes inside the repo and keeps stdout clean
  cs TASK-W-001 --repo "$d" --out gate-skeleton.md || { fail t02 "--out arm rc!=0: $(cat "$TMP/err")"; return; }
  [ -s "$TMP/out" ] && { fail t02 "--out arm leaked the skeleton to stdout"; return; }
  cmp -s "$d/gate-skeleton.md" "$TMP/t02/want-json.md" || { fail t02 "--out bytes differ from stdout bytes"; return; }
  grep -q "wrote gate-skeleton.md" "$TMP/err" || { fail t02 "--out confirmation missing from stderr"; return; }
  # determinism: a second run is byte-identical
  cs TASK-W-001 --repo "$d" && cmp -s "$TMP/out" "$TMP/t02/want-json.md" \
    || { fail t02 "rerun not byte-identical"; return; }
  ok t02
}

t03_unknown_report_refused() {
  local d="$TMP/t03/repo"; BASE_SHA=""; HEAD_SHA=""; emit_fixture "$d"
  printf '<coverage/>\n' > "$d/coverage/clover.xml"
  cs TASK-W-001 --repo "$d" --coverage coverage/clover.xml --out never.md; local rc=$?
  { [ "$rc" -eq 4 ] && grep -q "unsupported coverage report 'clover.xml'" "$TMP/err" \
      && grep -q "coverage-summary.json" "$TMP/err" && grep -q "lcov.info" "$TMP/err"; } \
    || { fail t03 "clover rc=$rc err=$(cat "$TMP/err")"; return; }
  [ ! -e "$d/never.md" ] || { fail t03 "refused report still wrote --out"; return; }
  [ -s "$TMP/out" ] && { fail t03 "refused report still emitted a skeleton"; return; }
  # shape recognized by name but content invalid: loud exit 2, not a guess
  printf '{"no_total_here":true}\n' > "$d/coverage/coverage-summary.json"
  cs TASK-W-001 --repo "$d"; rc=$?
  { [ "$rc" -eq 2 ] && grep -q "no 'total' key" "$TMP/err"; } \
    || { fail t03 "invalid summary rc=$rc err=$(cat "$TMP/err")"; return; }
  ok t03
}

t04_payload_vendored() {
  bash "$repo/tools/install/build.sh" "$TMP/payload" >/dev/null 2>&1 || { fail t04 "build.sh failed"; return; }
  [ -s "$TMP/payload/docs-tools/coverage-scope.mjs" ] || { fail t04 "payload docs-tools/coverage-scope.mjs missing or empty"; return; }
  cmp -s "$CS" "$TMP/payload/docs-tools/coverage-scope.mjs" \
    || { fail t04 "payload copy differs from tools/install/docs-tools/coverage-scope.mjs"; return; }
  node "$TMP/payload/docs-tools/coverage-scope.mjs" --help > "$TMP/out" 2>&1 \
    || { fail t04 "vendored copy --help failed"; return; }
  grep -q "coverage-summary.json" "$TMP/out" && grep -q "lcov.info" "$TMP/out" && grep -q -- "--base <ref>" "$TMP/out" \
    || { fail t04 "--help does not document the shapes/flags"; return; }
  # the vendored copy runs end-to-end against a scratch fixture
  local d="$TMP/t04/repo"; BASE_SHA=""; HEAD_SHA=""; emit_fixture "$d"
  node "$TMP/payload/docs-tools/coverage-scope.mjs" TASK-W-001 --repo "$d" > "$TMP/out" 2> "$TMP/err" \
    || { fail t04 "vendored copy run failed: $(cat "$TMP/err")"; return; }
  grep -q "files_below_90pct: \[src/low.js\]" "$TMP/out" || { fail t04 "vendored run output wrong"; return; }
  ok t04
}

want t01 && t01_base_resolution
want t02 && t02_skeleton_from_fixture
want t03 && t03_unknown_report_refused
want t04 && t04_payload_vendored

echo "test_coverage_scope: pass=$PASS fail=$FAIL"
[ "$FAIL" -eq 0 ]
