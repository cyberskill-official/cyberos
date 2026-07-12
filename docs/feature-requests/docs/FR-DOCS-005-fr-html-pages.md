---
id: FR-DOCS-005
title: "Per-FR CDS HTML pages - every spec renders to its own self-contained deliverable page with assets"
module: docs
priority: MUST
status: ready_to_implement
class: product
verify: T
phase: Wave D - visual deliverables
owner: Stephen Cheng (CTO)
created: 2026-07-12
shipped: null
memory_chain_hash: null
related_frs: [FR-TPL-001, FR-DOCS-004, FR-DOCS-006]
depends_on: [FR-TPL-001, FR-DOCS-004]
blocks: []
source_pages:
  - tools/docs-site/md.mjs
  - modules/templates/html/deliverable.html
source_decisions:
  - "2026-07-12 operator decision: FR deliverables get better UI, visual-rich (images, videos), CDS-styled, directly viewable on all platforms."
  - "2026-07-12 viewing answer: published site + local build; generated HTML stays uncommitted (FR-DOCS-002 doctrine holds)."
language: javascript (node stdlib) + html
service: tools/docs-site/
new_files:
  - tools/docs-site/render-fr-pages.mjs
  - tools/docs-site/tests/test_render_fr_pages.sh
modified_files:
  - tools/docs-site/build.sh
  - tools/docs-site/render-fr-catalog.mjs
---

# FR-DOCS-005: Per-FR CDS HTML pages

## §1 - Description

Every FR folder renders to one CDS-styled page a human can read anywhere - spec body, audit verdict, media - while markdown stays the only authored source.

Normative clauses:

1. A builder `tools/docs-site/render-fr-pages.mjs` MUST walk `docs/feature-requests/<module>/<STEM>/spec.md`, render markdown via the existing dependency-free `md.mjs`, and emit `dist/website/frs/<module>/<STEM>/index.html` through the `deliverable@1` template (FR-TPL-001), node stdlib only.
2. Each page MUST show: id, title, status badge, module + class + priority badges, key frontmatter (created/shipped/depends_on/blocks as links when those FRs have pages), the rendered spec body with heading anchors per §-section, and the audit verdict + score when `audit.md` exists (rendered below the spec, visually separated).
3. Assets MUST work: `<STEM>/assets/**` is copied beside the page; relative `assets/...` references in the markdown resolve unchanged; image links render as `<img>`, and links to video files (mp4/webm/mov) render as `<video controls>` - both capped to content width.
4. Self-containment (template rule): CDS tokens + shell styles inlined into each page; the only external references are the page's own relative assets - pages work from file://.
5. `build.sh` MUST run the builder before nav generation; the FR catalog's cards MUST link to the pages (`render-fr-catalog.mjs` href swap); determinism and honesty per house rules: byte-identical rebuilds, unreadable spec fails naming the file, missing referenced asset fails non-zero (FR-DOCS-002 §1 #7 discipline).
6. Scale envelope: the full corpus (~486 pages) MUST build in under 30s on CI hardware and add no external asset weight beyond the corpus's own media.

## §2 - Why this design

Rendering through the templates module is what makes "CDS everywhere" a property instead of a habit - one shell, hundreds of pages. Reusing md.mjs keeps a single markdown dialect across docs and FRs.

## §3 - Contract

Page path: `frs/<module>/<STEM>/index.html`; template `deliverable@1`; asset copy: sibling `assets/`. Build summary line: `fr-pages: N pages, M assets copied, K with audits`.

## §4 - Acceptance criteria

1. **Pages render for the corpus** (§1 #1, #2) - fixture tree (spec+audit+asset mix) renders all pages; badges/anchors/audit block assert on emitted HTML.
2. **Media works** (§1 #3) - fixture with png + mp4 emits `<img>` + `<video controls>`; asset files copied beside the page; relative hrefs unchanged.
3. **Self-contained** (§1 #4) - a rendered page has zero http(s)/absolute asset references; tokens present inline.
4. **Wired + deterministic + honest** (§1 #5) - build.sh invokes it; double build byte-identical; broken spec and missing asset each fail naming the file.
5. **Catalog links** (§1 #5) - fr-catalog cards href the new pages (fixture assert).
6. **Envelope** (§1 #6) - timed corpus build under the cap on the runner.

## §5 - Verification

`tools/docs-site/tests/test_render_fr_pages.sh`: t01_corpus_renders, t02_media, t03_selfcontained, t04_wired_deterministic_honest, t05_catalog_links, t06_envelope. (AC 1-6.)

## §6 - Implementation skeleton

Walk folders -> frontmatter + md.mjs body -> slot-substitute deliverable.html -> write + copy assets; video extension map at top; audit.md rendered with the same md pipeline.

## §7 - Dependencies

FR-TPL-001 (shell + tokens), FR-DOCS-004 (layout). FR-DOCS-006 links these pages from the board.

## §8 - Example payloads

`fr-pages: 486 pages, 12 assets copied, 209 with audits`

## §9 - Open questions

None blocking. PDF export of a page is future scope (print stylesheet suffices meanwhile).

## §10 - Failure modes inventory

1. Giant video bloats the site - assets are copied as-is, never inlined; page weight = media weight, a content decision.
2. Spec with raw HTML - md.mjs escapes unsupported constructs (FR-DOCS-002 §1 #3), so injection dies at the renderer.
3. Broken relative asset ref - missing-asset build failure names spec + path.
4. 486-page nav pollution - pages are NOT in the shared nav; entry via catalog, board, and direct links.
5. Template drift - data-template-id asserted in t03, so a fork or hand-edit surfaces in tests.

## §11 - Implementation notes

Keep the summary line stable (deploy log health). Anchor slugs reuse check_doc_anchors' grammar so §-citations deep-link.

*End of FR-DOCS-005.*
