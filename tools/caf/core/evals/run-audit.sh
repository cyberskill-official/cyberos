#!/usr/bin/env bash
# run-audit.sh — runner-mode kickoff composer (the protocol never leaves this repo).
#
#   ./core/evals/run-audit.sh /path/to/target               # print the kickoff prompt
#   ./core/evals/run-audit.sh /path/to/target claude -p     # launch via an agent CLI
#
# The target carries only audit-profile.yaml (config: + optional denylist
# packs). Copy mode (a pinned AUDIT.md inside the target) keeps working and
# wins precedence — see core/evals/README.md.

set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
PROTOCOL="$(cd "$HERE/.." && pwd)/AUDIT.md"
TARGET="${1:?usage: run-audit.sh <target-path> [agent command...]}"
TARGET="$(cd "$TARGET" && pwd)"
shift || true

[ -f "$TARGET/audit-profile.yaml" ] || [ -f "$TARGET/AUDIT.md" ] || {
  echo "ERROR: $TARGET has neither audit-profile.yaml (runner mode) nor AUDIT.md (copy mode)." >&2
  exit 2
}

PROMPT="Read the audit protocol at $PROTOCOL and execute it on the target repository at $TARGET. \
The target's CONFIG comes from $TARGET/audit-profile.yaml (config: section); PROJECT_PATH is $TARGET. \
All artifacts (docs/BACKLOG.md, docs/HANDOFF.md) belong in the target. Begin at PHASE 0."

if [ "$#" -eq 0 ]; then
  echo "$PROMPT"
else
  cd "$TARGET" && exec "$@" "$PROMPT"
fi
