#!/usr/bin/env bash
# test_ci_truth.sh — TASK-IMP-136 §1.5: the CI-truth regrowth guard.
#
# Spec name: scripts/tests/test_ci_truth.sh (TASK-IMP-136). Mid-wave it landed as
# test_benchmark_ci_truth.sh under the batch/8 ownership partition; the final sequential
# pass renamed it here. Test ids t01–t06 are unchanged.
#
# The four §1.5 truths, asserted OFFLINE (no network, no builds), each as a function
# parameterized by root so t05 can prove the negative on scratch fixtures:
#   (a) some root workflow invokes the CAF eval validator (validate.py --all)
#   (b) some root workflow invokes caf_precommit_check.sh
#   (c) NO file under .github/workflows/ carries the stub placeholder marker
#       ('Stub — see task specs' / 'Stub - see task specs') — the regrowth guard
#   (d) .pre-commit-config.yaml is absent OR every entry: it declares is also invoked
#       from .githooks/pre-commit (no split-brain hook story)
#
#   t01_caf_gate_wired            (AC 1) workflow exists, names both commands, both trigger
#        paths, a schedule, and explicit read-only permissions; negative: the validator
#        DISCRIMINATES — a pristine fixture validates clean, a corrupted scratch copy of the
#        same fixture exits non-zero. (--run on one fixture, not --all: --all is already red
#        at HEAD on the pre-existing B17/B18 regressions — recorded in the task evidence —
#        so an --all-based negative would be vacuous.)
#   t02_awh_hook_wired_safely     (AC 2) the hook invokes awh-gate.sh via the matches()
#        herestring idiom, no `git diff --cached | grep -q` pipeline form exists outside
#        comments, and the trigger regex itself rejects unrelated paths.
#   t03_dead_config_gone          (AC 3) assert (d) + the config file is gone + CHANGELOG
#        names the removal.
#   t04_no_stub_survives          (AC 4) assert (c) + the committed 9-row disposition table
#        names every deleted file and its declaring task.
#   t05_self_test_negative_paths  (AC 5) each of (a)(b)(c)(d) fails on a scratch fixture
#        that breaks exactly its precondition.
#   t06_changelog_records_sweep   (AC 6) CHANGELOG's top entry names the caf-evals gate, the
#        awh hook wiring, the removed config file, and the stub disposition counts.
#
# Registration: run_all.sh's scripts/tests/test_*.sh glob (the glob IS the registration).
set -uo pipefail

here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../.." && pwd)"

TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
case "$TMP/" in "$repo"/*) echo "FATAL scratch $TMP is under the repo $repo"; exit 1 ;; esac

PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

WF_DIR="$repo/.github/workflows"
HOOK="$repo/.githooks/pre-commit"
GATE_WF="$WF_DIR/caf-evals-gate.yml"

# ---- the four §1.5 asserts, root-parameterized so t05 can break each in a scratch -------

# (a) some workflow in $1 invokes the CAF eval validator
assert_a() { grep -rl -- 'validate\.py --all' "$1" >/dev/null 2>&1; }
# (b) some workflow in $1 invokes caf_precommit_check.sh
assert_b() { grep -rl -- 'caf_precommit_check\.sh' "$1" >/dev/null 2>&1; }
# (c) NO file in $1 carries the stub placeholder marker (both dash variants, per spec)
assert_c() {
  local hits
  hits="$(grep -rlE 'Stub (—|-) see task specs' "$1" 2>/dev/null || true)"
  if [ -n "$hits" ]; then
    echo "    stub placeholder marker found in:" >&2
    printf '      %s\n' $hits >&2
    return 1
  fi
  return 0
}
# (d) config $1 absent, or every entry: it declares is invoked from hook $2
assert_d() {
  local cfg="$1" hook="$2" entry base bad=0
  [ -f "$cfg" ] || return 0
  while IFS= read -r entry; do
    [ -n "$entry" ] || continue
    base="$(basename "$entry")"
    if ! grep -qF "$base" "$hook"; then
      echo "    $cfg declares entry '$entry' which $hook never invokes" >&2
      bad=1
    fi
  done < <(sed -n 's/^[[:space:]]*entry:[[:space:]]*//p' "$cfg")
  return "$bad"
}

