#!/usr/bin/env bash
# fleet-init-test.sh — init + audit every repo under CyberSkill + Personal (23 targets).
# Loop: init → audit → print FAIL; optional --fix-once re-install fails only.
# Usage:
#   bash tools/cyberos-init/fleet-init-test.sh [payload-dir]
# Env:
#   FLEET_MAX_ROUNDS=3   re-install/audit rounds on failures
set -uo pipefail

payload="${1:-$(cd "$(dirname "$0")/../../dist/cyberos" && pwd)}"
[ -d "$payload/cuo" ] || { echo "fleet: not a payload: $payload"; exit 2; }
want="$(tr -d ' \n\r' < "$payload/VERSION" 2>/dev/null || echo 1.0.0)"
rounds="${FLEET_MAX_ROUNDS:-3}"

repos=(
  /Users/stephencheng/Projects/CyberSkill/code-audit-field-data
  /Users/stephencheng/Projects/CyberSkill/code-audit-framework
  /Users/stephencheng/Projects/CyberSkill/cyber-click
  /Users/stephencheng/Projects/CyberSkill/cyberos
  /Users/stephencheng/Projects/CyberSkill/design-system
  /Users/stephencheng/Projects/CyberSkill/design-system-audit-framework
  /Users/stephencheng/Projects/CyberSkill/doc-templates
  /Users/stephencheng/Projects/CyberSkill/gam
  /Users/stephencheng/Projects/CyberSkill/landing-page
  /Users/stephencheng/Projects/CyberSkill/sachviet
  /Users/stephencheng/Projects/CyberSkill/shared
  /Users/stephencheng/Projects/CyberSkill/shopass
  /Users/stephencheng/Projects/CyberSkill/ssl
  /Users/stephencheng/Projects/CyberSkill/strategem
  /Users/stephencheng/Projects/CyberSkill/styx
  /Users/stephencheng/Projects/CyberSkill/tamagochi
  /Users/stephencheng/Projects/Personal/3d-preriodic-table
  /Users/stephencheng/Projects/Personal/claude-certified-architect-foundations
  /Users/stephencheng/Projects/Personal/dom-defender
  /Users/stephencheng/Projects/Personal/issue-hunter
  /Users/stephencheng/Projects/Personal/kristen-calendar
  /Users/stephencheng/Projects/Personal/my-cv
  /Users/stephencheng/Projects/Personal/wife-cv
)

audit="$(cd "$(dirname "$0")" && pwd)/audit-fleet.sh"
logdir="${FLEET_LOGDIR:-/tmp/cyberos-fleet-$$}"
mkdir -p "$logdir"
echo "fleet: payload=$payload want=$want repos=${#repos[@]} logs=$logdir"

fail_list=()
for r in "${repos[@]}"; do
  name="$(basename "$(dirname "$r")")/$(basename "$r")"
  if [ ! -d "$r" ]; then
    echo "SKIP  $name (missing dir)"
    continue
  fi
  # Skip pure non-git empty? still try init
  printf '\n=== INSTALL %s ===\n' "$name"
  if out="$(CYBEROS_OFFLINE=1 bash "$payload/install.sh" "$r" 2>&1)"; then
    printf '%s\n' "$out" | sed 's/^/  | /' | tail -30
    echo "  INSTALL: ok" | tee -a "$logdir/ok.txt"
  else
    printf '%s\n' "$out" | sed 's/^/  | /' | tee "$logdir/$(echo "$name" | tr / _).init.log" | tail -40
    echo "  INSTALL: FAILED" | tee -a "$logdir/fail.txt"
    fail_list+=("$r")
  fi
done

echo
echo "=== AUDIT round 1 ==="
# audit-fleet expects roots that contain children — run per-repo wrapper
FAILED=0
: > "$logdir/audit.txt"
for r in "${repos[@]}"; do
  [ -d "$r" ] || continue
  name="$(basename "$(dirname "$r")")/$(basename "$r")"
  # single-repo audit by inventing a temp parent? audit-fleet iterates base/*
  # Call inline checks instead using audit script on parent with filter
  parent="$(dirname "$r")"
  child="$(basename "$r")"
  line="$(bash "$audit" "$want" "$parent" 2>/dev/null | grep -E " $child |/$child " || true)"
  # audit prints CyberSkill/name — match end
  line="$(bash "$audit" "$want" "$parent" 2>&1 | grep "/$child" || true)"
  if echo "$line" | grep -q '^PASS'; then
    echo "$line" | tee -a "$logdir/audit.txt"
  else
    echo "FAIL  $name  ${line:-no-audit-line}" | tee -a "$logdir/audit.txt"
    FAILED=$((FAILED + 1))
    fail_list+=("$r")
  fi
done

# de-dupe fail_list
unique_fails=()
for f in "${fail_list[@]:-}"; do
  skip=0
  for u in "${unique_fails[@]:-}"; do [ "$u" = "$f" ] && skip=1; done
  [ "$skip" = 0 ] && unique_fails+=("$f")
done

round=1
while [ "$round" -lt "$rounds" ] && [ "${#unique_fails[@]}" -gt 0 ]; do
  round=$((round + 1))
  echo
  echo "=== RE-INSTALL fails round $round (${#unique_fails[@]} repos) ==="
  next=()
  for r in "${unique_fails[@]}"; do
    name="$(basename "$(dirname "$r")")/$(basename "$r")"
    echo "--- re-install $name ---"
    if CYBEROS_OFFLINE=1 bash "$payload/install.sh" "$r" >"$logdir/reinit-$(echo "$name" | tr / _).log" 2>&1; then
      parent="$(dirname "$r")"
      child="$(basename "$r")"
      line="$(bash "$audit" "$want" "$parent" 2>&1 | grep "/$child" || true)"
      if echo "$line" | grep -q '^PASS'; then
        echo "PASS  $name (after re-install)"
      else
        echo "FAIL  $name still: $line"
        next+=("$r")
      fi
    else
      echo "FAIL  $name re-install still failing"
      next+=("$r")
    fi
  done
  unique_fails=("${next[@]:-}")
done

echo
echo "=== FLEET SUMMARY ==="
pass_n=$(grep -c '^PASS' "$logdir/audit.txt" 2>/dev/null || echo 0)
fail_n=${#unique_fails[@]}
echo "audit PASS lines≈$pass_n  remaining_fails=$fail_n  logs=$logdir"
if [ "$fail_n" -gt 0 ]; then
  printf 'Still failing:\n'
  printf '  %s\n' "${unique_fails[@]}"
  exit 1
fi
echo "fleet-init-test: all green"
exit 0
