#!/usr/bin/env bash
# Stand up the AI module gate and, only if the suite is truly GREEN, capture a sealed baseline
# and promote the 20 ai tasks. AI was red-deferred, so this is deliberately strict:
#   - it never captures a baseline from a red run (that would weaken the gate to whatever passes),
#   - it requires a running Redis (the cache-isolation tests connect to 127.0.0.1:6379).
#
#   bash scripts/awh_ai_gate.sh            # verify -> (if green) capture + seal + promote
#
# Run on your Mac from the repo root, with the toolchain (cargo) and Redis available.
set -uo pipefail
REPO="$(git rev-parse --show-toplevel)" || exit 1
cd "$REPO" || exit 1

GS="modules/ai/.awh/goldenset.yaml"
BASE="modules/ai/.awh/eval-baseline.json"
HELDOUT="services/ai-gateway/tests/cache_isolation_adversarial_test.rs"

echo "== 1/4  Redis reachable at 127.0.0.1:6379? =="
if ! (exec 3<>/dev/tcp/127.0.0.1/6379) 2>/dev/null; then
  echo "  Redis is NOT reachable. Start it, then re-run:"
  echo "      docker run -d --name cyberos-redis -p 6379:6379 redis:7"
  exit 1
fi
exec 3>&- 2>/dev/null
echo "  ok"

echo "== 2/4  capture the AI gate via awh (runs the real suite once) =="
PYTHONPATH="tools/awh${PYTHONPATH:+:$PYTHONPATH}" python3 -m harness.cli eval "$GS" \
  --base-dir . --seeds 1 --out "$BASE" || { echo "  awh eval failed"; rm -f "$BASE"; exit 1; }

echo "== 3/4  accept the baseline ONLY if every task fully passed =="
if ! python3 - "$BASE" <<'PY'
import json, sys
d = json.load(open(sys.argv[1])); ts = d.get("tasks", [])
den = sum(t.get("weight", 1) for t in ts) or 1
weighted = sum(t.get("pass_at_1", 0) * t.get("weight", 1) for t in ts) / den
failing = [t.get("task_id", "?") for t in ts if t.get("pass_at_1", 0) < 1.0]
print(f"  weighted pass@1 = {weighted:.3f}" + (f"  failing={failing}" if failing else "  (all green)"))
sys.exit(0 if (weighted >= 0.999 and not failing) else 1)
PY
then
  echo "  AI suite is RED -> discarding the captured baseline (never seal a red bar)."
  echo "  The 20 ai tasks stay ready_to_test. Fix the failing task(s) above, then re-run."
  rm -f "$BASE"
  exit 1
fi

echo "== 4/4  GREEN -> seal the held-out test, then promote ai (auto-detected) =="
chmod a-w "$HELDOUT" 2>/dev/null && echo "  sealed read-only: $HELDOUT"
python3 scripts/awh_promote.py --apply
echo
echo "Done. Rebuild docs and review:"
echo "    ( cd website && bash build/build.sh )"
echo "    git --no-optional-locks diff -- docs/tasks"
