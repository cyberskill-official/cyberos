#!/usr/bin/env bash
# test_cli_memory_verb.sh - TASK-IMP-131 contract: `cs memory` dispatches via
# `python3 -m cyberos` when cyberos-memory is locally available, and fails closed
# with a clear message when it is not. Never resolves via bare `$PATH` lookup of
# the name `cyberos` (the ambient collision this whole rename plan removes).
#
#   t01_memory_is_known_command              memory is routed (not "unknown command");
#                                            with a working stub, stub output is printed.
#   t02_resolution_not_via_path_cyberos      PATH `cyberos` is ignored; python3 -m wins.
#   t03_dispatch_forwards_args_and_exit_code child args + exit code forwarded.
#   t04_missing_python_clear_error           no python3 → exit 2 + "not bundled" message.
#   t05_docs_state_gating                    help.sh + docs/index.md mention memory +
#                                            local-availability caveat (shared marker).
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

# Absolute node so PATH stubs cannot hide the interpreter that runs cli.mjs.
NODE="$(command -v node)" || { echo "FATAL: node not on PATH"; exit 1; }

echo "building scratch payload..."
bash "$repo/tools/install/build.sh" "$TMP/payload" >/dev/null 2>&1 || { echo FATAL build; exit 1; }
CLI="$TMP/payload/cli/bin/cli.mjs"

# A python3 stand-in that implements `python3 -m cyberos ...` for the resolution probe
# and the real dispatch. Behaviour is controlled by env vars the individual tests set.
install_python3_stub() {
  local bin_dir="$1"
  mkdir -p "$bin_dir"
  cat >"$bin_dir/python3" <<'PYSTUB'
#!/bin/bash
# Minimal python3 stand-in for TASK-IMP-131 tests. Honours `-m cyberos`.
# Absolute shebang so a stub-only PATH (no /usr/bin/env) still executes.
set -uo pipefail
if [ "${1:-}" = "-m" ] && [ "${2:-}" = "cyberos" ]; then
  shift 2
  # Probe: `python3 -m cyberos --help` — succeed unless CYBEROS_STUB_PROBE_FAIL=1.
  if [ "${1:-}" = "--help" ] && [ "$#" -eq 1 ]; then
    if [ "${CYBEROS_STUB_PROBE_FAIL:-0}" = "1" ]; then
      echo "ModuleNotFoundError: No module named 'cyberos'" >&2
      exit 1
    fi
    echo "cyberos-memory stub help"
    exit 0
  fi
  # Real dispatch: echo a marker / args / exit code as the test configures.
  if [ -n "${CYBEROS_STUB_PRINT:-}" ]; then
    printf '%s\n' "$CYBEROS_STUB_PRINT"
  fi
  if [ "${CYBEROS_STUB_ECHO_ARGS:-0}" = "1" ]; then
    printf 'ARGS:%s\n' "$*"
  fi
  exit "${CYBEROS_STUB_EXIT:-0}"
fi
echo "python3-stub: unexpected invocation: $*" >&2
exit 99
PYSTUB
  chmod +x "$bin_dir/python3"
}

# PATH with ONLY the stub bin dir. cli.mjs is invoked via absolute $NODE, so node
# need not be on PATH; this also keeps a real system python3 from leaking into t04.
stub_path() {
  printf '%s' "$1"
}

t01_memory_is_known_command() {
  # (a) no python3 on PATH — must NOT say unknown command 'memory' (recognised but unusable).
  local empty="$TMP/empty-bin"; mkdir -p "$empty"
  local out rc
  out="$(PATH="$(stub_path "$empty")" "$NODE" "$CLI" memory --help 2>&1)"; rc=$?
  if printf '%s' "$out" | grep -qi "unknown command 'memory'"; then
    fail t01_memory_is_known_command "(a) still treated as unknown: $out"
    return
  fi
  # (b) with working python3 -m cyberos stub — stub output IS printed.
  local stub="$TMP/stub-bin-t01"; install_python3_stub "$stub"
  out="$(PATH="$(stub_path "$stub")" CYBEROS_STUB_PRINT="STUB-HELP-OK" "$NODE" "$CLI" memory doctor 2>&1)"; rc=$?
  if printf '%s' "$out" | grep -q 'STUB-HELP-OK'; then
    ok t01_memory_is_known_command
  else
    fail t01_memory_is_known_command "(b) stub output missing (rc=$rc): $out"
  fi
}

