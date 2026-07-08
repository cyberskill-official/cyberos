# Phase 3 tasks - the auto-evolution loop

Gate to enter: P1 done (facts + hybrid recall live); MEM-009 runner mature enough to gate consolidation. The loop rule from report section 8 governs everything here: no consolidation, prompt, or config change reaches serving without a passing golden-set delta, and everything the loop applies is reversible.

---

## MEM-045 - Calibrated judge + golden growth + internal benchmark

refs R46, R56 | est 14h | deps MEM-009 | priority critical

Files: `services/eval` memory-judge module (gateway-called rubric judge), golden set growth to 100+ cases, benchmark runner `memory-bench` with five ability buckets (single-session extraction, multi-session reasoning, temporal reasoning, knowledge update, abstention), human-label calibration set.

Steps: write the judge rubric with few-shot anchors; label 50 recall outcomes by hand (operator assists); iterate the judge until agreement is 75-90%; freeze judge version; generate benchmark cases from CyberOS domains (chat threads, proj activity, obs incidents); report per-bucket scores per release into a tracked JSON under `docs/improvement/memory/bench/`.

Accept: judge agreement documented; benchmark runs nightly; baseline recorded.

Tests: judge-agreement meta-test against the frozen label set; benchmark determinism test.

Review (human): you label the calibration set (or delegate + spot-check); judge disagreements above threshold block the freeze.

---

## MEM-046 - Usage-signal loop

refs R47 | est 8h | deps MEM-028 | priority high

