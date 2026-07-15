#!/usr/bin/env bash
# test_init_hygiene.sh - install.sh hygiene contract:
#   t01  .gitignore = ONE managed block, regenerated in place; legacy scattered entries lifted;
#        operator lines outside the markers survive re-init byte-for-byte.
#   t02  CHANGELOG.md is created exactly once (never clobbered on re-init).
#   t03  auto-migration: root-level flat tasks relocate to <module>/<STEM>/spec.md (module from
#        frontmatter), module-level flat tasks migrate, references rewrite, verify line is clean.
#   t04  payload self-cleanup: <repo>/.cyberos-init removes itself on success; kept with
#        CYBEROS_KEEP_PAYLOAD=1; non-canonical in-repo copies are never removed.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

echo "building scratch payload..."
bash "$repo/tools/cyberos-init/build.sh" "$TMP/payload" >/dev/null 2>&1 || { echo FATAL build; exit 1; }

mkrepo() { mkdir -p "$1"; (cd "$1" && git init -q . 2>/dev/null || true); }

t01_gitignore_managed_block() {
  local d="$TMP/gi"; mkrepo "$d"
  # operator rules + the exact legacy lines a pre-block init used to append
  cat > "$d/.gitignore" <<'GI'
node_modules/
*.log

# CyberOS vendored machine + local BRAIN at .cyberos/memory/store (regenerable via init; tenant data). Do not commit.
.cyberos/

# CyberOS skill symlinks -> .cyberos/plugin/skills (regenerable via init).
.claude/skills/ship-tasks
.grok/skills/ship-tasks
.commandcode/skills/ship-tasks
.codex/skills/ship-tasks
.opencode/skill/ship-tasks
GI
  bash "$TMP/payload/install.sh" "$d" >/dev/null 2>&1
  bash "$TMP/payload/install.sh" "$d" >/dev/null 2>&1     # second run must not duplicate anything
  local blocks legacy cyb ops
  blocks="$(grep -c '>>> cyberos' "$d/.gitignore")"
  legacy="$(grep -c 'CyberOS vendored machine' "$d/.gitignore" || true)"
  cyb="$(grep -cx '\.cyberos/' "$d/.gitignore")"
  ops=1; { grep -qx 'node_modules/' "$d/.gitignore" && grep -qx '\*.log' "$d/.gitignore"; } || ops=0
  [ "$blocks" = 1 ] && [ "$legacy" = 0 ] && [ "$cyb" = 1 ] && [ "$ops" = 1 ] \
    && ok t01 || fail t01 "blocks=$blocks legacy=$legacy .cyberos-lines=$cyb operator-lines=$ops"
}

t02_changelog_once() {
  local d="$TMP/cl"; mkrepo "$d"
  bash "$TMP/payload/install.sh" "$d" >/dev/null 2>&1
  [ -f "$d/CHANGELOG.md" ] || { fail t02 "no CHANGELOG.md created"; return; }
  echo "OPERATOR-EDIT" >> "$d/CHANGELOG.md"
  bash "$TMP/payload/install.sh" "$d" >/dev/null 2>&1
  grep -q "OPERATOR-EDIT" "$d/CHANGELOG.md" && [ "$(grep -c '^# Changelog' "$d/CHANGELOG.md")" = 1 ] \
    && ok t02 || fail t02 "clobbered or duplicated on re-init"
}

t03_root_fr_migration() {
  local d="$TMP/mig"; mkrepo "$d"
  mkdir -p "$d/docs/tasks/api"
  printf -- '---\nid: TASK-CORE-001\ntitle: Root task\nmodule: widget-core\nstatus: draft\n---\nbody\n' \
    > "$d/docs/tasks/TASK-CORE-001-root-thing.md"
  printf -- '---\nid: TASK-API-001\ntitle: Mod task\nstatus: draft\n---\nbody\n' \
    > "$d/docs/tasks/api/TASK-API-001-mod-thing.md"
  printf -- '# idx\nsee docs/tasks/TASK-CORE-001-root-thing.md and TASK-CORE-001-root-thing.md\n' \
    > "$d/docs/tasks/INDEX.md"
  out="$(bash "$TMP/payload/install.sh" "$d" 2>&1)"
  local all=1
  [ -f "$d/docs/tasks/widget-core/TASK-CORE-001-root-thing/spec.md" ] || { fail t03-root "not relocated by frontmatter module"; all=0; }
  [ -f "$d/docs/tasks/api/TASK-API-001-mod-thing/spec.md" ]           || { fail t03-mod "module-level flat not migrated"; all=0; }
  grep -q "docs/tasks/widget-core/TASK-CORE-001-root-thing/spec.md" "$d/docs/tasks/INDEX.md" \
    && ! grep -q "TASK-CORE-001-root-thing\.md" "$d/docs/tasks/INDEX.md" || { fail t03-refs "references not rewritten"; all=0; }
  grep -q "flat_fr_files_remaining=0 fr_folders_missing_spec=0" <<<"$out" || { fail t03-verify "verify line not clean"; all=0; }
  if command -v node >/dev/null 2>&1; then
    [ -f "$d/docs/status/index.html" ] && [ -f "$d/docs/status/assets/status.css" ] && [ -f "$d/docs/status/assets/favicon.svg" ] \
      || { fail t03-page "docs/status/ folder incomplete"; all=0; }
    [ ! -f "$d/.cyberos/status.html" ] && [ ! -f "$d/docs/status.html" ] || { fail t03-oldpage "stale pre-folder page not cleaned"; all=0; }
    grep -q "$(basename "$d") status" "$d/docs/status/index.html" || { fail t03-title "page not titled after the repo"; all=0; }
    ! grep -q "CyberOS status" "$d/docs/status/index.html" || { fail t03-brand "page still titled 'CyberOS status'"; all=0; }
    [ -x "$d/.git/hooks/pre-commit" ] && grep -q cyberos-status-hook "$d/.git/hooks/pre-commit" \
      || { fail t03-hook "auto-sync pre-commit hook missing"; all=0; }
  fi
  # idempotent: second run moves nothing
  out2="$(bash "$TMP/payload/install.sh" "$d" 2>&1)"
  grep -q "nothing to do (already folder-per-task)" <<<"$out2" || { fail t03-idem "second run not a no-op"; all=0; }
  [ "$all" -eq 1 ] && ok t03
}

