# Wave 5 - platform and process (IMP-046..062)

Goal: operational independence (backups, rebuilds, retention), data-layer discipline, knowledge upkeep, and the governance rituals that keep autonomy accountable. Report sections: 3.4, 3.6-3.8, Stage 5.

---

### IMP-046: backup independence and restore drill

`refs: R33 | prio: p1 | effort: m | deps: - | area: deploy`

Context: Supabase PITR is one vendor decision away from gone; the attachments volume has nightly backups that have never been restore-tested.

Scope: nightly `pg_dump` (schema-per-service aware) to versioned off-site object storage (Vultr object storage or equivalent) with encryption at rest and a 30-day cycle; restore script that stands up a scratch Postgres from the latest dump; quarterly drill procedure covering both the DB and the attachments volume; document RTO/RPO actually achieved.

Acceptance:
- [ ] Two consecutive nightly dumps present off-site; sizes recorded.
- [ ] Full restore drill executed once: scratch DB + attachments mount, chat renders a restored message with its attachment.
- [ ] `docs/deploy/backup-and-restore.md` merged with measured RTO/RPO.

Touches: `deploy/vps/` (job), `scripts/`, `docs/deploy/`.

---

### IMP-047: rebuild-in-60-minutes runbook

`refs: R34 | prio: p2 | effort: s | deps: IMP-046 | area: deploy`

Context: the single-VPS SPOF is accepted for now; the compensating control is a rehearsed rebuild.

Scope: step-by-step runbook from a blank VPS: DNS (TTL pre-lowered), OS hardening basics, compose + env recovery from the secrets inventory (IMP-041), image pulls, DB restore (IMP-046), probe re-point; timed walkthrough on a scratch VPS.

Acceptance:
- [ ] Runbook merged; timed rehearsal recorded in the ledger with the actual duration.
- [ ] Every referenced credential resolvable via the secrets inventory.

Touches: `docs/deploy/rebuild-runbook.md`.

---

### IMP-048: build caching with cargo-chef

`refs: R35 | prio: p2 | effort: s | deps: - | area: ci`

Context: image builds recompile the workspace; loop speed taxes every evolution cycle.

Scope: cargo-chef layering in `services/Dockerfile` (plan + cook stages), GH Actions cache wiring, measure before/after for the auth and chat images.

Acceptance:
- [ ] Warm-cache build time cut recorded (target: at least 40% on no-dep-change builds).
- [ ] Images byte-compatible in behavior (P0 smoke unchanged).

Touches: `services/Dockerfile`, `.github/workflows/deploy.yml`.

---

### IMP-049: deploy events into audit chain

`refs: R36 | prio: p1 | effort: s | deps: - | area: deploy`

Context: deploys are invisible to the BRAIN; cause-effect learning needs them on the chain.

Scope: deploy.yml's roll step posts a signed payload (version, image digests, migrator output hash, canary result) to an admin endpoint that writes a `deploy.rolled` audit row via the standard writer; failure to record must not block the deploy (retry + alert instead).

Acceptance:
- [ ] A staging roll produces exactly one chain row with correct digests.
- [ ] Endpoint rejects unauthenticated posts (test).
- [ ] Recording failure path alerts without blocking (simulated once).

Touches: `.github/workflows/deploy.yml`, `services/` (small admin route), `deploy/vps/`.

---

### IMP-050: client and service error tracking

`refs: R40 | prio: p2 | effort: m | deps: - | area: obs`

Context: web-client breakage relies on user reports; service errors rely on log spelunking.

Scope: self-hosted GlitchTip (or Sentry) in the obs compose; browser SDK in apps/web (source maps uploaded in CI, PII scrubbing on); Rust services report panics/error events via the sentry crate behind an env flag; alert routing joins the IMP-005 path.

Acceptance:
- [ ] A seeded web error and a seeded service error both appear with symbolicated traces.
- [ ] PII scrub verified on a synthetic event.
- [ ] Alert fires to chat/email once (proven).

Touches: `deploy/vps/`, `apps/web/`, `services/shared/` hook, CI.

---

### IMP-051: least-privilege DB roles and layout doc

`refs: R42 | prio: p2 | effort: m | deps: - | area: data`

Context: services share broad credentials against Supabase; the schema layout lives in heads, not docs.