Files: `src/brain/scoring.rs` (used-ratio term), counters from MEM-028 feedback + automatic citation detection (string/id match of injected memories in agent answers, reported by callers via the feedback endpoint's `used` field), demotion job on the maintenance tick.

Steps: maintain retrieved/used counts per fact and event; used-ratio below threshold after N retrievals lowers effective importance (bounded, never to zero, always audited); surface top demotions in the weekly report.

Accept: a seeded never-used memory demotes after N retrievals; ratio visible in explain and on the ops tile.

Tests: demotion integration test; bound tests (never below floor).

Review (human): pick N and the floor; they trade recall safety against noise.

---

## MEM-047 - Dream loop on the BRAIN

refs R33, R34, R55 | est 20h | deps MEM-013, MEM-045 | priority high

Files: new `src/brain/dream/` (detectors, proposals, applier), proposal table + review queue surfaced in the console, config `BRAIN_DREAM_ENABLED=0` default, ACE-style playbook curation for the `procedural` kind.

Steps:
1. Port the four detectors from `modules/memory/cyberos/core/dream/detectors.py` to brain tables: duplicates (cosine + jaccard), stale (age + invalidation candidates), patterns (recurring topics -> playbook/fact proposals), verify (provenance spot checks).
2. Proposals land in a queue with evidence (rows, similarity, diff); the applier executes only approved proposals (human approval or, later, auto-approve classes gated by MEM-048's eval bracket), through the MEM-013 op pipeline so every apply is chained.
3. Rewrite governance: `revision` lineage + depth cap 3, then re-derive from sources; playbooks curated by delta updates only (add/update/remove single bullets), never whole-document rewrites.
4. Nightly schedule on the maintenance tick; disabled by default per the cap-4 dream-loop precedent.

Accept: detectors produce sensible proposals on dev corpus; nothing applies without approval; every apply reversible and chained; depth cap enforced.

Tests: detector unit tests ported from Python; applier gating tests; depth-cap test.

Review (human): review the first two weeks of proposals manually before approving any auto-approve class.

---

## MEM-048 - Promotion + dedup + contradiction, eval-bracketed

refs R28, R29, R30, R35 | est 12h | deps MEM-047 | priority high

Files: `src/brain/dream/` jobs (promotion, dedup, contradiction), eval bracket wrapper (golden run before/after each batch, auto-revert on regression via op-pipeline inverse ops), thresholds config.

Steps: promotion = episodes past hot-age or access threshold distilled into facts with `derived_from`; dedup = cosine>0.95 pairs merged via UPDATE/DELETE ops; contradiction = new facts trigger neighbor adjudication, losers get `invalid_at`; each nightly batch runs the golden set before and after and reverts itself on >threshold regression, filing a report row.

Accept: a synthetic contradiction resolves bi-temporally; a planted regression auto-reverts (test); batch reports chained.

Tests: promotion/dedup/contradiction integration; auto-revert drill.

Review (human): approve the regression threshold; audit the first auto-revert report end to end once.

---

## MEM-049 - Retrieval A/B + shadow evaluation

refs R48, R54 | est 10h | deps MEM-045, MEM-021 | priority medium

Files: `retrieval_config` table (weights, k, floor, rerank blend, MMR lambda, version), hash-based assignment in recall, shadow executor (sampled requests re-run under candidate config, results logged not returned), judge-scored comparison job, flag-flip promotion.

Accept: two configs measurably compared on live-sampled traffic without user impact; promotion is one flagged row flip with rollback.

Tests: assignment determinism; shadow isolation (never returned to caller); comparison-report test.

Review (human): approve each config promotion from the comparison report (one-line decision, recorded).

---

## MEM-050 - GEPA prompt optimization

refs R49 | est 10h | deps MEM-045 | priority medium

Files: `tools/memory-gepa/` (Python, dspy.GEPA runner driving the extraction/summary/judge prompts against the golden + benchmark sets via the gateway), prompt registry under `services/memory/prompts/` with versions, PR workflow.

Steps: wrap each optimizable prompt as a DSPy program with the golden metric; run GEPA offline on a budget (gateway spend cap respected); emit the winning prompt + eval evidence as a PR; never hot-swap prompts outside the repo.

Accept: one full optimization cycle produces a measurable golden lift merged through review; spend within cap.

Tests: runner smoke on a toy metric; registry version pinning test.

Review (human): review optimized prompts for tone/safety before merge (optimizers exploit metrics).

---

## MEM-051 - Self-healing registry + drift sentinels + DLQ

refs R52, R43, R97 | est 12h | deps MEM-026, MEM-045 | priority high

Files: `src/brain/jobs.rs` registry (re-embed pending, re-summarize stale, re-dedupe, reindex, refresh profiles) with before/after metric capture + auto-revert hooks; sentinel corpus + nightly re-embed comparison job; DLQ state for poison events (repeated Malformed) with ops-tile card; alerts via obs.

Accept: each job idempotent, bracketed, ledger-logged; sentinel drift alert fires on a simulated model swap; poison event parks in DLQ instead of retry-looping.

Tests: job bracket tests; sentinel simulation; DLQ transition test.

Review (human): confirm alert routing (who gets paged) matches the obs on-call reality.

---

## MEM-052 - Telemetry to standard + ops tile + weekly report

refs R50, R51, R53, R104 | est 14h | deps MEM-008 | priority medium

Files: OTel GenAI semconv attributes on gateway-call spans (`gen_ai.request.model`, token usage) + `memory.op` counters; console memory tile (apps/web): ingest lag, pending/DLQ, tier counts, recall p50/p95, hit-rate, golden trend, spend, quarantine + erasure queues, chain-walker status; weekly report generator (chained markdown artifact + optional chat post to Stephen).

Accept: tile live on the console with real data; weekly report generated and chained; semconv attributes visible in the obs backend.

Tests: metric emission tests; report generation snapshot test.

Review (human): first weekly report reviewed for usefulness; cut anything you will not read.

---

## MEM-053 - Poisoning defenses

refs R79, R80, R58, F24 | est 12h | deps MEM-013 | priority critical

Files: trust scoring in `fact_ops.rs` + `scoring.rs` (source_kind-based priors, decay for external/tool), quarantine enforcement in recall (default exclude `quarantined` rows for privileged callers; aging + approval clears it), prompt-wrapping contract (recall consumers receive snippets wrapped as quoted data with a no-instructions preamble; document + enforce in the MCP surface), red-team suite in CI (MINJA-style: instructions inside captured content, adversarial near-duplicates targeting another subject, tool-output injection).

Accept: red-team cases end quarantined or de-ranked, never executed; trust visibly affects ranking in explain; privileged recall (evaluation engine) excludes quarantined by construction.

Tests: the red-team suite; quarantine lifecycle test; wrapper contract test.

Review (human): review the red-team corpus and add cases from your own threat ideas; this list is never finished.

---

## MEM-054 - Calibrated abstention

refs R57 | est 6h | deps MEM-045 | priority medium

Files: `recall.rs` confidence output (calibrated from rerank/fused scores against judge outcomes), benchmark abstention bucket wiring, ops-tile false-memory metric.

Accept: below-floor recalls return `confidence` low enough that agent callers answer "not in memory" per contract; false-memory rate tracked release-over-release.

Tests: calibration monotonicity test; abstention benchmark cases.

Review (human): none beyond benchmark trend.

---

## MEM-055 - MCP server + memory-tool adapter

refs R101, R102 | est 12h | deps MEM-028 | priority high

Files: new `services/mcp-gateway` tools (or a `memory-mcp` slice within it): `memory.recall`, `memory.remember` (fact ADD via the op pipeline, quarantined by default), `memory.feedback`; memory-tool adapter exposing path-addressed `/memories` CRUD (list/read/write) backed by facts + profiles per Anthropic's memory-tool contract; JWT auth + rate limits inherited.

Accept: Claude-family agent drives recall/remember end to end against dev; memory-tool adapter passes the contract's basic flows; writes land as quarantined facts with provenance `source_kind='agent'`.

Tests: MCP tool contract tests; adapter flow tests; poisoning check (remember cannot inject instructions that recall would unwrap).

Review (human): try it from Cowork/Claude Code yourself; the developer experience here is a product surface.

---

## MEM-056 - Perf + scale pass

refs R91, R92, R94, R96, R99, F30 | est 12h | deps MEM-026 | priority medium

Files: `memory_tenant_registry` table + registration on first emit (replaces DISTINCT scans in `main.rs::discover_tenants`); bounded-concurrency per-tenant ingest (semaphore, fairness); decide + implement the `l2_memory` HNSW question (create with halfvec or delete the dead code path); nightly SLO smoke (synthetic corpus: recall p95 <300ms at 1M hot rows, ingest lag p95 <5s) tracked in `docs/improvement/memory/bench/`; partitioning plan doc with concrete triggers (row counts / index size) for `l1_audit_log` + `brain_event_embedding`.

Accept: discovery scans gone; ingest parallel + fair; SLO smoke green and tracked; partition plan merged.

Tests: registry integration; fairness test (busy tenant cannot starve others); the smoke itself.

Review (human): approve the partition triggers (they schedule future migration work).

---

## MEM-057 - Module-author guide + namespaces/budgets

refs R107, R108 | est 6h | deps MEM-025 | priority medium

Files: `docs/knowledge/memory-module-authors.md` (emit contract, attributes vs content_ref, recall scopes, feedback, poisoning rules: memory output is data, never instructions), gateway policy entries for per-module spend caps + memory namespaces, AGENTS.md link.

Accept: guide covers every contract a module author touches; a new emitter built from the guide alone passes review; per-module caps active on the gateway.

Tests: doc examples compile/run where extractable.

Review (human): read it as if you were a new hire; anything you had to ask about is a gap.

---

## MEM-058 - Lumi integration boundary

refs R105, R106 | est 8h | deps MEM-028, MEM-042 | priority high

Files: cuo/evaluation engine recall client (JWT as Lumi's own subject with explicit grants; no direct SQL against brain tables - enforce by role permissions), citation enforcement in the evaluation engine (claims without `audit_row_id` citations rejected pre-render, FR-EVAL-003), employee-facing boundary text (personal `sync_class: private` never enters the enterprise brain; what is captured, who can see it) added to the monitoring notice material.

Accept: Lumi's DB role cannot SELECT brain tables directly (test); an uncited evaluation claim fails validation; boundary text merged into the notice pack.

Tests: role-permission negative test; citation validation test.

Review (human): the boundary text is employee-facing; review wording with the same care as the monitoring notice (governance plan Phase 0).
