#!/usr/bin/env bash
# check-pair-parity.sh <skills-dir> - TASK-SKILL-118 §1 #6.
# Verifies every author/audit pair carries its file classes. The two arrays below
# ARE the policy (TASK-SKILL-118 §10 #3): change them only by amending that task.
set -uo pipefail
dir="${1:-}"
[ -d "$dir" ] || { echo "cyberos: ERROR: unreadable skills dir: $dir" >&2; exit 2; }

AUTHOR_CLASSES=(PIPELINE.md INVARIANTS.md envelopes/input.json envelopes/output.json references/FAILURE_MODES.md acceptance/README.md)
AUDIT_CLASSES=(RUBRIC.md AUDIT_LOOP.md REPORT_FORMAT.md envelopes/input.json envelopes/output.json acceptance/README.md)

# Scope list: EVERY author/audit pair in the vendored payload, in build.sh VENDORED_SKILLS
# lifecycle order (TASK-SKILL-202 §1 #4 inverted TASK-CUO-209's grows-as-deepened design:
# all pairs are held to the floor, and a pair vendored without SCOPE membership is a defect —
# scripts/tests/test_skill_stub_lint.sh asserts SCOPE == the vendored pair set).
SCOPE=(statement-of-work product-requirements-document software-requirements-specification
       plan task architectural-spike architecture-decision-record threat-model
       software-design-document repo-context-map implementation-plan edge-case-matrix
       mock-contract-test observability-injection backlog-state-update code-review
       test-strategy coverage-gate debugging-cycle deployment-checklist release-notes
       runbook retrospective postmortem decommissioning)

rc=0
for name in "${SCOPE[@]}"; do
  for side in author audit; do
    d="$dir/$name-$side"
    [ -d "$d" ] || continue   # pair not present in this tree (e.g. trimmed payload) - presence is chain-coverage's job
    classes_var="AUTHOR_CLASSES[@]"; [ "$side" = audit ] && classes_var="AUDIT_CLASSES[@]"
    for f in "${!classes_var}"; do
      [ -f "$d/$f" ] || { echo "PARITY $name-$side: missing $f"; rc=10; }
    done
  done
done
[ "$rc" -eq 0 ] && echo "parity OK: $(ls -d "$dir"/*-author 2>/dev/null | wc -l | tr -d ' ') author dirs scanned, scope ${#SCOPE[@]} pairs"
exit "$rc"