Scope: one role per service with grants limited to its schema/tables; rotate connection strings via the secrets inventory; `docs/deploy/database-layout.md` mapping schemas, owners, roles, and the `*_DATABASE_URL` matrix; close the known gaps (MCP_DATABASE_URL wiring, per-service DB init, migrate one-shot in the P0 compose).

Acceptance:
- [ ] Each P0 service connects with its own role; a cross-schema write attempt fails (tested on staging).
- [ ] Layout doc merged; known gaps closed or explicitly ticketed.

Touches: migrations/roles SQL, `deploy/vps/`, docs.

---

### IMP-052: migration discipline CI check

`refs: R43 | prio: p2 | effort: xs | deps: - | area: ci`

Context: obs-router ships with zero migrations; nothing asserts that deployed services own their schema.

Scope: CI script: every deployable service crate has a `migrations/` dir with at least one migration and, where sqlx is used, up-to-date offline data; allowlist file for intentionally stateless services with a reason string.

Acceptance:
- [ ] Check red on current obs-router until resolved (migration added or allowlisted with reason).
- [ ] Wired into services.yml.

Touches: `scripts/`, `.github/workflows/services.yml`.

---

### IMP-053: pgvector operations plan

`refs: R44 | prio: p2 | effort: m | deps: - | area: data`

Context: BRAIN Phase 2 ingestion lands on pgvector; index type, rebuild jobs and the L2 consistency check from the memory spec are undecided or unimplemented.

Scope: choose and document index strategy (HNSW parameters vs IVFFlat) for the expected corpus; implement the L2 index rebuild job and the consistency check the AGENTS.md spec describes (manifest-marked rebuilds, seqlock re-read); load-test insert/search at 10x current volume on staging.

Acceptance:
- [ ] Decision doc merged with parameter rationale.
- [ ] Rebuild + consistency check implemented and green in tests.
- [ ] Load numbers recorded.

Touches: `modules/memory/`, `services/memory/`, docs.

---

### IMP-054: generalized retention schedule

`refs: R45 | prio: p2 | effort: s | deps: - | area: data`

Context: eval has a retention sweeper; chat attachments, audit rows, session ledgers and LLM-ledger rows have no enforced schedule, and PDPD expects one.

Scope: `docs/legal/retention-schedule.md` table (data class, store, retention, legal basis, sweep mechanism); implement sweeps where missing (chat attachments orphan sweep exists via delete-purge - verify; llm_calls per IMP-009; session ledgers per §18 policy); each sweep emits an audit row.

Acceptance:
- [ ] Schedule doc merged and cross-linked from the PDPD notice.
- [ ] Every listed class has a working sweep with a test.

Touches: `services/*/src/bin/` sweepers, docs.

---

### IMP-055: spec-only module manifest

`refs: R9 | prio: p2 | effort: xs | deps: - | area: docs`

Context: ~88% of modules/ is spec-only but looks importable; agents and newcomers waste time discovering that.

Scope: `modules/MANIFEST.yaml` (module -> kind: code|spec|hybrid, entry points, test command); README badge line per module dir; the backlog engine and docs build read the manifest rather than guessing.

