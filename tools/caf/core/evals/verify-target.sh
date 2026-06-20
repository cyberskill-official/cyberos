#!/usr/bin/env bash
# verify-target.sh — the Phase 5 TARGET HEALTH GATE (AUDIT.md v1.5.0).
#
# Strictly verifies that an AUDITED TARGET still runs without error: it reads the
# target's own RUN_COMMANDS (build / lint / typecheck / test) from its
# audit-profile.yaml (or AUDIT.md CONFIG), runs each end-to-end with a timeout,
# and exits NON-ZERO if any command fails or hangs. Print the result into the
# run's HANDOFF as `Target health: PASS` / `Target health: FAIL — <what>`.
#
# Why this exists: an audit can leave a target green in the agent's head but red
# in the target's own CI (e.g. a lint fix scoped to src/ that missed tests/, or a
# change that breaks the build). The artifact-validator cannot run builds, so this
# is the executable half of the gate. (FAILURE_LOG 2026-06-13: kymondongiap CI
# lint failure shipped because the target's own RUN_COMMANDS were never run.)
#
# Usage:   core/evals/verify-target.sh <target-dir> [--timeout SECONDS]
# Exit:    0 = every RUN_COMMAND passed; 1 = a command failed/timed out; 2 = usage.
# Env:     VERIFY_TIMEOUT (per-command seconds, default 600).

set -uo pipefail

TARGET="${1:-}"
TIMEOUT="${VERIFY_TIMEOUT:-600}"
shift || true
while [ $# -gt 0 ]; do
  case "$1" in
    --timeout) TIMEOUT="$2"; shift 2;;
    *) shift;;
  esac
done

if [ -z "$TARGET" ] || [ ! -d "$TARGET" ]; then
  echo "usage: verify-target.sh <target-dir> [--timeout SECONDS]" >&2
  exit 2
fi

# Locate RUN_COMMANDS: prefer the target's audit-profile.yaml config:, else AUDIT.md CONFIG.
RC=""
for src in "$TARGET/audit-profile.yaml" "$TARGET/AUDIT.md"; do
  [ -f "$src" ] || continue
  RC="$(grep -E '^[[:space:]]*RUN_COMMANDS:' "$src" | head -1 | sed -E 's/^[[:space:]]*RUN_COMMANDS:[[:space:]]*//')"
  # strip an inline comment that starts at >=2 spaces before '#'
  RC="$(printf '%s' "$RC" | sed -E 's/[[:space:]]{2,}#.*$//')"
  [ -n "$RC" ] && { echo "RUN_COMMANDS (from $(basename "$src")): $RC"; break; }
done
if [ -z "$RC" ]; then
  echo "TARGET-HEALTH: no RUN_COMMANDS found in audit-profile.yaml / AUDIT.md at $TARGET" >&2
  exit 2
fi

# Portable per-command timeout: timeout > gtimeout > perl-alarm fallback.
_run() {
  local secs="$1" cmd="$2"
  if command -v timeout >/dev/null 2>&1; then timeout "$secs" bash -c "$cmd"
  elif command -v gtimeout >/dev/null 2>&1; then gtimeout "$secs" bash -c "$cmd"
  else
    perl -e '
      my $s=shift; my $pid=fork;
      if(!defined $pid){exit 127}
      if($pid==0){exec "bash","-c",$ARGV[0] or exit 127}
      local $SIG{ALRM}=sub{kill 9,$pid; waitpid $pid,0; exit 124};
      alarm $s; waitpid $pid,0; exit($?>>8)
    ' "$secs" "$cmd"
  fi
}

FAILED=0
FAILED_CMDS=()
# Split RUN_COMMANDS on ';' (the profile separator) and run each from the target dir.
IFS=';' read -ra CMDS <<< "$RC"
for raw in "${CMDS[@]}"; do
  cmd="$(printf '%s' "$raw" | sed -E 's/^[[:space:]]+//; s/[[:space:]]+$//')"
  [ -z "$cmd" ] && continue
  echo ""
  echo "::: \$ $cmd   (timeout ${TIMEOUT}s)"
  if ( cd "$TARGET" && _run "$TIMEOUT" "$cmd" ); then
    echo "::: PASS — $cmd"
  else
    rc=$?
    if [ "$rc" -eq 124 ]; then echo "::: TIMEOUT (${TIMEOUT}s) — $cmd"; else echo "::: FAIL (exit $rc) — $cmd"; fi
    FAILED=1
    FAILED_CMDS+=("$cmd")
  fi
done

echo ""
if [ "$FAILED" -eq 0 ]; then
  echo "Target health: PASS — all RUN_COMMANDS passed"
  exit 0
else
  echo "Target health: FAIL — ${#FAILED_CMDS[@]} command(s) red: ${FAILED_CMDS[*]}"
  exit 1
fi
