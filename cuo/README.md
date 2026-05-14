# CyberOS CUO module

The CUO (Chief Universal Officer) is the agentic orchestrator that consumes the CyberOS memory and skill modules. Given a natural-language request, it routes to the right skill(s), invokes them, and records the decision in the memory audit chain.

## Quick start

```bash
cd cuo
pip install -e .
cyberos-cuo route "Validate the MST 0312345678"
# → {"skill": "vn-mst-validate", "input": {...}, "output": {"ok": true, "kind": "entity"}}
```

## Layout

| Folder | Purpose |
|---|---|
| `cuo/` | Python package — catalog, router, invoker, memory bridge |
| `docs/` | Protocol spec + design docs + changelog |
| `scripts/` | Install + housekeeping |
| `tests/` | Routing test fixtures + unit tests |
| `tools/` | Test runners + diagnostics |

## Status

| Phase | Status |
|---|---|
| Phase 1 — Rule-based router | shipped (15/15 fixtures, 15/15 tests) |
| Phase 2 — LLM-driven router | pending |
| Phase 3 — Multi-skill chains (depends_on walk) | pending |
| Phase 4 — Persona switching (CPO/CTO/...) | pending |
| Memory bridge -> audit chain integration | pending (currently writes flat files under `meta/cuo-decisions/`) |

Phase 1 ships with deterministic rule-based scoring (keyword + skill-description matching) and routes the 6 `cyberskill-vn` skills correctly. Later phases retire the keyword bank in favour of catalog-driven model prompts (Phase 2) and chained envelope walks (Phase 3).

## Place in the CyberOS architecture

CyberOS has three modules today:

| Module | Role | Lives at |
|---|---|---|
| `memory/` | The BRAIN — append-only audit-chained personal memory store | `~/.cyberos-memory/` per project |
| `skill/` | Catalog of agentic Skills + Rust host + Bun toolchain | `skill/skills/` + Rust crates |
| `cuo/` | Router — natural-language -> skill chain -> memory record | Python package |

This module is **cuo**. It interacts with:
- `skill/` — discovers available skills via the skill module's catalog (`SKILL.md` frontmatter under `../skill/skills/`) and shells out to `cyberos-skill run` to invoke them.
- `memory/` — every routing decision becomes a memory record. Phase 1 writes flat files under `<memory-root>/meta/cuo-decisions/`; Phase 2+ will route through the canonical `Writer` so each decision lands on the BRAIN's audit chain.

For the full picture see `../website/docs/index.html` (interactive multi-layer architecture doc, 31 pages).
