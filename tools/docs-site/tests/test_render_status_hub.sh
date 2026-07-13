#!/usr/bin/env bash
# test_render_status_hub.sh - FR-DOCS-006 §5 (t01-t06) carried forward to the single-page
# hub of FR-IMP-074: Roadmap | Backlog | Changelog are no longer three tabs but three lenses
# (board | table | releases) over one corpus, plus an FR drawer that carries the full spec.
# t07-t09 cover what the merge added: changelog->FR binding, lazy spec chunks, no-JS truth.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
R="$repo/tools/docs-site/render-status-hub.mjs"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

mkfix() {
  d="$1"
  mkdir -p "$d/docs/feature-requests/aa/FR-AA-001-first" "$d/docs/feature-requests/bb/FR-BB-001-third" \
           "$d/.git/refs/heads" "$d/modules/templates/html" "$d/modules/templates/cds"
  cp "$repo/modules/templates/html/status-hub.html" "$repo/modules/templates/html/status-app.js" "$d/modules/templates/html/"
  cp "$repo/modules/templates/cds/tokens.css" "$repo/modules/templates/cds/status.css" "$d/modules/templates/cds/"
  echo "ref: refs/heads/main" > "$d/.git/HEAD"; echo "abcdef1234567890" > "$d/.git/refs/heads/main"
  printf -- '---\nid: FR-AA-001\ntitle: First\nmodule: aa\npriority: MUST\nstatus: done\nclass: product\nphase: P0\nowner: Ada\neffort_hours: 3\nshipped: 2026-07-01\ndepends_on: []\nblocks: [FR-BB-001]\nsub_tasks:\n  - "wire it"\n  - "prove it"\n---\n## §1 — Description\n\nFirst FR body paragraph.\n\n## §4 — Acceptance criteria\n\n- one\n' > "$d/docs/feature-requests/aa/FR-AA-001-first/spec.md"
  printf -- '---\nid: FR-BB-001\ntitle: Third\nmodule: bb\npriority: SHOULD\nstatus: draft\nclass: improvement\nphase: P1\ndepends_on: [FR-AA-001]\n---\n## §1 — Description\n\nThird FR body paragraph.\n' > "$d/docs/feature-requests/bb/FR-BB-001-third/spec.md"
  printf '# CL\n\n## [2.0.0] - 2026-07-01\n\nAdded\n- FR-AA-001 first thing landed\n' > "$d/CHANGELOG.md"
  echo "2.0.0" > "$d/VERSION"
}

