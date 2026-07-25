#!/usr/bin/env bash
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }
CFG="$repo/.github/dependabot.yml"
t_npm_dirs() {
  [ -f "$CFG" ] && grep -q '/tools/install/mcp' "$CFG" && grep -q '/tools/install/docs-tools' "$CFG" \
    && grep -q 'package-ecosystem: npm' "$CFG" && ok t_npm_dirs || fail t_npm_dirs "npm"
}
t_actions_ecosystem() { grep -q 'package-ecosystem: github-actions' "$CFG" && ok t_actions_ecosystem || fail t_actions_ecosystem "actions"; }
t_cargo_ecosystem() { grep -q 'package-ecosystem: cargo' "$CFG" && grep -q '/services' "$CFG" && ok t_cargo_ecosystem || fail t_cargo_ecosystem "cargo"; }
t_groups_present() { grep -q '^[[:space:]]*groups:' "$CFG" && ok t_groups_present || fail t_groups_present "groups"; }
t_no_fake_awh_merge() {
  if grep -Eiq 'merge[_-]?(requires|when|if).*awh|awh[_-]?(merge|required|gate)|auto[_-]?merge:.*awh' "$CFG"; then
    fail t_no_fake_awh_merge "fabricated"; else ok t_no_fake_awh_merge; fi
}
t_npm_dirs; t_actions_ecosystem; t_cargo_ecosystem; t_groups_present; t_no_fake_awh_merge
echo "----"; echo "pass=$PASS fail=$FAIL"; [ "$FAIL" -eq 0 ]
