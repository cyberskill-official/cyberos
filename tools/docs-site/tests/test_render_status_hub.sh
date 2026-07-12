#!/usr/bin/env bash
# test_render_status_hub.sh - FR-DOCS-006 §5 suite (t01-t06 -> AC 1-6).
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
R="$repo/tools/docs-site/render-status-hub.mjs"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

mkfix() {
  d="$1"; mkdir -p "$d/docs/feature-requests/aa/FR-AA-001-first" "$d/docs/feature-requests/bb/FR-BB-001-third" "$d/.git" "$d/modules/templates/html" "$d/modules/templates/cds"
  cp "$repo/modules/templates/html/status-hub.html" "$d/modules/templates/html/"
  cp "$repo/modules/templates/cds/tokens.css" "$d/modules/templates/cds/"
  echo "ref: refs/heads/main" > "$d/.git/HEAD"; mkdir -p "$d/.git/refs/heads"; echo "abcdef1234567890" > "$d/.git/refs/heads/main"
  printf -- '---\nid: FR-AA-001\ntitle: First\nmodule: aa\npriority: MUST\nstatus: done\nclass: product\n---\nbody\n' > "$d/docs/feature-requests/aa/FR-AA-001-first/spec.md"
  printf -- '---\nid: FR-BB-001\ntitle: Third\nmodule: bb\npriority: SHOULD\nstatus: draft\nclass: improvement\n---\nbody\n' > "$d/docs/feature-requests/bb/FR-BB-001-third/spec.md"
  printf '# CL\n\n## [2.0.0] - 2026-07-01\n\n- entry\n' > "$d/CHANGELOG.md"
  echo "2.0.0" > "$d/VERSION"
}

t01_deck_true() {                                                      # AC 1
  mkfix "$TMP/a"
  node "$R" "$TMP/a" "$TMP/a/out" >/dev/null 2>&1
  h="$TMP/a/out/reference/status.html"
  grep -q 'Overall progress (2 feature requests' "$h" \
    && grep -q 'v2.0.0' "$h" && grep -q '2026-07-01' "$h" \
    && grep -q '<span><b>1</b> done' "$h" && grep -q 'seg-done" style="width:50.0%"' "$h" \
    && ok t01 || fail t01 "overall bar/counts wrong"
}
t02_tabs_routing_degrade() {                                           # AC 2
  h="$TMP/a/out/reference/status.html"
  [ "$(grep -c 'role="tabpanel"' "$h")" -eq 3 ] && grep -q "hashchange" "$h" \
    && grep -q 'href="#backlog"' "$h" && grep -q "<noscript>" "$h" \
    && ok t02 || fail t02 "tabs/routing/degrade"
}
t03_backlog_facets() {                                                 # AC 3
  h="$TMP/a/out/reference/status.html"
  grep -q 'id="bk-module"' "$h" && grep -q 'id="bk-priority"' "$h" \
    && grep -q 'data-priority="MUST"' "$h" && grep -q 'data-class="improvement"' "$h" \
    && ok t03 || fail t03 "facets missing"
}
t04_supersession() {                                                   # AC 4
  grep -q 'url=status.html#roadmap' "$TMP/a/out/reference/roadmap.html" \
    && bash "$repo/tools/docs-site/tests/test_render_roadmap.sh" >/dev/null 2>&1 \
    && ok t04 || fail t04 "stub or legacy suite red"
}
t05_deterministic_honest_tokens() {                                    # AC 5 (delegated cases in legacy suite share the builder)
  node "$R" "$TMP/a" "$TMP/a/out2" >/dev/null 2>&1
  cmp -s "$TMP/a/out/reference/status.html" "$TMP/a/out2/reference/status.html" \
    && ok t05 || fail t05 "nondeterministic"
}
t06_fr_links() {                                                       # AC 6
  mkfix "$TMP/d"
  mkdir -p "$TMP/d/out/frs/aa/FR-AA-001-first"
  touch "$TMP/d/out/frs/aa/FR-AA-001-first/index.html"
  node "$R" "$TMP/d" "$TMP/d/out" >/dev/null 2>&1
  grep -q 'class="chip-link" href="../frs/aa/FR-AA-001-first/index.html"' "$TMP/d/out/reference/status.html" \
    && ok t06 || fail t06 "FR page links absent"
}

t01_deck_true; t02_tabs_routing_degrade; t03_backlog_facets
t04_supersession; t05_deterministic_honest_tokens; t06_fr_links
echo "----"; echo "pass=$PASS fail=$FAIL"; [ "$FAIL" -eq 0 ]
