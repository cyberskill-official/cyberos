#!/usr/bin/env bash
# run_all.sh - every shell test in the repo, discovered by glob.
#
# WHY A GLOB AND NOT A LIST: on 2026-07-15 all six shell test files were reachable from no
# gate at all - not .githooks/pre-commit, not scripts/local_verify.sh, not any of the 24 CI
# workflows. Three had been red for months and reported it to nobody:
#
#   test_task_layout.sh      t09 asserted against tools/cyberos-init/init.sh, deleted at
#                            bb0f2392e. `grep` on a missing file returns 1, short-circuits
#                            the && chain into fail. Red since that commit.
#   test_templates_module.sh t02 pinned status-hub@1; ac33beb54 shipped status-hub@2.
#                            t03 knew :html and bare slots; status-hub@2 added :json.
#   test_render_status_hub.sh t03/t05/t07/t08 - stale fixture + a dead FR- regex that
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

# Sub-suites invoked BY another test still run standalone here; double-running is cheap and
# an orphan is not. Add a path here only if a file genuinely cannot run on its own.
SKIP=""

pass=0; fail=0; failed=""
for t in scripts/tests/test_*.sh tools/docs-site/tests/test_*.sh; do
  [ -e "$t" ] || continue
  b="$(basename "$t")"
  case " $SKIP " in *" $b "*) echo "  skip $b"; continue ;; esac
  if timeout 300 bash "$t" >/tmp/run_all.$$.log 2>&1; then
    pass=$((pass+1)); printf '  \033[32mok\033[0m   %s\n' "$b"
  else
    fail=$((fail+1)); failed="$failed $b"
    printf '  \033[31mFAIL\033[0m %s\n' "$b"
    sed 's/^/         /' /tmp/run_all.$$.log | grep -iE 'fail|error' | head -3
  fi
  rm -f /tmp/run_all.$$.log
done

echo "----"
echo "suites: pass=$pass fail=$fail"
[ "$fail" -eq 0 ] || { echo "failed:$failed"; exit 1; }
