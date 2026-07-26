#!/usr/bin/env bash
# test_commit_links_ledger.sh - TASK-DOCS-012: ledger validation via task-lint.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
LINT="$repo/tools/install/docs-tools/task-lint.mjs"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

mk() {
  d="$1"
  mkdir -p "$d/docs/tasks/aa/TASK-AA-001-x" "$d/docs/tasks/_state"
  printf -- '---\nid: TASK-AA-001\ntitle: A\ntemplate: task@1\nmodule: aa\npriority: MUST\nstatus: draft\ntype: feature\nclass: product\ncreated_at: 2026-07-01T00:00:00+07:00\ndepartment: engineering\nauthor: "@t"\nowner: t\ncreated: 2026-07-01\nshipped: null\neu_ai_act_risk_class: not_ai\nai_authorship: human\nclient_visible: false\nlanguage: text\nservice: x\nnew_files: []\nmodified_files: []\neffort_hours: 1\nverify: T\n---\n\n# TASK-AA-001: A\n\n## Summary\n\nx\n\n## Problem\n\nx\n\n## Proposed Solution\n\nx\n\n## Alternatives Considered\n\n- none\n\n## Success Metrics\n\n- Primary: x\n- Guardrail: x\n\n## Scope\n\nIn: x\nOut: y\n\n## Dependencies\n\nUpstream: none.\nDownstream: none.\n\n## 1. Description\n\n- 1.1 This task MUST do the thing.\n\n## Acceptance Criteria\n\n- [ ] AC 1 (traces_to: #1.1) — done - test: `true`\n\n## Test plan\n\n1. true\n\n## AI Authorship Disclosure\n\n- **Tools used:** none\n- **Scope:** human\n- **Human review:** yes\n' > "$d/docs/tasks/aa/TASK-AA-001-x/spec.md"
}

t01_unknown_task_fails() {
  mk "$TMP/a"
  printf 'deadbeef: [TASK-NOPE-001]\n' > "$TMP/a/docs/tasks/_state/commit-links.yaml"
  out="$(node "$LINT" "$TMP/a/docs/tasks" 2>&1)"; rc=$?
  [ "$rc" -ne 0 ] && grep -q 'LEDGER-001' <<<"$out" && grep -q 'TASK-NOPE-001' <<<"$out" \
    && ok t01 || fail t01 "unknown task did not fail lint (rc=$rc)"
}

t02_known_task_ok() {
  mk "$TMP/b"
  printf 'deadbeef: [TASK-AA-001]\n' > "$TMP/b/docs/tasks/_state/commit-links.yaml"
  out="$(node "$LINT" --json "$TMP/b/docs/tasks" 2>&1)"; rc=$?
  ! grep -q 'LEDGER-001' <<<"$out" \
    && ok t02 || fail t02 "known task still flagged (rc=$rc)"
}

t01_unknown_task_fails; t02_known_task_ok
echo "----"; echo "pass=$PASS fail=$FAIL"; [ "$FAIL" -eq 0 ]
