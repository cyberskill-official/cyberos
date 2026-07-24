#!/usr/bin/env bash
# test_verdict_artifact.sh — TASK-IMP-143 AC2: content-addressed HITL verdict artifacts.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
BM="$repo/tools/install/docs-tools/backlog-mutate.mjs"
VA="$repo/tools/install/docs-tools/verdict-artifact.mjs"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok() { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }
export CYBEROS_NOW="2026-07-25T00:00:00Z"
unset CYBEROS_STORE

mk_repo() {
  mkdir -p "$1/docs/tasks/improvement/TASK-GUARD-001-truth-index"
  cat > "$1/docs/tasks/BACKLOG.md" <<EOF
# CyberOS task backlog (regenerated 2026-07-09)

Totals: 1 $2

## improvement  (1 $2)

- [$2] TASK-GUARD-001-truth-index - Truth precedes index (improvement)
EOF
  printf -- '---\nid: TASK-GUARD-001\nstatus: %s\n---\n# body\n' "$3" \
    > "$1/docs/tasks/improvement/TASK-GUARD-001-truth-index/spec.md"
}
EVID="$TMP/review-note.md"
printf 'Reviewed 2026-07-25: acceptance verdict recorded here.\n' > "$EVID"

echo "test_verdict_artifact.sh (TASK-IMP-143)"

t01_mint_on_gated_flip() {
  local d="$TMP/t01" rc art
  mk_repo "$d" reviewing ready_to_test
  node "$BM" --json flip TASK-GUARD-001 reviewing ready_to_test --root "$d" \
    --verdict-by "fixture-human" --verdict-evidence "$EVID" >"$TMP/out" 2>"$TMP/err"; rc=$?
  art="$(python3 -c "import json,sys; print(json.load(open(sys.argv[1])).get('verdict_artifact',''))" "$TMP/out")"
  [ "$rc" -eq 0 ] || { fail t01 "rc=$rc err=$(cat "$TMP/err")"; return; }
  [ -n "$art" ] && [ -f "$d/$art" ] || { fail t01 "artifact missing path=$art"; return; }
  python3 - <<PY "$d/$art" "$EVID" || { fail t01 "artifact self-check"; return; }
import json,hashlib,sys
art=json.load(open(sys.argv[1]))
ev=open(sys.argv[2],'rb').read()
assert art['schema']=='cyberos.verdict@1'
assert art['actor']=='fixture-human'
assert art['evidence_sha256']==hashlib.sha256(ev).hexdigest()
assert art['from']=='reviewing' and art['to']=='ready_to_test'
keys=["schema","actor","timestamp","task_id","from","to","evidence_path","evidence_sha256"]
canon=json.dumps({k:art[k] for k in keys}, separators=(',', ':'))+"\n"
assert art['artifact_sha256']==hashlib.sha256(canon.encode()).hexdigest()
print('ok')
PY
  ok t01_mint_on_gated_flip
}

t02_bad_artifact_refused() {
  local d="$TMP/t02" rc pre
  mk_repo "$d" testing done
  printf '{"schema":"cyberos.verdict@1","actor":"wrong","timestamp":"t","task_id":"TASK-GUARD-001","from":"testing","to":"done","evidence_path":"x","evidence_sha256":"00","artifact_sha256":"00"}\n' \
    > "$TMP/bad.json"
  pre=$(shasum -a 256 "$d/docs/tasks/BACKLOG.md" | awk '{print $1}')
  node "$BM" flip TASK-GUARD-001 testing done --root "$d" \
    --verdict-by "fixture-human" --verdict-evidence "$EVID" \
    --verdict-artifact "$TMP/bad.json" >"$TMP/out" 2>"$TMP/err"; rc=$?
  [ "$rc" -eq 8 ] && [ "$(shasum -a 256 "$d/docs/tasks/BACKLOG.md" | awk '{print $1}')" = "$pre" ] \
    && ok t02_bad_artifact_refused \
    || fail t02 "rc=$rc err=$(cat "$TMP/err")"
}

t03_non_gate_unchanged() {
  local d="$TMP/t03" rc
  mk_repo "$d" testing ready_to_implement
  # frontmatter must already equal target (IMP-120)
  printf -- '---\nid: TASK-GUARD-001\nstatus: ready_to_implement\n---\n# body\n' \
    > "$d/docs/tasks/improvement/TASK-GUARD-001-truth-index/spec.md"
  node "$BM" flip TASK-GUARD-001 testing ready_to_implement --root "$d" >"$TMP/out" 2>"$TMP/err"; rc=$?
  [ "$rc" -eq 0 ] && [ ! -d "$d/docs/tasks/_verdicts" ] && ok t03_non_gate_unchanged \
    || fail t03 "rc=$rc err=$(cat "$TMP/err") verdicts=$(ls "$d/docs/tasks/_verdicts" 2>/dev/null)"
}

t01_mint_on_gated_flip
t02_bad_artifact_refused
t03_non_gate_unchanged

echo "verdict_artifact: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]
