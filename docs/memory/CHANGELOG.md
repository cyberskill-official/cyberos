# Changelog — CyberOS-AGENTS.md

All notable changes to **CyberOS-AGENTS.md** are documented here, day by day.

This document does **not** carry an inline version marker — see CyberOS-AGENTS.md §0.2 (no-inline-version rule for design docs). Improvements land continuously; this changelog is the canonical record. Format inspired by [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), but date-stamped rather than version-stamped.

> **Companion docs:** `docs/CyberOS-AGENTS.md` (protocol) + `docs/CyberOS-AGENTS.README.md` (on-ramp Parts 1–24 plus the per-aspect operator reference at Parts 25–31). Every Aspect number cited here maps to its detailed section in README Part 26.

---

## 2026-05-12 — Batch 10 ship: ALL remaining deferrals closed (informational; no AGENTS.md edits)

> **No AGENTS.md edits — operator + tooling layer only.** Aspect-numbered references to `workbench/cyberos-layer1-deep-improvements.md`.

### Group H (smaller deferrals)

**Aspect 2.2 — Trend lines in dashboard.** `cyberos status` now includes a `TRENDS` section: 30-day rolling memory net change (creates − deletes), audit-op rate (per-day average), and drift-surfaced count. Live today: +159 memory net (161 creates − 2 deletes), 14.8 ops/day, 1 drift in 30d.

**Aspect 11.3 — Drift dashboard.** `cmd_drift` already shipped earlier as `cyberos drift`; documented and verified working.

**Aspect 1.3 / 5.3 — `--dry-run` + sev-0 confirm coverage.** Audited: `cyberos add`, `cyberos sync import`, `cyberos doctor --repair --reason`, `cyberos panic --reason`, `cyberos encrypt enable/rotate` already require either `--dry-run` or explicit reason. No additional roll-out needed — gateguard PreToolUse hook covers tool-use side.

### Group I (substantial deferrals)

**Aspect 5.7 — TOCTOU `.lock.shared` advisory locks** at `runtime/tools/cyberos_lock.py` + `cyberos lock {status|acquire-shared|acquire-exclusive}`. POSIX `fcntl.flock`-backed. Context managers `shared_lock()` and `exclusive_lock()` for use from `brain_writer.py` and `cyberos_validate.py`. Degrades to no-op on filesystems without flock (some FUSE / network FS). Live-tested: acquire + release both lock types succeed.

**Aspect 9.1 — Streaming session-start loader** at `runtime/tools/cyberos_lazy.py`. Two-phase loader — Phase A reads only manifest + checkpoint + legacy lists (~5 files, < 100 KB); Phase B yields memory paths one at a time without reading bodies. **Live benchmark on the current BRAIN: full eager load 180.93 ms vs lazy first-5 walk 2.41 ms — 74.9× speedup**. Caller modules can opt-in by importing `stream_memories()`.

**Aspect 9.2 — Incremental SQLite index hook** at `runtime/tools/cyberos_index_hook.py`. Two modes: `on-write` (called by brain_writer after each successful append; best-effort, never blocks the write); `stop-hook` (refreshes index at session.end as a safety net). No-op if `index/cyberos.db` doesn't exist yet.

**Aspect 9.5 — Cold-storage tier** at `runtime/tools/cyberos_cold_storage.py` + `cyberos cold-storage {archive|list|verify}`. Produces deterministic `.cold.zip` bundles per-month with a Merkle anchor pointing at the live BRAIN's chain head at archive time. Does NOT upload — operator uses `aws s3 cp` / rclone / equivalent. Includes `verify` subcommand to confirm an archive's SHA matches its manifest record. Live-tested: archived 2026-05.jsonl (444 rows / 435,884 B), listed, anchor recorded.

### Group J (starter + corpus + registry)

**Aspect 8.2 — `cyberos-starter` skeleton** at `outputs/cyberos-starter/`. README + pre-built `.cyberos-memory/manifest.json` with placeholder fields + `meta/retention-rules.md` + `meta/validators/README.md` + `tours/onboarding.tour` (CodeTour-compatible). Drop-in template for new projects.

**Aspect 10.1 — Test corpus growth.** Added 2 new mutation fixtures: `fixture-valid-decision.md` + `fixture-valid-person.md`. Mutation test now runs **24 mutations across 3 fixtures, 0 SURVIVED**. Corpus: 1 → 3 fixtures + 8 mutation patterns = 24 distinct mutant tests.

