#!/usr/bin/env bash
# test_render_stamp.sh - TASK-IMP-082 §2 suite (t01-t06 -> AC 1-6).
# The status page's provenance stamp is a corpus fingerprint now: 'fp-' + first 12 hex of
# sha256 over the ordered render inputs (every task spec's raw bytes in bytewise repo-relative
# path order, then CHANGELOG.md, then VERSION). These asserts pin the properties that ended
# the HEAD chase: byte-stable re-renders, byte-stable render -> commit page -> render, stamp
# moves exactly once per corpus edit, CYBEROS_COMMIT pin still wins, and no git anywhere on
# the default path. Invocation matches the production wrapper (tools/install/lib/
# task-migrate.sh:61 via tools/install/lib/status-page.sh): node render-status-hub.mjs
# <root> <out>, templates resolved from the fixture's modules/templates.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../.." && pwd)"
R="$repo/tools/docs-site/render-status-hub.mjs"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

# first 12 hex of sha256 of stdin - GNU (sha256sum) or macOS (shasum) spelling
sha12() { if command -v sha256sum >/dev/null 2>&1; then sha256sum; else shasum -a 256; fi | cut -c1-12; }
# the stamp as the header meta carries it: built from <span class="code">STAMP</span>
stamp_of() { grep -o 'built from <span class="code">[^<]*</span>' "$1" | sed 's/.*">//;s/<.*//'; }

mkfix() {  # scratch corpus in the exact shape the renderer discovers: docs/tasks/<module>/<STEM>/spec.md
  d="$1"
  mkdir -p "$d/docs/tasks/aa/TASK-AA-001-first" "$d/docs/tasks/bb/TASK-BB-001-second" \
           "$d/modules/templates/html" "$d/modules/templates/cds"
  cp "$repo/modules/templates/html/status-hub.html" "$repo/modules/templates/html/status-app.js" "$d/modules/templates/html/"
  cp "$repo/modules/templates/cds/tokens.css" "$repo/modules/templates/cds/status.css" "$d/modules/templates/cds/"
  printf -- '---\nid: TASK-AA-001\ntitle: First\ntemplate: task@1\nmodule: aa\npriority: MUST\nstatus: done\ntype: product\nshipped: 2026-07-01\n---\n## §1 — Description\n\nFirst task body paragraph.\n' > "$d/docs/tasks/aa/TASK-AA-001-first/spec.md"
  printf -- '---\nid: TASK-BB-001\ntitle: Second\ntemplate: task@1\nmodule: bb\npriority: SHOULD\nstatus: draft\ntype: improvement\n---\n## §1 — Description\n\nSecond task body paragraph.\n' > "$d/docs/tasks/bb/TASK-BB-001-second/spec.md"
  printf '# CL\n\n## [2.0.0] - 2026-07-01\n\nAdded\n- TASK-AA-001 first thing landed\n' > "$d/CHANGELOG.md"
  echo "2.0.0" > "$d/VERSION"
}

