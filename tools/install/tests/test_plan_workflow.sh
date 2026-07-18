#!/usr/bin/env bash
# test_plan_workflow.sh - TASK-IMP-111 §2 suite (t01-t06 -> AC 1-6; AC 7 is a HITL halt, verified
# by the recorded gate-log transcript, NOT suite-asserted - a halt cannot be exercised without
# simulating the human, same rationale as the existing gate arms in test_workflow_improver.sh).
#
# t07 is BEYOND the declared AC set and is here on purpose: build.sh vendors by an explicit copy
# list, and a skill/rubric correct in modules/ and absent from dist/ is correct nowhere. The
# declared build.sh change is the thing t07 gates.
#
# The plan workflow is PROSE (LLM-executed markdown), so the mechanically-testable surface is:
#   - the deterministic mode-detect predicate (§2 of plan-author) applied to real fixtures,
#   - the additive scope change to repo-context-map-author (the task-scope OUTPUT byte-identical),
#   - the three completeness rules the rubric REDS on (option / decision / out list),
#   - the never-writes-tasks invariant asserted as BYTES on a fixture corpus,
#   - the create-tasks-consumable shape of the proposed task set, parsed and checked,
#   - the BRAIN chain, driven end-to-end through the real (vendored) memory-append appender.
# Each arm asserts it is NOT VACUOUS.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
PA="$repo/modules/skill/plan-author/SKILL.md"
PU="$repo/modules/skill/plan-audit/SKILL.md"
PR="$repo/modules/skill/rubrics/plan_rubric.md"
CMD="$repo/tools/install/plugin/commands/plan.md"
RCM="$repo/modules/skill/repo-context-map-author/SKILL.md"
MA="$repo/tools/install/docs-tools/memory-append.mjs"
CREATE="$repo/tools/install/plugin/commands/create-tasks.md"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }
only="$*"
want() { [ -z "$only" ] && return 0; case " $only " in *" $1 "*) return 0;; *) return 1;; esac; }
_sha() { if command -v sha256sum >/dev/null 2>&1; then sha256sum "$@"; else shasum -a 256 "$@"; fi; }

# AC 2 durable baseline: sha256 of repo-context-map-author's "## 2. Output schema" block, captured
# from HEAD BEFORE TASK-IMP-111 (see the task's AC2 proof). If the task-scope output contract ever
# drifts, this goes red - that is the whole point of "byte-identical to today's".
RCM_S2_GOLDEN="1d55bdf37c6e488509ca17f2c67b75fd8930e5143253cb4238dfb6edd101a876"
rcm_s2_block() { awk '/^## 2\. Output schema/{f=1} /^## 3\. Quality gates/{f=0} f' "$1"; }

# ── the mode-detect ORACLE - a faithful reimplementation of plan-author §2's predicate, so the
# fixtures below are graded against the doctrine the skill states, not an arbitrary rule ──────────
detect_mode() {
  local d="$1" has_cyberos=0 has_head=0 has_tasks=0 has_src=0
  [ -d "$d/.cyberos" ] && has_cyberos=1
  ( cd "$d" && git rev-parse --verify HEAD >/dev/null 2>&1 ) && has_head=1
  [ -d "$d/docs/tasks" ] && has_tasks=1
  [ -n "$(find "$d" -type f -not -path '*/.git/*' -not -path '*/.cyberos/*' \
        \( -name '*.js' -o -name '*.ts' -o -name '*.py' -o -name '*.rs' -o -name '*.go' \) 2>/dev/null | head -1)" ] && has_src=1
  if [ "$has_head" = 1 ] || [ "$has_tasks" = 1 ] || [ "$has_cyberos" = 1 ]; then echo brownfield; return; fi
  [ "$has_src" = 1 ] && { echo ambiguous; return; }
  echo greenfield
}

