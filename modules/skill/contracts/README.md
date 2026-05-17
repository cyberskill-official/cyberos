# `cyberos/docs/contracts/` — Versioned schema artefacts

> A **contract** is a versioned schema that one or more skills produce or consume. Contracts are NOT skills — they don't act, they constrain. Skills declare contract dependencies via `depends_on_contracts:` in their frontmatter so the build pipeline can ship contract + skill as a single bundle.

## Why contracts live separately from skills

Skills under `cyberos/docs/skills/` *do work*: take input, run inference or deterministic logic, emit output, write an audit row. Contracts under this folder *constrain shape*: artefact frontmatter, envelope JSON schemas, wire protocols. Conflating the two leads to skill folders with empty `allowed_mcp_tools: []`, `expects: null`, `confidence_band: 1.0` — schemas wearing skill costumes. The v0.2.0 registry split (DEC-090) ends that conflation.

## Layout (locked: 2026-05-06; simplified 2026-05-06 in registry v0.2.4)

```
cyberos/docs/contracts/
├── README.md                                   # this file
└── <contract-id>/                              # kebab-case
    ├── CONTRACT.md                             # frontmatter + spec (replaces SKILL.md)
    ├── template.md                             # the schema body (artefact contracts only)
    ├── schema.json                             # the schema body (envelope contracts only)
    ├── protocol.md                             # operational protocol (wire-protocol contracts only)
    ├── CHANGELOG.md                            # all version history (single file, all majors)
    └── examples/                               # optional reference instances
```

The major version is tracked inside `CONTRACT.md`'s frontmatter (`contract_version: v1`), not in the folder hierarchy. When a contract MAJOR-bumps to v2, the options are:

- **Option A (preferred when no parallel maintenance is needed):** keep the flat layout. CONTRACT.md grows to document v1 (deprecated) + v2 (current); template.md becomes template-v2.md + template-v1.md (kept until all consumers migrate). Single CHANGELOG entry threads through both majors.
- **Option B (revive a `v<n>/` sub-tree):** if parallel maintenance becomes burdensome (e.g., a partner connector is pinned to v1 while internal skills migrate to v2), reintroduce `v<n>/` folders at THAT point, with v1 contents moved into `v1/` and v2 contents in `v2/`. Only do this when there's evidence the simpler layout broke down.

Per registry v0.2.4 audit (REF-018 in BRAIN), the v<n>/-folder layout was over-engineered for current scale; it solved a parallel-version problem we don't have yet. Defer the structural complexity until it pays for itself.

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
    pin_path:  cyberos/docs/contracts/feature-request/
```

The registry validator confirms:

1. The path resolves to a real `CONTRACT.md`.
2. The skill body's references to that contract use the declared path (no hard-coded paths to alternate versions).
3. On contract MAJOR bumps, every declared consumer is updated (or explicitly opts in to staying on the older version with a CHANGELOG entry).

## Index of contracts

| Contract | Latest version | Kind | Stewarded by | Consumed by |
| --- | --- | --- | --- | --- |
| [`feature-request`](./feature-request/CONTRACT.md) | v1 (`feature_request@1`) | artefact_schema | `cuo-cpo` | `cuo/cpo/fr-author`, `cuo/cpo/fr-audit`, `cuo/chief-technology-officer/fr-to-tech-spec` |
| [`nats-subjects`](./nats-subjects/CONTRACT.md) | v1 (`nats_subjects@1`) | wire_protocol | `cuo-cto` | `cuo/cpo/fr-author`, `cuo/cpo/fr-audit`, `cuo/chief-technology-officer/fr-to-tech-spec`, the supervisor |
| [`project-brief`](./project-brief/CONTRACT.md) | v1 (`project_brief@1`) | artefact_schema | `cuo-cpo` | `cuo/cpo/requirements-discovery`, `cuo/cpo/prd-author` |
| [`prd`](./prd/CONTRACT.md) | v1 (`prd@1`) | artefact_schema | `cuo-cpo` | `cuo/cpo/prd-author` v0.1.0+, `cuo/cpo/prd-audit` v0.1.0+, `cuo/chief-technology-officer/srs-author` v0.1.0+ (input), `cuo/cpo/fr-author` v0.3.0+ (planned) |
| [`srs`](./srs/CONTRACT.md) | v1 (`srs@1`) | artefact_schema | `cuo-cto` | `cuo/chief-technology-officer/srs-author` v0.1.0+, `cuo/chief-technology-officer/srs-audit` v0.1.0+, `cuo/chief-technology-officer/fr-to-tech-spec` v0.2.0+ (input context) |
| [`impl-plan`](./impl-plan/CONTRACT.md) | v1 (`impl_plan@1`) | artefact_schema | `cuo-cto` | `cuo/chief-technology-officer/spec-to-impl-plan` v0.1.0+ |

## How to add a new contract

1. Decide the kind: artefact, envelope, or wire-protocol.
2. `mkdir cyberos/docs/contracts/<contract-id>/`.
3. Author `CONTRACT.md` with the small contract frontmatter (see `feature-request/CONTRACT.md` as the worked example). Carry `contract_version: v1` in the frontmatter — that's where major-version is tracked.
4. Author the schema body — `template.md` for artefact, `schema.json` (+ optional `protocol.md`) for envelope/wire-protocol.
5. Author `CHANGELOG.md` with a v1.0.0 entry. Future major versions append to this single CHANGELOG.
6. Add a row to the index above.
7. For every skill that should consume this contract, add a `depends_on_contracts:` entry to its SKILL.md frontmatter (with `pin_path: cyberos/docs/contracts/<contract-id>/`, no v<n>/).
8. **Run the audit-fix-audit loop** per registry README Recipe 13 — drift between contract intent and consumer reality is common; the loop catches it before merge.

## Citations

- DEC-090 (registry v0.2.0) — split contracts from skills.
- Registry README v0.2.0 Part 8 — full skill-vs-contract semantics.
- AGENTS.md §5.1 — `emitted_source_freshness_tier` for cross-source ranking.
