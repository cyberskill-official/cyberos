# CHANGELOG — `cyberos/docs/contracts/project-brief/`

> Format: [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/). SemVer at the contract level: MAJOR breaks the body shape (renames a required H2, removes a frontmatter field, changes an enum); MINOR adds a new optional section or extends an enum additively; PATCH is editorial / clarification.

---

## v1.0.0 — 2026-05-06 (initial release)

### Added

- `CONTRACT.md` — frontmatter contract (16 fields: 13 required, 3 conditional/optional) + 9 required H2 body sections + 4 conditionally-required sections.
- `template.md` — Markdown skeleton `requirements-discovery` adapts when synthesising a brief.
- This file.

### Driver

User's request after registry v0.2.3: "the first inputs should be the BRAIN info itself, because i'll create new project and begin interact with it: so BRAIN + human inputs => PRD/SRS/other specs.... => cuo/cpo/feature-request-author". The chain currently jumps directly from "PRD/spec docs" to feature-request-author — there's no upstream skill that consumes BRAIN + human dialogue and produces structured intake. v0.2.4 fills this gap. `project-brief@1` is the artefact that sits between "user has an idea" and "we have a PRD".

### Layout decision

First contract authored under the v0.2.4 simplified flat-folder layout (no `v<n>/` sub-tree per REF-018 in BRAIN). The `contract_version: v1` lives in CONTRACT.md frontmatter; future major versions will append to this single CHANGELOG.

### Backwards compatibility

First version. No predecessor.

### Acceptance evidence (when harness ships)

- A round-trip test: `requirements-discovery` produces a brief; `product-requirements-document-audit` (v0.2.5) validates frontmatter; `product-requirements-document-author` consumes brief; produced PRD references the brief by `memory_id` or path.

## How to add a future entry

Standard sub-sections:

- **Added** — new optional sections, new enum values, new conditional rules.
- **Changed** — semantics changes that don't break existing briefs.
- **Removed** — deprecated sections / fields (always MAJOR).
- **Backwards compatibility** — what briefs from prior versions still validate, what migrates automatically.
