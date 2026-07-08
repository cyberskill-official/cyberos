# Wave 1 - see and survive (IMP-001..011)

Goal: production becomes visible, deploys become reversible, the test spine exists, and the first evolution-loop data sources start recording. Every task here is p0. Report sections: 3.2-3.5 and Stage 0.

---

### IMP-001: dependency audit in CI

`refs: R19 | prio: p0 | effort: s | deps: - | area: ci`

Context: no cargo-audit or cargo-deny config exists anywhere in the repo, so RUSTSEC advisories can ship silently.

Scope:
- Add `deny.toml` at `services/` (advisories, licenses, bans, sources sections; start permissive except advisories).
- New job in `.github/workflows/services.yml`: `cargo deny check advisories` + `cargo audit`, failing on critical/high, warning otherwise.
- Add a weekly `schedule:` trigger so quiet weeks still scan.
- Document triage flow (ignore-list with expiry comments) in the workflow header.

Acceptance:
- [ ] CI fails on a seeded known-bad advisory (test by pinning an old vulnerable crate on a scratch branch, then revert).
- [ ] Weekly schedule visible in the Actions tab.
- [ ] Zero unexplained ignores in `deny.toml`.

Touches: `services/deny.toml`, `.github/workflows/services.yml`.

---

### IMP-002: refuse dev CORS in production boot

`refs: R23 | prio: p0 | effort: xs | deps: - | area: security`

Context: `AUTH_DEV_CORS`, `CHAT_DEV_CORS`, `MCP_DEV_CORS`, `AI_GATEWAY_DEV_CORS` open permissive CORS for local dev; a compose typo could enable them in prod.

Scope:
- In each service's startup (or the shared bootstrap once IMP-032 lands): if `APP_ENV=production` (or the existing prod-detection env) and any `*_DEV_CORS` is truthy, log a fatal error naming the variable and exit non-zero.
- Unit test per service for the refusal path.
- Note the behavior in `deploy/vps/.env.p0.example`.

Acceptance:
- [ ] Each of the four services exits non-zero with a clear message when misconfigured.
- [ ] P0 compose boots unchanged (no dev flags present).

Touches: `services/{auth,chat,mcp-gateway,ai-gateway}/src/main.rs` (or config module), `deploy/vps/.env.p0.example`.

---

### IMP-003: secret scanning in CI and pre-push

`refs: R27 | prio: p0 | effort: xs | deps: - | area: security`

Context: only `*.example` env files are tracked (good), but nothing prevents the next accidental commit; the gam updater-key leak is the recorded failure mode.

Scope:
- Add gitleaks (pinned version) as a CI job on push/PR with a repo `.gitleaks.toml` (allowlist the docs that legitimately show fake keys).
- Add a gitleaks pass to `.githooks/` pre-push.
- Run once over full history; record findings (paths only) in the ledger and rotate anything real before merging.

Acceptance:
- [ ] CI job red on a seeded fake secret in a scratch commit, green on main.
- [ ] Pre-push hook blocks a seeded secret locally.
- [ ] History scan results recorded in `LEDGER.md`.

Touches: `.github/workflows/`, `.githooks/`, `.gitleaks.toml`.

---

### IMP-004: deploy observability stack to P0

`refs: R29 | prio: p0 | effort: m | deps: - | area: obs`

Context: the obs services exist in the repo but production has no metrics, logs, or traces beyond `docker logs` over ssh; the P0 compose has 8 images and none observe.

Scope:
- Add to `deploy/vps/docker-compose.p0.yml` (or a compose overlay): Prometheus + node-exporter + cadvisor, Loki + promtail (or the docker Loki logging driver), Grafana behind Caddy basic-auth on a subpath or subdomain.
- Scrape container and host metrics now; service `/metrics` endpoints arrive with IMP-018.
- One starter dashboard: host CPU/mem/disk, per-container restarts, Caddy request rate and 5xx.
- Retention capped to fit the VPS disk (7-14 days); document sizes.

Acceptance:
- [ ] Grafana reachable over TLS with auth; dashboard shows live P0 data.
- [ ] `docker compose config` valid; roll executed by the operator (agent stops before deploy).
- [ ] Disk headroom after 48 h burn-in recorded in the ledger.

