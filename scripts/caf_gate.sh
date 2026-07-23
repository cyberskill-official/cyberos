#!/usr/bin/env bash
# caf_gate.sh - the CyberOS code-audit gate (absorbed from CyberSkill/code-audit-framework).
#
# The code-audit analog of scripts/awh_ai_gate.sh. Runs a module's deterministic CAF floor and
# fails closed, so ship-tasks step 29 (and the pre-commit hook) can require it
# ALONGSIDE the awh test-rerun gate before a task flips testing -> done.
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
#         bash .cyberos/cuo/gates/caf/caf_gate.sh . # installed repos: "." gates the repo root
# Exit:   0 = CLEAN (gate passes); 1 = RED (gate fails - route the task back); 2 = usage/setup error.
set -uo pipefail

M="${1:-}"
[ -n "$M" ] || { echo "usage: caf_gate.sh <module-name|path>" >&2; exit 2; }
here="$(cd "$(dirname "$0")" && pwd)"
# Two layouts, one script. Source: scripts/caf_gate.sh at the platform repo root with
# tools/caf beside it. Vendored: .cyberos/cuo/gates/caf/caf_gate.sh with tools/caf copied
# to ./caf by tools/install/build.sh - there $here/.. is .cyberos/cuo/gates, NOT the repo.
if [ -d "$here/caf/core" ]; then
  CAF="$here/caf"
  ROOT="$(git -C "$here" rev-parse --show-toplevel 2>/dev/null)" \
    || ROOT="$(cd "$here/../../../.." && pwd)"   # .cyberos/cuo/gates/caf -> target repo root
else
  ROOT="$(cd "$here/.." && pwd)"
  CAF="$ROOT/tools/caf"
fi
MOD="$ROOT/modules/$M"
if [ ! -d "$MOD" ]; then
  # No modules/<m>: treat the argument as a path relative to the repo root.
  # "." (the seeded CAF_CMD default in installed repos) gates the repo itself.
  case "$M" in .) MOD="$ROOT" ;; *) MOD="$ROOT/$M" ;; esac
fi
[ -d "$MOD" ] || { echo "FAIL: no module dir modules/$M and no $M under $ROOT" >&2; exit 2; }
[ -f "$CAF/core/evals/verify-target.sh" ] || { echo "FAIL: tools/caf not vendored (missing verify-target.sh)" >&2; exit 2; }
[ -f "$MOD/audit-profile.yaml" ] || {
  echo "FAIL-CLOSED: $MOD/audit-profile.yaml missing - a gated module MUST declare its target health (RUN_COMMANDS)." >&2
  exit 1
}

echo "== caf-gate: $M =="
echo "-- 1/2 target health ($MOD RUN_COMMANDS) --"
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
  echo "  (no sealed audit at $MOD/.caf - target-health-only floor; generate one with tools/caf/core/evals/run-audit.sh)"
fi

echo "[caf] $M -> CLEAN"
exit 0
