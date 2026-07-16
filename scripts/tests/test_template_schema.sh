#!/usr/bin/env bash
# test_template_schema.sh — the task templates must satisfy the task-audit rubric.
#
# WHY THIS EXISTS: on 2026-07-15 `contracts/task/templates/feature.md` still carried the
# PRE-migration schema — `feature_type: user_facing` (retired by FM-108), no `type:`, no
# `id:`, no `module:` — while RUBRIC.md FM-108 required `type` at ERROR severity. A task
# authored from that skeleton failed the audit gate the moment it was written. Since
# improvement.md and chore.md are pointers to feature.md, 3 of the 4 types inherited it.
#
# It survived because NOTHING EXECUTES A TEMPLATE. Templates are prompt text an LLM
# renders: no test imports them, no gate parses them, no build step touches them. The
# rubric and the templates were authored in the same change and never checked against
# each other. `bug.md` was correct only because it happened to be written afterwards.
#
# tools/install/templates/TASK-TEMPLATE.md — the one install.sh hands to every new
# repo — was worse: `class: product` and `priority: SHOULD`, a schema retired on
# 2026-07-14. That is the FIRST artifact a new project touches.
#
# The rule: a document that another document must agree with needs a gate, or the two
# drift the moment one is edited. "Nobody runs it" is not "nobody depends on it".
#
#   bash scripts/tests/test_template_schema.sh
set -uo pipefail
repo="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$repo"
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); printf '  \033[32mok\033[0m   %s\n' "$1"; }
fail() { FAIL=$((FAIL+1)); printf '  \033[31mFAIL\033[0m %s: %s\n' "$1" "$2"; }
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT   # t08 fixture + scratch payload build

TPL="$repo/modules/skill/contracts/task/templates"
RUBRIC="$repo/modules/skill/task-audit/RUBRIC.md"

# Fields RUBRIC.md FM-108 retired. Any template still offering them teaches a schema the
# audit gate rejects.
RETIRED='^(feature_type|class):'

t01_rubric_present() {
  [ -f "$RUBRIC" ] && grep -q 'FM-108' "$RUBRIC" \
    && ok t01 || fail t01 "RUBRIC.md missing or has no FM-108 — this test is checking nothing"
}

# A skeleton declares frontmatter. A pointer says "load feature.md" and declares none.
# Both are legitimate; only skeletons are schema-checked.
is_skeleton() { head -1 "$1" 2>/dev/null | grep -q '^---$'; }

