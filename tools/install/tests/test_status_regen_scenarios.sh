#!/usr/bin/env bash
# test_status_regen_scenarios.sh - TASK-DOCS-017 / TASK-DOCS-018 Gate P3 scenarios
# Scratch-clone style: task edit, code-only, version bump each leave docs/status
# consistent with HEAD per the coverage disclosure rule.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

unset GIT_INDEX_FILE GIT_DIR GIT_WORK_TREE GIT_PREFIX GIT_COMMON_DIR 2>/dev/null || true

SP="$repo/tools/install/lib/status-page.sh"
R="$repo/tools/docs-site/render-status-hub.mjs"
[ -f "$SP" ] && [ -f "$R" ] || { echo FATAL missing tools; exit 1; }

mk_scratch() {
  local d="$1"
  mkdir -p "$d/docs/tasks/docs/TASK-DOCS-998-scratch" "$d/modules/templates/html" "$d/modules/templates/cds"
  printf '1.0.0\n' > "$d/VERSION"
  cat > "$d/CHANGELOG.md" <<'EOF'
## [1.0.0] - 2026-07-01
### Added
- scratch
EOF
  cat > "$d/docs/tasks/docs/TASK-DOCS-998-scratch/spec.md" <<'EOF'
---
id: TASK-DOCS-998
title: "scratch scenario"
type: improvement
class: improvement
status: ready_to_implement
module: docs
priority: p3
created: 2026-07-01
---
# scratch
EOF
  for sub in html cds; do
    for f in "$repo/modules/templates/$sub"/*; do
      [ -f "$f" ] || continue
      ln -sf "$f" "$d/modules/templates/$sub/$(basename "$f")"
    done
  done
  # Point status-page at platform tools via a fake .cyberos that delegates — use tools path.
  mkdir -p "$d/.cyberos/lib" "$d/.cyberos/docs-tools/templates"
  cp "$repo/tools/install/lib/status-page.sh" "$d/.cyberos/lib/"
  cp "$repo/tools/install/lib/task-migrate.sh" "$d/.cyberos/lib/"
  # Minimal docs-tools: symlink renderer + md + status-feed from platform
  ln -sf "$repo/tools/docs-site/render-status-hub.mjs" "$d/.cyberos/docs-tools/render-status-hub.mjs"
  ln -sf "$repo/tools/docs-site/status-feed.mjs" "$d/.cyberos/docs-tools/status-feed.mjs"
  ln -sf "$repo/tools/docs-site/md.mjs" "$d/.cyberos/docs-tools/md.mjs"
  for f in status-hub.html status-app.js status-hub-legacy.html status-app-legacy.js; do
    ln -sf "$repo/modules/templates/html/$f" "$d/.cyberos/docs-tools/templates/$f" 2>/dev/null || true
  done
  for f in status.css status-legacy.css tokens.css; do
    ln -sf "$repo/modules/templates/cds/$f" "$d/.cyberos/docs-tools/templates/$f" 2>/dev/null || true
  done
  (cd "$d" && git init -q && git config user.email t@t && git config user.name t \
    && git add -A && git commit -qm "chore: seed (TASK-DOCS-998)")
}

assert_consistent() {
  local d="$1" label="$2"
  local head tip
  head="$(cd "$d" && git rev-parse --short HEAD)"
  tip="$(node -e "const d=JSON.parse(require('fs').readFileSync(process.argv[1],'utf8')); console.log(d.coverageAsOf||d.head||'')" \
    "$d/docs/status/data/status-feed.json")"
  # Disclosure rule: tip equals HEAD at render (= current HEAD after the scenario commit
  # that staged the page... but the page was staged IN that commit, so tip is parent.
  # After the commit lands, HEAD advanced; the committed page tip is the parent.
  local parent
  parent="$(cd "$d" && git rev-parse --short HEAD^)"
  if [ "$tip" = "$parent" ] || [ "$tip" = "$head" ]; then
    # Accept tip==parent (truth window in the commit) OR tip==head (post-hoc regen).
    ok "$label"
  else
    fail "$label" "coverageAsOf=$tip head=$head parent=$parent"
  fi
  grep -q "\"coverageAsOf\":\"$tip\"" "$d/docs/status/data/status-feed.json" \
    && ok "${label}_disclosure" || fail "${label}_disclosure" "missing coverageAsOf=$tip in feed"
}

# --- scenario A: task edit commit ---
A="$TMP/a"; mk_scratch "$A"
bash "$SP" "$A" >/dev/null
(cd "$A" && git add docs/status && git commit -qm "chore: initial status (TASK-DOCS-998)" || true)
echo "updated summary" >> "$A/docs/tasks/docs/TASK-DOCS-998-scratch/spec.md"
(cd "$A" && git add docs/tasks && bash "$SP" "$A" >/dev/null && git add docs/status \
  && git commit -qm "docs(tasks): edit scratch task (TASK-DOCS-998)")
assert_consistent "$A" t01_task_edit
verA="$(node -e "console.log(JSON.parse(require('fs').readFileSync(process.argv[1],'utf8')).version)" "$A/docs/status/data/status-feed.json")"
[ "$verA" = "1.0.0" ] && ok t01b_version_stable || fail t01b_version_stable "$verA"

# --- scenario B: code-only commit (coverage-only path) ---
B="$TMP/b"; mk_scratch "$B"
bash "$SP" "$B" >/dev/null
(cd "$B" && git add docs/status && git commit -qm "chore: initial status (TASK-DOCS-998)")
echo 'console.log(1)' > "$B/app.js"
(cd "$B" && git add app.js && bash "$SP" "$B" --coverage-only >/dev/null && git add docs/status \
  && git commit -qm "feat: code-only change (TASK-DOCS-998)")
assert_consistent "$B" t02_code_only

# --- scenario C: version bump ---
C="$TMP/c"; mk_scratch "$C"
bash "$SP" "$C" >/dev/null
(cd "$C" && git add docs/status && git commit -qm "chore: initial status (TASK-DOCS-998)")
printf '1.1.0\n' > "$C/VERSION"
(cd "$C" && git add VERSION && bash "$SP" "$C" >/dev/null && git add docs/status \
  && git commit -qm "chore(release): v1.1.0")
assert_consistent "$C" t03_version_bump
verC="$(node -e "console.log(JSON.parse(require('fs').readFileSync(process.argv[1],'utf8')).version)" "$C/docs/status/data/status-feed.json")"
[ "$verC" = "1.1.0" ] && ok t03b_version_in_page || fail t03b_version_in_page "$verC"

echo "status_regen_scenarios: pass=$PASS fail=$FAIL"
[ "$FAIL" -eq 0 ]
