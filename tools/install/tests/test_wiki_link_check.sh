#!/usr/bin/env bash
# test_wiki_link_check.sh — TASK-IMP-059 suite.
#
#   t01 -> tool exists and --help exits 0
#   t02 -> fixture with a good relative link passes
#   t03 -> fixture with a broken relative link fails (exit 1)
#   t04 -> missing depends_on TASK id fails; allowlisted id passes
#   t05 -> live repo scan (docs/) exits 0 with the committed allowlist
#   t06 -> build.sh vendors wiki-link-check.mjs into the payload docs-tools/
#
# run_all discovers this via tools/install/tests/test_*.sh.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
MJS="$repo/tools/install/docs-tools/wiki-link-check.mjs"
ALLOW="$repo/tools/install/docs-tools/wiki-link-allowlist.txt"
BUILD="$repo/tools/install/build.sh"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

command -v node >/dev/null 2>&1 || { echo "  SKIP test_wiki_link_check.sh — node not on PATH"; exit 0; }

t01_help() {
  [ -f "$MJS" ] || { fail t01_help "missing $MJS"; return; }
  node "$MJS" --help >/dev/null 2>&1 || { fail t01_help "--help exited non-zero"; return; }
  ok t01_help
}

scratch_repo() {
  local d="$1"
  mkdir -p "$d/docs/tasks/improvement/TASK-IMP-999-fixture" \
           "$d/docs/tasks/improvement/TASK-IMP-998-peer" \
           "$d/docs/notes"
  printf '%s\n' '# peer' > "$d/docs/tasks/improvement/TASK-IMP-998-peer/spec.md"
  printf '%s\n' '# backlog' > "$d/docs/tasks/BACKLOG.md"
  ( cd "$d" && git init -q . && git config user.email t@t && git config user.name t ) >/dev/null 2>&1
}

t02_good_link_passes() {
  local d="$TMP/t02"; scratch_repo "$d"
  cat > "$d/docs/notes/a.md" <<'EOF'
# a
See [peer](../tasks/improvement/TASK-IMP-998-peer/spec.md).
EOF
  node "$MJS" --root "$d" --docs docs --allowlist /dev/null >/dev/null 2>&1
  local rc=$?
  [ "$rc" -eq 0 ] || { fail t02_good_link_passes "expected exit 0, got $rc"; return; }
  ok t02_good_link_passes
}

t03_broken_link_fails() {
  local d="$TMP/t03"; scratch_repo "$d"
  cat > "$d/docs/notes/a.md" <<'EOF'
# a
See [missing](./nope.md).
EOF
  node "$MJS" --root "$d" --docs docs --allowlist /dev/null >/dev/null 2>&1
  local rc=$?
  [ "$rc" -eq 1 ] || { fail t03_broken_link_fails "expected exit 1, got $rc"; return; }
  ok t03_broken_link_fails
}

t04_missing_task_and_allowlist() {
  local d="$TMP/t04"; scratch_repo "$d"
  cat > "$d/docs/tasks/improvement/TASK-IMP-999-fixture/spec.md" <<'EOF'
---
id: TASK-IMP-999
depends_on: [TASK-IMP-000]
---
# fixture
EOF
  node "$MJS" --root "$d" --docs docs --allowlist /dev/null >/dev/null 2>&1
  local rc=$?
  [ "$rc" -eq 1 ] || { fail t04_missing_task_and_allowlist "expected fail on missing TASK-IMP-000, got $rc"; return; }

  printf '%s\n' 'TASK-IMP-000' > "$d/allow.txt"
  node "$MJS" --root "$d" --docs docs --allowlist "$d/allow.txt" >/dev/null 2>&1
  rc=$?
  [ "$rc" -eq 0 ] || { fail t04_missing_task_and_allowlist "allowlisted id should pass, got $rc"; return; }
  ok t04_missing_task_and_allowlist
}

t05_live_repo_clean() {
  [ -f "$ALLOW" ] || { fail t05_live_repo_clean "missing allowlist $ALLOW"; return; }
  local out rc
  out="$(node "$MJS" --root "$repo" --docs docs --allowlist "$ALLOW" 2>&1)"
  rc=$?
  [ "$rc" -eq 0 ] || { fail t05_live_repo_clean "live scan exit $rc: $out"; return; }
  echo "$out" | grep -q 'ok:' || { fail t05_live_repo_clean "missing ok line: $out"; return; }
  ok t05_live_repo_clean
}

t06_vendored_in_build() {
  grep -q 'wiki-link-check.mjs' "$BUILD" \
    || { fail t06_vendored_in_build "build.sh does not vendor wiki-link-check.mjs"; return; }
  ok t06_vendored_in_build
}

echo "test_wiki_link_check.sh"
t01_help
t02_good_link_passes
t03_broken_link_fails
t04_missing_task_and_allowlist
t05_live_repo_clean
t06_vendored_in_build
echo "  result pass=$PASS fail=$FAIL"
[ "$FAIL" -eq 0 ]
