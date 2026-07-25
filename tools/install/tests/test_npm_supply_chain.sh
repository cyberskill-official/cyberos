#!/usr/bin/env bash
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }
WF="$repo/.github/workflows/npm-supply-chain.yml"
CHK="$repo/tools/install/check-npm-supply-chain.sh"
FIX="$repo/tools/install/tests/fixtures/npm-supply-chain"
LOGS=()
cleanup() { rm -f "${LOGS[@]:-}"; }
trap cleanup EXIT
mklog() { local f; f="$(mktemp "${TMPDIR:-/tmp}/npm-sc.XXXXXX")"; LOGS+=("$f"); printf '%s' "$f"; }
t_workflow_declared() {
  [ -f "$WF" ] || { fail t_workflow_declared "missing workflow"; return; }
  grep -q 'pull_request' "$WF" && grep -q 'push' "$WF" && grep -q 'check-npm-supply-chain.sh' "$WF" \
    && grep -q 'ubuntu-latest' "$WF" && ! grep -q 'continue-on-error:\s*true' "$WF" \
    && ! grep -q 'branches: \[main\]' "$WF" \
    && ok t_workflow_declared || fail t_workflow_declared "workflow shape wrong"
}
t_docs_tools_package_json() {
  local p="$repo/tools/install/docs-tools/package.json"
  [ -f "$p" ] && grep -q '"private"[[:space:]]*:[[:space:]]*true' "$p" && ok t_docs_tools_package_json || fail t_docs_tools_package_json "missing"
}
t_license_allowlist_exists() {
  [ -f "$repo/tools/install/npm-license-allowlist.txt" ] && grep -q '^MIT$' "$repo/tools/install/npm-license-allowlist.txt" \
    && ok t_license_allowlist_exists || fail t_license_allowlist_exists "missing"
}
t_planted_critical_fails() {
  chmod +x "$CHK"
  local log; log="$(mklog)"
  if bash "$CHK" --audit-json "$FIX/critical-audit.json" >"$log" 2>&1; then fail t_planted_critical_fails "expected fail"; else ok t_planted_critical_fails; fi
}
t_planted_gpl_fails() {
  local log; log="$(mklog)"
  if bash "$CHK" --license-package "$FIX/gpl-package.json" >"$log" 2>&1; then fail t_planted_gpl_fails "expected fail"; else ok t_planted_gpl_fails; fi
}
t_clean_scopes_pass() {
  local log; log="$(mklog)"
  if bash "$CHK" --root "$repo" >"$log" 2>&1; then ok t_clean_scopes_pass; else fail t_clean_scopes_pass "$(tail -3 "$log")"; fi
}
t_workflow_declared; t_docs_tools_package_json; t_license_allowlist_exists
t_planted_critical_fails; t_planted_gpl_fails; t_clean_scopes_pass
echo "----"; echo "pass=$PASS fail=$FAIL"; [ "$FAIL" -eq 0 ]
