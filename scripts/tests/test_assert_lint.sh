#!/usr/bin/env bash
# test_assert_lint.sh — TASK-IMP-022 §1 suite (t01–t10 → AC1–AC10).
#
# WHAT THIS GUARDS: `scripts/check_defensive_asserts.sh`, the mechanical half of TRACE-008 —
# assertions in the gated test corpus that cannot fail. A lint against defensive assertions is
# the one lint that must not itself be asserted defensively, so every detector here is proved
# from BOTH sides: a fixture it MUST flag and a near-miss fixture it MUST NOT.
#
# The two near-miss arms (t02, t05b) are the load-bearing ones. A regex `assert .* or .*` flags
# `assert s == {"dropbox-or-gdrive"}` and misses a backslash-continued disjunction; t02 and t03
# are the on-disk proof that the AST implementation does neither. t08 asserts the live corpus is
# clean — that is the gate; t01–t07 only prove the gate can fail.
#
#   bash scripts/tests/test_assert_lint.sh
set -uo pipefail
repo="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$repo"
LINT="$repo/scripts/check_defensive_asserts.sh"
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); printf '  \033[32mok\033[0m   %s\n' "$1"; }
fail() { FAIL=$((FAIL+1)); printf '  \033[31mFAIL\033[0m %s: %s\n' "$1" "$2"; }
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT

echo "test_assert_lint.sh (TASK-IMP-022)"

[ -f "$LINT" ] || { fail t00 "missing $LINT"; echo "  pass=$PASS fail=$FAIL"; exit 1; }

# A scratch repo shaped like the corpus roots the lint walks. `git init` so the lint's
# `git rev-parse --show-toplevel` resolves to the SCRATCH root and not to whatever repo TMPDIR
# happens to sit inside — a fixture that silently scanned the real corpus would make every
# arm below meaningless.
new_scratch() {
  local d; d="$(mktemp -d "$TMP/scratch.XXXXXX")"
  mkdir -p "$d/scripts/tests" "$d/tools/install/tests" "$d/tools/docs-site/tests" "$d/modules/m/tests"
  cp "$LINT" "$d/scripts/check_defensive_asserts.sh"
  git -C "$d" init -q 2>/dev/null
  printf '%s' "$d"
}
# Runs the lint in the scratch repo; echoes "<exit>|<stdout+stderr on one line>".
run_lint() {
  local d="$1" out rc
  out="$( cd "$d" && bash scripts/check_defensive_asserts.sh 2>&1 )"; rc=$?
  printf '%s|%s' "$rc" "$(printf '%s' "$out" | tr '\n' '~')"
}

# --- t01 / AC1 — DA-001 flags a disjunctive python assertion ------------------------------
# The R13 shape verbatim: `processed == 3 or failed > 0` passes whenever the writer FAILED.
t01_da001_flags_disjunction() {
  local d; d="$(new_scratch)"
  cat >"$d/modules/m/tests/test_x.py" <<'EOF'
def test_writer():
    assert processed == 3 or failed > 0
EOF
  local r; r="$(run_lint "$d")"
  case "$r" in
    10\|*test_x.py:2\ \[DA-001\]*) ok t01 ;;
    *) fail t01 "expected exit 10 with DA-001 at test_x.py:2, got: $r" ;;
  esac
}

# --- t02 / AC2 — DA-001 does NOT fire on the string that only CONTAINS "or" ----------------
# The false positive a grep implementation cannot avoid. This arm is why the detector parses.
t02_da001_ignores_or_inside_a_string() {
  local d; d="$(new_scratch)"
  cat >"$d/modules/m/tests/test_x.py" <<'EOF'
def test_sources():
    assert sources == {"dropbox-or-gdrive", "syncthing"}


def test_message_mentions_or():
    assert types, "FM-108 row not found or admits no types"
EOF
  local r; r="$(run_lint "$d")"
  case "$r" in
    0\|*) ok t02 ;;
    *) fail t02 "grep-shaped false positive: a string containing 'or' was flagged: $r" ;;
  esac
}