t01_deck_true() {                                                      # AC 1 - the deck is generated truth
  mkfix "$TMP/a"
  node "$R" "$TMP/a" "$TMP/a/out" >/dev/null 2>&1
  h="$TMP/a/out/reference/status.html"
  grep -q 'Overall progress · 2 feature requests · 2 modules' "$h" \
    && grep -q 'VERSION <span class="code">2.0.0</span>' "$h" && grep -q 'abcdef123456' "$h" \
    && grep -Eq 'data-bucket="done"[^>]*><b>1</b>' "$h" \
    && grep -q 'seg-done" style="width:50.0%"' "$h" \
    && ok t01 || fail t01 "deck counts / stamp wrong"
}
t02_one_page_three_lenses() {                                          # AC 2 - the merge itself
  h="$TMP/a/out/reference/status.html"
  ! grep -q 'role="tabpanel"' "$h" \
    && [ "$(grep -co 'class="ln" role="tab"' "$h")" -eq 3 ] \
    && grep -q 'data-lens="board"' "$h" && grep -q 'data-lens="table"' "$h" && grep -q 'data-lens="timeline"' "$h" \
    && grep -q 'roadmap: "board", backlog: "table", changelog: "timeline"' "$h" \
    && ok t02 || fail t02 "lenses / legacy hash mapping"
}
t03_facets_and_search() {                                              # AC 3 - one filter set, every lens
  h="$TMP/a/out/reference/status.html"
  for id in f-m f-s f-p f-c f-ph f-g; do grep -q "id=\"$id\"" "$h" || { fail t03 "facet $id missing"; return; }; done
  grep -q 'id="q"' "$h" && grep -q '"p":"MUST"' "$h" && grep -q '"c":"improvement"' "$h" \
    && grep -q '"ph":"P0"' "$h" && grep -q '"st":\["wire it","prove it"\]' "$h" \
    && ok t03 || fail t03 "search box or corpus fields missing"
}
t04_supersession() {                                                   # AC 4 - bookmarks survive
  grep -q 'url=status.html#roadmap' "$TMP/a/out/reference/roadmap.html" \
    && bash "$repo/tools/docs-site/tests/test_render_roadmap.sh" >/dev/null 2>&1 \
    && ok t04 || fail t04 "stub or legacy suite red"
}
t05_deterministic() {                                                  # AC 5 - same input, same bytes
  node "$R" "$TMP/a" "$TMP/a/out2" >/dev/null 2>&1
  cmp -s "$TMP/a/out/reference/status.html" "$TMP/a/out2/reference/status.html" \
    && cmp -s "$TMP/a/out/reference/data/fr/FR-AA-001.js" "$TMP/a/out2/reference/data/fr/FR-AA-001.js" \
    && ok t05 || fail t05 "nondeterministic"
}
t06_fr_page_links() {                                                  # AC 6 - the drawer links the FR page
  mkfix "$TMP/d"
  mkdir -p "$TMP/d/out/frs/aa/FR-AA-001-first"; touch "$TMP/d/out/frs/aa/FR-AA-001-first/index.html"
  node "$R" "$TMP/d" "$TMP/d/out" >/dev/null 2>&1
  grep -q '"pg":"../frs/aa/FR-AA-001-first/index.html"' "$TMP/d/out/reference/status.html" \
    && ok t06 || fail t06 "FR page link absent from the corpus"
}
t07_changelog_binds_frs() {                                            # the changelog references FRs, not prose
  h="$TMP/a/out/reference/status.html"
  grep -q '"cited":\["FR-AA-001"\]' "$h" \
    && grep -q 'data-fr=\\"FR-AA-001\\"' "$h" \
    && grep -q '"bound":\["FR-AA-001"\]' "$h" \
    && ok t07 || fail t07 "release -> FR binding missing"
}
t08_spec_chunks() {                                                    # full spec, lazily
  c="$TMP/a/out/reference/data/fr/FR-AA-001.js"
  [ -f "$c" ] && grep -q 'window.CS_SPEC\["FR-AA-001"\]' "$c" \
    && grep -q 'First FR body paragraph' "$c" \
    && grep -q '"sp":1' "$TMP/a/out/reference/status.html" || { fail t08 "chunk missing"; return; }
  CYBEROS_STATUS_SPECS=0 node "$R" "$TMP/a" "$TMP/a/out3" >/dev/null 2>&1
  [ ! -d "$TMP/a/out3/reference/data" ] && ! grep -q '"sp":1' "$TMP/a/out3/reference/status.html" \
    && ok t08 || fail t08 "CYBEROS_STATUS_SPECS=0 still emitted chunks"
}
t09_nojs_and_honest_failures() {                                       # degrade, and fail loudly
  h="$TMP/a/out/reference/status.html"
  grep -q '<noscript>' "$h" && grep -q 'FR-BB-001' "$h" || { fail t09 "no-JS fallback missing"; return; }
  mkfix "$TMP/b"
  mkdir -p "$TMP/b/docs/feature-requests/aa/FR-AA-009-broken"
  printf -- '---\nid: FR-AA-009\nno closing fence\n' > "$TMP/b/docs/feature-requests/aa/FR-AA-009-broken/spec.md"
  out="$(node "$R" "$TMP/b" "$TMP/b/out" 2>&1)"; rc=$?
  [ "$rc" -ne 0 ] && grep -q "FR-AA-009" <<<"$out" && ok t09 || fail t09 "broken frontmatter did not fail loudly (rc=$rc)"
}
t10_token_clean() {                                                    # colour literals stay in the token layer
  h="$TMP/a/out/reference/status.html"
  hexes="$(sed -e '/:root {/,/^}/d' -e '/^\[data-theme="dark"\] {/,/^}/d' "$h" \
    | grep -oE '#[0-9a-fA-F]{3,6}\b' | wc -l | tr -d ' ')"
  [ "$hexes" -eq 0 ] && grep -q -- '--cs-color-brand-umber' "$h" \
    && ok t10 || fail t10 "$hexes raw hex outside the token blocks"
}

t01_deck_true; t02_one_page_three_lenses; t03_facets_and_search; t04_supersession
t05_deterministic; t06_fr_page_links; t07_changelog_binds_frs; t08_spec_chunks
t09_nojs_and_honest_failures; t10_token_clean
echo "----"; echo "pass=$PASS fail=$FAIL"; [ "$FAIL" -eq 0 ]
