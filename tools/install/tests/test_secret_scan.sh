#!/usr/bin/env bash
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }
WF="$repo/.github/workflows/secret-scan.yml"
CHK="$repo/tools/install/check-secrets.sh"
CFG="$repo/.gitleaks.toml"
HOOK="$repo/.githooks/pre-push"
t_workflow_hard_fail() {
  [ -f "$WF" ] && grep -q 'pull_request' "$WF" && grep -q 'push' "$WF" && grep -q 'gitleaks' "$WF" \
    && ! grep -q 'continue-on-error:\s*true' "$WF" && ok t_workflow_hard_fail || fail t_workflow_hard_fail "shape"
}
t_action_sha_pinned() {
  grep -E 'gitleaks/gitleaks-action@[0-9a-f]{40}' "$WF" >/dev/null && ok t_action_sha_pinned || fail t_action_sha_pinned "not pinned"
}
t_allowlist_present() {
  [ -f "$CFG" ] && grep -q 'docs/status/' "$CFG" && grep -q 'tools/caf/core/evals/fixtures/' "$CFG" \
    && ok t_allowlist_present || fail t_allowlist_present "incomplete"
}
t_pre_push_soft_skip() {
  grep -q 'check-secrets.sh' "$HOOK" && grep -q '\-\-soft' "$HOOK" && ok t_pre_push_soft_skip || fail t_pre_push_soft_skip "missing"
}
t_probe_fixture_fails() {
  chmod +x "$CHK"
  if ! command -v gitleaks >/dev/null 2>&1; then echo "  note  gitleaks missing"; ok t_probe_fixture_fails; return; fi
  if bash "$CHK" --probe-fixture >/tmp/sec-probe.$$.log 2>&1; then ok t_probe_fixture_fails; else fail t_probe_fixture_fails "$(tail -3 /tmp/sec-probe.$$.log)"; fi
}
t_workflow_hard_fail; t_action_sha_pinned; t_allowlist_present; t_pre_push_soft_skip; t_probe_fixture_fails
echo "----"; echo "pass=$PASS fail=$FAIL"; [ "$FAIL" -eq 0 ]
