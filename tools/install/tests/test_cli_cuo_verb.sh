#!/usr/bin/env bash
# test_cli_cuo_verb.sh - TASK-IMP-132 contract: `cs cuo` is a redirect stub that
# prints the matching slash command for the four LLM-orchestrated workflows and
# never spawns a subprocess.
#
#   t01_plan_redirect_and_recognition
#   t02_other_three_redirects
#   t03_bare_invocation_lists_and_exits_0
#   t04_unrecognised_name_lists_and_exits_2
#   t05_no_subprocess_spawned
#   t06_docs_describe_as_redirect_and_count_correct
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

NODE="$(command -v node)" || { echo "FATAL: node not on PATH"; exit 1; }

echo "building scratch payload..."
bash "$repo/tools/install/build.sh" "$TMP/payload" >/dev/null 2>&1 || { echo FATAL build; exit 1; }
CLI="$TMP/payload/cli/bin/cli.mjs"

t01_plan_redirect_and_recognition() {
  local out rc
  out="$("$NODE" "$CLI" cuo plan 2>&1)"; rc=$?
  if printf '%s' "$out" | grep -q '/plan' && [ "$rc" -eq 0 ]; then
    ok t01_plan_redirect_and_recognition
  else
    fail t01_plan_redirect_and_recognition "rc=$rc out=$out"
  fi
}

t02_other_three_redirects() {
  local bad="" out rc
  for pair in "create-tasks:/create-tasks" "ship-tasks:/ship-tasks" "improve:/improve"; do
    local name="${pair%%:*}" want="${pair##*:}"
    out="$("$NODE" "$CLI" cuo "$name" 2>&1)"; rc=$?
    printf '%s' "$out" | grep -q "$want" || bad="$bad missing-$want"
    [ "$rc" -eq 0 ] || bad="$bad $name-exit-$rc"
  done
  if [ -z "$bad" ]; then
    ok t02_other_three_redirects
  else
    fail t02_other_three_redirects "$bad"
  fi
}

t03_bare_invocation_lists_and_exits_0() {
  local out rc bad=""
  out="$("$NODE" "$CLI" cuo 2>&1)"; rc=$?
  for n in plan create-tasks ship-tasks improve; do
    printf '%s' "$out" | grep -qw "$n" || bad="$bad missing-$n"
  done
  [ "$rc" -eq 0 ] || bad="$bad exit-$rc"
  if [ -z "$bad" ]; then
    ok t03_bare_invocation_lists_and_exits_0
  else
    fail t03_bare_invocation_lists_and_exits_0 "$bad :: $out"
  fi
}

t04_unrecognised_name_lists_and_exits_2() {
  local out rc bad=""
  out="$("$NODE" "$CLI" cuo nonexistent-workflow 2>&1)"; rc=$?
  for n in plan create-tasks ship-tasks improve; do
    printf '%s' "$out" | grep -qw "$n" || bad="$bad missing-$n"
  done
  [ "$rc" -eq 2 ] || bad="$bad exit-$rc-want-2"
  if [ -z "$bad" ]; then
    ok t04_unrecognised_name_lists_and_exits_2
  else
    fail t04_unrecognised_name_lists_and_exits_2 "$bad :: $out"
  fi
}

t05_no_subprocess_spawned() {
  local stub="$TMP/tripwire-bin"; mkdir -p "$stub"
  # Tripwire stand-ins: write a marker if ever invoked.
  cat >"$stub/python3" <<EOF
#!/bin/bash
echo tripped >"$TMP/python3.marker"
exit 0
EOF
  cat >"$stub/bash" <<EOF
#!/bin/bash
echo tripped >"$TMP/bash.marker"
# Still need to be able to... wait, we must NOT be invoked at all.
exit 0
EOF
  chmod +x "$stub/python3" "$stub/bash"
  rm -f "$TMP/python3.marker" "$TMP/bash.marker"
  # Absolute node; PATH only has tripwires so a spawn would hit them.
  local node_dir; node_dir="$(dirname "$NODE")"
  for name in plan create-tasks ship-tasks improve; do
    PATH="$stub:$node_dir" "$NODE" "$CLI" cuo "$name" >/dev/null 2>&1 || true
  done
  if [ ! -f "$TMP/python3.marker" ] && [ ! -f "$TMP/bash.marker" ]; then
    ok t05_no_subprocess_spawned
  else
    fail t05_no_subprocess_spawned "tripwire fired (python3=$([ -f "$TMP/python3.marker" ] && echo yes || echo no) bash=$([ -f "$TMP/bash.marker" ] && echo yes || echo no))"
  fi
}

