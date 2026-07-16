#!/usr/bin/env bash
# run_all.sh - every shell test in the repo, discovered by glob.
#
# WHY A GLOB AND NOT A LIST: on 2026-07-15 all six shell test files were reachable from no
# gate at all - not .githooks/pre-commit, not scripts/local_verify.sh, not any of the 24 CI
# workflows. Three had been red for months and reported it to nobody:
#
#   test_task_layout.sh      t09 asserted against tools/install/install.sh, deleted at
#                            bb0f2392e. `grep` on a missing file returns 1, short-circuits
#                            the && chain into fail. Red since that commit.
#   test_templates_module.sh t02 pinned status-hub@1; ac33beb54 shipped status-hub@2.
#                            t03 knew :html and bare slots; status-hub@2 added :json.
#   test_render_status_hub.sh t03/t05/t07/t08 - stale fixture + a dead id regex that
#                            silently emptied the changelog->task binding in production.
#
# A hand-maintained list has the same failure mode as those asserts: it is a second place
# that must be updated, so eventually it isn't. A glob cannot forget. Adding a test file is
# now sufficient to gate it.
#
#   bash scripts/tests/run_all.sh
set -uo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"

# `timeout` is GNU coreutils and is NOT on stock macOS — the dev machine this hook runs on.
# The first cut hard-coded `timeout 300`, which exits 127 (command not found) for EVERY
# suite: pass=0 fail=6 on macOS, pass=6 fail=0 on the Linux CI box. A uniform failure
# across unrelated suites is the tell that the runner broke, not the tests.
#
# Exactly what made check-chain-coverage.sh a no-op on macOS (bash 3.2 has no `declare -A`)
# — a gate that cannot run on the platform it gates reports nothing and blocks everything.
# Degrade to no timeout rather than fail: a hung test is a worse problem than a slow one,
# but a gate that cannot execute is the worst of the three.
if command -v timeout >/dev/null 2>&1; then TO="timeout 300"
elif command -v gtimeout >/dev/null 2>&1; then TO="gtimeout 300"
else TO=""; fi

# Sub-suites invoked BY another test still run standalone here; double-running is cheap and
# an orphan is not. Add a path here only if a file genuinely cannot run on its own.
SKIP=""

pass=0; fail=0; skip=0; failed=""
for t in scripts/tests/test_*.sh tools/docs-site/tests/test_*.sh tools/install/tests/test_*.sh; do
  [ -e "$t" ] || continue
  b="$(basename "$t")"
  case " $SKIP " in *" $b "*) echo "  skip $b"; continue ;; esac
  if $TO bash "$t" >/tmp/run_all.$$.log 2>&1; then
    # A suite may exit 0 having skipped itself on a platform requirement it cannot meet
    # (test_release_assets.sh needs GNU tar; BSD tar has no --sort/--owner/--mtime). Report
    # that as SKIP, not ok: a skip counted as a pass is how a gate quietly stops gating.
    if grep -q '^  SKIP ' /tmp/run_all.$$.log 2>/dev/null; then
      skip=$((skip+1)); printf '  \033[33mskip\033[0m %s — %s\n' "$b" "$(sed -n 's/^  SKIP [^—]*— //p' /tmp/run_all.$$.log | head -1)"
      rm -f /tmp/run_all.$$.log; continue
    fi
    pass=$((pass+1)); printf '  \033[32mok\033[0m   %s\n' "$b"
  else
    fail=$((fail+1)); failed="$failed $b"
    printf '  \033[31mFAIL\033[0m %s\n' "$b"
    sed 's/^/         /' /tmp/run_all.$$.log | grep -iE 'fail|error' | head -3
  fi
  rm -f /tmp/run_all.$$.log
done

echo "----"
echo "suites: pass=$pass fail=$fail skip=$skip"
[ "$fail" -eq 0 ] || { echo "failed:$failed"; exit 1; }
