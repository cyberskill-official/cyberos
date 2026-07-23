#!/usr/bin/env bash
# test_skill_stub_lint.sh - TASK-SKILL-202 skill-quality-floor suite (§1 #5/#6, ACs 3-6).
#
# The floor keys on PROPERTIES of a stub, never on a name or a count:
#
#   SIZE       total lines across the skill's contract surface (SKILL.md + the
#              TASK-SKILL-118 class files: PIPELINE/INVARIANTS/RUBRIC/AUDIT_LOOP/
#              REPORT_FORMAT + envelopes/*.json + references/*.md + acceptance/*.md)
#              must be >= FLOOR_LINES.
#   STRUCTURE  SKILL.md body (below frontmatter) must carry >= FLOOR_SECTIONS
#              `## ` headings - a contract has sections; a name reservation has prose.
#
# Calibration, measured 2026-07-23 against the vendored set (56 dirs): the four NFR
# stubs measure 20-22 total lines / 0 body sections; the smallest REAL vendored skill
# (task-reconcile) measures 163 lines / 5 sections, smallest section count 2
# (architectural-spike-audit). FLOOR_LINES=60 / FLOOR_SECTIONS=2 sit far above any
# stub and far below any real skill - the spec's calibration claim, made true against
# the measured corpus. NOTE the spec's literal wording ("60 non-frontmatter lines" of
# SKILL.md alone) mis-measures reality - 10 real audit-side skills carry 12-39 body
# lines with their contract mass in RUBRIC/AUDIT_LOOP/etc - so the floor counts the
# skill's whole contract surface instead; deviation recorded in the task's
# implementation-evidence.md for the review gate.
#
# Placeholder-SYNTAX detection stays with the TASK-SKILL-115 sweep tooling (spec §1 #5);
# this floor is size + structure only.
#
# The vendored set is parsed from build.sh's VENDORED_SKILLS heredoc, so the suite
# needs no payload build (siblings own dist/) and follows the list as it changes.
# While the four NFR stubs remain in VENDORED_SKILLS, t04/t05 fail LOUDLY naming them -
# that is the finding (H7), not a suite defect; they go green when the TASK-SKILL-202
# delist lands in build.sh.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../.." && pwd)"
SKILLS="$repo/modules/skill"
BUILD="$repo/tools/install/build.sh"
PARITY="$repo/tools/install/check-pair-parity.sh"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

FLOOR_LINES=60
FLOOR_SECTIONS=2
# Reviewed exemption path (spec §3 edge case): a legitimately-small future skill is
# added HERE, by name, in a reviewed change. Starts empty by design.
FLOOR_EXEMPT=""