t06_docs_describe_as_redirect_and_count_correct() {
  local help_out index_md="$repo/tools/install/docs/index.md" bad=""
  help_out="$(cd "$TMP/payload" && bash help.sh 2>&1)"
  # cuo + redirect-describing word on same or adjacent line in help.sh output
  if ! printf '%s\n' "$help_out" | awk '
    /cuo/ { hit=1; if ($0 ~ /redirect/) ok=1; getline; if ($0 ~ /redirect/) ok=1 }
    END { exit !(hit && ok) }
  '; then
    bad="$bad help.sh-cuo-redirect"
  fi
  # Same for docs/index.md: find the cuo line and require redirect nearby
  if ! awk '
    /cuo/ {
      hit=1
      if ($0 ~ /redirect/) ok=1
      prev=$0
      if (NR>1 && getline > 0 && $0 ~ /redirect/) ok=1
    }
    END { exit !(hit && ok) }
  ' "$index_md"; then
    # Also accept redirect on the SAME line as cuo (the usual case).
    if grep -E 'cuo.*redirect|redirect.*cuo' "$index_md" >/dev/null; then
      :
    else
      bad="$bad index.md-cuo-redirect"
    fi
  fi
  # Command-count sentence: "the same N commands" must match the listed verb count
  # (ten once memory+cuo land), not a stale "eight".
  local sentence count listed
  sentence="$(grep -E 'same [a-z0-9-]+ commands are' "$index_md" | head -1)"
  [ -n "$sentence" ] || bad="$bad missing-count-sentence"
  count="$(printf '%s' "$sentence" | grep -oE 'same [a-z0-9-]+ commands' | head -1 | awk '{print $2}')"
  # Map word numbers / digits
  case "$count" in
    eight) count=8 ;; nine) count=9 ;; ten) count=10 ;; eleven) count=11 ;;
  esac
  listed="$(printf '%s' "$sentence" | grep -oE '`[a-z-]+`' | sed 's/`//g' | grep -E '^(install|uninstall|version|status|create|gates|mcp|help|memory|cuo)$' | sort -u | wc -l | tr -d ' ')"
  # Also count top-level verbs recognised by the built CLI usage text.
  local usage_verbs
  usage_verbs="$("$NODE" "$CLI" -h 2>&1 | awk '/^  [a-z]/ {print $1}' | grep -E '^(install|uninstall|version|status|create|gates|mcp|help|memory|cuo)$' | sort -u | wc -l | tr -d ' ')"
  if [ "$count" != "$listed" ] || [ "$count" != "$usage_verbs" ]; then
    bad="$bad count-mismatch(sentence=$count listed=$listed usage=$usage_verbs)"
  fi
  if [ -z "$bad" ]; then
    ok t06_docs_describe_as_redirect_and_count_correct
  else
    fail t06_docs_describe_as_redirect_and_count_correct "$bad :: $sentence"
  fi
}

t01_plan_redirect_and_recognition
t02_other_three_redirects
t03_bare_invocation_lists_and_exits_0
t04_unrecognised_name_lists_and_exits_2
t05_no_subprocess_spawned
t06_docs_describe_as_redirect_and_count_correct

echo
echo "pass=$PASS fail=$FAIL"
[ "$FAIL" -eq 0 ]
