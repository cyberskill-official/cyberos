---
id: TASK-IMP-080
title: "Served-bundle version drift — live site announced v0.1.0 after the 1.0.0 release; refreshed bundle + version-sync gate coverage for apps/console/web"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: improvement
created_at: 2026-07-13T00:00:00+07:00
department: engineering
author: @stephencheng
template: task@1
module: improvement
priority: p0
status: done
verify: T
phase: "Wave 6 - go-live (web channel)"
owner: Stephen Cheng (CTO)
created: 2026-07-13
shipped: 2026-07-13
memory_chain_hash: null
related_tasks: [TASK-IMP-068, TASK-DOCS-007]
depends_on: []
blocks: []
source_pages:
  - "Stephen's screenshot 2026-07-13: os.cyberskill.world topbar badge 'v0.1.0' while release #56 / deploy #129 / payload-gate #14 / version #41 were all green on the 1.0.0 tag"
  - "apps/web/src/components/VersionBadge.tsx: badge fetches /version.json at runtime; apps/web/scripts/stamp-sw.mjs writes it from root VERSION on every `npm run build`"
  - "apps/console/web/version.json (pre-fix): {build 20260712021051, version 0.1.0} - the tracked vite output the VPS serves via git pull, last rebuilt before the 1.0.0 pin; CI rebuilds the web app fresh for the mobile shells but never recommits this dir"
source_decisions:
  - "2026-07-13 Stephen: 'all CI green but live site (os.cyberskill.world) still show v0.1.0?'"
language: bash (one gate check), generated assets (vite rebuild)
service: apps/console + tools/cyberos-init
new_files: []
modified_files:
  - apps/console/web/version.json
  - apps/console/web/sw.js
  - tools/cyberos-init/check-version-sync.sh
effort_hours: 1
subtasks:
  - "Rebuild apps/web (tsc + vite + stamp-sw) -> apps/console/web at 1.0.0, fresh sw cache id - DONE (js hashes unchanged: badge reads version.json at runtime)"
  - "check-version-sync.sh check 7: apps/console/web/version.json .version == VERSION, message names the rebuild command - DONE, both directions tested"
risk_if_skipped: "The public product page contradicts every store listing the release wave is creating: the site says v0.1.0 while GitHub, TestFlight, and Play say 1.0.0 - and the same silent lag repeats on every future version bump, because no gate or CI leg owns recommitting the served bundle."
---
## §1
1. `apps/console/web/` (the tracked vite output the VPS serves via git pull) **MUST** be rebuilt so `version.json` carries the current VERSION - done: `{"build":"20260712204020","version":"1.0.0"}`; the new service-worker cache id makes connected clients pick up the deploy via the existing update banner (useUpdateCheck polls version.json).
2. `check-version-sync.sh` **MUST** cover the served bundle as artifact 7: `apps/console/web/version.json .version == VERSION`, failing with the exact rebuild command. This makes bundle lag loud at every pre-commit, payload-gate, version bump, and release run that already calls the gate (TASK-IMP-068 wiring).
3. The check reads the REPO copy, not the payload - the served bundle is a repo-side channel like the status page, and it rides the same gate chain rather than a new one.

*Lean profile: one gate check + one regenerated bundle; defect, fix, and both gate directions machine-verified in-session.*

## §2 — Why this shape
The badge pipeline itself was already correct (VersionBadge -> /version.json -> stamp-sw.mjs -> root VERSION); what was missing is anything that forces the TRACKED output to be regenerated after a version change. The gate converts "someone must remember to rebuild" into "the next commit fails with the command to run" - same posture as every other artifact in the sync check. A CI leg that rebuilds and ships the bundle (docs-job style, via ship.sh) would remove even that manual step; recorded in §9 as the structural follow-up rather than done here, because it changes the serving model (git-pull-tracked dir) that deploy.sh's caddy bind documents.

## §5 (run 2026-07-13)
- Rebuild: `npm run build` green (tsc + vite 142ms + stamp-sw); version.json 0.1.0 -> 1.0.0, sw cache `cyberos-shell-20260712204020`; js asset hashes unchanged. PASS
- Positive gate: `sync OK 1.0.0 across 7 artifacts`. PASS
- Negative gate: version.json temporarily set to 0.9.9 -> `DRIFT ... 0.9.9 != 1.0.0 (stale served bundle - rebuild: cd apps/web && npm run build)`, exit 10; file restored. PASS
- Sandbox note: the build needed `@rolldown/binding-linux-arm64-gnu` added to node_modules (mac-installed tree, linux sandbox) - `--no-save`, lock/manifest untouched.
- Testing pass 2026-07-13 (post gate-1 "approve all"): positive gate re-run green (`sync OK 1.0.0 across 7 artifacts`); negative gate re-run green (version.json set to 0.9.9 -> `DRIFT ... 0.9.9 != 1.0.0 (stale served bundle - rebuild: cd apps/web && npm run build)`, exit 10, file restored, `git status --short` on the file clean after restore).

## §9
- Structural follow-up: a CI leg that rebuilds apps/web on `apps/web/**`/VERSION changes and ships it like the docs job (ship.sh pattern), retiring the tracked-output model. Decide together with the deploy.sh caddy-bind implications; not blocking 1.0.0.
- The old sw cache id lingers in clients until their next poll; the update banner handles it (existing machinery, no change).

## §10
| Failure | Detection | Recovery |
|---|---|---|
| version bump without web rebuild | gate check 7 fails the next commit/CI run with the command | run the printed rebuild, commit |
| version.json deleted/mangled | check 7 explicit missing/unreadable branch | rebuild |
| rebuild forgotten AND gate bypassed | impossible via the normal chain (pre-commit + payload-gate + version.yml + release all call the gate) | any one path fires |
| badge shows stale after deploy | sw cache id changed -> update banner prompts reload | client reload |
*End of TASK-IMP-080.*
