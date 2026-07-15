# CHANGELOG — `cyberos/docs/contracts/software-requirements-specification/`

> Format: Keep a Changelog 1.1.0. SemVer at the contract level.

---

## v1.0.0 — 2026-05-06 (initial release)

### Added

- `CONTRACT.md` — frontmatter contract (12 fields: 8 required, 4 conditional/optional) + 10 required H2 body sections + 3 conditionally-required sections.
- `template.md` — Markdown skeleton `software-requirements-specification-author` adapts.
- This file.

### Driver

Registry v0.2.6 Stage C. The chain needed: `product-requirements-document@1` (product spec) → `software-requirements-specification@1` (system spec) → tech-specs (per-task engineering plans). Without `software-requirements-specification@1`, fr-to-tech-spec would have to consume `product-requirements-document@1` directly — losing the architectural-review seam where engineering signs off on the system design before per-task work begins.

### Backwards compatibility

First version. No predecessor.
