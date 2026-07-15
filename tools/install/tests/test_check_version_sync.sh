#!/usr/bin/env bash
# test_check_version_sync.sh - TASK-IMP-068 §5 verification suite (t01-t10 -> AC 1-10).
# Standalone bash, no framework. Run: bash tools/install/tests/test_check_version_sync.sh
set -uo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
CHECK="$repo/tools/install/check-version-sync.sh"
BUILD="$repo/tools/install/build.sh"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0

ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

echo "building scratch payload..."
bash "$BUILD" "$TMP/payload" >/dev/null 2>&1 || { echo "FATAL: scratch build failed"; exit 1; }

t01_fresh_build_syncs() {                                             # AC 1
  # Count NOT pinned. This asserted "across 6 artifacts" and went red at dd9070e33
  # (TASK-IMP-080), which added a 7th stamped artifact and updated the comparator's own
  # header comment to "across 7 artifacts" — but not this assert. Nobody noticed: this
  # file is in no gate.
  #
  # AC 1 is "a fresh build is in sync", not "there are exactly six artifacts". Pinning a
  # number the tool is expected to grow guarantees the next artifact breaks the test —
  # the same mistake as pinning status-hub@1 in test_templates_module.sh t02. Assert the
  # shape and rc; t02 below is what proves each artifact is individually guarded.
  out="$(bash "$CHECK" "$TMP/payload")" && rc=0 || rc=$?
  [ "$rc" -eq 0 ] && echo "$out" | grep -qE "sync OK [0-9]+\.[0-9]+\.[0-9]+ across [0-9]+ artifacts" \
    && ok t01 || fail t01 "rc=$rc out=$out"
}

t02_each_artifact_guarded() {                                         # AC 2
  local all=1
  # bash 3.2 SAFE. This was `declare -A tamper=(...)`, a bash-4 associative array. macOS
  # ships bash 3.2 (GPLv2, frozen 2007), where `declare -A` is a syntax error — so on the
  # machine this repo is developed on, t02 never ran at all. Exactly what made
  # check-chain-coverage.sh a no-op on macOS. A `name|command` list plus a case split is
  # ordered, readable, and runs everywhere.
  #
  # `sed -i EXPR file` is likewise GNU-only: BSD/macOS sed reads the next arg as the
  # backup suffix, so the tamper would silently not apply and t02 would report a false
  # PASS — a gate that cannot fail. `-i.bak` + cleanup is accepted by both.
  local tamper='VERSION|echo 9.9.9 > "$P/VERSION"
plugin/.claude-plugin/plugin.json|_bump_json "$P/plugin/.claude-plugin/plugin.json"
.claude-plugin/marketplace.json|_bump_json "$P/.claude-plugin/marketplace.json"
mcp/package.json|_bump_json "$P/mcp/package.json"
manifest.yaml|sed -i.bak "s/^cyberos_version: [0-9.]*/cyberos_version: 9.9.9/" "$P/manifest.yaml" && rm -f "$P/manifest.yaml.bak"'

  _bump_json() { sed -i.bak 's/"version": "[0-9.]*"/"version": "9.9.9"/' "$1" && rm -f "$1.bak"; }

  local art cmd line
  while IFS= read -r line; do
    art="${line%%|*}"; cmd="${line#*|}"
    P="$TMP/t02"; rm -rf "$P"; cp -r "$TMP/payload" "$P"
    eval "$cmd"
    # The tamper MUST have applied, or the assert below tests nothing and reports a pass.
    grep -q '9\.9\.9' "$P/$art" || { fail "t02[$art]" "tamper did not apply — sed portability?"; all=0; continue; }
    out="$(bash "$CHECK" "$P" 2>&1)" && rc=0 || rc=$?
    if [ "$rc" -ne 10 ] || ! echo "$out" | grep -q "DRIFT $P/$art"; then
      fail "t02[$art]" "rc=$rc out=$out"; all=0
    fi
    n="$(echo "$out" | grep -c '^DRIFT ')"
    [ "$n" -eq 1 ] || { fail "t02[$art]" "expected 1 DRIFT line, got $n"; all=0; }
  done <<EOF
$tamper
EOF
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
  local F="$TMP/fakerepo"; rm -rf "$F"; mkdir -p "$F/tools/install"
  cp "$BUILD" "$F/tools/install/build.sh"
  echo banana > "$F/VERSION"
  bash "$F/tools/install/build.sh" "$F/dist" >/dev/null 2>&1 && fail "t04[banana]" "build passed" || rc=$?
  [ "$rc" -eq 2 ] && [ ! -d "$F/dist" ] || { fail "t04[banana]" "rc=$rc distExists=$([ -d "$F/dist" ] && echo y)"; return; }
  echo "1.7.1-rc1" > "$F/VERSION"   # pre-release rejected (edge row 4)
  bash "$F/tools/install/build.sh" "$F/dist" >/dev/null 2>&1 && { fail "t04[rc1]" "build passed"; return; }
  rm "$F/VERSION"
  bash "$F/tools/install/build.sh" "$F/dist" >/dev/null 2>&1 && { fail "t04[absent]" "build passed"; return; }
  ok t04
}

