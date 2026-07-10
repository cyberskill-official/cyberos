# CyberOS go-live guide

The remaining steps to take CyberOS from "built" to "fully on." Everything here is operator work - accounts,
server settings, and governance - because the feature engineering is done. There are three independent tracks;
you can do them in any order, but the order below gives the fastest visible wins first. Each track links the
detailed runbook; this page is the sequence. Where I can help mid-step, it says so.

> These three tracks are tracked as FR-IMP-063..067 in `docs/feature-requests/improvement/` (agent/operator
> split, acceptance gates, ledger evidence). One caveat before flipping everything on for unattended team use:
> the feature engineering is done, but the production-safety layer from the 2026-07-06 audit is not yet in
> place - no observability on the P0 box, no external uptime probes, no canary/auto-rollback, no independent
> backups. Turning on a single feature to try it is fine; running fully on and unattended should wait for the
> Wave-1 safety nets (IMP-004, IMP-005, IMP-006, IMP-046). IMP-067 is the readiness gate that spells this out.

## Track A: turn on the AI assistant (fastest visible win)

The chat AI features - catch me up, action items, suggested replies, translate - are built and today degrade
to a quiet note because no model is serving. Pick one way to serve one.

Option 1, run the model on the current server (simplest):

1. Resize the VPS to at least 8 GB RAM. The bge-m3 embeddings and a 3B chat model do not fit together on the
   4 GB box.
2. On the VPS, edit `~/cyberos/deploy/vps/.env.p0` and add the line `COMPOSE_PROFILES=llm`.
3. Deploy: run `bash ~/cyberos/deploy/vps/deploy.sh` (or push any change). It starts ollama and pulls the
   chat model `qwen2.5:3b-instruct` automatically, and keeps doing so on every later deploy.
4. Verify: open os.cyberskill.world/status - the AI group turns healthy - and in chat, "catch me up" now
   returns a summary.

Option 2, run the model on a separate box (keeps embeddings and the chat model apart): follow
`docs/deploy/remote-llm-vultr.md` - a second Vultr instance in Singapore on a private VPC running ollama, then
set `OLLAMA_ENDPOINT` in `.env.p0` to its private URL. Keep the provider kind as ollama so the residency, ZDR,
and cost gates still hold.

Tell me when it is up and I will verify the AI path end to end.

## Track B: sign and ship the desktop and mobile apps

The web app and the installable PWA already work. This produces signed, installable native apps.

Desktop (do this first, it is simplest):

1. Buy an Apple Developer account (USD 99/year) if you want a signed, notarized Mac app. You can ship
   unsigned to start and skip this.
2. Export your Developer ID certificate and add the six `APPLE_*` secrets in the repo under Settings ->
   Secrets and variables -> Actions. The exact names are in RELEASE.md.
3. Cut a release: bump `version` in `apps/desktop/src-tauri/tauri.conf.json` and `apps/web/package.json`, then
   `git tag vX.Y.Z && git push origin main vX.Y.Z`. The release workflow builds the installers and opens a
   draft GitHub Release.
4. Optional auto-update: run `cargo tauri signer generate`, then paste me the PUBLIC key (it is not secret)
   and I will add the `plugins.updater` block to `tauri.conf.json` and confirm it compiles. Set the
   `TAURI_SIGNING_PRIVATE_KEY` secrets and `bundle.createUpdaterArtifacts: true`, and tagged releases then
   auto-update installed apps.

Mobile (when you want App Store and Play listings):

5. One-time init locally, then commit the generated projects:

       cd apps/web
       npm i -D @capacitor/core @capacitor/cli @capacitor/ios @capacitor/android
       npx cap add ios && npx cap add android
       git add android ios capacitor.config.ts package.json && git commit -m "chore: add Capacitor shells"

6. Add the Android keystore secrets and the iOS App Store Connect API key secrets (RELEASE.md), and set the
   repo variable `MOBILE_RELEASE=true`.
7. Cut a tag as in step 3 - the mobile jobs now build the Android bundle and upload to TestFlight.

## Track C: turn on the company brain (governance first)

This starts recording platform work interactions into the brain. Do the steps in order. Skipping ahead
records nobody - the consent gate denies until a person has acknowledged - and skips the transparency the law
requires. Full detail is in `docs/deploy/brain-capture-activation.md`.

1. Finalize the notice. Review `docs/legal/data-monitoring-and-evaluation-notice.md` with Vietnamese counsel:
   set the lawful-basis wording, the retention periods, and the data contact.
2. Get database headroom. Raise the Supabase pooler limit (or lower per-service pool sizes) so memory and eval
   can run alongside auth and chat without exhausting connections.
3. Deploy eval and memory. In `.env.p0` set `EVAL_DATABASE_URL`, `MEMORY_DATABASE_URL`, `DEPLOY_EVAL=1`, and
   `DEPLOY_MEMORY=1`; apply the eval and memory migrations deliberately (reconcile the shared `l1_audit_log`
   first); deploy.
4. Publish the notice. `POST /v1/eval/notice` (founder only) with the finalized text; confirm with
   `GET /v1/eval/notice`. Tell me and I will give you the exact command.
5. Collect acknowledgements. Each employee acknowledges via a signed addendum to their agreement; record each
   with `POST /v1/eval/ack`.
6. Flip capture. In `.env.p0` set `CHAT_AUDIT_DATABASE_URL` to the brain database and `CAPTURE_ENABLED=true`;
   deploy. From here, acknowledged people's chat chains into the brain; everyone else is skipped.
7. Verify. Confirm interaction-event rows appear for an acknowledged test user and none for an unacknowledged
   one.

## Where I can help mid-track

- Verify live status, AI health, and the capture behaviour after each change.
- Author the `plugins.updater` block in `tauri.conf.json` from your public key and confirm it compiles.
- Give you the exact commands for publishing the notice and recording acknowledgements.
- Write any `.env.p0` changes for you to paste on the VPS.

Tell me which track you are on and I will take the next concrete step with you.
