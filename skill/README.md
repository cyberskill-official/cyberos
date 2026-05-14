# CyberOS Skill module

The skill module hosts CyberOS's portable, capability-sandboxed agentic skills. It implements (and extends) the **Anthropic Agent Skills open standard** (SKILL.md + progressive disclosure + filesystem discovery), the same format adopted by VS Code, GitHub Copilot, OpenAI Codex CLI, Cursor, Goose, Amp, Gemini CLI, and 20+ other clients.

## Quick start

```bash
# Rust host — the canonical execution path
cd skill
cargo build
cargo run -p cyberos-skill-cli -- list
cargo run -p cyberos-skill-cli -- run vn-mst-validate --executor script

# Legacy Python runners — preserved until the 30-day soak ends (Phase 7 retirement)
python -m runners.fr_with_tasks --help
```

## Layout

| Folder | Purpose |
|---|---|
| `crates/` | Rust workspace — host, manifest parser, resolver, CLI |
| `toolchain/` | Bun + esbuild authoring toolchain (TS skills → wasm32-wasi components) |
| `skills/` | First-party CyberSkill skill bundles (20 SKILL.md, including 6 `cyberskill-vn`) |
| `contracts/` | Artefact schemas the skills emit (PRD, SRS, FR, task, …) |
| `runners/` | Legacy Python runners — preserved for parity testing during the soak window |
| `tools/` | Skill registry + build helpers |
| `docs/` | Protocol spec, design docs, changelog |
| `tests/` | Parity + correctness tests |
| `tours/` | Skill-flow tours (`.tour` files) |

## Strategic posture

CyberSkill is a citizen of the open Agent Skills ecosystem. We do not invent a competing format. We publish high-quality Vietnamese-market skills (VAT/e-invoice, VNeID, local bank APIs, Vietnamese legal/compliance) to the open registry. See `docs/SPEC.md` for the full architectural audit and migration plan.

## Status

| Phase | Status |
|---|---|
| Phase 0 — Inventory + freeze | shipped |
| Phase 1 — Rust + Bun scaffold | shipped |
| Phase 2 — Parity harness | shipped (12/12 pass) |
| Phase 3 — Executor selection | shipped |
| Phase 4 — DashMap concurrency | shipped (>=2x at contention) |
| Phase 5 — WASM execution | scaffolded; runtime gated on user install (see `docs/PHASE_5_ACTIVATION.md`) |
| Phase 6 — Capability broker | shipped |
| Phase 7 — Legacy retirement | runbook ready; execute after 30-day soak |
| VN catalog (6 skills) | shipped |
| OCI registry distribution | pending |
| Cosign signature verification | pending |
| `agentskills.io` submission | pending (waits for registry API) |

## Place in the CyberOS architecture

CyberOS has three modules today:

| Module | Role | Lives at |
|---|---|---|
| `memory/` | The BRAIN — append-only audit-chained personal memory store | `~/.cyberos-memory/` per project |
| `skill/` | Catalog of agentic Skills + Rust host + Bun toolchain | `skill/skills/` + Rust crates |
| `cuo/` | Router — natural-language → skill chain → memory record | Python package |

This module is **skill**. It interacts with:
- `memory/` — skill bundles declare `allowed_brain_scopes` (read/write) in SKILL.md frontmatter; the host's capability broker enforces them against the BRAIN.
- `cuo/` — the CUO router reads this module's catalog (via `cyberos-cuo catalog`), picks the right skill for a natural-language request, and shells out to `cyberos-skill run`.

For the full picture see `../website/docs/index.html` (interactive multi-layer architecture doc, 31 pages).
