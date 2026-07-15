# CyberOS deep audit and auto-evolution plan (2026-07-06)

Purpose: a full-repo investigation of CyberOS with concrete recommendations to strengthen the platform, ending in a staged program that turns today's operator-driven loop into a system that fine-tunes and evolves itself. Method: three targeted audits (architecture and services; autonomy and evolution infrastructure; devops, security, observability), a verification pass that re-checked every load-bearing claim against the working tree, and outside research on self-improving agent systems (sources at the end). Recommendations are numbered R1-R52 so they can be referenced, turned into tasks, or fed to the backlog engine directly.

## 1. Where the project stands

The numbers, verified 2026-07-06 on `main`:

- 100,037 lines of Rust across 465 files in `services/`, 18 workspace crates (13 services + 5 shared crates). P0 stack (auth, chat, eval, Caddy) is live at https://os.cyberskill.world; 8 images in `deploy/vps/docker-compose.p0.yml`.
- 613 markdown files under `docs/tasks/`; by frontmatter grep: 141 draft, 113 done, 13 implementing, 5 ready_to_implement, 1 blocked, 20 superseded.
- 17 GitHub Actions workflows, including domain-specific gates (awh-gate, rls-property-gate, cache-isolation-gate, obs-correlation-gate, vn-pii-recall, zdr-staleness-check).
- 49 migration files across services mention ROW LEVEL SECURITY; auth alone carries 32 migrations.
- `modules/`: 26 directories, but roughly 88% are spec-only. Real code: memory (~28k LOC Python, 42 test files), cuo (~16k LOC, 20 tests), skill (small stub).
- The dream loop (TASK-CUO-204) is armed: `modules/cuo/config/dream.yaml` has `enabled: true`, `mode: propose`, a 30-minute idle window, hard caps (5 changes per window, 600 s wall clock), an allowlist limited to SKILL.md/workflow/threshold files, a denylist covering auth, audit, RLS, PII, cost ledger, secrets, deploy and tooling paths, plus a `CYBEROS_DREAM_KILL` env kill switch. Auto-apply requires mode `auto` AND a runtime `--allow-auto-apply` flag: triple-locked.
- `.awh/` evidence logs exist and are small but real: 7 evolution-log entries, 136 promotion-log entries.
- `apps/web` (Vite + React + TS) is ~5.5k LOC with `strict: true` TS and zero test tooling (no vitest, no Playwright, no test script in package.json).

Read together: the platform is production-real, unusually well documented for its age, and already owns the three hardest assets for self-evolution (a machine-readable backlog, a deterministic promotion gate, and a tamper-evident memory/audit spine). What it lacks is measurement: production telemetry, outcome scoring, and eval suites that would let the loop learn from what it ships.

## 2. What is already strong

Keep and build on these; several are ahead of industry practice.

- Task discipline. Uniform frontmatter (id, status, priority, depends_on, effort, ai_authorship, eu_ai_act_risk_class) makes the backlog machine-plannable; `backlog_reader.py` already computes dependency cones, and `scripts/rebaseline_task_status.py` keeps BACKLOG.md honest.
- The awh gate. `scripts/caf_gate.sh` + `scripts/awh_ai_gate.sh` give a deterministic testing-to-done gate with sealed per-task audits under `docs/tasks/_audits/`, and outcomes append to immutable JSONL logs in `.awh/`.
- The memory protocol. `AGENTS.md` is a normative, RFC-2119 spec for a hash-chained, lock-disciplined, ACL-guarded store, with dreaming semantics (snapshot, precondition on body hash, audit rows) already specified. Decision ledger entries (DEC-xxxx) persist rationale across sessions.
- A fenced self-modification loop. The dream loop's enablement ladder, envelope, caps and kill switch match current published safety practice for self-evolving agents better than most research prototypes do.
- Security groundwork. NIST-aligned password policy, bcrypt, OIDC with JWKS (rotation refetch shipped), non-root slim containers, wide RLS coverage, per-IP and per-account login rate limits, nightly attachment backups, tracked env files limited to `*.example`.
- CI breadth. Path-filtered Rust gates plus unusual domain gates (RLS property tests, PII recall, ZDR staleness) show the gate-first culture the evolution program needs.

