#!/usr/bin/env bash
# help.sh — root CLI surface. Soft update-check runs on every use of .cyberos.
here="$(cd "$(dirname "$0")" && pwd)"
if [ -f "$here/lib/update-check.sh" ]; then
  # shellcheck source=/dev/null
  source "$here/lib/update-check.sh"
  _cyberos_update_check || true
fi
cat <<'TXT'
CyberOS — root CLI (payload dir or installed .cyberos/)

Once (lifecycle):
  bash install.sh [repo]     install / re-vendor CyberOS into a repo
  bash uninstall.sh [repo]   remove the vendored machine (keeps FRs + BRAIN by default)

Ongoing (auto soft-check on any .cyberos use; also manual):
  bash update.sh             check installed vs payload vs latest
  bash update.sh --apply     apply update (re-runs install from this payload)

Manual report only:
  bash status.sh             installed version + rules_sha + doc pointers
  bash help.sh               this text

Machine gates (auto update-check):
  bash cuo/gates/run-gates.sh

Channels:
  node mcp/cyberos-mcp.mjs             MCP stdio
  node mcp/cyberos-mcp.mjs --http 8799 MCP HTTP connector
  cli/bin/*.mjs                        npx channel

Status page (docs/status/) regenerates automatically via pre-commit + run-gates.
There is no separate --page / --check user command.

Docs: GUIDE.md, manifest.yaml. Site: https://cyberos.cyberskill.world/docs
TXT