# ── a plan@1 fixture generator (a complete, rubric-passing plan) + section surgeons ─────────────
write_complete_plan() {  # write_complete_plan <path>
  mkdir -p "$(dirname "$1")"
  cat > "$1" <<'PLAN'
---
plan_id: PLAN-demo-20260719
template: plan@1
mode: greenfield
intent: "a demo idea"
decision_confidence: low
created: 2026-07-19
scan_ref: null
memory_rows: [abc123def456]
---

# PLAN-demo-20260719

## 1. Intent
Turn the demo idea into something shippable; done = the demo runs.

## 2. Context
For an internal audience. No existing repo. Constraint: keep it small.

## 3. Options

### Option A -- build it new
Hypothesis: a fresh module is cleanest. Evidence: `README.md` shows no prior demo module. Cost: low. Risks: none.

### Option B -- extend an existing tool
Hypothesis: reuse beats rebuild. Evidence: https://example.com/existing-tool documents the extension point. Cost: medium. Risks: coupling.

## 4. Decision
Decision: Option A (confidence: low). Operator verdict: APPROVE (2026-07-19).

## 5. Scope

### In scope
- the demo runner

### Out of scope
- production hardening
- any network calls

## 6. Proposed Task Set
- **Build the demo runner** (class: product) -- scaffold the runner and its entrypoint
- **Add a smoke test for the runner** (class: improvement) -- one test that proves it runs

## 7. Risks
- the demo scope creeps; mitigated by the out list above

## 8. BRAIN Rows
- artefact_write chained at abc123def456 (memory-append verify: OK)
PLAN
}

# ── the three-completeness ORACLE - a faithful reimplementation of plan_rubric PLAN-OPT-001 /
# PLAN-DEC-001 / PLAN-OUT-001, so a fixture's redness is graded against the rubric's own predicate ─
read -r -d '' RUBRIC_PY <<'PY' || true
import sys, re
txt = open(sys.argv[1], encoding="utf-8").read()
def section(name):
    m = re.search(r'^## '+re.escape(name)+r'\s*$(.*?)(?=^## |\Z)', txt, re.M|re.S)
    return m.group(1) if m else None
red = []
opts = section('3. Options')
n_opt = len(re.findall(r'^### Option ', opts, re.M)) if opts else 0
if n_opt < 2: red.append('PLAN-OPT-001')            # missing an option
dec = section('4. Decision')
n_dec = len(re.findall(r'(?im)^\s*(?:\*\*)?Decision:', dec)) if dec else 0
if n_dec != 1: red.append('PLAN-DEC-001')           # zero OR more-than-one decision
scope = section('5. Scope')
out_ok = False
if scope:
    m = re.search(r'^### Out of scope\s*$(.*?)(?=^### |\Z)', scope, re.M|re.S)
    if m and re.search(r'^\s*-\s+\S', m.group(1), re.M): out_ok = True
if not out_ok: red.append('PLAN-OUT-001')           # missing / empty out list
print(','.join(sorted(red)))
PY
rubric_red() { python3 -c "$RUBRIC_PY" "$1"; }

# ── the proposed-task-set PARSER - what create-tasks/task-author reads out of §6 ─────────────────
read -r -d '' TASKSET_PY <<'PY' || true
import sys, re
txt = open(sys.argv[1], encoding="utf-8").read()
m = re.search(r'^## 6\. Proposed Task Set\s*$(.*?)(?=^## |\Z)', txt, re.M|re.S)
body = m.group(1) if m else ''
bullets = re.findall(r'^-\s+\*\*(.+?)\*\*', body, re.M)                       # every task-shaped row
valid   = re.findall(r'^-\s+\*\*(.+?)\*\*\s*\(class:\s*(product|improvement)\)', body, re.M)
print(f"{len(bullets)} {len(valid)}")                                        # total rows, rows with a valid class
PY
taskset_counts() { python3 -c "$TASKSET_PY" "$1"; }

