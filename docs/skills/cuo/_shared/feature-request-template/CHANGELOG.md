# CHANGELOG — `cuo/_shared/feature-request-template/`

> Format: [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/).
> SemVer: this skill's `skill_version` advances lockstep with the
> `feature_request@N` template literal in `template.md`. MAJOR bump =
> structural template change (renaming/adding/removing required sections,
> renaming/adding/removing required frontmatter fields). MINOR = adding
> optional fields/sections. PATCH = editorial.

---

## v1.0.0 — 2026-05-05 (initial extraction)

### Added

- `SKILL.md` — frontmatter + the audit-rule cross-reference table that maps
  every FM-NNN, SEC-NNN, COND-NNN audit rule to the corresponding template
  region.
- `template.md` — verbatim copy of the body skeleton from
  `feature-request/FR_CREATE_AND_AUDIT.md` v2.0.0 §18.

### Template version

`feature_request@1` — pinned. Any change to required frontmatter or
required H2 sections requires bumping to `feature_request@2`, which in
turn forces:

1. This skill's `skill_version` to v2.0.0.
2. `cuo/cpo/fr-create/RUBRIC.md` (audit rubric) to `audit_rubric@3.0`.
3. `cuo/cpo/fr-audit/SKILL.md` to acknowledge the bump in CHANGELOG.
4. A `MIGRATE_FORWARD` audit row appended to BRAIN noting the schema
   advance for any in-flight `fr-manifest@2` instance.

### Backwards compatibility

This is the first release; nothing to be compatible with except the
pre-existing `feature-request/FR_CREATE_AND_AUDIT.md` v2.0.0 prompt,
whose §18 template is byte-identical.

## How to add a future entry

Standard sub-sections:

- **Added** — new optional frontmatter fields, new optional sections.
- **Changed** — wording within fixed sections (PATCH only).
- **Template** — only on MAJOR bumps; describe the structural change and
  list every consumer skill that needs updating.
- **Backwards compatibility** — what existing FRs still validate, what
  needs migration.
