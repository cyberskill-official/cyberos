# `cyberos/docs/contracts/` — Versioned schema artefacts

> A **contract** is a versioned schema that one or more skills produce or consume. Contracts are NOT skills — they don't act, they constrain. Skills declare contract dependencies via `depends_on_contracts:` in their frontmatter so the build pipeline can ship contract + skill as a single bundle.

## Why contracts live separately from skills

Skills under `cyberos/docs/skills/` *do work*: take input, run inference or deterministic logic, emit output, write an audit row. Contracts under this folder *constrain shape*: artefact frontmatter, envelope JSON schemas, wire protocols. Conflating the two leads to skill folders with empty `allowed_mcp_tools: []`, `expects: null`, `confidence_band: 1.0` — schemas wearing skill costumes. The v0.2.0 registry split (DEC-090) ends that conflation.

## Layout (locked: 2026-05-06)

```
cyberos/docs/contracts/
├── README.md                                   # this file
└── <contract-id>/                              # kebab-case
    └── v<n>/                                   # one folder per major version
        ├── CONTRACT.md                         # frontmatter + spec (replaces SKILL.md)
        ├── template.md                         # the schema body (artefact contracts only)
        ├── schema.json                         # the schema body (envelope contracts only)
        ├── CHANGELOG.md                        # version history
        └── examples/                           # optional reference instances
```

`CONTRACT.md` carries a smaller frontmatter than a SKILL.md. Skill-only fields (`allowed_mcp_tools`, `expects/produces`, `audit`, `confidence_band`, `untrusted_inputs`, `gated_until_phase`) are absent. Contract-only fields are present (`contract_id`, `contract_version`, `contract_kind`, `template_literal`, `steward_persona`, `escalation_on_breach`).

## Three contract kinds

| `contract_kind` | Schema body lives in | Used for |
| --- | --- | --- |
| `artefact_schema` | `template.md` (Markdown skeleton) | Markdown artefacts written by skills (e.g. Feature Requests, tech specs, postmortems). |
| `envelope_schema` | `schema.json` (JSON Schema) | Skill input/output envelopes referenced via `expects.schema_ref` / `produces.schema_ref`. |
| `wire_protocol` | `schema.json` + `protocol.md` | MCP tool descriptors, audit row formats, plug-in manifests. |

## How a skill consumes a contract

```yaml
# In the skill's SKILL.md frontmatter:
depends_on_contracts:
  - id:        feature-request          # contract folder name
    version:   v1                        # locks to this major version
    purpose:   generation_skeleton       # human-readable: why this skill needs it
    pin_path:  cyberos/docs/contracts/feature-request/v1/
```

The registry validator confirms:

1. The path resolves to a real `CONTRACT.md`.
2. The skill body's references to that contract use the declared path (no hard-coded paths to alternate versions).
3. On contract MAJOR bumps, every declared consumer is updated (or explicitly opts in to staying on the older version with a CHANGELOG entry).

## Index of contracts

| Contract | Latest version | Kind | Stewarded by | Consumed by |
| --- | --- | --- | --- | --- |
| [`feature-request`](./feature-request/v1/CONTRACT.md) | v1 (`feature_request@1`) | artefact_schema | `cuo-cpo` | `cuo/cpo/fr-create`, `cuo/cpo/fr-audit` |

## How to add a new contract

1. Decide the kind: artefact, envelope, or wire-protocol.
2. `mkdir -p cyberos/docs/contracts/<contract-id>/v1/`.
3. Author `CONTRACT.md` with the small contract frontmatter (see `feature-request/v1/CONTRACT.md` as the worked example).
4. Author the schema body — `template.md` for artefact, `schema.json` for envelope/protocol.
5. Author `CHANGELOG.md` with a v1.0.0 entry.
6. Add a row to the index above.
7. For every skill that should consume this contract, add a `depends_on_contracts:` entry to its SKILL.md frontmatter.

## Citations

- DEC-090 (registry v0.2.0) — split contracts from skills.
- Registry README v0.2.0 Part 8 — full skill-vs-contract semantics.
- AGENTS.md §5.1 — `emitted_source_freshness_tier` for cross-source ranking.