# ---- AC 1: the CAF gate workflow is wired, and the validator discriminates --------------
t01_caf_gate_wired() {
  local all=1
  [ -f "$GATE_WF" ] || { fail t01_caf_gate_wired "$GATE_WF missing — expected the TASK-IMP-136 root CAF gate workflow"; return; }
  assert_a "$WF_DIR" || { fail t01_caf_gate_wired "no root workflow invokes 'validate.py --all' (expected in $GATE_WF)"; all=0; }
  assert_b "$WF_DIR" || { fail t01_caf_gate_wired "no root workflow invokes 'caf_precommit_check.sh' (expected in $GATE_WF)"; all=0; }
  grep -qF -- "tools/caf/**" "$GATE_WF" || { fail t01_caf_gate_wired "$GATE_WF lacks the 'tools/caf/**' trigger path (spec §1.1)"; all=0; }
  grep -qF -- "scripts/caf_*" "$GATE_WF" || { fail t01_caf_gate_wired "$GATE_WF lacks the 'scripts/caf_*' trigger path (spec §1.1)"; all=0; }
  grep -qE '^[[:space:]]*schedule:' "$GATE_WF" || { fail t01_caf_gate_wired "$GATE_WF lacks a schedule: trigger (spec §1.1 weekly drift net)"; all=0; }
  grep -qE '^[[:space:]]*contents:[[:space:]]*read' "$GATE_WF" || { fail t01_caf_gate_wired "$GATE_WF lacks explicit 'permissions: contents: read' (spec §3 security-class)"; all=0; }

  # negative: the validator DISCRIMINATES between a clean and a corrupted fixture.
  if command -v python3 >/dev/null 2>&1; then
    local fx="$repo/tools/caf/core/evals/fixtures/G01-clean-run"
    if [ -d "$fx" ]; then
      ( cd "$repo/tools/caf" && python3 core/evals/validate.py --run "$fx" >/dev/null 2>&1 ) \
        || { fail t01_caf_gate_wired "pristine fixture G01-clean-run did not validate clean — the negative below would be vacuous"; all=0; }
      cp -R "$fx" "$TMP/broken-fixture"
      # corrupt the run: a task status outside the closed set is the R5 violation class
      # (the B05-bad-status fixture's shape) — verified to gate with R5-BAD-STATUS.
      sed 's/| DONE |/| ALMOST_DONE |/' "$TMP/broken-fixture/docs/BACKLOG.md" > "$TMP/broken-fixture/docs/BACKLOG.md.tmp" \
        && mv "$TMP/broken-fixture/docs/BACKLOG.md.tmp" "$TMP/broken-fixture/docs/BACKLOG.md"
      if ( cd "$repo/tools/caf" && python3 core/evals/validate.py --run "$TMP/broken-fixture" >/dev/null 2>&1 ); then
        fail t01_caf_gate_wired "validator passed a deliberately corrupted copy of G01-clean-run (status ALMOST_DONE should raise R5-BAD-STATUS) — the gate cannot discriminate"; all=0
      fi
    else
      echo "  defer t01 negative — fixture $fx not found (name the expectation: tools/caf/core/evals/fixtures/G01-clean-run must exist)"
    fi
  else
    echo "  defer t01 negative — python3 not on PATH (the validator is python)"
  fi
  [ "$all" -eq 1 ] && ok t01_caf_gate_wired
}