t05_no_fallback_left() {                                              # AC 5
  grep -q 'echo 0\.0\.0' "$BUILD" && fail t05 "0.0.0 fallback still present" || ok t05
}

t06_workflow_shape() {                                                # AC 6
  local W="$repo/.github/workflows/payload-gate.yml" all=1
  [ -f "$W" ] || { fail t06 "missing workflow"; return; }
  for pat in 'name: payload-gate' 'pull_request:' 'push:' 'tools/install/\*\*' 'modules/skill/\*\*' 'modules/cuo/\*\*' "'VERSION'" 'build.sh "\$RUNNER_TEMP/payload"' 'check-version-sync.sh "\$RUNNER_TEMP/payload"'; do
    grep -q "$pat" "$W" || { fail t06 "missing: $pat"; all=0; }
  done
  if command -v python3 >/dev/null && python3 -c "import yaml" 2>/dev/null; then
    python3 -c "import yaml,sys; yaml.safe_load(open('$W'))" || { fail t06 "yaml parse"; all=0; }
  fi
  [ "$all" -eq 1 ] && ok t06
}

hook_fixture() { # builds a fixture repo with the real wrapper + recording stubs
  local F="$1"; rm -rf "$F"
  mkdir -p "$F/.githooks" "$F/.pre-commit-hooks" "$F/tools/install" "$F/modules/skill" "$F/docs"
  (cd "$F" && git init -q && git config user.email t@t && git config user.name t && git config core.hooksPath .githooks)
  cp "$repo/.githooks/pre-commit" "$F/.githooks/pre-commit"; chmod +x "$F/.githooks/pre-commit"
  printf '#!/usr/bin/env bash\necho engine >> "%s/calls.log"\nexit ${ENGINE_RC:-0}\n' "$F" > "$F/.pre-commit-hooks/cyberos-payload-build.sh"
  printf '#!/usr/bin/env bash\necho check >> "%s/calls.log"\nexit 0\n' "$F" > "$F/tools/install/check-version-sync.sh"
  # EVERY command the real hook invokes needs a stub here, or the fixture aborts on a
  # missing file and the assert reports something unrelated. This fixture copies the real
  # .githooks/pre-commit but stubs only what it called ON THE DAY IT WAS WRITTEN, so every
  # new gate added to that hook silently breaks t07/t08 until someone re-stubs it.
  #
  # Added 2026-07-15, both by me, both unnoticed for hours because this file is in no gate:
  #   no-legacy-fr-vocabulary.sh  — unconditional, so it broke EVERY hook_fixture commit
  #   scripts/tests/run_all.sh    — fires on ^modules/skill/, which is exactly what t07 stages
  # SILENT stubs — they must not touch calls.log. t07 uses that file to decide whether the
  # STATUS-SYNC engine fired, and asserts it stays absent on a non-trigger path. The vocab
  # gate is unconditional, so a logging stub writes on every commit and t07 reads it as
  # "engine fired on a non-trigger path" — a real assert failing on a fixture artefact.
  # Stub what the hook calls; log only what the test is actually measuring.
  mkdir -p "$F/scripts/tests"
  printf '#!/usr/bin/env bash\nexit 0\n' > "$F/.pre-commit-hooks/no-legacy-fr-vocabulary.sh"
  printf '#!/usr/bin/env bash\nexit 0\n' > "$F/scripts/tests/run_all.sh"
  chmod +x "$F/.pre-commit-hooks/cyberos-payload-build.sh" "$F/tools/install/check-version-sync.sh" \
           "$F/.pre-commit-hooks/no-legacy-fr-vocabulary.sh" "$F/scripts/tests/run_all.sh"
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
  # The needle below names a STALE CLAIM that must stay absent -- `! grep -q`. It is not
  # our vocabulary, so the init->install rename must NOT touch it: renaming the needle
  # would leave the real stale wording free to return with nothing watching. Same rule as
  # audit-fleet's `[ ! -f "$cy/init.sh" ]` and the no-legacy-fr-vocabulary gate filename:
  # a check that forbids a string has to spell that string.
  local R="$repo/docs/deploy/RELEASE.md"
  grep -q 'payload-gate.yml' "$R" && grep -q '.githooks/pre-commit' "$R" \
    && ! grep -q 'hook rebuilds `dist/cyberos` so the init payload always matches' "$R" \
    && ok t09 || fail t09 "enforcement wording missing or stale claim remains"
}

t10_version_yml_inline_check() {                                      # AC 10
  local W="$repo/.github/workflows/version.yml"
  local b c p
  b="$(grep -n 'tools/install/build.sh' "$W" | head -1 | cut -d: -f1)"
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
