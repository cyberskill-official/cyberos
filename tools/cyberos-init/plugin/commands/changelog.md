---
description: Show the installed CyberOS version and what changed - from the repo's vendored version stamp and the platform changelog.
argument-hint: "[repo path, default: current repo]"
---
Report CyberOS versions and recent changes for repo_root = ${1:-the current repo}:

1. Installed: read `.cyberos/VERSION` and `.cyberos/manifest.yaml` (profile, built_from_commit, built_at) and summarise them. Absent = not initialised; suggest `/init`.

2. Available: if a payload is reachable (`$CYBEROS_PAYLOAD`, a sibling CyberOS checkout's `dist/cyberos/`, or `~/Projects/CyberSkill/cyberos/dist/cyberos/`), compare its `VERSION` with the installed one and say whether an update exists (`/update` applies it).

3. What changed: if run inside a CyberOS checkout, summarise the top entries of the repo `CHANGELOG.md`. Otherwise point at the published changelog: https://cyberos.cyberskill.world/docs/reference/changelog.html (per-module changelogs live on each module's page).

Keep it to a short, factual report - versions first, then the change summary.
