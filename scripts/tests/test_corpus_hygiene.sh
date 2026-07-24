#!/usr/bin/env bash
# test_corpus_hygiene.sh - TASK-IMP-139 (t01-t07 -> AC 1-7): corpus hygiene guards.
#
# Gate-1 (UNREVIEWED markers) and Gate-2 (12 reconcile verdicts) are OPERATOR-GATED —
# this suite asserts the mechanical halves that landed without those verdicts, and
# DEFERs (ok + loud line, never fail) the halves that wait on the operator. Clearing
# markers or flipping stuck-task statuses from this suite is forbidden.
#
# Census methodology (deviation recorded in implementation-evidence.md): FM-112-
# equivalent scan of top-level frontmatter lines only — a naive `grep -rl '# UNREVIEWED'`
# would corrupt five quote-only files (TASK-IMP-084/108/117 + TASK-IMP-139/140).
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../.." && pwd)"
TASKDIR="$repo/docs/tasks/improvement/TASK-IMP-139-corpus-hygiene-sweep"
LINT="$repo/tools/install/docs-tools/task-lint.mjs"
RUBRIC="$repo/modules/skill/task-audit/RUBRIC.md"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

echo "test_corpus_hygiene.sh (TASK-IMP-139)"

# Frontmatter-only marker scan (FM-112-equivalent): count non-draft specs whose
# top-level frontmatter has an OWN-LINE `# UNREVIEWED` comment (not title/quote substrings).
nondraft_marker_count() {
  python3 - "$repo/docs/tasks" <<'PY'
import re, sys
from pathlib import Path
root = Path(sys.argv[1])
n = 0
for spec in root.glob("*/TASK-*/spec.md"):
    text = spec.read_text(encoding="utf-8", errors="replace")
    m = re.match(r"\A---\n(.*?)\n---\n", text, re.S)
    if not m:
        continue
    fm = m.group(1)
    status_m = re.search(r"(?m)^status:\s*(\S+)", fm)
    status = status_m.group(1) if status_m else ""
    if status in ("", "draft"):
        continue
    if any(line.lstrip().startswith("# UNREVIEWED") for line in fm.splitlines()):
        n += 1
print(n)
PY
}

t01_fork_verdict_recorded() {                                          # AC 1
  local brief="$TASKDIR/assets/unreviewed-fork-brief.md" spec="$TASKDIR/spec.md"
  [ -f "$brief" ] || { fail t01 "missing assets/unreviewed-fork-brief.md"; return; }
  if grep -qE 'Gate 1 verdict|Branch clear' "$spec" \
     && grep -qE '2026-0[0-9]-[0-9]{2} operator Gate 1' "$spec"; then
    ok t01
  else
    fail t01 "Gate-1 UNREVIEWED fork verdict missing from source_decisions (see assets/unreviewed-fork-brief.md)"
  fi
}

t02_no_nondraft_markers() {                                            # AC 2
  local n; n="$(nondraft_marker_count)"
  if [ "$n" = "0" ]; then
    ok t02
  else
    fail t02 "$n non-draft specs still carry own-line # UNREVIEWED (FM-112-equivalent census)"
  fi
}

t03_module_case_conformant() {                                         # AC 3
  local bad
  bad="$(python3 - "$repo/docs/tasks" <<'PY'
import re, sys
from pathlib import Path
root = Path(sys.argv[1])
bad = []
for spec in sorted(root.glob("*/TASK-*/spec.md")):
    folder = spec.parent.parent.name
    if folder.startswith(("_", ".")):
        continue
    text = spec.read_text(encoding="utf-8", errors="replace")
    m = re.match(r"\A---\n(.*?)\n---\n", text, re.S)
    if not m:
        continue
    mm = re.search(r"(?m)^module:\s*(.+?)\s*$", m.group(1))
    if not mm:
        continue
    val = mm.group(1).strip().strip("\"'")
    if val != val.lower() or val != folder:
        bad.append(f"{spec.relative_to(root)}: module={val!r} folder={folder!r}")
print("\n".join(bad[:20]))
print(f"COUNT={len(bad)}")
PY
)"
  local count; count="$(echo "$bad" | sed -n 's/^COUNT=//p')"
  [ "$count" = "0" ] && ok t03 || fail t03 "$count module-case mismatches (sample): $(echo "$bad" | head -5)"
}

