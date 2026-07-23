#!/usr/bin/env bash
# test_fail_closed_gates.sh - TASK-CUO-302 (t01-t06 -> AC 1-6): the machine-gate floor
# fails CLOSED. An all-empty floor exits 3 (distinct from 1 = gate failed, 2 = bad config);
# CYBEROS_ALLOW_EMPTY_GATES=1 (the literal 1 only) prints a distinct EMPTY-ACKNOWLEDGED
# line; autodetect learns the ordered monorepo fallback (run_all.sh beats Makefile) without
# executing either probe; the gates.env header stops inviting edits; CHANGELOG names the
# breaking change.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

echo "building scratch payload..."
bash "$repo/tools/install/build.sh" "$TMP/payload" >/dev/null 2>&1 || { echo FATAL build; exit 1; }

# The doctor gate (TASK-MEMORY-303 #1.6) probes `python3 -c "import cyberos.core"`. On a
# developer machine with the memory package pip-installed (editable), that probe succeeds
# even inside scratch fixtures, and the REAL doctor rejects /tmp stores (sandbox-path
# invariant) - which would pollute this suite's exit codes with doctor noise. This suite
# tests the FLOOR, so the import is shadowed with a raising stub: the doctor gate SKIPs
# deterministically on every machine. test_doctor_gate.sh owns the doctor states.
mkdir -p "$TMP/py-noimport/cyberos"
printf 'raise ImportError("doctor gate stubbed out for floor tests")\n' > "$TMP/py-noimport/cyberos/__init__.py"

initrepo() { ( cd "$1" && git init -q . 2>/dev/null; CYBEROS_OFFLINE=1 CYBEROS_NO_MIGRATE=1 bash "$TMP/payload/install.sh" "$1" >/dev/null 2>&1 ); }
rungates() { local d="$1"; shift; ( cd "$d" && env CYBEROS_OFFLINE=1 PYTHONPATH="$TMP/py-noimport" "$@" bash "$d/.cyberos/cuo/gates/run-gates.sh" 2>&1 ); }
genv()     { grep "^$2=" "$1/.cyberos/gates.env" | head -1 | cut -d= -f2- | tr -d '"'; }
# top CHANGELOG entry = everything between the first '## [' heading and the second
top_entry() { awk '/^## \[/{n++} n==1{print} n==2{exit}' "$repo/CHANGELOG.md"; }

mkdir -p "$TMP/empty" && initrepo "$TMP/empty"

t01_empty_floor_exits_red() {                                          # AC 1 (#1.1)
  local d="$TMP/empty" out rc
  out="$(rungates "$d")"; rc=$?
  [ "$rc" -eq 3 ] || { fail t01 "empty floor rc=$rc (want 3): $out"; return; }
  # any one configured command keeps today's semantics: 0 on pass, 1 on gate failure
  printf 'gates:\n  test: "true"\n' > "$d/.cyberos/config.yaml"
  out="$(rungates "$d")"; rc=$?
  [ "$rc" -eq 0 ] && grep -q "GATES: GREEN" <<<"$out" || { fail t01 "one passing cmd rc=$rc (want 0): $out"; return; }
  printf 'gates:\n  test: "false"\n' > "$d/.cyberos/config.yaml"
  out="$(rungates "$d")"; rc=$?
  [ "$rc" -eq 1 ] || { fail t01 "one failing cmd rc=$rc (want 1): $out"; return; }
  rm -f "$d/.cyberos/config.yaml"
  # the escape hatch is the LITERAL 1: any other value behaves as unset
  out="$(rungates "$d" CYBEROS_ALLOW_EMPTY_GATES=true)"; rc=$?
  [ "$rc" -eq 3 ] || { fail t01 "ALLOW=true rc=$rc (want 3 - literal 1 only)"; return; }
  out="$(rungates "$d" CYBEROS_ALLOW_EMPTY_GATES=0)"; rc=$?
  [ "$rc" -eq 3 ] || { fail t01 "ALLOW=0 rc=$rc (want 3 - literal 1 only)"; return; }
  ok t01_empty_floor_exits_red
}

t02_red_message_actionable() {                                         # AC 2 (#1.2)
  local out; out="$(rungates "$TMP/empty")"
  grep -q "\.cyberos/config\.yaml" <<<"$out" || { fail t02 "config.yaml fix not named: $out"; return; }
  grep -q "re-run the install" <<<"$out"     || { fail t02 "re-install fix not named: $out"; return; }
  grep -q "CYBEROS_ALLOW_EMPTY_GATES" <<<"$out" || { fail t02 "escape hatch not named: $out"; return; }
  ok t02_red_message_actionable
}

