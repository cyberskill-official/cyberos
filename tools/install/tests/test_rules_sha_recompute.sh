#!/usr/bin/env bash
# test_rules_sha_recompute.sh — TASK-IMP-122 §2 suite (t01–t15 → AC 1–15).
#
# rules_sha must be recomputed (not recalled); cone = vendored set; reconciler both directions.
# Builds with CYBEROS_SKIP_RULES_RECONCILE=1 for speed; reconciler exercised separately in t06/t07.
#
# Disk-frugal: builds ONE payload, installs ONE pristine reference ($REF) and ONE mutable
# work tree ($WORK). Drift arms mutate a $WORK file, assert, then restore that file from $PAY.
# This bounds scratch usage to ~two installs regardless of arm count.
#
# Registration: scripts/tests/run_all.sh globs tools/install/tests/test_*.sh.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
RC="$repo/tools/install/lib/rules-cone.sh"
BUILD="$repo/tools/install/build.sh"
TMP="$(mktemp -d)"; trap 'chmod -R u+rwX "$TMP" 2>/dev/null; rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

export CYBEROS_SKIP_RULES_RECONCILE=1
export CYBEROS_SYNC_HOST_PLUGINS=0
export CYBEROS_OFFLINE=1
export CYBEROS_NONINTERACTIVE=1
export CYBEROS_UPDATE_CHECK=0
export CYBEROS_GLOBAL_SKILLS=0
export CYBEROS_NO_MIGRATE=1
export CYBEROS_NO_HOOK=1

echo "building scratch payload (reconcile skipped)..."
bash "$BUILD" "$TMP/payload" >/dev/null 2>&1 || { echo FATAL build; exit 1; }
# shellcheck source=/dev/null
. "$TMP/payload/lib/rules-cone.sh"
PAY="$TMP/payload"
pv="$(tr -d ' \n\r' < "$PAY/VERSION")"

install_into() {
  local r="$1"
  rm -rf "$r"; mkdir -p "$r"
  (cd "$r" && git init -q)
  bash "$PAY/install.sh" "$r" >/dev/null 2>&1
}

echo "installing pristine reference + work machine..."
install_into "$TMP/ref"  || { echo FATAL ref install; exit 1; }
install_into "$TMP/work" || { echo FATAL work install; exit 1; }
REF="$TMP/ref"
WORK="$TMP/work"

# Restore a relative path in $WORK from the pristine payload copy.
restore() { cp "$PAY/$1" "$WORK/.cyberos/$1"; }
# Mutate (append) a vendored file in $WORK.
mutate()  { printf 'mutation-%s\n' "$RANDOM" >> "$WORK/.cyberos/$1"; }

# ── t01: installed side recomputed, not recalled ─────────────────────────────
t01_installed_side_recomputed_not_recalled() {
  local all=1 out
  mutate cuo/ship-tasks.md
  out="$(bash "$PAY/version.sh" "$WORK" 2>/dev/null)"
  echo "$out" | grep -q '^verdict=rules_drift$' || { fail t01 "version.sh: $out"; all=0; }
  out="$(cd "$WORK" && CYBEROS_PAYLOAD="$PAY" CYBEROS_UPDATE_CHECK=always bash "$PAY/lib/update-check.sh" 2>&1)"
  echo "$out" | grep -q 'RULE DRIFT' || { fail t01 "update-check: $out"; all=0; }
  rm -f "$WORK/.cyberos/.update-check-cache"
  # audit-fleet recomputes too
  mkdir -p "$TMP/t01fleet"; cp -R "$WORK" "$TMP/t01fleet/x"
  out="$(CYBEROS_EXPECT_RULES_SHA="$(_rules_sha_of "$PAY")" bash "$repo/tools/install/audit-fleet.sh" "$pv" "$TMP/t01fleet" 2>&1)" || true
  echo "$out" | grep -q 'FAIL' || { fail t01 "audit-fleet did not fail: $out"; all=0; }
  rm -rf "$TMP/t01fleet"
  restore cuo/ship-tasks.md
  # Recall check: rewrite ONLY the stored token; recompute must stay current.
  local tok_before
  tok_before="$(grep '^rules_sha:' "$WORK/.cyberos/manifest.yaml" | awk '{print $2}')"
  sed -i.bak 's/^rules_sha:.*/rules_sha: deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef/' \
    "$WORK/.cyberos/manifest.yaml"
  out="$(bash "$PAY/version.sh" "$WORK" 2>/dev/null)"
  echo "$out" | grep -q '^verdict=up_to_date$' \
    || { fail t01 "rewritten stored token caused drift (still recalling?): $out"; all=0; }
  # restore manifest
  printf 'rules_sha: %s\n' "$tok_before" >/dev/null
  cp "$PAY/manifest.yaml" "$WORK/.cyberos/manifest.yaml"
  rm -f "$WORK/.cyberos/manifest.yaml.bak"
  [ "$all" -eq 1 ] && ok t01_installed_side_recomputed_not_recalled
}

