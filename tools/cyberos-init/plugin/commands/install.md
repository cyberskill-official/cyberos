---
description: Install CyberOS into the current repo (self-hosting — gate autodetect, .cyberos/ machine, FR backlog, agent entry). Idempotent re-vendor; also used when the user accepts an update from /version.
argument-hint: "[repo path, default: current repo]"
---
Install CyberOS into repo_root = ${1:-the current repo}.

1. Locate a payload: `$CYBEROS_PAYLOAD`, sibling `dist/cyberos/`, or `~/Projects/CyberSkill/cyberos/dist/cyberos/`. Run `bash <payload>/install.sh <repo_root>` and report output.

2. install does: vendor machine, gates, backlog seed, FR migrate + status page, pre-commit auto-sync, agent surface (root `AGENTS.md` → `.cyberos/AGENT-ENTRY.md`; memory protocol only at `.cyberos/memory/AGENTS.md`), managed `.gitignore`.

3. After install: day-to-day soft update-checks run on any `.cyberos` use. Manual check: `/version`. Open status page: `/status`. Remove: `/uninstall`.

4. There is no `install --page` or `install --check`. Re-vendor when the user wants an update is just install again.
