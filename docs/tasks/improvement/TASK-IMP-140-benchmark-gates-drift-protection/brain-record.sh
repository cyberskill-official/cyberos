#!/usr/bin/env bash
# brain-record.sh — TASK-IMP-140 §1.6: record the 2026-07-23 deep-audit verdict, the
# sixteen benchmark gates (by reference to their published home), and the hardening-wave
# decisions into the BRAIN through the canonical writer, with chained audit rows.
#
# HARD PRECONDITION (spec §1.6 + AGENTS.md §12): this script REFUSES to run unless
# TASK-MEMORY-303's store repair has landed and `cyberos doctor` reports the live store
# READY. Recording into a frozen store would violate the protocol the audit measured.
# It was deliberately NOT executed during the batch/8 implementation wave (the store was
# FROZEN_RECOVERABLE and the wave's ownership partition forbade memory writes); the final
# sequential pass runs it, post-303, with the operator present.
#
# Usage: bash docs/tasks/improvement/TASK-IMP-140-benchmark-gates-drift-protection/brain-record.sh
# Env:   CYBEROS_ACTOR (default: stephen) — the actor recorded on the audit rows.
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../../.." && pwd)"
STORE="$root/.cyberos/memory/store"
ACTOR="${CYBEROS_ACTOR:-stephen}"

CY() { PYTHONPATH="$root/modules/memory${PYTHONPATH:+:$PYTHONPATH}" python3 -m cyberos --store "$STORE" --actor "$ACTOR" "$@"; }

echo "== brain-record: preconditions (§1 pre-write checklist) =="
[ -d "$STORE" ] || { echo "REFUSED: no store at $STORE"; exit 2; }

state="$(CY state 2>&1 || true)"
if ! grep -q 'READY' <<<"$state"; then
  echo "REFUSED: store state is not READY (§12):"
  sed 's/^/  /' <<<"$state"
  echo "Run TASK-MEMORY-303's operator-gated layout repair first, then re-run this script."
  exit 2
fi
CY doctor || { echo "REFUSED: cyberos doctor reports the store below READY (§12) - recording would violate §1 step 1."; exit 2; }
echo "doctor: READY - proceeding."

# ---- bodies (three memory files, kind: decisions) -----------------------------------
BODIES="$(mktemp -d)"; trap 'rm -rf "$BODIES"' EXIT

cat > "$BODIES/audit-verdict.md" <<'BODY'
# 2026-07-23 CyberOS deep audit — verdict record

Operator-approved deep audit of the CyberOS platform repo (plan file
cyberos_hardening_plan_49404998). Headline findings, all verified first-hand at HEAD:
fail-open machine-gate floor; honor-system HITL (no mechanical lock at the two
human-acceptance transitions); vendored CAF gate structurally broken in consumer installs
(ROOT resolution); status-enum forks (10-vs-12); payload/doc divergence (skill-log.mjs
class); nine always-green stub CI workflows; CAF eval suite running in no CI; loop-bound
fork (route-back ceiling 3 in doctrine vs 2 in api.py); live BRAIN store frozen by layout
pollution. Remediation shipped as the batch/8 hardening wave: Phase 1 polish (d3652a5b),
Phase 2 improvement tasks TASK-CUO-302/303/304, TASK-SKILL-202, TASK-MEMORY-303,
TASK-IMP-136/137/138/139, Phase 3 TASK-IMP-140 (benchmark gates). Drift protection:
sixteen benchmark gates, published at docs/verification/benchmark-gates.md.
BODY

cat > "$BODIES/benchmark-gates.md" <<'BODY'
# Benchmark gates G1-G16 — canonical reference record

The sixteen drift-prevention gates defined by the 2026-07-23 deep audit live at
docs/verification/benchmark-gates.md (published home; normative embedded copy in
docs/tasks/improvement/TASK-IMP-140-benchmark-gates-drift-protection/spec.md). One gate,
one checker, one owner: G3/G4/G5/G6/G13/G16 are enforced by
scripts/tests/test_benchmark_gates.sh (TASK-IMP-140); G1 TASK-CUO-302, G2 TASK-CUO-303,
G7/G8 TASK-SKILL-202, G9/G10 TASK-MEMORY-303, G11 TASK-CUO-304, G12 TASK-IMP-139,
G14 TASK-IMP-136 (+TASK-IMP-128 for run_all-in-CI), G15 TASK-IMP-138. The doc's status
table is the coordination surface for report-only -> enforcing flips.
BODY

cat > "$BODIES/hardening-decisions.md" <<'BODY'
# 2026-07-23 hardening-wave decisions (operator-approved)

1. Gates fail closed: an empty gate env exits RED unless explicitly acknowledged
   (CYBEROS_ALLOW_EMPTY_GATES); GREEN is never vacuous (G1).
2. HITL is mechanical: backlog-mutate refuses the two human-acceptance flips without a
   recorded verdict artifact; verdicts emit status_overridden rows (G2).
3. Route-back ceiling unified at 3 (ship-tasks §11b = api.py = CLI); the constant is
   doctrine-parsed, not remembered (G11).
4. Always-green stub workflows are deleted, never labeled: an always-green check is worse
   than no check. Nine deleted 2026-07-23 with per-file evidence
   (TASK-IMP-136 stub-disposition.md); the placeholder marker is a test failure (G14).
5. The pre-commit framework config is removed: one hook mechanism
   (core.hooksPath=.githooks), awh wired into the real hook.
6. Durable operator gate overrides live in .cyberos/config.yaml, never gates.env
   (the C1 config-wipe class; G16).
7. The audit's risk classes are tracked as R-EXT-01..07 in docs/reference/risk-register.md.
8. This record was deferred behind TASK-MEMORY-303's store repair per §12 (no writes
   below READY) — the deferral itself is part of the record.
BODY

# ---- puts through the canonical writer (chained audit rows come with each put) -------
put_one() { # $1 = slug, $2 = body file
  local slug="$1" body="$2" h d1 d2 dest
  if command -v sha256sum >/dev/null 2>&1; then h="$(printf '%s' "$slug" | sha256sum | cut -c1-4)"; else h="$(printf '%s' "$slug" | shasum -a 256 | cut -c1-4)"; fi
  d1="${h:0:2}"; d2="${h:2:2}"
  dest="memories/decisions/$d1/$d2/$slug.md"
  echo "put -> $dest"
  CY put "$dest" "$body" --kind decisions
}

put_one "2026-07-23-deep-audit-verdict"     "$BODIES/audit-verdict.md"
put_one "2026-07-23-benchmark-gates-g1-g16" "$BODIES/benchmark-gates.md"
put_one "2026-07-23-hardening-decisions"    "$BODIES/hardening-decisions.md"

echo "== chain verify + post-state =="
CY verify
CY doctor
echo
echo "== §13 end-of-response block (paste into the session close) =="
echo "  file ops: 3 puts (memories/decisions/ - audit verdict, gate index, wave decisions)"
echo "  memories read: 0"
echo "  rejections: none (path validation + ACL green; doctor READY before and after)"
echo "  token budget: n/a (shell script, no model in the loop)"
