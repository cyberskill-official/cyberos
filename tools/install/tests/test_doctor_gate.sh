#!/usr/bin/env bash
# test_doctor_gate.sh - TASK-MEMORY-303 #1.6 (AC 6): run-gates gains a presence-gated
# doctor gate. With a healthy store it shows PASS doctor; with a violating store it FAILs
# and the run exits RED; with no store it emits a SKIP provenance line and the exit is
# unchanged; with a store but no importable memory CLI it SKIPs too (the import probe is
# `python3 -c "import cyberos.core"`, never a bare $PATH name - the TASK-IMP-130 lesson).
#
# The three contract states run against a STUB cyberos module (PYTHONPATH-injected) so the
# suite is deterministic on machines without the memory package; a bonus arm runs the REAL
# doctor when the package is importable (CYBEROS_HOST_MOUNT_PREFIX exempts the tmp store
# from the sandbox-path invariant, the same exemption the memory module's own tests use).
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

echo "building scratch payload..."
bash "$repo/tools/install/build.sh" "$TMP/payload" >/dev/null 2>&1 || { echo FATAL build; exit 1; }

# stub package A: importable; `python3 -m cyberos doctor` FAILs iff the store carries a
# dir named stray-noncanonical (the fixture's layout-violation knob), passes otherwise
mkdir -p "$TMP/py-ok/cyberos/core"
: > "$TMP/py-ok/cyberos/__init__.py"
: > "$TMP/py-ok/cyberos/core/__init__.py"
cat > "$TMP/py-ok/cyberos/__main__.py" <<'PY'
import os, sys
if len(sys.argv) > 1 and sys.argv[1] == "doctor":
    store = os.path.join(os.getcwd(), ".cyberos", "memory", "store")
    bad = os.path.isdir(os.path.join(store, "stray-noncanonical"))
    print("stub doctor: overall:", "FAIL" if bad else "OK")
    sys.exit(1 if bad else 0)
sys.exit(2)
PY
# stub package B: NOT importable (raises on import) - forces the CLI-absent state even on
# machines where the real memory package is pip-installed
mkdir -p "$TMP/py-noimport/cyberos"
printf 'raise ImportError("forced non-importable for the doctor-gate SKIP state")\n' > "$TMP/py-noimport/cyberos/__init__.py"

d="$TMP/fix"; mkdir -p "$d"
( cd "$d" && git init -q . 2>/dev/null; CYBEROS_OFFLINE=1 CYBEROS_NO_MIGRATE=1 bash "$TMP/payload/install.sh" "$d" >/dev/null 2>&1 )
# a passing floor, so the run's exit code reflects the doctor gate alone
printf 'gates:\n  test: "true"\n' > "$d/.cyberos/config.yaml"

rungates() { local pp="$1"; shift; ( cd "$d" && env CYBEROS_OFFLINE=1 PYTHONPATH="$pp" "$@" bash "$d/.cyberos/cuo/gates/run-gates.sh" 2>&1 ); }

t01_doctor_gate_three_states() {                                       # AC 6 (#1.6)
  local out rc
  # state 1: healthy store -> PASS doctor, run green
  [ -d "$d/.cyberos/memory/store" ] || { fail t01 "install did not scaffold the store"; return; }
  out="$(rungates "$TMP/py-ok")"; rc=$?
  grep -q "gate doctor: python3 -m cyberos doctor" <<<"$out" || { fail t01 "doctor provenance line missing: $out"; return; }
  grep -q "PASS  doctor" <<<"$out" || { fail t01 "healthy store did not PASS doctor: $out"; return; }
  [ "$rc" -eq 0 ] || { fail t01 "healthy run rc=$rc (want 0)"; return; }
  # state 2: violating store -> FAIL doctor, RED exit
  mkdir -p "$d/.cyberos/memory/store/stray-noncanonical"
  out="$(rungates "$TMP/py-ok")"; rc=$?
  grep -q "FAIL  doctor" <<<"$out" || { fail t01 "violating store did not FAIL doctor: $out"; return; }
  grep -q "GATES: RED" <<<"$out" || { fail t01 "doctor FAIL did not go RED: $out"; return; }
  [ "$rc" -eq 1 ] || { fail t01 "violating run rc=$rc (want 1)"; return; }
  rmdir "$d/.cyberos/memory/store/stray-noncanonical"
  # state 3: no store -> SKIP provenance line, exit unchanged
  mv "$d/.cyberos/memory/store" "$d/.cyberos/memory/store.aside"
  out="$(rungates "$TMP/py-ok")"; rc=$?
  grep -q "SKIP  doctor" <<<"$out" || { fail t01 "no-store run lacks the SKIP line: $out"; return; }
  grep -q "no memory store" <<<"$out" || { fail t01 "SKIP line lacks the store provenance: $out"; return; }
  [ "$rc" -eq 0 ] || { fail t01 "no-store run rc=$rc (want 0 - behavior unchanged)"; return; }
  mv "$d/.cyberos/memory/store.aside" "$d/.cyberos/memory/store"
  ok t01_doctor_gate_three_states
}

t02_cli_absent_skips() {                                               # #1.6 edge: store present, CLI not importable
  local out rc
  out="$(rungates "$TMP/py-noimport")"; rc=$?
  grep -q "SKIP  doctor" <<<"$out" || { fail t02 "non-importable CLI did not SKIP: $out"; return; }
  grep -q "not importable" <<<"$out" || { fail t02 "SKIP line lacks the import provenance: $out"; return; }
  [ "$rc" -eq 0 ] || { fail t02 "rc=$rc (want 0 - the gate must not invent a failure)"; return; }
  ok t02_cli_absent_skips
}

t03_real_doctor_when_available() {                                     # integration arm (best-effort, real module)
  if ! env PYTHONPATH= python3 -c "import cyberos.core" >/dev/null 2>&1; then
    ok "t03_real_doctor_when_available (real memory package not installed here - stub arms above carry the contract)"
    return
  fi
  local out rc
  out="$( cd "$d" && env CYBEROS_OFFLINE=1 PYTHONPATH= CYBEROS_HOST_MOUNT_PREFIX="$TMP" bash "$d/.cyberos/cuo/gates/run-gates.sh" 2>&1 )"; rc=$?
  grep -q "PASS  doctor" <<<"$out" || { fail t03 "real doctor did not PASS on the fresh scaffolded store: $out"; return; }
  [ "$rc" -eq 0 ] || { fail t03 "real-doctor run rc=$rc (want 0)"; return; }
  ok t03_real_doctor_when_available
}

echo "test_doctor_gate.sh (TASK-MEMORY-303 #1.6 carve-out)"
t01_doctor_gate_three_states; t02_cli_absent_skips; t03_real_doctor_when_available
echo "----"; echo "pass=$PASS fail=$FAIL"; [ "$FAIL" -eq 0 ]
