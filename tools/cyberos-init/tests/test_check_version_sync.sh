#!/usr/bin/env bash
# test_check_version_sync.sh - FR-IMP-068 §5 verification suite (t01-t10 -> AC 1-10).
# Standalone bash, no framework. Run: bash tools/cyberos-init/tests/test_check_version_sync.sh
set -uo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
CHECK="$repo/tools/cyberos-init/check-version-sync.sh"
BUILD="$repo/tools/cyberos-init/build.sh"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0

ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

echo "building scratch payload..."
bash "$BUILD" "$TMP/payload" >/dev/null 2>&1 || { echo "FATAL: scratch build failed"; exit 1; }

t01_fresh_build_syncs() {                                             # AC 1
  out="$(bash "$CHECK" "$TMP/payload")" && rc=0 || rc=$?
  [ "$rc" -eq 0 ] && echo "$out" | grep -q "sync OK .* across 6 artifacts" \
    && ok t01 || fail t01 "rc=$rc out=$out"
}

t02_each_artifact_guarded() {                                         # AC 2
  local all=1
  declare -A tamper=(
    [VERSION]='echo 9.9.9 > "$P/VERSION"'
    [plugin/.claude-plugin/plugin.json]='sed -i "s/\"version\": \"[0-9.]*\"/\"version\": \"9.9.9\"/" "$P/plugin/.claude-plugin/plugin.json"'
    [.claude-plugin/marketplace.json]='sed -i "s/\"version\": \"[0-9.]*\"/\"version\": \"9.9.9\"/" "$P/.claude-plugin/marketplace.json"'
    [mcp/package.json]='sed -i "s/\"version\": \"[0-9.]*\"/\"version\": \"9.9.9\"/" "$P/mcp/package.json"'
    [manifest.yaml]='sed -i "s/^cyberos_version: [0-9.]*/cyberos_version: 9.9.9/" "$P/manifest.yaml"'
  )
  for art in "${!tamper[@]}"; do
    P="$TMP/t02"; rm -rf "$P"; cp -r "$TMP/payload" "$P"
    eval "${tamper[$art]}"
    out="$(bash "$CHECK" "$P" 2>&1)" && rc=0 || rc=$?
    if [ "$rc" -ne 10 ] || ! echo "$out" | grep -q "DRIFT $P/$art"; then
      fail "t02[$art]" "rc=$rc out=$out"; all=0
    fi
    n="$(echo "$out" | grep -c '^DRIFT ')"
    [ "$n" -eq 1 ] || { fail "t02[$art]" "expected 1 DRIFT line, got $n"; all=0; }
  done
  # missing artifact (edge rows 3/9): removed file -> exit 2, never a pass
  P="$TMP/t02"; rm -rf "$P"; cp -r "$TMP/payload" "$P"; rm "$P/manifest.yaml"
  bash "$CHECK" "$P" >/dev/null 2>&1 && { fail "t02[missing]" "passed with missing manifest"; all=0; } || rc=$?
  [ "${rc:-0}" -eq 2 ] || { fail "t02[missing]" "rc=$rc, want 2"; all=0; }
  [ "$all" -eq 1 ] && ok t02
}

t03_sealed_zip_checked() {                                            # AC 3
  P="$TMP/t03"; rm -rf "$P"; cp -r "$TMP/payload" "$P"
  local d="$TMP/t03zip"; rm -rf "$d"; mkdir -p "$d/.claude-plugin"
  unzip -p "$P/cyberos.plugin" .claude-plugin/plugin.json | sed 's/"version": "[0-9.]*"/"version": "0.0.1"/' > "$d/.claude-plugin/plugin.json"
  (cd "$d" && zip -q "$P/cyberos.plugin" .claude-plugin/plugin.json)   # overwrite entry inside the zip only
  out="$(bash "$CHECK" "$P" 2>&1)" && rc=0 || rc=$?
  [ "$rc" -eq 10 ] && echo "$out" | grep -q 'cyberos.plugin!.claude-plugin/plugin.json' \
    && ok t03 || fail t03 "rc=$rc out=$out"
  # corrupted zip (edge row 7): truncated bundle -> exit 2
  P="$TMP/t03b"; rm -rf "$P"; cp -r "$TMP/payload" "$P"; head -c 100 "$TMP/payload/cyberos.plugin" > "$P/cyberos.plugin"
  bash "$CHECK" "$P" >/dev/null 2>&1; rc=$?
  [ "$rc" -eq 2 ] || fail "t03[corrupt]" "rc=$rc, want 2"
}

t04_invalid_version_refused() {                                       # AC 4
  local F="$TMP/fakerepo"; rm -rf "$F"; mkdir -p "$F/tools/cyberos-init"
  cp "$BUILD" "$F/tools/cyberos-init/build.sh"
  echo banana > "$F/VERSION"
  bash "$F/tools/cyberos-init/build.sh" "$F/dist" >/dev/null 2>&1 && fail "t04[banana]" "build passed" || rc=$?
  [ "$rc" -eq 2 ] && [ ! -d "$F/dist" ] || { fail "t04[banana]" "rc=$rc distExists=$([ -d "$F/dist" ] && echo y)"; return; }
  echo "1.7.1-rc1" > "$F/VERSION"   # pre-release rejected (edge row 4)
  bash "$F/tools/cyberos-init/build.sh" "$F/dist" >/dev/null 2>&1 && { fail "t04[rc1]" "build passed"; return; }
  rm "$F/VERSION"
  bash "$F/tools/cyberos-init/build.sh" "$F/dist" >/dev/null 2>&1 && { fail "t04[absent]" "build passed"; return; }
  ok t04
}