# ── t02: one shared vendored declaration (cone + exclusions + _rsha) ─────────
t02_one_shared_vendored_declaration() {
  local all=1
  [ -f "$RC" ] || { fail t02 "missing $RC"; return; }
  grep -q '_rules_cone_entries' "$RC" && grep -q '^_rsha()' "$RC" \
    || { fail t02 "cone/_rsha missing from rules-cone.sh"; all=0; }
  grep -q 'exempt:manifest.yaml' "$RC" || { fail t02 "exclusions not in shared file"; all=0; }
  for f in build.sh version.sh lib/update-check.sh audit-fleet.sh; do
    grep -q 'rules-cone.sh' "$repo/tools/install/$f" \
      || { fail t02 "$f does not source rules-cone.sh"; all=0; }
  done
  if grep -vE '^\s*#' "$repo/tools/install/build.sh" | grep -E 'find cuo plugin mcp cli memory' >/dev/null; then
    fail t02 "build.sh still inlines old find cone"; all=0
  fi
  [ -f "$REF/.cyberos/lib/rules-cone.sh" ] || { fail t02 "not vendored to .cyberos/lib/"; all=0; }
  for f in version.sh lib/update-check.sh audit-fleet.sh; do
    grep -E '^_rsha\(\)' "$repo/tools/install/$f" >/dev/null \
      && { fail t02 "$f defines its own _rsha"; all=0; }
  done
  [ "$all" -eq 1 ] && ok t02_one_shared_vendored_declaration
}

# ── t03: four-kind grammar + invariants (NEW5-002) ───────────────────────────
# Capture list into a var before grep -q: under `set -o pipefail`, piping a writer
# into grep -q yields SIGPIPE (141) on a successful match and falsely fails the arm.
t03_element_grammar_four_kinds_and_invariants() {
  local all=1 d="$TMP/t03tree" list
  mkdir -p "$d/cuo/sub" "$d/memory"
  echo a > "$d/cuo/sub/f.txt"; echo b > "$d/cuo/only.txt"
  echo c > "$d/memory/AGENTS.md"; echo d > "$d/install.sh"
  list="$(_rules_cone_list "$d")"
  printf '%s\n' "$list" | grep -q 'cuo/sub/f.txt' || { fail t03 "dir: missed nested"; all=0; }
  printf '%s\n' "$list" | grep -q '^install.sh$'  || { fail t03 "file: miss"; all=0; }
  # prune:memory/store/ absent → defensive OK (validate passes)
  _rules_cone_validate_grammar "$d" || { fail t03 "validate failed on clean tree (defensive prune?)"; all=0; }
  # prune actively removes when present
  mkdir -p "$d/memory/store"; echo s > "$d/memory/store/x.md"
  list="$(_rules_cone_list "$d")"
  printf '%s\n' "$list" | grep -q 'memory/store/x.md' && { fail t03 "prune:memory/store/ did not remove"; all=0; }
  # exempt does not enter the hashed list
  echo e > "$d/gates.env"
  list="$(_rules_cone_list "$d")"
  printf '%s\n' "$list" | grep -q '^gates.env$' && { fail t03 "exempt hashed"; all=0; }
  # noise filter: .pyc / __pycache__ skipped
  mkdir -p "$d/cuo/__pycache__"; echo p > "$d/cuo/__pycache__/m.pyc"; echo q > "$d/cuo/x.pyc"
  list="$(_rules_cone_list "$d")"
  printf '%s\n' "$list" | grep -q '__pycache__' && { fail t03 "__pycache__ not skipped"; all=0; }
  printf '%s\n' "$list" | grep -q 'x.pyc' && { fail t03 ".pyc not skipped"; all=0; }
  [ "$all" -eq 1 ] && ok t03_element_grammar_four_kinds_and_invariants
}

