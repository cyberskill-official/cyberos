#!/usr/bin/env bash
# test_regen_backlog.sh - TASK-IMP-091 §2 suite (t01-t03 -> AC 1-3; AC 4 is the runner's
# own glob discovery, evidenced in the gate log).
#
# regen_backlog is the byte authority every insert contract cites, and it could not
# regenerate a truthful index: an ACTIVE-status filter dropped every terminal row, so a
# regen deleted done rows and left Totals short (TASK-IMP-086 gate-log E1 recorded the
# trial: zero rows for fourteen done tasks, Totals 155 vs 158). These asserts pin the
# repair: every status emits a row, Totals comes from a frontmatter tally, and today's
# corpus regenerates byte-identical to the committed section.
#
# The script resolves ROOT from its own path (parents[1]), so every scenario copies it into
# a scratch tree and runs it there - the live docs/tasks/BACKLOG.md is never a write target.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../.." && pwd)"
S="$repo/scripts/migrate_improvement_to_task.py"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

python3 -c "import yaml" 2>/dev/null || { echo "test_regen_backlog: pyyaml absent - suite cannot run"; exit 2; }

# scratch <dir>: a runnable copy of the script rooted at <dir> (scripts/ + docs/tasks/)
scratch() { mkdir -p "$1/scripts" "$1/docs/tasks"; cp "$S" "$1/scripts/"; }
# section <file> <name>: the section's lines, header through the blank line before the next H2
section() { awk -v s="## $2" '$0==s{f=1} f&&/^## /&&$0!=s{exit} f{print}' "$1"; }

# ---------------------------------------------------------------- t01 live-corpus parity
# AC 1 (§1 #1.1, #1.3): regen over today's real corpus reproduces the committed improvement
# section byte-for-byte - rows for every status, in the committed grammar and order.
t01_live_corpus_parity() {
  local d="$TMP/t01"; scratch "$d"
  cp -R "$repo/docs/tasks/." "$d/docs/tasks/" 2>/dev/null
  ( cd "$d" && python3 scripts/migrate_improvement_to_task.py --backlog ) >"$d/out.txt" 2>&1 \
    || { fail t01_live_corpus_parity "regen exited non-zero: $(tail -1 "$d/out.txt")"; return; }
  git -C "$repo" show HEAD:docs/tasks/BACKLOG.md > "$d/committed.md" 2>/dev/null \
    || { fail t01_live_corpus_parity "cannot read committed BACKLOG.md"; return; }
  section "$d/docs/tasks/BACKLOG.md" improvement > "$d/regen.section"
  section "$d/committed.md"          improvement > "$d/committed.section"
  if cmp -s "$d/regen.section" "$d/committed.section"; then ok t01_live_corpus_parity
  else fail t01_live_corpus_parity "improvement section differs from the committed object: $(diff "$d/committed.section" "$d/regen.section" | head -4 | tr '\n' ' ')"; fi
}

# ---------------------------------------------------------------- t02 totals are a tally
# AC 2 (§1 #1.2): the Totals line equals a per-status tally computed independently here.
t02_totals_true() {
  local d="$TMP/t02"; scratch "$d"
  cp -R "$repo/docs/tasks/." "$d/docs/tasks/" 2>/dev/null
  ( cd "$d" && python3 scripts/migrate_improvement_to_task.py --backlog ) >/dev/null 2>&1 \
    || { fail t02_totals_true "regen exited non-zero"; return; }
  local got want
  got="$(grep -m1 '^Totals: ' "$d/docs/tasks/BACKLOG.md" | sed 's/^Totals: //')"
  want="$(python3 - "$d/docs/tasks" <<'TALLY'
import sys, yaml
from pathlib import Path
# Independent tally: the assert must not inherit the script's own reader. Boundary rule is
# the corpus convention - first line '---', block ends at the next line that is exactly
# '---' (a body line containing '---' is not a fence).
ORDER = ["draft","ready_to_implement","implementing","ready_to_review","reviewing",
         "ready_to_test","testing","done","on_hold","closed"]
tally = {}
for f in sorted(Path(sys.argv[1]).glob("*/TASK-*/spec.md")):
    ls = f.read_text().splitlines()
    if not ls or ls[0].strip() != "---":
        sys.exit("tally: %s has no frontmatter fence" % f)
    end = next((i for i, l in enumerate(ls[1:], 1) if l.strip() == "---"), None)
    if end is None:
        sys.exit("tally: %s has an unterminated frontmatter fence" % f)
    fm = yaml.safe_load("\n".join(ls[1:end])) or {}
    st = str(fm.get("status", "")).strip()
    tally[st] = tally.get(st, 0) + 1
order = ORDER + sorted(s for s in tally if s not in ORDER and s)
print(", ".join("%d %s" % (tally[s], s) for s in order if tally.get(s)))
TALLY
)"
  if [ "$got" = "$want" ]; then ok t02_totals_true
  else fail t02_totals_true "Totals '$got' != frontmatter tally '$want'"; fi
}