t02_skeletons_carry_type() {
  local bad=""
  for f in "$TPL"/*.md; do
    is_skeleton "$f" || continue
    grep -qE '^type:' "$f" || bad="$bad $(basename "$f")"
  done
  [ -z "$bad" ] && ok t02 || fail t02 "no \`type:\` (FM-108, error severity):$bad"
}

t03_no_retired_fields() {
  local bad=""
  for f in "$TPL"/*.md; do
    is_skeleton "$f" || continue
    grep -qE "$RETIRED" "$f" && bad="$bad $(basename "$f")"
  done
  [ -z "$bad" ] && ok t03 || fail t03 "offers a field FM-108 retired:$bad"
}

t04_pointers_resolve() {
  local bad=""
  for f in "$TPL"/*.md; do
    is_skeleton "$f" && continue
    # A pointer must name a template that exists, or task-author HALTs at W2.
    for target in $(grep -oE '\btemplates/[a-z]+\.md|`[a-z]+\.md`' "$f" 2>/dev/null | tr -d '`' | sed 's|templates/||' | sort -u); do
      [ -f "$TPL/$target" ] || bad="$bad $(basename "$f")->$target"
    done
  done
  [ -z "$bad" ] && ok t04 || fail t04 "pointer names a missing template:$bad"
}

t05_every_fm108_type_has_a_template() {
  # FM-108's enum and this directory must agree, or task-author HALTs on a real type.
  local bad=""
  for ty in feature bug improvement chore; do
    [ -f "$TPL/$ty.md" ] || bad="$bad $ty"
  done
  [ -z "$bad" ] && ok t05 || fail t05 "FM-108 enum has no template:$bad"
}

t06_init_template_current() {
  # The template install.sh hands to every new repo (23 of them) must not teach a schema
  # the audit gate rejects.
  local f="$repo/tools/install/templates/TASK-TEMPLATE.md"
  [ -f "$f" ] || { fail t06 "missing — install.sh:651 tells every new user to cp it"; return; }
  local why=""
  grep -qE '^type:' "$f" || why="$why no-type(FM-108)"
  grep -qE "$RETIRED" "$f" && why="$why offers-retired-field"
  grep -qE '^priority: *(MUST|SHOULD|COULD)' "$f" && why="$why MoSCoW-priority"
  [ -z "$why" ] && ok t06 || fail t06 "init template teaches a retired schema:$why"
}

t07_payload_ships_the_templates() {
  # task-author dispatches on templates/{type}.md and HALTs when one is missing. If the
  # payload does not carry them, every installed repo halts on first author.
  #
  # Assert the EXACT path build.sh writes. The first cut used
  #     find "$p" -name "$ty.md" -path '*task*'
  # which requires "task" somewhere in the path — but build.sh flattens these into
  # cuo/templates/ (matching STATUS-REFERENCE.md), so the real path is
  # dist/cyberos/cuo/templates/feature.md and the filter excluded it. The payload was
  # correct and the test failed: an assert written against the path I expected instead of
  # the one I had chosen an hour earlier, in build.sh, myself.
  #
  # A `find` filter that silently matches nothing is indistinguishable from a real
  # absence. Name the path.
  local d="$repo/dist/cyberos/cuo/templates"
  [ -d "$repo/dist/cyberos" ] || { ok t07; return; }   # payload not built here; not this test's job
  [ -d "$d" ] || { fail t07 "payload has no cuo/templates/ at all"; return; }
  local bad=""
  for ty in feature bug improvement chore; do
    [ -f "$d/$ty.md" ] || bad="$bad $ty"
  done
  [ -z "$bad" ] && ok t07 || fail t07 "payload ships no template for:$bad (task-author HALTs on every installed repo)"
}


# --- TASK-IMP-089: one out-of-scope home -----------------------------------------------
# Until 2026-07-16 this template mandated TWO out-of-scope homes: the PRD half's
# `## Scope > ### Out of scope / Non-Goals` (the rubric's home, SEC-006 — carried by the
# per-type templates and every authored spec) and an engineering-half
# `## 4. Out of scope / non-goals`. Two homes for one statement invite divergence: all
# nine specs shipped that run needed a pointer line reconciling them. Per the recorded
# IMP-07 decision the duplicate is gone and Protected invariants renumbered 5 -> 4.
# TASK-TEMPLATE.md is the engineering half only — it carries no `## Scope` — so the
# truthful shape is: NO out-of-scope H2 at all, invariants at `## 4.`, nothing left at
# `## 5.`. The H3 PRD home is deliberately out of this oracle's reach (`^## ` cannot
# match `###`).
INIT_TPL="$repo/tools/install/templates/TASK-TEMPLATE.md"

# Shape oracle shared by all three t08 arms, so the live template, the reintroduction
# fixture and the vendored payload copy are judged by ONE set of rules. Echoes reason
# tokens; empty output = conforming.
shape_why() {
  local f="$1" why=""
  # numbered (any number — a duplicate at `## 6.` is still a duplicate) or unnumbered H2
  grep -qiE '^## +([0-9]+\. *)?out of scope' "$f" && why="$why duplicate-out-of-scope-H2"
  [ "$(grep -cE '^## 4\. Protected invariants this task must not weaken$' "$f")" = 1 ] || why="$why invariants-not-at-##4"
  grep -qE '^## 5\.' "$f" && why="$why stray-##5-heading"
  printf '%s' "$why"
}

t08_single_out_of_scope_home() {
  [ -f "$INIT_TPL" ] || { fail t08_single_out_of_scope_home "missing — install.sh quickstart cps it into every new repo"; return; }
  local why; why="$(shape_why "$INIT_TPL")"
  # "renumbered, content unchanged": the invariants BODY must have survived the renumber
  grep -q 'must never be made green by weakening' "$INIT_TPL" || why="$why invariants-body-missing"
  [ -z "$why" ] && ok t08_single_out_of_scope_home || fail t08_single_out_of_scope_home "template regrew the duplicate shape:$why"
}

t08_duplicate_reintroduction_fails() {
  # The oracle must go red on the OLD shape, or t08 green means nothing. Re-add the
  # retired section 4 above the invariants and demand shape_why names exactly that —
  # a decayed regex failing for some other reason must not count as detection.
  local fx="$TMP/TASK-TEMPLATE.reintroduced.md"
  awk '/^## 4\. Protected invariants/ { print "## 4. Out of scope / non-goals"; print ""; print "- ..."; print "" } { print }' "$INIT_TPL" > "$fx"
  case "$(shape_why "$fx")" in
    *duplicate-out-of-scope-H2*) ok t08_duplicate_reintroduction_fails ;;
    *) fail t08_duplicate_reintroduction_fails "oracle missed the reintroduced section 4" ;;
  esac
}

t08_payload_carries_shape() {
  # The template reaches consumers only through the payload. Build to SCRATCH (never
  # dist/ — rebuilding that is the batch parent's step) and assert the vendored copy is
  # byte-identical to source and conforming. Name the exact path, per t07's lesson.
  local p="$TMP/payload"
  bash "$repo/tools/install/build.sh" "$p" >/dev/null 2>&1 || { fail t08_payload_carries_shape "scratch build.sh failed"; return; }
  local v="$p/cuo/templates/TASK-TEMPLATE.md"
  [ -f "$v" ] || { fail t08_payload_carries_shape "payload carries no cuo/templates/TASK-TEMPLATE.md"; return; }
  local why; why="$(shape_why "$v")"
  cmp -s "$INIT_TPL" "$v" || why="$why payload-copy-diverges-from-source"
  [ -z "$why" ] && ok t08_payload_carries_shape || fail t08_payload_carries_shape "vendored template wrong:$why"
}

t01_rubric_present; t02_skeletons_carry_type; t03_no_retired_fields; t04_pointers_resolve
t05_every_fm108_type_has_a_template; t06_init_template_current; t07_payload_ships_the_templates
t08_single_out_of_scope_home; t08_duplicate_reintroduction_fails; t08_payload_carries_shape
echo "----"; echo "pass=$PASS fail=$FAIL"
[ "$FAIL" -eq 0 ]
