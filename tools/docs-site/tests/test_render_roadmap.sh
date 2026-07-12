#!/usr/bin/env bash
# test_render_roadmap.sh - FR-DOCS-003 §5 suite (t01-t08 -> AC 1-8).
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
R="$repo/tools/docs-site/render-roadmap.mjs"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

mkfix() { # fixture: 3 inputs only
  d="$1"; mkdir -p "$d/docs/feature-requests/aa/FR-AA-001-first" "$d/docs/feature-requests/aa/FR-AA-002-second" "$d/docs/feature-requests/bb/FR-BB-001-third" "$d/.git"
  echo "ref: refs/heads/main" > "$d/.git/HEAD"; mkdir -p "$d/.git/refs/heads"; echo "abcdef1234567890" > "$d/.git/refs/heads/main"
  printf -- '---\nid: FR-AA-001\ntitle: First\nmodule: aa\npriority: MUST\nstatus: done\nclass: product\n---\nbody\n' > "$d/docs/feature-requests/aa/FR-AA-001-first/spec.md"
  printf -- '---\nid: FR-AA-002\ntitle: Second\nmodule: aa\npriority: SHOULD\nstatus: draft\nclass: improvement\n---\nbody\n' > "$d/docs/feature-requests/aa/FR-AA-002-second/spec.md"
  printf -- '---\nid: FR-BB-001\ntitle: Third\nmodule: bb\npriority: MUST\nstatus: implementing\nclass: product\n---\nbody\n' > "$d/docs/feature-requests/bb/FR-BB-001-third/spec.md"
  printf -- '---\nid: FR-AA-003\ntitle: Audit\n---\n' > "$d/docs/feature-requests/aa/FR-AA-003-x/audit.md"
  printf '# CL\n\n## [2.0.0] - 2026-07-01\n\n- newest entry\n\n## [1.0.0] - 2026-01-01\n\n- old entry\n' > "$d/CHANGELOG.md"
  echo "2.0.0" > "$d/VERSION"
}

t01_inputs_and_stdlib_only() {                                         # AC 1
  ! grep -qE "require\(|from ['\"](?!node:)" "$R" || { fail t01 "non-stdlib import"; return; }
  grep -q "node:fs" "$R" && mkfix "$TMP/a" && node "$R" "$TMP/a" "$TMP/a/out" >/dev/null 2>&1 \
    && ok t01 || fail t01 "fixture-only run failed"
}
t02_four_blocks_true_counts() {                                        # AC 2
  h="$TMP/a/out/reference/roadmap.html"
  grep -q 'VERSION <span class="code">2.0.0' "$h" && grep -q "abcdef123456" "$h" \
    && grep -q "v2.0.0" "$h" && grep -q "v1.0.0" "$h" \
    && [ "$(grep -o 'data-status="done"' "$h" | wc -l)" -eq 1 ] \
    && [ "$(grep -o 'data-status="draft"' "$h" | wc -l)" -eq 1 ] \
    && grep -q '<td class="code">aa</td><td>2</td><td>1</td><td>1</td>' "$h" \
    && awk '/v2.0.0/{f2=NR} /v1.0.0/{f1=NR} END{exit !(f2<f1)}' "$h" \
    && ok t02 || fail t02 "blocks/counts wrong"
}
t03_filters_and_nojs_degrade() {                                       # AC 3
  h="$TMP/a/out/reference/roadmap.html"
  grep -q 'id="f-module"' "$h" && grep -q "dataset.module" "$h" \
    && [ "$(grep -o 'class="fr-row"' "$h" | wc -l)" -eq 3 ] \
    && ! grep -q 'fr-row" hidden' "$h" \
    && ok t03 || fail t03 "filter script or rows missing/hidden by default"
}
t04_wired_build_deploy_release() {                                     # AC 4
  grep -q "render-roadmap.mjs" "$repo/tools/docs-site/build.sh" \
    && grep -q "tools/docs-site/build.sh" "$repo/.github/workflows/deploy.yml" \
    && grep -q "tools/docs-site/build.sh" "$repo/.github/workflows/release.yml" \
    && grep -q "apps/console/docs" "$repo/.github/workflows/release.yml" \
    && ok t04 || fail t04 "wiring missing"
}
t05_byte_identical_rebuilds() {                                        # AC 5
  node "$R" "$TMP/a" "$TMP/a/out2" >/dev/null 2>&1
  if cmp -s "$TMP/a/out/reference/roadmap.html" "$TMP/a/out2/reference/roadmap.html"; then
    mkdir -p "$TMP/a/docs/feature-requests/bb/FR-BB-002-new"
    printf -- '---\nid: FR-BB-002\ntitle: New\nmodule: bb\npriority: MUST\nstatus: draft\nclass: product\n---\n' > "$TMP/a/docs/feature-requests/bb/FR-BB-002-new/spec.md"
    node "$R" "$TMP/a" "$TMP/a/out3" >/dev/null 2>&1
    ! cmp -s "$TMP/a/out/reference/roadmap.html" "$TMP/a/out3/reference/roadmap.html" \
      && ok t05 || fail t05 "output did not change with new FR"
    rm "$TMP/a/docs/feature-requests/bb/FR-BB-002-new/spec.md"
  else fail t05 "double build diverged"; fi
}
t06_honest_failures() {                                                # AC 6
  mkfix "$TMP/b"
  mkdir -p "$TMP/b/docs/feature-requests/aa/FR-AA-009-broken"
  printf -- '---\nid: FR-AA-009\nno closing fence\n' > "$TMP/b/docs/feature-requests/aa/FR-AA-009-broken/spec.md"
  out="$(node "$R" "$TMP/b" "$TMP/b/out" 2>&1)"; rc=$?
  [ "$rc" -ne 0 ] && grep -q "FR-AA-009" <<<"$out" || { fail t06 "broken fm rc=$rc"; return; }
  mkfix "$TMP/c"; printf '# empty\n' > "$TMP/c/CHANGELOG.md"
  out="$(node "$R" "$TMP/c" "$TMP/c/out" 2>&1)"; rc=$?
  [ "$rc" -ne 0 ] && grep -q "zero version sections" <<<"$out" && ok t06 || fail t06 "empty changelog rc=$rc"
}
t07_nav_link_present() {                                               # AC 7
  grep -q 'reference/roadmap.html' "$repo/tools/docs-site/render-docs.mjs" \
    && { [ ! -d "$repo/dist/website" ] || grep -rq "roadmap.html" "$repo/dist/website/assets/nav.html" 2>/dev/null || grep -rq "roadmap.html" "$repo/dist/website/chrome/nav.html" 2>/dev/null || true; } \
    && ok t07 || fail t07 "nav hook missing"
}
t08_token_clean_styles() {                                             # AC 8
  h="$TMP/a/out/reference/roadmap.html"
  hexes="$(sed -n '/:root {/,/}/!p' "$h" | grep -oE '#[0-9a-fA-F]{3,6}\b' | grep -v "^#$" | wc -l)"
  [ "$hexes" -eq 0 ] && grep -q "var(--bg)" "$h" && ok t08 || fail t08 "$hexes hex colors outside :root token block"
}

t01_inputs_and_stdlib_only; t02_four_blocks_true_counts; t03_filters_and_nojs_degrade
t04_wired_build_deploy_release; t05_byte_identical_rebuilds; t06_honest_failures
t07_nav_link_present; t08_token_clean_styles
echo "----"; echo "pass=$PASS fail=$FAIL"; [ "$FAIL" -eq 0 ]
