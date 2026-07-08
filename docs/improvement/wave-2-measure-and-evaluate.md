# Wave 2 - measure and evaluate (IMP-012..022)

Goal: quality becomes measurable (coverage, contracts, chain integrity), a staging rung exists, SLOs are defined, and the evaluation layer that gates all later autonomy comes online. Report sections: 3.2-3.5 and Stage 1.

---

### IMP-012: coverage measurement and ratchet

`refs: R11 | prio: p1 | effort: s | deps: - | area: ci`

Context: test coverage is unmeasured; the evolution program needs an honest baseline before it can claim gates are strong.

Scope:
- Add cargo-llvm-cov to `services.yml` per touched crate; store per-crate baselines in `services/.coverage-baselines.toml`.
- Ratchet: fail only when a crate regresses more than 1 point below its baseline; a passing run that beats baseline updates it via a bot commit or a printed instruction.
- Publish the per-crate table in the job summary.

Acceptance:
- [ ] Baselines recorded for all 13 service crates.
- [ ] Seeded coverage regression turns CI red (prove once on scratch).
- [ ] No absolute-threshold failures (ratchet only).

Touches: `.github/workflows/services.yml`, `services/.coverage-baselines.toml`.

---

### IMP-013: cross-service contract tests

`refs: R14 | prio: p1 | effort: m | deps: - | area: testing`

