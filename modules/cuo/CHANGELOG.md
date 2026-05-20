# Changelog — CUO Module

All notable changes to the `cuo` module will be documented in this file.

---

## 2026-05-19 — [CUO] ship-feature-requests workflow v1.0.0 → v1.1.0 — adds feature-request-audit at two strategic points

Operator question: "double check if ship-feature-requests trigger feature-request-audit, if not then it should." Investigation confirmed: the workflow's 18-step skill chain ran spec-side audits like `implementation-plan-audit`, `edge-case-matrix-audit`, `coverage-gate-audit` — but **never validated that the FR's spec itself passed `audit_rubric@2.0`** (FM-101+, SEC-001..009, TRACE-001..005). This is the missing piece that allowed the FR-AUTH-005 "17 spec gaps in one audit" episode to slip through.

### What changed

Inserted `feature-request-audit` at TWO positions in `modules/cuo/chief-technology-officer/workflows/ship-feature-requests.md`:

- **New step 3 — Pre-flight FR spec audit** (after `repo-context-map-audit`, before `architecture-decision-record-author`): catches FM-101+ frontmatter gaps, SEC-001..009 missing-section gaps, TRACE-001..005 traceability gaps in the spec itself BEFORE any FR-consuming downstream work runs. Failure halts the chain; operator patches the spec first.
- **New step 18 — Post-implementation FR audit** (after `debugging-cycle-audit`, before `backlog-state-update-author`): enforces `TRACE-004` — every §1 clause's cited §5 test MUST appear as `passed` in the coverage_report before the FR can ship. Failure forces the outcome to be `[BLOCKED: …]` rather than `shipped + strict-audited`.

Chain length: **18 → 20 steps**. Renumbered steps 3-18 (existing) to 4-19. Output schema gains two new artefacts: `fr_audit_report` (pre-flight) + `fr_audit_postimpl_report` (post-impl) — both land in the memory audit chain and append to per-FR `.audit.md §10` dossiers.

### Verification

Programmatic check via Python YAML parse:
- ✅ Chain length 20, step numbers sequentially [1..20]
- ✅ Both `feature-request-audit` invocations present at steps 3 + 18
- ✅ All 20 referenced skill directories exist under `modules/skill/`
- ✅ `workflow_version: 1.1.0`

### Files touched

Modified:
- `modules/cuo/chief-technology-officer/workflows/ship-feature-requests.md` — frontmatter (`workflow_version` 1.0.0 → 1.1.0, +2 output artefacts), `skill_chain` (+2 steps + renumber), prose sections (renumbered §2-§9, new §2.5 pre-flight + §8.5 post-impl), new "Changelog" trailing block with v1.1.0 + v1.0.0 entries.

### Operator impact

Existing in-flight workflow runs are unaffected (each FR pass starts fresh; the v1.1 chain engages on the next supervisor invocation). All future ship-feature-requests runs now produce 2 additional artefacts in the memory audit chain per FR — `fr_audit_report` and `fr_audit_postimpl_report`. Storage cost: ~1-2 KB per artefact. Latency cost: ~2 additional LLM calls per FR (≈30-60 s on Sonnet).

---

## [CUO] 2026-05-17 evening — CUO v2.0.0 (persona-folder + workflow-file model)

### Changed (BREAKING — Python runtime wipe + paradigm shift)

- **Wiped legacy v0.1.0 Python rule-based router** (`cuo/cuo/`, `cuo/tests/`, `cuo/scripts/`, `cuo/tools/`, `cuo/pyproject.toml`). The CUO is now markdown-driven: persona folders + workflow files. A v3.0.0 Python supervisor is designed (see `SPEC.md`) but not yet implemented.
- **Paradigm shift: persona-aware orchestration.** v0.1.0 was a flat skill router (query → skill). v2.0.0 is two-stage: query → persona → workflow → skill chain. Persona match locks in context (whose work is this?); workflow match picks the chain; chain validates against the SKILL module's shipped catalog.
- **Patterns preserved conceptually** in `AGENTS.md`: `RoutingDecision` shape → `(persona, workflow, arguments)`; threshold-based confidence (0.5 on 0.0–1.0 scale); `ARG_EXTRACTORS` dispatch pattern for v3.0.0 supervisor; Vietnamese-diacritic-aware persona scoring.

### Added

- `cuo/MODULE.md` — canonical persona catalog covering every C-role in C-Suite Reference §5 (48 total: 47 personas + chief-of-staff).
- `cuo/README.md` — module quickstart for the new model.
- `cuo/_template/persona/` + `cuo/_template/workflow/` — canonical scaffolds for new personas + workflows; full `HOW_TO_USE.md`.
- `cuo/<persona-slug>/` — 48 persona folders, each with `README.md` rendering the 9-block schema (C-Suite Reference §4) + a `workflows/` subdirectory.
- `cuo/chief-technology-officer/` — canonical reference persona at full depth (CyberSkill IS technical; CTO is highest-traffic persona for an internal-eng-led consultancy).
- `cuo/chief-technology-officer/workflows/` — 5 fully-wired workflows referencing only SHIPPED SKILL module skills: `architect-new-system` (10-step chain through SRS+ADR+threat-model+SDD+impl-plan); `adr-quick-capture` (2-step ADR pair); `post-incident-review` (postmortem chain with blameless rewrite discipline); `deploy-readiness-review` (release-notes + deploy-checklist chain with DORA baseline capture); `threat-model-refresh` (quarterly + on-architectural-change refresh).
- `cuo/docs/AGENTS.md` v2.0.0 — full rewrite for the persona/workflow model. 16 normative sections.
- `cuo/docs/SPEC.md` v2.0.0 — quick-reference contract summary. Documents v3.0.0 Python supervisor data shapes.
- `cuo/docs/ROUTING.md` — full routing algorithm + design rationale for two-stage routing with worked examples.
- `cuo/docs/NEEDED_SKILLS.md` — punch list of 66 author+audit pairs (132 bundles) needed by planned workflows. Split Tier-1 (29 pairs — CyberSkill scale-up critical) / Tier-2 (29 pairs — growth/enterprise) / Tier-3 (8 pairs — niche). Subsequent build sessions march tier-by-tier.

