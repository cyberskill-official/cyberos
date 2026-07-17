#!/usr/bin/env bash
# test_workflow_improver.sh - TASK-IMP-110 §2 suite (t01-t06 -> AC 1-6; AC 7 is a corpus
# state change, verified by the FM-113 lint pass on TASK-IMP-028 + the recorded gate log).
#
# t07 is BEYOND the declared AC set and is here on purpose: build.sh vendors by an explicit
# copy list, and a rule correct in modules/ and absent from dist/ is correct nowhere. The
# declared build.sh change is the thing t07 gates.
#
# Every arm builds a scratch git repo shaped like a real corpus - done tasks with gate logs,
# a BACKLOG, a modules/skill tree - and bends exactly one thing. The guardrail arms (t04, t05)
# assert the FILESYSTEM, never a log line: "it says it did not write" is not "it did not write".
# Each arm also asserts it is NOT VACUOUS: a run that found nothing would satisfy "left
# modules/** alone" and "wrote no file" trivially, and prove neither.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
WI="$repo/tools/install/docs-tools/workflow-improve.mjs"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }
# portable: sha256sum on linux CI, shasum -a 256 on operator macs (same two-space output)
_sha() { if command -v sha256sum >/dev/null 2>&1; then sha256sum "$@"; else shasum -a 256 "$@"; fi; }
# py <script> [args...] - script from -c, JSON from stdin. NEVER a heredoc: `python3 - <<PY`
# plus a `<<<` herestring silently drops the heredoc and feeds the JSON in as the program.
py() { local s="$1"; shift; python3 -c "$s" "$@" 2>"$TMP/perr"; }
perr() { tail -2 "$TMP/perr" 2>/dev/null | tr '\n' ' '; }

SKILLS="coverage-gate-author task-audit edge-case-matrix-author implementation-plan-author code-review-author"

# mkrepo <dir> - a corpus-shaped scratch repo with a real modules/skill tree
mkrepo() {
  local d="$1" s
  mkdir -p "$d/docs/tasks/demo" "$d/modules/skill"
  ( cd "$d" && git init -qb main . && git config user.email t@t && git config user.name t )
  for s in $SKILLS; do
    mkdir -p "$d/modules/skill/$s"
    printf -- '---\nname: %s\n---\n# %s\n\nDoctrine. Not this tool to edit.\n' "$s" "$s" > "$d/modules/skill/$s/SKILL.md"
  done
  printf '# BACKLOG\n\n| id | status |\n|---|---|\n' > "$d/docs/tasks/BACKLOG.md"
}