Context: the ai-gateway -> memory writer stdin/stdout contract drifted once and shipped broken (docs/KNOWN-ISSUES.md #1); nothing prevents the next drift between services.

Scope:
- Contract test crate or per-service test modules covering: auth JWKS document consumed by chat and mcp-gateway verifiers (shape + rotation refetch); ai-gateway `memory_writer` subprocess protocol against the real `python3 -m cyberos.writer` (spawn it, round-trip a payload); chat notify event schema consumed by apps/web (assert against a checked-in JSON schema).
- Run in `services.yml` with the memory package installed in CI.
- Each contract gets a named owner comment: which FR defines it.

Acceptance:
- [ ] All three contracts covered; the memory-writer test fails when the module is absent (regression-proof for the known bug).
- [ ] CI green; contracts listed in `docs/verification/contracts.md`.

Touches: `services/*/tests/`, `.github/workflows/services.yml`, `docs/verification/`.

---

### IMP-014: external audit-chain anchoring

`refs: R20 | prio: p1 | effort: s | deps: - | area: security`

Context: chain integrity currently lives and dies with the database; tamper-evidence must survive DB compromise.

Scope:
- Nightly job: read the `l1_audit_log` chain head and the `.cyberos-memory` binlog tip, build an anchor record (heads, row counts, ts), sign it (age or minisign key held by operator; public key in repo), and publish to a write-once location (private git repo tag or object-storage bucket with versioning).
- Verifier script that, given an anchor and DB access, re-walks and confirms.
- Runbook: what to do when verification fails.

Acceptance:
- [ ] Two consecutive anchors produced in staging/dev; verifier passes; a manually mutated row makes it fail (prove on a scratch DB).
- [ ] Private signing key never enters the repo (checked by IMP-003 scan).
- [ ] Runbook merged under `docs/deploy/`.

Touches: `scripts/` (anchor + verify), `deploy/vps/` (schedule), `docs/deploy/`.

---

### IMP-015: nightly chain-integrity monitor

`refs: R21 | prio: p1 | effort: s | deps: - | area: security`

Context: hash-chain corruption today would sit silent until someone looked; pairs with IMP-014 (anchoring proves the past, this alerts on the present).

Scope:
- Scheduled job walking both chains end to end (or incrementally from the last verified seq, persisted in a state row) verifying hash continuity per the AGENTS.md spec.
- Alert on divergence via the IMP-005 alert path and a chat message; emit a `chain.verify_ok`/`chain.verify_failed` audit row (the monitor's own writes go through the normal writer).
- Budget: must finish under 5 minutes at current volumes; note growth plan.

Acceptance:
- [ ] Green run on real data; red run on a seeded broken copy.
- [ ] Alert delivery proven once.
- [ ] Incremental state survives restart.

Touches: `services/memory/` or `scripts/`, `deploy/vps/` (schedule).

---

### IMP-016: staging environment

`refs: R31 | prio: p1 | effort: m | deps: - | area: deploy`

Context: every later autonomy step (canary, chaos probes, auto-apply) needs a rung below production; none exists.

Scope:
- Second compose project on the VPS (or a small second instance): `staging.os.cyberskill.world` behind Caddy, same GHCR images, separate Postgres schema/project, seeded synthetic tenant only (no production data).
- deploy.yml rolls staging first on every push; prod roll proceeds only after staging health passes (reuse IMP-006 logic).
- Document the promotion flow and the data-isolation guarantee.

Acceptance:
- [ ] Staging live over TLS with seeded data; login works with a test account.
- [ ] A push rolls staging automatically; prod remains gated as today.
- [ ] Isolation statement reviewed (no prod secrets or data in staging env).

Touches: `deploy/vps/`, `.github/workflows/deploy.yml`, `docs/deploy/`.

---

### IMP-017: OTLP tracing export

`refs: R37 | prio: p1 | effort: m | deps: - | area: obs`

Context: services use the tracing crate but spans die in stdout; correlated traces are how an agent debugs production without ssh.

Scope:
- Add an OTLP export layer (opentelemetry-otlp) behind an env flag, wired in each service's tracing init (or the service kit once IMP-032 lands; do not block on it - add a small shared helper now).
- Collector in the obs compose (IMP-004) - Tempo or the OTLP-capable collector - with Grafana datasource.
- Propagate request ids across service calls (header already exists in parts; make it uniform).

Acceptance:
- [ ] A chat message send in staging produces a multi-service trace visible in Grafana.
- [ ] Export disabled by default in dev; enabled in staging/prod env files.
- [ ] Overhead measured and recorded (latency delta on the health route).

Touches: `services/*/src/main.rs` or shared helper, `deploy/vps/`, `deploy/obs/`.

---

### IMP-018: Prometheus metrics endpoints

`refs: R38 | prio: p1 | effort: s | deps: - | area: obs`

Context: host/container metrics (IMP-004) cannot see request rates, error rates, ws connections, or LLM spend; services expose no `/metrics`.

Scope:
- Shared helper (or service kit module): axum layer recording per-route request count, latency histogram, status class; `/metrics` endpoint bound to the internal network only.
- Service-specific gauges: chat ws connections and fanout queue depth; auth login attempts/limits; ai-gateway calls and cost (reads IMP-009 counters).
- Prometheus scrape config for all services in the obs compose.

Acceptance:
- [ ] All P0 services scraped; starter dashboard panels show live request rates.
- [ ] `/metrics` unreachable from the public internet (Caddy/network check).
- [ ] Cardinality reviewed (no per-user labels).

Touches: `services/shared/`, each service main, `deploy/vps/`, `deploy/obs/`.

---

### IMP-019: SLO definitions and burn-rate alerts

`refs: R39 | prio: p1 | effort: s | deps: IMP-004, IMP-018 | area: obs`

Context: without SLOs there is no definition of "production got worse", which Stage 2 needs as a gate criterion.

Scope:
- Define 4 SLOs in `docs/deploy/slos.md`: auth login p99 latency, chat message post-to-deliver p95, platform uptime (probe-based), 5xx error budget.
- Encode as Prometheus recording rules + multi-window burn-rate alerts (page at fast burn, ticket at slow burn) routed via the IMP-005 path.
- These SLO queries become the reference "fitness functions" cited by IMP-025.

Acceptance:
- [ ] Rules loaded; a forced error burst in staging fires the fast-burn alert.
- [ ] SLO doc merged with the exact PromQL for each.

Touches: `deploy/obs/`, `docs/deploy/slos.md`.

---

### IMP-020: FR outcome scoring

`refs: Stage 1 | prio: p1 | effort: s | deps: - | area: process`

Context: nothing measures whether a shipped FR solved its problem; "gate passed" is currently the only success signal, which is exactly how loops Goodhart.

Scope:
- Add optional frontmatter fields to the FR schema: `measured_outcome:` (free text + verdict better|neutral|worse|unknown) and `outcome_due:` (date, default done-date + 21 days).
- Extend `scripts/rebaseline_fr_status.py` (or a sibling `scripts/fr_outcomes.py`) to list overdue outcome reviews; weekly reminder into chat.
- Where telemetry applies, the entry cites numbers (latency delta, error delta, usage); otherwise a one-line human verdict is acceptable.
- Backfill: score the 10 most recent done FRs as the pilot.

Acceptance:
- [ ] Schema documented in `docs/feature-requests/README` (or the template file); rebaseline tolerates the new fields.
- [ ] 10 backfilled outcomes merged; overdue report runs clean.

Touches: `docs/feature-requests/`, `scripts/`.

---

### IMP-021: rubric evals for LLM outputs, anchored judge

`refs: Stage 1 | prio: p1 | effort: m | deps: IMP-008 | area: eval`

Context: GENIE/Lumi artifacts (triage notes, workflows, assessments, chat answers) have no quality regression check; the recorded obs-triage fabrication (invented runbook URL) is the canonical failure.

Scope:
- Rubric spec per artifact class under `docs/verification/rubrics/` (grounding, format, safety, usefulness; 1-5 scales with written anchors).
- Judge runner in `modules/cuo` or `services/eval` tooling: LLM judge via ai-gateway scoring sampled outputs; ~50 human-graded samples stored as the anchor set; report judge-human agreement and alert when it drifts below threshold.
- Nightly run over goldenset tasks plus a sample of fresh outputs (from the IMP-009 ledger ids where content is retrievable via the session ledger); scorecard posted to chat and written to `docs/verification/scorecards/YYYY-MM-DD.md`.
- The obs-triage fabrication case becomes a permanent regression item.

Acceptance:
- [ ] Two artifact classes covered end to end (obs triage, chat answer) with anchored judges.
- [ ] Nightly scorecard produced twice in a row without manual help.
- [ ] Judge-human agreement number reported and above the documented floor.

Touches: `docs/verification/`, `modules/cuo/`, `services/eval/` (optional storage), schedule in `deploy/vps/` or Actions.

---

### IMP-022: ban defensive asserts

`refs: R13 | prio: p1 | effort: xs | deps: - | area: testing`

Context: the memory-writer bug shipped behind `assert!(processed == 3 || failed > 0)`; a test that cannot fail is a placebo.

Scope:
- Audit: grep test code for or-patterns inside asserts (`|| .*failed`, `.is_ok() ||`, etc.); fix or delete each hit (the known one first).
- Add a lightweight lint to `scripts/caf_precommit_check.sh`: flag new or-asserts in `#[test]`/`#[tokio::test]` bodies with an allowlist comment escape (`// allow-or-assert: reason`).
- One line in the review checklist (the `cyberos-improve-review` skill carries it).

Acceptance:
- [ ] Zero unexplained or-asserts in services/ test code; each fix keeps or strengthens the original intent.
- [ ] Precommit check catches a seeded violation.

Touches: `services/*/tests/`, `services/*/src/` test modules, `scripts/caf_precommit_check.sh`.