### Removed

- `cuo/cuo/__init__.py`, `__main__.py`, `core/{catalog,router,invoker,memory_bridge,trace}.py`, `requirements.txt` — all Python runtime.
- `cuo/tests/` — 5 test files + 1 fixture file referencing legacy skill names.
- `cuo/scripts/install.sh`, `cuo/tools/run_fixtures.py` — install scripts + parity harness.
- `cuo/pyproject.toml` — Python packaging.

### Preserved (no changes)

- `cuo/docs/CHANGELOG.md` — this file. Lineage retained.

### Planned (subsequent sessions)

Per `NEEDED_SKILLS.md` §4:

- **Session A:** Build Tier-1 (29 pairs) — unblocks CFO / CEO / CHRO / CRO-Revenue / Chief-of-Staff / CCO-Customer / COO / CPO-Privacy / CAIO / Chief-Ethics-Officer workflows.
- **Session B:** Build Tier-2 (29 pairs).
- **Session C:** Build Tier-3 (8 pairs).
- **Sessions D-E:** Author the ~100-150 per-persona workflow files (each persona's `workflows/` folder currently stub).
- **Future:** v3.0.0 Python supervisor implementation.

---

---

## [CUO] 2026-05-14 (state-of-the-module) — comprehensive shipped state

> Docs-only consolidation pass. Snapshot of what the CUO module actually ships as of today.

### Shipped (Phase 1)

- `cuo/core/catalog.py` — SKILL.md frontmatter discovery under `../skill/skills/`.
- `cuo/core/router.py` — deterministic rule-based scoring with regex argument extractors; routes the 6 `cyberskill-vn` skills correctly.
- `cuo/core/invoker.py` — shells out to `cyberos-skill run --executor script`.
- `cuo/core/memory_bridge.py` — Phase-1 flat-file decision writer under `<memory-root>/meta/cuo-decisions/<ts_ns>.md`.
- `cuo/core/trace.py` — JSONL structured-event tracer.
- `cuo/__main__.py` — CLI with `catalog` / `route` subcommands.
- AGENTS.md routing protocol (RFC-style, BCP 14).
- 15/15 routing fixtures pass; 15/15 pytest tests pass.

### Pending — future work

- Phase 2 — LLM-driven router. Replace the keyword bank with catalog-driven model prompts so adding a skill requires no router edits.
- Phase 3 — Multi-skill chains. Walk `next_skill_recommendation` / `depends_on_contracts` to compose skill calls (e.g. validate MST -> generate VAT invoice) into a single user-facing request.
- Phase 4 — Persona switching. Route through CUO sub-personas (CPO, CTO, ...) per PRD §6.1 based on intent class.
- Memory bridge -> Writer integration. Today decisions are flat files; Phase-2 will route through the canonical `cyberos.core.writer.Writer` so each routing decision lands on the memory's audit chain.

---

---

## [CUO] 2026-05-14 — Phase 1 shipped: rule-based router

> Initial scaffold of the agentic orchestrator. Routes natural-language requests to the six `cyberskill-vn` skills using a deterministic rule-based scorer; records every decision in the memory audit chain.

### Added

* `cuo/core/catalog.py` — discovers SKILL.md frontmatter under `../skill/skills/` and returns a list of `SkillEntry` records (name, description, capabilities, region, collection, dir).
* `cuo/core/router.py` — rule-based scoring (verbatim-name match `+5.0`, per-keyword `+3.0`, VN-region bonus `+2.0`); per-skill regex argument extractors for MST, CCCD, bank transfer; confidence threshold `3.0` (saturation `10.0`).
* `cuo/core/invoker.py` — shells out to `cyberos-skill run --executor script`; auto-selects release / debug / `cargo run` invocation paths.
* `cuo/core/memory_bridge.py` — Phase-1 write of decisions under `<memory-root>/meta/cuo-decisions/<ts_ns>.md`; the chain-touching `Writer` integration is a Phase-2 follow-up.
* `cuo/core/trace.py` — JSONL structured-event tracer; stderr by default, file sink optional.
* `cuo/__main__.py` — CLI with `catalog` and `route` subcommands; `--invoke` and `--record` flags on `route`.
* `pyproject.toml` — registers the `cyberos-cuo` console script; sole runtime dep is `pyyaml>=6`.
* `docs/AGENTS.md` — normative routing protocol (RFC-style, BCP 14).
* `docs/SPEC.md` — contract summary.
* `docs/ROUTING.md` — heuristics rationale + Phase 2 LLM design.
* `docs/CHANGELOG.md` — this file.
* `tests/` — 11 pytest tests across catalog, router, invoker, memory_bridge; routing fixtures in `tests/fixtures/routing-cases.json`.
* `tools/run_fixtures.py` — parity harness for routing fixtures.
* `scripts/install.sh` — dev-install + smoke-test entrypoint.

### Notes

* Phase 1 does **not** call the memory module's `Writer`; decisions are written as flat memory files. Phase 2 will route through the chain.
* Subprocess invocation is mocked in tests — exercise it with `cyberos-cuo route '...' --invoke` against a built skill CLI.
* The keyword bank in `router.py` is the rule-based stand-in for an LLM. Adding a skill means adding 4–8 keywords there; Phase 2 retires this in favour of catalog-driven model prompts.

---
