#!/usr/bin/env bash
# test_full_sdp_payload.sh - TASK-CUO-209 §5 suite (t01-t08 -> AC 1-8).
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
BUILD="$repo/tools/cyberos-init/build.sh"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

echo "building scratch payload..."
OUT="$(bash "$BUILD" "$TMP/payload" 2>&1)" || { echo "FATAL: build failed: $OUT"; exit 1; }

t01_stage_matrix_ships() {                                            # AC 1
  local all=1
  # one representative pair per SDP stage 1..14 (spot matrix per AC 1)
  local matrix="statement-of-work product-requirements-document software-requirements-specification
nfr-certification task architectural-spike software-design-document implementation-plan
code-review coverage-gate deployment-checklist release-notes runbook retrospective"
  for s in $matrix; do
    case "$s" in nfr-certification) names="$s-author" ;; *) names="$s-author $s-audit" ;; esac
    for n in $names; do for t in cuo plugin; do
      [ -f "$TMP/payload/$t/skills/$n/SKILL.md" ] || { fail t01 "missing $t/skills/$n"; all=0; }
    done; done
  done
  n="$(ls "$TMP/payload/cuo/skills" | wc -l)"
  [ "$n" -eq 52 ] || { fail t01 "expected 52 vendored skills, got $n"; all=0; }
  [ "$all" -eq 1 ] && ok t01
}
t02_set_is_reviewable_data() {                                        # AC 2
  grep -q "VENDORED_SKILLS" "$BUILD" && grep -q "# SDP 1  SOW" "$BUILD" && grep -q "# SDP 14" "$BUILD" \
    && ! grep -q 'skills="repo-context-map-author repo' "$BUILD" \
    && ok t02 || fail t02 "set is not the commented one-per-line block"
}
t03_counts_computed() {                                               # AC 3
  grep -q 'author_audit_skills: \$vendored_skills' "$BUILD" || { fail t03 "manifest count not computed"; return; }
  m="$(grep -o 'author_audit_skills: [0-9]*' "$TMP/payload/manifest.yaml" | awk '{print $2}')"
  d="$(ls "$TMP/payload/cuo/skills" | wc -l | tr -d ' ')"
  [ "$m" = "$d" ] && ok t03 || fail t03 "manifest=$m dir=$d"
}
t04_lifecycle_map_total() {                                           # AC 4
  local G="$TMP/payload/GUIDE.md"
  rows="$(grep -c '^| 1[0-4] \|^| [1-9] ' "$G")"
  [ "$rows" -eq 14 ] || { fail t04 "expected 14 stage rows, got $rows"; return; }
  grep -q "TBD" "$G" && { fail t04 "TBD row present"; return; }
  bad="$(grep '^| [0-9]' "$G" | grep -cv -E '/(create|ship)-tasks|standalone')"
  [ "$bad" -eq 0 ] && ok t04 || fail t04 "$bad rows lack a valid invoker"
}
t05_sibling_checks_green() {                                          # AC 5
  bash "$repo/tools/cyberos-init/check-chain-coverage.sh" "$TMP/payload" >/dev/null 2>&1 \
    && ok t05 || fail t05 "chain-coverage red over the expanded set"
  [ -f "$repo/tools/cyberos-init/check-pair-parity.sh" ] \
    || echo "  SKIP t05b pair-parity (checker lands with TASK-SKILL-118)"
}
t06_size_budget() {                                                   # AC 6
  echo "$OUT" | grep -q "payload=[0-9]" && echo "$OUT" | grep -q "plugin_zip=[0-9]" || { fail t06 "sizes not reported"; return; }
  pz="$(echo "$OUT" | grep -o 'plugin_zip=[0-9]*' | cut -d= -f2)"
  [ "$pz" -le 2097152 ] || { fail t06 "plugin zip over budget: $pz"; return; }
  grep -q "2097152\|2 MB budget" "$BUILD" && ok t06 || fail t06 "budget assert missing from build.sh"
}
t07_reduced_floor_intact() {                                          # AC 7
  local F="$TMP/fakerepo"
  mkdir -p "$F/modules/cuo/chief-technology-officer/workflows" "$F/modules/skill/contracts/task" "$F/modules/memory/cyberos/data" "$F/tools"
  cp -r "$repo/tools/cyberos-init" "$F/tools/cyberos-init"
  cp "$repo/modules/cuo/chief-technology-officer/workflows/ship-tasks.md" "$F/modules/cuo/chief-technology-officer/workflows/"
  cp "$repo/modules/cuo/EXECUTION-DISCIPLINE.md" "$F/modules/cuo/"
  cp "$repo/modules/skill/contracts/task/STATUS-REFERENCE.md" "$F/modules/skill/contracts/task/"
  cp "$repo/modules/memory/cyberos/data/AGENTS.md" "$F/modules/memory/cyberos/data/" 2>/dev/null || true
  cp "$repo/modules/memory/memory.schema.json" "$F/modules/memory/" 2>/dev/null || true
  cp "$repo/modules/memory/memory.invariants.yaml" "$F/modules/memory/" 2>/dev/null || true
  cp "$repo/AGENTS.md" "$F/AGENTS.md" 2>/dev/null || cp "$repo/modules/memory/cyberos/data/AGENTS.md" "$F/AGENTS.md"
  echo "9.9.9" > "$F/VERSION"
  rm -rf "$F/modules/skill"/*-author "$F/modules/skill"/*-audit 2>/dev/null
  out7="$(bash "$F/tools/cyberos-init/build.sh" "$F/dist" 2>&1)" || { fail t07 "skill-less build failed: $(echo "$out7" | tail -1)"; return; }
  echo "$out7" | grep -q "profile=reduced" && grep -q "cyberos_version: 9.9.9" "$F/dist/manifest.yaml" \
    && ok t07 || fail t07 "reduced floor broken: $(echo "$out7" | grep done)"
}
t08_workflows_vendored_intact() {                                     # AC 8 (amended post-ship: durable form)
  # Original diff-clean form was TASK-CUO-209's point-in-time scope guard; git history holds that proof.
  # Durable invariant: both workflow docs ship in the payload and the chain structure is intact.
  local ship="$TMP/payload/cuo/ship-tasks.md"
  local create="$TMP/payload/plugin/commands/create-tasks.md"
  [ -f "$ship" ] && [ -f "$create" ] && grep -q "skill_chain:" "$ship" \
    && grep -q "Resume semantics" "$ship" \
    && ok t08 || fail t08 "workflow docs missing from payload or chain structure broken"
}

t01_stage_matrix_ships; t02_set_is_reviewable_data; t03_counts_computed; t04_lifecycle_map_total
t05_sibling_checks_green; t06_size_budget; t07_reduced_floor_intact; t08_workflows_vendored_intact
echo "----"; echo "pass=$PASS fail=$FAIL"
[ "$FAIL" -eq 0 ]
