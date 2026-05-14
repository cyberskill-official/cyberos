# CHANGELOG — `cuo/cto/` persona-card

> Format: [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/). SemVer at the persona-card level: MAJOR breaks the voice / scope-ceiling / escalation graph. MINOR adds a new owned workflow or extends scope additively. PATCH is editorial.

---

## v0.2.0 — 2026-05-06 (scope-ceiling expansion; pre-emptive for srs-author / srs-audit)

### Added — read scopes (mirrors cpo v0.3.0)

- `company:values` — required for srs-author strategic-context awareness.
- `memories:refinements`
- `member:*` (with `read_excluded: member:*/private/`) — capacity awareness in tech-spec sizing decisions.
- `client:*` — commissioned-project tech-spec context.

### Driver

cto's persona-card v0.1.0 inherited cpo's v0.2.0 ceiling. With cpo bumping to v0.3.0 (registry v0.2.4) to support the chain entry point, cto needs the same scope expansion pre-emptively for the srs-author and srs-audit workflows landing in registry v0.2.6 (Stage C). Doing this now (along with cpo) avoids two coordinated MAJOR bumps spread across releases.

### Backwards compatibility

- `write` scopes UNCHANGED.
- Existing workflow (`fr-to-tech-spec` v0.1.0) is a valid subset of the expanded ceiling.

---

## v0.1.0 — 2026-05-06 (initial release)

### Added

- `SKILL.md` — CTO persona-card, modeled directly on `cuo/cpo/SKILL.md` v0.2.0 with audience-appropriate voice deltas. Same scope ceilings (BRAIN read/write, MCP tools, escalation graph). Owns the technical-artefact lifecycle: tech specs, ADRs, runtime stewardship.
- First owned workflow: `fr-to-tech-spec/` — consumes audited FR markdowns from `cuo-cpo` and emits tech specs. Scaffolded at v0.1.0; full implementation gated on the runtime build (registry README Part 26).

### Driver

User-driven request to "scaffold the cuo/cto/fr-to-tech-spec skill structure so it can sit ready while the runtime is being built." Doable now (per registry README Part 26 + Q2 next-steps in this conversation): the skill folder + SKILL.md + dependency declarations + INVARIANTS.md scaffolding all exist as documentation, not code. They become executable when the runtime/harness ships in v0.3.0. Until then, the structure documents the intent and serves as the contract any future runtime must satisfy.

### Backwards compatibility

First version. No predecessor.

## How to add a future entry

Standard sub-sections:

- **Added** — new owned workflows, scope additions, voice deltas.
- **Changed** — rule semantics that don't change the persona's identity or scope ceiling.
- **Removed** — deprecated workflows.
- **Backwards compatibility** — what consumers of this persona-card still work, what migrates automatically.
