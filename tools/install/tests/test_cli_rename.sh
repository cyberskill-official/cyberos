#!/usr/bin/env bash
# test_cli_rename.sh - TASK-IMP-130 contract: the public npm CLI bin renamed cyberos -> cs.
#   t01_bin_renamed_to_cs            a scratch build's package.json declares bin.cs (not
#                                    bin.cyberos) and leaves the package `name` unchanged.
#   t02_usage_text_says_cs           `node cli/bin/cli.mjs --help` describes itself as `cs`,
#                                    not `cyberos`.
#   t03_help_sh_says_cs              `help.sh`'s Channels section describes the npm channel as
#                                    `npx cs <command>`, not `npx cyberos <command>`.
#   t04_docs_sweep_replaced_not_just_removed   help.md / docs/index.md / README.md now carry the
#                                    POSITIVE `cs` replacement text (not just an absence of the
#                                    old string - a deleted example would also pass a
#                                    negative-only check, per TASK-IMP-130 audit ISS-001).
#   t05_stale_domain_fixed           cli.mjs / help.sh / help.md no longer point at the stale
#                                    `cyberos.cyberskill.world` domain; all three now use the
#                                    canonical `os.cyberskill.world/docs`.
#   t06_changelog_entry_present      CHANGELOG.md's TOP entry names both `cyberos` and `cs` and
#                                    says "breaking".
#   t07_memory_module_untouched      this change touches no file under modules/memory/ (the
#                                    separate, PyPI-unpublished `cyberos-memory` console script
#                                    is explicitly out of scope - TASK-IMP-130 §1.7).
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

echo "building scratch payload..."
bash "$repo/tools/install/build.sh" "$TMP/payload" >/dev/null 2>&1 || { echo FATAL build; exit 1; }

t01_bin_renamed_to_cs() {
  local pkg="$TMP/payload/package.json"
  [ -f "$pkg" ] || { fail t01_bin_renamed_to_cs "no package.json at $pkg"; return; }
  local bin_cs bin_cyberos name
  bin_cs="$(node -e 'const p=require(process.argv[1]); process.stdout.write(p.bin && p.bin.cs || "")' "$pkg")"
  bin_cyberos="$(node -e 'const p=require(process.argv[1]); process.stdout.write(p.bin && p.bin.cyberos || "")' "$pkg")"
  name="$(node -e 'const p=require(process.argv[1]); process.stdout.write(p.name || "")' "$pkg")"
  if [ "$bin_cs" = "cli/bin/cli.mjs" ] && [ -z "$bin_cyberos" ] && [ "$name" = "@cyberskill/cyberos" ]; then
    ok t01_bin_renamed_to_cs
  else
    fail t01_bin_renamed_to_cs "bin.cs='$bin_cs' bin.cyberos='$bin_cyberos' name='$name'"
  fi
}

t02_usage_text_says_cs() {
  local out
  out="$(node "$TMP/payload/cli/bin/cli.mjs" --help 2>&1)"
  if printf '%s' "$out" | grep -q 'cs <command>' && ! printf '%s' "$out" | grep -q 'cyberos <command>'; then
    ok t02_usage_text_says_cs
  else
    fail t02_usage_text_says_cs "usage text missing 'cs <command>' or still contains 'cyberos <command>': $out"
  fi
}

t03_help_sh_says_cs() {
  local out
  out="$(cd "$TMP/payload" && bash help.sh 2>&1)"
  if printf '%s' "$out" | grep -q 'npx cs <command>' && ! printf '%s' "$out" | grep -q 'npx cyberos'; then
    ok t03_help_sh_says_cs
  else
    fail t03_help_sh_says_cs "help.sh output missing 'npx cs <command>' or still contains 'npx cyberos': $out"
  fi
}

t04_docs_sweep_replaced_not_just_removed() {
  local help_md="$repo/tools/install/plugin/commands/help.md"
  local index_md="$repo/tools/install/docs/index.md"
  local readme="$repo/tools/install/README.md"
  local bad=""
  grep -q ' cs ' "$help_md" 2>/dev/null || bad="$bad help.md-missing-cs-mention"
  grep -q 'npx cs <command>' "$index_md" 2>/dev/null || bad="$bad index.md-missing-npx-cs"
  grep -q 'npx cs install \[dir\]' "$readme" 2>/dev/null || bad="$bad readme-missing-npx-cs-install"
  # Negative half: a deleted example (rather than an updated one) must not pass this test.
  if grep -l 'cyberos <command>' "$help_md" "$index_md" "$readme" >/dev/null 2>&1; then
    bad="$bad literal-'cyberos <command>'-still-present"
  fi
  if [ -z "$bad" ]; then
    ok t04_docs_sweep_replaced_not_just_removed
  else
    fail t04_docs_sweep_replaced_not_just_removed "$bad"
  fi
}

t05_stale_domain_fixed() {
  local cli="$repo/tools/install/cli/bin/cli.mjs"
  local help_sh="$repo/tools/install/help.sh"
  local help_md="$repo/tools/install/plugin/commands/help.md"
  local bad=""
  for f in "$cli" "$help_sh" "$help_md"; do
    grep -q 'cyberos\.cyberskill\.world' "$f" 2>/dev/null && bad="$bad $f-still-has-stale-domain"
    grep -q 'os\.cyberskill\.world/docs' "$f" 2>/dev/null || bad="$bad $f-missing-canonical-domain"
  done
  if [ -z "$bad" ]; then
    ok t05_stale_domain_fixed
  else
    fail t05_stale_domain_fixed "$bad"
  fi
}

t06_changelog_entry_present() {
  local changelog="$repo/CHANGELOG.md"
  # The "top entry" = everything between the first "## [" heading and the next one.
  local top
  top="$(awk '/^## \[/{n++} n==1' "$changelog")"
  if printf '%s' "$top" | grep -qi 'cyberos' \
     && printf '%s' "$top" | grep -qi '`cs`\|`cs ' \
     && printf '%s' "$top" | grep -qi 'breaking'; then
    ok t06_changelog_entry_present
  else
    fail t06_changelog_entry_present "top CHANGELOG entry missing cyberos/cs/breaking: $top"
  fi
}

t07_memory_module_untouched() {
  local diff
  diff="$(cd "$repo" && git diff --name-only -- modules/memory 2>/dev/null; git status --porcelain -- modules/memory 2>/dev/null)"
  if [ -z "$diff" ]; then
    ok t07_memory_module_untouched
  else
    fail t07_memory_module_untouched "modules/memory has pending changes: $diff"
  fi
}

t01_bin_renamed_to_cs
t02_usage_text_says_cs
t03_help_sh_says_cs
t04_docs_sweep_replaced_not_just_removed
t05_stale_domain_fixed
t06_changelog_entry_present
t07_memory_module_untouched

echo "cli-rename: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]
