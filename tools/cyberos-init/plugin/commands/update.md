---
description: Check the repo's installed CyberOS version against the available payload and apply the update on request. Read-only until the user says apply.
argument-hint: "[repo path, default: current repo]"
---
Check and update CyberOS in repo_root = ${1:-the current repo}.

1. Read the installed version from `.cyberos/VERSION` (absent = never initialised; suggest `/init` and stop).

2. Locate an available payload (`$CYBEROS_PAYLOAD`, a sibling CyberOS checkout's `dist/cyberos/`, or `~/Projects/CyberSkill/cyberos/dist/cyberos/`):
   - Payload found: run `bash <payload>/init.sh --check <repo_root>` and report its three values verbatim - `installed=` (the repo), `payload=` (the local payload), `latest=` (the newest published release) - then act on the single `verdict=` line (FR-IMP-070):
     - `verdict=up_to_date`: say so. If the output carries the `latest=unknown` offline note, say the remote check was skipped and the answer is only as fresh as the local payload - NEVER claim "up to date" from the local-payload comparison alone when the remote check was expected.
     - `verdict=repo_stale`: an update exists for this repo. Ask the user; on yes run `bash <payload>/init.sh <repo_root>` (init IS the update - idempotent, re-vendors the machine, backs up `gates.env`, never touches BACKLOG/FRs/AGENTS.md/BRAIN) and report what changed.
     - `verdict=payload_stale`: the LOCAL payload lags the published release. Fetch the latest release payload first (the `next:` line prints the exact command - download `cyberos-payload.tar.gz` from `releases/latest/download/`, or rebuild via `build.sh` in a current checkout), THEN re-run the check and proceed as above.
   - No payload: report the installed version and offer the self-host refresh - re-copy the bundled doctrine from `${CLAUDE_PLUGIN_ROOT}/skills/ship-feature-requests/cuo/` into `.cyberos/cuo/` on request, and note that a full update (memory protocol, author/audit skills, caf/awh gates) needs the payload: the desktop app's Ops tab (Build payload, then Init) or `bash tools/cyberos-init/build.sh` in a CyberOS checkout.

3. Never modify anything before the user has said to apply.