t02_resolution_not_via_path_cyberos() {
  local stub="$TMP/stub-bin-t02"; install_python3_stub "$stub"
  # Fake `cyberos` on PATH that would silently mis-dispatch if the CLI used PATH lookup.
  cat >"$stub/cyberos" <<'EOF'
#!/bin/bash
echo WRONG-PATH-DISPATCH
exit 0
EOF
  chmod +x "$stub/cyberos"
  local out
  out="$(PATH="$(stub_path "$stub")" CYBEROS_STUB_PRINT="CORRECT-DISPATCH" "$NODE" "$CLI" memory doctor 2>&1)"
  if printf '%s' "$out" | grep -q 'CORRECT-DISPATCH' && ! printf '%s' "$out" | grep -q 'WRONG-PATH-DISPATCH'; then
    ok t02_resolution_not_via_path_cyberos
  else
    fail t02_resolution_not_via_path_cyberos "got: $out"
  fi
}

t03_dispatch_forwards_args_and_exit_code() {
  local stub="$TMP/stub-bin-t03"; install_python3_stub "$stub"
  local out rc
  out="$(PATH="$(stub_path "$stub")" CYBEROS_STUB_ECHO_ARGS=1 CYBEROS_STUB_EXIT=3 "$NODE" "$CLI" memory foo bar 2>&1)"; rc=$?
  if printf '%s' "$out" | grep -q 'ARGS:foo bar' && [ "$rc" -eq 3 ]; then
    ok t03_dispatch_forwards_args_and_exit_code
  else
    fail t03_dispatch_forwards_args_and_exit_code "rc=$rc out=$out"
  fi
}

t04_missing_python_clear_error() {
  local empty="$TMP/empty-bin-t04"; mkdir -p "$empty"
  # Also plant a tempting PATH cyberos so a wrong resolution still fails loudly.
  cat >"$empty/cyberos" <<'EOF'
#!/bin/bash
echo SHOULD-NOT-RUN
exit 0
EOF
  chmod +x "$empty/cyberos"
  local out rc
  out="$(PATH="$(stub_path "$empty")" "$NODE" "$CLI" memory doctor 2>&1)"; rc=$?
  local bad=""
  [ "$rc" -eq 2 ] || bad="$bad exit=$rc-want-2"
  printf '%s' "$out" | grep -qi 'cyberos-memory' || bad="$bad missing-cyberos-memory"
  printf '%s' "$out" | grep -qi 'not bundled' || bad="$bad missing-not-bundled"
  printf '%s' "$out" | grep -qi 'Traceback\|ModuleNotFoundError' && bad="$bad has-python-traceback"
  printf '%s' "$out" | grep -q 'SHOULD-NOT-RUN' && bad="$bad path-cyberos-ran"
  printf '%s' "$out" | grep -qi "unknown command 'memory'" && bad="$bad treated-as-unknown"
  if [ -z "$bad" ]; then
    ok t04_missing_python_clear_error
  else
    fail t04_missing_python_clear_error "$bad :: $out"
  fi
}

t05_docs_state_gating() {
  # Shared marker: "local-availability" appears in both help.sh output and docs/index.md
  # next to the memory verb (AC 5).
  local help_out index_md="$repo/tools/install/docs/index.md" bad=""
  help_out="$(cd "$TMP/payload" && bash help.sh 2>&1)"
  printf '%s' "$help_out" | grep -q 'memory' || bad="$bad help.sh-missing-memory"
  printf '%s' "$help_out" | grep -qi 'local-availability' || bad="$bad help.sh-missing-local-availability"
  grep -q 'memory' "$index_md" || bad="$bad index.md-missing-memory"
  grep -qi 'local-availability' "$index_md" || bad="$bad index.md-missing-local-availability"
  if [ -z "$bad" ]; then
    ok t05_docs_state_gating
  else
    fail t05_docs_state_gating "$bad"
  fi
}

t01_memory_is_known_command
t02_resolution_not_via_path_cyberos
t03_dispatch_forwards_args_and_exit_code
t04_missing_python_clear_error
t05_docs_state_gating

echo
echo "pass=$PASS fail=$FAIL"
[ "$FAIL" -eq 0 ]
