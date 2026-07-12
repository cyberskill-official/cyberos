#!/usr/bin/env bash
# help.sh - root CLI entry (FR-IMP-076): what this payload/.cyberos install can do, directly
# from the shell - no agent, plugin, or MCP required. Mirrors the plugin's /cyberos:help.
cat <<'TXT'
CyberOS - root CLI (run from the payload dir or an installed .cyberos/)

  bash init.sh              vendor/refresh CyberOS into the current repo (idempotent)
  bash init.sh --check      three-value version report: installed / payload / latest
  bash update.sh            check for updates (wraps init.sh --check)
  bash update.sh --apply    apply the update (re-runs init.sh from this payload)
  bash changelog.sh         installed version, rules_sha fingerprint, changelog pointer
  bash help.sh              this text

  bash cuo/gates/run-gates.sh          machine-gate floor (build/lint/test/coverage per gates.env)
  node mcp/cyberos-mcp.mjs             MCP server, stdio (Claude Code, any MCP agent)
  node mcp/cyberos-mcp.mjs --http 8799 MCP server, remote-connector mode (agent UIs' custom
                                       connector dialogs; serve behind TLS - docs/deploy/mcp-connector.md)
  cli/bin/*.mjs                        npx channel (cyberos-init / cyberos-gates / cyberos-mcp)

Workflow surface (any MCP agent): fr_init, fr_gates, fr_status, ship_fr.
Docs: GUIDE.md (this payload), manifest.yaml (version + rules_sha + channels).
TXT