t05_no_fallback_left() {                                              # AC 5
  grep -q 'echo 0\.0\.0' "$BUILD" && fail t05 "0.0.0 fallback still present" || ok t05
}

t06_workflow_shape() {                                                # AC 6
  local W="$repo/.github/workflows/payload-gate.yml" all=1
  [ -f "$W" ] || { fail t06 "missing workflow"; return; }
  for pat in 'name: payload-gate' 'pull_request:' 'push:' 'tools/cyberos-init/\*\*' 'modules/skill/\*\*' 'modules/cuo/\*\*' "'VERSION'" 'build.sh "\$RUNNER_TEMP/payload"' 'check-version-sync.sh "\$RUNNER_TEMP/payload"'; do
    grep -q "$pat" "$W" || { fail t06 "missing: $pat"; all=0; }
  done
  if command -v python3 >/dev/null && python3 -c "import yaml" 2>/dev/null; then
    python3 -c "import yaml,sys; yaml.safe_load(open('$W'))" || { fail t06 "yaml parse"; all=0; }
  fi
  [ "$all" -eq 1 ] && ok t06
}

hook_fixture() { # builds a fixture repo with the real wrapper + recording stubs
  local F="$1"; rm -rf "$F"
  mkdir -p "$F/.githooks" "$F/.pre-commit-hooks" "$F/tools/cyberos-init" "$F/modules/skill" "$F/docs"
  (cd "$F" && git init -q && git config user.email t@t && git config user.name t && git config core.hooksPath .githooks)
  cp "$repo/.githooks/pre-commit" "$F/.githooks/pre-commit"; chmod +x "$F/.githooks/pre-commit"
  printf '#!/usr/bin/env bash\necho engine >> "%s/calls.log"\nexit ${ENGINE_RC:-0}\n' "$F" > "$F/.pre-commit-hooks/cyberos-payload-build.sh"
  printf '#!/usr/bin/env bash\necho check >> "%s/calls.log"\nexit 0\n' "$F" > "$F/tools/cyberos-init/check-version-sync.sh"
  chmod +x "$F/.pre-commit-hooks/cyberos-payload-build.sh" "$F/tools/cyberos-init/check-version-sync.sh"
  (cd "$F" && git add -A && git -c core.hooksPath=/dev/null commit -qm init)  # baseline commit without hooks
  rm -f "$F/calls.log"
}

t07_hook_trigger_matrix() {                                           # AC 7
  local F="$TMP/hookrepo"; hook_fixture "$F"
  (cd "$F" && echo x > modules/skill/a.md && git add modules/skill/a.md && git commit -qm "feat: trigger") || { fail t07 "trigger commit failed"; return; }
  [ -f "$F/calls.log" ] && grep -q engine "$F/calls.log" && grep -q check "$F/calls.log" || { fail t07 "hook did not fire on trigger path"; return; }
  rm -f "$F/calls.log"
  (cd "$F" && echo y > docs/b.md && git add docs/b.md && git commit -qm "docs: no trigger") || { fail t07 "no-trigger commit failed"; return; }
  [ ! -f "$F/calls.log" ] && ok t07 || fail t07 "hook fired on non-trigger path"
}

t08_hook_blocks_on_failure() {                                        # AC 8
  local F="$TMP/hookrepo8"; hook_fixture "$F"
  (cd "$F" && echo z > modules/skill/c.md && git add modules/skill/c.md && ENGINE_RC=1 git commit -qm "feat: should abort") 2>/dev/null && { fail t08 "commit succeeded despite failing rebuild"; return; }
  (cd "$F" && git log --oneline | grep -q "should abort") && fail t08 "commit landed" || ok t08
}

t09_release_md_updated() {                                            # AC 9
  local R="$repo/docs/deploy/RELEASE.md"
  grep -q 'payload-gate.yml' "$R" && grep -q '.githooks/pre-commit' "$R" \
    && ! grep -q 'hook rebuilds `dist/cyberos` so the init payload always matches' "$R" \
    && ok t09 || fail t09 "enforcement wording missing or stale claim remains"
}

t10_version_yml_inline_check() {                                      # AC 10
  local W="$repo/.github/workflows/version.yml"
  local b c p
  b="$(grep -n 'tools/cyberos-init/build.sh' "$W" | head -1 | cut -d: -f1)"
  c="$(grep -n 'check-version-sync.sh' "$W" | head -1 | cut -d: -f1)"
  p="$(grep -n 'Commit the bump back to main' "$W" | head -1 | cut -d: -f1)"
  [ -n "$b" ] && [ -n "$c" ] && [ -n "$p" ] && [ "$b" -lt "$p" ] && [ "$c" -lt "$p" ] \
    && ok t10 || fail t10 "inline proof steps missing or after the push step (b=$b c=$c p=$p)"
}

t01_fresh_build_syncs; t02_each_artifact_guarded; t03_sealed_zip_checked; t04_invalid_version_refused
t05_no_fallback_left; t06_workflow_shape; t07_hook_trigger_matrix; t08_hook_blocks_on_failure
t09_release_md_updated; t10_version_yml_inline_check

echo "----"; echo "pass=$PASS fail=$FAIL"
[ "$FAIL" -eq 0 ]