t04_lint_rule_fires() {                                                # AC 4
  grep -q '`FM-117`' "$RUBRIC" || { fail t04 "RUBRIC.md missing FM-117 documentation"; return; }
  mkdir -p "$TMP/docs/tasks/improvement/TASK-IMP-991-mixed-case" \
           "$TMP/docs/tasks/improvement/TASK-IMP-992-folder-mismatch" \
           "$TMP/docs/tasks/improvement/TASK-IMP-990-conformant"
  # minimal frontmatter fixtures (only module field under test; linter may emit other FMs)
  for id_status in "990:improvement" "991:IMPROVEMENT" "992:auth"; do
    id="${id_status%%:*}"; mod="${id_status#*:}"
    dir="$TMP/docs/tasks/improvement/TASK-IMP-${id}-x"
    [ "$id" = "990" ] && dir="$TMP/docs/tasks/improvement/TASK-IMP-990-conformant"
    [ "$id" = "991" ] && dir="$TMP/docs/tasks/improvement/TASK-IMP-991-mixed-case"
    [ "$id" = "992" ] && dir="$TMP/docs/tasks/improvement/TASK-IMP-992-folder-mismatch"
    mkdir -p "$dir"
    cat > "$dir/spec.md" <<EOF
---
id: TASK-IMP-${id}
title: fixture
author: @fixture
department: engineering
template: task@1
status: draft
priority: p3
created_at: 2026-07-23T00:00:00Z
ai_authorship: none
type: improvement
eu_ai_act_risk_class: not_ai
client_visible: false
module: ${mod}
---
# body
EOF
  done
  local out rc
  out="$(node "$LINT" "$TMP/docs/tasks/improvement/TASK-IMP-991-mixed-case/spec.md" 2>&1)"; rc=$?
  # Use <<< not echo|grep -q: under pipefail, grep -q SIGPIPEs echo and falsely fails the && chain.
  grep -q 'FM-117' <<<"$out" && [ "$rc" -ne 0 ] \
    || { fail t04 "mixed-case fixture did not fire FM-117 (rc=$rc): $out"; return; }
  out="$(node "$LINT" "$TMP/docs/tasks/improvement/TASK-IMP-992-folder-mismatch/spec.md" 2>&1)"; rc=$?
  grep -q 'FM-117' <<<"$out" && [ "$rc" -ne 0 ] \
    || { fail t04 "folder-mismatch fixture did not fire FM-117 (rc=$rc): $out"; return; }
  out="$(node "$LINT" "$TMP/docs/tasks/improvement/TASK-IMP-990-conformant/spec.md" 2>&1)"; rc=$?
  if grep -q 'FM-117' <<<"$out"; then
    fail t04 "conformant fixture wrongly fired FM-117: $out"; return
  fi
  ok t04
}

