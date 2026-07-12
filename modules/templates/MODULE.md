# templates module - presentation shells for CyberOS deliverables (FR-TPL-001)

Scope: HTML faces only. This module stores the CDS-adapted shells every renderer and skill uses to
present deliverables. It generates no content, reads no data, and owns no workflow - consumers
substitute slots (contracts/TEMPLATE.md) and ship the result. Markdown remains the authored source
of every deliverable (FR-DOCS-002 doctrine); these shells are how that source is SHOWN.

| path | what |
|---|---|
| cds/tokens.css | vendored CDS tokens (--cs-*), pinned per cds/PROVENANCE.md |
| cds/glass.css | vendored Liquid Glass materials (opt-in .cs-surface-*) |
| contracts/TEMPLATE.md | template@1 - slot grammar, escape set, self-containment rules |
| html/deliverable.html | any single artefact page (FR spec, SOW, PRD, runbook...) |
| html/status-hub.html | command deck + tab panels (FR-DOCS-006 consumes) |
| html/catalog.html | card grid with facet strip |

Rules: shells style through --cs-* tokens only (hex lives solely in vendored cds/*.css); rendered
output must work from file:// (no external fetch - fonts fall back through the CDS system stack);
consumers never fork a shell - extend via slot content, or amend this module by FR.
