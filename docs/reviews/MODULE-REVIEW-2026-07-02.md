# CyberOS module review - 2026-07-02

Deep review of every module and task, focused on those marked done, verifying they work RIGHT NOW.
Method: fresh full gate on the Mac (workspace fmt + clippy -D warnings + 1,173 tests, web tsc + vite +
richtext smoke 25/25 - ALL GREEN), live production probes against os.cyberskill.world, and a four-way
Task-by-task sweep of all 287 spec files (574 files including .audit companions) cross-checked against the
code that exists today. Companion improvement plan: `IMPROVEMENT-PLAN-2026-07-02.md` (same directory).

## 1. What is verified working right now

Production (os.cyberskill.world, probed 2026-07-02):

- AUTH: live and healthy (/status/auth 200). Google Workspace SSO, OIDC provider for first-party apps,
  domain gate, directory, profiles. Migrations 0001-0032 accounted for, RLS boot-check in main.rs,
  audit-chain emits on every decision path. 15 done tasks verified against code.
- CHAT: live and healthy (/status/chat 200). The full native stack verified end to end: messaging core,
  threads, reactions (64-byte emoji), rich text, full emoji picker, multi-file attachments on the VPS
  volume (raw upload route live), global search (401-gated, deployed), jump windows, channel management
  (migration 0011), notification prefs (migration 0012, route live - 422 on empty body as expected),
  @-mentions + per-user notify socket, AI endpoints deployed (401), i18n VN/EN + mute + drafts + mobile
  drawer (commits ab2989d, 3b82968 - deployed; bundle index-DkYzkd4H.js serving). Migrations 0001-0012.
- AI-GATEWAY: deployed internal-only; embeddings live; /v1/status wired to the console. 20 done tasks
  verified in code (router, breaker, streaming, redaction VN/EN, cost ledger, CLI, otel...). Chat-side AI
  + translation return clean 502 until the llm profile is enabled (by design).
- WEB CONSOLE: React SPA live with dual theme, per-build SW cache stamp, PWA.
- Deploy loop: push -> hook gate -> GHCR images -> auto-deploy -> migrate.sh (globs new migrations) works;
  proven six times this week.

Built + gate-green but NOT deployed (code verified, no production presence - this is by design but the
roadmap must say so): mcp-gateway, memory (service; its migrations DO apply on deploy), email, proj,
skill-broker, obs-proxy/router/collector/compliance-view, plugin-host, cuo (python), embed-sidecar
variants. Pure-draft modules (no code): crm, doc, esop, hr, inv, kb, learn, okr, plugin, portal, res,
rew, ten, time.

## 2. Findings

### A. Live production issues (found by probing; corrected after reading deploy.sh)

1. EVAL 502 = INTENTIONAL (finding downgraded): deploy.sh stops eval unless DEPLOY_EVAL=1 (the small
   Supabase pooler tier cannot spare its connections next to auth + chat), and status.html already treats
   it as "Not deployed until first healthy". No action beyond documenting this in the review; eval
   redeploys with DEPLOY_EVAL=1 + BUILD_EVAL=1 when the pooler is raised and counsel clears.
2. STALE CADDY CONFIG IN PROD (real; root cause = the Docker file-bind inode gotcha): /status/ai returned
   404 although Caddyfile.p0 defines it. The Caddyfile is a FILE bind into the caddy container, and file
   binds follow the inode - `git pull` replaces the file with a new inode, so the running container kept
   reading the OLD content forever; reload and even restart re-read the stale inode. (First hypothesis -
   swallowed reload errors - was disproved live: a successful deploy with reload+restart still served the
   old config.) FIXED: deploy.sh force-recreates the caddy container whenever Caddyfile.p0 changed in the
   pulled range (re-resolving the bind), and keeps the reload for same-inode edits; Caddyfile.p0 carries a
   comment so any future editor knows. Verified live after deploy: /status/ai routes.

### B. Status integrity - the roadmap's "Done" is wrong in BOTH directions

False dones (marked done, deliverable does not exist in the current architecture):
- TASK-CHAT-003..012 (10 tasks): Fargate deployment, PGroonga VN search, memory bridge, Slack import, Zalo
  import, Lumi mention, retro capture, Signal decommission, mobile push, DSAR export - ALL are
  Mattermost-era specs. The Mattermost fork was retired (TASK-CHAT-101 supersedes wholesale); none of these
  exist in the native chat. They must flip to `superseded` (or their intents re-homed as new native-chat
  tasks - Slack/Zalo import, mobile push, and DSAR are still WANTED features, just unbuilt).
- TASK-MEMORY-104 (tauri app): CORRECTED - the app exists at services/memory/desktop (Svelte + Tauri 2);
  the sweep looked for apps/tauri. Verified done; carries two of the dependabot vite alerts (dev-server
  class, same ledger as apps/web).