# ══ t01 (AC 1, #1.1): greenfield / brownfield / ambiguous route; ambiguous halts ════════════════
t01_mode_detect() {
  # four fixtures, three outcomes - each bent by exactly one thing
  local gf="$TMP/t01_gf" bfc="$TMP/t01_bfc" bfi="$TMP/t01_bfi" amb="$TMP/t01_amb"
  mkdir -p "$gf" "$bfc" "$bfi" "$amb"
  ( cd "$gf"  && git init -q . )                                             # no commit, no .cyberos, no src
  ( cd "$bfc" && git init -q . && git config user.email t@t && git config user.name t \
       && echo x > f && git add f && git commit -qm c )                      # HEAD -> brownfield
  ( cd "$bfi" && git init -q . ) && mkdir -p "$bfi/.cyberos"                 # .cyberos, no commit -> brownfield
  ( cd "$amb" && git init -q . ) && mkdir -p "$amb/src" && echo 'console.log(1)' > "$amb/src/app.js"  # uncommitted source -> ambiguous

  local m
  m="$(detect_mode "$gf")";  [ "$m" = greenfield ] || { fail t01_mode_detect "greenfield fixture routed $m"; return; }
  m="$(detect_mode "$bfc")"; [ "$m" = brownfield ] || { fail t01_mode_detect "brownfield(commit) fixture routed $m"; return; }
  m="$(detect_mode "$bfi")"; [ "$m" = brownfield ] || { fail t01_mode_detect "brownfield(.cyberos, no commit) fixture routed $m"; return; }
  m="$(detect_mode "$amb")"; [ "$m" = ambiguous ]  || { fail t01_mode_detect "ambiguous fixture routed $m"; return; }

  # not vacuous: the four fixtures produced three distinct outcomes, incl. the ambiguous trap
  # bind the oracle to the DOCTRINE: plan-author §2 must state the same predicate + the halt
  grep -q 'greenfield' "$PA" && grep -q 'brownfield' "$PA" && grep -q 'ambiguous' "$PA" \
    || { fail t01_mode_detect "plan-author does not name all three modes"; return; }
  grep -Eq 'no .?\.cyberos.? +(AND|and).*(no|without).*(git )?HEAD' "$PA" \
    || grep -q 'no `.cyberos/` AND no git HEAD' "$PA" \
    || { fail t01_mode_detect "plan-author does not state the greenfield predicate (no .cyberos AND no HEAD)"; return; }
  # ambiguous MUST halt/ask - the doctrine half of AC 1 (the halt itself is gate-log-verified)
  grep -iEq 'ambiguous.*(HALT|ASK)|(HALT|ASK).*ambiguous' "$PA" \
    || { fail t01_mode_detect "plan-author does not say ambiguous HALTs/ASKs"; return; }
  ok t01_mode_detect
}

# ══ t02 (AC 2, #1.2/#1.3): brownfield scans BEFORE the interview; --scope task byte-identical ═══
t02_scan_first_task_scope_unchanged() {
  # (a) scan-before-interview: in plan-author, the brownfield repo-wide scan step precedes the interview
  local scan_ln int_ln
  scan_ln="$(grep -nE 'scan .*BEFORE the interview|repo-WIDE scan MUST run' "$PA" | head -1 | cut -d: -f1)"
  int_ln="$(grep -nE '^## §4  The interview' "$PA" | head -1 | cut -d: -f1)"
  [ -n "$scan_ln" ] && [ -n "$int_ln" ] || { fail t02_scan_first_task_scope_unchanged "plan-author lacks the scan-before-interview ordering"; return; }
  [ "$scan_ln" -lt "$int_ln" ] || { fail t02_scan_first_task_scope_unchanged "the scan section ($scan_ln) does not precede the interview ($int_ln)"; return; }
  grep -q 'MUST NOT emit a decision without it' "$PA" \
    || { fail t02_scan_first_task_scope_unchanged "plan-author does not forbid a brownfield decision without the scan (#1.2)"; return; }
  # plan-author asks repo-context-map for scope: repo (the new mode), never touching --scope task
  grep -Eq 'scope: *repo' "$PA" || { fail t02_scan_first_task_scope_unchanged "plan-author does not invoke repo-context-map with scope: repo"; return; }

  # (b) THE regression: --scope task output byte-identical to pre-111.
  # (b.1) durable guard: the "## 2. Output schema" block (the task-scope OUTPUT contract) is unchanged.
  local got; got="$(rcm_s2_block "$RCM" | _sha | cut -d' ' -f1)"
  [ "$got" = "$RCM_S2_GOLDEN" ] \
    || { fail t02_scan_first_task_scope_unchanged "repo-context-map §2 Output schema changed (sha $got != golden $RCM_S2_GOLDEN) - task-scope output is NOT byte-identical"; return; }
  # (b.2) additive proof (meaningful while uncommitted): the working tree change vs HEAD has ZERO deletions.
  local numstat del add
  numstat="$(cd "$repo" && git diff --numstat HEAD -- modules/skill/repo-context-map-author/SKILL.md 2>/dev/null)"
  if [ -n "$numstat" ]; then
    add="$(printf '%s' "$numstat" | awk '{print $1}')"; del="$(printf '%s' "$numstat" | awk '{print $2}')"
    [ "$del" = 0 ] || { fail t02_scan_first_task_scope_unchanged "repo-context-map edit deleted/modified $del line(s) - not purely additive"; return; }
    [ "$add" -gt 0 ] || { fail t02_scan_first_task_scope_unchanged "the arm is vacuous: no scope mode was actually added"; return; }
  fi
  # (b.3) task is the DEFAULT so ship-tasks' no-scope call is unchanged; and a repo mode was added
  grep -Eq 'default: *task' "$RCM" || { fail t02_scan_first_task_scope_unchanged "repo-context-map does not default scope to task"; return; }
  grep -q 'byte-identical' "$RCM" || { fail t02_scan_first_task_scope_unchanged "repo-context-map does not assert task-scope byte-identity"; return; }
  grep -Eq '`scope: repo`|scope: repo' "$RCM" || { fail t02_scan_first_task_scope_unchanged "repo-context-map does not document the repo scope"; return; }
  ok t02_scan_first_task_scope_unchanged
}

