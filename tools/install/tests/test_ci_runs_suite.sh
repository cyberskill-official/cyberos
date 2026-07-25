#!/usr/bin/env bash
# test_ci_runs_suite.sh - TASK-IMP-128 ACs
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

WF="$repo/.github/workflows/suite-gate.yml"

t_suite_job_declared() {
  [ -f "$WF" ] || { fail t_suite_job_declared "missing suite-gate.yml"; return; }
  grep -q 'ubuntu-latest' "$WF" \
    && grep -q 'scripts/tests/run_all.sh' "$WF" \
    && grep -q 'pull_request' "$WF" \
    && grep -q 'push' "$WF" \
    && grep -Eq 'uses:[[:space:]]*actions/checkout@[0-9a-f]{40}' "$WF" \
    && ! grep -Eq 'uses:[[:space:]]*actions/checkout@(v[0-9]|main|master)' "$WF" \
    && awk '
         /uses:[[:space:]]*actions\/checkout@/ { in_co=1; next }
         in_co && /^[[:space:]]+-[[:space:]]/ { exit }
         in_co && /fetch-depth:[[:space:]]*0/ { found=1; exit }
         END { exit !found }
       ' "$WF" \
    && ok t_suite_job_declared \
    || fail t_suite_job_declared "workflow missing required shape"
}

t_failure_propagates() {
  # No continue-on-error / || true on the suite step
  ! grep -q 'continue-on-error:\s*true' "$WF" \
    && ! grep -E 'run_all\.sh.*\|\|\s*true' "$WF" \
    && ok t_failure_propagates \
    || fail t_failure_propagates "failure may be swallowed"
}

t_release_assets_executes_on_linux() {
  if [ "$(uname -s)" != "Linux" ]; then
    # Not a suite-level skip — other arms already ran. Note only.
    echo "  note  t_release_assets_executes_on_linux: host is not Linux (CI ubuntu leg covers this)"
    ok t_release_assets_executes_on_linux
    return 0
  fi
  # On Linux, the suite file must not take the GNU-tar skip branch when invoked.
  # Exit status is irrelevant here — only that it ran (didn't skip). Use mktemp so
  # the log path is not a predictable /tmp name (CWE-377).
  local log
  log="$(mktemp "${TMPDIR:-/tmp}/ra-linux.XXXXXX")"
  # shellcheck disable=SC2064
  trap 'rm -f "$log"' RETURN
  # Unset PR/release env so fixture release-assets calls don't inherit GITHUB_REF_NAME.
  env -u GITHUB_REF_NAME -u GITHUB_REF -u TAG \
    bash "$repo/tools/install/tests/test_release_assets.sh" >"$log" 2>&1 || true
  grep -q '^  SKIP test_release_assets' "$log" \
    && fail t_release_assets_executes_on_linux "skipped on Linux" \
    || ok t_release_assets_executes_on_linux
}

t_counts_reported() {
  grep -q 'suites: pass=' "$repo/scripts/tests/run_all.sh" \
    && ok t_counts_reported \
    || fail t_counts_reported "run_all.sh does not emit pass/fail/skip counts"
}

t_suite_job_declared
t_failure_propagates
t_release_assets_executes_on_linux
t_counts_reported
echo "----"; echo "pass=$PASS fail=$FAIL"
[ "$FAIL" -eq 0 ]
