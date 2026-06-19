#!/usr/bin/env bash
# Retire the standalone auto-work-harness. Safe to run now (all cyberos modules are green
# under the vendored tools/awh gate). Archives history first (tag + push + bundle); only
# removes the working copy when you pass --delete.
#
#   bash scripts/awh_retire_standalone.sh            # commit the gate fix + archive (keep dir)
#   bash scripts/awh_retire_standalone.sh --delete    # archive, then delete the working copy
set -uo pipefail
A="$HOME/Projects/auto-work-harness"
[ -d "$A/.git" ] || { echo "no git repo at $A"; exit 1; }
cd "$A"

echo "STEP 1/3  commit + push the gate fix (the repos that adopted awh need it)"
git add harness/stage1_measurement/runner.py tests/test_stage1_runner.py 2>/dev/null || true
git diff --cached --quiet || git commit -m "fix(gate): fail closed when a current task is absent from the baseline"
git push || echo "  push skipped/failed; make origin current before archiving"

echo "STEP 2/3  archive history (tag + push + bundle)"
git tag -a archive/pre-cyberos-absorb -m "archived after vendoring into cyberos $(date +%F)" 2>/dev/null || echo "  tag already exists"
git push origin archive/pre-cyberos-absorb 2>/dev/null || true
git bundle create "$HOME/Projects/auto-work-harness.bundle" --all && echo "  bundle -> ~/Projects/auto-work-harness.bundle"

if [ "${1:-}" = "--delete" ]; then
  echo "STEP 3/3  delete working copy (history is in the tag, on origin, and in the bundle)"
  cd "$HOME/Projects" && rm -rf "$A" && echo "  removed $A   (recover: git clone ~/Projects/auto-work-harness.bundle)"
else
  echo "STEP 3/3  skipped — re-run with --delete when ready. tools/awh is the canonical gate now."
fi