## 3. Recommendations

### 3.1 Architecture and code health

- R1. Unify the error envelope. At least three response shapes exist across services; extract one error type + IntoResponse into a shared crate and migrate service by service. Agents and clients both pay for the inconsistency today.
- R2. Extract a `shared/cyberos-service-kit`: axum bootstrap, DB pool init, config parsing, JWT/JWKS verify middleware, the `*_DEV_CORS` toggle, health endpoint, and (R38) metrics. Every service currently re-implements this by copy, which multiplies review surface and drift.
- R3. Wire the three cloud router adapters in ai-gateway (`services/ai-gateway/src/router/{openai,anthropic,bedrock}.rs`, each with a line-27 TODO). Local models are proven; the evolution program needs frontier-model quality on tap, behind the existing alias map and spend caps.
- R4. Chat realtime is an in-process hub, so the service is single-instance by construction. Add a fanout seam now (Postgres LISTEN/NOTIFY is enough at current scale; Redis pub/sub later) so scaling is a config change, and move the login rate-limit state behind the same seam when that day comes (R24).
- R5. Burn down `unwrap()`/`expect()` in request paths: rough grep (tests included) shows auth 99, ai-gateway 158, mcp-gateway 113, memory 32, chat 21, eval 6. Adopt `clippy::unwrap_used`/`expect_used` deny at crate level with explicit test allowances, and fix the worst offenders first.
- R6. Remove the ~16 panic sites flagged in obs-proxy and mcp-gateway hot paths; a panicking proxy is an outage amplifier.
- R7. Finish `shared/cyberos-audit-chain` (currently ~129 LOC) to fully match the AGENTS.md chain spec, and property-test it. It is the trust anchor for memory, eval and the BRAIN; it should be the best-tested code in the repo.
- R8. Generate OpenAPI per service (utoipa) from the axum routes and publish to the wiki. This enables typed clients, contract tests (R14) and an API-drift gate.
- R9. Resolve the spec-only Python modules honestly: mark them `spec` in a manifest (they already serve as excellent specs) rather than leaving 20+ importable-looking packages that contain no code. The task catalog remains the source of truth.
- R10. Write a short API versioning and deprecation policy (all routes are /v1 today); the moment a second consumer appears (mobile, partners), silent breaking changes become incidents.

### 3.2 Testing and quality gates

