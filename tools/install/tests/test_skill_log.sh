#!/usr/bin/env bash
# test_skill_log.sh — TASK-IMP-113 §1 suite (t01-t04 -> AC1-AC4).
#
# skill-log records an audit verdict and renders the aggregate; the ledger is append-only and the
# tier is a REPORT, never a gate. Each test bends exactly one thing and fails if its clause breaks.
#
#   t01 -> clause 1.1 / 1.3 / AC1   append pass/fail rows; --render shows correct runs, passes, rate,
#                                   and the (informational) tier label
#   t02 -> clause 1.2 / AC2         append is append-only: a second append never rewrites row 1
#   t03 -> clause 1.5 / AC3         a ZERO-run skill renders 'no data', NOT '0%'; a measured skill
#                                   with 0 passes renders a real '0.0%' (the two are different facts)
#   t04 -> clause 1.6 / AC4         the install.sh seed gains 'skill-trust.tsv' append-once (no
#                                   duplicate on re-install) and git check-ignore of the ledger
#                                   exits 0 after the seed (and did NOT before — 1.6's premise)
#
# run_all discovers this file via its tools/install/tests/test_*.sh glob. Node stdlib only.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
MJS="$repo/tools/install/docs-tools/skill-log.mjs"
INSTALL="$repo/tools/install/install.sh"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

# a fresh git repo with the .workflow dir the ledger lives under
scratch() { local d="$1"; mkdir -p "$d/docs/tasks/.workflow"; ( cd "$d" && git init -q . && git config user.email t@t && git config user.name t ) >/dev/null 2>&1; }
app() { node "$MJS" append --skill "$1" --verdict "$2" --task "$3" --at "2026-07-19T10:00:00Z" --repo "$4" >/dev/null 2>&1; }

# ── t01: append then render shows correct runs / passes / rate / tier ──────────
t01_append_and_render() {
  local d="$TMP/t01"; scratch "$d"
  # alpha: 3 pass + 1 fail = 4 runs, 3 passes, 75.0%, watch (runs<10)
  app alpha pass A1 "$d"; app alpha pass A2 "$d"; app alpha pass A3 "$d"; app alpha fail A4 "$d"
  # beta: 1 pass = 1 run, 100.0%, watch (runs<10)
  app beta pass B1 "$d"
  # gamma: 20 pass = 20 runs, 100.0%, auto (>=20 runs & >=95%) — proves the informational tier
  local i; for i in $(seq 1 20); do app gamma pass "G$i" "$d"; done
  local out; out="$(node "$MJS" --render --repo "$d" 2>/dev/null)"; local rc=$?
  [ "$rc" -eq 0 ] || { fail t01_append_and_render "render exited $rc"; return; }
  echo "$out" | grep -qE '^[[:space:]]*alpha[[:space:]]+4[[:space:]]+3[[:space:]]+75\.0%[[:space:]]+watch$' \
    || { fail t01_append_and_render "alpha row wrong (want runs=4 passes=3 75.0% watch): $out"; return; }
  echo "$out" | grep -qE '^[[:space:]]*beta[[:space:]]+1[[:space:]]+1[[:space:]]+100\.0%[[:space:]]+watch$' \
    || { fail t01_append_and_render "beta row wrong (want 1/1 100.0% watch)"; return; }
  echo "$out" | grep -qE '^[[:space:]]*gamma[[:space:]]+20[[:space:]]+20[[:space:]]+100\.0%[[:space:]]+auto$' \
    || { fail t01_append_and_render "gamma row wrong (want 20/20 100.0% auto)"; return; }
  ok t01_append_and_render
}

# ── t02: append-only — a second append never rewrites row 1 (1.2) ──────────────
t02_append_only() {
  local d="$TMP/t02"; scratch "$d"; local L="$d/docs/tasks/.workflow/skill-trust.tsv"
  app one pass T1 "$d"
  local row1_before; row1_before="$(sed -n '1p' "$L")"
  local sha_before; sha_before="$(sha256sum "$L" | cut -d' ' -f1)"
  app one fail T2 "$d"                                   # a second, DIFFERENT verdict
  app two pass T3 "$d"                                   # and a third for another skill
  local row1_after; row1_after="$(sed -n '1p' "$L")"
  [ "$row1_before" = "$row1_after" ] || { fail t02_append_only "row 1 changed after later appends"; return; }
  # the file GREW (append), the old bytes are an untouched prefix, and lines == 3
  [ "$(wc -l < "$L" | tr -d ' ')" = "3" ] || { fail t02_append_only "expected 3 rows after 3 appends"; return; }
  local sha_prefix; sha_prefix="$(head -c "$(printf '%s\n' "$row1_before" | wc -c)" "$L" | sha256sum | cut -d' ' -f1)"
  local sha_expect; sha_expect="$(printf '%s\n' "$row1_before" | sha256sum | cut -d' ' -f1)"
  [ "$sha_prefix" = "$sha_expect" ] || { fail t02_append_only "row 1's bytes were rewritten, not preserved as a prefix"; return; }
  ok t02_append_only
}

