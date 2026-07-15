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

  bash install.sh [repo]     install / re-vendor CyberOS into a repo
  bash uninstall.sh [repo]   remove the vendored machine (keeps tasks + BRAIN by default)
  bash version.sh [repo]     check for a newer CyberOS; if stale, ask to run install
  bash status.sh [repo]      open docs/status/index.html in your default browser
  bash help.sh               this text

Soft update-check runs automatically whenever anything under .cyberos is used
(gates, status-page hooks, MCP, help, version, status). Day-to-day: install once, then forget.

Machine gates:
  bash cuo/gates/run-gates.sh

Channels — same commands, three front doors:
  npx cyberos <command>      install | uninstall | version | status | create | gates | mcp | help
  node mcp/cyberos-mcp.mjs   stdio MCP server (or: npx cyberos mcp)

Plugin slash commands (Claude Code): /install /uninstall /version /status /help
  plus /ship-tasks and /create-tasks

Docs: GUIDE.md · https://cyberos.cyberskill.world/docs
TXT
