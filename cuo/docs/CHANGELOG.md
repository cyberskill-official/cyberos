# Changelog — CUO module

All notable changes to the CyberOS CUO module, newest-first. Date-stamped, not version-stamped, to match the memory and skill modules' convention.

Entries follow [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) conventions.

---

## 2026-05-14 (state-of-the-module) — comprehensive shipped state

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
- Memory bridge -> Writer integration. Today decisions are flat files; Phase-2 will route through the canonical `cyberos.core.writer.Writer` so each routing decision lands on the BRAIN's audit chain.

---

## 2026-05-14 — Phase 1 shipped: rule-based router

> Initial scaffold of the agentic orchestrator. Routes natural-language requests to the six `cyberskill-vn` skills using a deterministic rule-based scorer; records every decision in the BRAIN audit chain.

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
