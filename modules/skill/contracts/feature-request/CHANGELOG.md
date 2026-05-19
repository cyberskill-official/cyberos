# CHANGELOG — `cyberos/docs/contracts/feature-request/`

> Format: [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/). SemVer: this contract's `contract_version` advances lockstep with the `feature_request@N` template literal in `template.md`. MAJOR bump = structural change (renaming/adding/removing required sections, renaming/adding/removing required frontmatter fields). MINOR = adding optional fields/sections. PATCH = editorial.

---

## v1.1.0 — 2026-05-06 (promoted to contract; relocated)

### Changed

- Moved from `cyberos/docs/skills/cuo/_shared/feature-request-template/` to `cyberos/docs/contracts/feature-request/` per registry v0.2.0 + DEC-090. Body of `template.md` is byte-identical to v1.0.0.
- Renamed `SKILL.md` → `CONTRACT.md`. Frontmatter contract changed: drops skill-only fields (`allowed_memory_scopes`, `allowed_mcp_tools`, `expects/produces`, `audit`, `confidence_band`, `untrusted_inputs`, `gated_until_phase`); gains contract-only fields (`contract_id`, `contract_version`, `contract_kind`, `template_literal`, `steward_persona`, `escalation_on_breach`, `moved_from`).

### Driver

DEC-090: a contract is not a skill. The previous "schema living as a skill" model conflated two things — packaging (a folder with a `SKILL.md`) and semantics (an artefact that *acts* on input). Promoting schemas to a `_contracts/` namespace makes the dependency graph explicit (consumer skills declare `depends_on_contracts:`), the build pipeline machine-readable, and the architecture honest.

### Backwards compatibility

- Body of `template.md` is byte-identical → existing consumers (`cuo/cpo/feature-request-author` v0.1.0, `cuo/cpo/feature-request-audit` v0.1.0) continue to validate against `feature_request@1`.
- The path move requires every consumer to update its `references/` cross-links. Tracked via the v0.2.0 registry-level CHANGELOG.
- The old location (`cuo/_shared/feature-request-template/`) is deleted in the same commit that lands this v1.1.0.

---

## v1.0.0 — 2026-05-05 (initial extraction)

### Added

- `SKILL.md` (now `CONTRACT.md`) — frontmatter + the audit-rule cross-reference table that maps every FM-NNN, SEC-NNN, COND-NNN audit rule to the corresponding template region.
- `template.md` — verbatim copy of the body skeleton from `feature-request/FR_CREATE_AND_AUDIT.md` v2.0.0 §18.

### Template version

`feature_request@1` — pinned. Any change to required frontmatter or required H2 sections requires bumping to `feature_request@2`, which in turn forces:

1. This contract's `contract_version` to v2.
2. `cuo/cpo/feature-request-audit/RUBRIC.md` (audit rubric) to `audit_rubric@3.0`.
3. Every consumer skill's `depends_on_contracts:` pin to update (`feature-request@v1` → `feature-request@v2`), with a MAJOR `skill_version` bump.
4. A `MIGRATE_FORWARD` audit row appended to memory noting the schema advance for any in-flight `fr-manifest@2` instance.

### Backwards compatibility

This is the first release; nothing to be compatible with except the pre-existing `feature-request/FR_CREATE_AND_AUDIT.md` v2.0.0 prompt, whose §18 template is byte-identical.

---

## How to add a future entry

Standard sub-sections:

- **Added** — new optional frontmatter fields, new optional sections.
- **Changed** — wording within fixed sections (PATCH only).
- **Template** — only on MAJOR bumps; describe the structural change and list every consumer skill that needs updating.
- **Backwards compatibility** — what existing FRs still validate, what needs migration.
