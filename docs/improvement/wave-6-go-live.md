# Wave 6 - go-live (IMP-063..067)

Goal: operationalize `docs/deploy/go-live-guide.md` - the operator sequence that takes CyberOS from "built" to "fully on" - as trackable tasks with acceptance gates and ledger evidence, and gate the full flip on the Wave-1 safety nets. The go-live guide is the source of truth for the steps and the exact commands; these blocks add the tracking, the agent/operator split, and the cross-references to the rest of the backlog.

Read `docs/deploy/go-live-guide.md` and the runbooks it links before starting any task here. Every task in this wave has operator-only steps (accounts, secrets, VPS edits, deploys) that the agent must NOT perform - the agent authors, verifies, and hands over per the operator-only hard-stops in the `cyberos-improve-implement` skill.

---

### IMP-063: Track A - serve a chat model in P0 and verify AI end-to-end

`refs: go-live-guide Track A | prio: p0 | effort: s | deps: - | area: gateway/obs`

Context: the chat AI features (catch me up, action items, suggested replies, translate) are built but degrade to a quiet note because no model serves. Track A turns one on - either on the resized VPS (`COMPOSE_PROFILES=llm`, ollama pulls `qwen2.5:3b-instruct`) or on a separate Vultr box via `OLLAMA_ENDPOINT` (`docs/deploy/remote-llm-vultr.md`), keeping provider kind `ollama` so the residency/ZDR/cost gates still hold.

Operator (agent must not do): resize the VPS to >=8 GB, edit `.env.p0` (`COMPOSE_PROFILES=llm` or `OLLAMA_ENDPOINT`), deploy.

