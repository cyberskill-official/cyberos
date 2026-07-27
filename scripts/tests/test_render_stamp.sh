#!/usr/bin/env bash
# test_render_stamp.sh - TASK-IMP-082 §2 suite (t01-t06 -> AC 1-6), widened for TASK-DOCS-010.
# The status page's provenance stamp is a corpus fingerprint: 'fp-' + first 12 hex of
# sha256 over the ordered render inputs:
#   1. every task spec's raw bytes (bytewise repo-relative path order)
#   2. batch ledgers under docs/batches/ (same path order), when present
#   3. CHANGELOG.md, VERSION
#   4. docs/tasks/_state/commit-links.yaml (ledger), when present
#   5. modules/manifest.yaml, when present
#   6. status shell/client/CSS template bodies, hashed by basename order
#       (status-app.js, status-hub.html, status.css, tokens.css)
# Live git commit-set is intentionally NOT in the page stamp (would restore the IMP-082
# HEAD chase when the generated page is committed). status-feed@1 carries its own fp over
# feed bytes, including git-derived coverage. These asserts pin: byte-stable re-renders,
# byte-stable render -> commit page -> render, stamp moves exactly once per corpus edit,
# CYBEROS_COMMIT pin still wins. Git may be spawned for feed coverage; the page stamp path
# itself does not require git.
# Invocation matches production (tools/install/lib/task-migrate.sh via status-page.sh):
# node render-status-hub.mjs <root> <out>, templates from the fixture's modules/templates.
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

# Independent recompute matching corpusFingerprint(): specs (path order) + CL + VERSION
# + optional ledger/manifest + template bodies by basename order.
stamp_recompute() {
  local root="$1"
  local tpl="$root/modules/templates"
  {
    if [ -d "$root/docs/tasks" ]; then
      find "$root/docs/tasks" -name spec.md -print 2>/dev/null | LC_ALL=C sort | while read -r f; do
        cat "$f"
      done
    fi
    [ -f "$root/CHANGELOG.md" ] && cat "$root/CHANGELOG.md"
    [ -f "$root/VERSION" ] && cat "$root/VERSION"
    [ -f "$root/docs/tasks/_state/commit-links.yaml" ] && cat "$root/docs/tasks/_state/commit-links.yaml"
    [ -f "$root/modules/manifest.yaml" ] && cat "$root/modules/manifest.yaml"
    # basename order: status-app.js, status-hub.html, status.css, tokens.css
    [ -f "$tpl/html/status-app.js" ] && cat "$tpl/html/status-app.js"
    [ -f "$tpl/html/status-hub.html" ] && cat "$tpl/html/status-hub.html"
    [ -f "$tpl/cds/status.css" ] && cat "$tpl/cds/status.css"
    [ -f "$tpl/cds/tokens.css" ] && cat "$tpl/cds/tokens.css"
  } | sha12
}

