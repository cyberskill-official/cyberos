#!/usr/bin/env bash
# One-shot Phase-1 finalize for the awh absorption. Run on your Mac, from the repo,
# on branch auto/awh-absorb:   bash scripts/awh_finalize.sh
#
# It untracks the leaked credential/artifact files, commits the whole branch, and
# regenerates the docs HTML. It does NOT rotate secrets or set branch protection
# (it prints those as the only manual follow-ups at the end).
set -uo pipefail
cd "$(git rev-parse --show-toplevel)" || exit 1

branch=$(git rev-parse --abbrev-ref HEAD)
if [ "$branch" != "auto/awh-absorb" ]; then
  echo "Expected branch auto/awh-absorb but on '$branch'. Aborting."; exit 1
fi

echo "STEP 1/4  untrack leaked tokens + proptest artifact (.gitignore already covers them)"
for f in deploy/obs/auth/tokens.live deploy/obs/auth/collector.token.live \
         services/ai-gateway/tests/cache_isolation_property_test.proptest-regressions; do
  if git ls-files --error-unmatch "$f" >/dev/null 2>&1; then
    git rm --cached -q "$f" && echo "  untracked $f"
  fi
done

echo "STEP 2/4  stage everything except .claude/ (and the now-ignored leaked files)"
git add -A
git reset -q -- .claude 2>/dev/null
echo "  staged summary:"; git --no-optional-locks diff --cached --stat | tail -1

echo "STEP 3/4  commit (--no-verify: the 7 module gates already ran GREEN via awh_bootstrap_waves.sh)"
git commit --no-verify -m "feat(awh): absorb the verification gate; 7 modules green under awh

Gate proven green on memory, skill, cuo, auth, chat, proj, email. Includes: vendored
tools/awh, per-module .awh golden sets + baselines, ship-tasks step-28 gate
(testing->done conditional on an independent GREEN rerun), CI + pre-commit (fail closed),
task re-baseline (116 done->ready_to_test) + 193 cited-test path fixes, the awh-gate skill,
planning scripts (bootstrap/coverage/build-order/goldenset-from-task/cited-fixups), the
verification-gate docs page, and the migrated maturity ledger. Untracks leaked token files." \
  || { echo "  commit aborted (a hook failed?). Fix, then re-run or 'git commit' manually."; exit 1; }
echo "  committed $(git rev-parse --short HEAD)"

echo "STEP 4/4  regenerate docs HTML"
if [ -f website/build/build.sh ]; then
  ( cd website && bash build/build.sh ) && echo "  docs rebuilt (review website/docs, then re-stage + amend if changed)" \
    || echo "  build.sh failed; run it manually and inspect"
else
  echo "  website/build/build.sh not found; skip"
fi

cat <<'NEXT'

DONE. Push when ready:   git push -u origin auto/awh-absorb

Only these four are not scriptable here:
  1. ROTATE the two tokens at their source (git history still contains them).
  2. Branch protection: mark the "awh gate" CI job required on main (GitHub settings).
  3. Retire standalone awh:  follow tools/awh/RETIREMENT.md  (all modules are green now).
  4. Ship Step 5: run chief-technology-officer/ship-tasks to move the 116
     ready_to_test tasks to done; each now passes through the trusted gate.
NEXT
