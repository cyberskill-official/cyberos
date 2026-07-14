---
description: Manually check the repo's installed CyberOS version against payload/latest; apply with --apply. Soft checks already run on any .cyberos use.
argument-hint: "[repo path, default: current repo]"
---
Check (and optionally apply) a CyberOS update for repo_root = ${1:-the current repo}.

1. Soft checks already fire whenever gates, MCP tools, status-page hooks, help, or status run. This command is the **manual** check.

2. Run `bash .cyberos/update.sh` (or `bash <payload>/update.sh <repo_root>`). Report `installed=`, `payload=`, `latest=`, and `verdict=` verbatim.

3. On `verdict=repo_stale` or `payload_stale`: ask the user before applying. On yes: `bash <payload>/update.sh --apply <repo_root>` (re-runs `install.sh` from the payload; idempotent, backs up `gates.env`, never destroys BACKLOG/FRs/BRAIN).

4. On `verdict=not_installed`: suggest `/install` (or `bash <payload>/install.sh`).

5. Never modify the repo before the user says apply.
