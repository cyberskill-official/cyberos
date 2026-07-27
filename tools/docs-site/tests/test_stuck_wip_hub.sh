#!/usr/bin/env bash
# test_stuck_wip_hub.sh — TASK-IMP-143 AC1: G13 stuck-WIP surface renders on the status hub.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
RENDER="$repo/tools/docs-site/render-status-hub.mjs"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok() { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

echo "test_stuck_wip_hub.sh (TASK-IMP-143)"

mkfix() {
  local d="$1"
  mkdir -p "$d/docs/tasks/improvement/TASK-STALE-001-old" \
           "$d/modules/templates/html" "$d/modules/templates/cds" "$d/docs/batches"
  cp "$repo/modules/templates/html/status-hub.html" "$repo/modules/templates/html/status-app.js" \
     "$d/modules/templates/html/"
  cp "$repo/modules/templates/cds/tokens.css" "$repo/modules/templates/cds/status.css" \
     "$d/modules/templates/cds/"
  printf '1.0.0\n' > "$d/VERSION"
  cat > "$d/CHANGELOG.md" <<'EOF'
# Changelog
## [1.0.0] - 2026-07-25
### Added
- fixture
EOF
  cat > "$d/docs/tasks/improvement/TASK-STALE-001-old/spec.md" <<'EOF'
---
id: TASK-STALE-001
title: Stale WIP fixture
type: improvement
module: improvement
status: implementing
created: 2026-01-01
created_at: 2026-01-01T00:00:00Z
---
# body
EOF
}

mkfix "$TMP/a"
OUT="$TMP/a/out"
CYBEROS_STATUS_SPECS=0 CYBEROS_HUB_ASOF=2026-07-25 CYBEROS_G13_THRESHOLD_DAYS=30 \
  node "$RENDER" "$TMP/a" "$OUT" >"$TMP/log" 2>&1 \
  || { fail t01 "render failed: $(cat "$TMP/log")"; echo "FAIL $FAIL"; exit 1; }

html="$OUT/reference/status.html"
visible="$(python3 -c "
import re,sys
h=open(sys.argv[1]).read()
h=re.sub(r'<script[\s\S]*?</script>','',h,flags=re.I)
h=re.sub(r'<style[\s\S]*?</style>','',h,flags=re.I)
print(h)
" "$html")"

grep -q 'Stuck WIP (G13)' <<<"$visible" || { fail t01 "no Stuck WIP heading outside payload"; echo "fail"; exit 1; }
grep -q 'resume / route_back / on_hold' <<<"$visible" || { fail t01 "no triage hint"; exit 1; }
grep -q 'STALE-001' <<<"$visible" || { fail t01 "stale task id not rendered"; exit 1; }
# Ensure Date.now is absent from the stuck-WIP classifier block
python3 - <<'PY' "$RENDER" || { fail t01 "Date.now() call used near stuck WIP"; exit 1; }
import sys,re
src=open(sys.argv[1]).read()
i=src.find('stuck WIP (G13')
j=src.find('The default stamp is a fingerprint', i)
assert i>0 and j>i
block=src[i:j]
# Strip // comments, then forbid Date.now( calls (wall clock).
stripped=re.sub(r'//.*?$','',block,flags=re.M)
assert 'Date.now(' not in stripped, 'wall-clock Date.now() in stuck-WIP classifier'
print('clean')
PY
ok t01_stale_renders

CYBEROS_STATUS_SPECS=0 CYBEROS_HUB_ASOF=2026-01-01 CYBEROS_G13_THRESHOLD_DAYS=30 \
  node "$RENDER" "$TMP/a" "$OUT" >"$TMP/log2" 2>&1
visible2="$(python3 -c "
import re,sys
h=open(sys.argv[1]).read()
h=re.sub(r'<script[\s\S]*?</script>','',h,flags=re.I)
print(h)
" "$OUT/reference/status.html")"
grep -q 'No in-flight tasks older than' <<<"$visible2" && ok t02_empty_state \
  || fail t02 "expected empty-state line"

echo "stuck_wip_hub: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]