# ---- AC 2: awh hook wiring uses the safe idiom and scopes to module sources -------------
t02_awh_hook_wired_safely() {
  local all=1
  [ -f "$HOOK" ] || { fail t02_awh_hook_wired_safely "$HOOK missing"; return; }
  grep -qF 'awh-gate.sh' "$HOOK" || { fail t02_awh_hook_wired_safely "$HOOK never invokes .pre-commit-hooks/awh-gate.sh (spec §1.2)"; all=0; }
  grep -qE '^matches\(\)' "$HOOK" || { fail t02_awh_hook_wired_safely "$HOOK lost its matches() herestring helper"; all=0; }
  grep -qE 'matches "\$awh_trigger"' "$HOOK" || { fail t02_awh_hook_wired_safely "the awh block does not gate via matches() on \$awh_trigger (spec §1.2 mandates the herestring idiom)"; all=0; }
  # The forbidden pipeline form must not exist as CODE (the hook's own header documents it
  # as a warning — comment lines are excluded from the scan).
  if grep -vE '^[[:space:]]*#' "$HOOK" | grep -qE 'git diff --cached --name-only \| grep'; then
    fail t02_awh_hook_wired_safely "$HOOK pipes 'git diff --cached --name-only' straight into grep — the SIGPIPE pitfall the hook header forbids"; all=0
  fi
  # the trigger regex itself: unrelated staged paths must not match, module sources must.
  local trig
  trig="$(sed -n "s/^awh_trigger='\(.*\)'$/\1/p" "$HOOK" | head -1)"
  if [ -z "$trig" ]; then
    fail t02_awh_hook_wired_safely "could not extract awh_trigger from $HOOK (expected awh_trigger='<regex>')"; all=0
  else
    grep -qE "$trig" <<<"README.md" && { fail t02_awh_hook_wired_safely "awh_trigger '$trig' matches the unrelated path README.md — the gate would fire on every commit"; all=0; }
    grep -qE "$trig" <<<"modules/skill/SKILL.md" || { fail t02_awh_hook_wired_safely "awh_trigger '$trig' does not match module source modules/skill/SKILL.md"; all=0; }
  fi
  [ "$all" -eq 1 ] && ok t02_awh_hook_wired_safely
}

# ---- AC 3: the dead framework config is gone and the removal is recorded ----------------
t03_dead_config_gone() {
  local all=1 cfg="$repo/.pre-commit-config.yaml"
  if [ -f "$cfg" ]; then
    fail t03_dead_config_gone ".pre-commit-config.yaml still exists — spec §1.3 removes it once the awh hook wiring lands (operator veto fallback: a non-authoritative header, which this file also lacks)"; all=0
  fi
  assert_d "$cfg" "$HOOK" || { fail t03_dead_config_gone "assert (d): a declared hook entry is not invoked by .githooks/pre-commit"; all=0; }
  # top CHANGELOG entry = first "## [" block
  local top
  # Scan every versioned ## […] section — top entry moves with each cut (same class as CUO doctrine pin).
  top="$(awk '/^## \[/{p=1} p' "$repo/CHANGELOG.md")"
  grep -qF '.pre-commit-config.yaml' <<<"$top" \
    || { fail t03_dead_config_gone "CHANGELOG.md versioned entry does not name the .pre-commit-config.yaml removal — paste the prepared entry from docs/tasks/improvement/TASK-IMP-136-ci-caf-evals-and-stub-truth/implementation-evidence.md"; all=0; }
  [ "$all" -eq 1 ] && ok t03_dead_config_gone
}

# ---- AC 4: zero stubs survive, and the disposition table records every judgment ---------
t04_no_stub_survives() {
  local all=1
  assert_c "$WF_DIR" || { fail t04_no_stub_survives "a file under .github/workflows/ carries the stub placeholder marker (listed above) — the honest states are a real gate or no gate"; all=0; }
  local table="$repo/docs/tasks/improvement/TASK-IMP-136-ci-caf-evals-and-stub-truth/stub-disposition.md"
  if [ ! -f "$table" ]; then
    fail t04_no_stub_survives "disposition table missing at $table (spec §1.4: the 9-row table ships with the PR)"; all=0
  else
    local wf declarer missing=0
    for wf in cache-isolation-gate.yml:TASK-AI-018 memory-rebuild.yml:TASK-MEMORY-102 \
              obs-correlation-gate.yml:TASK-OBS-005 proj-a11y-gate.yml:TASK-PROJ-018 \
              proj-storybook-chromatic.yml:TASK-PROJ-018 rew-memory-exclusion.yml:TASK-REW-010 \
              vn-pii-quarterly-refresh.yml:TASK-AI-013 vn-pii-recall.yml:TASK-AI-013 \
              zdr-staleness-check.yml:TASK-AI-015; do
      declarer="${wf#*:}"; wf="${wf%%:*}"
      # Avoid echo|grep -q / grep|grep -q under pipefail (SIGPIPE false fail).
      row="$(grep -F "$wf" "$table" || true)"
      grep -qF "$wf" <<<"$row" && grep -qF "$declarer" <<<"$row" \
        || { echo "    table row missing or unnamed declarer: $wf (expected $declarer)" >&2; missing=1; }
    done
    [ "$missing" -eq 0 ] || { fail t04_no_stub_survives "disposition table does not name every file + declaring task (see above)"; all=0; }
  fi
  [ "$all" -eq 1 ] && ok t04_no_stub_survives
}