t03_ack_line_distinct() {                                              # AC 3 (#1.3)
  local out rc
  out="$(rungates "$TMP/empty" CYBEROS_ALLOW_EMPTY_GATES=1)"; rc=$?
  [ "$rc" -eq 0 ] || { fail t03 "rc=$rc (want 0): $out"; return; }
  grep -q "GATES: EMPTY-ACKNOWLEDGED" <<<"$out" || { fail t03 "ack line missing: $out"; return; }
  grep -q "GATES: GREEN" <<<"$out" && { fail t03 "GREEN printed on an acknowledged-empty run: $out"; return; }
  ok t03_ack_line_distinct
}

t04_monorepo_fallback_seeds_test_cmd() {                               # AC 4 (#1.4)
  # fixture A: canonical suite entrypoint, no detectable ecosystem. The probe target is a
  # SENTINEL: executing it writes a marker, so non-execution is observable (audit ISS-003).
  local a="$TMP/fxa"; mkdir -p "$a/scripts/tests"
  printf '#!/usr/bin/env bash\ntouch "%s/.executed-run-all"\n' "$a" > "$a/scripts/tests/run_all.sh"
  chmod +x "$a/scripts/tests/run_all.sh"
  initrepo "$a"
  [ "$(genv "$a" TEST_CMD)" = "bash scripts/tests/run_all.sh" ] || { fail t04 "fxA TEST_CMD=$(genv "$a" TEST_CMD)"; return; }
  [ "$(genv "$a" SRC_TEST)" = "fallback:run_all" ]              || { fail t04 "fxA SRC_TEST=$(genv "$a" SRC_TEST)"; return; }
  [ ! -f "$a/.executed-run-all" ] || { fail t04 "install EXECUTED the run_all probe target"; return; }
  # fixture B: Makefile-only (test: target is a sentinel too)
  local b="$TMP/fxb"; mkdir -p "$b"
  printf 'test:\n\ttouch %s/.executed-make\n' "$b" > "$b/Makefile"
  initrepo "$b"
  [ "$(genv "$b" TEST_CMD)" = "make test" ]        || { fail t04 "fxB TEST_CMD=$(genv "$b" TEST_CMD)"; return; }
  [ "$(genv "$b" SRC_TEST)" = "fallback:make" ]    || { fail t04 "fxB SRC_TEST=$(genv "$b" SRC_TEST)"; return; }
  [ ! -f "$b/.executed-make" ] || { fail t04 "install EXECUTED the Makefile probe target"; return; }
  # fixture C: BOTH present - the ordered list is contractual, run_all wins (edge case)
  local c="$TMP/fxc"; mkdir -p "$c/scripts/tests"
  printf '#!/usr/bin/env bash\ntouch "%s/.executed-run-all"\n' "$c" > "$c/scripts/tests/run_all.sh"
  printf 'test:\n\ttouch %s/.executed-make\n' "$c" > "$c/Makefile"
  initrepo "$c"
  [ "$(genv "$c" TEST_CMD)" = "bash scripts/tests/run_all.sh" ] || { fail t04 "fxC precedence TEST_CMD=$(genv "$c" TEST_CMD)"; return; }
  [ "$(genv "$c" SRC_TEST)" = "fallback:run_all" ]              || { fail t04 "fxC precedence SRC_TEST=$(genv "$c" SRC_TEST)"; return; }
  [ ! -f "$c/.executed-run-all" ] && [ ! -f "$c/.executed-make" ] || { fail t04 "fxC executed a probe target"; return; }
  ok t04_monorepo_fallback_seeds_test_cmd
}

t05_header_machine_owned() {                                           # AC 5 (#1.5)
  local env_file="$TMP/empty/.cyberos/gates.env"
  grep -q "edit freely" "$env_file" && { fail t05 "header still says 'edit freely'"; return; }
  grep -q "machine-owned" "$env_file" || { fail t05 "machine-owned wording missing"; return; }
  grep -q "\.cyberos/config\.yaml" "$env_file" || { fail t05 "config.yaml override home not named"; return; }
  ok t05_header_machine_owned
}

t06_changelog_breaking_entry() {                                       # AC 6 (#1.6)
  local top; top="$(top_entry)"
  grep -qi "breaking" <<<"$top" || { fail t06 "top entry lacks 'breaking'"; return; }
  grep -q "CYBEROS_ALLOW_EMPTY_GATES" <<<"$top" || { fail t06 "top entry lacks the migration env var"; return; }
  grep -q "RED" <<<"$top" || { fail t06 "top entry lacks the RED-on-empty change"; return; }
  ok t06_changelog_breaking_entry
}

echo "test_fail_closed_gates.sh (TASK-CUO-302)"
t01_empty_floor_exits_red; t02_red_message_actionable; t03_ack_line_distinct
t04_monorepo_fallback_seeds_test_cmd; t05_header_machine_owned; t06_changelog_breaking_entry
echo "----"; echo "pass=$PASS fail=$FAIL"; [ "$FAIL" -eq 0 ]