# ══ t03 (AC 3, #1.4/#1.6): a plan missing an option / a decision / the out list REDS ════════════
t03_rubric_refuses_incomplete() {
  # the rubric must NAME the three red rules
  for rule in PLAN-OPT-001 PLAN-DEC-001 PLAN-OUT-001; do
    grep -q "$rule" "$PR" || { fail t03_rubric_refuses_incomplete "rubric missing rule $rule"; return; }
  done
  # a complete plan is GREEN under all three (proves the checker is not always-red)
  local good="$TMP/t03/good.md"; write_complete_plan "$good"
  local r; r="$(rubric_red "$good")"
  [ -z "$r" ] || { fail t03_rubric_refuses_incomplete "a complete plan reds: [$r] (checker is broken or fixture is not complete)"; return; }

  # missing an OPTION -> only PLAN-OPT-001 reds
  local mo="$TMP/t03/missing_option.md"; write_complete_plan "$mo"
  python3 - "$mo" <<'PY'
import sys, re
p=sys.argv[1]; t=open(p,encoding="utf-8").read()
# drop Option B, leaving exactly one option
t=re.sub(r'\n### Option B.*?(?=\n## 4\. Decision)', '\n', t, flags=re.S)
open(p,"w",encoding="utf-8").write(t)
PY
  r="$(rubric_red "$mo")"; [ "$r" = "PLAN-OPT-001" ] || { fail t03_rubric_refuses_incomplete "missing-option red set was [$r], want PLAN-OPT-001"; return; }

  # missing a DECISION -> only PLAN-DEC-001 reds
  local md="$TMP/t03/missing_decision.md"; write_complete_plan "$md"
  python3 - "$md" <<'PY'
import sys, re
p=sys.argv[1]; t=open(p,encoding="utf-8").read()
# blank the decision body (keep the heading) so no "Decision:" line remains
t=re.sub(r'(## 4\. Decision\n).*?(?=\n## 5\. Scope)', r'\1(TODO: no decision recorded)\n', t, flags=re.S)
open(p,"w",encoding="utf-8").write(t)
PY
  r="$(rubric_red "$md")"; [ "$r" = "PLAN-DEC-001" ] || { fail t03_rubric_refuses_incomplete "missing-decision red set was [$r], want PLAN-DEC-001"; return; }

  # TWO decisions -> PLAN-DEC-001 also reds ("exactly one" is non-vacuous in both directions)
  local td="$TMP/t03/two_decisions.md"; write_complete_plan "$td"
  python3 - "$td" <<'PY'
import sys, re
p=sys.argv[1]; t=open(p,encoding="utf-8").read()
t=t.replace("Decision: Option A (confidence: low). Operator verdict: APPROVE (2026-07-19).",
            "Decision: Option A (confidence: low).\nDecision: Option B (confidence: low).\nOperator verdict: APPROVE (2026-07-19).")
open(p,"w",encoding="utf-8").write(t)
PY
  r="$(rubric_red "$td")"; [ "$r" = "PLAN-DEC-001" ] || { fail t03_rubric_refuses_incomplete "two-decision red set was [$r], want PLAN-DEC-001"; return; }

  # missing the OUT LIST -> only PLAN-OUT-001 reds
  local ml="$TMP/t03/missing_out.md"; write_complete_plan "$ml"
  python3 - "$ml" <<'PY'
import sys, re
p=sys.argv[1]; t=open(p,encoding="utf-8").read()
# remove the "### Out of scope" subsection and its bullets, keep In scope
t=re.sub(r'\n### Out of scope\n(?:- .*\n)+', '\n', t)
open(p,"w",encoding="utf-8").write(t)
PY
  r="$(rubric_red "$ml")"; [ "$r" = "PLAN-OUT-001" ] || { fail t03_rubric_refuses_incomplete "missing-out red set was [$r], want PLAN-OUT-001"; return; }

  # plan-audit refuses below 10/10 (#1.6)
  grep -Eq 'refuse.* below .*10/10|below 10/10|10/10 passes|Refuses to pass below 10/10|Refuse to pass below 10/10' "$PU" \
    || grep -q 'refuse' "$PU" || { fail t03_rubric_refuses_incomplete "plan-audit does not state the 10/10 refusal"; return; }
  ok t03_rubric_refuses_incomplete
}

