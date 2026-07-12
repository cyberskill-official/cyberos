---
id: FR-TPL-001
title: "templates module - CDS-adapted HTML shells (template@1) every workflow/skill renders deliverables through"
module: templates
priority: MUST
status: done
class: product
verify: T
phase: Wave D - visual deliverables
owner: Stephen Cheng (CTO)
created: 2026-07-12
shipped: 2026-07-12
memory_chain_hash: null
related_frs: [FR-DOCS-004, FR-DOCS-005, FR-DOCS-006, FR-SKILL-120]
depends_on: []
blocks: [FR-DOCS-005, FR-DOCS-006]
source_pages:
  - https://github.com/cyberskill-official/design-system (v1.3.0)
  - tools/docs-site/render-docs.mjs
source_decisions:
  - "2026-07-12 operator decision: deliverable HTML must adapt CDS; a new templates module stores the shells so workflows/skills stop hand-rolling page HTML."
  - "2026-07-12 architecture answer: Option A - HTML faces, markdown bones. Templates are the faces' single home."
language: html + css + markdown
service: modules/templates/
new_files:
  - modules/templates/MODULE.md
  - modules/templates/cds/tokens.css
  - modules/templates/cds/glass.css
  - modules/templates/cds/PROVENANCE.md
  - modules/templates/contracts/TEMPLATE.md
  - modules/templates/html/deliverable.html
  - modules/templates/html/status-hub.html
  - modules/templates/html/catalog.html
modified_files: []
---

# FR-TPL-001: templates module (CDS shells)

## §1 - Description

One module owns the HTML faces of every deliverable, so renderers and skills compose pages from audited shells instead of inventing markup.

Normative clauses:

1. A module `modules/templates/` MUST exist with `MODULE.md` (scope: presentation shells only - no content generation, no data reads) and the file set in `new_files`.
2. CDS MUST be vendored, not linked: `cds/tokens.css` and `cds/glass.css` copied verbatim from `@cyberskill/tokens` / `@cyberskill/react` dist of design-system v1.3.0, with `cds/PROVENANCE.md` recording source repo, version, commit, copy date, and the re-vendor procedure. No CDN reference anywhere.
3. A contract `template@1` MUST be defined at `contracts/TEMPLATE.md`: a template is an HTML file with named slots `{{slot:<name>}}` (text-safe) and `{{slot:<name>:html}}` (pre-rendered HTML), a required `data-template-id` on the root element, and a self-containment rule - rendered output MUST work from file:// with no external network fetch (styles inlined or relative).
4. Three shells MUST ship: `deliverable.html` (any single artefact page: header with id/status/module badges, meta strip, body slot, assets-aware figure styling, footer provenance), `status-hub.html` (command-deck strip + tab bar + three tab panels; hash-routed, JS-free fallback), `catalog.html` (card grid with facet strip).
5. All shell styling MUST use `--cs-*` tokens (or derive from them); the only hex literals allowed live inside the vendored `cds/*.css`. Typography per CDS (`--cs-font-family-ui`, Be Vietnam Pro stack).
6. Templates MUST be render-engine-agnostic: consumable by a node builder or by an agent doing string substitution - slot substitution is defined as plain string replacement with HTML-escaping for text slots, and the contract carries the exact escape set.

## §2 - Why this design

Vendoring pins the look to an audited CDS release (the design system's own doctrine: three canonical files, everything else regenerable) while keeping deliverables buildable offline. Slots-as-string-replacement keeps the contract usable by both tooling and doc-driven agents - same reason the skill chain is markdown-first.

## §3 - Contract

`template@1`: `{{slot:title}}`, `{{slot:body:html}}`, `data-template-id="deliverable@1"`; escape set: `& < > "` for text slots. Full grammar in contracts/TEMPLATE.md.

## §4 - Acceptance criteria

1. **Vendored, pinned, provenanced** (§1 #2) - tokens.css/glass.css byte-match the pinned upstream dist; PROVENANCE.md names version+commit; grep finds no CDN/external URL in any shell.
2. **Contract complete** (§1 #3, #6) - TEMPLATE.md defines slot grammar, escape set, data-template-id rule, self-containment rule; each shell carries its id.
3. **Shells parse and self-contain** (§1 #4) - each shell substituted with fixture slots yields HTML whose only asset references are inline or relative.
4. **Token-only styling** (§1 #5) - no hex color outside cds/*.css (checker-level grep).

## §5 - Verification

`tools/docs-site/tests/test_templates_module.sh`: t01_vendored_pinned, t02_contract_complete, t03_shells_selfcontained, t04_token_only. (AC 1-4.)

## §6 - Implementation skeleton

Copy dist css from the design-system clone at the pinned commit; shells authored once with CDS variables; contract mirrors the skills' contracts formatting.

## §7 - Dependencies

None upstream. Blocks FR-DOCS-005/006 (they consume the shells). FR-SKILL-120 cites the contract from authoring skills.

## §8 - Example payloads

`<article data-template-id="deliverable@1"><h1>{{slot:title}}</h1><section>{{slot:body:html}}</section></article>`

## §9 - Open questions

None blocking. Style-packs (CDS Part 22) are future scope; the shells take the default pack.

## §10 - Failure modes inventory

1. Upstream CDS drifts - PROVENANCE re-vendor procedure + byte-match test catch silent edits.
2. Template edited into non-self-contained form - AC 3 fixture render fails.
3. Slot injection (unescaped user text) - text slots escape `& < > "`; html slots are builder-owned only, stated in the contract.
4. Shell forked per consumer - data-template-id + the contract make forks visible in review.
5. Dark-mode regressions - tokens.css light default; [data-theme] opt-in preserved as vendored.

## §11 - Implementation notes

Keep shells minimal; page-specific layout belongs to the consumer's slot content, not new shells.

*End of FR-TPL-001.*