- R11. Measure coverage (cargo llvm-cov) per crate in `services.yml`, record a baseline, and ratchet: fail CI only on regression, not on an absolute bar. Coverage is currently invisible.
- R12. Give `apps/web` a test spine: vitest for lib/components (richtext.ts already has a smoke script to fold in) and one Playwright flow (login -> dashboard -> open channel -> send -> see it live). This automates exactly the browser proof that past sessions had to do by hand and that the CiC extension hang repeatedly blocked.
- R13. Ban defensive asserts. The memory-writer contract bug shipped because a test asserted `processed == 3 || failed > 0` (docs/KNOWN-ISSUES.md #1). Grep-audit for or-conditions inside asserts and add a review rule: a test must fail when the feature is broken.
- R14. Add cross-service contract tests: auth JWKS consumed by chat and mcp-gateway; the ai-gateway -> memory writer stdin/stdout contract (which has already bitten once); console/web clients against OpenAPI (R8).
- R15. Extend the RLS property gate to chat and memory tables and add a scheduled cross-tenant probe against staging (R31): isolation should be continuously proven, not just denied to the dream loop.
- R16. Make goldensets first-class: `scripts/awh_goldenset_from_task.py` exists; standardize per-module `.awh/goldenset.yaml`, run them in awh-gate.yml on every PR, and treat every fixed gate failure as a new goldenset case. This is the seed corpus for Stage 1 below.
- R17. Add a small load/soak suite (k6) for chat ws fanout, message post, and auth token issuance; record baselines so regressions become gate failures instead of production surprises.
- R18. Pilot mutation testing (cargo-mutants) on the shared crates. The whole auto-evolution premise is "the gate catches bad changes"; mutation score is the honest measure of that.

### 3.3 Security

- R19. Add cargo-audit + cargo-deny to CI (none exist today anywhere in the repo): fail on critical RUSTSEC advisories, warn otherwise, plus a weekly scheduled run so quiet weeks still get scanned.
- R20. Anchor the audit chain externally. Periodically sign the chain head (and memory binlog tip) and publish the anchor outside the database (signed git tag, object-store write-once bucket, or public timestamp). Tamper-evidence must survive DB compromise; the planned Merkle checkpointing should be pulled forward.
- R21. Chain-integrity monitor: a nightly job that walks `l1_audit_log` and the `.cyberos/memory/store` binlog, verifies hashes end to end, and alerts on divergence. Today corruption would sit silent until a human happened to look.
- R22. Write a secrets inventory (which secret, where it lives, who rotates it, blast radius) plus a rotate-on-leak runbook. The gam updater-key leak showed the failure mode; the fix is a standing process, not a one-off.
- R23. Make dev CORS unrepresentable in prod: services refuse to boot when any `*_DEV_CORS` is set while `APP_ENV=production`. One compose typo currently stands between the team and a wide-open API.
- R24. Extend rate limiting beyond login: message post, attachment upload (the 50 MB raw-body route is a cheap DoS surface), search, and the MCP endpoint; keep state behind a seam so multi-instance does not silently break it.
- R25. Supply chain: pin GitHub Actions by SHA, emit an SBOM during image builds (syft or cargo-auditable), and sign images with cosign; the VPS pulls from GHCR on every push, so image provenance is the deploy trust boundary.
- R26. Turn on Renovate or Dependabot with the awh gate as the merge condition, so dependency freshness stops depending on operator attention.
- R27. Add secret scanning (gitleaks) to CI and the pre-push hook.
- R28. Validate the session story in code: refresh-token rotation and reuse detection, revocation latency, break-glass admin actions writing distinct audit rows. Documentation says the right things; tests should prove them.

### 3.4 CI/CD and deploy

- R29. Deploy observability to P0. The obs services exist in the repo but nothing observes production: no metrics, logs, or traces off the VPS beyond `docker logs`. Minimum viable: Prometheus + node/container exporters + Loki + Grafana in the compose, or the obs-collector wired to the same.
- R30. External uptime probes on os.cyberskill.world (auth health, chat health, ws connect) with alerts into chat and email. The system cannot self-correct what nobody notices.
- R31. Stand up staging: a second compose project on a subdomain, fed by the same GHCR images, migrated before prod. Every later autonomy step (canary, auto-apply, chaos probes) needs this rung.
- R32. Canary + auto-rollback in deploy.yml: after the roll, watch healthchecks for N minutes and revert to the previous image tag on failure. Rollback is currently a manual archaeology exercise.
- R33. Backup independence: Supabase PITR is good but is one vendor decision away from gone; add nightly pg_dump to off-site object storage, verify the existing nightly attachment backup restores, and run a quarterly restore drill with a written RTO/RPO.
- R34. Accept the single-VPS SPOF consciously: a rebuild-in-60-minutes runbook (DNS TTL low, compose + env recovery, image pull, restore) is cheap insurance; full HA can wait until usage justifies it.
- R35. Speed the loop: cargo-chef layer caching in image builds. Iteration speed is a first-order input to an evolution loop; slow deploys tax every cycle.
- R36. Emit a deploy event into the audit chain from deploy.yml (version, image digests, migrator output). Deploys are currently invisible to the BRAIN, which breaks cause-effect learning later.

### 3.5 Observability as fitness signal

- R37. Adopt OTLP export via the tracing stack in every service (one layer in the service kit, R2), collected in the compose. Correlated traces are how an agent debugs production without ssh.
- R38. `/metrics` in the service kit: request rate, latency, error rate per route; ws connection count; queue depths; LLM tokens and spend.
- R39. Define 3-5 SLOs (login p99, message-delivery latency, uptime, error budget) and alert on burn rate. These same SLOs later become fitness functions and gate criteria for auto-applied changes.
- R40. Error tracking for web and services (self-hosted GlitchTip or Sentry) so client-side breakage stops relying on user reports.
- R41. An LLM call ledger in ai-gateway: per-call model, tokens, cost, latency, caller, purpose, written to Postgres. Prerequisite for the cost circuit breaker, for eval sampling, and for fine-tuning data collection (Stage 4).

### 3.6 Data layer

- R42. Least-privilege DB roles per service with a documented Supabase schema layout; finish the known gaps (MCP_DATABASE_URL wiring, per-service DB init, a migrate one-shot in the P0 compose).
- R43. CI check: every deployed service has a migrations dir and sqlx offline data; obs-router currently has zero migrations and would fail an honest check.
- R44. Operationalize pgvector: index choice (HNSW), a rebuild job, and the L2 index-consistency check the AGENTS.md spec calls for. BRAIN Phase 2 ingestion lands on this.
- R45. Generalize retention: the eval retention sweeper exists; publish a per-table retention schedule (chat attachments, audit rows, session ledgers) aligned with the Vietnam PDPD notice and enforce it with scheduled jobs.

### 3.7 Frontend

- R46. Add apps/web typecheck + lint + (R12) tests as a path-filtered CI gate; the web app currently ships on manual discipline alone.
- R47. Consolidate client state on one store pattern with the rule "derive from the store, never re-read the DOM"; the landing page's duplicate-across-lazy-chunk bug is the recorded cost of ad-hoc state.
- R48. One fetch wrapper: retry with backoff, 401 -> refresh -> replay, offline banner. The chat client already refreshes tokens; make that behavior uniform across panels.

### 3.8 Process and knowledge

- R49. Groom the 141 draft tasks with two new frontmatter fields, `value:` and `confidence:`, then rank. The dream loop's proposal ranker (Stage 2) needs exactly this metadata to order work by expected impact.
- R50. Backfill ADRs for the big irreversible calls (own-chat-from-scratch, AGE removal, RouterBackend, unified SSO) and cross-link them to DEC ledger entries, so future agents read why, not only what.
- R51. Add a wiki-link integrity check to the docs gate: broken task cross-references rot the very corpus agents plan from.
- R52. Generate CONTINUE-HERE.md instead of hand-writing it: compose it from the decision ledger, BACKLOG deltas and recent git log at session end. The bootstrap doc is load-bearing; it must not be able to go stale.

## 4. The auto-evolution program

Target loop: propose -> implement -> verify -> deploy -> measure -> learn. Today the first three legs are real (dream loop in propose mode; AUTO_WORK sessions; the awh gate), deploy is operator-gated, and measure/learn do not exist. The staging below closes the loop from the back forward, because widening autonomy before measurement exists is how self-improving systems Goodhart themselves: current research is explicit that the quality of the checking layer, not the proposing layer, determines outcomes, and that loops should converge toward a specification rather than optimize a proxy metric.

### Stage 0 - close the measurement gap (prerequisite, ~30 days)

Ship R29, R30, R36, R37-R41. Then two bridges:

- Telemetry -> backlog. A scheduled triage job (the obs.triage-alert skill already sketches it) that converts SLO burns, error clusters and crash signatures into task drafts with evidence links, deduplicated against open tasks. Production pain becomes proposals without waiting for a human to notice.
- Gate-failure taxonomy. caf_gate and awh_ai_gate currently record HOLD/PROMOTE; extend the JSONL rows with a structured failure class (build, lint, test-name, flake, audit-conformance) so failures can be mined weekly for systemic fixes and for new goldenset cases.

### Stage 1 - eval-driven development as the spine (~60 days)

- Per-module goldensets in CI (R16), with thresholds. Deterministic evals gate code; rubric evals gate LLM artifacts.
- LLM-output regression evals: for GENIE/Lumi-produced artifacts (workflows, assessments, chat answers, triage notes), build small rubric evals scored by an LLM judge that is itself anchored by ~50 human-graded samples; re-anchor quarterly. Run nightly; post the scorecard to chat.
- Outcome scoring: add `measured_outcome:` to task frontmatter, filled 2-4 weeks after `done` from telemetry where possible (latency delta, error delta, usage) or a one-line human verdict otherwise. This is the reward signal the learn leg needs; without it the system optimizes for "gate passed", which is not the same as "problem solved".
- Fix the known eval-integrity hole first (R13); an eval suite with defensive asserts is a placebo.

### Stage 2 - widen the envelope carefully (after Stages 0-1 have run ~4 weeks)

- Proposal ranking: order dream proposals by value/confidence/risk (R49 metadata + the TASK-CUO-202 risk classifier). High-impact low-risk first; `eu_ai_act_risk_class: high` never auto-applies.
- New gates on auto-apply, beyond the existing three: an LLM spend budget per window (from the R41 ledger), a latency check against SLO baselines (a prompt change that doubles inference time currently passes), and an envelope-drift validator that replays the loop's historical actions against dream.yaml and alerts on any mismatch.
- Auto-revert: if the first gate run after an auto-applied change regresses, revert without waiting for morning review; keep `cuo.dream_reverted` rows as the learning record.
- Then flip `mode: auto` for the existing docs/skills/thresholds allowlist only. Code auto-apply comes later and in this order: tests first (the loop may add tests freely), then leaf crates, never the denylist (auth, audit, RLS, PII, cost, deploy), which is already correct.
- Deploy autonomy rides on R31/R32: staging soak -> canary -> auto-rollback, with human approval per release train at first, relaxing to auto for docs-only and skill-only changes once the rollback path has fired successfully at least once in anger.

### Stage 3 - skill library and context evolution (parallel from day 60)

The published lesson from skill-library agents is blunt: the library is the performance (Voyager's ablation lost 15x task speed without it). CyberOS already has the substrate: 104 skills as SKILL.md files, workflows per persona, and AGENTS.md as the operating context.

- Run an ACE-style loop over that substrate: a Reflector mines session transcripts (AGENTS.md §18 ledger), gate logs and DEC entries for lessons; a Curator merges compact delta entries into the relevant SKILL.md or AGENTS.md section, with dedup and aging. The memory dream detectors (TASK-MEMORY-115) already sketch the curation half; connect them.
- Paired-trajectory auditing before accepting a skill edit: run old and new skill on the same goldenset tasks and require non-regression; this is the cheap, ground-truth-free check from recent skill-evolution work and maps directly onto the awh gate.
- Grow the corpus automatically: every fixed production incident and every mined gate failure becomes a goldenset case and, where general, a skill delta. The obs-triage smoke already caught a perfect example (the local model fabricating a runbook URL from the SKILL.md sample): that failure should exist forever as a regression case.

### Stage 4 - the fine-tuning flywheel (from day 90)

Everything above generates supervised data as a by-product; this stage spends it.

- Data: accepted diffs and answers paired with their task context, harvested from the session ledger, chat, gate outcomes and the R41 call ledger. Curate per domain into ChatML; current practice puts 500-2,000 well-curated examples as the useful band for a LoRA pass, with distillation from accepted frontier-model outputs (where terms allow) filling gaps.
- Training: QLoRA adapters on a 7-8B open model (Qwen-class) via MLX on the Mac or a short-lived rented GPU; an 8B QLoRA run fits in about 10 GB VRAM and finishes in hours, so weekly refreshes are realistic.
- Evaluation and rollout: a tuned adapter must beat the base model on the module goldensets and rubric evals before it ships; ship by pointing an ai-gateway alias (for example `chat.fast`) at the adapter, keeping the alias map as the instant rollback. The alias -> model indirection already exists; it is the deployment mechanism.
- First targets, chosen for cheap wins and contained blast radius: the obs triage assistant (its known fabrication failure is exactly what tuning plus retrieval fixes), Vietnamese business correspondence in the company voice, and task drafting in house style.

### Stage 5 - governance that scales with autonomy (continuous)

- Complete BRAIN Phase 0 before capture widens: the monitoring-and-evaluation notice with employee acknowledgment is drafted but not cleared; running evaluation on captured work data without it is the plan's single biggest legal exposure (Vietnam PDPD).
- Keep humans at the irreversible edges: pushes to main, prod deploys beyond docs, envelope changes, anything touching the denylist. Record each veto or approval as a DEC entry so the policy itself has a history.
- Sign the evolution history (R20/R21): the chain that records what the system did to itself must be the most tamper-evident artifact in the company.
- Quarterly envelope review: replay the quarter's dream actions, re-derive the allowlist/denylist from incidents, and re-anchor the LLM judges against fresh human grades.

## 5. First moves

Thirty days: R19, R23, R27 (cheap, closes real holes); R29, R30, R32 (see and survive production); R12, R16 (test spine + goldensets); R41 (call ledger); the telemetry -> task bridge prototype. Sixty days: R11, R14, R20, R21, R31, R37-R39; outcome scoring live; nightly eval scorecard in chat; dream loop still in propose but emitting ranked proposals. Ninety days: `mode: auto` for the docs/skills envelope; first QLoRA pilot on obs triage; skill-curation loop running; one full restore drill completed.

## 6. Sources

Research consulted for sections 4-5:

- [Demystifying evals for AI agents - Anthropic](https://www.anthropic.com/engineering/demystifying-evals-for-ai-agents)
- [The self-improving AI agent is a production pattern now - Adaline Labs](https://labs.adaline.ai/p/self-improving-ai-agent-production-pattern)
- [Eval-driven development: build and evaluate reliable AI agents - Red Hat](https://developers.redhat.com/articles/2026/03/23/eval-driven-development-build-evaluate-ai-agents)
- [Eval-driven development - Vercel](https://vercel.com/blog/eval-driven-development-build-better-ai-faster)
- [Evaluation best practices - OpenAI](https://developers.openai.com/api/docs/guides/evaluation-best-practices)
- [The Kitchen Loop: user-spec-driven development for a self-evolving codebase (arXiv)](https://arxiv.org/pdf/2603.25697)
- [Darwin Godel Machine: open-ended evolution of self-improving agents (arXiv 2505.22954)](https://arxiv.org/abs/2505.22954) and the [reference implementation](https://github.com/jennyzzt/dgm)
- [AlphaEvolve and Darwin Godel Machines: a tale of two agent futures](https://interestingengineering.substack.com/p/the-quiet-revolution-was-louder-in)
- [Agentic Context Engineering: evolving contexts for self-improving language models (arXiv 2510.04618)](https://arxiv.org/html/2510.04618v1)
- [Voyager and skill-library agents guide](https://aiunderstanding.org/learn/voyager-and-skill-library-agents)
- [SkillForge: forging domain-specific, self-evolving agent skills (arXiv)](https://arxiv.org/pdf/2604.08618)
- [SkillAudit: ground-truth-free skill evolution via paired trajectory auditing (arXiv)](https://arxiv.org/pdf/2606.14239)
- [Memory for autonomous LLM agents: mechanisms, evaluation, frontiers (arXiv)](https://arxiv.org/html/2603.07670v1)
- [Self-improving AI agents: the 2026 guide - o-mega](https://o-mega.ai/articles/self-improving-ai-agents-the-2026-guide)
- [Fine-tuning LLMs in 2026: LoRA, QLoRA, Unsloth, MLX - Codersera](https://codersera.com/blog/fine-tuning-llms-complete-guide-2026/)
- [How much data do you need to fine-tune an LLM in 2026 - Particula](https://particula.tech/blog/how-much-data-fine-tune-llm)
- [ICLR 2026 workshop on AI with recursive self-improvement](https://iclr.cc/virtual/2026/workshop/10000796)

Repo evidence cited throughout was verified on 2026-07-06 against the working tree at `~/Projects/CyberSkill/cyberos` (branch `main`, HEAD `6d257e9`).