Touches: `deploy/vps/docker-compose.p0.yml`, `deploy/vps/Caddyfile`, `deploy/obs/`, `docs/deploy/`.

---

### IMP-005: external uptime probes and alerting

`refs: R30 | prio: p0 | effort: xs | deps: - | area: obs`

Context: an on-box monitor dies with the box; nothing external watches os.cyberskill.world.

Scope:
- Configure an external probe service (healthchecks.io, UptimeRobot, or equivalent free tier) for: auth health endpoint, chat health, web root, ws connect (TCP probe at minimum).
- Alerts to email (info@cyberskill.world) and, where supported, a chat webhook.
- Document probe list, cadence, and alert routing in `docs/deploy/uptime-probes.md`; keep credentials out of the repo.

Acceptance:
- [ ] A deliberate 2-minute stop of the chat container triggers an alert (test in a maintenance window with operator).
- [ ] Runbook doc merged.

Touches: `docs/deploy/uptime-probes.md` (external config is operator-held).

---

### IMP-006: canary healthcheck and auto-rollback in deploy

`refs: R32 | prio: p0 | effort: s | deps: - | area: deploy`

Context: deploy.yml rolls new images and stops; a bad image stays live until a human notices, and rollback is manual archaeology.

Scope:
- In `deploy.yml`'s SSH roll step: record current image digests, roll, then poll health endpoints for N minutes (start N=3, config at top of file).
- On failure: re-tag previous digests, roll back, mark the workflow failed, print both digest sets in the job summary.
- Emit a line into the deploy log (and, once IMP-049 lands, the audit chain).

Acceptance:
- [ ] Simulated failure on staging or a scratch service (image whose healthcheck fails) auto-rolls back within the window.
- [ ] Success path adds no more than ~1 minute to deploy time.
- [ ] Rollback procedure documented inline in `deploy.yml`.

Touches: `.github/workflows/deploy.yml`, `deploy/vps/`.

---

### IMP-007: apps/web test spine

`refs: R12, R46 | prio: p0 | effort: m | deps: - | area: web`

Context: apps/web (~5.5k LOC) has zero test tooling; the richtext parser has only a standalone smoke script; browser proofs are manual today.

Scope:
- Add vitest + config; fold `apps/web/scripts/richtext-smoke.ts` into real vitest cases; add tests for stores and the fetch/auth wrapper.
- Add Playwright with one flow against the dev stack (`scripts/dev`): login (test IdP bypass or seeded session), open channel, send message, assert it renders; keep it skippable when the stack is absent.
- Add `test` scripts to package.json and a path-filtered CI job (typecheck + lint + vitest; Playwright as a nightly or labeled job).

Acceptance:
- [ ] `npm test` green locally and in CI; richtext cases ported (same assertions or stronger).
- [ ] Playwright flow green against the dev stack on the Mac.
- [ ] CI fails on a seeded richtext regression (prove once on a scratch branch).

Touches: `apps/web/`, `.github/workflows/` (new or extended path-filtered job).

---

### IMP-008: goldensets as first-class gate inputs

`refs: R16 | prio: p0 | effort: s | deps: - | area: awh`

Context: `scripts/awh_goldenset_from_fr.py` exists but goldensets are not standardized or enforced per module; they are the seed corpus for every later eval task.

Scope:
- Define `.awh/goldenset.yaml` schema (task id, input, expected, checker: exact|contains|script, threshold) in `docs/auto-work/goldensets.md`.
- Generate initial goldensets for 3 modules (chat, memory, obs) from done FRs via the existing script; hand-verify each case.
- Wire evaluation into `scripts/awh_ai_gate.sh` and the awh-gate workflow: missing goldenset = warning; failing case = gate failure.
- Rule: every fixed gate failure or production incident adds a case (enforced by review checklist, automated later via IMP-011).

Acceptance:
- [ ] Three modules carry verified goldensets; gate runs them in CI.
- [ ] A seeded failing case turns the gate red (prove once, then remove).
- [ ] Schema doc merged.