# ══ t04 (AC 4, #1.7): a plan run leaves docs/tasks/** and BACKLOG.md BYTE-IDENTICAL ═════════════
# The clause is "MUST NOT write". So this asserts the BYTES, not a claim about the bytes.
t04_never_writes_tasks() {
  local d="$TMP/t04"; mkdir -p "$d/docs/tasks/demo"
  printf '# BACKLOG\n\n| id | status |\n|---|---|\n| TASK-DEMO-001 | done |\n' > "$d/docs/tasks/BACKLOG.md"
  printf -- '---\nid: TASK-DEMO-001\nstatus: done\n---\n# demo\n' > "$d/docs/tasks/demo/spec.md"
  # fingerprint EVERY file under docs/tasks (path + digest catches create/delete, not just edit)
  local before after
  before="$(cd "$d" && find docs/tasks -type f | LC_ALL=C sort | while IFS= read -r f; do _sha "$f"; done | _sha)"

  # simulate the plan's ONLY write: a plan@1 under docs/plans/** (plan-author §5 output path)
  write_complete_plan "$d/docs/plans/PLAN-demo-20260719/plan.md"
  [ -s "$d/docs/plans/PLAN-demo-20260719/plan.md" ] \
    || { fail t04_never_writes_tasks "the arm is vacuous: the plan wrote no artefact, so 'left docs/tasks alone' proves nothing"; return; }

  after="$(cd "$d" && find docs/tasks -type f | LC_ALL=C sort | while IFS= read -r f; do _sha "$f"; done | _sha)"
  [ "$before" = "$after" ] || { fail t04_never_writes_tasks "docs/tasks/** changed across a plan run (#1.7)"; return; }
  # the plan output path is DISJOINT from docs/tasks - nothing was written under it
  [ -z "$(find "$d/docs/tasks" -path '*plan*' 2>/dev/null)" ] || { fail t04_never_writes_tasks "a plan artefact leaked under docs/tasks"; return; }

  # doctrine: plan-author declares the four MUST-NOTs; the rubric enforces PLAN-SAFE
  grep -q 'Write `docs/tasks/\*\*`' "$PA" || { fail t04_never_writes_tasks "plan-author does not forbid writing docs/tasks/**"; return; }
  grep -q 'BACKLOG' "$PA"   || { fail t04_never_writes_tasks "plan-author does not forbid a BACKLOG row"; return; }
  grep -q 'set any task .status.\|set any task `status`\|set any task status' "$PA" || { fail t04_never_writes_tasks "plan-author does not forbid setting task status"; return; }
  grep -q 'PLAN-SAFE-001' "$PR" || { fail t04_never_writes_tasks "rubric lacks PLAN-SAFE-001 (no-write-to-tasks)"; return; }

  # AC 7 DOCTRINE (the halt itself is gate-log-verified, not suite-asserted): the decision gate
  # HALTs and emits NO artefact without a verdict, and the rubric carries the gate rule.
  grep -q 'No .*plan@1.* is written without a recorded verdict\|write NO artefact without a verdict\|emits NO artefact without a verdict\|emit NO artefact without a verdict' "$PA" \
    || { fail t04_never_writes_tasks "plan-author does not state the decision-gate halt (AC 7 doctrine)"; return; }
  grep -q 'PLAN-GATE-001' "$PR" || { fail t04_never_writes_tasks "rubric lacks PLAN-GATE-001 (operator-verdict gate)"; return; }
  ok t04_never_writes_tasks
}

