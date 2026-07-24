#!/usr/bin/env bash
# TASK-MCP-003 slice 3 / DEC-2362 - CI grep gate for SEP-986 skill IDs.
#
# A cheap, source-side tripwire that complements the runtime validator in
# services/mcp-gateway/src/naming (which rejects non-conforming tool IDs at registration). It scans
# production source for quoted three-part `cyberos.<module>.<tool>` identifiers and FAILS CLOSED on any
# whose module is a real registry module but whose ID breaks the convention - e.g. a reintroduced
# `cyberos.obs.triage` or a bad verb. The point is to catch a hardcoded bad ID in review before it ever
# reaches the gateway.
#
# Scope decision: only IDs whose module is in the 23-module registry are enforced. IDs with any other
# module segment (the `demo` reference fixture, unit-test fixtures like `cyberos.test.*` / `cyberos.nope.*`,
# or a genuinely new module pending a registry RFC) are skipped here - the runtime validator owns those.
# This keeps the gate free of test-fixture false positives while still guarding every real module.
#
# Portability: bash 3.2+ (macOS /bin/bash) and bash 4+/5 (CI ubuntu). No `mapfile` / associative arrays.
#
# Usage:  bash scripts/check_sep986_naming.sh
# Exit:   0 = clean, 1 = at least one violation.
set -eu
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

# Mirror services/mcp-gateway/src/naming: the 23-module registry and the 15 approved verbs.
MODULES="ai auth chat crm cuo doc email esop hr inv kb learn mcp memory obs okr portal proj res rew skill ten time"
VERBS="get list create update delete send fetch sync validate generate execute search replay accept reject"

SHAPE='^cyberos\.[a-z][a-z0-9_]*\.[a-z]+_[a-z][a-z0-9_]*$'

# Collect candidate IDs from PRODUCTION source only. Test code legitimately hardcodes non-conforming IDs
# (the validator's own negative tests, gateway unit fixtures), so we exclude: test paths (tests/ dirs,
# *_test.rs, *_test.py, test_*.py), the inline Rust `#[cfg(test)]` section of each .rs file (tests sit at
# file end by convention), and logger names (logging.getLogger("cyberos.<m>.<x>") collides with the tool
# pattern but is not a tool).
emit_production_lines() {
  while IFS= read -r f; do
    case "$f" in
      *.rs) awk 'index($0, "#[cfg(test)]") { exit } { print }' "$f" ;;
      *)    cat "$f" ;;
    esac
  done < <(find "$ROOT/services" "$ROOT/modules" -type f \( -name '*.rs' -o -name '*.py' \) 2>/dev/null \
             | grep -vE '/tests?/|_test\.(rs|py)$|/test_[^/]*\.py$' || true)
}

# bash 3.2-safe: pipe into a while-read instead of mapfile.
IDS_TMP="$(mktemp)"
trap 'rm -f "$IDS_TMP"' EXIT
emit_production_lines \
  | grep -v 'getLogger' \
  | grep -oE '"cyberos\.[a-z][a-z0-9_]*\.[a-z0-9_]+"' \
  | tr -d '"' | sort -u >"$IDS_TMP" || true

if [ ! -s "$IDS_TMP" ]; then
  echo "OK: no cyberos.<module>.<tool> identifiers found to scan."
  exit 0
fi

fail=0
checked=0
while IFS= read -r id; do
  [ -n "$id" ] || continue
  module="$(printf '%s' "$id" | cut -d. -f2)"
  # Only enforce on real registry modules; skip fixtures / pending-registry modules.
  case " $MODULES " in *" $module "*) ;; *) continue ;; esac
  checked=$((checked + 1))
  if ! printf '%s' "$id" | grep -qE "$SHAPE"; then
    echo "SEP-986 violation (shape): $id"
    fail=1
    continue
  fi
  verb="$(printf '%s' "$id" | cut -d. -f3-)"
  verb="${verb%%_*}"
  case " $VERBS " in
    *" $verb "*) ;;
    *) echo "SEP-986 violation (unapproved verb '$verb'): $id"; fail=1 ;;
  esac
done <"$IDS_TMP"

if [ "$fail" -ne 0 ]; then
  echo "FAIL: non-conforming SEP-986 skill IDs found. Fix the ID, or extend the module registry / verb"
  echo "      enum in services/mcp-gateway/src/naming (governance: module-owner sign-off + RFC)."
  exit 1
fi
echo "OK: $checked registry-module skill IDs scanned, all SEP-986 conforming."
exit 0