# task <dir> <NNN> <gate-log-body> - one completed task carrying its gate log.
# `shipped` mirrors the real corpus (a YYYY-MM-DD completion date) and increases with the task
# number, so the window's recency order is exercised, not bypassed: TASK-DEMO-009 is the most
# recently completed and is therefore read first.
task() {
  local d="$1" n="$2" body="$3"
  local t="$d/docs/tasks/demo/TASK-DEMO-$n-thing"
  mkdir -p "$t"
  cat > "$t/spec.md" <<SPEC
---
id: TASK-DEMO-$n
title: demo thing $n
template: task@1
type: improvement
status: done
priority: p2
created_at: 2026-07-${n#0}T08:00:00Z
shipped: 2026-07-${n#0}
---

# TASK-DEMO-$n: demo thing
SPEC
  printf '# TASK-DEMO-%s gate-log evidence\n\n%s\n' "$n" "$body" > "$t/gate-log.md"
}
commit() { ( cd "$1" && git add -A && git commit -qm "fixture" ); }
hash_modules() { ( cd "$1" && find modules -type f | LC_ALL=C sort | while IFS= read -r f; do _sha "$f"; done | _sha ); }

# the recurring shape: a recorded route-back reason naming a code AND an existing skill
R_TRACE='- reason: "trace-004: coverage-gate-author did not list the clause test"'

# ── t01 (AC 1, #1.1/#1.2/#1.3): a 3-occurrence pattern proposes, citing >=2 rows BY ID ──
t01_pattern_proposes_with_evidence() {
  local d="$TMP/t01"; mkrepo "$d"
  task "$d" 001 "$R_TRACE"; task "$d" 002 "$R_TRACE"; task "$d" 003 "$R_TRACE"
  commit "$d"
  local out; out="$(node "$WI" --repo "$d" --json 2>"$TMP/err")" \
    || { fail t01_pattern_proposes_with_evidence "tool exited non-zero: $(tail -1 "$TMP/err")"; return; }
  py '
import json, sys
r = json.load(sys.stdin)
assert r["window"]["completed_tasks_read"] == 3, "fixture: read {} tasks".format(r["window"]["completed_tasks_read"])
p = r["proposals"]
assert len(p) == 1, "expected exactly 1 proposal, got {}".format(len(p))
p = p[0]
assert p["artefact"] == "skill-amendment@1", p["artefact"]
assert p["target_skill"] == "coverage-gate-author", "target_skill={}".format(p["target_skill"])
assert p["signal"] == "trace-004", "signal={}".format(p["signal"])
ev = p["evidence"]
assert len(ev) >= 2, "#1.3: proposal cites {} evidence row(s), needs >=2".format(len(ev))
assert len(set(e["id"] for e in ev)) == len(ev), "evidence ids are not unique"
assert all(e["id"].startswith("EV-") for e in ev), "evidence rows are not cited by id"
assert len(set(e["source"] for e in ev)) >= 2, "evidence rows are not independent (same source file)"
want = "trace-004: coverage-gate-author did not list the clause test"
assert all(e["quote"] == want for e in ev), "quote not verbatim: {}".format([e["quote"] for e in ev])
' <<<"$out" || { fail t01_pattern_proposes_with_evidence "$(perr)"; return; }
  # the ids must survive into what a HUMAN reads, not only into --json
  local n; n="$(node "$WI" --repo "$d" 2>/dev/null | grep -o 'EV-[0-9a-f]\{8\}' | sort -u | wc -l | tr -d ' ')"
  [ "$n" -ge 2 ] || { fail t01_pattern_proposes_with_evidence "the rendered proposal cites $n evidence id(s), needs >=2"; return; }
  ok t01_pattern_proposes_with_evidence
}

# ── t02 (AC 2, #1.3): one occurrence is an anecdote, not a pattern ──────────
t02_anecdote_rejected() {
  local d="$TMP/t02"; mkrepo "$d"
  task "$d" 001 "$R_TRACE"                                   # the ONLY source carrying it
  task "$d" 002 '- reason: "unrelated: task-audit halted on a needs_human verdict"'
  commit "$d"
  local out; out="$(node "$WI" --repo "$d" --json 2>/dev/null)"
  py '
import json, sys
r = json.load(sys.stdin)
assert r["proposals"] == [], "a single-occurrence pattern produced {} proposal(s) - an anecdote was promoted".format(len(r["proposals"]))
assert r["verdict"] == "no amendment proposed", r["verdict"]
# the cluster was SEEN and rejected on the floor, not simply never read: a reader that never
# ran would also report zero proposals, and that is a different (passing-looking) bug
assert r["clusters"] >= 2, "the evidence was never clustered: clusters={}".format(r["clusters"])
assert r["evidence"]["attributed"] == 2, "attributed={}".format(r["evidence"]["attributed"])
assert r["qualifying"] == 0, "qualifying={}".format(r["qualifying"])
' <<<"$out" || { fail t02_anecdote_rejected "$(perr)"; return; }
  ok t02_anecdote_rejected
}

# ── t03 (AC 3, #1.2): 8 patterns -> exactly 3, highest-evidence first ───────
t03_cap_enforced() {
  local d="$TMP/t03"; mkrepo "$d"
  # sig-0j gets j+1 independent sources: sig-01 in 2, sig-02 in 3, ... sig-08 in 9.
  # Task i carries sig-0j iff j >= 9-i, so task 009 (read FIRST - the window is task-number
  # descending) carries all eight in ASCENDING order.
  #
  # That inversion is the point. The ranked answer (sig-08, sig-07, sig-06) is the exact
  # REVERSE of the order the clusters are discovered in. An earlier fixture planted them the
  # other way round and this arm passed with the ranking DELETED - it was asserting an
  # accident of Map insertion order, not the rank. A test that passes when its clause is
  # removed is decoration.
  local i j body
  for i in $(seq 1 9); do
    body=""
    for j in $(seq 1 8); do
      if [ "$j" -ge $((9 - i)) ]; then
        body="${body}- reason: \"sig-0${j}: coverage-gate-author padded the report\"
"
      fi
    done
    task "$d" "$(printf '%03d' "$i")" "$body"
  done
  commit "$d"
  local out; out="$(node "$WI" --repo "$d" --json 2>/dev/null)"
  py '
import json, sys
r = json.load(sys.stdin)
assert r["qualifying"] == 8, "fixture is wrong: {} qualifying clusters, expected 8".format(r["qualifying"])
p = r["proposals"]
assert len(p) == 3, "#1.2 cap: {} proposals emitted, the cap is 3".format(len(p))
sigs = [x["signal"] for x in p]
assert sigs == ["sig-08", "sig-07", "sig-06"], "not highest-evidence first: {}".format(sigs)
srcs = [x["independent_sources"] for x in p]
assert srcs == [9, 8, 7], "not ranked by independent evidence: {}".format(srcs)
' <<<"$out" || { fail t03_cap_enforced "$(perr)"; return; }
  ok t03_cap_enforced
}

# ── t04 (AC 4, #1.4): a run leaves modules/** BYTE-IDENTICAL ────────────────
# The clause is "MUST NOT write". So this asserts the bytes, not a claim about the bytes.
t04_never_writes_skills() {
  local d="$TMP/t04"; mkrepo "$d"
  task "$d" 001 "$R_TRACE"; task "$d" 002 "$R_TRACE"; task "$d" 003 "$R_TRACE"
  commit "$d"
  # _sha prints the PATH beside the digest, so this fingerprint catches a file CREATED or
  # DELETED under modules/, not only one edited in place.
  local before after
  before="$(hash_modules "$d")"
  local out; out="$(node "$WI" --repo "$d" --out docs/tasks/_audits/proposals.md --json 2>/dev/null)"
  py '
import json, sys
r = json.load(sys.stdin)
assert len(r["proposals"]) == 1, "the arm is vacuous: the run proposed {}".format(len(r["proposals"]))
' <<<"$out" || { fail t04_never_writes_skills "$(perr)"; return; }
  [ -s "$d/docs/tasks/_audits/proposals.md" ] \
    || { fail t04_never_writes_skills "the arm is vacuous: --out wrote nothing, so 'did not write modules/**' proves nothing"; return; }
  after="$(hash_modules "$d")"
  [ "$before" = "$after" ] || { fail t04_never_writes_skills "the tool MUTATED modules/** (§1.4)"; return; }

  # the direct guard: an --out aimed INTO the doctrine trees is refused, and writes nothing
  local rc o bad
  for bad in modules/skill/coverage-gate-author/SKILL.md modules/notes.md modules/skill/rubrics/x.md; do
    o="$(node "$WI" --repo "$d" --out "$bad" 2>&1)"; rc=$?
    { [ "$rc" -eq 2 ] && grep -q "REFUSED" <<<"$o"; } \
      || { fail t04_never_writes_skills "--out $bad was not refused (rc=$rc): $(head -1 <<<"$o")"; return; }
  done
  [ "$before" = "$(hash_modules "$d")" ] || { fail t04_never_writes_skills "a refused --out still touched modules/**"; return; }
  o="$(node "$WI" --repo "$d" --out ../escaped.md 2>&1)"; rc=$?
  { [ "$rc" -eq 2 ] && grep -q "outside the repo root" <<<"$o"; } \
    || { fail t04_never_writes_skills "--out ../escaped.md was not refused (rc=$rc)"; return; }
  if [ -e "$(dirname "$d")/escaped.md" ]; then fail t04_never_writes_skills "--out escaped the root"; return; fi

  # §3: gate logs are UNTRUSTED INPUT. A crafted reason must be quoted verbatim and never run.
  local e="$TMP/t04b"; mkrepo "$e"
  local evil='trace-004: coverage-gate-author $(touch '"$TMP"'/PWNED) `touch '"$TMP"'/PWNED2`'
  task "$e" 001 "- reason: \"$evil\""
  task "$e" 002 "- reason: \"$evil\""
  commit "$e"
  out="$(node "$WI" --repo "$e" --json 2>/dev/null)"
  if [ -e "$TMP/PWNED" ] || [ -e "$TMP/PWNED2" ]; then fail t04_never_writes_skills "evidence text was EXECUTED"; return; fi
  py '
import json, sys
r = json.load(sys.stdin)
assert len(r["proposals"]) == 1, "the arm is vacuous: {}".format(r["proposals"])
for e in r["proposals"][0]["evidence"]:
    assert e["quote"] == sys.argv[1], "quote not verbatim:\n  got  {!r}\n  want {!r}".format(e["quote"], sys.argv[1])
' "$evil" <<<"$out" || { fail t04_never_writes_skills "$(perr)"; return; }

  # §3 / fail-closed: an evidence file present on disk but NOT tracked at HEAD is REFUSED and
  # named (exit 3), never silently skipped. A guard that skips itself when its input looks
  # wrong is not a guard - it is a passing-looking hole.
  ( cd "$e" && git rm -q --cached docs/tasks/demo/TASK-DEMO-002-thing/gate-log.md && git commit -qm "untrack a gate log" )
  o="$(node "$WI" --repo "$e" 2>&1)"; rc=$?
  { [ "$rc" -eq 3 ] && grep -q "REFUSED.*not tracked at HEAD" <<<"$o"; } \
    || { fail t04_never_writes_skills "an untracked evidence file was not refused (rc=$rc, want 3): $(head -1 <<<"$o")"; return; }
  ok t04_never_writes_skills
}

# ── t05 (AC 5, #1.6): a clean window reports no proposal and WRITES NOTHING ─
t05_clean_window_silent() {
  local d="$TMP/t05"; mkrepo "$d"
  # real gate logs on real completed tasks - just no recorded reason recurring anywhere
  task "$d" 001 'E1 - gating suite ran green, verbatim: `pass=4 fail=0`.'
  task "$d" 002 'E1 - vendor line verified; the payload copy is byte-identical to source.'
  task "$d" 003 'E1 - coverage gate: every clause traced to a passing test.'
  commit "$d"
  local out rc
  out="$(node "$WI" --repo "$d" --out docs/tasks/_audits/none.md 2>&1)"; rc=$?
  [ "$rc" -eq 0 ] || { fail t05_clean_window_silent "clean window exited $rc: $(head -2 <<<"$out")"; return; }
  grep -q "no amendment proposed" <<<"$out" || { fail t05_clean_window_silent "clean window did not report 'no amendment proposed'"; return; }
  # THE assertion: it emitted NOTHING. Not an empty report, not a placeholder, not a padded third.
  if [ -e "$d/docs/tasks/_audits/none.md" ]; then
    fail t05_clean_window_silent "a clean window WROTE --out (first line: $(head -1 "$d/docs/tasks/_audits/none.md")) - it must emit nothing (§1.6)"; return
  fi
  local j; j="$(node "$WI" --repo "$d" --json 2>/dev/null)"
  py '
import json, sys
r = json.load(sys.stdin)
assert r["proposals"] == [], "a clean window padded to {} proposal(s) (§1.6)".format(len(r["proposals"]))
# not vacuous: it really did read the window and really did find it clean
assert r["window"]["completed_tasks_read"] == 3, "the arm is vacuous: it read {} tasks".format(r["window"]["completed_tasks_read"])
assert r["verdict"] == "no amendment proposed", r["verdict"]
' <<<"$j" || { fail t05_clean_window_silent "$(perr)"; return; }
  ok t05_clean_window_silent
}

# ── t06 (AC 6, #1.5): proposals land at draft, and are never self-audited ───
t06_proposals_land_draft() {
  local d="$TMP/t06"; mkrepo "$d"
  task "$d" 001 "$R_TRACE"; task "$d" 002 "$R_TRACE"
  commit "$d"
  local out; out="$(node "$WI" --repo "$d" --out docs/tasks/_audits/proposals.md --json 2>/dev/null)"
  py '
import json, sys
r = json.load(sys.stdin)
assert r["proposals"], "the arm is vacuous: nothing was proposed"
for p in r["proposals"]:
    assert p["status"] == "draft", "#1.5: proposal landed at status={!r}, must be draft".format(p["status"])
' <<<"$out" || { fail t06_proposals_land_draft "$(perr)"; return; }
  local f="$d/docs/tasks/_audits/proposals.md"
  [ -s "$f" ] || { fail t06_proposals_land_draft "--out wrote nothing"; return; }
  grep -q "status: draft" "$f" || { fail t06_proposals_land_draft "the emitted proposal does not carry 'status: draft'"; return; }
  if grep -q "ready_to_implement" "$f"; then fail t06_proposals_land_draft "the tool self-audited a proposal to ready_to_implement (§1.5)"; return; fi
  # #1.5's other half: it lands proposals, it does not land AUDITS
  if [ -n "$(find "$d/docs/tasks" -name 'audit.md' | head -1)" ]; then
    fail t06_proposals_land_draft "the tool wrote an audit for its own proposal (§1.5)"; return
  fi
  ok t06_proposals_land_draft
}

# ── t07 (beyond the AC set): the declared build.sh vendor lines actually vendor ──
t07_payload_vendored() {
  bash "$repo/tools/install/build.sh" "$TMP/payload" >"$TMP/build.log" 2>&1 \
    || { fail t07_payload_vendored "build.sh failed: $(tail -3 "$TMP/build.log" | tr '\n' ' ')"; return; }
  local p="$TMP/payload/docs-tools/workflow-improve.mjs" t
  [ -s "$p" ] || { fail t07_payload_vendored "workflow-improve.mjs not vendored into the payload"; return; }
  cmp -s "$p" "$WI" || { fail t07_payload_vendored "the payload copy differs from the source"; return; }
  # the skill body ships in BOTH trees, or the vendored /improve command names a skill nobody has
  for t in cuo/skills plugin/skills; do
    [ -s "$TMP/payload/$t/workflow-improver/SKILL.md" ] \
      || { fail t07_payload_vendored "workflow-improver missing from payload/$t"; return; }
  done
  [ -s "$TMP/payload/plugin/commands/improve.md" ] || { fail t07_payload_vendored "improve.md not vendored"; return; }
  # and the vendored copy RUNS (a file that ships and cannot execute is not vendored)
  local d="$TMP/t07"; mkrepo "$d"
  task "$d" 001 "$R_TRACE"; task "$d" 002 "$R_TRACE"
  commit "$d"
  node "$p" --repo "$d" --json 2>/dev/null | grep -q '"artefact": "improvement-window@1"' \
    || { fail t07_payload_vendored "the payload copy does not run"; return; }
  ok t07_payload_vendored
}

echo "workflow-improver suite (TASK-IMP-110):"
t01_pattern_proposes_with_evidence
t02_anecdote_rejected
t03_cap_enforced
t04_never_writes_skills
t05_clean_window_silent
t06_proposals_land_draft
t07_payload_vendored
echo "test_workflow_improver: pass=$PASS fail=$FAIL"
[ "$FAIL" -eq 0 ] || exit 1