# ------------------------------------------------- t03 every status emits; bad fm halts
# AC 3 (§1 #1.4 + §3): a fixture carrying one task per lifecycle status yields one row each
# (the filter cannot silently return), and unparseable frontmatter halts before any write.
t03_every_status_emitted() {
  local d="$TMP/t03"; scratch "$d"
  local statuses="draft ready_to_implement implementing ready_to_review reviewing ready_to_test testing done on_hold closed cannot_reproduce duplicate"
  local n=0
  for st in $statuses; do
    n=$((n+1)); local id; id="$(printf 'TASK-FIX-%03d' "$n")"
    mkdir -p "$d/docs/tasks/fix/$id-scenario"
    printf -- '---\nid: %s\ntitle: status %s fixture\nstatus: %s\nclass: improvement\n---\n' "$id" "$st" "$st" \
      > "$d/docs/tasks/fix/$id-scenario/spec.md"
  done
  ( cd "$d" && python3 scripts/migrate_improvement_to_task.py --backlog ) >/dev/null 2>&1 \
    || { fail t03_every_status_emitted "regen exited non-zero on the per-status fixture"; return; }
  local missing=""
  for st in $statuses; do
    grep -qE "^- \[$st\] TASK-FIX-[0-9]{3}-scenario - status $st fixture \(improvement\)$" "$d/docs/tasks/BACKLOG.md" \
      || missing="$missing $st"
  done
  local rows; rows="$(grep -c '^- \[' "$d/docs/tasks/BACKLOG.md")"
  [ -n "$missing" ] && { fail t03_every_status_emitted "no row emitted for:$missing"; return; }
  [ "$rows" -eq "$n" ] || { fail t03_every_status_emitted "row count $rows != folder count $n"; return; }

  # the halt half: one unparseable spec.md, and BACKLOG.md must stay exactly as it was
  local before; before="$(sha256sum < "$d/docs/tasks/BACKLOG.md")"
  mkdir -p "$d/docs/tasks/fix/TASK-FIX-999-broken"
  printf -- '---\nid: TASK-FIX-999\ntitle: [unclosed\n  bad: : :\n---\n' > "$d/docs/tasks/fix/TASK-FIX-999-broken/spec.md"
  local rc=0
  ( cd "$d" && python3 scripts/migrate_improvement_to_task.py --backlog ) >"$d/halt.txt" 2>&1 || rc=$?
  local after; after="$(sha256sum < "$d/docs/tasks/BACKLOG.md")"
  if [ "$rc" -eq 0 ]; then fail t03_every_status_emitted "unparseable frontmatter did not halt the regen"
  elif ! grep -q "TASK-FIX-999-broken/spec.md" "$d/halt.txt"; then fail t03_every_status_emitted "halt did not name the offending file"
  elif [ "$before" != "$after" ]; then fail t03_every_status_emitted "BACKLOG.md was written despite the halt"
  else ok t03_every_status_emitted; fi
}

echo "regen-backlog suite (TASK-IMP-091):"
t01_live_corpus_parity
t02_totals_true
t03_every_status_emitted
echo "regen-backlog: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] || exit 1