Stale statuses (built + live, still marked draft/implementing):
- TASK-CHAT-101 "implementing" -> the native chat is the flagship live module; flip to done.
- TASK-CHAT-013 "implementing" -> obsolete (Mattermost OIDC config for a retired fork); close superseded.
- TASK-AUTH-110 marked draft -> the OIDC provider is LIVE IN PRODUCTION; flip to done.
- TASK-CUO-204 marked draft -> dream-loop envelope built; flip (ready_to_implement/done per gate).
- TASK-AI-003, TASK-AI-005 "ready_to_implement" -> memory-writer bridge + policy loader shipped; flip to done.
- TASK-APP-001..007 draft/implementing -> the six console tiles WERE built (static console) and then
  superseded by the React SPA (apps/web). Close as superseded-by-react-console with a pointer, or re-home
  as "React console parity" tasks.
- TASK-EVAL-001 "implementing" -> code built + gated; blocked on counsel, and the prod container is down.
  Status should be `built_blocked_on_legal` (or equivalent) + a note.
- TASK-MCP-004: MCP-STATUS-FLIP.md says "flip on green" if the redirect-host allowlist deferral is
  accepted; decision pending - either flip with the deferral ledgered or keep implementing.

Metadata corruption:
- TASK-SKILL-111..115 carry TWO status lines each (done + needs_human/fixed); TASK-PROJ-012 has done + draft.
  These explain the odd one-off statuses in the corpus (needs_human x7, completed/delivered/fixed/ready).
  One canonical `status:` per file, values from a fixed vocabulary.

### C. Spec-vs-code drift (code works; the done task describes a different shape)

- memory: TASK-101/102/108 still name Apache AGE (removed - relational l2_edge + pgvector now); ~9 tasks
  promise Rust services that landed in python modules/memory or consolidated crates; TASK-108 promises a
  modular search/ tree, shipped as search.rs monolith.
- skill: TASK-101/102 promise skill-host + skill-registry services; everything lives in skill-broker.
- proj: TASK-014..017 reference web/proj-client/, real path apps/web/src; TASK-001 promises split handlers.
None of these are functional problems; they make done tasks unauditable. Fix with a short "as-built" note
per task (not a rewrite).

### D. Risks

1. OBS-007 (alert triage): the local model fabricates a runbook URL by copying the SKILL.md example -
   known, unremediated; must be fixed (URL allowlist/validation) before obs-router ships.
2. Dependabot: 11 vulnerabilities on the default branch (4 high) - untriaged.
3. Backups: chat attachments now live in the `chat-attachments` Docker volume and Supabase holds the DBs;
   there is no documented backup/restore for the volume (a VPS loss = attachment loss). Same for Caddy
   certs (re-issuable) - the attachments are the real exposure.
4. Single-VPS SPOF: accepted for team scale, but worth one line in the deploy doc (RTO = redeploy time).
5. In-process fan-out (chat Hub/Notifier) pins chat to one container - already ledgered in code comments;
   fine at team scale, listed here so scaling work has a name.
6. Deferred MCP DB-slice items (worker pool, NATS, sweepers, rate limits, PRM drift table) are honestly
   ledgered in MCP-STATUS-FLIP.md - keep them there; do not let TASK-MCP-005..008 flip until landed.
7. Parallel-session commits ab2989d/3b82968 (i18n, prefs, drafts, drawer) passed the gate and are live,
   but have had no second-pair-of-eyes review - a quick review pass is cheap insurance.
8. AI features (translate/summarize/replies) stay dormant until the VPS resize + llm profile - users see
   "unavailable"; roadmap should show "awaiting infra", not "broken".

### E. Documentation staleness

- docs/tasks/BACKLOG.md: written 2026-05-19, pre-APP/EVAL/chat-native era.
- docs/tasks/remaining-build-plan.md: 2026-06-24, mislabels mcp, omits app/eval waves.
- docs/CONTINUE-HERE.md: current through the chat waves (kept updated), but does not reflect this review.
- docs/roadmap.html: renders task frontmatter -> inherits every status error above; fixing statuses fixes
  the roadmap.

### F. Orphans + cleanup candidates

- services/chat-legacy-mattermost/: the retired fork, outside the cargo workspace - dead weight; archive
  or delete (license-watcher rationale died with the fork).
- services/eval_writetest/: throwaway crate outside the workspace - delete.
- services/business-suite/: outside the workspace members - confirm intent (scaffold?) or remove.
- 287 `.audit.md` companions: half the files in docs/tasks; they were one-time review
  artifacts. Move to docs/tasks/_audits/ (or delete) so the backlog reads clean.
- Superseded Mattermost-era chat tasks + closed TASK-CHAT-002 + app-era tasks: move to docs/tasks/
  _archive/ per the grooming rule (backlog = only not-yet-implemented + new).
