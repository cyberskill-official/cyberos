#!/usr/bin/env bash
# rollout-fleet.sh - re-init every repo under the given roots from THIS payload, and migrate
# only the repos that actually carry FRs (or already publish a status page - those must not be
# left holding a stale one). Prints one line per repo; never aborts the fleet on one failure.
# Usage: bash tools/cyberos-init/rollout-fleet.sh <payload-dir> <root-dir> [<root-dir> ...]
set -uo pipefail
payload="${1:?usage: rollout-fleet.sh <payload> <root> [...]}"; shift
[ -d "$payload/cuo" ] || { echo "rollout-fleet: $payload is not an assembled payload"; exit 2; }
FAILED=0

has_frs() {   # a repo "has FRs" when a spec.md exists, or a flat FR-*.md still sits in the tree
  # NB: -print -quit, never `find | grep -q` - under `set -o pipefail` grep's early exit
  # SIGPIPEs find and the pipeline reports 141, so a LARGE corpus reads as "no FRs".
  local r="$1"
  [ -d "$r/docs/tasks" ] || return 1
  [ -n "$(find "$r/docs/tasks" -name 'spec.md' -not -path '*/_*' -print -quit 2>/dev/null)" ] && return 0
  [ -n "$(find "$r/docs/tasks" -name 'FR-*.md' -not -name '*.audit.md' -not -path '*/_*' -print -quit 2>/dev/null)" ] && return 0
  return 1
}

for base in "$@"; do
  for r in "$base"/*; do
    [ -d "$r" ] || continue
    name="$(basename "$base")/$(basename "$r")"
    if has_frs "$r"; then why="has FRs"; mig=0
    elif [ -d "$r/docs/status" ]; then why="no FRs, but publishes a status page"; mig=0
    else why="no FRs"; mig=1; fi
    printf '\n=== %s (%s) ===\n' "$name" "$why"
    # init treats a migration failure as non-fatal (a docs-render bug must not brick init), so
    # its EXIT CODE cannot be the whole verdict: scan the output too, or a renderer that crashed
    # in every repo reads as a clean fleet roll.
    out="$(CYBEROS_NO_MIGRATE="$mig" CYBEROS_OFFLINE=1 bash "$payload/install.sh" "$r" 2>&1)"; rc=$?
    printf '%s\n' "$out" | sed 's/^/  /'
    if [ "$rc" -ne 0 ]; then
      echo "  RESULT: INSTALL FAILED (rc=$rc)"; FAILED=$((FAILED + 1))
    elif printf '%s' "$out" | grep -qE 'FAILED|SyntaxError|ERROR|Cannot find|not found'; then
      echo "  RESULT: INSTALL ok BUT a step reported failure - see the log above"; FAILED=$((FAILED + 1))
    else
      echo "  RESULT: ok"
    fi
  done
done

echo
if [ "$FAILED" -eq 0 ]; then echo "rollout-fleet: all repos rolled clean"; exit 0; fi
echo "rollout-fleet: $FAILED repo(s) reported a failing step"; exit 1
