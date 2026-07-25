#!/usr/bin/env bash
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }
CFG="$repo/.github/dependabot.yml"
# Require weekly schedule + groups for each ecosystem block of interest.
_has_weekly_and_groups_for() {
  # $1 = awk pattern matching the update block start line (package-ecosystem value)
  local eco="$1" dir_pat="${2:-}"
  awk -v eco="$eco" -v dir_pat="$dir_pat" '
    BEGIN { in_block=0; weekly=0; groups=0; dir_ok=(dir_pat=="") }
    /^[[:space:]]*-[[:space:]]*package-ecosystem:[[:space:]]*/ {
      if (in_block) { if (weekly && groups && dir_ok) found=1 }
      in_block = ($0 ~ eco)
      weekly=0; groups=0; dir_ok=(dir_pat=="")
      next
    }
    in_block && /interval:[[:space:]]*weekly/ { weekly=1 }
    in_block && /^[[:space:]]*groups:/ { groups=1 }
    in_block && dir_pat != "" && $0 ~ dir_pat { dir_ok=1 }
    END {
      if (in_block && weekly && groups && dir_ok) found=1
      exit !found
    }
  ' "$CFG"
}
t_npm_dirs() {
  [ -f "$CFG" ] || { fail t_npm_dirs "missing"; return; }
  _has_weekly_and_groups_for 'npm' '/tools/install/mcp' \
    && _has_weekly_and_groups_for 'npm' '/tools/install/docs-tools' \
    && ok t_npm_dirs || fail t_npm_dirs "npm weekly/groups"
}
t_actions_ecosystem() {
  _has_weekly_and_groups_for 'github-actions' \
    && ok t_actions_ecosystem || fail t_actions_ecosystem "actions weekly/groups"
}
t_cargo_ecosystem() {
  _has_weekly_and_groups_for 'cargo' '/services' \
    && ok t_cargo_ecosystem || fail t_cargo_ecosystem "cargo weekly/groups"
}
t_groups_present() {
  # Kept as an aggregate AC pointer — each ecosystem predicate already requires groups.
  grep -c '^[[:space:]]*groups:' "$CFG" | grep -Eq '^[1-9]' \
    && ok t_groups_present || fail t_groups_present "groups"
}
t_no_fake_awh_merge() {
  if grep -Eiq 'merge[_-]?(requires|when|if).*awh|awh[_-]?(merge|required|gate)|auto[_-]?merge:.*awh' "$CFG"; then
    fail t_no_fake_awh_merge "fabricated"; else ok t_no_fake_awh_merge; fi
}
t_npm_dirs; t_actions_ecosystem; t_cargo_ecosystem; t_groups_present; t_no_fake_awh_merge
echo "----"; echo "pass=$PASS fail=$FAIL"; [ "$FAIL" -eq 0 ]