Agent scope:
- Pre-flight doc check: confirm the compose `llm` profile pulls the named model and that the provider stays `ollama` (residency/ZDR/cost gates intact); note any drift from the guide.
- Post-deploy verification (against the operator's live box): os.cyberskill.world/status AI group goes healthy; in chat, "catch me up", action items, suggested replies and translate each return real output; capture the request/response shape.
- Record first-call latency and box memory headroom (ties to IMP-004 once obs is up); if IMP-009 (LLM call ledger) is merged, confirm rows are written.
- Write a short "AI path verified" note under `docs/verification/` with the evidence.

Acceptance:
- [ ] Operator confirms model serving; agent confirms /status AI healthy and all four chat AI features return real output (evidence captured).
- [ ] Provider kind is `ollama`; residency/ZDR/cost gates confirmed still in force.
- [ ] Latency + memory headroom recorded; ledger rows confirmed if IMP-009 is live.

Touches: verification only (no code unless a defect is found - then file a new task); `docs/verification/`.

---

### IMP-064: Track B - desktop signing, auto-update, release verify

`refs: go-live-guide Track B (desktop) | prio: p1 | effort: m | deps: - | area: release`

Context: the web app and PWA work; Track B produces signed, installable native apps. Desktop first (simplest): optional Apple Developer account + six `APPLE_*` secrets (names in RELEASE.md), version bump in `apps/desktop/src-tauri/tauri.conf.json` and `apps/web/package.json`, tag -> release workflow builds installers + draft Release. Optional auto-update needs a `plugins.updater` block authored from the operator's public key.

Operator (agent must not do): buy/hold the Apple account, add `APPLE_*` and `TAURI_SIGNING_PRIVATE_KEY` secrets, run `cargo tauri signer generate`, cut the tag/push.

Agent scope:
- When the operator provides the updater PUBLIC key (not secret), author the `plugins.updater` block in `tauri.conf.json` and set `bundle.createUpdaterArtifacts: true`; confirm it compiles (`cargo tauri build` dry path on the Mac via Desktop Commander) - do not commit the private key or trigger a release.
- Verify release-workflow wiring: the six `APPLE_*` secret names in the workflow match RELEASE.md; the version-bump locations are correct; the draft-Release path is sound. Report mismatches; do not edit secrets.
- Document the operator's exact release steps (bump -> tag -> push) inline where useful and confirm RELEASE.md is current.
- Unsigned-first path documented as the zero-cost start.

Acceptance:
- [ ] Updater block authored from the operator's public key and compiles (evidence: build output); private key never enters the repo (IMP-003 scan clean).
- [ ] `APPLE_*` secret names reconciled against RELEASE.md; version-bump locations verified.
- [ ] A test tag on a scratch branch (or operator-run tag) produces installers + draft Release; steps recorded in the ledger.

Touches: `apps/desktop/src-tauri/tauri.conf.json`, `apps/web/package.json` (version only, operator-cut), `RELEASE.md`, `.github/workflows/release.yml` (verify, not secrets).

---

### IMP-065: Track B - mobile shells and store release pipeline

`refs: go-live-guide Track B (mobile) | prio: p2 | effort: m | deps: IMP-064 | area: release`

Context: mobile listings (App Store, Play) need one-time Capacitor init committed, keystore + App Store Connect API key secrets (RELEASE.md), and the repo variable `MOBILE_RELEASE=true`; a tag then builds the Android bundle and uploads to TestFlight.

Operator (agent must not do): hold Apple/Google developer accounts, add keystore + ASC secrets, set `MOBILE_RELEASE=true`, cut the tag.

Agent scope:
- Run the one-time Capacitor init locally on the Mac (`@capacitor/core|cli|ios|android`, `npx cap add ios && npx cap add android`), commit the generated `android`, `ios`, `capacitor.config.ts`, `package.json` on a branch; confirm the shells build in CI without signing.
- Verify the mobile release jobs are gated on `MOBILE_RELEASE` and that secret names match RELEASE.md; report gaps.
- Document the store-submission steps that remain operator-only (accounts, review submission).

Acceptance:
- [ ] Capacitor shells committed and building unsigned in CI (evidence).
- [ ] Mobile jobs correctly gated on `MOBILE_RELEASE`; secret names reconciled with RELEASE.md.
- [ ] Remaining operator-only store steps documented.

Touches: `apps/web/` (Capacitor config + generated `ios`/`android`), `.github/workflows/release.yml` (verify), `RELEASE.md`.

---

### IMP-066: Track C - brain activation rollout

`refs: go-live-guide Track C, Stage 5 | prio: p0 | effort: m | deps: IMP-061 | area: governance/data`

Context: the engineering half of turning on the company brain. Governance comes first (IMP-061 handles the notice review + acknowledgment mechanism); this task deploys eval+memory, gives the DB headroom, publishes the notice, collects acks, and flips capture - in the exact order of the guide, because the consent gate denies until a person has acknowledged. Detail in `docs/deploy/brain-capture-activation.md`.

Operator (agent must not do): edit `.env.p0` (`EVAL_DATABASE_URL`, `MEMORY_DATABASE_URL`, `DEPLOY_EVAL=1`, `DEPLOY_MEMORY=1`, later `CHAT_AUDIT_DATABASE_URL` + `CAPTURE_ENABLED=true`), raise the Supabase pooler limit, deploy, and `POST /v1/eval/notice` with the finalized text.

Agent scope:
- DB headroom check (pairs with IMP-051): compute the connection-pool math for auth+chat+eval+memory against the Supabase pooler limit; recommend per-service pool sizes or the pooler raise; do not change prod.
- Migration reconciliation: verify the eval and memory migrations reconcile the shared `l1_audit_log` before deploy (this is the known sharp edge); produce the exact deliberate-apply order.
- Provide the exact commands for `POST /v1/eval/notice`, `GET /v1/eval/notice` (confirm), and `POST /v1/eval/ack` per employee.
- Post-flip verification: interaction-event rows appear for an acknowledged test user and none for an unacknowledged one (the guide's step 7); capture both cases as evidence.
- Parks in `review` until the operator has counsel clearance (IMP-061) and has run the flips.

Acceptance:
- [ ] Pool-math recommendation recorded; migration apply-order for eval+memory (with `l1_audit_log` reconciliation) documented and dry-verified where possible.
- [ ] Notice publish/confirm and ack commands provided and tested against a non-prod instance.
- [ ] Capture verified: rows for an acknowledged user, none for an unacknowledged one (both shown).
- [ ] DEC entry recorded by the operator for the capture flip.

Touches: `docs/deploy/brain-capture-activation.md` (verify/annotate), `deploy/vps/.env.p0.example` (document only), verification docs; live flips are operator-run.

---

### IMP-067: go-live readiness gate (safety nets before fully on)

`refs: go-live-guide (reconciliation with the audit) | prio: p0 | effort: xs | deps: - | area: process`

Context: the go-live guide opens with "the engineering is done" - true for the three feature tracks, but the audit found the production-safety layer is not yet in place (no observability on P0, no external uptime probes, no canary/rollback, no independent backups). This task is the one-page readiness gate that reconciles the two: it does not block turning on a single feature, but it defines what "fully on for the team, unattended" requires, so the operator flips with eyes open.

Scope:
- Author `docs/deploy/go-live-readiness.md`: a checklist mapping each go-live track to its recommended safety-net precondition -
  - Track A (AI serving): obs deployed (IMP-004) so model memory/latency is visible; LLM call ledger (IMP-009) for cost visibility.
  - Any track (unattended operation): external uptime probes (IMP-005), canary + auto-rollback in deploy (IMP-006), backup independence + one restore drill (IMP-046).
  - Track C (brain/PDPD): consent complete (IMP-061), retention schedule (IMP-054).
- Each item: status (from BACKLOG), why it matters for that track, and a "safe to proceed without?" verdict (yes for single-feature trials, no for unattended full-on).
- Add a short additive pointer at the top of `docs/deploy/go-live-guide.md` to this readiness gate (do not rewrite the operator's guide; one linked sentence).
- Keep it a living checklist: it references BACKLOG statuses rather than duplicating them.

Acceptance:
- [ ] Readiness gate doc merged; every go-live track has its safety-net preconditions listed with BACKLOG cross-refs.
- [ ] One-line pointer added atop the go-live guide (additive, non-destructive).
- [ ] Operator has reviewed the "safe to proceed without?" verdicts (recorded in ledger).

Touches: `docs/deploy/go-live-readiness.md`, `docs/deploy/go-live-guide.md` (one-line addition).