t05_triage_verdict_per_task() {                                        # AC 5
  local missing=0
  for id in MCP-003 MCP-005 MCP-006 MCP-007 MCP-008 \
            OBS-001 OBS-003 OBS-005 OBS-007 OBS-008 OBS-009 APP-001; do
    [ -f "$TASKDIR/assets/reconcile/TASK-${id}.md" ] || { echo "  missing dossier TASK-${id}"; missing=1; }
  done
  [ "$missing" -eq 0 ] || { fail t05 "reconcile dossiers incomplete"; return; }
  if ! grep -qE '2026-0[0-9]-[0-9]{2} operator Gate 2' "$TASKDIR/spec.md"; then
    fail t05 "Gate-2 dated verdict missing from source_decisions"; return
  fi
  # Spot-check applied statuses match the recorded tally (11 route_back + 1 resume).
  local rb=0
  for id in MCP-003 MCP-005 MCP-006 MCP-007 MCP-008 \
            OBS-001 OBS-003 OBS-005 OBS-007 OBS-008 OBS-009; do
    local st
    st="$(python3 - "$repo/docs/tasks" "$id" <<'PY'
import re, sys
from pathlib import Path
root, needle = Path(sys.argv[1]), sys.argv[2]
hits = list(root.glob(f"*/TASK-{needle}*/spec.md"))
if len(hits) != 1:
    print(f"AMBIGUOUS:{len(hits)}"); raise SystemExit(0)
m = re.search(r"(?m)^status:\s*(\S+)", hits[0].read_text(encoding="utf-8", errors="replace"))
print(m.group(1) if m else "MISSING")
PY
)"
    [ "$st" = "ready_to_implement" ] || { fail t05 "TASK-$id expected ready_to_implement got $st"; return; }
    rb=$((rb+1))
  done
  local app
  app="$(python3 - "$repo/docs/tasks/app/TASK-APP-001-desktop-cyberos-operations/spec.md" <<'PY'
import re, sys
from pathlib import Path
m = re.search(r"(?m)^status:\s*(\S+)", Path(sys.argv[1]).read_text(encoding="utf-8", errors="replace"))
print(m.group(1) if m else "MISSING")
PY
)"
  [ "$app" = "implementing" ] || { fail t05 "TASK-APP-001 resume expected implementing got $app"; return; }
  [ "$rb" -eq 11 ] || { fail t05 "expected 11 route_backs counted $rb"; return; }
  ok t05
}

t06_registered_and_idempotent() {                                      # AC 6
  # Self-discovery: this file matches run_all's test_*.sh glob under scripts/tests/.
  [ -f "$here/test_corpus_hygiene.sh" ] || { fail t06 "suite file missing"; return; }
  # Regenerator idempotence: two consecutive --backlog runs produce byte-identical BACKLOG.md
  # (post module-case normalization the regenerator groups by folder — should be stable).
  local a="$TMP/bl-a.md" b="$TMP/bl-b.md"
  python3 "$repo/scripts/migrate_improvement_to_task.py" --backlog >"$a" 2>"$TMP/bl-err" \
    || { fail t06 "regen 1 failed: $(cat "$TMP/bl-err")"; return; }
  python3 "$repo/scripts/migrate_improvement_to_task.py" --backlog >"$b" 2>"$TMP/bl-err2" \
    || { fail t06 "regen 2 failed: $(cat "$TMP/bl-err2")"; return; }
  cmp -s "$a" "$b" && ok t06 || fail t06 "BACKLOG regenerator not byte-identical across consecutive runs"
}

t07_changelog_records_hygiene() {                                      # AC 7
  local top
  # Scan every versioned ## […] section — top entry moves with each cut (same class as CUO doctrine pin).
  top="$(awk '/^## \[/{p=1} p' "$repo/CHANGELOG.md")"
  local all=1
  for want in '251' 'FM-117' 'module' 'route_back' 'resume'; do
    echo "$top" | grep -q "$want" || { fail t07 "CHANGELOG versioned entry lacks '$want'"; all=0; }
  done
  if ! echo "$top" | grep -qiE 'UNREVIEWED|Branch clear|corpus hygiene|IMP-139'; then
    fail t07 "CHANGELOG versioned entry does not name corpus hygiene / Branch clear / IMP-139"; all=0
  fi
  [ "$all" -eq 1 ] && ok t07
}

t01_fork_verdict_recorded
t02_no_nondraft_markers
t03_module_case_conformant
t04_lint_rule_fires
t05_triage_verdict_per_task
t06_registered_and_idempotent
t07_changelog_records_hygiene

echo "----"; echo "pass=$PASS fail=$FAIL"; [ "$FAIL" -eq 0 ]
