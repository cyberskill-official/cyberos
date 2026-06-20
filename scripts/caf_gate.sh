#!/usr/bin/env bash
# caf_gate.sh - the CyberOS code-audit gate (absorbed from CyberSkill/code-audit-framework).
#
# The code-audit analog of scripts/awh_ai_gate.sh. Runs a module's deterministic CAF floor and
# fails closed, so ship-feature-requests step 28.5 (and the pre-commit hook) can require it
# ALONGSIDE the awh test-rerun gate before an FR flips testing -> done.
#
# Two deterministic checks (no LLM, no API key):
#   1. TARGET HEALTH  - tools/caf/core/evals/verify-target.sh modules/<m>
#                       runs the module's own RUN_COMMANDS (build / lint / typecheck / test) from
#                       modules/<m>/audit-profile.yaml, fail-closed. This is the half awh does NOT
#                       run; it catches build/lint breaks, route 404s, and changed data contracts
#                       (the CCAF / kymondongiap class). REQUIRES the module's toolchain - run on a
#                       build machine, not the sandbox. Some modules need services up first (e.g.
#                       ai needs Redis at 127.0.0.1:6379).
#   2. AUDIT CONFORMANCE (only if a sealed audit exists at modules/<m>/.caf/) -
#                       code-audit-validate --run modules/<m>/.caf --fail-on High: the committed
#                       audit artefacts (BACKLOG.md / HANDOFF.md) must be conformant and carry no
#                       new High/Critical finding vs the sealed baseline. No audit yet ->
#                       target-health-only floor (generate one later with
#                       tools/caf/core/evals/run-audit.sh).
#
# Usage:  bash scripts/caf_gate.sh <module>        # e.g. bash scripts/caf_gate.sh ai
# Exit:   0 = CLEAN (gate passes); 1 = RED (gate fails - route the FR back); 2 = usage/setup error.
set -uo pipefail

M="${1:-}"
[ -n "$M" ] || { echo "usage: caf_gate.sh <module>" >&2; exit 2; }
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MOD="$ROOT/modules/$M"
CAF="$ROOT/tools/caf"
[ -d "$MOD" ] || { echo "FAIL: no module dir modules/$M" >&2; exit 2; }
[ -f "$CAF/core/evals/verify-target.sh" ] || { echo "FAIL: tools/caf not vendored (missing verify-target.sh)" >&2; exit 2; }
[ -f "$MOD/audit-profile.yaml" ] || {
  echo "FAIL-CLOSED: modules/$M/audit-profile.yaml missing - a gated module MUST declare its target health (RUN_COMMANDS)." >&2
  exit 1
}

echo "== caf-gate: $M =="
echo "-- 1/2 target health (modules/$M RUN_COMMANDS) --"
if ! bash "$CAF/core/evals/verify-target.sh" "$MOD"; then
  echo "[caf] $M -> RED (target health failed)"
  exit 1
fi

echo "-- 2/2 audit conformance --"
AUDIT=""
for d in "$MOD/.caf/docs" "$MOD/.caf"; do
  [ -f "$d/HANDOFF.md" ] && { AUDIT="$d"; break; }
done
if [ -n "$AUDIT" ]; then
  if ! PYTHONPATH="$CAF/core/evals" python3 -m code_audit_validator --run "$AUDIT" --fail-on High; then
    echo "[caf] $M -> RED (audit non-conformant or new High/Critical finding)"
    exit 1
  fi
else
  echo "  (no sealed audit at modules/$M/.caf - target-health-only floor; generate one with tools/caf/core/evals/run-audit.sh)"
fi

echo "[caf] $M -> CLEAN"
exit 0
