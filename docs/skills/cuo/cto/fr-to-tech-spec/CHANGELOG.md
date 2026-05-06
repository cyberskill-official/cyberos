# CHANGELOG — `cuo/cto/fr-to-tech-spec`

> Format: [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/). SemVer at the skill level: MAJOR breaks the input/output envelope or the `tech_spec@1` body shape; MINOR adds backwards-compatible fields or new optional behaviour; PATCH is editorial / reference-doc clarification.

---

## v0.1.0 — 2026-05-06 (initial scaffold)

### Added

- `SKILL.md` — entry. Frontmatter at v0.2.0 contract level (33 fields per registry README Part 2). Owns the FR-to-tech-spec lifecycle: CONTRACT_ECHO → PLAN → WORKER → BATCH_COMPLETE.
- `CHANGELOG.md` — this file.
- `INVARIANTS.md` — scaffold listing the 6 invariants the future runtime MUST enforce.
- `STANDALONE_INTERVIEW.md` — scaffold for the chat-mode entry script.
- `HUMAN_SUMMARY.md` — scaffold for the chat-rendered batch-completion summary template.
- `envelopes/fr-to-tech-spec.input.json` — JSON Schema for the input envelope (3 required fields, 5 optional).
- `envelopes/fr-to-tech-spec.output.json` — JSON Schema for the output envelope (mirrors fr-author's batch-completion shape).
- `acceptance/README.md` — priority scenarios pending v0.3.0 harness.

### Driver

User-driven request: "scaffold the cuo/cto/fr-to-tech-spec skill structure so it can sit ready while the runtime is being built." Acts on the next-step Q2 plan from registry v0.2.2 audit conversation. The skill is documented at the contract level (every field a future runtime needs is named, every interface is shaped), but no executable code exists. The skill carries `gated_until_phase: runtime_v0_3_0` in its frontmatter so the validator + supervisor know not to route to it yet.

### Backwards compatibility

First version. No predecessor.

### Acceptance evidence

- All 33 v0.2.0 frontmatter fields present and validate against the registry's frontmatter schema (verified by hand; harness gate pending).
- `depends_on_contracts:` cites two contracts (`feature-request@v1`, `nats-subjects@v1`); both pin_paths resolve to existing CONTRACT.md files.
- Cross-skill consistency: persona-card (`cuo/cto/SKILL.md`) v0.1.0 lists this workflow in its owned-workflows table; registry README Part 23.1 index lists this skill at v0.1.0 (scaffold).

### What this version DOESN'T do (intentionally)

- No executable runtime — gated on the harness build (registry README Part 26).
- No reference docs (`HITL_PROTOCOL.md`, `UNTRUSTED_CONTENT.md`, `ANTI_FABRICATION.md`, etc.) — authored at v0.2.0 when the runtime needs them.
- No `tech_spec@1` contract — promotion to `cyberos/docs/contracts/tech-spec/` happens at v0.2.0.
- No PIPELINE.md worked example — pending the first chained run against a real FR.

## How to add a future entry

Standard sub-sections:

- **Added** — new fields, new sections, new BOOT codes, new references/*.md docs.
- **Changed** — semantics changes that don't break the schema.
- **Removed** — fields/rules deprecated.
- **Backwards compatibility** — what specs/envelopes from prior versions still work.
- **Acceptance evidence** — pointer to the test artifact or run that validated the release.
