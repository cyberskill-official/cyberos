---
description: Install CyberOS into the current repo (self-hosting — gate autodetect, .cyberos/ machine, FR backlog, agent entry files). Idempotent; also applies when re-run after an update.
argument-hint: "[repo path, default: current repo]"
---
Install CyberOS into repo_root = ${1:-the current repo}.

1. Full payload path (preferred). Locate a CyberOS payload: `$CYBEROS_PAYLOAD`, a sibling checkout's `dist/cyberos/`, or `~/Projects/CyberSkill/cyberos/dist/cyberos/`. If `install.sh` is found there, run `bash <payload>/install.sh <repo_root>` and report its output. install.sh does the whole job: vendor + gates + backlog + CHANGELOG seed + automatic FR migration + status page at `docs/status/` + auto-sync (pre-commit + run-gates) + agent surface (root `AGENTS.md` is a thin pointer to `.cyberos/AGENT-ENTRY.md`, like `CLAUDE.md` / `GEMINI.md`; memory protocol lives only at `.cyberos/memory/AGENTS.md`) + managed `.gitignore` block. Relay the `cyberos-migrate verify:` line and any WARNs. Done.

2. No payload: use the plugin self-host path if available, or tell the user to download `cyberos-payload.tar.gz` from GitHub Releases and run `bash install.sh`.

3. After install: day-to-day soft update checks run automatically on any `.cyberos` use. Manual check: `/update`. Manual version report: `/status`. Remove: `bash .cyberos/uninstall.sh`.

Never invent `install --page` or `install --check` — those are not user commands (status page is internal; update check is `update`).