# ---- AC 5: each assert fails when its precondition is broken in a scratch copy ----------
t05_self_test_negative_paths() {
  local all=1 d="$TMP/selftest"
  mkdir -p "$d/workflows-empty" "$d/workflows-stub"
  printf 'name: x\non: push\njobs: {}\n' > "$d/workflows-empty/x.yml"
  # (a) and (b): a workflows dir naming neither command
  assert_a "$d/workflows-empty" && { fail t05_self_test_negative_paths "assert (a) passed on a workflows dir that never names validate.py --all"; all=0; }
  assert_b "$d/workflows-empty" && { fail t05_self_test_negative_paths "assert (b) passed on a workflows dir that never names caf_precommit_check.sh"; all=0; }
  # (c): a resurrected stub
  printf "name: ghost\njobs:\n  p:\n    steps:\n      - run: echo 'Stub — see task specs for canonical workflow YAML'\n" > "$d/workflows-stub/ghost.yml"
  assert_c "$d/workflows-stub" 2>/dev/null && { fail t05_self_test_negative_paths "assert (c) passed on a workflows dir carrying the stub placeholder marker"; all=0; }
  # (d): a framework config declaring an entry the hook never invokes
  printf -- "- repo: local\n  hooks:\n    - id: ghost\n      entry: .pre-commit-hooks/ghost-gate.sh\n" > "$d/config.yaml"
  printf '#!/usr/bin/env bash\ntrue\n' > "$d/hook"
  assert_d "$d/config.yaml" "$d/hook" 2>/dev/null && { fail t05_self_test_negative_paths "assert (d) passed on a config whose entry the hook never invokes"; all=0; }
  # construction check: the same asserts PASS on the real repo surfaces they gate — except
  # (c)/(d)-adjacent failures already reported by t01–t04, so only re-run the pure pair.
  assert_c "$WF_DIR" >/dev/null 2>&1 || { fail t05_self_test_negative_paths "assert (c) fails on the real workflows dir — negatives above prove nothing while the positive is red"; all=0; }
  [ "$all" -eq 1 ] && ok t05_self_test_negative_paths
}

# ---- AC 6: the CHANGELOG records the sweep -----------------------------------------------
t06_changelog_records_sweep() {
  local all=1 top
  # Scan every versioned ## […] section — top entry moves with each cut (same class as CUO doctrine pin).
  top="$(awk '/^## \[/{p=1} p' "$repo/CHANGELOG.md")"
  local want
  for want in 'caf-evals-gate' 'awh' '.pre-commit-config.yaml' '9 deleted'; do
    grep -qF "$want" <<<"$top" \
      || { fail t06_changelog_records_sweep "CHANGELOG.md versioned entry lacks '$want' — paste the prepared entry from docs/tasks/improvement/TASK-IMP-136-ci-caf-evals-and-stub-truth/implementation-evidence.md"; all=0; }
  done
  [ "$all" -eq 1 ] && ok t06_changelog_records_sweep
}

t01_caf_gate_wired
t02_awh_hook_wired_safely
t03_dead_config_gone
t04_no_stub_survives
t05_self_test_negative_paths
t06_changelog_records_sweep

echo "benchmark-ci-truth: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]