Touches: `scripts/awh_ai_gate.sh`, `scripts/awh_goldenset_from_fr.py`, `modules/*/.awh/`, `.github/workflows/awh-gate.yml`, `docs/auto-work/`.

---

### IMP-009: LLM call ledger in ai-gateway

`refs: R41 | prio: p0 | effort: s | deps: - | area: gateway`

Context: no per-call record of model, tokens, cost, latency, caller, purpose exists; the cost circuit breaker, eval sampling, and fine-tuning data all need it.

Scope:
- Migration: `llm_calls` table (ts, tenant, caller service+skill, alias, resolved model, prompt/completion tokens, cost estimate, latency ms, outcome, request id; no prompt bodies here - transcripts stay in the session ledger).
- Write path in ai-gateway's dispatch (both local adapters and, later, cloud adapters from IMP-033); failure to write must not fail the call (log + counter).
- `GET /v1/ai/usage` summary endpoint (per day, per alias, per caller) admin-scoped.
- Retention: raw rows 90 days, daily rollup kept (align with IMP-054 later).

Acceptance:
- [ ] Every dispatched call in the dev stack produces exactly one row; tests cover success, provider error, and ledger-write failure.
- [ ] Usage endpoint returns correct aggregates for seeded data.
- [ ] Gate green on `cargo clippy -p ai-gateway`, `cargo test -p ai-gateway`.

Touches: `services/ai-gateway/` (migration, dispatch, new route).

---

### IMP-010: telemetry-to-FR bridge

`refs: Stage 0 | prio: p0 | effort: m | deps: IMP-004 | area: cuo`

Context: production pain currently becomes work only when a human notices; the obs.triage-alert skill sketches triage but nothing files FRs.

Scope:
- A scheduled job (compose cron container or GH Actions schedule hitting an admin endpoint) that reads error clusters, restart counts, probe failures and (once IMP-019 lands) SLO burns from the obs stack.
- For each new signal: dedupe against open FRs (title-hash + evidence fingerprint file under `docs/feature-requests/_bridge/`), then file `docs/feature-requests/<module>/FR-<MOD>-9xx.md` with status draft, `author: "@bridge"`, `ai_authorship: generated`, evidence links (Grafana panel URL, log excerpt), and a proposed severity.
- Never assigns priority above p1 automatically; never edits existing FRs.
- Weekly digest of filed/deduped items posted to chat (reuse the notify path or a webhook).

Acceptance:
- [ ] Seeded synthetic error cluster produces exactly one draft FR with working evidence links; second run dedupes.
- [ ] Frontmatter passes `scripts/rebaseline_fr_status.py` regeneration.
- [ ] Kill switch env documented (`BRIDGE_ENABLED=0`).

Touches: `modules/cuo/` or `services/obs-router/` (job), `docs/feature-requests/_bridge/`, `deploy/vps/` (schedule).

---

### IMP-011: structured gate-failure taxonomy

`refs: Stage 0 | prio: p0 | effort: s | deps: - | area: awh`

Context: `.awh/promotion-log.jsonl` records HOLD/PROMOTE (136 entries) with no failure class, so failures cannot be mined for systemic fixes or goldenset cases.

Scope:
- Extend the JSONL row schema: `failure_class` (build | lint | typecheck | test | flake | audit-conformance | infra), `failing_target` (crate/test name), `duration_s`, `toolchain`.
- Update `scripts/caf_gate.sh` and `scripts/awh_ai_gate.sh` to classify from exit points (each already knows which step failed); unknown = `infra`.
- `scripts/awh_failure_report.py`: weekly summary (top failure classes, repeat offenders, flake suspects = same target alternating pass/fail) written to `docs/auto-work/failure-reports/YYYY-WW.md`.
- Backfill is not required; old rows stay as-is (parser tolerates both shapes).

Acceptance:
- [ ] New rows carry the fields for all three failure paths (prove with one forced failure each: fmt, clippy, test).
- [ ] Report script runs on the existing log without error and renders a sane summary.
- [ ] Row schema documented in `docs/auto-work/`.

Touches: `scripts/caf_gate.sh`, `scripts/awh_ai_gate.sh`, `scripts/awh_failure_report.py`, `docs/auto-work/`.
