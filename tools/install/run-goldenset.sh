#!/usr/bin/env bash
# run-goldenset.sh — run the CyberOS install/payload goldenset (TASK-IMP-008).
#
# Prefers `awh eval` when available; otherwise a python3 YAML fallback that
# executes each task cmd. Skips cleanly when CYBEROS_SKIP_GOLDENSET=1 or when
# neither awh nor python3 can run the suite.
#
# Usage: bash tools/install/run-goldenset.sh [--no-baseline]
# Exit: 0 green or SKIP; 1 regression/failure; 2 usage/missing goldenset
set -uo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
GS="${CYBEROS_INSTALL_GOLDENSET:-$root/tools/install/.awh/goldenset.yaml}"
BASE="${CYBEROS_INSTALL_GOLDENSET_BASELINE:-$root/tools/install/.awh/eval-baseline.json}"
NO_BASELINE=0
for a in "$@"; do
  case "$a" in
    --no-baseline) NO_BASELINE=1 ;;
    --help|-h)
      echo "usage: run-goldenset.sh [--no-baseline]"
      echo "  CYBEROS_SKIP_GOLDENSET=1  skip cleanly (exit 0)"
      exit 0
      ;;
  esac
done

if [ "${CYBEROS_SKIP_GOLDENSET:-}" = "1" ]; then
  echo "  SKIP install-goldenset — CYBEROS_SKIP_GOLDENSET=1"
  exit 0
fi

if [ ! -f "$GS" ]; then
  echo "run-goldenset: missing $GS" >&2
  exit 2
fi

run_awh() {
  if command -v awh >/dev/null 2>&1; then
    if [ "$NO_BASELINE" -eq 1 ]; then
      awh eval "$GS" --base-dir "$root" --seeds 1
    else
      [ -f "$BASE" ] || { echo "run-goldenset: goldenset present but baseline missing ($BASE) — fail closed" >&2; return 1; }
      awh eval "$GS" --base-dir "$root" --seeds 1 --baseline "$BASE" --max-regression 0.0
    fi
    return $?
  fi
  if command -v python3 >/dev/null 2>&1 && [ -d "$root/tools/awh" ]; then
    local args=(eval "$GS" --base-dir "$root" --seeds 1)
    if [ "$NO_BASELINE" -eq 0 ]; then
      [ -f "$BASE" ] || { echo "run-goldenset: goldenset present but baseline missing ($BASE) — fail closed" >&2; return 1; }
      args+=(--baseline "$BASE" --max-regression 0.0)
    fi
    ( cd "$root" && PYTHONPATH="tools/awh${PYTHONPATH:+:$PYTHONPATH}" python3 -m harness.cli "${args[@]}" )
    return $?
  fi
  return 127
}

run_fallback() {
  # Minimal offline runner: execute each tasks[].cmd; ignore baseline comparison
  # (baseline sealing remains an awh/CI concern). Still fail closed if YAML missing tasks.
  command -v python3 >/dev/null 2>&1 || return 127
  INSTALL_GS="$GS" INSTALL_ROOT="$root" python3 - <<'PY'
import os, subprocess, sys
from pathlib import Path
try:
    import yaml
except ImportError:
    # stdlib-only tiny parser for our sealed shape: id/cmd/weight/timeout_sec lines
    yaml = None

gs = Path(os.environ["INSTALL_GS"])
root = Path(os.environ["INSTALL_ROOT"])
text = gs.read_text()
tasks = []
if yaml is not None:
    data = yaml.safe_load(text) or {}
    tasks = data.get("tasks") or ([data] if "cmd" in data else [])
else:
    # Extremely small extractor: blocks starting with "  - id:" 
    cur = None
    for line in text.splitlines():
        if line.strip().startswith("- id:"):
            if cur and "cmd" in cur: tasks.append(cur)
            cur = {"id": line.split(":",1)[1].strip().strip('"').strip("'")}
        elif cur is not None and line.strip().startswith("cmd:"):
            raw = line.split(":",1)[1].strip()
            if raw.startswith('"') and raw.endswith('"'): raw = raw[1:-1]
            if raw.startswith("'") and raw.endswith("'"): raw = raw[1:-1]
            cur["cmd"] = raw
    if cur and "cmd" in cur: tasks.append(cur)

if not tasks:
    print("run-goldenset fallback: no tasks parsed", file=sys.stderr)
    sys.exit(2)
failed = 0
for t in tasks:
    tid = t.get("id", "?")
    cmd = t.get("cmd")
    if not cmd:
        print(f"FAIL {tid}: missing cmd"); failed = 1; continue
    print(f"GATE  goldenset/{tid}: {cmd}")
    r = subprocess.run(cmd, shell=True, cwd=str(root))
    if r.returncode != 0:
        print(f"FAIL  {tid} (exit {r.returncode})")
        failed = 1
    else:
        print(f"PASS  {tid}")
sys.exit(failed)
PY
}

echo "run-goldenset: $GS"
if run_awh; then
  echo "run-goldenset: GREEN (awh)"
  exit 0
fi
rc=$?
if [ "$rc" -ne 127 ]; then
  echo "run-goldenset: RED (awh exit $rc)" >&2
  exit 1
fi

echo "run-goldenset: awh unavailable — trying python fallback"
if run_fallback; then
  echo "run-goldenset: GREEN (fallback)"
  exit 0
fi
frc=$?
if [ "$frc" -eq 127 ]; then
  echo "  SKIP install-goldenset — neither awh nor python3 available"
  exit 0
fi
echo "run-goldenset: RED (fallback exit $frc)" >&2
exit 1