# ── t03: zero-run => 'no data' (never '0%'); measured-failing => a real '0.0%' (1.5) ──
t03_zero_runs_no_data() {
  local d="$TMP/t03"; scratch "$d"
  app measured pass M1 "$d"; app measured pass M2 "$d"      # measured, passing
  app brokenskill fail X1 "$d"; app brokenskill fail X2 "$d" # measured, 0 passes => real 0.0%
  local out; out="$(node "$MJS" --render --skills unmeasured --repo "$d" 2>/dev/null)"
  # the zero-run skill renders 'no data' and NOT a percentage
  local urow; urow="$(echo "$out" | grep -E '^[[:space:]]*unmeasured[[:space:]]')"
  [ -n "$urow" ] || { fail t03_zero_runs_no_data "unmeasured skill absent from render"; return; }
  echo "$urow" | grep -q 'no data' || { fail t03_zero_runs_no_data "zero-run skill did not render 'no data': $urow"; return; }
  echo "$urow" | grep -qE '[0-9]+\.[0-9]%|[0-9]%' && { fail t03_zero_runs_no_data "zero-run skill rendered a percentage (must be 'no data'): $urow"; return; }
  # the measured-but-failing skill DOES render a real 0.0% (a different fact than 'no data')
  echo "$out" | grep -qE '^[[:space:]]*brokenskill[[:space:]]+2[[:space:]]+0[[:space:]]+0\.0%[[:space:]]+watch$' \
    || { fail t03_zero_runs_no_data "measured-failing skill did not render a real 0.0%: $out"; return; }
  # --json encodes the unmeasured skill's rate as null, never 0
  local rate; rate="$(node "$MJS" --render --skills unmeasured --repo "$d" --json 2>/dev/null \
    | node -e 'const d=JSON.parse(require("fs").readFileSync(0,"utf8"));const s=d.skills.find(x=>x.skill==="unmeasured");process.stdout.write(JSON.stringify(s.rate))')"
  [ "$rate" = "null" ] || { fail t03_zero_runs_no_data "json rate for zero runs was '$rate', want null"; return; }
  ok t03_zero_runs_no_data
}

# ── t04: the install.sh seed gitignores the ledger, append-once (1.6 / AC4) ─────
# Runs the REAL seed block extracted from install.sh (not a replica) against scratch repos.
extract_seed() { awk '/^wf_ignore=/{f=1} f&&/^# 0a\./{f=0} f{print}' "$INSTALL"; }
run_seed() { local d="$1"; local seed; seed="$(extract_seed)"; root="$d" bash -c "set -euo pipefail
mkdir -p \"\$root/docs/tasks/.workflow\"
$seed"; }
t04_ledger_gitignored() {
  # structural bind: install.sh carries the ledger pattern AND 090's append-once guard for it
  grep -qF 'skill-trust.tsv' "$INSTALL" || { fail t04_ledger_gitignored "install.sh has no skill-trust.tsv pattern"; return; }
  grep -qF "grep -qxF 'skill-trust.tsv'" "$INSTALL" || { fail t04_ledger_gitignored "install.sh lacks the append-once guard for skill-trust.tsv"; return; }
  # extraction must be non-empty (guards against the awk anchors drifting)
  [ -n "$(extract_seed)" ] || { fail t04_ledger_gitignored "could not extract the seed block from install.sh"; return; }

  # fresh repo: NOT covered before the seed (1.6's premise), covered after
  local d="$TMP/t04a"; scratch "$d"
  ( cd "$d" && git check-ignore docs/tasks/.workflow/skill-trust.tsv >/dev/null 2>&1 ) \
    && { fail t04_ledger_gitignored "ledger was ALREADY gitignored before the seed — 1.6's premise is false"; return; }
  run_seed "$d" || { fail t04_ledger_gitignored "seed failed on a fresh repo"; return; }
  local gi="$d/docs/tasks/.workflow/.gitignore"
  [ "$(tr '\n' ' ' < "$gi")" = "*.ship.json *.manifest.json skill-trust.tsv " ] \
    || { fail t04_ledger_gitignored "fresh seed did not write the 3 patterns: $(cat "$gi")"; return; }
  ( cd "$d" && git check-ignore docs/tasks/.workflow/skill-trust.tsv >/dev/null 2>&1 ) \
    || { fail t04_ledger_gitignored "git check-ignore of the ledger did not exit 0 after the seed"; return; }
  # re-run the seed (re-install): NO duplicate line
  run_seed "$d" || { fail t04_ledger_gitignored "seed failed on re-run"; return; }
  local n; n="$(grep -cxF 'skill-trust.tsv' "$gi")"
  [ "$n" = "1" ] || { fail t04_ledger_gitignored "re-install duplicated skill-trust.tsv ($n copies)"; return; }

  # legacy repo: a pre-existing 2-pattern seed (predates 1.6) gains the pattern EXACTLY once
  local e="$TMP/t04b"; scratch "$e"; local egi="$e/docs/tasks/.workflow/.gitignore"
  printf '%s\n' '*.ship.json' '*.manifest.json' > "$egi"
  run_seed "$e" || { fail t04_ledger_gitignored "seed failed on a legacy 2-pattern repo"; return; }
  [ "$(grep -cxF 'skill-trust.tsv' "$egi")" = "1" ] || { fail t04_ledger_gitignored "legacy seed did not append skill-trust.tsv exactly once"; return; }
  grep -qxF '*.ship.json' "$egi" && grep -qxF '*.manifest.json' "$egi" \
    || { fail t04_ledger_gitignored "legacy append clobbered the IMP-090 patterns"; return; }
  run_seed "$e" || { fail t04_ledger_gitignored "seed failed on legacy re-run"; return; }
  [ "$(grep -cxF 'skill-trust.tsv' "$egi")" = "1" ] || { fail t04_ledger_gitignored "legacy re-install duplicated skill-trust.tsv"; return; }
  ok t04_ledger_gitignored
}

echo "skill-log suite (TASK-IMP-113):"
t01_append_and_render
t02_append_only
t03_zero_runs_no_data
t04_ledger_gitignored
echo "test_skill_log: pass=$PASS fail=$FAIL"
[ "$FAIL" -eq 0 ] || exit 1
