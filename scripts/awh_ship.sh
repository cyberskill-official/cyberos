#!/usr/bin/env bash
# Ship the ready_to_test tasks to done THROUGH the awh gate (ship-tasks step 28).
#
#   bash scripts/awh_ship.sh TASK-MEMORY-116   # ship one task (dry-run preview, then execute)
#   bash scripts/awh_ship.sh --drain         # drain every remaining ready_to_test task
#
# It runs THIS BRANCH's cuo source directly (no install), so it always has the current flags
# (--fr-id, and the now-required --output-dir). That sidesteps the `pip install -e modules/cuo`
# failure entirely: cuo only needs click + pyyaml, which the interpreter already behind your
# installed cyberos-cuo command has. We just force the branch's cuo/ package ahead of it with
# PYTHONPATH. Runs on your Mac (needs an LLM invoker for --invoker llm).
set -uo pipefail
REPO="$(git rev-parse --show-toplevel)" || exit 1
cd "$REPO" || exit 1

WF="chief-technology-officer/ship-tasks"
OUT="${CUO_OUT:-/tmp/cuo-ship}"; mkdir -p "$OUT"

# Interpreter that has cuo's deps: prefer the one behind the installed console script, else python3.
PYBIN=""
if command -v cyberos-cuo >/dev/null 2>&1; then
  PYBIN=$(sed -n '1s/^#![[:space:]]*//p' "$(command -v cyberos-cuo)")
fi
[ -n "${PYBIN:-}" ] && [ -x "$PYBIN" ] || PYBIN=$(command -v python3)
[ -n "$PYBIN" ] || { echo "no python3 found"; exit 1; }

# Force the branch's cuo package (modules/cuo) ahead of any installed copy.
export PYTHONPATH="$REPO/modules/cuo${PYTHONPATH:+:$PYTHONPATH}"
CUO=( "$PYBIN" -m cuo.cli )

# Prove deps import + the branch flags are present before doing real work.
err=$(mktemp)
if ! "${CUO[@]}" --help >/dev/null 2>"$err"; then
  echo "Could not run cuo from source with: $PYBIN"; echo "----"; cat "$err"; echo "----"
  echo "If it is a missing module, install the two deps for THAT interpreter:"
  echo "    $PYBIN -m pip install click pyyaml"
  rm -f "$err"; exit 1
fi
rm -f "$err"
"${CUO[@]}" execute --help 2>/dev/null | grep -q -- '--fr-id' || {
  echo "Branch source loaded but execute lacks --fr-id (unexpected) - stop."; exit 1; }

echo "using interpreter: $PYBIN"
echo "branch cuo source: $REPO/modules/cuo/cuo   |   output-dir: $OUT"
echo

if [ "${1:-}" = "--drain" ]; then
  echo "== drain: ship every remaining ready_to_test task through the gate =="
  echo "   each task flips testing->done ONLY if step-28 awh-gate reruns GREEN; RED routes it back."
  exec "${CUO[@]}" drain "$WF" --invoker llm --output-dir "$OUT" --rework
fi

task="${1:?usage: awh_ship.sh TASK-XXXX-NNN   |   awh_ship.sh --drain}"
echo "== dry-run: preview the workflow skill chain =="
"${CUO[@]}" dry-run "$WF" || true
echo
echo "== execute $task through the gate =="
echo "   step 28 (awh-gate) reruns the module suite vs the sealed baseline;"
echo "   GREEN is required for the step-29 testing->done flip."
exec "${CUO[@]}" execute "$WF" --fr-id "$task" --invoker llm --output-dir "$OUT"
