#!/usr/bin/env bash
# Turnkey wave bootstrap: capture a baseline, seal the held-out test, and report green/red
# for every gated module in ONE run. Use this instead of running each wave by hand.
#
# Run from anywhere inside the repo, on a machine with the toolchain (cargo + pytest + make):
#   bash scripts/awh_bootstrap_waves.sh           # only modules missing a baseline
#   bash scripts/awh_bootstrap_waves.sh --force    # recapture every module
#
# Requires the vendored gate on PATH:  pip install -e tools/awh
# Writes baselines + a policy.json (via awh lock) but never commits. Review with git diff.
#
# Sealing scope: we seal only the HELD-OUT acceptance target (the bar the agent must not
# edit), NOT the whole tests directory. Sealing a whole Python tests dir read-only breaks
# suites whose tests/conftest write artifacts in place (e.g. cuo). Rust test dirs are safe
# to seal as a dir because cargo does not write into the source tree. The agent-level
# deny-write policy (policy.json) still covers the broader tree.
set -uo pipefail
cd "$(git rev-parse --show-toplevel)" || exit 1

FORCE=0; [ "${1:-}" = "--force" ] && FORCE=1

# module -> held-out target to seal read-only
seal_target() {
  case "$1" in
    memory) echo "modules/memory/tests/core/test_consolidate_semantic_dedup.py" ;;
    skill)  echo "services/skill-broker/tests" ;;
    cuo)    echo "modules/cuo/tests/test_langgraph_runtime.py" ;;
    auth)   echo "services/auth/tests" ;;
    chat)   echo "services/chat/tests" ;;
    proj)   echo "services/proj/tests" ;;
    email)  echo "services/email/tests" ;;
    *)      echo "" ;;
  esac
}

ORDER="memory skill cuo auth chat proj email"
summary=""

# weighted pass@1 computed directly from the baseline's tasks (robust to JSON key names);
# also prints the ids of any task that did not fully pass.
verdict() {
  python3 - "$1" <<'PY'
import json, sys
d = json.load(open(sys.argv[1])); ts = d.get("tasks", [])
num = sum(t.get("pass_at_1", 0) * t.get("weight", 1) for t in ts)
den = sum(t.get("weight", 1) for t in ts) or 1
fail = [t["task_id"] for t in ts if t.get("pass_at_1", 0) < 1.0]
print(f"{num/den:.3f}|{'GREEN' if not fail else 'RED failing='+','.join(fail)}")
PY
}

for m in $ORDER; do
  gs="modules/$m/.awh/goldenset.yaml"
  base="modules/$m/.awh/eval-baseline.json"
  [ -f "$gs" ] || { echo "skip $m (no golden set)"; continue; }

  if [ -f "$base" ] && [ "$FORCE" -eq 0 ]; then
    echo "=== $m: baseline already present ==="
  else
    echo "=== $m: capture baseline ==="
    awh eval "$gs" --base-dir . --seeds 1 --out "$base"
    st=$(seal_target "$m")
    if [ -d "$st" ]; then
      echo "=== $m: seal dir $st ==="; awh lock "$st" --write-policy || true
    elif [ -f "$st" ]; then
      # awh lock expects a directory; for a single held-out file, chmod it read-only directly
      # (reading it during the full suite is fine; only writing into a sealed dir breaks pytest).
      echo "=== $m: seal file $st (read-only) ==="; chmod a-w "$st" || true
    fi
  fi
  v=$(verdict "$base"); summary+="  $m: ${v#*|} (weighted=${v%%|*})\n"
done

echo; echo "=== maturity ==="
awh maturity report --log .awh/evolution-log.jsonl || true
echo; echo "=== gate-coverage ==="
python3 scripts/awh_gate_coverage.py | sed -n '1,9p' || true
echo; echo "=== SUMMARY ==="; printf "%b" "$summary"
echo "Review: git --no-optional-locks diff --stat ; then commit the baselines + policy files."
echo "If a Python module shows RED only because an earlier run sealed its whole tests dir,"
echo "unseal and recapture it:  chmod -R u+w modules/<m>/tests ; bash scripts/awh_bootstrap_waves.sh --force"
echo "Every module GREEN means the roadmap is gated end to end and awh can be retired."
