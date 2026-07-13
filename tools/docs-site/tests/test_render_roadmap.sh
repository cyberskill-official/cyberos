#!/usr/bin/env bash
# test_render_roadmap.sh - FR-DOCS-003 suite, repointed at the status hub per FR-DOCS-006 §1 #4
# and re-anchored on the single-page hub of FR-IMP-074. Assertions preserved: board counts,
# release order, determinism, honest failures, token-clean, plus the supersession contract.
# The roadmap CONTENT is the board lens now; #roadmap still lands there.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
R="$repo/tools/docs-site/render-status-hub.mjs"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

mkfix() {
  d="$1"
  mkdir -p "$d/docs/feature-requests/aa/FR-AA-001-first" "$d/docs/feature-requests/aa/FR-AA-002-second" \
           "$d/docs/feature-requests/bb/FR-BB-001-third" "$d/.git/refs/heads" \
           "$d/modules/templates/html" "$d/modules/templates/cds"
  cp "$repo/modules/templates/html/status-hub.html" "$repo/modules/templates/html/status-app.js" "$d/modules/templates/html/"
  cp "$repo/modules/templates/cds/tokens.css" "$repo/modules/templates/cds/status.css" "$d/modules/templates/cds/"
  echo "ref: refs/heads/main" > "$d/.git/HEAD"; echo "abcdef1234567890" > "$d/.git/refs/heads/main"
  printf -- '---\nid: FR-AA-001\ntitle: First\nmodule: aa\npriority: MUST\nstatus: done\nclass: product\n---\nbody\n' > "$d/docs/feature-requests/aa/FR-AA-001-first/spec.md"
  printf -- '---\nid: FR-AA-002\ntitle: Second\nmodule: aa\npriority: SHOULD\nstatus: draft\nclass: improvement\n---\nbody\n' > "$d/docs/feature-requests/aa/FR-AA-002-second/spec.md"
  printf -- '---\nid: FR-BB-001\ntitle: Third\nmodule: bb\npriority: MUST\nstatus: implementing\nclass: product\n---\nbody\n' > "$d/docs/feature-requests/bb/FR-BB-001-third/spec.md"
  printf '# CL\n\n## [2.0.0] - 2026-07-01\n\n- newest entry\n\n## [1.0.0] - 2026-01-01\n\n- old entry\n' > "$d/CHANGELOG.md"
  echo "2.0.0" > "$d/VERSION"
}

t01_builds_from_three_inputs() {
  mkfix "$TMP/a"
  node "$R" "$TMP/a" "$TMP/a/out" >/dev/null 2>&1 && [ -f "$TMP/a/out/reference/status.html" ] \
    && ok t01 || fail t01 "hub build failed"
}
t02_board_counts_and_release_order() {
  h="$TMP/a/out/reference/status.html"
  grep -q 'VERSION' "$h" && grep -q 'abcdef123456' "$h" \
    && grep -q '"s":"done"' "$h" && grep -q '"s":"implementing"' "$h" \
    && grep -Eq 'data-bucket="done"[^>]*><b>1</b>' "$h" && grep -Eq 'data-bucket="active"[^>]*><b>1</b>' "$h" \
    && grep -q '"vl":"v2.0.0"' "$h" && grep -q '"vl":"v1.0.0"' "$h" \
    && node -e 'const fs=require("fs");const h=fs.readFileSync(process.argv[1],"utf8");
        const d=JSON.parse(h.match(/id="cs-data">([\s\S]*?)<\/script>/)[1].replace(/\\u003c/g,"<"));
        process.exit(d.releases[0].v==="2.0.0"&&d.releases[1].v==="1.0.0"?0:1);' "$h" \
    && ok t02 || fail t02 "board counts / release order"
}
t03_supersession_stub() {
  s="$TMP/a/out/reference/roadmap.html"
  grep -q 'url=status.html#roadmap' "$s" && grep -q 'status hub' "$s" \
    && grep -q 'reference/status.html' "$repo/tools/docs-site/render-docs.mjs" \
    && ! grep -q '"reference/roadmap.html"' "$repo/tools/docs-site/render-docs.mjs" \
    && ok t03 || fail t03 "stub or nav wrong"
}
t04_wired() {
  grep -q "render-status-hub.mjs" "$repo/tools/docs-site/build.sh" \
    && ! grep -q "render-roadmap.mjs" "$repo/tools/docs-site/build.sh" \
    && ok t04 || fail t04 "build.sh wiring"
}
t05_deterministic() {
  node "$R" "$TMP/a" "$TMP/a/out2" >/dev/null 2>&1
  cmp -s "$TMP/a/out/reference/status.html" "$TMP/a/out2/reference/status.html" \
    && ok t05 || fail t05 "double build diverged"
}
t06_honest_failures() {
  mkfix "$TMP/b"
  mkdir -p "$TMP/b/docs/feature-requests/aa/FR-AA-009-broken"
  printf -- '---\nid: FR-AA-009\nno closing fence\n' > "$TMP/b/docs/feature-requests/aa/FR-AA-009-broken/spec.md"
  out="$(node "$R" "$TMP/b" "$TMP/b/out" 2>&1)"; rc=$?
  [ "$rc" -ne 0 ] && grep -q "FR-AA-009" <<<"$out" || { fail t06 "broken fm rc=$rc"; return; }
  mkfix "$TMP/c"; printf '# empty\n' > "$TMP/c/CHANGELOG.md"
  out="$(node "$R" "$TMP/c" "$TMP/c/out" 2>&1)"; rc=$?
  [ "$rc" -ne 0 ] && grep -q "zero version sections" <<<"$out" && ok t06 || fail t06 "empty changelog rc=$rc"
}
t07_token_clean() {
  h="$TMP/a/out/reference/status.html"
  hexes="$(sed -e '/:root {/,/^}/d' -e '/^\[data-theme="dark"\] {/,/^}/d' "$h" \
    | grep -oE '#[0-9a-fA-F]{3,6}\b' | wc -l | tr -d ' ')"
  [ "$hexes" -eq 0 ] && grep -q -- "--cs-color-brand-umber" "$h" && ok t07 || fail t07 "$hexes hex outside tokens"
}

t01_builds_from_three_inputs; t02_board_counts_and_release_order; t03_supersession_stub
t04_wired; t05_deterministic; t06_honest_failures; t07_token_clean
echo "----"; echo "pass=$PASS fail=$FAIL"; [ "$FAIL" -eq 0 ]