# ── t04: cone == vendored set (set equality + mutation arms) ─────────────────
t04_cone_is_exactly_the_vendored_set() {
  local all=1 f
  grep -q 'dir:cli' "$RC" && { fail t04 "cli is in the cone"; all=0; }
  grep -q 'dir:lib' "$RC" && grep -q 'dir:docs-tools' "$RC" \
    || { fail t04 "lib/docs-tools missing from cone"; all=0; }
  for s in install.sh uninstall.sh version.sh status.sh help.sh check-latest.sh; do
    grep -q "file:$s" "$RC" || { fail t04 "missing file:$s"; all=0; }
  done
  local pay_list inst_list
  pay_list="$(_rules_cone_list "$PAY" | LC_ALL=C sort -u)"
  inst_list="$(_rules_cone_list "$REF/.cyberos" | LC_ALL=C sort -u)"
  [ "$pay_list" = "$inst_list" ] || { fail t04 "payload≠install cone sets"; all=0; }
  # mutation arms: one file under each cone dir + a root script, each via restore
  local out
  for f in cuo/ship-tasks.md lib/version-compare.sh docs-tools/md.mjs help.sh; do
    mutate "$f"
    out="$(bash "$PAY/version.sh" "$WORK" 2>/dev/null || true)"
    printf '%s\n' "$out" | grep -q rules_drift \
      || { fail t04 "mutation of $f not drift"; all=0; }
    restore "$f"
  done
  [ "$all" -eq 1 ] && ok t04_cone_is_exactly_the_vendored_set
}