# ══ t05 (AC 5, #1.8): a greenfield idea-only plan@1 is consumed by create-tasks UNMODIFIED ══════
t05_output_feeds_create_tasks() {
  local p="$TMP/t05/plan.md"; write_complete_plan "$p"   # a greenfield (scan_ref: null) plan
  grep -q '^mode: greenfield' "$p" && grep -q '^scan_ref: null' "$p" \
    || { fail t05_output_feeds_create_tasks "fixture is not a greenfield idea-only plan"; return; }
  # it is plain UTF-8 markdown (the input TYPE task-author accepts as source_files)
  iconv -f UTF-8 -t UTF-8 "$p" >/dev/null 2>&1 || { fail t05_output_feeds_create_tasks "plan@1 is not valid UTF-8"; return; }

  # THE format check: §6 rows are exactly what create-tasks parses - title + class in {product, improvement}
  read -r total valid < <(taskset_counts "$p")
  [ "${total:-0}" -ge 1 ]        || { fail t05_output_feeds_create_tasks "no proposed-task rows found in §6"; return; }
  [ "$total" = "$valid" ]        || { fail t05_output_feeds_create_tasks "not every §6 row carries a valid class ($valid/$total)"; return; }

  # not vacuous: a class-less row is DETECTED (the parser discriminates, it is not always-green)
  local bad="$TMP/t05/plan_bad.md"; write_complete_plan "$bad"
  printf '\n- **A task with no class** -- create-tasks would have to guess\n' >> "$bad"
  # append it INSIDE §6 (before §7) so the parser sees it
  python3 - "$bad" <<'PY'
import sys, re
p=sys.argv[1]; t=open(p,encoding="utf-8").read()
t=t.replace("\n- **A task with no class** -- create-tasks would have to guess\n","")
t=t.replace("- **Add a smoke test for the runner** (class: improvement) -- one test that proves it runs\n",
            "- **Add a smoke test for the runner** (class: improvement) -- one test that proves it runs\n- **A task with no class** -- create-tasks would have to guess\n")
open(p,"w",encoding="utf-8").write(t)
PY
  read -r btotal bvalid < <(taskset_counts "$bad")
  [ "$btotal" -gt "$bvalid" ] || { fail t05_output_feeds_create_tasks "the parser did not flag a class-less row ($bvalid/$btotal)"; return; }

  # cross-check the CONSUMER: create-tasks distinguishes exactly these two classes
  grep -q 'class: improvement' "$CREATE" && grep -q 'class: product' "$CREATE" \
    || { fail t05_output_feeds_create_tasks "create-tasks does not name the product/improvement classes the plan emits"; return; }
  # doctrine: plan-author mandates §6 as the create-tasks input contract, no new input shape
  grep -q '## 6. Proposed Task Set' "$PA" && grep -q 'create-tasks' "$PA" \
    || { fail t05_output_feeds_create_tasks "plan-author does not bind §6 to the create-tasks input contract"; return; }
  ok t05_output_feeds_create_tasks
}