t01_fingerprint_on_all_surfaces() {                                    # AC 1 - §1.1 + §1.7
  mkfix "$TMP/a"
  node "$R" "$TMP/a" "$TMP/a/out" >/dev/null 2>&1 || { fail t01 "render failed"; return; }
  h="$TMP/a/out/reference/status.html"
  s="$(stamp_of "$h")"
  grep -Eq '^fp-[0-9a-f]{12}$' <<<"$s" || { fail t01 "stamp shape wrong: '$s'"; return; }
  # the exact value, recomputed independently: sha256 over the spec files in bytewise
  # repo-relative path order, then CHANGELOG.md, then VERSION (concatenation == the
  # renderer's per-file updates)
  want="fp-$(cat "$TMP/a/docs/tasks/aa/TASK-AA-001-first/spec.md" \
             "$TMP/a/docs/tasks/bb/TASK-BB-001-second/spec.md" \
             "$TMP/a/CHANGELOG.md" "$TMP/a/VERSION" | sha12)"
  [ "$s" = "$want" ] || { fail t01 "stamp $s != independently computed $want"; return; }
  grep -qF "($s)" "$h" && grep -qF "\"commit\":\"$s\"" "$h" \
    && ok t01 || fail t01 "footer or cs-data commit field does not carry $s"
}
t02_double_render_stable() {                                           # AC 2 - §1.3
  # populated corpus: three renders (default locale from t01, then C, then a UTF-8 locale)
  # must agree byte-for-byte across the whole output tree - the path sort never consults
  # the environment
  LC_ALL=C       node "$R" "$TMP/a" "$TMP/a/o1" >/dev/null 2>&1 || { fail t02 "render (LC_ALL=C) failed"; return; }
  LC_ALL=C.UTF-8 node "$R" "$TMP/a" "$TMP/a/o2" >/dev/null 2>&1 || { fail t02 "render (LC_ALL=C.UTF-8) failed"; return; }
  diff -r "$TMP/a/o1" "$TMP/a/o2" >/dev/null 2>&1 || { fail t02 "populated corpus diverged across locales"; return; }
  diff -r "$TMP/a/out" "$TMP/a/o1" >/dev/null 2>&1 || { fail t02 "re-render diverged from first render"; return; }
  # empty corpus (0 tasks, no CHANGELOG - the lenient install shape): fingerprint over
  # VERSION alone, still deterministic
  mkdir -p "$TMP/e/docs/tasks" "$TMP/e/modules/templates/html" "$TMP/e/modules/templates/cds"
  cp "$repo/modules/templates/html/status-hub.html" "$repo/modules/templates/html/status-app.js" "$TMP/e/modules/templates/html/"
  cp "$repo/modules/templates/cds/tokens.css" "$repo/modules/templates/cds/status.css" "$TMP/e/modules/templates/cds/"
  echo "0.0.1" > "$TMP/e/VERSION"
  CYBEROS_HUB_LENIENT=1 node "$R" "$TMP/e" "$TMP/e/o1" >/dev/null 2>&1 || { fail t02 "empty-corpus render failed"; return; }
  CYBEROS_HUB_LENIENT=1 node "$R" "$TMP/e" "$TMP/e/o2" >/dev/null 2>&1 || { fail t02 "empty-corpus render 2 failed"; return; }
  diff -r "$TMP/e/o1" "$TMP/e/o2" >/dev/null 2>&1 \
    && grep -Eq 'built from <span class="code">fp-[0-9a-f]{12}</span>' "$TMP/e/o1/reference/status.html" \
    || { fail t02 "empty corpus diverged or stamp not fp-"; return; }
  # the fully empty input set (no tasks, no CHANGELOG, no VERSION): the stamp MUST be
  # sha256 of zero bytes - the constant proves nothing hidden (paths, clock) leaks in
  rm "$TMP/e/VERSION"
  CYBEROS_HUB_LENIENT=1 node "$R" "$TMP/e" "$TMP/e/o3" >/dev/null 2>&1 || { fail t02 "no-input render failed"; return; }
  [ "$(stamp_of "$TMP/e/o3/reference/status.html")" = "fp-e3b0c44298fc" ] \
    && ok t02 || fail t02 "empty input set != sha256('') - a hidden input leaked into the hash"
}
t03_commit_chase_ended() {                                             # AC 3 - §1.4
  command -v git >/dev/null 2>&1 || { fail t03 "git missing (the chase case needs a scratch repo)"; return; }
  G() { git -C "$TMP/g" -c user.email=t@t -c user.name=t -c commit.gpgsign=false "$@"; }
  mkfix "$TMP/g"
  git -C "$TMP/g" init -q >/dev/null 2>&1
  G add -A >/dev/null 2>&1 && G commit -qm corpus >/dev/null 2>&1 || { fail t03 "scratch repo setup failed"; return; }
  node "$R" "$TMP/g" "$TMP/g/docs/status" >/dev/null 2>&1 || { fail t03 "render failed"; return; }
  G add -A >/dev/null 2>&1 && G commit -qm page >/dev/null 2>&1 || { fail t03 "page commit failed"; return; }
  node "$R" "$TMP/g" "$TMP/g/out2" >/dev/null 2>&1 || { fail t03 "re-render failed"; return; }
  cmp -s "$TMP/g/docs/status/reference/status.html" "$TMP/g/out2/reference/status.html" \
    && ok t03 || fail t03 "render -> commit page -> render differed (the chase is back)"
}
t04_corpus_edit_changes_once() {                                       # AC 4 - §1.5
  s1="$(stamp_of "$TMP/a/out/reference/status.html")"
  printf '\nOne more paragraph, so the input bytes move.\n' >> "$TMP/a/docs/tasks/bb/TASK-BB-001-second/spec.md"
  node "$R" "$TMP/a" "$TMP/a/o3" >/dev/null 2>&1 || { fail t04 "render after edit failed"; return; }
  s2="$(stamp_of "$TMP/a/o3/reference/status.html")"
  [ -n "$s2" ] && [ "$s1" != "$s2" ] || { fail t04 "corpus edit did not move the stamp ($s1 -> $s2)"; return; }
  node "$R" "$TMP/a" "$TMP/a/o4" >/dev/null 2>&1 || { fail t04 "second render after edit failed"; return; }
  diff -r "$TMP/a/o3" "$TMP/a/o4" >/dev/null 2>&1 \
    && ok t04 || fail t04 "stamp not stable again after the edit"
}
t05_env_pin_wins() {                                                   # AC 5 - §1.2
  CYBEROS_COMMIT=abc123 node "$R" "$TMP/a" "$TMP/a/o5" >/dev/null 2>&1 || { fail t05 "pinned render failed"; return; }
  h="$TMP/a/o5/reference/status.html"
  grep -q 'built from <span class="code">abc123</span>' "$h" \
    && grep -qF '"commit":"abc123"' "$h" && grep -qF '(abc123)' "$h" \
    || { fail t05 "CYBEROS_COMMIT=abc123 did not win on all three surfaces"; return; }
  # empty pin falls through to the fingerprint (the || default, preserved)
  CYBEROS_COMMIT= node "$R" "$TMP/a" "$TMP/a/o6" >/dev/null 2>&1 || { fail t05 "empty-pin render failed"; return; }
  grep -Eq 'built from <span class="code">fp-[0-9a-f]{12}</span>' "$TMP/a/o6/reference/status.html" \
    && ok t05 || fail t05 "empty CYBEROS_COMMIT did not fall through to fp-"
}
t06_no_git_needed() {                                                  # AC 6 - §1.6
  # Mechanism, two halves: (a) the fixture carries NO .git, so any stamp derivation that
  # read repo state would surface as 'unknown' (the old fallback) or a crash - we assert a
  # real fp- stamp instead; (b) a tripwire `git` sits FIRST on PATH for the render and logs
  # every invocation to $TMP/git.calls before exiting 99 - the renderer is node stdlib file
  # reads only, so the log must stay absent (a regression that shells out to git trips it).
  # Then the same corpus is rendered inside a real git checkout: repo position is not an
  # input, so the two pages must be byte-identical.
  mkfix "$TMP/n"                                     # no .git anywhere under this root
  mkdir -p "$TMP/bin"
  printf '#!/bin/sh\necho "git $*" >> "%s/git.calls"\nexit 99\n' "$TMP" > "$TMP/bin/git"
  chmod +x "$TMP/bin/git"
  PATH="$TMP/bin:$PATH" CYBEROS_PROJECT=scratch node "$R" "$TMP/n" "$TMP/n/out" >/dev/null 2>&1 \
    || { fail t06 "render failed in a non-git dir"; return; }
  s="$(stamp_of "$TMP/n/out/reference/status.html")"
  grep -Eq '^fp-[0-9a-f]{12}$' <<<"$s" || { fail t06 "non-git stamp is '$s', not an fp- fingerprint"; return; }
  [ ! -e "$TMP/git.calls" ] || { fail t06 "renderer spawned git: $(cat "$TMP/git.calls")"; return; }
  if command -v git >/dev/null 2>&1; then
    mkfix "$TMP/n2"                                  # byte-identical corpus, this time a checkout
    git -C "$TMP/n2" init -q >/dev/null 2>&1
    git -C "$TMP/n2" -c user.email=t@t -c user.name=t -c commit.gpgsign=false add -A >/dev/null 2>&1
    git -C "$TMP/n2" -c user.email=t@t -c user.name=t -c commit.gpgsign=false commit -qm x >/dev/null 2>&1
    CYBEROS_PROJECT=scratch node "$R" "$TMP/n2" "$TMP/n2/out" >/dev/null 2>&1 || { fail t06 "render failed in git copy"; return; }
    cmp -s "$TMP/n/out/reference/status.html" "$TMP/n2/out/reference/status.html" \
      || { fail t06 "git presence changed the page bytes"; return; }
  fi
  ok t06
}

t01_fingerprint_on_all_surfaces; t02_double_render_stable; t03_commit_chase_ended
t04_corpus_edit_changes_once; t05_env_pin_wins; t06_no_git_needed
echo "----"; echo "pass=$PASS fail=$FAIL"; [ "$FAIL" -eq 0 ]
