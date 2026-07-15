---
title: Risk register
source: website/docs/reference/risk-register.html
migrated: TASK-DOCS-002
---

# Risk register

The register tracks the top risks reviewed in the Founder weekly sync, across six categories: technical, compliance, operational, strategic, financial, and legal. Each risk carries a likelihood x impact score, an owner, a mitigation, a contingency, and a status. The set is pulled from the "Top 15 risks" and extended with the risks one would expect for a 24-month, 23-module, regulated-market platform build. Severity is re-cast here as impact for heatmap clarity.

Numbering: RSK-01 through RSK-15 are the canonical top risks; R-EXT-* additions are inferred from project context and marked with their rationale. Summary counters on the site page track risks tracked, high/catastrophic, open, mitigated, accepted, and the six categories.

## Likelihood x impact heatmap

The generated site renders the register as a likelihood x impact heatmap plus a filterable table (category, likelihood, impact, status, free-text search). Cells are colour-coded by composite score:

- low: low concern
- med: monitor
- high: active mitigation required
- crit: sprint-blocking, Founder review

Any high-likelihood / high-impact cell is "sprint-blocking": it auto-creates a Question to the Founder via the Compliance Cockpit.

The risk rows themselves (ID, title, category, owner, likelihood, impact, score, status, description, mitigation, contingency, last reviewed, reference) are rendered client-side by the interactive page on the generated site; the row data did not survive the HTML-to-markdown migration, so it is not reproduced on this page.

## Operational rules

- Severity x likelihood produces a heat-mapped score; any High-High lands on the Compliance Cockpit and triggers a Question to the Founder.
- Risks are reviewed weekly during the Founder weekly sync; status (Open / Mitigated / Realised / Closed) is updated.
- A realised risk triggers an AAR (After-Action Review); the AAR is captured in memory Layer 2 with the `lesson-learned` tag and is surfaced in future similar contexts via GraphRAG.
- New risks added between phases require Founder approval; they are not auto-accepted from CUO's suggestion stream.

## Changelog

History lives in the [changelog](./changelog.html); this page describes only the current state.