# ── the floor detector ───────────────────────────────────────────────
# floor_check <skill-dir>  -> exit 0 clean | exit 10 + one "FLOOR <name>: ..." line
# per violation. Distinct from exit 2 (unusable dir), mirroring check-pair-parity.sh.
floor_check() {
  local d="$1" name; name="$(basename "$1")"; local rc=0
  [ -d "$d" ] || { echo "FLOOR $name: skill dir missing under modules/skill/"; return 2; }
  [ -f "$d/SKILL.md" ] || { echo "FLOOR $name: no SKILL.md"; return 10; }
  local size sections
  size=$(cat "$d/SKILL.md" "$d/PIPELINE.md" "$d/INVARIANTS.md" "$d/RUBRIC.md" \
             "$d/AUDIT_LOOP.md" "$d/REPORT_FORMAT.md" "$d"/envelopes/*.json \
             "$d"/references/*.md "$d"/acceptance/*.md 2>/dev/null | wc -l | tr -d ' ')
  sections=$(awk 'BEGIN{c=0} /^---$/{c++; next} c>=2 && /^## /{n++} END{print n+0}' "$d/SKILL.md")
  if [ "$size" -lt "$FLOOR_LINES" ]; then
    echo "FLOOR $name: contract surface is $size lines (< $FLOOR_LINES) - a name reservation, not a skill"; rc=10
  fi
  if [ "$sections" -lt "$FLOOR_SECTIONS" ]; then
    echo "FLOOR $name: SKILL.md body has $sections '## ' sections (< $FLOOR_SECTIONS) - no contract structure"; rc=10
  fi
  return "$rc"
}

# ── shared inputs ────────────────────────────────────────────────────
VENDORED="$(sed -n "/<<'VENDORED_SKILLS'/,/^VENDORED_SKILLS\$/p" "$BUILD" \
  | sed '1d;$d' | sed 's/#.*//' | tr -d ' \t' | grep -v '^$')"
[ -n "$VENDORED" ] || { echo "  FAIL setup: could not parse VENDORED_SKILLS from $BUILD"; exit 2; }

# The 20 skills TASK-SKILL-202 §1 #3 / AC 3 enumerates (the injection-discipline
# backport set - the 24 measured gaps minus the four NFR stubs it delists instead).
BACKPORT_20="plan-author plan-audit task-reconcile workflow-improver
architectural-spike-author architectural-spike-audit
repo-context-map-author repo-context-map-audit
edge-case-matrix-author edge-case-matrix-audit
mock-contract-test-author mock-contract-test-audit
observability-injection-author observability-injection-audit
backlog-state-update-author backlog-state-update-audit
coverage-gate-author coverage-gate-audit
debugging-cycle-author debugging-cycle-audit"

# ── t01: detector passes a real-shaped fixture ───────────────────────
t01_detector_passes_real_shape() {
  local d="$TMP/fixtures/good-skill"; mkdir -p "$d/references" "$d/acceptance" "$d/envelopes"
  { echo "---"; echo "name: good-skill"; echo "description: fixture"; echo "---"; echo
    echo "# good-skill"
    for i in 1 2 3; do
      echo; echo "## Section $i"; echo
      for j in $(seq 1 15); do echo "Line $j of section $i - real contract prose."; done
    done
  } > "$d/SKILL.md"
  echo '{}' > "$d/envelopes/input.json"
  out="$(floor_check "$d")"; rc=$?
  [ "$rc" -eq 0 ] && [ -z "$out" ] && ok t01 || fail t01 "rc=$rc out=$out"
}

# ── t02: detector fails a stub fixture (small AND structureless) ─────
t02_detector_fails_stub() {
  local d="$TMP/fixtures/stub-skill"; mkdir -p "$d"
  { echo "---"; echo "name: stub-skill"; echo "description: fixture stub"; echo "---"; echo
    echo "# stub-skill"; echo; echo "One paragraph of intent. Not a contract."
  } > "$d/SKILL.md"
  out="$(floor_check "$d")"; rc=$?
  [ "$rc" -eq 10 ] && grep -q "FLOOR stub-skill: contract surface is" <<<"$out" \
    && grep -q "FLOOR stub-skill: SKILL.md body has 0" <<<"$out" \
    && ok t02 || fail t02 "rc=$rc out=$out"
}

# ── t03: detector fails a padded fixture missing the sections ────────
t03_detector_fails_missing_sections() {
  local d="$TMP/fixtures/padded-skill"; mkdir -p "$d"
  { echo "---"; echo "name: padded-skill"; echo "description: fixture"; echo "---"; echo
    echo "# padded-skill"; echo
    for j in $(seq 1 80); do echo "Padding line $j - size alone must not satisfy the floor."; done
  } > "$d/SKILL.md"
  out="$(floor_check "$d")"; rc=$?
  [ "$rc" -eq 10 ] && grep -q "sections" <<<"$out" && ! grep -q "contract surface is" <<<"$out" \
    && ok t03 || fail t03 "rc=$rc out=$out (size half must pass, structure half must fail)"
}

# ── t04: every vendored skill meets the floor in the source tree ─────
t04_vendored_set_meets_floor() {
  local bad=""
  for s in $VENDORED; do
    case " $FLOOR_EXEMPT " in *" $s "*) continue ;; esac
    floor_check "$SKILLS/$s" >"$TMP/floor.$s" 2>&1 || bad="$bad $s"
  done
  if [ -z "$bad" ]; then ok t04; else
    for s in $bad; do sed 's/^/         /' "$TMP/floor.$s"; done
    case "$bad" in
      *nfr-*) echo "         ^ nfr-* are the TASK-SKILL-202 H7 stubs: the delist (drop 4 names from build.sh VENDORED_SKILLS + 4 chain-allowlist.txt lines) is a FINAL-PASS item - this failure clears when it lands." ;;
    esac
    fail t04 "vendored skills under the floor:$bad"
  fi
}

