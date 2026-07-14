# CHANGELOG — `cyberos/docs/contracts/product-requirements-document/`

> Format: [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/). SemVer at the contract level: MAJOR breaks the body shape; MINOR adds optional sections / extends enums; PATCH is editorial.

---

## v1.0.0 — 2026-05-06 (initial release)

### Added

- `CONTRACT.md` — frontmatter contract (15 fields: 11 required, 4 conditional/optional) + 11 required H2 body sections + 4 conditionally-required sections.
- `template.md` — Markdown skeleton `product-requirements-document-author` adapts when synthesising a PRD. Carries inline `<!-- authority: ... -->` markers per AGENTS.md §5.3.
- This file.

### Driver

Same conversation as `project-brief@1` (registry v0.2.4). The chain needed both contracts: `project-brief` for the structured intake (consumed by product-requirements-document-author), `prd` for the negotiated product spec (consumed by task-author + downstream skills).

### Layout decision

Authored under the v0.2.4 simplified flat-folder layout (no `v<n>/` sub-tree per REF-018). `contract_version: v1` lives in CONTRACT.md frontmatter.

### Rationale for separating PRD from project-brief

PRD ≠ project-brief. The brief is structured intake (15-20 question interview output); the PRD is the negotiated product spec (multiple iterations, denser content, broader audience). Conflating them collapses two distinct artefacts. The brief is the input to PRD authoring; the PRD is the output.

PRD ≠ SRS. The SRS will register as `software-requirements-specification@1` in registry v0.2.6 and document the system in technical detail; the PRD documents the product at a user-outcome level. Different audiences (product/leadership vs. engineering), different lifecycles (PRD changes when goals shift; SRS changes when tech changes), different content density.

### Backwards compatibility

First version. No predecessor.

### Acceptance evidence (when harness ships)

- A round-trip test: `product-requirements-document-author` produces a PRD from a `project_brief@1`; `product-requirements-document-audit` (registry v0.2.5) validates frontmatter + section presence + authority markers; `task-author` consumes the audited PRD; produced FRs reference the PRD by `memory_id` or path.
- Authority-marker compliance test: every claim in `## Goals` carries at least `llm-explicit` (rejected if `llm-implicit`); every claim in `## User Stories` acceptance criteria carries at least `llm-explicit`.

## How to add a future entry

Standard sub-sections:

- **Added** — new optional sections, new enum values, new authority levels.
- **Changed** — semantics changes that don't break existing PRDs.
- **Removed** — deprecated sections / fields (always MAJOR).
- **Backwards compatibility** — what PRDs from prior versions still validate, what migrates automatically.
