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
  # Prefer OSS binary over gitleaks-action (org license required). Checkout must stay SHA-pinned.
  grep -E 'actions/checkout@[0-9a-f]{40}' "$WF" >/dev/null \
    && grep -q 'check-secrets.sh' "$WF" \
    && grep -q 'gitleaks_' "$WF" \
    && grep -q 'sha256sum -c' "$WF" \
    && ok t_action_sha_pinned \
    || fail t_action_sha_pinned "not pinned / not using check-secrets.sh / missing checksum"
}
t_allowlist_present() {
  [ -f "$CFG" ] && grep -q 'useDefault = true' "$CFG" \
    && grep -q 'docs/status/' "$CFG" && grep -q 'tools/caf/core/evals/fixtures/' "$CFG" \
    && ok t_allowlist_present || fail t_allowlist_present "incomplete"
}
t_pre_push_soft_skip() {
  grep -q 'check-secrets.sh' "$HOOK" && grep -q '\-\-soft' "$HOOK" \
    && grep -q '\[ -f "\$ROOT/tools/install/check-secrets.sh" \]' "$HOOK" \
    && ok t_pre_push_soft_skip || fail t_pre_push_soft_skip "missing"
}
t_probe_fixture_fails() {
  chmod +x "$CHK"
  if ! command -v gitleaks >/dev/null 2>&1; then echo "  note  gitleaks missing"; ok t_probe_fixture_fails; return; fi
  local log
  log="$(mktemp "${TMPDIR:-/tmp}/sec-probe.XXXXXX")"
  # shellcheck disable=SC2064
  trap 'rm -f "$log"' RETURN
  if bash "$CHK" --probe-fixture >"$log" 2>&1; then ok t_probe_fixture_fails; else fail t_probe_fixture_fails "$(tail -3 "$log")"; fi
}
t_allowlist_path_scoped() {
  # Path allowlist must not suppress a planted secret outside allowlisted trees.
  if ! command -v gitleaks >/dev/null 2>&1; then echo "  note  gitleaks missing"; ok t_allowlist_path_scoped; return; fi
  local tmp log
  tmp="$(mktemp -d "${TMPDIR:-/tmp}/sec-allow.XXXXXX")"
  log="$(mktemp "${TMPDIR:-/tmp}/sec-allow-log.XXXXXX")"
  # shellcheck disable=SC2064
  trap 'rm -rf "$tmp"; rm -f "$log"' RETURN
  mkdir -p "$tmp/docs/status" "$tmp/elsewhere"
  printf '%s\n' \
    'aws_access_key_id = AKIA0B1C2D3E4F5G6H7J' \
    'aws_secret_access_key = wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLZKEY' \
    > "$tmp/elsewhere/planted.env"
  printf '%s\n' \
    'aws_access_key_id = AKIA0B1C2D3E4F5G6H7J' \
    'aws_secret_access_key = wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLZKEY' \
    > "$tmp/docs/status/planted.env"
  cp "$CFG" "$tmp/.gitleaks.toml"
  local report="$tmp/report.json"
  set +e
  gitleaks dir --no-banner --exit-code=1 --config "$tmp/.gitleaks.toml" \
    --report-format json --report-path "$report" \
    "$tmp" >"$log" 2>&1
  rc=$?
  set -e
  if [ "$rc" -eq 0 ]; then
    fail t_allowlist_path_scoped "planted secret outside allowlist not detected"
    return
  fi
  if ! grep -Eq 'elsewhere/planted\.env|wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLZKEY' "$report"; then
    fail t_allowlist_path_scoped "report missing non-allowlisted path"
    return
  fi
  ok t_allowlist_path_scoped
}
t_workflow_hard_fail; t_action_sha_pinned; t_allowlist_present; t_pre_push_soft_skip
t_probe_fixture_fails; t_allowlist_path_scoped
echo "----"; echo "pass=$PASS fail=$FAIL"; [ "$FAIL" -eq 0 ]