t04_payload_self_cleanup() {
  local all=1
  local d="$TMP/pc"; mkrepo "$d"
  cp -R "$TMP/payload" "$d/.cyberos-init"
  out="$(bash "$d/.cyberos-init/install.sh" "$d" 2>&1)"
  { [ ! -e "$d/.cyberos-init" ] && grep -q "payload: removed .cyberos-init/" <<<"$out"; } || { fail t04-rm "canonical copy not removed"; all=0; }
  local k="$TMP/pk"; mkrepo "$k"
  cp -R "$TMP/payload" "$k/.cyberos-init"
  out="$(CYBEROS_KEEP_PAYLOAD=1 bash "$k/.cyberos-init/install.sh" "$k" 2>&1)"
  { [ -d "$k/.cyberos-init" ] && grep -q "payload: kept .cyberos-init/" <<<"$out"; } || { fail t04-keep "KEEP_PAYLOAD not honored"; all=0; }
  local n="$TMP/nc"; mkrepo "$n"
  cp -R "$TMP/payload" "$n/vendor-payload"
  out="$(bash "$n/vendor-payload/install.sh" "$n" 2>&1)"
  { [ -d "$n/vendor-payload" ] && grep -q "payload: kept vendor-payload/" <<<"$out"; } || { fail t04-noncanon "non-canonical copy was touched"; all=0; }
  [ "$all" -eq 1 ] && ok t04
}

t05_supersede_old_docs() {
  local all=1 d="$TMP/sup"; mkrepo "$d"
  mkdir -p "$d/docs"
  printf '# my roadmap\nQ3 plans here\n'            > "$d/docs/ROADMAP.md"
  printf '# my backlog\n- TASK-A-001 pending row\n'   > "$d/docs/BACKLOG.md"
  printf '# changes\n\n## [0.2.0] - 2026-01-01\n\n- old release\n' > "$d/docs/CHANGELOG.md"
  bash "$TMP/payload/install.sh" "$d" >/dev/null 2>&1
  # adoption: content preserved in the canonical homes
  grep -q "TASK-A-001 pending row" "$d/docs/tasks/BACKLOG.md" || { fail t05-adopt-bl "docs/BACKLOG.md not adopted"; all=0; }
  grep -q "0.2.0" "$d/CHANGELOG.md" || { fail t05-adopt-cl "docs/CHANGELOG.md not adopted to root"; all=0; }
  if command -v node >/dev/null 2>&1; then
    # the old standalone docs are REMOVED outright - the page replaces them
    [ ! -f "$d/docs/ROADMAP.md" ] && [ ! -f "$d/docs/BACKLOG.md" ] && [ ! -f "$d/docs/CHANGELOG.md" ] \
      || { fail t05-removed "old docs/ROADMAP|BACKLOG|CHANGELOG still present"; all=0; }
    grep -q "v0.2.0" "$d/docs/status/index.html" || { fail t05-page "adopted changelog not on the page"; all=0; }
    # idempotent: re-init keeps them removed and keeps the adopted content
    bash "$TMP/payload/install.sh" "$d" >/dev/null 2>&1
    [ ! -f "$d/docs/ROADMAP.md" ] || { fail t05-idem "removed doc resurrected on re-init"; all=0; }
    grep -q "TASK-A-001 pending row" "$d/docs/tasks/BACKLOG.md" || { fail t05-idem-bl "adopted backlog lost on re-init"; all=0; }
  fi
  [ "$all" -eq 1 ] && ok t05
}

t06_hook_append_foreign() {
  local all=1 d="$TMP/hk"; mkrepo "$d"
  mkdir -p "$d/.git/hooks"
  printf '#!/bin/sh\necho foreign-lint\nfalse\n' > "$d/.git/hooks/pre-commit"; chmod +x "$d/.git/hooks/pre-commit"
  bash "$TMP/payload/install.sh" "$d" >/dev/null 2>&1
  grep -q "echo foreign-lint" "$d/.git/hooks/pre-commit" || { fail t06-keep "foreign hook body lost"; all=0; }
  [ "$(grep -c '>>> cyberos-status-hook' "$d/.git/hooks/pre-commit")" = 1 ] || { fail t06-append "marked block not appended exactly once"; all=0; }
  bash "$TMP/payload/install.sh" "$d" >/dev/null 2>&1
  [ "$(grep -c '>>> cyberos-status-hook' "$d/.git/hooks/pre-commit")" = 1 ] || { fail t06-idem "block duplicated on re-init"; all=0; }
  # the appended block runs AFTER the foreign body and must preserve its failing exit code (1)
  ( cd "$d" && sh .git/hooks/pre-commit >/dev/null 2>&1 ); [ "$?" = 1 ] || { fail t06-exit "foreign exit code not preserved"; all=0; }
  [ "$all" -eq 1 ] && ok t06
}

t01_gitignore_managed_block
t02_changelog_once
t03_root_fr_migration
t04_payload_self_cleanup
t05_supersede_old_docs
t06_hook_append_foreign

echo "init-hygiene: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]
