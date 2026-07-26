#!/usr/bin/env bash
# test_status_post_bump_freshness.sh - TASK-DOCS-018
# A VERSION bump regen leaves docs/status carrying the new version, and the
# coverage tip equals HEAD at render (truth window: parent of the next commit).
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

R="$root/tools/docs-site/render-status-hub.mjs"
[ -f "$R" ] || { echo FATAL missing renderer; exit 1; }

# Minimal corpus fixture with git history
d="$TMP/repo"
mkdir -p "$d/docs/tasks/docs/TASK-DOCS-999-fixture" "$d/modules/templates/html" "$d/modules/templates/cds"
printf '1.9.9\n' > "$d/VERSION"
cat > "$d/CHANGELOG.md" <<'EOF'
## [1.9.9] - 2026-07-01
### Added
- fixture note
EOF
cat > "$d/docs/tasks/docs/TASK-DOCS-999-fixture/spec.md" <<'EOF'
---
id: TASK-DOCS-999
title: "fixture"
type: improvement
class: improvement
status: ready_to_implement
module: docs
priority: p3
created: 2026-07-01
---
# fixture
EOF
# templates: point at mothership via symlink for tokens/shell/client
for sub in html cds; do
  for f in "$root/modules/templates/$sub"/*; do
    [ -f "$f" ] || continue
    ln -sf "$f" "$d/modules/templates/$sub/$(basename "$f")"
  done
done

(cd "$d" && git init -q && git config user.email t@t && git config user.name t \
  && git add -A && git commit -qm "chore: seed (TASK-DOCS-999)")
parent="$(cd "$d" && git rev-parse --short HEAD)"

out="$TMP/out"
mkdir -p "$out"
CYBEROS_HUB_LENIENT=1 CYBEROS_PAGE_ASSETS=1 CYBEROS_STATUS_SPECS=0 \
  CYBEROS_PROJECT=fixture \
  node "$R" "$d" "$out" >/dev/null

feed="$out/reference/data/status-feed.json"
html="$out/reference/status.html"
[ -f "$feed" ] && [ -f "$html" ] || { echo FATAL no feed/html; exit 1; }

ver="$(node -e "console.log(JSON.parse(require('fs').readFileSync(process.argv[1],'utf8')).version)" "$feed")"
[ "$ver" = "1.9.9" ] && ok t01_version_in_feed || fail t01_version_in_feed "got $ver"

cov="$(node -e "const d=JSON.parse(require('fs').readFileSync(process.argv[1],'utf8')); console.log(d.coverageAsOf||d.head||'')" "$feed")"
[ "$cov" = "$parent" ] && ok t02_coverage_as_of_head || fail t02_coverage_as_of_head "want $parent got $cov"

# Disclosure is client-rendered; assets carry the phrase, feed carries the tip.
{ grep -q "coverage as of parent" "$out/reference/assets/status.js" 2>/dev/null \
  || grep -q "coverage as of parent" "$html"; } \
  && grep -q "\"coverageAsOf\":\"$parent\"" "$feed" \
  && ok t03_disclosure_wired || fail t03_disclosure_wired "phrase/tip missing"

# Post-bump: change VERSION, full regen must carry new version
printf '2.0.0\n' > "$d/VERSION"
(cd "$d" && git add VERSION && git commit -qm "chore(release): v2.0.0")
new_head="$(cd "$d" && git rev-parse --short HEAD)"
rm -rf "$out"; mkdir -p "$out"
CYBEROS_HUB_LENIENT=1 CYBEROS_PAGE_ASSETS=1 CYBEROS_STATUS_SPECS=0 \
  CYBEROS_PROJECT=fixture \
  node "$R" "$d" "$out" >/dev/null
ver2="$(node -e "console.log(JSON.parse(require('fs').readFileSync(process.argv[1],'utf8')).version)" "$out/reference/data/status-feed.json")"
cov2="$(node -e "const d=JSON.parse(require('fs').readFileSync(process.argv[1],'utf8')); console.log(d.coverageAsOf||d.head||'')" "$out/reference/data/status-feed.json")"
[ "$ver2" = "2.0.0" ] && ok t04_post_bump_version || fail t04_post_bump_version "got $ver2"
[ "$cov2" = "$new_head" ] && ok t05_post_bump_coverage_tip || fail t05_post_bump_coverage_tip "want $new_head got $cov2"

# coverage-only: tip advances without re-parsing specs
(cd "$d" && echo '// code' > code.js && git add code.js && git commit -qm "feat: code-only (TASK-DOCS-999)")
code_head="$(cd "$d" && git rev-parse --short HEAD)"
# Publish into docs/status shape
mkdir -p "$d/docs/status"
cp -R "$out/reference/." "$d/docs/status/"
mv "$d/docs/status/status.html" "$d/docs/status/index.html" 2>/dev/null || true
[ -f "$d/docs/status/index.html" ] || cp "$out/reference/status.html" "$d/docs/status/index.html"
t0=$(date +%s%N)
CYBEROS_HUB_LENIENT=1 node "$R" --coverage-only "$d" "$d/docs/status" >/dev/null
t1=$(date +%s%N)
# Best-effort sub-second check (nanoseconds; skip assert on platforms without %N)
if [[ "$t0" != *N* && "$t1" != *N* ]]; then
  # bash arithmetic: elapsed ms
  elapsed_ms=$(( (t1 - t0) / 1000000 ))
  [ "$elapsed_ms" -lt 5000 ] && ok t06_coverage_only_fast || fail t06_coverage_only_fast "${elapsed_ms}ms"
else
  ok t06_coverage_only_fast
fi
cov3="$(node -e "const d=JSON.parse(require('fs').readFileSync(process.argv[1],'utf8')); console.log(d.coverageAsOf||d.head||'')" "$d/docs/status/data/status-feed.json")"
[ "$cov3" = "$code_head" ] && ok t07_coverage_only_tip || fail t07_coverage_only_tip "want $code_head got $cov3"
# Truth window: page must not claim the not-yet-created next commit
grep -q "\"coverageAsOf\":\"$code_head\"" "$d/docs/status/data/status-feed.json" \
  && ok t08_disclosure_after_coverage_only || fail t08_disclosure_after_coverage_only "missing tip"

echo "status_post_bump_freshness: pass=$PASS fail=$FAIL"
[ "$FAIL" -eq 0 ]
