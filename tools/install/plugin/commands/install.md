---
description: Install CyberOS into the current repo (self-hosting — gate autodetect, .cyberos/ machine, task backlog, agent entry). Idempotent re-vendor; also used when the user accepts an update from /version.
argument-hint: "[repo path, default: current repo]"
---
Install CyberOS into repo_root = ${1:-the current repo}.

1. Locate a payload: `$CYBEROS_PAYLOAD`, sibling `dist/cyberos/`, or `~/Projects/CyberSkill/cyberos/dist/cyberos/`. Run `bash <payload>/install.sh <repo_root>` and report output.

2. For what `install` actually does (task workflow + BRAIN memory protocol, task migration, status page, pre-commit auto-sync, the full per-agent wiring table, what's tracked vs. gitignored) see `tools/install/README.md` — that file is the single reference; this command does not restate it, so the two cannot drift apart.

3. After install: day-to-day soft update-checks run on any `.cyberos` use. Manual check: `/version`. Open status page: `/status`. Remove: `/uninstall`.

4. There is no `install --page` or `install --check`. Re-vendor when the user wants an update is just install again.
