#!/usr/bin/env bash
# test_offline_status_cert.sh - TASK-DOCS-023
# From a built payload alone (no network): install into a scratch repo, create a
# task, commit, and confirm docs/status/index.html is a v3 page with local cov.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

unset GIT_INDEX_FILE GIT_DIR GIT_WORK_TREE GIT_PREFIX GIT_COMMON_DIR 2>/dev/null || true

echo "building scratch payload..."
bash "$repo/tools/install/build.sh" "$TMP/payload" >/dev/null 2>&1 || { echo FATAL build; exit 1; }

d="$TMP/scratch"
mkdir -p "$d"
(cd "$d" && git init -q && git config user.email t@t && git config user.name t \
  && echo '# scratch' > README.md && git add README.md && git commit -qm "chore: seed (TASK-DOCS-023)")

# Offline: clear proxy/network hints; install from local payload only.
env -u http_proxy -u https_proxy -u HTTP_PROXY -u HTTPS_PROXY -u ALL_PROXY \
  bash "$TMP/payload/install.sh" "$d" >/dev/null 2>&1 \
  && ok t01_offline_install || fail t01_offline_install "install failed"

[ -f "$d/.cyberos/lib/check_task_link.sh" ] && ok t02_checker_vendored || fail t02_checker_vendored "missing"
[ -f "$d/.cyberos/lib/status-page.sh" ] && ok t03_status_page_vendored || fail t03_status_page_vendored "missing"
grep -q 'cutoff:' "$d/.cyberos/config.yaml" && ok t04_cutoff_written || fail t04_cutoff_written "no cutoff"
[ ! -f "$d/.github/workflows/cyberos-traceability.yml" ] \
  && ok t05_ci_not_scaffolded_by_default || fail t05_ci_not_scaffolded_by_default "CI scaffolded despite opt-in default"

# Create a minimal task + commit with task link
mkdir -p "$d/docs/tasks/app/TASK-APP-001-hello"
cat > "$d/docs/tasks/app/TASK-APP-001-hello/spec.md" <<'EOF'
---
id: TASK-APP-001
title: "hello"
type: feature
class: product
status: ready_to_implement
module: app
priority: p3
created: 2026-07-27
---
# hello
EOF
(cd "$d" && git add docs/tasks && git commit -qm "feat(app): add hello task (TASK-APP-001)")

# Status page should exist (install migrate) or be regenerable
if [ ! -f "$d/docs/status/index.html" ]; then
  bash "$d/.cyberos/lib/status-page.sh" "$d" >/dev/null 2>&1 || true
fi
[ -f "$d/docs/status/index.html" ] && ok t06_status_page_present || fail t06_status_page_present "no index.html"

grep -q 'status-hub@3\|sv3-data\|status-feed' "$d/docs/status/index.html" \
  && ok t07_v3_markup || fail t07_v3_markup "not v3 page"

# file:// friendly: no remote runtime deps in the page shell
! grep -qiE 'cdn\.|unpkg\.|jsdelivr|https://.*\.js' "$d/docs/status/index.html" \
  && ok t08_no_network_script_tags || fail t08_no_network_script_tags "external scripts"

[ -f "$d/docs/status/data/status-feed.json" ] && ok t09_feed_json || fail t09_feed_json "no feed"

echo "offline_status_cert: pass=$PASS fail=$FAIL"
[ "$FAIL" -eq 0 ]