# ── t05: every vendored skill carries both injection-discipline halves ─
t05_injection_discipline_present() {
  local bad=""
  for s in $VENDORED; do
    local f="$SKILLS/$s/SKILL.md" fm
    fm="$(awk '/^---$/{c++; next} c==1{print} c>=2{exit}' "$f" 2>/dev/null)"
    grep -q '^untrusted_inputs:' <<<"$fm" \
      && grep -q 'wrap_in_marker: "untrusted_content"' <<<"$fm" \
      && grep -q 'injection_scan: required' <<<"$fm" \
      && grep -q 'on_marker_hit: surface_to_human' <<<"$fm" \
      && [ -s "$SKILLS/$s/references/UNTRUSTED_CONTENT.md" ] || bad="$bad $s"
  done
  if [ -z "$bad" ]; then ok t05; else
    case "$bad" in
      *nfr-*) echo "         nfr-* stubs are delisted by TASK-SKILL-202 (final-pass build.sh item), not backported - this failure clears when the delist lands." ;;
    esac
    fail t05 "missing untrusted_inputs frontmatter and/or references/UNTRUSTED_CONTENT.md:$bad"
  fi
}

# ── t06: the 20 backported docs are per-skill, not byte-copies ───────
t06_backport_is_per_skill() {
  local bad="" seen=""
  for s in $BACKPORT_20; do
    local doc="$SKILLS/$s/references/UNTRUSTED_CONTENT.md"
    [ -s "$doc" ] || { bad="$bad $s(missing)"; continue; }
    grep -q "$s" "$doc" || bad="$bad $s(does-not-name-itself)"
    grep -q '§0' "$doc" || bad="$bad $s(no-input-surface-section)"
    for t in $seen; do
      cmp -s "$doc" "$SKILLS/$t/references/UNTRUSTED_CONTENT.md" && bad="$bad $s==$t(byte-copy)"
    done
    seen="$seen $s"
  done
  [ -z "$bad" ] && ok t06 || fail t06 "copy-paste compliance detected:$bad"
}

# ── t07: parity SCOPE == the vendored pair set (both directions) ─────
t07_parity_scope_complete() {
  local pairs="" scope missing="" extra=""
  for s in $VENDORED; do
    case "$s" in
      *-author)
        base="${s%-author}"
        grep -qx "$base-audit" <<<"$VENDORED" && pairs="$pairs $base" ;;
    esac
  done
  scope="$(sed -n '/^SCOPE=(/,/)/p' "$PARITY" | tr '()' '  ' | sed 's/^SCOPE=//' | tr ' ' '\n' | grep -v '^$')"
  for p in $pairs; do grep -qx "$p" <<<"$scope" || missing="$missing $p"; done
  for p in $scope; do
    case " $pairs " in *" $p "*) ;; *) extra="$extra $p" ;; esac
  done
  [ -z "$missing" ] && [ -z "$extra" ] && ok t07 \
    || fail t07 "SCOPE drift - vendored pairs not in SCOPE:${missing:- none}; SCOPE names not vendored pairs:${extra:- none}"
}

t01_detector_passes_real_shape; t02_detector_fails_stub; t03_detector_fails_missing_sections
t04_vendored_set_meets_floor; t05_injection_discipline_present
t06_backport_is_per_skill; t07_parity_scope_complete
echo "----"; echo "pass=$PASS fail=$FAIL"; [ "$FAIL" -eq 0 ]