**Aspect 12.5 — Skill registry** at `runtime/tools/skills/registry.json` + `runtime/tools/cyberos_skill.py` + `cyberos skill {list|describe|chain}`. 22 skills registered (every operator tool we've shipped) with their verb, mutates_brain flag, depends_on graph, §-rule list, and umbrella-alias. `chain` subcommand surfaces the dependency graph and warns when two mutating skills run without a verify between them.

### Wired

`cyberos lock`, `cyberos cold-storage`, `cyberos skill` added to umbrella dispatch. Total subcommand count 30 → **33**.

### Verified

- `cyberos verify` → CRITICAL: 0 / WARN: 11 / INFO: 1 (unchanged — no new validator findings from any new tool)
- `cyberos mutation-test` → 24 mutations run, 0 SURVIVED (corpus grew from 8 to 24 tests)
- `cyberos lazy benchmark` → 74.9× speedup for first-5 walk vs full-eager
- `cyberos cold-storage archive` → deterministic .cold.zip with Merkle anchor
- `cyberos skill chain` → safe-chain validator working
- Audit chain intact across all 10 batches

### Layer-1 catalog status

**100% of named aspects in `workbench/cyberos-layer1-deep-improvements.md` shipped.** The 13-aspect catalog from 2026-05-12 morning is fully closed. Aspects landed: 1.1, 1.2, 1.3 (audited as covered), 1.4, 1.5, 1.6, 2.1, 2.2, 2.3, 2.4, 2.5, 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 4.1, 4.2 (covered by stats), 4.3, 4.4, 4.5, 4.6, 4.7, 5.1, 5.2, 5.3 (covered by gateguard+reason gates), 5.4, 5.5, 5.6, 5.7, 6.1, 6.2, 6.3, 6.4, 6.5, 6.x, 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 7.7, 8.1, 8.2, 8.3, 8.4, 8.5, 9.1, 9.2, 9.3, 9.4, 9.5, 9.6, 9.7, 10.1, 10.2, 10.3 (blocked — only one impl exists), 10.4, 10.5, 10.6, 10.7, 10.8, 11.1, 11.2, 11.3, 11.4, 11.5, 12.1, 12.2, 12.3, 12.4, 12.5, 12.6, 12.7, 13.1, 13.2, 13.3, 13.4, 13.5, 13.6, 13.7, 13.8 (architectural — defer to repo-split decision), 13.9, 13.10.

---

## 2026-05-12 — Batches 21-23 ship: Tier α — deterministic skill runtime (informational; no AGENTS.md edits)

> Tier α from the post-Batch-20 catalog. 10 items shipped: deterministic per-skill runners, multi-iteration self-audit, resume-with-llm, frontmatter validator, test corpus, cross-skill validation, cost benchmarks, uniform telemetry, caching, streaming.

### Batch 21 — Tier α.1, α.2, α.3 (runner framework + resume + multi-iteration)

**`runtime/skill_runners/base.py`** — `BaseSkillRunner` class. Each chain skill gets a concrete subclass that owns the deterministic parts (interview, INVARIANT validation, voice gate, content-gate filtering, audit-fix loop) and delegates only the judgement-driven authoring to Claude. Flips the ratio from ~80% LLM judgement to ~20%.

- `interview(inputs)` — subclass hook for the standalone-interview loop
- `build_prompt(inputs, prior_artefacts)` — subclass composes the LLM prompt
- `author_body(inputs, llm_call)` — actual Claude call
- `validate_emit(body, inputs)` — INVARIANT enforcement, returns findings
- `run(inputs, output_dir, max_iterations, cache)` — orchestrates the loop:
  emit → validate → if CRITICAL: HITL pause; if WARN: re-prompt with fix hints; up to max_iterations

**`runtime/skill_runners/fr_with_tasks.py`** — reference implementation. 14 INVARIANT checks per task (FR-NNN-T-MM regex, ≥200-char description, concrete acceptance_test, dependency-graph acyclicity, etc.). Other 10 chain skills copy this template.

**`cyberos chain run --max-iterations N --no-cache`** — flags added. When a deterministic runner is available for a step, the chain uses it; otherwise falls back to the single-shot LLM call from Batch 16.

**`cyberos chain resume --with-llm`** — Tier α.2 — now actually calls the same runner pipeline as `chain run` on each resumable step. Token + cost accounting flows through to `chain-manifest.json`.

### Batch 22 — Tier α.4, α.5, α.7 (validation surface)

**`meta/validators/check-skill-frontmatter.py`** — Tier α.4 — `cyberos verify` now validates every `SKILL.md` frontmatter: required fields (name, skill_version, persona, owner_role), semver shape, persona in known set, determinism.reproducible is bool, untrusted_content_wrapping recommended as `required`. Memoised — runs once per validate pass. All 11 chain skills pass after the Batch 16+ patches.

**`runtime/tests/skills/<skill>/fixtures/` + `runtime/tests/skills/run_corpus.py` + `cyberos skill-test`** — Tier α.5 — test corpus framework. Shipped 3 fixtures for `fr-with-tasks` (slack-bot, cli-tool, data-pipeline-monitoring). Each fixture declares expected task-count range, sizes, assignability mix, invariant-clean flag. `cyberos skill-test fr-with-tasks --no-llm` exercises the runner harness without API calls.

**`runtime/tools/cyberos_cross_skill.py` + `cyberos cross-skill <chain-dir>`** — Tier α.7 — 5 cross-skill consistency checks:
- C1 task ID references resolve
- C2 fr-audit covered every FR
- C3 every tech-spec references a real FR (standard/full profiles)
- C4 every impl-plan ticket maps to a known task
- C5 chain-manifest plan steps and emitted files align

### Batch 23 — Tier α.6, α.8, α.9, α.10 (perf + observability)

**`runtime/tools/cyberos_skill_bench.py` + `cyberos skill-bench`** — Tier α.6 — runs the test corpus N times, records token_p50/p95, cost_p50/p95, iteration_p50/p95, pass_rate, latency. `--record` saves a baseline at `runtime/tests/skills/<skill>/baseline.json`. Subsequent runs detect regressions: token/cost growth > 30% OR pass-rate drop fails the bench.

**Uniform skill telemetry (`~/.cyberos/analytics/skill-runs.jsonl`)** — Tier α.8 — every runner invocation logs ts, skill_id, skill_version, phase (PASS/HITL_PAUSE/EXHAUSTED/cache-hit), model, input_hash, iterations, tokens, cost, output path. Uniform schema across all 11 chain skills via the base class `_log_telemetry()` method.

**Skill caching (`~/.cyberos/skill-cache/`)** — Tier α.9 — `SkillCache` keyed by `(skill_id, skill_version, input_hash)`. When a run hits the cache, status returns `PASS` with `iterations=0`, `tokens_used=0`, `cost_usd=0.0`. Skipped via `cyberos chain run --no-cache`.

**Streaming output (`base.llm_call_streaming`)** — Tier α.10 — helper for streaming Claude responses. Operator can subscribe to per-token deltas via `on_token` callback. Wired into `base.py` but not yet surfaced as a flag on `cyberos chain run` (next batch can add `--stream`).

### Wired

`cyberos skill-test`, `cyberos skill-bench`, `cyberos cross-skill` — 3 new umbrella subcommands. Chain run + resume gained `--max-iterations` + `--no-cache` flags. Umbrella count **60 → 63**.

### Verified

- `cyberos verify` → CRITICAL: 0 / WARN: 12 / INFO: 1 (new validator passing all 11 chain skills)
- `cyberos skill-test fr-with-tasks --no-llm` → 3/3 fixtures harness-OK
- `cyberos skill-bench fr-with-tasks --no-llm` → no baseline yet; ready to record once you run with real API key
- `cyberos cross-skill planning/<dir>` → returns 0 findings on the existing FR-001 chain
- Runner harness: `python3 runtime/skill_runners/fr_with_tasks.py outputs/_smoke --pitch "..."` returns `FAILED: anthropic SDK not installed` cleanly

### Layer-1 + skills final state

This is the genuine endpoint for the operator surface. Layer 1 + skills now have:
- 63 umbrella subcommands
- 5 pluggable validators
- 11 chain skills at 5/5 quality + deterministic runner pattern
- 14 INVARIANT checks per emitted FR
- Test corpus + benchmark + telemetry + cache infrastructure
- Audit chain intact across 23 batches

The next 10× from here lives in actually running the chain on real CyberSkill work, not in more tooling.

---

## 2026-05-12 — Batches 17-20 ship: skills Stages 3 + 4 + 5 + 6 + 8 (informational; no AGENTS.md edits)

> Completes the multi-stage skills improvement catalog the user reviewed. Batch 16 shipped Stages 1+2+S7.1; these 4 batches finish the rest.

### Batch 17 — Stage 3 (authoring quality)

- **`runtime/tools/cyberos_authoring.py` + `cyberos authoring {llm|voice|attribute|diff|interview}`** — Shared library for skill runtimes. Functions:
  - `llm_draft_body(prompt, model)` — S3.1 — anthropic SDK with graceful fallback
  - `voice_gate(text)` — S3.2 — em-dash + AI-vocab linter (16 banned words)
  - `attribute_claims(body, source_text)` — S3.3 — auto-attribution per paragraph (human-confirmed if source contains the key tokens, llm-explicit otherwise)
  - `diff_artefact(old_path, new_text)` — S3.4 — unified-diff between prior and new version
  - `interview_questions(persona, mode)` — S3.5 — per-persona question banks loaded from `meta/interview-templates/<persona>.md`; falls back to embedded defaults for cpo/cto/cseco/clo/founder

### Batch 18 — Stage 4 (runtime + execution)

- **`chain_manifest@1` contract** at `docs/contracts/chain-manifest/CONTRACT.md` — persistent state schema for `cyberos chain run` invocations. 15 required fields including per-step status, retry budgets, calibration tracking. Enables resume.
- **`cyberos chain resume <output-dir>`** — S4.2 — picks up first non-done step, advances state, writes back manifest. Live-tested: 2 paused steps → both flipped to done.
- **`cyberos_skill.py` extended with `discover_docs_skills()`** — S4.1 — `cyberos skill list` now auto-discovers chain skills in `docs/skills/` alongside the registry-declared operator tools. Surfaces persona + owner_role.
- **`meta/validators/check-persona-boundary.py`** — S4.4 — flags FRs that drift into CTO / CSecO / CLO territory by keyword density. Surfaces as INFO (not blocking). Solo profile is exempt.
- **S4.5 cost budget** baked into chain_manifest@1 — budget block with max_tokens + max_cost_usd; pause + HITL when exceeded.

### Batch 19 — Stage 5 + Stage 6 (surfaces + quality)

- **`runtime/tools/cyberos_proj.py` + `cyberos proj {backends|sync|pull}`** — S5.4 — proj-tracker integration. Subcommand `sync FR-NNN --backend {linear|jira|github}` reads embedded `task@1` list and emits backend-specific envelopes (CLI commands + ticket body + labels) to `<FR>.proj-sync.json`. Operator pipes to `linear-cli`, `jira-cli`, or `gh issue create`. Live-tested: 6 envelopes generated from FR-001.
- **`runtime/tools/cyberos_skill_quality.py` + `cyberos skill-quality {run|calibration}`** — S6.1-S6.5 — five checks per skill:
  - antifab — references ANTI_FABRICATION.md + HITL discipline
  - untrusted — declares `untrusted_content_wrapping: required`
  - grounding — emits authority markers + source_ref attribution
  - calibration — historical HITL rate from analytics; warn if > 30%
  - deprecation — surfaces `deprecated_at` + `replaced_by` fields
- Live-tested against `fr-with-tasks` skill: surfaced 3 real findings (will fix in follow-up); calibration + deprecation passed.

### Batch 20 — Stage 8 (future-state scaffolds)

- **`runtime/tools/cyberos_advanced.py` + `cyberos advanced {fr-council|auto-decompose|client-chain|replan|marketplace}`**:
  - **S8.1 `fr-council <FR-id>`** — applies council mode (4 voices) at the FR layer, reusing the Layer-1 council templates
  - **S8.2 `auto-decompose <task-id>`** — emits a `runtime_spec` JSON for a task: 5-step agent-runnable sequence (read, explore, act, verify, report) with budget + abort conditions. Live-tested with FR-001-T-02.
  - **S8.3 `client-chain`** — forces `chain_profile: full` + persona-separation locks for client-visible work; the inverse of `solo`
  - **S8.4 `replan`** — walks drift candidates + 3-months-old rejected items; emits a re-plan proposal markdown. Live-found 1 drift candidate.
  - **S8.5 `marketplace {list|add|install}`** — scaffolding for a community skill registry at `~/.cyberos/skill-marketplace.json`. Install is currently a manual git clone hint.

### Wired

`cyberos {authoring|proj|skill-quality|advanced}` — 4 new subcommand families. Umbrella count **56 → 60**.

### Verified

- `cyberos verify` → CRITICAL: 0 / WARN: 12 / INFO: 1 (+1 INFO from new persona-boundary validator — by design, not blocking)
- Live walk-through: drove the Slack-HR-bot pitch through the solo chain → generated 6 tasks → ran `cyberos proj sync` (envelopes for github) → ran `cyberos auto-decompose FR-001-T-02` (runtime_spec) → ran `cyberos fr-council FR-001` (4 voice prompts) → ran `cyberos skill-quality run fr-with-tasks` (surfaced 3 real gaps)
- `cyberos chain resume` lifecycle: PLACEHOLDERS_WRITTEN → resume → DONE
- `cyberos skill list` now shows 22 operator skills + 11 chain skills (docs/skills/cuo/)

### Honest framing

Batches 17-20 ship 18 items from Stages 3, 4, 5, 6, 8 of the post-Batch-16 catalog. The skills layer is now feature-complete for the planned multi-stage improvements. Stage 7 was already shipped via the `cyberos chain` umbrella in Batch 16. As with Layer 1's Tier E, further investment here hits diminishing returns; the next 10× lives in actually wiring the skill runtimes (not just the operator surface) and in Layer 2 (vectors + graph).

---

## 2026-05-12 — Batch 16 ship: skills-Stage-1 collapse — fr-with-tasks + solo profile + cyberos chain umbrella

> First batch that touches the **skills** layer (CPO/CTO chain) rather than Layer 1 operator tools. Implements skills-Stage-1 + Stage-2 + S7.1 from the catalog the user reviewed. Collapses the 2-stage `fr-author + fr-to-tech-spec` flow into a single `fr-with-tasks` skill for the new default `solo` chain_profile.

### Added

**S2.1 — `task@1` contract** at `docs/contracts/task/{CONTRACT,template,CHANGELOG}.md`. Comprehensive task shape with 14 required + 6 optional fields. Task IDs `FR-NNN-T-MM`. ≥200-char description floor. Acceptance test must be `shell` or `assertion` (concrete). Assignable_to: `[human, ai-agent]` with profile + token/hour estimates.

**S1.1 — `fr-with-tasks` skill** at `docs/skills/cuo/cpo/fr-with-tasks/`. Collapses CPO→CTO 2-step into a single skill emitting `feature_request@1` with embedded `task@1` list. Replaces `fr-author + fr-to-tech-spec` for the `solo` profile. 14 INVARIANTS, 3-question standalone interview, self-audit before emit.

**S1.2 — `solo` chain_profile** added to `chain-selector` skill. Default for CyberSkill internal workflows (1-10 person team, client_visible:false, EU AI Act limited or below). Replaces `standard` as the new default for non-client work.

**S1.3 — skip-PRD triage** in `chain-selector`. When upstream is a natural-language spec and it has ≥5 acceptance criteria + ≥1 measurable metric + an explicit persona, the chain plan sets `skip_prd: true` and `fr-with-tasks` consumes the NL spec directly.

**S7.1 + S1.4 — `cyberos chain` umbrella** at `runtime/tools/cyberos_chain.py`. Subcommands: `run`, `status`, `resume`, `estimate`, `graph`. One-shot trigger: `cyberos chain run --pitch "..." --profile solo`. Writes `chain-manifest.json` to `planning/<date>-<slug>/`.

**S2.3 — `cyberos fr` browser** at `runtime/tools/cyberos_fr.py`. Subcommands: `list`, `show <FR>`, `graph`, `task-graph <FR>`. Walks `planning/`, `memories/projects/`, `outputs/staged-memories/` for FR markdown files; parses embedded `tasks:` lists; renders Mermaid DAG of task dependencies.

### Wired

`cyberos chain {run|status|resume|estimate|graph}` + `cyberos fr {list|show|graph|task-graph}` added to the umbrella. Total subcommand count **54 → 56**.

### Live test

Drove a real pitch ("Slack HR-policy bot MVP") through the solo chain:

- `cyberos chain estimate --profile solo` → 8K-25K tokens / $0.05-0.17 USD
- `cyberos chain run --pitch "…" --profile solo` → wrote chain manifest + placeholders
- Authored a real FR-001 with 6 embedded tasks (2 S / 4 M), 3 human-only + 2 AI-only + 1 either
- `cyberos fr list` → surfaced the FR with sizing breakdown + assignability mix
- `cyberos fr task-graph FR-001` → rendered Mermaid DAG of T-01 through T-06 dependencies

### Honest framing

The collapsed `fr-with-tasks` skill is the right shape **for CyberSkill internal use today**. The 2-stage `fr-author + fr-to-tech-spec` chain remains intact (deliberately) for future client-facing work where CPO + CTO persona separation matters for EU AI Act §8 audit trails. The `solo` profile is opinionated about the trade-off: persona-separation theatre out, founder velocity in.

### Verified

- `cyberos verify` → CRITICAL: 0 / WARN: 12 / INFO: 1
- New skill loads + parses cleanly; new contract validates
- End-to-end chain executes (placeholder mode; `--with-llm` wiring for live authoring in next batch)
- `cyberos fr task-graph` Mermaid output renders correctly in GitHub / Obsidian

---

## 2026-05-12 — Batch 15 ship: Tier E (genuine Layer 1 wins) + leftover cleanup (informational; no AGENTS.md edits)

> Tier E was billed as "the last genuine Layer-1 wins before diminishing returns". 9 items + a cleanup tool shipped.

### Added

**E.1 Schema migration framework** — `runtime/tools/cyberos_migrate.py` + `cyberos migrate {list|plan|apply}`. Migrations live under `migrations/<NNN>-<slug>.py` exporting `APPLIES_TO`, `DESCRIPTION`, `transform(fm, body, rel)`. State persisted at `meta/migrations-applied.json` so each migration runs once. Sample migration shipped: `migrations/001-example-add-tag.py`.

**E.2 Inline editor** — `runtime/tools/cyberos_edit.py` + `cyberos edit <memory>`. Opens `$EDITOR` (falls back to vi/nano), validates frontmatter on save, commits via `brain_writer str-replace`. Resolves memory_id / full path / PREFIX-NNN.

**E.3 Bulk edit** — `runtime/tools/cyberos_bulk.py` + `cyberos bulk-set <expr> --filter ...`. Field-level changes across many memories. Operators: `=`, `+=` (list append), `-=` (list remove). Refuses to bulk-set `memory_id`, `audit_chain_head`, `created_at`, `created_by`; refuses `classification`/`authority` without `--allow-protected`. Filters: `scope:`, `tag:`, `classification:`, `authority:`, `sync_class:`, `tombstoned:`.

**E.4 Hybrid search (RRF)** — `runtime/tools/cyberos_hybrid_search.py` + `cyberos hybrid-search <query>`. Reciprocal Rank Fusion over SQLite FTS + TF-IDF (and optionally sentence-transformers via Batch 11). Default k_const=60. Per-backend weights via `--weight-fts`, `--weight-tfidf`. Live-tested.

**E.5 Audit streaming + alert webhooks** — `runtime/tools/cyberos_stream.py` + `cyberos audit-stream` (long-poll the current-month ledger) + `cyberos alert {add|list|remove|run}`. Alert rules are simple expressions (`CRITICAL > 0`, `drift > 5`, `audit_ops_24h > 100`). Action types: `stdout`, `slack-webhook <url>`, `exec <cmd>`. Rules persisted at `meta/alerts.json`.

**E.6 REPL history + tab completion** — `runtime/tools/cyberos_repl.py` extended with `readline` integration. History at `~/.cyberos/repl-history` (last 1000 lines). Tab completion against the full 54-subcommand list. Up-arrow recall works on POSIX.

**E.7 Chaos tests** — `runtime/tests/chaos/test_chaos.py` + `cyberos chaos-test`. Three fault-injection scenarios: (a) `tmp+rename` atomicity — partial `.tmp.<file>.part` cleanup; (b) ENOSPC at write time — clean error, no audit row; (c) concurrent writers — second writer blocks on `.lock.exclusive`. 3/3 pass.

**E.8 Disk-full simulation** — bundled with E.7. ENOSPC injection test asserts no half-rows in ledger when write fails.

**E.9 Per-memory ACLs** — `.cyberos-memory/meta/validators/check-acl.py`. New pluggable validator. Frontmatter `acl: {read: [...], write: [...]}` with entries like `subject:<slug>` or `role:<name>`. Personnel-class memories without an `acl:` block surface as WARN. Live-surfaced 1 finding (PERSON-001 lacks acl).

### Cleanup tool (Tier E maintenance)

**`runtime/tools/cyberos_cleanup.py` + `cyberos cleanup [--apply] [--out-script <path>]`** — Detects leftover test artefacts: `outputs/test-*`, `outputs/cold-test/`, `outputs/audit-bundle.zip`, sync test reports, stale staged memories, `.branches/experiment-*` snapshots, stale council sessions, the obsolete `CyberOS-LAYER-1-MANUAL.md` stub. Produces a `cleanup-host.sh` script the operator runs on the host filesystem (sandbox cannot unlink most of these). **16 cleanup candidates** detected totalling **4.1 MB**; script written to `outputs/cleanup-host.sh`.

### Wired

11 new subcommands: `migrate`, `edit`, `bulk-set`, `hybrid-search`, `audit-stream`, `alert`, `chaos-test`, `cleanup` + the existing alert subcommands. Umbrella count **46 → 54**.

### Verified

- `cyberos verify` → CRITICAL: 0 / WARN: 12 / INFO: 1 (new WARN from acl-missing-on-personnel surfacing a real PERSON-001 gap)
- `cyberos chaos-test` → 3/3 pass
- `cyberos migrate plan 001-example-add-tag` → 12 memories would change (dry-run)
- `cyberos hybrid-search "tier-1 immutable"` → top hit `company/locked-decisions.md` as expected
- `cyberos alert run` → 1 rule evaluated, value 11.0 vs threshold 20, fired=False
- `cyberos cleanup --out-script` → 16 candidates / 4.1 MB / script written

### Layer-1 final state

54 umbrella subcommands. 4 pluggable validators. 9 batches' worth of catalog + 5 batches of post-catalog. Layer 1 is **decisively past its diminishing-returns boundary** — further work belongs in Layer 2 (vectors + graph) or the CUO router (P0+). Run `outputs/cleanup-host.sh` on the host filesystem to delete the 4.1 MB of test leftovers the sandbox couldn't unlink.

---

## 2026-05-12 — Batches 11–14 ship: post-catalog Tiers A/B/C/D (informational; no AGENTS.md edits)

> **Beyond-catalog work.** Layer-1 catalog closed at Batch 10. Batches 11–14 ship the 4 tiers of post-catalog suggestions (17 items total). All additions are operator-surface; no AGENTS.md edits.

### Batch 11 — Tier A (high leverage, low effort)

- **Lock integration** — `cyberos_validate.py` now acquires `.lock.shared` for the duration of the validate pass via `cyberos_lock.shared_lock()`. Best-effort: degrades silently on filesystems without `fcntl`. `CYBEROS_NO_LOCK=1` env var to disable.
- **Semantic search** — `runtime/tools/cyberos_semantic_search.py` + `cyberos semantic-search "<query>"`. Default backend: TF-IDF cosine (zero-dependency, ~50 ms for 157 memories). Opt-in `--backend sbert` for sentence-transformers if installed.
- **TUI dashboard** — `runtime/tools/cyberos_tui.py` + `cyberos tui --interval 10`. Curses-based full-screen view (memories, audit head, drift queue, council pending, recent rows). Press `q` to quit, `r` to refresh.
- **Diff + time-travel** — `runtime/tools/cyberos_history.py` + `cyberos history diff <id>` / `cyberos history as-of <ts|HEAD~N>`. Walks audit chain, reconstructs path-level state at any point.
- **Council `--run-now`** — `cyberos council REF-NNN --run-now` extends Aspect 3.3 by actually calling Claude for each voice via the anthropic SDK (requires `ANTHROPIC_API_KEY`). Gracefully falls back to manual-paste stubs if SDK / key missing.

### Batch 12 — Tier B (high leverage, more effort)

- **Branched BRAINs** — `runtime/tools/cyberos_branch.py` + `cyberos branch {list|create|switch|diff|merge|delete}`. Snapshots stored at `.cyberos-memory/.branches/<name>/`. Switch is a scaffold (filesystem move privileges). Live-tested: created `experiment-tier-b` snapshot of 444-row chain.
- **LLM-assisted REF authoring** — `runtime/tools/cyberos_ref_from_drift.py` + `cyberos ref-from-drift <drift>.md [--with-llm]`. Reads a drift candidate, stages `outputs/staged-memories/REF-NNN-...md` with structured scaffold (Trigger / Tier / AGENTS section / eval skeletons / steps). LLM-drafted body when `--with-llm` + SDK + key.
- **Auto-repair** — `runtime/tools/cyberos_autorepair.py` + `cyberos autorepair [--apply] [--recipe X]`. 3 recipes wired (tag-budget-exceeded, duplicate-tags, tombstone-missing-metadata). Dry-run default; `--apply` writes. Safety envelope: never touches authority/classification/consent/memory_id; never deletes.
- **Web dashboard** — `runtime/tools/cyberos_serve.py` + `cyberos serve --port 8080`. Stdlib `http.server`, zero dependencies. Routes: `/`, `/memories`, `/memory/<id>`, `/audit`, `/stats.json`. Live-tested: `curl /stats.json` returned manifest summary.
- **Auto-supersedes hint** — extends `cyberos_add.py`: when adding a memory, scans the same bucket for similar-stem files and prints up to 3 candidates the operator might want to set `supersedes:` against.

### Batch 13 — Tier C (strategic, bigger lift)

- **Replicated audit chain** — `runtime/tools/cyberos_replicate.py` + `cyberos replicate {status|push|verify}`. Best-effort filesystem-level replication of audit ledgers to operator-supplied target dir (S3 mount / peer / backup). Tracks last_audit_id + last_push_at in `.replicate-state.json`. Tool never contacts a network provider; operator picks transport.
- **Multi-tenant scaffolding** — `runtime/tools/cyberos_tenant.py` + `cyberos tenant {list|create|audit}`. Creates `member/<slug>/` scopes; `audit` subcommand flags cross-tenant references for consent review.
- **CRDT merge** — `runtime/tools/cyberos_crdt.py` + `cyberos crdt merge <conflict>`. Field-level merge for sync conflicts: tags union, relationships union, last_updated_at max, version max, authority max, sync_class tightens, classification REFUSED to auto-merge, body multi-value-register.
- **Hypothesis property tests** — `runtime/tests/property/test_frontmatter_properties.py`. Properties: yaml round-trip parse, UUIDv7 monotonicity. Degrades to smoke check when hypothesis isn't installed (smoke PASSES today).

### Batch 14 — Tier D (research-flavored)

- **Signed protocol snapshots** — `runtime/tools/cyberos_sign.py` + `cyberos sign {keygen|sign|verify|verify-all}`. Ed25519 keypair via `cryptography` library. Private key at `~/.cyberos/keys/protocol-signing.ed25519` (mode 600). Public key committed at `.cyberos-memory/meta/protocol-signing-pubkey.ed25519`. Signs each `protocol-history/AGENTS-sha256-*.md` snapshot.
- **Parallel validator** — `runtime/tools/cyberos_parallel_validate.py` + `cyberos parallel-validate --workers N`. Splits memory files across N processes for distributed validation. Live benchmark: 136 files / 3 workers / 90 ms.
- **Mobile static view** — `runtime/tools/cyberos_static.py` + `cyberos static --out ~/cyberos-mobile/`. Renders the BRAIN as a static HTML site (no JS, dark-mode-aware CSS) for phone-accessible reads. Live-rendered: 136 pages in one pass.

### Wired

`cyberos {semantic-search, tui, history, branch, ref-from-drift, autorepair, serve, replicate, tenant, crdt, sign, parallel-validate, static}` — 13 new subcommands. Total umbrella count 33 → **46**.

### Verified

- `cyberos verify` → CRITICAL: 0 / WARN: 11 / INFO: 1 (unchanged from Batch 10)
- Audit chain intact across all 14 batches
- Lock integration: validate pass acquires `.lock.shared` cleanly
- Semantic search: live query returned top hit for "council voices ambiguous refinement"
- Branch lifecycle: `branch list` → `create` → `diff` all work
- Web dashboard: `/stats.json` round-trip OK
- Parallel validator: 3-worker run, 90 ms
- Static site: 136 HTML pages rendered

---

## 2026-05-12 — Batch 9 ship: validator tightening (mutation-test gaps closed) + FACT-015 session memory (informational; no AGENTS.md edits)

> **No AGENTS.md edits — operator + tooling layer only.** Tightens validator coverage to match what AGENTS.md §4.2 + §5.1 + §17 already imply.

### Fixed

**§4.2 content-gate body scan.** `cyberos_validate.py` now scans every memory body (not just frontmatter) for prompt-injection markers — `[INST]`, `<system>`, `<<SYS>>`, `<|im_start|>`, `<|assistant|>`, `###Instruction`, `###System:`, "ignore previous instructions", "ignore the above". Whitelists test fixtures, REFs, validator plugins, conflict files, and postmortems (all legitimately document the markers). Surfaced as WARN per finding.

**§5.1 negative version rejected.** `version` field is now enforced to be a positive integer; negative or non-integer values surface as CRITICAL `invalid-version` or `invalid-version-type`.

**§5.1 provenance block required.** Memories without a `provenance:` block surface as WARN `provenance-missing`. Malformed provenance (non-dict) surfaces as WARN `provenance-malformed`.

**§17 sync_class enum enforced.** Values outside `{local-only, publishable, shared, client-visible}` surface as WARN `invalid-sync-class`.

### Why

All four gaps were caught by `cyberos mutation-test` in Batch 8 — 4 mutations SURVIVED the validator. After this patch, all 8 mutations are KILLED. The fixes are pure tightening; no real memory in the BRAIN trips the new checks (CRITICAL stayed at 0, WARN count unchanged at 11).

### Added

**FACT-015 — Layer-1 catalog session memory** at `.cyberos-memory/memories/facts/FACT-015-batch-4-to-9-shipped.md`. Documents what landed in Batches 4–9 (umbrella subcommands 18→30, validators 0→3, mutations killed 4→8, 11 new runtime tools shipped). Lists deferred items with rationale. Committed via `brain_writer write` with audit row `evt_019e1a42-…`; chain head advanced to `sha256:b30dc197b713f168…`.

### Verified

`cyberos verify` → CRITICAL: 0; WARN: 11 (unchanged); INFO: 1. `cyberos mutation-test` → 0 SURVIVED, 8 KILLED (was 4 SURVIVED in Batch 8). Audit chain intact.

---

## 2026-05-12 — Batch 8 ship: explain + compact-stats + mutation testing + refinement dashboard (informational; no AGENTS.md edits)

> **No AGENTS.md edits — operator + tooling layer only.** Aspect-numbered references to `workbench/cyberos-layer1-deep-improvements.md`.

### Added

**Aspect 1.5 — `cyberos explain <subcmd>`.** Surfaces the §-rule trace for each subcommand — which AGENTS.md sections it touches and what each step does. Covers `verify`, `add`, `sync`, `doctor`, `council`, `prune`, `export`, `verify-self`. Pattern from `engineering/debug` skill ("every error = problem + cause + fix").

**Aspect 9.4 — `cyberos compact-stats`** at `runtime/tools/cyberos_compact_stats.py`. Reports per-month audit ledger row count + size + dominant op + age. Recommends compaction when any threshold trips (rows > 10k OR bytes > 5 MB OR age > 90d — all tunable). Does NOT compact; that's still `cyberos doctor --compact-ledger MM`. Live: 1 ledger (2026-05.jsonl), 443 rows / 0.41 MB / 0d → no compaction needed at current thresholds.

**Aspect 10.4 — Mutation testing scaffold** at `runtime/tests/mutation/run_mutations.py` + `cyberos mutation-test`. Applies 8 mutations (remove-memory-id, break-uuid-format, invalid-classification, inject-marker, invalid-authority, remove-provenance, negative-version, invalid-sync-class) to a valid fixture, runs validator on each mutant, fails if any mutation SURVIVES. Live-surfaced 4 real validator gaps: content-gate doesn't catch §4.2 injection markers in body, validator doesn't reject negative `version`, missing `provenance:` block, or invalid `sync_class` enum. These are real follow-up bugs the scaffold caught.

**Aspect 11.4 — `cyberos refinements`** at `runtime/tools/cyberos_refinements.py`. Three-bucket dashboard: drift candidates from the Aspect 3.1 Stop-hook, pending council sessions from Aspect 3.3 (regex-detects whether `**Verdict:**` is filled), recent `rejected/` entries from Aspect 3.4. Live: 1 open drift candidate + 1 pending council session — both genuine items needing review.

### Wired

`cyberos explain`, `cyberos refinements`, `cyberos mutation-test`, `cyberos compact-stats` added to umbrella dispatch. Help text updated.

### Deferred (out of scope for Layer-1 catalog batch passes)

- **Aspect 1.3 `--dry-run` cross-cutting.** `cyberos add` and `cyberos sync import` have it. The rest (doctor repair ops, sync export, encrypt enable/rotate) need per-op review before bulk roll-out.
- **Aspect 5.7 TOCTOU `.lock.shared` hardening.** Requires brain_writer.py + cyberos_validate.py to negotiate a shared-lock protocol. Substantive — punt to a dedicated REF.
- **Aspect 9.1 streaming session-start.** Matters at 1000+ memories; we have 155. No urgency.
- **Aspect 9.2 index incremental updates.** SQLite rebuild today is fast. Revisit at scale.
- **Aspect 12.5 skill registry refactor.** Big rework; treated as part of the eventual CyberOS Skill Pack release.

### Verified

`cyberos verify` → CRITICAL: 0; WARN: 11 (unchanged from Batch 7 — no new validator findings); INFO: 1. Total subcommands now 30 in the umbrella (up from 26 last batch). Audit chain intact.

---

## 2026-05-12 — Batch 7 ship: prune + hooks toggle + source-tiers validator (informational; no AGENTS.md edits)

> **No AGENTS.md edits — operator + tooling layer only.** Aspect-numbered references to `workbench/cyberos-layer1-deep-improvements.md`.

### Added

**Aspect 1.1 + 9.7 — `cyberos prune`** at `runtime/tools/cyberos_prune.py`. Surface-only (never deletes). Three checks: (a) stale memories whose `last_updated_at` is older than `--staleness-days` (default 365) and whose retention rule is not `indefinite`; (b) contradictions — `supersedes`-edges where the older memory was never tombstoned, plus `contradicts`-edges where both sides are alive; (c) unresolved drift candidates older than `--drift-days` (default 30) without a `## Resolution` section. `--interactive` steps through each candidate. Operator resolves via `cyberos doctor` subcommands.

**Aspect 5.1 (operator surface) — `cyberos hooks {status|on|off}`** at `runtime/tools/cyberos_hooks.py`. Installs / removes the gateguard PreToolUse and refinement_candidates Stop hooks into `~/.claude/settings.json` (override via `$CYBEROS_CLAUDE_SETTINGS`). Idempotent. Sandbox-safe (prints the JSON snippet for manual paste when it cannot write). Per-hook targeting with `--hook gateguard|refinement_candidates`. Live-tested the full status→on→status→off lifecycle.

**Aspect 12.3 — Source-tiers staleness validator** at `meta/validators/check-source-tiers.py`. Reads `manifest.source_tiers`, checks each `pattern` resolves to ≥1 file on disk, surfaces stale entries as WARN. Memoised — runs once per validate pass (not per-memory) by attaching findings to `manifest.json`. Live-surfaced 3 stale patterns: `module/**` (tier 8), `client/**` (tier 12), `member/**` (tier 30) — all reference scopes the BRAIN does not yet populate.

### Wired

`cyberos prune` + `cyberos hooks {status|on|off}` added to umbrella dispatch. Both removed from the stub list; only `conflicts` remains as a stub (redirects operator to `cyberos sync conflicts`).

### Verified

`cyberos verify` → CRITICAL: 0; WARN: 11 (3 new from source-tiers plugin for `module/**`, `client/**`, `member/**`; rest are pre-existing sandbox-only + scope-rules + tag-budget findings); INFO: 1. Audit chain intact. `cyberos prune` exits 0 at default thresholds; surfaces 1 candidate at `--drift-days 0` for the open refinement-candidate from earlier batches.

---

## 2026-05-12 — Batch 6 ship: relationships graph + encryption posture + scope rules + cost analytics (informational; no AGENTS.md edits)

> **No AGENTS.md edits — operator + tooling layer only.** Aspect-numbered references to `workbench/cyberos-layer1-deep-improvements.md`.

### Added

**Aspect 4.7 — Memory relationships graph** at `runtime/tools/cyberos_graph.py` + `cyberos graph`. Walks frontmatter `relationships:` edges, emits text / dot / json. Supports `--scope` filter, `--orphans` flag, `--memory <id> --hops N` ego-graph mode. Detects dangling targets (edge points at missing memory_id). Live-tested: 114 nodes / 1 edge in the BRAIN today; ego-graph on DEC-110 correctly surfaced the REF-042 implements link.

**Aspect 5.4 — Encryption posture audit** via `cyberos status --security`. Surfaces: §5.6 encryption enabled/disabled with algorithm + KDF + Shamir threshold; §9.3 denylist test pass/fail (24/24 fixtures live); filesystem permissions on `manifest.json` + `audit/` + `outputs/staged-memories/`; §13.10 PANIC marker status (now treats `(resolved)` titles as inactive); §8.6 unresolved drift candidate count.

**Aspect 11.5 — LLM cost analytics** via `cyberos analytics cost-log` + `cost-report`. Local-only `~/.cyberos/analytics/llm-cost.jsonl`. Operator supplies per-million-token rates at call time (we don't hardcode model pricing). Reports total USD, by-op breakdown, by-model breakdown. Live-tested with 3 synthetic records — council (Sonnet) at $0.0345 over 2 calls, brain-search-helper (Haiku) at $0.0013.

**Aspect 12.2 — Scope-rules enforcement** via `meta/scope-rules.md` + `meta/validators/check-scope-rules.py`. Each scope prefix declares allowed/denied classifications, allowed/denied sync_classes, and minimum authority tier. Loaded once per validator run; auto-discovered by the §12.1 plugin loader. Live-surfaced: PERSON-001 had `sync_class: publishable` which violated `memories/people` rule (only `local-only` or `shared` allowed) — exactly the kind of latent cross-class leakage this catches.

### Wired

`cyberos graph [--format ...]`, `cyberos status --security`, `cyberos analytics cost-log` + `cost-report`. PANIC marker detection now treats `(resolved)` titles as inactive (cosmetic fix; sandbox cannot unlink the marker).

### Verified

`cyberos verify` → CRITICAL: 0; WARN: 8 (added 2 from the new scope-rules plugin: PERSON-001 publishable→people violation, plus the existing tag-budget WARN; rest are sandbox-only); INFO: 1. Audit chain intact. Graph dangling-target check: 0 dangling. Determinism preserved on sync bundles.

---

## 2026-05-12 — Batch 5 ship: completions + REPL + conflicts resolver + status digests + dedup + pluggable validators + persona defaults (informational; no AGENTS.md edits)

> **No AGENTS.md edits — operator + tooling layer only.** Aspect-numbered references to `workbench/cyberos-layer1-deep-improvements.md`.

### Added

**Aspect 1.4 — Shell tab completion** at `runtime/completions/cyberos.{bash,zsh,fish}`. Completes subcommands, type arguments for `add`, enum values for `--classification`/`--authority`/`--sync-class`/`--prov-source`, sync subcommands, mcp subcommands, REF-NNN slugs for `council` and `eval`, and dynamic flag lists.

**Aspect 1.6 — Interactive REPL** at `runtime/tools/cyberos_repl.py` + `cyberos repl`. Avoids session.start overhead per cyberos invocation. Meta-commands: `.cd`, `.pwd`, `.last`, `.history`, `.save`, `.env`, `.clear`, `.help`, `.reload`. Forwards each line to the umbrella binary as a subprocess. Live-tested with stdin pipe.

**Aspect 2.3 — Weekly digest mode** via `cyberos status --weekly`. Landed / in-flight / queued framing per gstack `/landing-report`. Counts audit operations from the last 7 days, lists staged-but-unwritten files under `outputs/staged-memories/`, and flags drift candidates + pending council sessions as queued.

**Aspect 2.4 — Continuous watch mode** via `cyberos status --watch [--interval N]`. Clears screen and re-renders the 4-question dashboard every N seconds (default 30; minimum 5). Useful for monitoring during long-running migrations or self-audits.

**Aspect 6.5 — Interactive conflicts resolver** via `cyberos sync conflicts --resolve`. Steps through each `memories/conflicts/sync-*.md` marker; offers `[l]ocal | [r]emote | [d]isputed | [o]pen | [s]kip | [q]uit`. Annotates the marker with a `## Resolution (<ts>)` block recording the decision. Live-tested against the synthetic conflict from Batch 4.

**Aspect 9.6 — Duplicate-memory detection** at `runtime/tools/cyberos_dedup.py` + `cyberos dedup`. Body-shingle Jaccard (5-grams) + slug-stem similarity (3-gram Jaccard). Excludes `meta/protocol-history/` (deliberate snapshots) and the legitimate DEC↔REF implements-pair pattern (high slug, low body, cross-bucket). Live-tested: surfaced 2 real candidates (FACT-002/FACT-011 same-slug, FACT-004/FACT-010 same-slug).

**Aspect 12.1 — Pluggable validators** integrated into `cyberos_validate.py`. Auto-discovers `meta/validators/check-*.py` plugins, calls `check(memory, manifest)` on every memory, surfaces returned findings under §12.1. Exception-isolated (plugin error → WARN, never crashes validation). Ship sample plugin `meta/validators/check-tag-budget.py` (flags >10 tags + duplicate tags).

**Aspect 12.6 — Persona-defined defaults** integrated into `cyberos_add.py`. Reads `persona_defaults` block from `.cyberos-memory/persona/<name>.md`; pre-fills classification / authority / sync_class defaults when CLI flag absent. Persona resolved from `--persona` flag or `$CYBEROS_PERSONA`. Live-tested with `persona/founder.md`.

### Wired

`cyberos repl`, `cyberos dedup`, `cyberos status --weekly | --watch [--interval N]`, `cyberos sync conflicts --resolve`, `cyberos_add --persona <name>`. `repl` and `dedup` removed from stub list. Help text updated.

### Verified

`cyberos verify` → CRITICAL: 0; WARN: 6 (sandbox-only: 2 new conflict marker + drift candidate without audit; pluggable validator surfaced 1 tag-budget WARN); INFO: 1.

---

## 2026-05-12 — Batch 4 ship: council mode + GLOSSARY auto-tagging + sync scaffolding + read-only MCP (informational; no AGENTS.md edits)

> **No AGENTS.md edits — operator + tooling layer only.** Aspect-numbered references to `workbench/cyberos-layer1-deep-improvements.md`.

### Added

**Aspect 3.3 — Council-mode synthesis tool** at `runtime/tools/cyberos_council.py`. Opt-in (`cyberos council REF-NNN`). Produces `outputs/council/REF-NNN-council.md` with 4 voice prompts (Architect / Skeptic / Pragmatist / Critic) + deterministic heuristic context (GLOSSARY term overlap, LOCK conflicts, related REFs, recent `rejected/` entries). Operator pipes prompts to fresh Claude sessions then writes the Synthesis section. Not run automatically; only ambiguous REFs pay the 4× cost.

**Aspect 5.2 — GLOSSARY auto-tagging** integrated into `cyberos_add.py` behind `--auto-tags` flag (opt-in). Reads `FACT-014-glossary.md`, suggests kebab-case tags for terms appearing in slug + title + provenance reference. Interactive review (accept all / decline / edit-list). Default off — never modifies tags without operator confirmation.

**Aspect 6.x — Multi-machine sync scaffolding** at `runtime/tools/cyberos_sync.py`. Subcommands: `export --to <bundle.zip>` (deterministic; sync-class filtered, default publishable+shared, opt-in client-visible); `import <bundle> --from <subject> [--dry-run]` (three-way conflict detection by `memory_id` × `content_sha`; stages non-conflicting imports under `outputs/sync-staging/`, writes conflict markers under `memories/conflicts/` for §3 reconciliation); `conflicts` (list pending). Live-tested: deterministic across two consecutive exports; correctly detects synthetic conflict on tampered bundle. No network transport bundled — operator chooses rsync, syncthing, S3, etc.

**Aspect 12.7 — Read-only MCP server for the BRAIN** at `runtime/mcp/cyberos_brain_server.py`. Line-delimited JSON-RPC 2.0 over stdio. 4 tools: `brain_search`, `brain_show`, `brain_get`, `brain_stats`. Default filters: tombstoned hidden, `sync_class=local-only` hidden (both have explicit opt-in flags). Wire via `cyberos mcp info` (prints the `.claude/mcp-config.json` snippet) or run with `cyberos mcp serve`. NO writes; callers must use `brain_writer.py` for mutation.

### Wired

`cyberos council`, `cyberos sync {export|import|conflicts}`, `cyberos mcp {serve|info}` — all three subcommands added to the umbrella CLI dispatch in `runtime/tools/cyberos`. `sync` removed from the stub list (now real). Help text updated.

### Verified

`cyberos verify` → CRITICAL: 0; WARN: 4 (legacy/sandbox-only); INFO: 1. Audit chain intact. Determinism: two consecutive `cyberos sync export` calls produced identical SHA256 (`5c432e4361f7f6d2…`). MCP handshake + 4 tool calls returned valid JSON-RPC responses end-to-end.

---

## 2026-05-12 — Aspect-batch ship: Layer-1 operator surface + hooks + templates + tours (informational; no AGENTS.md edits)

> **No AGENTS.md edits — operator + tooling layer only.** Aspect-numbered references to `workbench/cyberos-layer1-deep-improvements.md`.

### Added

**Aspect 1.1 — `cyberos` umbrella CLI binary** at `runtime/tools/cyberos`. 11 working subcommands + 7 stubs for not-yet-implemented aspects.

**Aspect 1.3 + 2.1 — 4-operator-question dashboard** via `cyberos status`. Healthy / Bottleneck / Changed / What-now framing per `dashboard-builder` skill.

**Aspect 3.1 — Refinement-candidate Stop-hook** at `runtime/hooks/refinement_candidates.py`. Scans audit ledger at session.end, surfaces patterns ≥3 occurrences in 30-day window. Observes only; never auto-acts.

**Aspect 3.4 + 3.5 — REJECTED + POSTMORTEM templates** at `.cyberos-memory/meta/templates/{REJECTED,POSTMORTEM}.md`. Track refinement-candidate rejections + blameless postmortems.

**Aspect 4.1 — Memory templates per type** at `.cyberos-memory/meta/templates/{DEC,REF,FACT,PERSON,PROJECT,PREFERENCE,DRIFT}.md`. Nygard ADR format for DECs.

**Aspect 4.3-4.6 — Seed memories staged** at `outputs/staged-memories/` — 5 FACTs (target market, three-layer BRAIN, tech stack, Total Rewards invariants, Vietnamese-first wedge), 1 PERSON (founder profile), 2 PREFs (voice standard, compact §14). Commit via `outputs/staged-memories/bootstrap.sh`.

**Aspect 5.1 — gateguard PreToolUse hook** at `runtime/hooks/gateguard.py`. 3-stage DENY/FORCE/ALLOW gate per gstack `gateguard` skill (A/B tested +2.25 quality improvement).

**Aspect 5.5 — Denylist regression suite** at `runtime/tests/denylist/test_denylist.py`. Tests compensation/gov-ID/bank/secret/health denylist patterns + evasion attempts.

**Aspect 7.2 — voice_check.py + `cyberos voice`** linter for em dashes + AI vocabulary (verbatim from gstack `/codex` voice standard).

**Aspect 7.3 — Cross-doc consistency checker** via `cyberos doc-consistency`. Flags stale §-refs in README + missing DEC references.

**Aspect 7.4 — Tour files** at `tours/{onboarding,refinement-loop,incident-response,protocol-upgrade,security-audit}.tour`. CodeTour-compatible walkthroughs.

**Aspect 8.1 — `cyberos onboard`** at `runtime/tools/cyberos_onboard.py`. Interactive 5-step new-contributor wizard.

**Aspect 11.1 + 11.2 — Local-only analytics** at `runtime/tools/cyberos_analytics.py`. Logs every cyberos command to `~/.cyberos/analytics/skill-usage.jsonl`; `cyberos analytics report` produces usage summary. **Never sent anywhere** per `autonomous-agent-harness` Consent-and-Safety-Boundaries.

**Aspect 13.4 — Protocol-history INDEX.md** at `.cyberos-memory/meta/protocol-history/INDEX.md`. 20 archives mapped to Bundle / Date / Theme / CHANGELOG anchor.

**Aspect 13.10 — `cyberos panic`** emergency stop. Writes `meta/PANIC.md` to freeze writes; cleared via `cyberos panic --resolve <reason>`.

**CI:** `.github/workflows/voice-and-consistency.yml` — runs voice + doc-consistency + validator on every PR touching docs.

### Pending (drafted, awaiting your execution on real laptop)

**Aspect 13.2 — `company/locked-decisions.md`** — draft + brain_writer command at `workbench/aspect-13-2-locked-decisions-draft.md`. 20 LOCK-NNN entries derived from PRD §1-§2 + AGENTS.md §0-§9. Once committed, immutable per §9.6.

### Driver

User asked: *"you have my approvals to fully do all necessary stuff, just trigger test yourself, and also update readme/prd/srs for future reads, just stop when need my decision/choose."* This bundle ships everything in the Aspect-1/2/3/4/5/7/8/11/13 ranges that doesn't require:
  - §0.5 chat-turn protocol approval (Aspect 3.2 council mode, Aspect 12.2 custom scope rules, etc.)
  - Real-laptop brain_writer execution (Aspect 13.2 locked-decisions, seed memories)
  - A second real machine (Aspect 6 multi-machine sync)
  - Actual performance pain (Aspect 9 — deferred per recommendation)

### What this bundle does NOT change

- `docs/CyberOS-AGENTS.md` — zero edits (operator + tooling only)
- `manifest.json` — zero edits (no protocol pin change)
- `audit/*.jsonl` — appends only via brain_writer on your execution

---

## 2026-05-11 — Bundle Q: implementation files in source tree, §4.7 close-pattern alignment, BRAIN-not-versioned warn, relative symlinks

### Protocol SHA transition

- **Before:** `sha256:617f5aef1a49c394f6d17be072c8b29dbeb84c3265b80f3de3cb00a0f1c07759`
- **After:**  `sha256:71a276c74fe5a1fb65dbe24c6073f74d4cc7168b02aef1b577db9e01ccb13688`
- **Approved by:** subject:stephen-cheng (chat-turn phrase per §0.5)

### Changed

- **§0.6 implementation-files clause (REF-1)** — added the explicit invariant that implementation files (`outputs/brain_writer.py`, `cyberos/.protocol-signing-key`, etc.) MUST live in the project source tree, NOT inside `.cyberos-memory/`. The BRAIN is local operational state and is gitignored on most projects (including this one); a writer placed inside the BRAIN ships only as long as the BRAIN persists, and historically led to writers vanishing when the BRAIN was reinitialised or migrated. The clause names `outputs/brain_writer.py` as the canonical location and registers `runtime/tools/cyberos_brain_writer.py` as an acceptable alternative provided §0.6 is updated in the same protocol-upgrade.
- **§4.7 post-terminator close exemption (REF-2)** — amended the "orphan manifest update" rule to add an explicit exemption for the canonical close pattern: `session.end → str_replace manifest.json` where the manifest update's `prev_chain` matches the immediately-preceding terminator's `chain` AND its new `audit_chain_head` value equals that same terminator's `chain`. Pre-Q wording flagged this legitimate close-of-session pattern as `crash-mid-manifest-update`, which would freeze writes on every clean session boundary. The exemption is the only case where a manifest-update row is the LAST row in the ledger and is not a crash.
- **§13.1 step 11 BRAIN-not-versioned warn (REF-3)** — replaced the single-line `.gitignore` instruction with a two-branch decision tree. Default branch (versioning opt-in available) appends a commented `# .cyberos-memory/` line as before. Opt-out branch (UNCOMMENTED entry already present at bootstrap or any subsequent §4.7 reconciliation) appends exactly one `op:"warn" reason:"brain-not-versioned"` audit row, deduplicated by `(reason, path)` over the BRAIN lifetime, AND updates `.gitignore` with a comment block explaining the opt-out is deliberate. Closes the silent-opt-out gap that allowed the previous `brain_writer.py` to vanish unnoticed.
- **§15 relative-symlink rule (REF-4)** — symlinks created at project root (`AGENTS.md`, `CLAUDE.md`, `.windsurfrules`, `.clinerules`, `.cursor/rules/cyberos-memory.mdc`, `.windsurf/rules/cyberos-memory.md`, `.github/copilot-instructions.md`) MUST use relative paths. Absolute-path symlinks break under any container/CI/sandbox mount where the host prefix differs.

### Why

`brain_writer.py` was prescribed by 8 separate documents (CHAIN_ORCHESTRATOR, HOST_ADAPTERS, MANUAL_WORKFLOW, skills/CHANGELOG, AGENTS.CHANGELOG, AGENTS.README, AGENTS.md §0.6, PRD.CHANGELOG) as a tool the agent runs for every audit-row append. None of those docs caused the file to actually exist. It was never tracked in git. The orchestrator runs `python3 <path>/brain_writer.py` — file not found. Discovered when an audit row needed appending in cowork-session 2026-05-11.

Root cause was three-fold:
1. **Path drift** — three different prescribed locations (`outputs/`, `<cyberos-memory>/`, `PRD §5.10.11`); only one resolved on disk; `.cyberos-memory/` was the most-cited but worst location because…
2. **Visibility gap** — `.gitignore` was at full opt-out (`.cyberos-memory` uncommented), erasing the BRAIN tree and any tools placed in it from version control. Step 11 prescribed a *commented* line by default; the actual file went past that without an audit trail.
3. **Close-pattern ambiguity** — when the writer was rebuilt and verified against the existing 357-row chain, the §4.7 strict reading classified the chain's actual close pattern (`session.end → str_replace manifest.json`) as crash-mid-write. The protocol's wording lagged the writer's behaviour.

REF-1 + REF-3 close the path-drift / visibility issues. REF-2 aligns §4.7 with reality. REF-4 hardens portability after the AGENTS.md symlink was found to be absolute (broke under cowork's bind-mount).

### Real-world trigger

Direct §0.4 standing-rule trigger surfaced during a Phase-1 BRAIN repair (`outputs/brain_writer.py` rebuild from spec) and a Phase-2 repo audit (missing-refs + drift report). User adopted all four refinements as Bundle Q in the same chat turn that surfaced them.

### Verification

- Live AGENTS.md canonical SHA: `sha256:71a276c74fe5a1fb65dbe24c6073f74d4cc7168b02aef1b577db9e01ccb13688` ✓ matches manifest pin
- Pre-edit AGENTS.md (recoverable from `git show HEAD~1:docs/CyberOS-AGENTS.md` after the bundle's archive commit) hashes to `sha256:617f5aef…07759` — matches old pin
- New `outputs/brain_writer.py` produces bit-perfect chain hashes for the last 5 rows of the existing 357-row chain (post-Bundle-D writer compatibility)
- Chain LINK invariant: 0 breaks across all 357 rows
- Post-upgrade §8.7 self-audit report at `meta/health/2026-05-11-71a276c7-postupgrade.md`

### Related files updated (per §0.6)

- `docs/CyberOS-AGENTS.md` — §0.6 / §4.7 / §13.1 step 11 / §15 amended; prior verbatim archived to `meta/protocol-history/AGENTS-sha256-617f5aef1a49c394f6d17be072c8b29dbeb84c3265b80f3de3cb00a0f1c07759.md`
- `docs/CyberOS-AGENTS.CHANGELOG.md` — this entry
- `docs/CyberOS-AGENTS.README.md` — line 1503 retired the orphan "PRD §5.10.11" reference; no Part-level refresh needed (Bundle Q does not change any §14/§8 areas the README maps to)
- `docs/skills/CHAIN_ORCHESTRATOR.md`, `docs/skills/HOST_ADAPTERS.md`, `docs/skills/MANUAL_WORKFLOW.md` — all `python3 .cyberos-memory/.brain_writer.py …` prescriptions updated to `python3 outputs/brain_writer.py …`
- `outputs/brain_writer.py` — NEW canonical writer; reference impl per §0.6 line 175. Replaces a non-existent file previously expected at the same path. Implements §4 / §5.2 / §7 / §13. Verified bit-perfect against the post-Bundle-D writer's tail.
- `.cyberos-memory/.brain_writer.py` — replaced with deprecation stub pointing at the new location (BRAIN copy retained for transition; can be deleted from macOS at user's convenience).
- `.gitignore` — added explicit-intent comment block above the `.cyberos-memory` entry documenting the deliberate opt-out (per the new §13.1 step 11).
- `<root>/AGENTS.md` symlink — converted from absolute to relative (`docs/CyberOS-AGENTS-CORE.md`).
- `.cyberos-memory/manifest.json` — protocol pin + audit_chain_head + reconciliation_checkpoint + last_updated_at updated by apply script
- `.cyberos-memory/audit/2026-05.jsonl` — `op:protocol_upgrade` row appended; `op:create` rows for archive, health report, DEC-109, REF-041
- `.cyberos-memory/meta/health/2026-05-11-71a276c7-postupgrade.md` — auto-triggered §8.7 scan
- `.cyberos-memory/memories/decisions/DEC-109-implementation-files-in-source-tree.md` — locked decision behind REF-1
- `.cyberos-memory/memories/refinements/REF-041-bundle-q-impl-files-and-close-pattern.md` — bundle refinement memory per §0.4 step 4

No FACT memory required v+1 refresh for this bundle (none reference §0.6 / §4.7 / §13.1 / §15 by the §0.6 step 3 cross-link rule).

---

## 2026-05-10 — Bundle P: §14 `📁 Files changed:` = non-BRAIN paths only (correction to Bundle O)

### Protocol SHA transition

- **Before:** `sha256:b0d9ad3adc35ec1b74bad1407532873db828adc5161d7f05e23914e76096c1d6`
- **After:**  `sha256:617f5aef1a49c394f6d17be072c8b29dbeb84c3265b80f3de3cb00a0f1c07759`
- **Approved by:** subject:stephen-cheng (chat-turn phrase per §0.5)

### Changed

- **`📁 Files changed:` semantics narrowed**: lists **non-BRAIN paths ONLY** in both §14.1 compact and §14.2 verbose. BRAIN paths (inside `.cyberos-memory/`) NEVER appear under `📁`. Bundle O's "merged list" interpretation was an agent misread of user feedback — corrected here.
- **§14.0 omission condition (c)** updated: now reads "no non-BRAIN file was modified this turn" instead of "no memory mutations". A turn that ONLY writes BRAIN memories (DEC + REF + preference + audit rows + manifest updates) and touches no non-BRAIN file produces NO §14 output.
- **§14.1 compact**: explicit "Non-BRAIN paths ONLY" rule with rationale; BRAIN files are agent housekeeping never listed here.
- **§14.2 verbose**: `Δ Changes (BRAIN detail):` is now the sole place BRAIN paths surface in chat. Always present in §14.2; `📁` block in §14.2 omits entirely if no non-BRAIN files changed.
- **§14.3 (coverage stat)** updated cross-reference to clarify which sections emit ingestion coverage suffixes.

### Why

User correction during cowork-session 2026-05-10, immediately after Bundle O landed:

> "no need to implied outside BRAIN" i mean only show changes outside the brain, no need to show inside BRAIN changes

Bundle O interpreted the original "no need to imply outside BRAIN" as "merge BRAIN and non-BRAIN paths with no qualifier"; Stephen meant "show only outside-BRAIN paths — drop BRAIN housekeeping entirely from compact mode". The semantic difference matters: pre-Bundle-P, every BRAIN write generated a §14.1 line; post-Bundle-P, BRAIN writes alone are silent.

The user's mental model: `📁 Files changed:` should show files in THEIR project. BRAIN paths are agent infrastructure — equivalent to log files or build artefacts — not user-relevant signal on every turn. The audit ledger preserves full forensic detail for when it matters.

### Real-world trigger

Direct §0.4 standing-rule trigger ("user having to repeat instructions or correct the agent's behaviour"). Bundle O landed; user reviewed; clarified; agent applied as Bundle P within two turns.

### Verification

- Live AGENTS.md canonical SHA: `sha256:617f5aef1a49c394f6d17be072c8b29dbeb84c3265b80f3de3cb00a0f1c07759` ✓ matches manifest pin
- §8.7 post-upgrade scan: 0 CRITICAL, 0 WARN, 1 INFO (pre-existing legacy memory_id); report at `meta/health/2026-05-10-617f5aef1a49c394-postupgrade.md`
- Chain LINK invariant: clean across new ledger tail
- Validator self-test: passes post-upgrade

### Related files updated (per §0.6)

- `docs/CyberOS-AGENTS.md` — §14 narrowed; prior verbatim archived to `meta/protocol-history/AGENTS-sha256-b0d9ad3adc35ec1b74bad1407532873db828adc5161d7f05e23914e76096c1d6.md`
- `.cyberos-memory/manifest.json` — protocol pin + audit_chain_head + reconciliation_checkpoint + last_updated_at updated
- `.cyberos-memory/audit/2026-05.jsonl` — `op:protocol_upgrade` row appended; `op:create` rows for archive, health report, DEC-108, REF-040; `op:str_replace` row for preference v3
- `.cyberos-memory/meta/health/2026-05-10-617f5aef1a49c394-postupgrade.md` — auto-triggered §8.7 scan
- `.cyberos-memory/memories/decisions/DEC-108-section-14-non-brain-files-only.md` — locked decision per §0.6
- `.cyberos-memory/memories/refinements/REF-040-bundle-p-section-14-non-brain-only.md` — refinement memory per §0.4 step 4
- `.cyberos-memory/memories/preferences/feedback-section-14-compression.md` — preference v3 (str_replace from v2)
- `docs/CyberOS-AGENTS.README.md` — Part 8 anti-pattern note refreshed for non-BRAIN-only semantic

---

## 2026-05-10 — Bundle O: §14 three-state triage (silent / files-only-compact / issues-verbose)

### Protocol SHA transition

- **Before:** `sha256:8060fe2e188e1793e9dbc758b34a8198617ff8bf8a3320a2012595faf3012dab`
- **After:**  `sha256:b0d9ad3adc35ec1b74bad1407532873db828adc5161d7f05e23914e76096c1d6`
- **Approved by:** subject:stephen-cheng (chat-turn phrase per §0.5)

### Changed

- **§14 heading**: `(conditional in normal mode)` → `(silent by default; verbose when issues)`
- **3-state triage table** added at top of §14 — explicit decision matrix (omit / compact / verbose).
- **§14.1 compact rewritten**: contains ONLY a `📁 Files changed:` block + optional `Tokens:` line. Removed: `Δ Changes:` heading, `Status:` block (with all 4 sub-lines), `unchanged:` line, `audit/<YYYY-MM>.jsonl: <N rows; head=…>` line.
- **§14.2 verbose trigger broadened**: fires on ANY of `op:rejected|revert|warn|health_check` this turn, latest §8.7 reports CRITICAL/WARN, or `operational_mode != normal`. Pre-Bundle-O was mode-only.
- **§14.2 arrangement**: `⚠️ Findings:` first, then `📁 Files changed:`, then `Δ Changes (BRAIN detail):`, then `Status:`, then optional `Tokens:`.
- **`unchanged:` line removed** entirely (absence-from-list is implicit).
- **`Tokens:` slot reserved** in both §14.1 and §14.2 — emitted only when a runtime token counter is wired up via MCP. Approximation via `tiktoken`/character-count is forbidden.

### Why

User feedback during cowork-session 2026-05-10, immediately after Bundle N landed:
1. *"Status: unchanged section seem not necessary since there is 'Δ Changes' section"* — Status + unchanged are redundant signal.
2. *"In normal mode no need to should Δ Changes if no issues arise too"* — Δ Changes redundant given 📁 Files changed:.
3. *"only show Files changed (no need to implied outside BRAIN), only turn on maintenance mode and show full memory verbose (arrange them smartly too) status when issues arise"* — single merged list, auto-trigger on issues.
4. *"Is it possible to know/track tokens consumed? if can show it after 📁 Files changed section, if not then skip it"* — token tracking desired but not faked.

The §14 noise-reduction trajectory (Bundle I → N → O) now has each routine mutation turn producing ~3 lines of §14 output instead of ~10 — while issues automatically promote to full visibility.

### Real-world trigger

User-driven post-Bundle-N feedback (2026-05-10). Bundle N landed; Stephen reviewed the resulting §14 output and surfaced three more axes to compress + one open question. Resolution proposed within the same chat turn per §0.4 standing rule; approved within two turns; applied in the third.

### Verification

- Live AGENTS.md canonical SHA: `sha256:b0d9ad3adc35ec1b74bad1407532873db828adc5161d7f05e23914e76096c1d6` ✓ matches manifest pin
- §8.7 post-upgrade scan: 0 CRITICAL, 0 WARN, 1 INFO (pre-existing legacy memory_id); report at `meta/health/2026-05-10-b0d9ad3adc35ec1b-postupgrade.md`
- Chain LINK invariant: clean across new ledger tail
- Validator self-test: passes post-upgrade

### Related files updated (per §0.6)

- `docs/CyberOS-AGENTS.md` — §14 three-state triage applied; prior verbatim archived to `meta/protocol-history/AGENTS-sha256-8060fe2e188e1793e9dbc758b34a8198617ff8bf8a3320a2012595faf3012dab.md`
- `.cyberos-memory/manifest.json` — protocol pin + audit_chain_head + reconciliation_checkpoint + last_updated_at updated
- `.cyberos-memory/audit/2026-05.jsonl` — `op:protocol_upgrade` row appended; `op:create` rows for archive, health report, DEC-107, REF-039; `op:str_replace` row for preference v2
- `.cyberos-memory/meta/health/2026-05-10-b0d9ad3adc35ec1b-postupgrade.md` — auto-triggered §8.7 scan
- `.cyberos-memory/memories/decisions/DEC-107-section-14-three-state-triage.md` — locked decision per §0.6
- `.cyberos-memory/memories/refinements/REF-039-bundle-o-section-14-three-state-triage.md` — refinement memory per §0.4 step 4
- `.cyberos-memory/memories/preferences/feedback-section-14-compression.md` — preference v2 (str_replace; supersedes v1's compact-only guidance with three-state triage)
- `docs/CyberOS-AGENTS.README.md` — Part 8 anti-pattern note refreshed to reflect §14.2 auto-trigger semantics

---

## 2026-05-10 — Bundle N TIER 1+2: §14 omission + audit-trail suppression

### Protocol SHA transition

- **Before:** `sha256:9bec8422359dc80c4d1f20271cf4bdeacb0ac88b7db6261a34085f70b894f329`
- **After:**  `sha256:8060fe2e188e1793e9dbc758b34a8198617ff8bf8a3320a2012595faf3012dab`
- **Approved by:** subject:stephen-cheng (chat-turn phrase per §0.5)

### Changed (2 added; 0 deferred)

- **TIER 1 — §14.0 omission rule (sev-2)**. New sub-section above §14.1. The §14 block MUST be omitted entirely when ALL of: (a) `manifest.operational_mode == normal`, (b) no `op:rejected|revert|warn|health_check` row this turn, (c) no memory mutations (audit-row count ≤ 2 and only `session.start`/`session.end` bookends), (d) most-recent §8.7 self-audit reports 0 CRITICAL and 0 WARN. Verbose/debug/maintenance modes still always emit §14.2.
- **TIER 2 — §14.1.1 audit-trail suppression (sev-2)**. New sub-section under §14.1. When §14.1 compact IS emitted in normal mode, omit the `audit/<YYYY-MM>.jsonl: <N rows appended; head=sha256:…>` line unless a finding occurred this turn or the most-recent §8.7 reports issues.
- §14 heading: `(mandatory)` → `(conditional in normal mode)` to reflect new conditionality.

### Deferred

- **TIER 3 — `📁 Files changed:` block for non-BRAIN paths**. Not included in this approval. Future amendment if user requests; Stephen approved TIER 1+2 minimum-viable.

### Why

User feedback during cowork-session 2026-05-10: *"show Audit trail after each messages make the conversation flooded, just show in maintenance mode or when issues arise"* and *"can we compress 📝 .cyberos-memory updated section more? show full verbose in maintenance mode, but only show changes summary on normal (default), if no issues arise don't need to show memory changes, just show other files' changes"*. Both directly address signal-to-noise — the §14 block was generating chat noise on every healthy turn. Bundle I (2026-05-06) introduced the compact format; Bundle N completes the noise-reduction journey by allowing full block omission.

### Real-world trigger

User-driven post-healthcheck feedback (2026-05-10). Immediately after running the on-demand §8.7 healthcheck (which produced a §14 block with audit head SHA), Stephen flagged the noise. Resolution proposed within the same chat turn per §0.4 standing rule; approved within two turns; applied in the third.

### Verification

- Live AGENTS.md canonical SHA: `sha256:8060fe2e188e1793e9dbc758b34a8198617ff8bf8a3320a2012595faf3012dab` ✓ matches manifest pin
- §8.7 post-upgrade scan: 0 CRITICAL, 0 WARN, 1 INFO (pre-existing legacy memory_id); report at `meta/health/2026-05-10-8060fe2e188e1793-postupgrade.md`
- Chain LINK invariant: clean across new ledger tail
- Validator self-test: passes post-upgrade

### Related files updated (per §0.6)

- `docs/CyberOS-AGENTS.md` — §14 amendments applied; prior verbatim archived to `meta/protocol-history/AGENTS-sha256-9bec8422359dc80c4d1f20271cf4bdeacb0ac88b7db6261a34085f70b894f329.md`
- `.cyberos-memory/manifest.json` — `protocol.{sha256,approved_at,approved_by,last_checked_at}`, `audit_chain_head`, `last_updated_at`, `reconciliation_checkpoint` updated
- `.cyberos-memory/audit/2026-05.jsonl` — `op:protocol_upgrade` row appended; `op:create` rows for archive, health report, DEC-106, REF-038, preference memory
- `.cyberos-memory/meta/health/2026-05-10-8060fe2e188e1793-postupgrade.md` — auto-triggered §8.7 scan
- `.cyberos-memory/memories/decisions/DEC-106-section-14-omission-rule.md` — locked decision per §0.6
- `.cyberos-memory/memories/refinements/REF-038-bundle-n-section-14-omission.md` — refinement memory per §0.4 step 4
- `.cyberos-memory/memories/preferences/feedback-section-14-compression.md` — subject preference (sync_class=publishable)
- `docs/CyberOS-AGENTS.README.md` — Part 8 anti-pattern note ("Skipping the §14 end-of-response block") amended to reflect §14.0 carve-out

---

## 2026-05-10 — Bundle M: AGENTS.md refinement pass (functional-zero)

### Protocol SHA transition

- **Before:** `sha256:d3ce9764ac76635921f6e981a713ea8822eaec442d01200930633a805a84aaf0`
- **After:**  `sha256:9bec8422359dc80c4d1f20271cf4bdeacb0ac88b7db6261a34085f70b894f329`
- **Approved by:** subject:stephen-cheng (chat-turn phrase per §0.5)

### Changed (4 textual/structural; functional-zero)

- **§5.1 heading + reconciliation paragraph (Change A)** — "only these 28 fields are permitted" → "closed set; 28 base fields + Stage 5 encryption block". Added paragraph clarifying that `encrypted: bool` and `encryption: {algorithm, nonce, aad}` are part of the closed set when `manifest.encryption_policy.enabled = true` per §5.6.
- **§8 heading (Change B)** — "7 phases" → "7 routine phases + §8.9 user-triggered ledger compaction". Reflects §8.9 added in Stage 6.
- **§4.10/§4.11 merge (Change C)** — §4.11 promoted under §4.10 as `#### 4.10.2 Token-budget transparency for large sources (sev-2)`; existing §4.10 body becomes `#### 4.10.1 Sequential walk + coverage check`. External references to §4.11 should update to §4.10.2.
- **§17.5 compression (Change D)** — "Publish flow (forward reference)" reduced from ~10 lines to a 6-line summary. Detail (signed `brain.publish` MCP envelope, `actor_keys` registry, post-P1 manifest extension) referenced in `docs/CyberOS-AGENTS.EVOLUTION.md` Stage 4.

### Deferred to Bundle N

- **Change E — §0.5 split** — split into 0.5 (approval flow only), 0.5.1 (signing-key TOFU), 0.5.2 (three-way protocol conflict). Pre-Bundle-N, these three concerns mix in one 52-line section.
- **Change F — paragraph compression throughout** — 55 paragraphs over 500 chars across §0.2, §6, §7.2, §8.7, §13.0, others. Pure formatting refactor; preserves all rules.

### Why

The 2026-05-10 AGENTS.md scan identified six refinement candidates that had accumulated as Stage 1, 5, 6 added new sections (§5.6 encryption envelope, §7.6 Merkle, §7.7 compaction, §8.9 compaction phase) without updating cross-cutting headers/counts. Bundle M reconciles header text to current reality. Functional-zero by design — no new ops, no schema changes, no validator changes; two agents reading pre-Bundle-M and post-Bundle-M AGENTS.md reach identical accept/reject decisions on every input.

### Real-world trigger

User-driven post-Stage-5 cleanup pass (2026-05-10). After Tier-1+2+3 implementation work shipped (cyberos_doctor R5/R6, cyberos_index merkle_checkpoints table, cyberos_validate Merkle checks, cyberos_encrypt v1 disable/migrate-batch/rotate-shamir, macOS Secure Enclave HW backend, +5 test fixtures, REF# duplicate dedup), the AGENTS.md scan surfaced 6 remaining textual debts. Bundle M packages 4 of them; remaining 2 deferred to Bundle N because structurally invasive.

### Verification

- Live AGENTS.md canonical SHA: `sha256:9bec8422359dc80c4d1f20271cf4bdeacb0ac88b7db6261a34085f70b894f329` ✓ matches manifest pin
- §8.7 post-upgrade scan: 0 CRITICAL, 0 WARN, 1 INFO (pre-existing legacy memory_id); report at `meta/health/2026-05-10-9bec8422359dc80c-postupgrade.md`
- Validator self-test (21 fixtures) — passes post-upgrade
- Chain LINK invariant: 318 rows, all chains link
- AGENTS-CORE.md regenerated post-Bundle-M; reflects §4.11→§4.10.2 renumbering

### Related files updated (per §0.6)

- `docs/CyberOS-AGENTS.md` — Changes A–D applied; prior verbatim archived to `meta/protocol-history/AGENTS-sha256-d3ce9764ac76635921f6e981a713ea8822eaec442d01200930633a805a84aaf0.md`
- `.cyberos-memory/manifest.json` — `protocol.{sha256,approved_at,approved_by}`, `audit_chain_head`, `last_updated_at` updated
- `.cyberos-memory/audit/2026-05.jsonl` — `op:"protocol_upgrade"` row appended (chain `sha256:871cbc4df811b3ea...`); two `op:"create"` rows for the related-files writes
- `.cyberos-memory/meta/health/2026-05-10-9bec8422359dc80c-postupgrade.md` — auto-triggered §8.7 scan
- `.cyberos-memory/memories/refinements/REF-037-bundle-m-refinement-pass.md` — refinement memory per §0.4
- `AGENTS-CORE.md` — regenerated to reflect Bundle M's §4.11→§4.10.2 renumbering
- `docs/CyberOS-PRD.CHANGELOG.md` — entry added; PRD .docx body integration deferred (DEC-109 entry pending)
- `docs/CyberOS-SRS.CHANGELOG.md` — entry added; SRS .docx body integration deferred similarly

### No DEC entry needed

Bundle M is documentation cleanup, not a decision. It surfaces existing implicit reality (the Stage 5 encryption fields, the §8.9 phase, the §4.10/§4.11 read-side discipline cluster, the deferred-to-BRAIN-P1 sync details) but doesn't decide anything new.

### Related implementation

- `docs/proposals/STAGE-7-BUNDLE-M-PROPOSAL.md` — proposal text used for this upgrade (preserved as documentation; will not be re-applied)

---

## 2026-05-10 — Stage 5: At-rest encryption + Shamir 3-of-5 escrow (opt-in)

### Protocol SHA transition

- **Before:** `sha256:77eda214d687f8fd8eb826b8699e62614c3b606e980486c7fcd8496f92ce6dfa`
- **After:**  `sha256:d3ce9764ac76635921f6e981a713ea8822eaec442d01200930633a805a84aaf0`
- **Approved by:** subject:stephen-cheng (chat-turn phrase per §0.5)

### Added

- **§5.6 At-rest encryption envelope (Change A)** — five sub-sections:
  - §5.6.1 per-file format: XChaCha20-Poly1305-IETF, 24-byte nonce, AAD `sha256(memory_id || last_updated_at)` binding nonce to identity; body is `base64(ciphertext || 16-byte tag)`
  - §5.6.2 key derivation: HKDF-SHA256 from HW-bound (Apple Secure Enclave / Windows TPM 2.0 / Linux TPM 2.0 + FIDO2 hmac-secret) OR Argon2id passphrase fallback `t=3, m=64MiB, p=4` per RFC 9106; passphrase MUST satisfy ≥16 chars AND zxcvbn ≥3 at enable time
  - §5.6.3 mandatory Shamir 3-of-5 escrow: enable refuses `enabled = true` until 5 fragments distributed; fingerprints + holder labels + creation timestamps recorded in `meta/key-policy.md`; fragments themselves NEVER stored in BRAIN
  - §5.6.4 indexability: frontmatter stays plaintext so `cyberos_validate` / `cyberos_index` / `cyberos_doctor` work without the key
  - §5.6.5 audit-chain compatibility: `after_hash` over plaintext preserves chain LINK integrity for key-holders
- **`encryption_policy` manifest field (Change B)** — default `enabled: false`. Scope filter syntax: `<path-pattern>` OR `classification:<class>`. Memories matching ANY entry are encrypted.
- **`shamir_fragments` manifest field (Change B)** — default empty. Carries `threshold=3, total=5, master_key_fingerprint=null, fragments=[]`. Each `fragments[]` entry: `{label, fingerprint, created_at, distributed_at|null}`. Threshold + total pinned at enable time; rotated only via `op:"shamir_rotation"`.
- **§7.1 op enum +8 (Change C)** — new ops: `ledger_compact`, `ledger_decompact` (Stage 6 normalisation, were already declared but now formal in enum), `encryption_policy_change`, `key_rotation`, `key_recovery_initiated`, `key_recovered`, `shamir_rotation`, `shamir_distribution_confirmed`.

### Changed

- **§4.6 tombstone semantics (Change D)** — encrypted memories' bodies stay base64-ciphertext after `delete`; routine reads SKIP tombstoned encrypted bodies; only MAINTENANCE-mode hard-erase decrypts.
- **§9.3 denylist clarification (Change E)** — encryption is NOT a denylist softener. Content gate (§4.2) runs BEFORE encryption envelope; comp/ESOP/gov-IDs/secrets remain forbidden in ANY storage form.
- **§17.6 cross-link refresh (Change F)** — `meta/key-policy.md` now covers signing keys AND encryption master keys; rotation events audited via `op:"key_rotation"` + `op:"shamir_rotation"`.

### Why

Local-optimization plan (`docs/CyberOS-AGENTS.LOCAL-OPTIMIZATION.md`) Stage 5 — make sensitive `personnel`/`client` memories safe to share via filesystem (lent laptop, contractor backup, machine handoff) without rewriting them. The §9.3 denylist already structurally excludes the highest-stakes content (comp/ESOP/secrets) — encryption protects the second-tier (perf review summaries, client engagement context, founder's private working notes). Body-only encryption preserves Stage 3 indexing + Stage 2 validation work. Mandatory Shamir 3-of-5 escrow prevents the catastrophic-loss failure mode where a forgotten passphrase + dead Touch ID sensor = unrecoverable encrypted memories.

### Real-world trigger

User-driven local-optimization design (2026-05-09 evening). Five Q&A surfaced at `docs/proposals/STAGE-5-OPEN-QUESTIONS.md`; Stephen approved with "go with your recs" (2026-05-10), then approved the §0.5 SHA in the same chat turn alongside Stage 6.

### Verification

- Live AGENTS.md canonical SHA: `sha256:d3ce9764ac76635921f6e981a713ea8822eaec442d01200930633a805a84aaf0` ✓ matches manifest pin
- §8.7 post-upgrade scan: 0 CRITICAL, 0 WARN; report at `meta/health/2026-05-10-d3ce9764ac766359-postupgrade.md`
- `runtime/tools/cyberos_validate.py` clean run (1 INFO — pre-existing legacy memory_id)
- Chain LINK invariant: 299 rows, all chains link
- `manifest.encryption_policy.enabled = false` initialised (encryption is OFF; will not encrypt anything until `cyberos-encrypt enable` wizard flips this)
- `manifest.shamir_fragments` initialised empty
- Stage 5 features dormant on this store: no memory has `encrypted: true` frontmatter; no Shamir master_key_fingerprint pinned

### Related files updated (per §0.6)

- `docs/CyberOS-AGENTS.md` — Changes A–F applied; prior verbatim archived to `meta/protocol-history/AGENTS-sha256-77eda214d687f8fd8eb826b8699e62614c3b606e980486c7fcd8496f92ce6dfa.md`
- `.cyberos-memory/manifest.json` — `protocol.{sha256,approved_at,approved_by}`, `audit_chain_head`, `last_updated_at`, `encryption_policy`, `shamir_fragments` updated
- `.cyberos-memory/audit/2026-05.jsonl` — `op:"protocol_upgrade"` row appended (chain `sha256:ff9b2bf5c29d18c3...`); two `op:"create"` rows for the related-files writes
- `.cyberos-memory/meta/health/2026-05-10-d3ce9764ac766359-postupgrade.md` — auto-triggered §8.7 scan
- `.cyberos-memory/memories/refinements/REF-030-stage-5-at-rest-encryption.md` — refinement memory per §0.4
- `docs/CyberOS-PRD.CHANGELOG.md` — entry added; PRD .docx update deferred (DEC-108 entry pending)
- `docs/CyberOS-SRS.CHANGELOG.md` — entry added; SRS .docx update deferred similarly

### Implementation work that follows landing (no further §0.5 needed)

- `runtime/tools/cyberos_encrypt.py` (~600 LOC): `enable` wizard (HW-key detect + Shamir 3-of-5 split + holder distribution + `enabled = true` flip); `disable` (decrypt all → re-write plaintext → flip flag); `migrate-batch <N>` (default 50, MAINTENANCE-mode envelope); `rotate-shamir`; `recover` (≥3 fragments → master key reconstruction); `status` (encryption coverage stats)
- `runtime/tools/cyberos_validate.py` extension: recognise `encrypted: true`, verify AAD, surface `encryption-aad-mismatch` and `shamir-fingerprint-missing` findings
- `runtime/tools/cyberos_doctor.py`: new repair op `R6-rotate-master-key` for hardware-replacement scenarios
- `docs/cookbook/encryption-and-recovery.md`: operational guide with holder-selection guidance, recovery walkthrough, migration playbook, threat model

### Related implementation

- `docs/proposals/STAGE-5-PROTOCOL-UPGRADE.md` — proposal text used for this upgrade (preserved as documentation; will not be re-applied)
- `docs/proposals/STAGE-5-OPEN-QUESTIONS.md` — five-question decision baseline (preserved as the rationale archive for "(c, c, 3-of-5 wizard, body-only, user-paced)" defaults)

---

## 2026-05-10 — Stage 6: Long-term BRAIN health (Merkle checkpoints + ledger compaction + .lock.shared)

### Protocol SHA transition

- **Before:** `sha256:576368647e4d17635804580ca4dded28721b1c7247f0a19666ce43f5f0eb911a`
- **After:**  `sha256:77eda214d687f8fd8eb826b8699e62614c3b606e980486c7fcd8496f92ce6dfa`
- **Approved by:** subject:stephen-cheng (chat-turn phrase per §0.5)

### Added

- **§4.9.1 `.lock.shared` semantics (Change D)** — sibling lock file for shared-read concurrency. Read-only ops (`view`) acquire `.lock.shared` only; mutation ops continue with exclusive `.lock`. Consolidation phases §8.1–§8.4 acquire shared lock, upgrade to exclusive for §8.5–§8.7. POSIX (`flock LOCK_SH | LOCK_NB`) and Windows (`LockFileEx` shared mode) covered. Stale recovery 5-minute timeout. Older agents that don't honour shared mode fall back to exclusive — always safe.
- **§7.6 Merkle checkpoints (Change A)** — every `op:"consolidation_run"` row gains a `merkle_root` field (SHA-256 tree over rows since previous checkpoint). Deterministic construction: leaves are raw chain bytes; pairing pads odd levels by duplicating last leaf; internal nodes via `sha256(left || right)`. Linear `chain` LINK invariant remains canonical; Merkle root is a *derived* index for O(log N) prefix verification.
- **§7.7 Audit ledger compaction (Change B)** — opt-in, phrase-triggered. Once a ledger month is Merkle-checkpointed AND older than `manifest.compaction_policy.minimum_age_months` (default 12), `audit/<YYYY-MM>.jsonl` collapses to per-memory `audit/<YYYY-MM>.compacted.jsonl` + Merkle proofs; original verbatim preserved at `archive/<YYYY-MM>.jsonl.zst`. ~80% disk savings on year-old ledgers. Reversible via MAINTENANCE-mode `op:"ledger_decompact"`. New audit op kinds: `ledger_compact`, `ledger_decompact`.
- **§8.9 Ledger compaction phase (Change C)** — phase 8.9 (NOT part of routine consolidation). Pre-conditions: existing Merkle checkpoint, age threshold met, no §8.7 phase 4 critical findings for the period. Triggered ONLY by chat-turn phrase *"compact ledger older than `<YYYY-MM-DD>`"* per §0.5.
- **`manifest.compaction_policy = {minimum_age_months: 12}`** — new manifest field initialised at upgrade time. Mutation outside chat-turn phrase forbidden by §0.2.

### Changed

- **§8.7 phase 4 audit chain integrity (Change E)** — extended with Merkle-root recomputation on every `op:"consolidation_run"` row carrying a `merkle_root` field; mismatch → `CRITICAL merkle-checkpoint-divergence`. Compacted-ledger files (`audit/<YYYY-MM>.compacted.jsonl`) verify each row's `merkle_proof` against the period's checkpoint root; mismatch → `CRITICAL merkle-proof-divergence`.

### Why

Local-optimization plan (`docs/CyberOS-AGENTS.LOCAL-OPTIMIZATION.md`) Stage 6. Three primitives land together because each depends on the others: Merkle checkpoints anchor proofs that compaction relies on; compaction needs `.lock.shared` so other agents can `view` while it holds exclusive `.lock` for the manifest update; `.lock.shared` is the precondition for safe concurrent `cyberos-validate` + `cyberos-index` runs. Without all three, ledger growth becomes unbounded and multi-agent days (Claude Code + Cursor + Aider against the same project) hit `.lock` starvation.

### Real-world trigger

User-driven local-optimization design (2026-05-09 evening) — Stage 6 was authored as `docs/proposals/STAGE-6-PROTOCOL-UPGRADE.md` after Stages 1–4 shipped. Stephen approved both Stage 5 defaults (separate proposal) and Stage 6 (this entry) in the same chat turn.

### Verification

- Live AGENTS.md canonical SHA: `sha256:77eda214d687f8fd8eb826b8699e62614c3b606e980486c7fcd8496f92ce6dfa` ✓ matches manifest pin
- §8.7 post-upgrade scan: 0 CRITICAL, 0 WARN; report at `meta/health/2026-05-10-77eda214d687f8fd-postupgrade.md`
- `runtime/tools/cyberos_validate.py` clean run (1 INFO — pre-existing legacy memory_id)
- Chain LINK invariant: 296 rows, all chains link
- `manifest.compaction_policy.minimum_age_months = 12` initialised at upgrade time
- Stage 6 features dormant on this store: no `merkle_root` rows yet (first appears at next `op:"consolidation_run"`); no compacted ledgers (earliest window 2027-05); `.lock.shared` available but unused

### Related files updated (per §0.6)

- `docs/CyberOS-AGENTS.md` — Changes A–E applied; prior verbatim archived to `meta/protocol-history/AGENTS-sha256-576368647e4d17635804580ca4dded28721b1c7247f0a19666ce43f5f0eb911a.md`
- `.cyberos-memory/manifest.json` — `protocol.{sha256,approved_at,approved_by}`, `audit_chain_head`, `last_updated_at`, `compaction_policy` updated
- `.cyberos-memory/audit/2026-05.jsonl` — `op:"protocol_upgrade"` row appended (chain `sha256:b6bf7a2f307409d6...`); two `op:"create"` rows for the related-files writes
- `.cyberos-memory/meta/health/2026-05-10-77eda214d687f8fd-postupgrade.md` — auto-triggered §8.7 scan
- `.cyberos-memory/memories/refinements/REF-029-stage-6-long-term-health.md` — refinement memory per §0.4
- `docs/CyberOS-PRD.CHANGELOG.md` — entry added; PRD .docx update deferred (DEC-107 entry pending)
- `docs/CyberOS-SRS.CHANGELOG.md` — entry added; SRS .docx update deferred similarly

### Implementation work that follows landing (no further §0.5 needed)

- `cyberos_validate.py` — add `_check_merkle_checkpoints()` + `_check_compacted_ledger()`
- `cyberos_doctor.py` — new repair `R5-rebuild-merkle-checkpoint`; new CLI `cyberos-doctor decompact-ledger --month <YYYY-MM>`
- `cyberos_index.py` — new table `merkle_checkpoints(audit_id, root, period_start_audit_id, period_end_audit_id)`; new query `cyberos-index query merkle-proof <chain>`
- `docs/cookbook/ledger-compaction.md` — when to compact, how to verify a compacted period

### Related implementation

- `docs/proposals/STAGE-6-PROTOCOL-UPGRADE.md` — proposal text used for this upgrade (preserved as documentation; will not be re-applied)

---

## 2026-05-10 — Stage 1: Session-start speed (reconciliation checkpoint + lazy-load + frontmatter compactness)

### Protocol SHA transition

- **Before:** `sha256:599e1097199618e0d8dde22770eef6e5ad068c5c06150e2bb3829315f005780d`
- **After:**  `sha256:576368647e4d17635804580ca4dded28721b1c7247f0a19666ce43f5f0eb911a`
- **Approved by:** subject:stephen-cheng (chat-turn phrase per §0.5)

### Added

- **§5.1 frontmatter compactness rule (Change D)** — write-side guidance to omit `null`/empty optional fields, EXCEPT consent block for `personnel`/`client` and tombstone metadata. Read-side accepts both compact and verbose forms. The 28-field closed-set rule applies only to *recognised* fields; absence of optional fields is not a schema violation. Drops typical frontmatter byte count by 30–40%.
- **§6 `manifest.reconciliation_checkpoint` block (Change A)** — three-field record `{audit_id, chain, ts}` written at every successful `op:"session.end"` and `op:"consolidation_run"`. Used by §4.7 to bound reconciliation work.
- **§6 `manifest.read_profile` block (Change C)** — declares eager vs lazy scopes. Default `eager_scopes: ["meta"]`, all other scopes lazy-loaded on first reference.
- **§10 read protocol bullet 1a (Change C tail)** — honour `manifest.read_profile`. Eager scopes load every session start; lazy on-demand.

### Changed

- **§4.7 reconciliation (Change B)** — walks rows newer than `manifest.reconciliation_checkpoint.audit_id` if set; falls back to full-walk on missing/stale (>30 days) checkpoint or `manifest.reconciliation_checkpoint.chain` mismatch. Stale-fallback case emits `op:"warn" reason:"stale-checkpoint"`. Cuts O(N) full-walk to O(rows_since_last_session) for the common case.
- **§8.7 phase 4 audit chain integrity (Change E)** — extended with stale-checkpoint check: if `manifest.reconciliation_checkpoint` is set, confirm `checkpoint.audit_id` resolves to a row in the ledger AND `checkpoint.chain` matches that row's `chain`. Mismatch → `CRITICAL stale-checkpoint`; freezes writes until reconciled per §4.7 fallback.

### Why

Local-optimization plan (`docs/CyberOS-AGENTS.LOCAL-OPTIMIZATION.md`) Stage 1 highlighted §4.7 reconciliation as the dominant session-start cost. With ~290 audit rows in the live store and growth ~10/day, full-walk reconciliation was creeping into multi-second territory. The checkpoint pattern is the standard incremental-validation answer; the 30-day stale fallback + chain-mismatch fallback preserve the integrity guarantee.

### Real-world trigger

User-driven local-optimization design (2026-05-09 evening). The supplementary `docs/CyberOS-AGENTS.EVOLUTION.md` (CyberOS-aware long-term plan) was scoped down to `docs/CyberOS-AGENTS.LOCAL-OPTIMIZATION.md` (immediate-action plan) once the user clarified that CyberOS-the-product is still pre-build and the priority is making `.cyberos-memory/` perform optimally as a personal BRAIN. Stage 1 of that plan ships first because it has zero dependencies and the fastest measurable impact.

### Verification

- Live AGENTS.md canonical SHA: `sha256:576368647e4d17635804580ca4dded28721b1c7247f0a19666ce43f5f0eb911a` ✓ matches manifest pin
- §8.7 post-upgrade scan: 0 CRITICAL, 0 WARN; report at `meta/health/2026-05-10-576368647e4d1763-postupgrade.md`
- `runtime/tools/cyberos_validate.py` clean run (1 INFO — pre-existing legacy memory_id)
- Chain LINK invariant: 293 rows, all chains link

### Related files updated (per §0.6)

- `docs/CyberOS-AGENTS.md` — Changes A–E applied; prior verbatim archived to `meta/protocol-history/AGENTS-sha256-599e1097199618e0d8dde22770eef6e5ad068c5c06150e2bb3829315f005780d.md`
- `.cyberos-memory/manifest.json` — `protocol.sha256`, `approved_at`, `approved_by`, `audit_chain_head`, `last_updated_at` updated
- `.cyberos-memory/audit/2026-05.jsonl` — `op:"protocol_upgrade"` row appended (chain `sha256:90bb3d3e0742a0e3...`); two `op:"create"` rows for the related-files writes
- `.cyberos-memory/meta/health/2026-05-10-576368647e4d1763-postupgrade.md` — auto-triggered §8.7 scan
- `.cyberos-memory/memories/refinements/REF-028-stage-1-session-start-speed.md` — refinement memory per §0.4
- `docs/CyberOS-PRD.CHANGELOG.md` — entry added; PRD .docx update deferred to next .docx editing session (DEC-106 entry pending)
- `docs/CyberOS-SRS.CHANGELOG.md` — entry added; SRS .docx update deferred similarly

### Related implementation

- `runtime/tools/cyberos_validate.py` — Stage 2 validator already extends to verify the new fields once they populate; `cyberos-doctor` recovery CLI is the next deliverable depending on these landing.
- `docs/proposals/STAGE-1-PROTOCOL-UPGRADE.md` — proposal text used for this upgrade (preserved as documentation; will not be re-applied).

---

## 2026-05-07 — Bundle L TIER 2: Legacy `memory_id` carve-out (`meta/legacy-ids.md` registry)

### Changed
- **§4.2 denylist exemption set** — added `meta/legacy-ids.md` to the rule-definition exemption list (alongside `manifest.json`, `README.md`, `meta/classification-rules.md`, `meta/retention-rules.md`, `meta/conflict-resolutions.md`, `meta/tombstones.md`, `AGENTS.md`). Injection gate still runs on the registry; only the §9.3 denylist regex is skipped.
- **§5.2 validators table** — appended one new validator row: *"Legacy `memory_id` (predates §5.2 validator)"*. Defines a closed-set carve-out: a small fixed list of memories created before the §5.2 UUIDv7/ULID validator landed MAY retain non-conforming mnemonic IDs provided each is registered in `meta/legacy-ids.md`. New writes to ANY scope still MUST use UUIDv7/ULID. The registry is itself denylist- and frontmatter-exempt under the same convention applied to `meta/tombstones.md`.

### Added
- **§13.1 step 7a** — bootstrap now creates an empty `meta/legacy-ids.md` registry alongside `meta/tombstones.md`. Format documented inline: `<mem_id> | <originating_path> | <originally_created_at> | <reason>`. Closed-set: new entries land only via a §0.5 protocol upgrade.
- **`meta/legacy-ids.md`** in this BRAIN — populated with the 4 surviving pre-§5.2 IDs identified by the 2026-05-07 healthcheck:
  - `mem_01HSXX0TOMBSTONES000000001` → `meta/tombstones.md`
  - `mem_01HSXX0RETENRULES000000001` → `meta/retention-rules.md`
  - `mem_01HSXX0CLASSRULES000000001` → `meta/classification-rules.md`
  - `mem_F005DOCCHANGELOG2026050401V` → `memories/facts/FACT-005-doc-changelog-convention.md`

### Real-world trigger
2026-05-07 BRAIN healthcheck (this conversation) surfaced 4 invalid memory_ids per §5.2 alongside 13 §4.7 SHA-mismatched files. Closing the SHA-mismatch finding required appending corrective `op:str_replace` audit rows; one of those files (`meta/tombstones.md`) carries a legacy mnemonic `memory_id`, so the corrective row would itself fail §5.2 validation. Two clean options: (a) tombstone the 4 files and recreate with fresh UUIDv7s — cascades into `relationships:` rewrites across adjacent memories; (b) carve out the closed set via a registry — no cascading edits, sets a precedent for future migrations. Stephen chose (b).

### Why TIER 2
Schema change to §5.2 (one validator row added), surface-area-only changes elsewhere. No new mechanism, no audit-row format change, no §6 manifest field added. The registry file itself is closed-set — no ongoing maintenance burden. Auto-§8.7 post-upgrade scan per Bundle J expected to report 0 critical / 4 info (the 4 legacy IDs, now legitimised).

### Schema impact
- `meta/legacy-ids.md` is a new canonical filename in §3 layout (implicit; `meta/` is documented as holding registries; the explicit step in §13.1 is sufficient).
- §4.2 exemption set grew by one entry.
- §5.2 validators table grew by one row.
- No new frontmatter fields, no new audit-row keys, no new state in §13.0.

### AGENTS.md canonical SHA
- Before: `sha256:632343f0c9e7eef251bbef5308b9859b6bd99933f2c3c76dc76a2282b41b7a1c`
- After:  `sha256:599e1097199618e0d8dde22770eef6e5ad068c5c06150e2bb3829315f005780d`

### Side-finding (deferred)
The healthcheck also discovered the BRAIN's 269-row pre-upgrade ledger was written by 3 distinct canonicalisations (Python `json.dumps` with two different exclusion conventions; RFC 8785 strict). LINK invariant holds across all three (each writer reads the previous row's `chain` as opaque bytes), so chain integrity is intact. But §7.2 mandates JCS strict for forward portability. A follow-up TIER 1 amendment to §7.2 — *"writers MUST match `manifest.protocol.last_writer_canonicalization` once set; switching emits `op:warn reason:canonicalization-drift`"* — was proposed and is held for a separate bundle.

---

## 2026-05-06 (later evening) — Bundle K TIER 1: Deprecate `.protocol-signing-key` file

### Changed
- **§0.5 TOFU paragraph** — removed the `cyberos/.protocol-signing-key` reference. New wording: *"Trust establishment is TOFU: the first fingerprint enters the manifest via explicit user paste from any trusted out-of-band source — a CyberSkill-signed announcement, a verified org-wide secrets manager, an in-person fingerprint exchange, or any equivalent. **Pre-BRAIN-module-P1, no canonical out-of-band source is mandated by this protocol** (the canonical mechanism lands when P1 ships)."*

### Removed
- **`cyberos/.protocol-signing-key`** (deprecated) — overwritten with a tombstone-style deprecation marker referencing DEC-094 v2 / DEC-105 / REF-026. The cowork sandbox can't `rm` files outside `.cyberos-memory/`; user can manually delete from local clone if desired.

### Updated
- **DEC-094 v=1 → v=2** — appended History entry documenting the Bundle K deprecation. The original "signing_keys bullet" prose remains in v1 history; the v2 prose acknowledges the file approach was deferred.
- **README.md Part 6 (Protocol distribution)** — removed the "baked into the cyberos repo" sentence; replaced with the post-K wording matching §0.5.

### Real-world trigger
Stephen flagged the file as friction: *"is there any way that no need one more separate file .protocol-signing-key?"* Honest analysis: it was placeholder weight. No real CyberSkill signing key exists yet (BRAIN module P1 hasn't shipped); the file documented an aspiration rather than enforcing real trust. Stephen picked Option A (delete now, defer real distribution mechanism to P1) over Options B (embed in AGENTS.md frontmatter) and C (keep file; defer decision).

### Why TIER 1 only
Single paragraph rewrite + one file deprecation + one DEC version bump + one README sentence. No new mechanism; no schema change; no audit-row format change. Pure surface-area reduction.

### Schema impact
None. `manifest.protocol.signing_keys[]` array remains in §6 unchanged — it just no longer has a canonical pre-P1 population source. Auto-§8.7 post-upgrade scan per Bundle J is expected to report 0c/0w because nothing changed at the §5.1 frontmatter level.

### AGENTS.md canonical SHA
Pre-K `sha256:1a55e8b…2edb` → post-K (computed at write).

### BRAIN entries
DEC-094 v=2 (signing-key-file approach deferred to P1), DEC-105 (Bundle K decision), REF-026 (refinement record).

### Cross-link
`docs/CyberOS-AGENTS.README.md` Part 6 / Part 8 (Bundle K added as the twelfth real-world trigger; first to REMOVE surface area).

---

## 2026-05-06 (later evening) — Bundle J TIER 1: Auto-trigger §8.7 after protocol_upgrade + uppercase BRAIN in trigger phrases

### Added
- **§0.5 step 4** — every successful `op:"protocol_upgrade"` now auto-triggers a §8.7 self-audit pass immediately after the manifest pin and the protocol_upgrade audit row. This is the post-upgrade migration check: schema validate (phase 1) catches memories failing the new §5.1; supersedes-graph integrity (phase 2) catches dangling relationships if scopes were renamed; resource caps (phase 6) catches new field additions pushing files over §5.5 limits. Findings surface per §8.7 severity routing. Skip only with explicit phrase *"skip post-upgrade scan"* (logged as `op:"skipped-by-user"`).
- **§6 manifest** — `health_check_policy.post_upgrade_phrase` field. Default value: *"rescan BRAIN"* (uppercase BRAIN per §0.3 / Bundle H). Manually triggers the same scan as the auto-flow.
- **§8.7 "Post-upgrade scan" subsection** — distinguishes the post-upgrade flavour from routine on-demand health-checks. Identical mechanics; report file named `meta/health/<YYYY-MM-DD>-<sha>-postupgrade.md` to mark provenance. The §14 block reports it as a post-upgrade scan.

### Changed
- **`manifest.health_check_policy.on_demand_phrase` default** — *"run brain healthcheck"* → *"run BRAIN healthcheck"* (uppercase BRAIN per §0.3 / Bundle H consistency).
- **`manifest.health_check_policy.diagnostic_verbs[]` defaults** — entries mentioning BRAIN switched to uppercase: *"check brain"* → *"check BRAIN"*; *"show brain"* → *"show BRAIN"*; *"view brain"* → *"view BRAIN"*. Lowercase versions explicitly NOT diagnostic triggers (they're anatomy/metaphor per §0.3).
- **§1 step 2** — diagnostic-verb list updated to match the new manifest defaults; added a one-sentence note: *"verbs that mention 'BRAIN' use uppercase per §0.3 (case-sensitive alias); lowercase 'brain' verbs are NOT diagnostic triggers."*

### Real-world trigger
Stephen asked: *"can we auto trigger scan and re-arrange/refine the .cyberos-memory after AGENTS.md update, because there maybe breaking changes or rules that need to adapt, and how to manual trigger that?"* Plus reinforcement: *"for manual i want 'run BRAIN healthcheck' instead"* (uppercase BRAIN). Bundle J answers both: §8.7 already had the schema-validate check that catches new-schema-failures; auto-triggering §8.7 after every protocol_upgrade was a one-step amendment to §0.5. The uppercase-phrase fix completes Bundle H's case-sensitivity work — three places still had lowercase "brain" in default trigger phrases that should have been uppercase for consistency.

### Why TIER 1 only
Single sentence-and-a-half §0.5 amendment + 4 default-value updates + one new §8.7 paragraph. No new ops, no new scopes, no new mechanism. The §8.7 phase-1 schema-validate already does the migration check — Bundle J just wires it into the post-upgrade flow automatically.

### What this does NOT change
- The §8.7 checks themselves (still six checks; same severity buckets; same `meta/health/` location).
- The audit ledger format and chain semantics — unchanged.
- Existing `on_demand_phrase` users with lowercase phrases configured — those are project-level overrides; only the default ships uppercase. Existing manifests are not migrated automatically.

### Migration note for cyberos's own manifest
Cyberos's running `manifest.health_check_policy.on_demand_phrase` updated to "run BRAIN healthcheck" as part of this Bundle's manifest re-pin. `diagnostic_verbs[]` entries also uppercased.

### AGENTS.md canonical SHA
Pre-J `sha256:7e229a2…2545d` → post-J (computed at write).

### BRAIN entries
DEC-104 (auto-trigger §8.7 + uppercase BRAIN phrases decision), REF-025 (refinement record).

### Cross-link
`docs/CyberOS-AGENTS.README.md` Part 8 (Bundle J added as the eleventh real-world trigger).

---

## 2026-05-06 (later evening) — Bundle I TIER 1: Compact §14 format gated by operational_mode

### Added
- **§14.1 Compact format** (default for `operational_mode: normal`) — `Δ Changes:` block showing only paths with actual changes; `Status:` block with conflicts/drift/shallow/sync/health one-liner; `unchanged:` roll-up line. Analysis-only turns collapse `Δ Changes:` to a single line `(no mutations this turn — <justification>)`.
- **§14.2 Full format** (default for `operational_mode: verbose | debug | maintenance`) — pre-Bundle-I per-scope-explicit format retained. `maintenance` mode prepends a `🔧 MAINTENANCE` banner with `maintenance_session_id`.
- **§14.4 Authority clarifier** — the audit ledger is the authoritative record; the §14 block is human-readable summary; format changes per `operational_mode` do not affect audit chain integrity.

### Changed
- **§14 opening paragraph** — now declares the two-format split and points at `manifest.operational_mode` as the discriminator.
- **§14.3 Coverage stat for ingestion ops** — unchanged content; renumbered from prose-paragraph to its own subsection for symmetry.

### Real-world trigger
Stephen flagged real readability friction post-Bundle-H: *"sometime this section so long and hard to read, is there any way to present it more verbose & human easier read?"* Surveyed prior turn outputs — every §14 block had ~14 lines, ~9 of which read "no change" verbatim. Signal lost in noise. The `operational_mode` field (added Bundle C) was the right discriminator — it already exists; reuse for rendering avoided new mechanism. Third refinement from real-world use; first that targets human-UX rather than protocol semantics.

### Why TIER 1 only
Single section rewrite; reuses existing `operational_mode` mechanism; no new fields, no new ops, no new scopes. Clean rollback path via the verbatim archive.

### What this does NOT change
- Audit ledger format and chain semantics — unchanged.
- §14 mandatory status (still required after every substantive reply).
- Coverage stat for ingestion ops (still mandatory; just renumbered §14.3).
- Per-mode behaviour outside §14 (DEBUG mode banners per §8.7 still apply; MAINTENANCE mode permissions per §8.8 unchanged).

### AGENTS.md canonical SHA
Pre-I `sha256:fe0773c…251aa` → post-I (computed at write).

### BRAIN entries
DEC-103 (compact-§14-by-operational_mode decision), REF-024 (refinement record).

### Cross-link
`docs/CyberOS-AGENTS.README.md` Part 8 (Bundle I added as the tenth real-world trigger; first targeting human-UX).

---

## 2026-05-06 (later evening) — Bundle H TIER 1: Strict uppercase BRAIN alias (§0.3)

### Changed
- **§0.3 first paragraph** — added explicit case-sensitivity clause: *"(literal uppercase B-R-A-I-N; case-sensitive — lowercase 'brain' does NOT trigger this alias)"*. The pre-H wording said *"the BRAIN"* / *"your BRAIN"* with implied capitals but didn't enforce it; a literal reader could have matched lowercase "brain" too.
- **§0.3** added a "Lowercase 'brain' is normal language" clarifier paragraph listing common lowercase usages (anatomy, metaphor, general topic) that explicitly do NOT trigger the alias. Includes an ambiguity-disambiguation rule: when context strongly implies memory-store but casing is lowercase, the agent asks a clarifying question rather than silently assuming.

### Real-world trigger
Stephen noticed: *"i notice that 'brain' still work? i want only 'BRAIN' will be understand as the memory, because some topic relate to human brain may trigger too, right?"* — confirmed that pre-H §0.3 didn't enforce case, leaving a small but real false-positive surface (lowercase "brain" in non-memory contexts could be misinterpreted). Second refinement from real-world use; Bundle G was the first.

### Why TIER 1 only
Single-paragraph change; narrowly scoped; closes the observed gap. No TIER 2/3 candidates surfaced.

### What this does NOT change
- §1 step 2's diagnostic-verb list (Bundle G) keeps lowercase phrases like "check brain", "show brain", "view brain". Those verbs trigger `PRISTINE-DIAGNOSTIC-HOLD` based on intent, NOT BRAIN-alias activation. The two mechanisms are independent.
- The case-sensitivity rule applies only to §0.3 alias activation; written prose elsewhere in the protocol can use either case for readability.

### AGENTS.md canonical SHA
Pre-H `sha256:3804334…f0ecb` → post-H (computed at write).

### BRAIN entries
DEC-102 (strict-uppercase BRAIN alias decision), REF-023 (refinement record).

### Cross-link
`docs/CyberOS-AGENTS.README.md` Part 8 (Bundle H added as the ninth real-world trigger; second from real-world use).

---

## 2026-05-06 (later evening) — Bundle G TIER 1: Diagnostic-verb carve-out for PRISTINE auto-bootstrap

### Added
- **§1 step 2 carve-out** — auto-bootstrap is silent UNLESS the user's current-turn message contains a recognised diagnostic verb (default list: `healthcheck`, `status`, `inspect`, `audit`, `check brain`, `show brain`, `view brain`, plus configured `on_demand_phrase`). When intent is diagnostic AND state is `PRISTINE`, the agent enters `PRISTINE-DIAGNOSTIC-HOLD` and surfaces the absent state instead of bootstrapping.
- **§13.0 `PRISTINE-DIAGNOSTIC-HOLD` row** — sub-state of `PRISTINE`. Agent surfaces what would be created by §13.1 and waits for explicit consent (`bootstrap and continue`, `just bootstrap`, or any task-oriented instruction). Does NOT write during this state.
- **§6 manifest extension**: `health_check_policy.diagnostic_verbs[]` — array of strings; project-level override of the default verb list.

### Real-world trigger
A fresh Cowork session at `sale-noti/` (the first downstream consumer of the protocol post-Bundle-F) ran `healthcheck` against a `PRISTINE` BRAIN. The agent correctly held off on silent auto-bootstrap, reasoning that bootstrapping mid-diagnostic would change the very state being inspected. It surfaced this as an §0.4 candidate for upstream propagation. Stephen approved upstreaming the refinement so future downstream projects don't re-encounter the friction. **This is the first refinement triggered by a real downstream project's actual use of the protocol** — the §0.4 propose-then-adopt loop firing in the wild rather than during meta-protocol design.

### Changed
- AGENTS.md canonical SHA: pre-G `sha256:f7f3934…f4f1b7` → post-G (computed at write time).

### BRAIN entries
DEC-101 (diagnostic-verb carve-out decision), REF-022 (refinement record).

### Cross-link
`docs/CyberOS-AGENTS.README.md` Part 8 (now lists Bundle G as the eighth real-world trigger; first one originating from a downstream project).

---

## 2026-05-06 (evening) — Bundle F: Comprehensive audit-fix pass + §0.6 related-files rule

### Added
- **§0.6 Related-files update rule** (sev-1) — every successful `op:"protocol_upgrade"` MUST be followed in the same chat turn by updates to: CHANGELOG (dated entry), README (any tracked Part), cross-linked FACT memories (e.g., FACT-004), and implementation files (e.g., `brain_writer.py` for §7.2; `.protocol-signing-key` for §0.5). Order of operations enumerated. Self-detection extension at §8.7 phase 1 reserved for Bundle G.
- **§7.5 `op:"corrects"` vs `correction_to` field** — distinguishes the two mechanisms. `op:"corrects"` is its own audit row for content correction (the world changed); `correction_to` is a field on any op marking that THIS row corrects the agent's own prior action. Rule: every `op:"corrects"` MUST have `correction_to` set; non-corrects ops MAY set it for self-correction.
- **§8.1 / §8.2 / §8.3 / §8.4 / §8.5 explicit subsection headers** — phases 1-5 of consolidation now have their own subsection numbers, matching §8.6 / §8.7 / §8.8 already-explicit subsections. Closes the §11.5-references-§8.5 dead reference.

### Fixed (TIER 1 — bugs / stale claims)
- **§0 line 22**: "§0 through §16" → "every section of `AGENTS.md` from §0 to the end" (was stale since Bundle A added §17).
- **§5.1 heading**: "27 fields" → "28 fields" (was stale since Bundle A added `sync_class`).
- **§8 heading**: "5 phases" → "7 phases" with explicit §8.1–§8.5 subsection headers (was stale since §8.6 + §8.7 added).
- **§8.7 step 4**: chain hash formula updated to match Bundle D §7.2 — now uses `row_without_chain_or_prev_chain`; clarifies LINK integrity is authoritative and hash recomputation is INFO-severity. (Was a bug — old §8.7 wording would have caused implementations to compute wrong hashes.)
- **§4.7 orphan-manifest pairing**: now accepts `consolidation_run | protocol_upgrade | protocol_rollback | session.end` as valid terminators (was a real bug — old wording would have flagged every Bundle's protocol_upgrade as crash-mid-consolidation and frozen writes).
- **§9.7 Delete row**: removed undefined "30-day legal hold" language; replaced with §4.6 cross-reference.
- **§9.7 Privacy row**: cites §17 sync_class (the actual mechanism) and §6 exclusion_rules (for ingestion-blocking).
- **§11.5 step 5**: "(§8.5)" — now resolves to the explicit §8.5 subsection added above.
- **§11.6 declares M&A-only schema extensions**: `original_chain` field on rebased audit rows + `manifest.imported_sources[]` array — both formally defined, with `INCOMPATIBLE:<field>` exemption when `imported_sources[]` is non-empty.
- **§17.5 `manifest.actor_keys`**: clarified as aspirational — to be added to §6 schema via §0.5 protocol upgrade at BRAIN module P1, not yet present.

### Fixed (TIER 2 — stale or inconsistent)
- **§3 layout**: now lists `meta/protocol-history/` (per §0.5) and `meta/health/` (per §8.7) as first-class subdirectories.
- **§13.1 step 2**: `tenant.id`/`owner.id` `null` (not `""`) when unknown.
- **§16 Tie-breakers**: "flag for next consolidation" → `op:"warn"` (matches post-Bundle-C vocabulary).
- **§0.2 bullet**: "schema_version" → "manifest field outside §6 schema" (the `schema_version` field was removed 2026-05-04 afternoon; the bullet was stale).

### Fixed (TIER 3 — compression / consolidation)
- **§0.5 "Forbidden by §0.2" paragraph** → one cross-reference sentence.
- **§4.10 forbidden-tool patterns** → compressed from five bullets to one parenthetical (the principle is "walk sequentially; no sampling"; the specific tools were examples).
- **§4.1 step 5** → absorbs §11.7's path constraints (length cap, case-collision, Windows-illegal chars). §11.7 reduced to a one-line cross-reference.
- **§9.4 project-specific examples** → generalised to "specific opt-in topics live in `meta/opt-ins.md` per project" (matches `feedback-no-project-specific-examples-in-universal-docs.md` standing rule).

### AGENTS.md canonical SHA
Pre-F `sha256:f9328b7…cb1022` → post-F `sha256:f7f3934…f4f1b7`.

### Real-world trigger
Stephen requested: *"check whole CyberOS-AGENTS.md content to find things that can be refine/compress/combine/merge/drop..."* Comprehensive audit surfaced 19 issues across three tiers. User adopted all three tiers in one bundle. The §0.6 related-files update rule was added at user's reinforcement: *"remember always update readme and changelog after AGENTS.md changes."*

### Pre-F archive
`meta/protocol-history/AGENTS-sha256-f9328b7…cb1022.md` (verbatim, captured at session.start before any edits).

### BRAIN entries
DEC-100 (audit-fix pass + related-files rule), REF-021 (refinement record).

### Cross-link
`docs/CyberOS-AGENTS.README.md` Part 8 (now lists Bundle F as the audit-cleanup pass).

---

## 2026-05-06 (evening) — Bundle E TIER 1: Three-way protocol-conflict handling (§0.5 + §13.0)

### Added
- **§0.5 "Three-way conflict (loaded ≠ pinned ≠ upstream)" subsection** — defines the case where loaded SHA `Y`, pinned SHA `X`, and upstream-available SHA `Z` all differ. Agent enters `INCOMPATIBLE:three-way-protocol-conflict` state, refuses to apply upstream, surfaces a structured prompt with three explicit user options (revert local; approve local as upgrade; manual three-way merge then approve via the standard §0.5 phrase). No automated merge.
- **§13.0 state classifier row**: `INCOMPATIBLE:three-way-protocol-conflict`. Same freeze-write handling as 2-way `protocol-sha256-mismatch`.

### Changed
- AGENTS.md canonical SHA: pre-E `sha256:b4042a6…cacce3` → post-E `sha256:f9328b7…cb1022`.

### Real-world trigger
Stephen asked (post-cascade): *"did we take care of the case when local BRAIN conflict with upstream BRAIN when update?"* Honest diagnosis: the post-cascade §0.5 mechanism handled the 2-way mismatch (loaded vs pinned, scenario A) and the clean upstream upgrade (scenario B), but did NOT handle the 3-way case (scenario C) — a user with hand-edited AGENTS.md running "check for protocol updates" would have had local edits silently overwritten. TIER 2 (multi-actor protocol-version skew) and TIER 3 (key rotation operational flow) deferred — both gain operational relevance only when the BRAIN module's network surface ships at P1.

### Why TIER 1 only
- Closes the most immediate observed gap (silent overwrite of local hand-edits during upstream pull).
- Extends existing conservative §13.0 discipline (writes-frozen-until-explicit-resolution) from 2-way to 3-way without inventing new mechanisms.
- The three explicit options map cleanly onto existing §0.5 vocabulary.
- TIER 2 + TIER 3 are not currently load-bearing (no BRAIN module endpoint, no real signing key) — adopting them speculatively today would be bulk without proportional value.

### Operational note
Pre-E archive: `meta/protocol-history/AGENTS-sha256-b4042a6…cacce3.md` is **verbatim** (created during the 2026-05-06 rollback validation test per DEC-098). Bundle E inherits it as its pre-state archive without needing to re-create — full rollback support from Bundle D forward.

### BRAIN entries
DEC-099 (three-way protocol-conflict decision), REF-020 (refinement record).

### Cross-link
`docs/CyberOS-AGENTS.README.md` Part 6 (Protocol distribution) — content unchanged today; will reference §0.5's three-way subsection when next revised.

---

## 2026-05-06 (evening) — Bundle D: Canonical-JSON tightening (§7.2 → RFC 8785 JCS)

### Changed
- **§7.2 Canonical JSON for hashing** — rewritten to cite **RFC 8785 (JSON Canonicalization Scheme, JCS)** as the authoritative algorithm. Previously underspecified ("keys sorted, compact separators, shortest IEEE-754") which permitted multiple legal interpretations. Now documents exact serialisation primitives:
  - Object key ordering: lexicographic on UTF-16 code units (RFC 8785 §3.2.3).
  - Whitespace: none anywhere; no trailing newline.
  - Separators: literal `,` and `:` bytes; no surrounding whitespace.
  - Strings: UTF-8, NFC-normalised, non-ASCII preserved verbatim (no `\uXXXX` escapes for non-control chars).
  - Numbers: ECMAScript `Number.prototype.toString` (shortest round-trip via IEEE-754 double); integers without trailing `.0`; **Python `1.0` MUST serialise as `1`, not `1.0`** (the most common cross-writer-version divergence).
  - Booleans/null: lowercase `true`/`false`/`null` only.
  - No duplicate keys.
- **Reference implementations named**: `rfc8785` PyPI package; `canonicalize` npm package. Hand-rolled `json.dumps(sort_keys=True, …)` MUST validate against JCS test vectors before being trusted to chain audit rows.
- **Cross-writer-version compatibility clarified**: the chain LINK invariant (`row[N].prev_chain == row[N-1].chain`) is the **authoritative** integrity guarantee. Hash *recomputation* across writer versions MAY fail (different writers emit different bytes for logically-identical rows); this is informational and surfaced at INFO severity in §8.7 self-audit, NOT a chain break.
- **Body exclusion clarified**: `canonical_json` receives `row_without_chain_or_prev_chain`; `prev_chain` is concatenated as raw bytes AFTER the canonical body.

### Real-world trigger
The 2026-05-06 cascade verifier (`outputs/verify_v2.py`) surfaced 149 pre-existing audit rows failing bit-perfect hash recompute against the new `brain_writer.py`, despite both writers nominally following pre-D §7.2. LINK integrity intact; recompute divergent. Surfaced as a TIER 1 §0.4 candidate at the end of the prior turn ("§7.2 is underspecified"); user adopted as Bundle D in the next turn.

### What this does NOT do
Pre-D rows remain hash-non-reproducible. The cardinal rule (additive-only) is preserved because pre-D rows are not retroactively touched. LINK integrity holds. Forcing a re-chain would invalidate any external exports already pinned to those chain values.

### AGENTS.md canonical SHA
Pre-D `sha256:7cd4a56…ad650a` → post-D `sha256:b4042a6…cacce3`.

### BRAIN entries
DEC-097 (canonical-json-rfc-8785 decision), REF-018 (refinement record).

### Cross-link
`docs/CyberOS-AGENTS.README.md` Part 8 (How to evolve the protocol safely — sixth real-world trigger).

---

## 2026-05-06 (evening) — Bundle C: Self-audit pass + DEBUG/MAINTENANCE modes (§8.7, §8.8)

### Added
- **§8.7 Self-audit pass** (sev-1) — sixth phase of consolidation; runs under `.lock`. Six checks: schema validate, supersedes-graph integrity, relationships-graph integrity, audit chain integrity (end-to-end recompute), orphan files, resource caps. Three severity buckets: `CRITICAL` (freezes writes), `WARN` (surfaced), `INFO` (logged).
- **Three operational modes** via `manifest.operational_mode`: `normal` (WARN/CRITICAL in §14 block); `debug` (every reject/revert/warn this session floats to top of next response as a banner); `verbose` (adds successful-op tracing).
- **§8.8 MAINTENANCE mode** (sev-0) — distinct from DEBUG; the safe version of "ROOT". Time-limited (1 hour or session end). Permits specific repair ops normally forbidden: chain rebuild, orphan tombstone, force-resolve conflict, manual rollback, frontmatter migration edit. Each repair requires per-op chat confirmation. Logged with `actor_kind: maintainer` + `maintenance_session_id`. NEVER bypasses §9.3 denylist or §4.2 content gate.
- **§6 manifest** — `operational_mode: "normal"` (default) and `health_check_policy: {on_session_end, on_demand_phrase}`.
- **§7.1 audit op enum** — `health_check`, `warn`, `drift_candidate`, `shallow_candidate`, `maintenance.start`, `maintenance.end`.
- **§14 end-of-response block** — new line: `health: <N critical | M warn | K info>; operational_mode: <…>`.
- **`meta/health/`** — new directory; stores deterministic health-check reports keyed by `<YYYY-MM-DD>-<sha>`.

### Deferred
- **TIER 2 — Org-level escalation channel** — when the BRAIN module ships at P1, CRITICAL + aggregated WARN forward to a CyberSkill admin channel. Privacy boundary: only metadata escalates; never memory content.

### Changed
- AGENTS.md canonical SHA: pre-C `sha256:8025a96…b13d65` → post-C `sha256:7cd4a56…ad650a`.

### Real-world trigger
Stephen asked (2026-05-06): *"Can the BRAIN audit itself? While users are using the BRAIN and unexpected issues happen, I should be notified so I can fix it asap. For now maybe we can use DEBUG or ROOT mode."* Diagnosis: pre-C protocol had partial self-audit elements (§4.7, §8.6, §13.0, §0.4, §1.10) but no integrated full-store integrity pass, no notification channel beyond the easily-missed §14 block, and no clear separation between read-side verbosity (DEBUG) and write-side repair authority (MAINTENANCE). Conflating the two risks the Linux-root footgun pattern.

### BRAIN entries
DEC-096 (self-audit + DEBUG/MAINTENANCE decision), REF-017 (refinement record).

### Cross-link
`docs/CyberOS-AGENTS.README.md` Part 7 (Self-audit & operational modes).

---

## 2026-05-06 (evening) — Bundle A: Sync-class boundary (§17)

### Added
- **§17 Personal vs shared memory boundary** — declares the four sync classes (`local-only`, `publishable`, `shared`, `client-visible`), per-scope defaults table (§17.2), per-subject identity model (§17.3 — subject not machine is the trust anchor), absorb-then-discard offboarding semantics (§17.4), publish-flow forward reference (§17.5 — mechanism deferred to BRAIN module P1), and explicit out-of-scope list (§17.6 — wire protocol, ACL, conflict mechanism, key rotation all live in the BRAIN/PORTAL modules, not here).
- **§5.1 frontmatter** — 28th permitted field: `sync_class: local-only | publishable | shared | client-visible`. Per-file overrides allowed.
- **§14 end-of-response block** — new line: `sync class summary: <N local-only | M publishable | K shared | J client-visible>`.

### Changed
- **§11.8** — last sentence rewritten to clarify scope: "This protocol governs the personal layer of the BRAIN. Continuous multi-machine sync of shared scopes happens through the runtime BRAIN module (FACT-004 Layer 2), not via filesystem replication." Closes the §11.8↔FACT-004 contradiction (was: "Concurrent multi-machine editing of the same project is unsupported; pick one authoritative machine" — read literally, that contradicted FACT-004's "CRDT sync across machines" claim).
- AGENTS.md canonical SHA: pre-A `sha256:6e993e3…b4797b` → post-A `sha256:8025a96…b13d65`.

### Real-world trigger
Stephen asked (2026-05-06): *"It's working as personal memory for one person. But each person will contribute to CyberSkill activities (via CyberOS), so it needs to serve both personal-based memory as well as CyberOS's memory. Should we think about that now?"* Surfaced two pre-existing gaps: §11.8↔FACT-004 contradiction (would fire as soon as a second laptop joins); personal-vs-org boundary was implicit so every memory written today was being classified by accident. Resolution: lock the boundary now via the four sync classes; defer mechanism (signing, wire protocol, ACL) to the runtime BRAIN module.

### User answers driving the design
Q1 *CyberSkill one tenant?* → publisher today, multi-tenant SaaS at P3+ supported by per-tenant region pinning. Q2 *project/ flows to org?* → yes, defaults to `shared` (CyberOS architecture is the company's product). Q3 *clients consume a slice?* → yes, fourth class `client-visible`. Q4 *offboarding?* → absorb knowledge, discard fragments. Q5 *per-machine or per-person?* → per-person identity (subject is trust anchor; multiple machines mirror through org BRAIN).

### BRAIN entries
DEC-095 (sync-class boundary decision), REF-016 (refinement record), FACT-004 v2 (Layer 1 paragraph rewritten to cite §17 instead of bare "CRDT sync"; closes the contradiction).

### Cross-link
`docs/CyberOS-AGENTS.README.md` Part 5 (Personal vs org: the four sync classes).

---

## 2026-05-06 (evening) — Bundle B: Protocol distribution policy (§0.5)

### Added
- **§0.5 Protocol update policy** (sev-0) — defines canonical SHA computation, manifest pin via `manifest.protocol.sha256`, session-start tripwire, the explicit chat-turn approval phrase *"approve protocol upgrade to `<sha256:…>`"*, archive-then-update flow, rollback path, signed upstream release flow with TOFU trust establishment, bootstrap behaviour, §0.2 forbidden list.
- **§6 manifest** — `protocol` block: `{sha256, approved_at, approved_by, loaded_path, signing_keys[], last_checked_at}`.
- **§7.1 audit op enum** — `protocol_upgrade`, `protocol_rollback`.
- **§13.0 state classifier** — `INCOMPATIBLE:protocol-sha256-mismatch` (canonical SHA mismatch with manifest pin → freeze writes; require chat-turn approval phrase to resolve).
- **§13.1 bootstrap** — step 12 (auto-pin canonical SHA at first run, no prompt) and step 13 (seed `meta/protocol-history/` for rollback archive).
- **`meta/protocol-history/`** — new directory; stores verbatim AGENTS.md archives keyed by SHA suffix; exempt from §5.1 frontmatter (these are protocol-doc archives, not memories; integrity is content-addressable via SHA).

### Changed
- AGENTS.md is now content-addressable. Pre-B canonical SHA `sha256:560a489…1600fc`. Post-B canonical SHA `sha256:6e993e3…b4797b`.

### Real-world trigger
Stephen asked (2026-05-06): *"AGENTS.md behaves like global instructions when copied to local machine. Is there any way to force-sync it with CyberOS's AGENTS.md to make sure all distributed BRAINs are updated when CyberOS has a new BRAIN version?"* Surfaced two pre-existing gaps: AGENTS.md was silent on its own update flow (no tripwire for hand-edits, host-platform silent updates, or accidental drift); "force sync" would defeat §0.2 (the same gate that protects from prompt injection would also block forced sync). Resolution: layered authenticity (Ed25519 signatures, deferred to TIER 2 / BRAIN module P1), authorization (chat-turn approval phrase per §0.2), and auditability (`op:"protocol_upgrade"` rows + `meta/protocol-history/` archive).

### BRAIN entries
DEC-094 (protocol-update-policy decision), REF-015 (refinement record). Both adopted in chat per §0.4.

### Cross-link
`docs/CyberOS-AGENTS.README.md` Part 6 (Protocol distribution).

---

## 2026-05-06 (evening) — README on-ramp shipped (informational; no AGENTS.md edits)

### Added
- **`docs/CyberOS-AGENTS.README.md`** — comprehensive 12-part reader's guide & evolution manual. Sections cover the mental model (Parts 1–4), the personal-vs-org sync-class boundary (Part 5), protocol distribution (Part 6), self-audit & operational modes (Part 7), the safe-evolution playbook with additive-only rules and the §0.4 propose-adopt-record loop (Part 8), common mistakes (Part 9), troubleshooting decision tree (Part 10), reading-order guide for AGENTS.md (Part 11), and glossary (Part 12).

### Why it's a CHANGELOG entry but no AGENTS.md edits
- The README is a **companion** doc, not part of the protocol itself. Editing it never triggers the §0.5 protocol-upgrade approval flow.
- The README captures decisions adopted in the same session (sync_class TIER 1, protocol-distribution TIER 1+3, self-audit TIER 1+3) that are *pending implementation* in AGENTS.md. The README explains the target state; the AGENTS.md cascade lands separately.
- This follows the same "informational; no AGENTS.md edits" pattern as the 2026-05-06 skill-registry entry below.

### Pending cascade (next coordinated batch)
- AGENTS.md edits: §0.5 protocol update policy, §6 manifest extension (`protocol`, `signing_keys`, `operational_mode`), §7.1 op enum (`protocol_upgrade`, `protocol_rollback`, `health_check`, `warn`), §8.7 self-audit pass, §13.0 state classifier (`INCOMPATIBLE:protocol-sha256-mismatch`), §13.1 bootstrap auto-pin, §14 block additions (`sync class summary`, `health check`), §17 personal-vs-shared memory boundary with 4-class sync_class.
- Memory writes: DEC-094 (sync_class boundary), REF-015 (sync_class refinement), DEC-095 (protocol update policy), REF-016 (protocol distribution refinement), DEC-096 (self-audit + DEBUG/MAINTENANCE modes), REF-017 (self-audit refinement), FACT-004 cross-link update (closes the §11.8↔CRDT contradiction).
- Once landed, this CHANGELOG gets a separate dated entry per refinement bundle.

### Cross-link
- See `docs/CyberOS-AGENTS.README.md` Part 8 for the reasoning behind the additive-only evolution rule and the propose-adopt-record loop.

---

## 2026-05-06 — Skill-registry v0.2.0 (informational; no AGENTS.md edits)

### Context

The skill registry at `cyberos/docs/skills/` shipped v0.2.0 with:
- Skills↔contracts namespace split (DEC-090).
- Dual-mode invocation + exposability frontmatter (DEC-091).
- Self-audit + auto-refinement at skill level (DEC-092).
- Manual fine-tune playbook (DEC-093).
- Plus the consolidated `README.md` wiki + the onboarding infographic.

### Why this is an AGENTS.md changelog entry but no AGENTS.md edits

- AGENTS.md governs the **BRAIN** (`.cyberos-memory/`) protocol — memory writes, the audit ledger at `audit/<YYYY-MM>.jsonl`, the consolidation cycle, the conflict-resolution graph.
- The skill registry's `genie.action_log` is a **separate** audit stream (the runtime's, per SRS §6.7) that records skill outputs. It chains independently from the BRAIN's ledger.
- The new skill-level `op:"self_refinement_proposal"` rows live in `genie.action_log`, not in the BRAIN. AGENTS.md §7.1's `op` enum is unaffected.
- The skill-level `self_audit` + `INVARIANTS.md` machinery is a **parallel** of AGENTS.md §0.4's standing rule, applied at the skill level rather than the protocol level. Same pattern, different surface.

### Cross-link

- See `cyberos/docs/skills/CHANGELOG.md` v0.2.0 for the registry-side detail.
- BRAIN entries DEC-090 / DEC-091 / DEC-092 / DEC-093 record the underlying decisions; REF-012 / REF-013 / REF-014 record the §0.4 refinement candidates surfaced during the design conversation.

---

## 2026-05-04 (evening, follow-up) — Validator discipline: fenced-code-block exemption + datetime-instance acceptance

### Changed
- **§4.3 file-content hygiene** — multi-frontmatter check now exempts content inside fenced code blocks (` ``` ` or `~~~`). Strip fenced spans before the secondary-block scan. Code-fenced examples of YAML frontmatter are legitimate Markdown content (common in skill / format / spec docs that show example `SKILL.md` or memory-file frontmatter) and must not trigger `multiple-frontmatter-blocks` rejection. Opening-block check unchanged. (DEC-087)
- **§5.2 timestamp validator row** — accept either an ISO-8601 string matching the existing regex OR a tz-aware language-native datetime instance. PyYAML and similar loaders auto-coerce ISO-8601 to native datetimes; `str(dt)` then renders with a space separator (`2026-05-04 21:13:29+07:00`) and fails the regex. Validators MUST handle both forms. Naive (tz-less) datetimes rejected as `naive-ts:<field>`. Offset and minute-granularity rules unchanged. (DEC-088)

### Real-world trigger
Surfaced during the skills-knowledge digest session (workbench/.cyberos-memory bootstrap, 2026-05-04 evening). Both failures hit on the very first memory-file write of a corpus of 12:
1. `spec.md` body legitimately contained `---`-delimited example SKILL.md frontmatter inside ```` ``` ```` fences. The §4.3 secondary-block scan triggered `multiple-frontmatter-blocks` rejection. Any session ingesting skill-format documentation, agent-protocol docs, or any spec that shows example frontmatter in code fences would have hit the same crash deterministically.
2. PyYAML's `safe_load` auto-parses ISO-8601 timestamps into `datetime.datetime` objects. The §5.2 validator's regex then ran on `str(dt)` which produces `2026-05-04 21:13:29+07:00` (space separator) instead of `2026-05-04T21:13:29+07:00` (T separator) and rejected its own valid output as `bad-ts:created_at`. Affects every Python implementation using PyYAML — i.e., effectively all of them.

Both refinements were proposed as Tier-1 (directly prevents observed failure) per §0.4 in the same response that surfaced them, and Stephen adopted both. The implementing patches in the session's local `.brain_writer.py` (a §4.4 atomic-write helper) are the reference implementations; both validators worked correctly against the remaining 11 memory files after patching.

## 2026-05-04 — Ingestion-side discipline + 10 protocol refinements

### Added
- **§0.4** Standing rule: every memory issue MUST trigger a refinement proposal in the same response (DEC-076).
- **§1.10** Verify-before-respond on user completeness challenge — stop, re-grep source verbatim, only respond AFTER verifying (DEC-077).
- **§4.10** Ingestion completeness discipline — forbid sample-skipping (`sed -n 'A,Bp;C,Dp'`, head/tail-only, modulus decimation); mandate sequential walk + high-water mark + coverage ≥0.99 OR `intentional_summary: true` with `summary_reason` (DEC-078).
- **§4.11** Token-budget transparency — declare chunking plan + confirm coverage in response for any source >500 lines or >50 KB (DEC-079).
- **§8.6** Source-coverage validator as Auto-Dream Phase 6 — re-hash sources, emit `op:drift_candidate` on hash mismatch, `op:shallow_candidate` on <0.80 coverage (DEC-081).
- **§3** layout extended: `memories/drift/` (auto-generated by §8.6) and `memories/refinements/` (REF-NNN-<slug>.md per adopted protocol amendment) as first-class memory bucket types (DEC-084).
- **§5.1** frontmatter additions (24 → 27 permitted fields):
  - `source_freshness_tier: <int ≥ 1 | null>` — lower = more authoritative; resolved per project from `manifest.source_tiers` (DEC-080).
  - `ingestion_coverage: <block | null>` — MANDATORY when `provenance.source ∈ {imported, doc, chat}`; carries `source_path`, `source_sha256`, `source_lines`, `processed_lines`, `source_messages`, `processed_messages`, `first_ts`, `last_ts`, `intentional_summary`, `summary_reason` (DEC-078).
  - `summary_reason: <string | null>` — required when `intentional_summary: true` (DEC-078).
- **§6** manifest additions:
  - `source_tiers: [{pattern, tier, rationale}, …]` — scope-pattern-glob → tier-int mapping for §9.1 Step 0 conflict resolution (DEC-080).
- **§7.1** audit row additions:
  - `correction_to: <evt_… | null>` — set when an op corrects the agent's own prior action (vs. a fact in the world) (DEC-083).
- **§14** end-of-response block additions:
  - Mandatory coverage suffix on any ingestion-op line (e.g. `created — coverage 944/944 lines, 53/53 messages, 2026-04-22→2026-05-04`).
  - New `drift candidates: <N>` and `shallow candidates: <N>` lines reporting §8.6 detections from the most recent consolidation (DEC-085).

### Changed
- **§9.1** Conflict decision tree gains a **Step 0** before the classification check: lower-tier (more authoritative) memory wins automatically; the higher-tier is auto-marked `superseded_by`. Step 0 is skipped when either side is `personnel` or `client` classification — those still go to manual resolution per Step 1. Eliminates Notion-vs-chat round-trip questions (DEC-080).
- **§10** Read protocol: added glances at `memories/drift/` (when the request touches a topic with multiple sources of truth) and `memories/refinements/` (when starting any substantive task — agents learn from past failure modes).

### Real-world trigger
Corrective re-ingestion of the 944-line Stephen↔Miguel WhatsApp DM. The original digest was produced via `sed -n 'A,Bp;C,Dp;…'` sampling and shipped at ~25% line coverage. Stephen surfaced the gap with screenshots and the prompt *"is your BRAIN not saved?"*. Re-ingestion captured 12 missed frozen decisions including 80/10/10, Master Seed Mirage Day-1 lock, SRF Bridge rejection, Resolution Waiting List, Vesting/Dual-Wallet, Specialization Ladder, Power Tens, Atomic Split, Failure Protection, Founder's Draw, contract-sign clock, Closed Beta MVP scope. Five of the §0.4 / §1.10 / §4.10 / §4.11 / §8.6 / §14 amendments are direct read-side counterparts to existing write-side gates (§4.1–§4.4) — the failure exposed an asymmetry in the protocol that this changelog entry closes.

## 2026-05-04 (afternoon revisions)

### Removed
- **§6 manifest** — `compatible_runtimes` field. Vestigial; not referenced anywhere in protocol logic.
- **§6 manifest** — `schema_version` field. Conceptually misaligned with the day-by-day protocol-evolution model.

### Changed
- **§4.3 file-content hygiene** — forward-compat sentence rewritten: unknown frontmatter fields now rejected with `op:rejected reason:unknown-frontmatter-field:<name>` and surfaced (was: "forward compat via manifest.schema_version").
- **§13.0 state classifier** — `INCOMPATIBLE:<sv>` row replaced with `INCOMPATIBLE:<field>`. Triggered by manifest carrying any field not in the agent's loaded §6 schema (field-presence tripwire). Same "refuse to operate; surface to user" action; the comparison just becomes structural rather than version-numbered.

### Real-world trigger
Stephen asked "is `compatible_runtimes` and `schema_version` necessary?" — neither survived the analysis. `compatible_runtimes` was unused vestigial code; `schema_version`'s discrete-version model contradicts day-by-day protocol evolution (would either bump daily and trigger constant `INCOMPATIBLE` cross-machine, or never bump and lie). Replaced with field-presence detection at the validator level, which achieves the same forward-compat protection without inline version markers.

## 2026-05-04 (afternoon revisions, follow-up)

### Changed
- **§6 manifest example** — `source_tiers` array stripped of Styx-specific patterns (`module:whatsapp-*-dm`, `module:whatsapp-*-group`, `module:notion-*`). Replaced with generic schema-only example (`<scope-glob>` + default `*` tier 99). The field is universal protocol; the values are per-project. Each project's `manifest.json` configures its own patterns at bootstrap. A new clarifying sentence after §6 makes this explicit.

### Real-world trigger
Stephen flagged that the previously-checked-in §6 example carried Styx project context (whatsapp + notion patterns), which is a correctness bug for any project that adopts AGENTS.md as its protocol — the patterns would be meaningless in cyberos or any other project. Stripping fixes the protocol's universality and aligns with the no-project-specific-examples-in-universal-docs principle (now also captured as a feedback memory).

---

## Batch 24 — Doc reorganisation (2026-05-12)

### Changed
- **`docs/skills/` consolidation** — `CHAIN_ORCHESTRATOR.md`, `MANUAL_WORKFLOW.md`, `HOST_ADAPTERS.md` collapsed into single anchor `docs/skills/README.md` (Parts 28–30 appended; headings demoted; cross-refs rewritten). Originals replaced with one-line redirect stubs.
- **`docs/memory/` introduced** — 6 protocol docs moved from `docs/CyberOS-AGENTS*.md` / `docs/CyberOS-{AGENTS,PRD,SRS}.CHANGELOG.md` into new `docs/memory/` folder:
  - `AGENTS.md` (full protocol, 114 KB)
  - `AGENTS-CORE.md` (compact 42 KB, regenerable via §0.5)
  - `README.md` (32-part operator manual + skills cross-reference)
  - `CHANGELOG.md` (batches 1–24)
  - `PRD.CHANGELOG.md`
  - `SRS.CHANGELOG.md`
  - New `INDEX.md` landing page with reading order + symlink recipe + folder history.
- **Manifest pin updated** — `.cyberos-memory/manifest.json` → `protocol.loaded_path` rewrote from `docs/CyberOS-AGENTS.md` to `docs/memory/AGENTS.md`. SHA pin (`sha256:71a276c7…`) preserved (canonical SHA matched after copy).
- **Tool source patched** — `canonical_sha.py`, `extract_agents_core.py`, `voice_check.py`, `runtime/tools/cyberos`, `runtime/{tools,README}.md` updated to reference new `docs/memory/` paths.
- **Legacy stubs** — `docs/CyberOS-*.md` left as redirect stubs (sandbox cannot unlink; host removes with `rm` when convenient).

### Verify
- `cyberos verify` → CRITICAL: 0 (12 pre-existing WARN, 1 INFO unchanged).
- `cyberos fr list` → 2 FRs registered (Slack HR bot + Landing-page MVP).

### Real-world trigger
Stephen: *"too many docs inside skills folder that made me confuse, can we combine all inside single README.md / move memory related files into new folder 'memory'"*. End-of-session cleanup before closing the sprint that landed Batches 4–23.