mkfix() {  # scratch corpus in the exact shape the renderer discovers: docs/tasks/<module>/<STEM>/spec.md
  d="$1"
  mkdir -p "$d/docs/tasks/aa/TASK-AA-001-first" "$d/docs/tasks/bb/TASK-BB-001-second" \
           "$d/modules/templates/html" "$d/modules/templates/cds"
  cp "$repo/modules/templates/html/status-hub.html" "$repo/modules/templates/html/status-app.js" \
     "$d/modules/templates/html/"
  cp "$repo/modules/templates/cds/tokens.css" "$repo/modules/templates/cds/status.css" \
     "$d/modules/templates/cds/"
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
  want="fp-$(stamp_recompute "$TMP/a")"
  [ "$s" = "$want" ] || { fail t01 "stamp $s != independently computed $want"; return; }
  grep -qF "($s)" "$h" && grep -qF "\"commit\":\"$s\"" "$h" \
    && ok t01 || fail t01 "footer or cs-data commit field does not carry $s"
}
t02_double_render_stable() {                                           # AC 2 - §1.3
  LC_ALL=C       node "$R" "$TMP/a" "$TMP/a/o1" >/dev/null 2>&1 || { fail t02 "render (LC_ALL=C) failed"; return; }
  LC_ALL=C.UTF-8 node "$R" "$TMP/a" "$TMP/a/o2" >/dev/null 2>&1 || { fail t02 "render (LC_ALL=C.UTF-8) failed"; return; }
  diff -r "$TMP/a/o1" "$TMP/a/o2" >/dev/null 2>&1 || { fail t02 "populated corpus diverged across locales"; return; }
  diff -r "$TMP/a/out" "$TMP/a/o1" >/dev/null 2>&1 || { fail t02 "re-render diverged from first render"; return; }
  mkdir -p "$TMP/e/docs/tasks" "$TMP/e/modules/templates/html" "$TMP/e/modules/templates/cds"
  cp "$repo/modules/templates/html/status-hub.html" "$repo/modules/templates/html/status-app.js" \
     "$TMP/e/modules/templates/html/"
  cp "$repo/modules/templates/cds/tokens.css" "$repo/modules/templates/cds/status.css" \
     "$TMP/e/modules/templates/cds/"
  echo "0.0.1" > "$TMP/e/VERSION"
  CYBEROS_HUB_LENIENT=1 node "$R" "$TMP/e" "$TMP/e/o1" >/dev/null 2>&1 || { fail t02 "empty-corpus render failed"; return; }
  CYBEROS_HUB_LENIENT=1 node "$R" "$TMP/e" "$TMP/e/o2" >/dev/null 2>&1 || { fail t02 "empty-corpus render 2 failed"; return; }
  diff -r "$TMP/e/o1" "$TMP/e/o2" >/dev/null 2>&1 \
    && grep -Eq 'built from <span class="code">fp-[0-9a-f]{12}</span>' "$TMP/e/o1/reference/status.html" \
    || { fail t02 "empty corpus diverged or stamp not fp-"; return; }
  # no VERSION: stamp still covers template bytes (TASK-DOCS-010). Independent recompute
  # must match — proves nothing hidden (paths, clock, live git) leaks into the page stamp.
  rm "$TMP/e/VERSION"
  CYBEROS_HUB_LENIENT=1 node "$R" "$TMP/e" "$TMP/e/o3" >/dev/null 2>&1 || { fail t02 "no-input render failed"; return; }
  want="fp-$(stamp_recompute "$TMP/e")"
  [ "$(stamp_of "$TMP/e/o3/reference/status.html")" = "$want" ] \
    && ok t02 || fail t02 "no-VERSION stamp != recomputed $want (hidden input?)"
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
  # Stamp must not chase HEAD. Embedded feed may include git head/cov and thus differ;
  # the IMP-082 guarantee is the provenance stamp, not feed-byte identity.
  s1="$(stamp_of "$TMP/g/docs/status/reference/status.html")"
  s2="$(stamp_of "$TMP/g/out2/reference/status.html")"
  [ -n "$s1" ] && [ "$s1" = "$s2" ] \
    && ok t03 || fail t03 "render -> commit page -> render moved the stamp ($s1 -> $s2)"
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
  CYBEROS_COMMIT= node "$R" "$TMP/a" "$TMP/a/o6" >/dev/null 2>&1 || { fail t05 "empty-pin render failed"; return; }
  grep -Eq 'built from <span class="code">fp-[0-9a-f]{12}</span>' "$TMP/a/o6/reference/status.html" \
    && ok t05 || fail t05 "empty CYBEROS_COMMIT did not fall through to fp-"
}
t06_page_stamp_ignores_git() {                                         # AC 6 - §1.6 (updated TASK-DOCS-010)
  mkfix "$TMP/n"
  CYBEROS_PROJECT=scratch node "$R" "$TMP/n" "$TMP/n/out" >/dev/null 2>&1 \
    || { fail t06 "render failed in a non-git dir"; return; }
  s="$(stamp_of "$TMP/n/out/reference/status.html")"
  grep -Eq '^fp-[0-9a-f]{12}$' <<<"$s" || { fail t06 "non-git stamp is '$s', not an fp- fingerprint"; return; }
  [ -f "$TMP/n/out/reference/data/status-feed.json" ] || { fail t06 "status-feed.json missing"; return; }
  if command -v git >/dev/null 2>&1; then
    mkfix "$TMP/n2"
    git -C "$TMP/n2" init -q >/dev/null 2>&1
    git -C "$TMP/n2" -c user.email=t@t -c user.name=t -c commit.gpgsign=false add -A >/dev/null 2>&1
    git -C "$TMP/n2" -c user.email=t@t -c user.name=t -c commit.gpgsign=false commit -qm x >/dev/null 2>&1
    CYBEROS_PROJECT=scratch node "$R" "$TMP/n2" "$TMP/n2/out" >/dev/null 2>&1 || { fail t06 "render failed in git copy"; return; }
    # Page stamp must ignore live git; embedded feed may differ (head/cov). Compare stamps only.
    s2="$(stamp_of "$TMP/n2/out/reference/status.html")"
    [ "$s" = "$s2" ] || { fail t06 "git presence changed the page stamp ($s -> $s2)"; return; }
  fi
  ok t06
}

t01_fingerprint_on_all_surfaces; t02_double_render_stable; t03_commit_chase_ended
t04_corpus_edit_changes_once; t05_env_pin_wins; t06_page_stamp_ignores_git
echo "----"; echo "pass=$PASS fail=$FAIL"; [ "$FAIL" -eq 0 ]
