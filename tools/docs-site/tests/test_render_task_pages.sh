#!/usr/bin/env bash
# test_render_task_pages.sh - TASK-DOCS-005 §5 suite (t01-t06 -> AC 1-6).
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
R="$repo/tools/docs-site/render-task-pages.mjs"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

mkfix() {
  d="$1"; mkdir -p "$d/docs/tasks/aa/TASK-AA-001-first/assets" "$d/modules/templates/html" "$d/modules/templates/cds" "$d/tools/docs-site"
  cp "$repo/modules/templates/html/deliverable.html" "$d/modules/templates/html/"
  cp "$repo/modules/templates/cds/tokens.css" "$d/modules/templates/cds/"
  cp "$repo/tools/docs-site/md.mjs" "$d/tools/docs-site/"
  cp "$R" "$d/tools/docs-site/"
  printf 'PNG' > "$d/docs/tasks/aa/TASK-AA-001-first/assets/pic.png"
  printf 'MP4' > "$d/docs/tasks/aa/TASK-AA-001-first/assets/demo.mp4"
  cat > "$d/docs/tasks/aa/TASK-AA-001-first/spec.md" <<'SPEC'
---
id: TASK-AA-001
title: First
module: aa
priority: MUST
status: done
class: product
created: 2026-07-01
depends_on: [TASK-AA-002]
---
## §1 - Description
Body with ![shot](assets/pic.png) and ![demo](assets/demo.mp4).
SPEC
  printf -- '---\nfr_id: TASK-AA-001\nverdict: PASS\n---\n# audit\nScore 10/10.\n' > "$d/docs/tasks/aa/TASK-AA-001-first/audit.md"
  mkdir -p "$d/docs/tasks/aa/TASK-AA-002-second"
  printf -- '---\nid: TASK-AA-002\ntitle: Second\nmodule: aa\nstatus: draft\nclass: product\n---\nbody\n' > "$d/docs/tasks/aa/TASK-AA-002-second/spec.md"
}

t01_corpus_renders() {                                                 # AC 1
  mkfix "$TMP/a"
  out="$(node "$TMP/a/tools/docs-site/render-task-pages.mjs" "$TMP/a" "$TMP/a/out" 2>&1)" || { fail t01 "$out"; return; }
  h="$TMP/a/out/tasks/aa/TASK-AA-001-first/index.html"
  grep -q 'data-template-id="deliverable@1"' "$h" && grep -q "badge-status" "$h" \
    && grep -q "Audit" "$h" && grep -q 'href="../../aa/TASK-AA-002-second/index.html"' "$h" \
    && grep -q "fr-pages: 2 pages, 2 assets copied, 1 with audits" <<<"$out" \
    && ok t01 || fail t01 "page content/summary wrong: $out"
}
t02_media() {                                                          # AC 2
  h="$TMP/a/out/tasks/aa/TASK-AA-001-first/index.html"
  grep -q '<img src="assets/pic.png"' "$h" && grep -q '<video controls src="assets/demo.mp4"' "$h" \
    && [ -f "$TMP/a/out/tasks/aa/TASK-AA-001-first/assets/demo.mp4" ] \
    && ok t02 || fail t02 "media handling"
}
t03_selfcontained() {                                                  # AC 3
  h="$TMP/a/out/tasks/aa/TASK-AA-001-first/index.html"
  ! grep -qE '(src|href)="https?://' "$h" && grep -q -- "--cs-color-brand-umber" "$h" \
    && ok t03 || fail t03 "external refs or missing tokens"
}
t04_wired_deterministic_honest() {                                     # AC 4
  grep -q "render-task-pages.mjs" "$repo/tools/docs-site/build.sh" || { fail t04 "not in build.sh"; return; }
  node "$TMP/a/tools/docs-site/render-task-pages.mjs" "$TMP/a" "$TMP/a/out2" >/dev/null 2>&1
  cmp -s "$TMP/a/out/tasks/aa/TASK-AA-001-first/index.html" "$TMP/a/out2/tasks/aa/TASK-AA-001-first/index.html" || { fail t04 "nondeterministic"; return; }
  mkfix "$TMP/b"
  sed -i 's|assets/pic.png|assets/GONE.png|' "$TMP/b/docs/tasks/aa/TASK-AA-001-first/spec.md"
  out="$(node "$TMP/b/tools/docs-site/render-task-pages.mjs" "$TMP/b" "$TMP/b/out" 2>&1)"; rc=$?
  [ "$rc" -ne 0 ] && grep -q "GONE.png" <<<"$out" && ok t04 || fail t04 "missing asset not fatal: rc=$rc"
}
t05_catalog_links() {                                                  # AC 5
  grep -q "fr-page-link" "$repo/tools/docs-site/render-task-catalog.mjs" \
    && grep -q "tasks/" "$repo/tools/docs-site/render-task-catalog.mjs" \
    && ok t05 || fail t05 "catalog not linked"
}
t06_envelope() {                                                       # AC 6
  start=$(date +%s)
  node "$R" "$repo" "$TMP/full" >/dev/null 2>&1 || { fail t06 "corpus render failed"; return; }
  dur=$(( $(date +%s) - start ))
  [ "$dur" -lt 30 ] && ok t06 || fail t06 "corpus took ${dur}s (cap 30s)"
}

t01_corpus_renders; t02_media; t03_selfcontained; t04_wired_deterministic_honest
t05_catalog_links; t06_envelope
echo "----"; echo "pass=$PASS fail=$FAIL"; [ "$FAIL" -eq 0 ]