# ── t05: nine exclusions named; .install.lock fixture-created (NEW5-003) ─────
t05_exclusions_named_and_pruned() {
  local all=1
  for e in 'prune:memory/store/' 'exempt:gates.env' 'exempt:config.yaml' \
           'exempt:.update-check-cache' 'exempt:AGENT-ENTRY.md' 'exempt:gates.env.bak.*' \
           'exempt:.install.lock' 'exempt:manifest.yaml' 'exempt:VERSION'; do
    grep -qF "$e" "$RC" || { fail t05 "missing $e"; all=0; }
  done
  local out
  # class (a): store add + gates.env change — no drift
  echo noise > "$WORK/.cyberos/memory/store/extra.md"
  echo '# changed' >> "$WORK/.cyberos/gates.env"
  out="$(bash "$PAY/version.sh" "$WORK" 2>/dev/null || true)"
  printf '%s\n' "$out" | grep -q '^verdict=up_to_date$' \
    || { fail t05 "store/gates.env change caused drift"; all=0; }
  rm -f "$WORK/.cyberos/memory/store/extra.md"; cp "$PAY/gates.env" "$WORK/.cyberos/gates.env" 2>/dev/null || :
  # config.yaml, AGENT-ENTRY.md, .install.lock (fixture-created) — no drift
  : > "$WORK/.cyberos/.install.lock"
  [ -f "$WORK/.cyberos/.install.lock" ] || { fail t05 "could not create .install.lock fixture"; all=0; }
  echo '# c' >> "$WORK/.cyberos/config.yaml" 2>/dev/null || echo '# c' > "$WORK/.cyberos/config.yaml"
  out="$(bash "$PAY/version.sh" "$WORK" 2>/dev/null || true)"
  printf '%s\n' "$out" | grep -q '^verdict=up_to_date$' \
    || { fail t05 "config/.install.lock caused drift"; all=0; }
  rm -f "$WORK/.cyberos/.install.lock"
  # class (b): manifest token rewrite — no drift
  sed -i.bak 's/^rules_sha:.*/rules_sha: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/' \
    "$WORK/.cyberos/manifest.yaml"
  out="$(bash "$PAY/version.sh" "$WORK" 2>/dev/null || true)"
  printf '%s\n' "$out" | grep -q '^verdict=up_to_date$' \
    || { fail t05 "manifest token rewrite caused drift"; all=0; }
  cp "$PAY/manifest.yaml" "$WORK/.cyberos/manifest.yaml"; rm -f "$WORK/.cyberos/manifest.yaml.bak"
  [ "$all" -eq 1 ] && ok t05_exclusions_named_and_pruned
}