Acceptance:
- [ ] Manifest covers all 26 module dirs and matches reality (spot-checked against the audit's code/spec split).
- [ ] Wiki module pages show the kind.

Touches: `modules/MANIFEST.yaml`, `website/` build, module READMEs.

---

### IMP-056: API versioning and deprecation policy

`refs: R10 | prio: p2 | effort: xs | deps: - | area: docs`

Context: everything is /v1 with no written rules; the first external consumer turns silent breaking changes into incidents.

Scope: one-page policy in `docs/adrs/`: what counts as breaking, additive-change rules, deprecation window, sunset headers, how /v2 would be introduced; link from each service README.

Acceptance:
- [ ] Policy merged and linked; reviewed by operator.

Touches: `docs/adrs/`.

---

### IMP-057: frontend state and fetch consolidation

`refs: R47, R48 | prio: p2 | effort: m | deps: IMP-007 | area: web`

Context: hand-rolled stores already produced the duplicate-across-lazy-chunk class of bug elsewhere; fetch/auth/retry behavior is per-call-site.

Scope: adopt one store pattern (zustand) for cross-component state with the rule "derive from the store, never re-read the DOM"; single fetch wrapper: retry with backoff, 401 -> refresh -> replay once, offline banner, 429 handling (IMP-042); migrate screen by screen with vitest coverage from IMP-007.

Acceptance:
- [ ] No direct `fetch(` outside the wrapper (lint rule or grep gate).
- [ ] Store migration for chat screens complete; Playwright flow still green.

Touches: `apps/web/src/`.

---

### IMP-058: ADR backfill for irreversible decisions

`refs: R50 | prio: p2 | effort: s | deps: - | area: docs`

Context: the big calls (first-party chat over the Mattermost fork, AGE removal, RouterBackend, unified SSO) live in memory and DEC entries; agents need the why in-repo.

Scope: four ADRs in `docs/adrs/` (context, options considered, decision, consequences), each cross-linked to its DEC entry id; template committed for future ADRs.

Acceptance:
- [ ] Four ADRs merged, accurate to the recorded history (operator sanity-read).
- [ ] Template + index page merged.

Touches: `docs/adrs/`.

---

### IMP-059: wiki link-integrity gate

`refs: R51 | prio: p2 | effort: s | deps: - | area: docs`

Context: agents plan from the FR corpus; broken cross-references rot it silently.

Scope: extend the docs build (docs-prerender-gate.yml) with a link checker over docs/ and website/ output: internal anchors, FR id references (`FR-XXX-NNN` resolves to a file), relative paths; report file with all breaks; gate red on new breaks (baseline file for pre-existing ones).

Acceptance:
- [ ] Checker runs in CI; seeded broken link turns it red.
- [ ] Baseline established; pre-existing breaks ticketed or fixed.

Touches: `.github/workflows/docs-prerender-gate.yml`, `website/` build scripts.

---

### IMP-060: generated CONTINUE-HERE

`refs: R52 | prio: p2 | effort: s | deps: - | area: process`

Context: `docs/CONTINUE-HERE.md` is the session bootstrap and load-bearing; hand-maintenance means it can silently go stale.

Scope: `scripts/gen_continue_here.py` composing the doc from: BACKLOG deltas (FR + improvement), last 20 git commits annotated, open `review` items, latest scorecard link, and a hand-written "operator notes" include file that survives regeneration; run at session end and via a weekly schedule.

Acceptance:
- [ ] Generated output reviewed side by side with the hand-written version once; operator accepts.
- [ ] Hand-edited include survives regeneration (test).

Touches: `scripts/`, `docs/CONTINUE-HERE.md`, schedule.

---

### IMP-061: BRAIN Phase 0 consent completion

`refs: Stage 5 | prio: p0 | effort: m | deps: - | area: governance`

Context: the monitoring-and-evaluation notice (EN+VI) exists but is not cleared by counsel or acknowledged by employees; evaluation on captured work data without it is the plan's largest legal exposure (Vietnam PDPD). Operator-heavy: the agent prepares, the operator executes.

Scope (agent): assemble the counsel-review packet (notice, capture inventory - what is recorded today per the audit: sign-in events, chat; where it is stored; retention per IMP-054); draft the acknowledgment flow (signed doc or in-app acknowledgment with audit row); checklist of what stays disabled until clearance (eval image gate `BUILD_EVAL=1`, capture expansion).

Scope (operator): send to counsel, collect acknowledgments, record the DEC entry, flip gates only after both.

Acceptance:
- [ ] Packet merged under `docs/legal/` (no signed contracts in git - they stay gitignored).
- [ ] Acknowledgment mechanism implemented and tested (audit row per acknowledgment).
- [ ] Ledger records hand-off to operator; task parks in `review` until counsel clears.

Touches: `docs/legal/`, small auth/eval endpoint for acknowledgment, ledger.

---

### IMP-062: quarterly envelope review ritual

`refs: Stage 5 | prio: p2 | effort: s | deps: IMP-027 | area: governance`

Context: once auto mode exists, the envelope, judges and thresholds need scheduled human re-derivation or they fossilize.

Scope: quarterly checklist doc + calendar reminder: replay the quarter's dream actions (IMP-025 verify-envelope), re-derive allowlist/denylist from incidents, re-anchor LLM judges against ~20 fresh human grades (IMP-021), review spend/latency thresholds, write the outcome as a DEC entry; first execution scheduled for the quarter after IMP-027 flips.

Acceptance:
- [ ] Ritual doc merged under `docs/auto-work/`.
- [ ] First occurrence scheduled with the operator (recorded in ledger).

Touches: `docs/auto-work/`.