# --- t03 / AC3 — DA-001 catches a disjunction split across lines ---------------------------
# The false NEGATIVE a line-oriented grep cannot avoid. Same defect, opposite direction to t02.
t03_da001_catches_multiline_disjunction() {
  local d; d="$(new_scratch)"
  cat >"$d/modules/m/tests/test_x.py" <<'EOF'
def test_acl():
    assert not check(store, actor="alice").allowed \
        or check(store, actor="alice").mode == "read"
EOF
  local r; r="$(run_lint "$d")"
  case "$r" in
    10\|*DA-001*) ok t03 ;;
    *) fail t03 "a line-continued disjunction escaped the detector: $r" ;;
  esac
}

# --- t04 / AC4 — DA-002 flags statically-true assertions, and spares `assert False` --------
# `assert False` is an intentional unreachable marker, not a defensive assertion. A rule that
# cannot tell them apart would push authors to delete their unreachable markers.
t04_da002_vacuous_only() {
  local d; d="$(new_scratch)"
  cat >"$d/modules/m/tests/test_x.py" <<'EOF'
def test_a():
    assert True


def test_b():
    assert (result.ok, "result was not ok")


def test_c():
    assert len(rows) >= 0
EOF
  local r n; r="$(run_lint "$d")"
  n="$(printf '%s' "$r" | grep -o 'DA-002' | wc -l | tr -d ' ')"
  local d2; d2="$(new_scratch)"
  cat >"$d2/modules/m/tests/test_y.py" <<'EOF'
def test_unreachable():
    if never():
        assert False, "unreachable"
EOF
  local r2; r2="$(run_lint "$d2")"
  if [ "$n" = 3 ] && [ "${r2%%|*}" = 0 ]; then ok t04
  else fail t04 "expected 3 DA-002 hits and a clean `assert False` file, got n=$n and: $r2"; fi
}

# --- t05 / AC5 — DA-003 flags a swallowed probe, spares an output capture ------------------
# `n="$(grep -c x f)" || true` legitimately discards a status (grep exits 1 on zero matches)
# and is not an assertion. `grep -q x f || true` is an assertion with its verdict thrown away.
t05_da003_swallowed_probe() {
  local d; d="$(new_scratch)"
  cat >"$d/scripts/tests/test_a.sh" <<'EOF'
#!/usr/bin/env bash
grep -q needle "$f" || true
EOF
  local r; r="$(run_lint "$d")"
  local d2; d2="$(new_scratch)"
  cat >"$d2/scripts/tests/test_b.sh" <<'EOF'
#!/usr/bin/env bash
n="$(git diff -U0 HEAD -- "$f" | grep -c '^-[^-]')" || true
unset GITHUB_REF_NAME GITHUB_REF TAG || true
EOF
  local r2; r2="$(run_lint "$d2")"
  case "$r|$r2" in
    10\|*DA-003*\|0\|*) ok t05 ;;
    *) fail t05 "expected flag on the bare probe and silence on the capture, got: $r AND $r2" ;;
  esac
}

# --- t06 / AC6 — DA-004 flags a count comparison that holds at zero ------------------------
# AUTH-005 #6's shape: `n <= 1` is satisfied at n=0 — a test that passes when nothing happened.
t06_da004_vacuous_numeric() {
  local d; d="$(new_scratch)"
  cat >"$d/tools/install/tests/test_c.sh" <<'EOF'
#!/usr/bin/env bash
[ "$rows" -ge 0 ] || fail t01 "no rows"
EOF
  local r; r="$(run_lint "$d")"
  case "$r" in
    10\|*DA-004*) ok t06 ;;
    *) fail t06 "expected DA-004 on `-ge 0`, got: $r" ;;
  esac
}