# ══ t06 (AC 6, #1.9): the decision is appended to BRAIN and the chain verifies ══════════════════
t06_brain_rows_chain() {
  [ -f "$MA" ] || { fail t06_brain_rows_chain "memory-append.mjs missing at $MA"; return; }
  local s="$TMP/t06/store"
  printf '{"plan_id":"PLAN-demo-20260719","mode":"greenfield","decision":"Option A","decision_confidence":"low","plan_path":"docs/plans/PLAN-demo-20260719/plan.md"}\n' > "$TMP/t06_payload.json"
  # append the plan decision as an artefact_write row (the closed-set kind plan-author §8 uses)
  CYBEROS_NOW="2026-07-19T00:00:00Z" CYBEROS_ACTOR="suite" node "$MA" append "$s" artefact_write "$TMP/t06_payload.json" > "$TMP/t06.out" 2> "$TMP/t06.err" \
    || { fail t06_brain_rows_chain "append failed: $(tail -1 "$TMP/t06.err")"; return; }
  # a second decision row - the chain must extend, not reset
  printf '{"plan_id":"PLAN-demo-20260720","mode":"brownfield","decision":"Option B","decision_confidence":"medium","plan_path":"docs/plans/PLAN-demo-20260720/plan.md"}\n' > "$TMP/t06_payload2.json"
  CYBEROS_NOW="2026-07-19T00:00:01Z" CYBEROS_ACTOR="suite" node "$MA" append "$s" artefact_write "$TMP/t06_payload2.json" >/dev/null 2>&1 \
    || { fail t06_brain_rows_chain "second append failed"; return; }
  # the chain VERIFIES (exit 0)
  node "$MA" verify "$s" > "$TMP/t06.vout" 2>&1 || { fail t06_brain_rows_chain "verify failed on an intact chain: $(cat "$TMP/t06.vout")"; return; }
  grep -q "HEAD=2" "$TMP/t06.vout" || { fail t06_brain_rows_chain "verify did not report 2 chained rows: $(cat "$TMP/t06.vout")"; return; }

  # not vacuous: verify is a real check - one flipped payload byte makes it FAIL (exit non-zero)
  cp "$s/audit/current.binlog" "$TMP/t06.bak"
  node -e 'const fs=require("node:fs");const b=fs.readFileSync(process.argv[1]);b[24+5]^=0x01;fs.writeFileSync(process.argv[1],b);' "$s/audit/current.binlog"
  node "$MA" verify "$s" >/dev/null 2>&1 && { fail t06_brain_rows_chain "verify passed on a TAMPERED chain - it is not actually checking"; return; }
  cp "$TMP/t06.bak" "$s/audit/current.binlog"

  # doctrine: plan-author §8 names memory-append + the artefact_write kind + the verify step
  grep -q 'memory-append' "$PA" && grep -q 'artefact_write' "$PA" && grep -q 'verify' "$PA" \
    || { fail t06_brain_rows_chain "plan-author §8 does not bind the BRAIN append to memory-append/artefact_write/verify"; return; }
  grep -q 'PLAN-BRAIN-001' "$PR" || { fail t06_brain_rows_chain "rubric lacks PLAN-BRAIN-001"; return; }
  ok t06_brain_rows_chain
}

# ══ t07 (beyond the AC set): the declared build.sh vendor lines actually vendor ═════════════════
t07_payload_vendored() {
  bash "$repo/tools/install/build.sh" "$TMP/payload" > "$TMP/build.log" 2>&1 \
    || { fail t07_payload_vendored "build.sh failed: $(tail -3 "$TMP/build.log" | tr '\n' ' ')"; return; }
  # the two skills ship in BOTH trees (or the vendored /plan command names a skill nobody has)
  local t
  for t in cuo/skills plugin/skills; do
    [ -s "$TMP/payload/$t/plan-author/SKILL.md" ] || { fail t07_payload_vendored "plan-author missing from payload/$t"; return; }
    [ -s "$TMP/payload/$t/plan-audit/SKILL.md" ]  || { fail t07_payload_vendored "plan-audit missing from payload/$t"; return; }
  done
  # the standalone rubric flattens into cuo/rubrics/ next to bug.md/common.md, byte-identical to source
  [ -s "$TMP/payload/cuo/rubrics/plan_rubric.md" ] || { fail t07_payload_vendored "plan_rubric.md not vendored into cuo/rubrics/"; return; }
  cmp -s "$PR" "$TMP/payload/cuo/rubrics/plan_rubric.md" || { fail t07_payload_vendored "vendored plan_rubric.md differs from source"; return; }
  # the /plan command ships
  [ -s "$TMP/payload/plugin/commands/plan.md" ] || { fail t07_payload_vendored "plan.md not vendored into plugin/commands/"; return; }
  # the census matches: adding two skills makes the manifest count 56 (was 54); computed == dir count
  local m dcount
  m="$(grep -o 'author_audit_skills: [0-9]*' "$TMP/payload/manifest.yaml" | awk '{print $2}')"
  dcount="$(ls "$TMP/payload/cuo/skills" | wc -l | tr -d ' ')"
  [ "$m" = "$dcount" ] || { fail t07_payload_vendored "manifest count $m != vendored dir count $dcount"; return; }
  ok t07_payload_vendored
}

echo "plan workflow suite (TASK-IMP-111):"
want t01 && t01_mode_detect
want t02 && t02_scan_first_task_scope_unchanged
want t03 && t03_rubric_refuses_incomplete
want t04 && t04_never_writes_tasks
want t05 && t05_output_feeds_create_tasks
want t06 && t06_brain_rows_chain
want t07 && t07_payload_vendored
echo "----"; echo "test_plan_workflow: pass=$PASS fail=$FAIL"
[ "$FAIL" -eq 0 ]