# ── t06: reconciler fails both directions (structural conditions) ────────────
t06_reconciler_fails_both_directions() {
  local all=1
  [ -d "$PAY/cli" ] || { fail t06 "payload has no cli/ fixture"; return; }
  [ ! -d "$REF/.cyberos/cli" ] || { fail t06 "cli unexpectedly installed"; all=0; }
  # Direction 1: an unclassified path fails classify
  [ "$(_rules_cone_classify 'evil-unclassified.txt')" = unclassified ] \
    || { fail t06 "classify missed unclassified"; all=0; }
  # Direction 2: with dir:cli added, cli/* is in payload cone but absent from install.
  cp "$PAY/lib/rules-cone.sh" "$TMP/t06cone.sh"
  printf '\n_rules_cone_entries() { printf "%%s\\n" "$_RULES_CONE_ENTRIES"; printf "dir:cli\\n"; }\n' >> "$TMP/t06cone.sh"
  ( # subshell so we do not clobber parent's cone functions
    # shellcheck source=/dev/null
    . "$TMP/t06cone.sh"
    missing=0
    while IFS= read -r rel; do
      case "$rel" in cli/*) [ -f "$REF/.cyberos/$rel" ] || missing=1 ;; esac
    done < <(_rules_cone_list "$PAY")
    [ "$missing" -eq 1 ]
  ) || { fail t06 "dir:cli did not produce Direction-2 gap"; all=0; }
  # Reconciler is wired in build.sh and runs by default (skipped only via env).
  grep -q '_do_rules_reconcile' "$repo/tools/install/build.sh" || { fail t06 "no reconciler in build.sh"; all=0; }
  [ "$all" -eq 1 ] && ok t06_reconciler_fails_both_directions
}

# ── t07: reconciler runs install, does not parse it ──────────────────────────
t07_reconciler_runs_install_does_not_parse_it() {
  local all=1
  if grep -A90 'rules-cone reconciler' "$repo/tools/install/build.sh" | grep -qE 'grep[^|]*install\.sh|sed[^|]*install\.sh'; then
    fail t07 "reconciler appears to parse install.sh"; all=0
  fi
  grep -q 'bash "$out/install.sh"' "$repo/tools/install/build.sh" \
    || { fail t07 "reconciler does not exec install.sh"; all=0; }
  grep -q 'CYBEROS_GLOBAL_SKILLS=0' "$repo/tools/install/build.sh" \
    || { fail t07 "GLOBAL_SKILLS not pinned"; all=0; }
  grep -qE 'HOME=' "$repo/tools/install/build.sh" \
    || { fail t07 "HOME not redirected"; all=0; }
  if grep -A120 '_do_rules_reconcile=1' "$repo/tools/install/build.sh" | grep -qE 'AGENTS\.md|memory\.schema'; then
    fail t07 "reconciler hardcodes memory filenames"; all=0
  fi
  [ "$all" -eq 1 ] && ok t07_reconciler_runs_install_does_not_parse_it
}

# ── t08: equal tokens still drift ────────────────────────────────────────────
t08_equal_tokens_still_drift() {
  local tok out
  tok="$(grep '^rules_sha:' "$WORK/.cyberos/manifest.yaml" | awk '{print $2}')"
  mutate docs-tools/md.mjs
  grep -q "$tok" "$WORK/.cyberos/manifest.yaml" || { fail t08 "token changed unexpectedly"; restore docs-tools/md.mjs; return; }
  out="$(bash "$PAY/version.sh" "$WORK" 2>/dev/null || true)"
  if printf '%s\n' "$out" | grep -q rules_drift; then
    ok t08_equal_tokens_still_drift
  else
    fail t08 "equal tokens hid drift"
  fi
  restore docs-tools/md.mjs
}

# ── t09: naming follows invocation (sampled arms) ────────────────────────────
t09_naming_follows_invocation_not_component() {
  local all=1 out
  mutate cuo/ship-tasks.md; mutate lib/version-compare.sh; mutate help.sh
  # payload invocation names paths
  out="$(bash "$PAY/version.sh" "$WORK" 2>/dev/null)"
  echo "$out" | grep -q 'differing paths:' || { fail t09 "payload arm did not name paths"; all=0; }
  echo "$out" | grep -q 'cuo/ship-tasks.md' || { fail t09 "missing cuo path"; all=0; }
  echo "$out" | grep -q 'lib/version-compare.sh' || { fail t09 "missing lib path"; all=0; }
  echo "$out" | grep -q 'help.sh' || { fail t09 "missing help.sh path"; all=0; }
  # install-shaped: cannot name
  out="$(bash "$WORK/.cyberos/version.sh" "$WORK" 2>/dev/null)"
  echo "$out" | grep -q 'cannot name differing paths' \
    || { fail t09 "install arm named paths or missed disclaimer"; all=0; }
  restore cuo/ship-tasks.md; restore lib/version-compare.sh; restore help.sh
  # eighth arm: other-repo against THIS install's token. Mutate ref-copy's status.sh.
  mutate status.sh
  out="$(bash "$REF/.cyberos/version.sh" "$WORK" 2>/dev/null)"
  echo "$out" | grep -q 'rules_drift' || { fail t09 "eighth arm not drift: $out"; all=0; }
  echo "$out" | grep -q 'build token read from an install' \
    || { fail t09 "eighth arm missing install-token disclaimer"; all=0; }
  echo "$out" | grep -q 'differing paths:' && { fail t09 "eighth arm named paths"; all=0; }
  restore status.sh
  # update-check with CYBEROS_PAYLOAD names paths
  mutate cuo/ship-tasks.md
  out="$(cd "$WORK" && CYBEROS_PAYLOAD="$PAY" CYBEROS_UPDATE_CHECK=always bash "$PAY/lib/update-check.sh" 2>&1)"
  echo "$out" | grep -q 'differing paths:' || { fail t09 "update-check+PAYLOAD no names"; all=0; }
  rm -f "$WORK/.cyberos/.update-check-cache"; restore cuo/ship-tasks.md
  [ "$all" -eq 1 ] && ok t09_naming_follows_invocation_not_component
}

# ── t10: clean install is current (derived, no hardcoded digests) ────────────
t10_clean_install_is_current() {
  local all=1 out expect got
  expect="$(_rules_sha_of "$PAY")"
  got="$(_rules_sha_of "$REF/.cyberos")"
  [ "$got" = "$expect" ] || { fail t10 "fresh install digest mismatch pay=$expect inst=$got"; all=0; }
  out="$(bash "$PAY/version.sh" "$REF" 2>/dev/null)"
  echo "$out" | grep -q '^verdict=up_to_date$' || { fail t10 "version: $out"; all=0; }
  rm -f "$REF/.cyberos/.update-check-cache"
  out="$(cd "$REF" && CYBEROS_PAYLOAD="$PAY" CYBEROS_UPDATE_CHECK=always bash "$PAY/lib/update-check.sh" 2>&1)"
  [ -z "$out" ] || { fail t10 "update-check noisy: $out"; all=0; }
  rm -f "$REF/.cyberos/.update-check-cache"
  grep -q 'dir:cli' "$RC" && { fail t10 "old cone (cli) still present"; all=0; }
  [ "$all" -eq 1 ] && ok t10_clean_install_is_current
}

# ── t11: shared _rsha identity ───────────────────────────────────────────────
t11_shared_rsha_identity_and_cross_platform() {
  local all=1 a b
  grep -qE 'sha256sum|shasum' "$RC" || { fail t11 "no hasher in shared file"; all=0; }
  a="$(_rules_sha_of "$PAY")"; b="$(_rules_sha_of "$PAY")"
  [ "$a" = "$b" ] || { fail t11 "non-deterministic digest"; all=0; }
  grep -q 'LC_ALL=C sort' "$RC" || { fail t11 "LC_ALL=C sort missing"; all=0; }
  # cross-locale: same digest under C and a UTF-8 locale
  local c u
  c="$(LC_ALL=C _rules_sha_of "$PAY")"
  u="$(LC_ALL=en_US.UTF-8 _rules_sha_of "$PAY" 2>/dev/null)"
  [ -z "$u" ] || [ "$c" = "$u" ] || { fail t11 "locale changed digest"; all=0; }
  [ "$all" -eq 1 ] && ok t11_shared_rsha_identity_and_cross_platform
}

# ── t12: uncomputable → unknown ──────────────────────────────────────────────
t12_uncomputable_is_unknown_not_current() {
  local all=1 out
  # Remove cone lib from the WORK install; install-shaped invocation → unknown
  mv "$WORK/.cyberos/lib/rules-cone.sh" "$TMP/t12-cone.bak"
  out="$(bash "$WORK/.cyberos/version.sh" "$WORK" 2>/dev/null)"
  echo "$out" | grep -q '^verdict=unknown$' || { fail t12 "missing cone lib not unknown: $out"; all=0; }
  echo "$out" | grep -q 'up_to_date' && { fail t12 "emitted current"; all=0; }
  mv "$TMP/t12-cone.bak" "$WORK/.cyberos/lib/rules-cone.sh"
  [ "$all" -eq 1 ] && ok t12_uncomputable_is_unknown_not_current
}

# ── t13: exit contracts ──────────────────────────────────────────────────────
t13_exit_contracts() {
  local all=1
  mutate help.sh
  (cd "$WORK" && CYBEROS_PAYLOAD="$PAY" CYBEROS_UPDATE_CHECK=soft bash "$PAY/lib/update-check.sh" >/dev/null 2>&1) \
    || { fail t13 "soft non-zero on drift"; all=0; }
  rm -f "$WORK/.cyberos/.update-check-cache"
  (cd "$WORK" && CYBEROS_PAYLOAD="$PAY" CYBEROS_UPDATE_CHECK=strict bash "$PAY/lib/update-check.sh" >/dev/null 2>&1) \
    && { fail t13 "strict zero on drift"; all=0; }
  rm -f "$WORK/.cyberos/.update-check-cache"
  (cd "$WORK" && CYBEROS_PAYLOAD="$PAY" CYBEROS_UPDATE_CHECK=0 bash "$PAY/lib/update-check.sh" >/dev/null 2>&1) \
    || { fail t13 "off mode non-zero"; all=0; }
  restore help.sh
  # audit-fleet fail-open warning gone; unknown path present
  grep -q 'rule-drift check DISABLED' "$repo/tools/install/audit-fleet.sh" \
    && { fail t13 "fail-open warning still present"; all=0; }
  grep -q 'verdict=unknown' "$repo/tools/install/audit-fleet.sh" \
    || { fail t13 "unknown fail path missing"; all=0; }
  [ "$all" -eq 1 ] && ok t13_exit_contracts
}

# ── t14: no tamper claims ────────────────────────────────────────────────────
t14_no_tamper_claims() {
  local all=1 out
  mutate help.sh
  out="$(bash "$PAY/version.sh" "$WORK" 2>/dev/null)"
  echo "$out" | grep -qiE 'tamper|integrity|authenticity' && { fail t14 "version tamper wording"; all=0; }
  out="$(cd "$WORK" && CYBEROS_PAYLOAD="$PAY" CYBEROS_UPDATE_CHECK=always bash "$PAY/lib/update-check.sh" 2>&1)"
  echo "$out" | grep -qiE 'tamper|integrity|authenticity' && { fail t14 "update-check tamper wording"; all=0; }
  rm -f "$WORK/.cyberos/.update-check-cache"; restore help.sh
  [ "$all" -eq 1 ] && ok t14_no_tamper_claims
}

# ── t15: read-only except .update-check-cache (NEW5-004) ─────────────────────
t15_check_is_read_only() {
  local all=1
  ( cd "$REF/.cyberos" && find . -type f ! -name '.update-check-cache' | LC_ALL=C sort | while read -r f; do
      cksum "$f"; done ) > "$TMP/t15.before"
  bash "$PAY/version.sh" "$REF" >/dev/null 2>&1 || true
  ( cd "$REF/.cyberos" && find . -type f ! -name '.update-check-cache' | LC_ALL=C sort | while read -r f; do
      cksum "$f"; done ) > "$TMP/t15.after"
  cmp -s "$TMP/t15.before" "$TMP/t15.after" || { fail t15 "version.sh changed non-cache paths"; all=0; }
  rm -f "$REF/.cyberos/.update-check-cache"
  # audit-fleet must not write .update-check-cache into audited repo
  mkdir -p "$TMP/t15fleet"; cp -R "$REF" "$TMP/t15fleet/x"
  rm -f "$TMP/t15fleet/x/.cyberos/.update-check-cache"
  CYBEROS_EXPECT_RULES_SHA="$(_rules_sha_of "$PAY")" \
    bash "$repo/tools/install/audit-fleet.sh" "$pv" "$TMP/t15fleet" >/dev/null 2>&1 || true
  [ ! -f "$TMP/t15fleet/x/.cyberos/.update-check-cache" ] \
    || { fail t15 "audit-fleet wrote .update-check-cache"; all=0; }
  rm -rf "$TMP/t15fleet"
  [ "$all" -eq 1 ] && ok t15_check_is_read_only
}

echo "test_rules_sha_recompute.sh (TASK-IMP-122)"
t01_installed_side_recomputed_not_recalled
t02_one_shared_vendored_declaration
t03_element_grammar_four_kinds_and_invariants
t04_cone_is_exactly_the_vendored_set
t05_exclusions_named_and_pruned
t06_reconciler_fails_both_directions
t07_reconciler_runs_install_does_not_parse_it
t08_equal_tokens_still_drift
t09_naming_follows_invocation_not_component
t10_clean_install_is_current
t11_shared_rsha_identity_and_cross_platform
t12_uncomputable_is_unknown_not_current
t13_exit_contracts
t14_no_tamper_claims
t15_check_is_read_only
echo "----"; echo "pass=$PASS fail=$FAIL"
[ "$FAIL" -eq 0 ]
