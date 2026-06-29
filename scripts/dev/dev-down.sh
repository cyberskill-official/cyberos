#!/usr/bin/env bash
# Stop everything dev-up.sh started.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PIDS="$HERE/.pids"
[ -f "$PIDS" ] || { echo "no .pids file; nothing to stop"; exit 0; }
while read -r name pid; do
  [ -n "${pid:-}" ] || continue
  if kill "$pid" 2>/dev/null; then echo "stopped ${name} (${pid})"; else echo "already stopped: ${name} (${pid})"; fi
done < "$PIDS"
rm -f "$PIDS"