# --- t07 / AC7 — a waiver suppresses ONLY when it carries a reason -------------------------
# A waiver with no reason is the defect wearing the fix's clothes, so it is its own finding.
t07_waiver_requires_a_reason() {
  local d; d="$(new_scratch)"
  cat >"$d/modules/m/tests/test_x.py" <<'EOF'
def test_ok():
    # defensive-assert-ok: upstream returns either code until v2 lands
    assert a == 1 or b == 2
EOF
  local r; r="$(run_lint "$d")"
  local d2; d2="$(new_scratch)"
  cat >"$d2/modules/m/tests/test_y.py" <<'EOF'
def test_bad():
    assert a == 1 or b == 2  # defensive-assert-ok: meh
EOF
  local r2; r2="$(run_lint "$d2")"
  case "$r|$r2" in
    0\|*waived*\|10\|*DA-005*) ok t07 ;;
    *) fail t07 "expected reasoned waiver clean + bare waiver DA-005, got: $r AND $r2" ;;
  esac
}

# --- t08 / AC8 — the LIVE corpus is clean --------------------------------------------------
# The gate itself. t01–t07 prove the lint can fail; this proves the repo currently passes it.
t08_live_corpus_clean() {
  local out rc
  out="$(bash "$LINT" 2>&1)"; rc=$?
  if [ "$rc" -eq 0 ]; then ok t08
  else fail t08 "live corpus has defensive assertions (exit $rc): $(printf '%s' "$out" | head -4 | tr '\n' ' ')"; fi
}

# --- t09 / AC9 — the lint's shell roots are run_all.sh's globbed roots ---------------------
# Two places that must agree is the failure mode run_all.sh's own header warns about. A suite
# the runner reaches but the lint does not is gated and unaudited; the reverse is a dead scan.
t09_roots_match_runner() {
  local runner="$repo/scripts/tests/run_all.sh" missing=""
  [ -f "$runner" ] || { fail t09 "missing run_all.sh"; return; }
  local roots; roots="$(sed -n 's|.*for t in \(.*\); do.*|\1|p' "$runner" | tr ' ' '\n' \
    | sed -n 's|^\(.*\)/test_\*\.sh$|\1|p')"
  [ -n "$roots" ] || { fail t09 "could not extract the runner's globbed roots"; return; }
  local r
  for r in $roots; do
    grep -q "\"$r\"" "$LINT" || missing="$missing $r"
  done
  if [ -z "$missing" ]; then ok t09
  else fail t09 "run_all.sh globs roots the lint does not scan:$missing"; fi
}

# --- t10 / AC10 — RUBRIC.md carries TRACE-008 and its substance ----------------------------
# The review-rule half of R13. Presence of the token alone would be exactly the weaker-than-
# the-clause assertion TRACE-006 forbids, so the defining parts are named individually.
t10_rubric_carries_trace_008() {
  local rubric="$repo/modules/skill/task-audit/RUBRIC.md" body missing=""
  [ -f "$rubric" ] || { fail t10 "missing RUBRIC.md"; return; }
  grep -q '`TRACE-008`' "$rubric" || { fail t10 "no TRACE-008 row in the §9 rule table"; return; }
  body="$(awk '/^### TRACE-008/{f=1} f&&/^---$/{exit} f&&/^## /{exit} f{print}' "$rubric")"
  [ -n "$body" ] || { fail t10 "no ### TRACE-008 subsection"; return; }
  local needle
  for needle in "cannot fail" "check_defensive_asserts.sh" "TRACE-006" "R13"; do
    grep -qF "$needle" <<<"$body" || missing="$missing '$needle'"
  done
  if [ -z "$missing" ]; then ok t10
  else fail t10 "TRACE-008 subsection does not name:$missing"; fi
}

t01_da001_flags_disjunction
t02_da001_ignores_or_inside_a_string
t03_da001_catches_multiline_disjunction
t04_da002_vacuous_only
t05_da003_swallowed_probe
t06_da004_vacuous_numeric
t07_waiver_requires_a_reason
t08_live_corpus_clean
t09_roots_match_runner
t10_rubric_carries_trace_008

echo "  pass=$PASS fail=$FAIL"
[ "$FAIL" -eq 0 ]
