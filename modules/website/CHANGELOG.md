# Changelog — Website & Infrastructure

## 2026-05-18 — Wave-1+2 impl sessions 15-18: embed sidecar, NFR audits, slice-3 universal wiring + admin REST

End-of-day continuation of the Wave-1+2 implementation phase. Module-specific work moved to per-module changelogs (see [AUTH](../auth/changelog.html), [MEMORY](../memory/changelog.html), [AI](../ai/changelog.html)).

**[AI] FR-AI-019 embedding sidecar closed end-to-end.** New `services/embed-sidecar/` — FastAPI server with mock + real backends behind `CYBEROS_EMBED_MODE`. `POST /embed` matches the Rust `EmbeddingClient` wire protocol. **10/10 pytest cases pass.**

**NFR audit-pair coverage.** All 153 NFR specs across 18 module directories now have `.audit.md` siblings on the `nfr-spec@1` rubric. 153/153 scored 10/10.

---

## 2026-05-15 — UI bug fixes from screenshots (Mermaid syntax + diagram sizing + title overlap + mobile overflow + PRD/SRS sweep cleanup)

Stephen flagged five UI bugs from live deploy screenshots; all fixed.

**Bug 1 — Hero h1 overlap (`.h-1 mb-3 + p` collision on index.html):**
- `assets/styles.css:325–355` — bumped `.h-1` line-height 1.25 → 1.3, `margin-block-end` 1.25rem → 1.5rem, added `padding-block-end: 0.25rem` to protect BVP descenders.
- Changed sibling rule line 346: `margin-block-start: 0` → `0.5rem !important` for h-display + h-1 successors. Guarantees min-gap even when Tailwind `mb-3` overrides.

**Bug 2 — Mermaid "Syntax error in text" in memory §3:**
- Root cause: `FILES["memories/<kind>/<hex>/<file>.md"]` — Mermaid 11.4.1 parses `<kind>`/`<hex>`/`<file>` as unknown HTML tags inside node labels.
- Fixed 3 locations in `modules/memory.html` (lines 288, 454, 503): `<kind>` → `{kind}` etc.
- Fixed 1 location in `modules/hr.html:841` (same root cause inside a Mermaid sequence).
- Repo-wide sweep confirmed no other `<placeholder>` patterns in Mermaid blocks.

**Bug 3 — Stage 0→5 flowchart rendered microscopic:**
- Root cause: `.mermaid svg { max-width: 100%; height: auto; }` forced wide flowcharts to shrink to ~700px parent, making labels unreadable.
- Fix at `assets/styles.css:429–449`: dropped `display:flex; justify-content:center;` (which fought overflow scroll), changed `max-width: 100%` → `max-width: none !important` on SVG. Now wide diagrams scroll horizontally instead of shrinking. Added scrollbar styling for visual hint.

**Bug 4 — Mobile horizontal overflow:**
- Added 70-line mobile safety net at `assets/styles.css:1017–1085`:
  - `html, body { overflow-x: hidden; max-width: 100vw; }` to clamp viewport
  - `.container { min-width: 0 }` so flex/grid children can shrink
  - `.bbg-card { overflow-wrap: anywhere }` so long URLs/codes wrap
  - `@media (max-width: 768px)`: tables wrap their card in scroll, code blocks pre-wrap, fact-grid `minmax(140px, 1fr)`, h-display clamp 1.875–2.5rem
  - `@media (max-width: 480px)`: tighter container padding + 120px fact-card minimum
  - Mermaid `max-height: 70vh` on mobile to prevent monstrous portrait diagrams

**Bug 5 — Lingering PRD/SRS references:**
- 47 textual edits across 28 HTML files in `website/docs/` (per Agent sweep). Removed: "PRD/SRS narrative remains authoritative" disclaimers (23), "PRD coverage" eyebrows, broken `<a href="#"></a>` empty anchors, "Generated from PRD + SRS source" footer, "DEC-NNN in SRS" → "DEC-NNN" rewrites (5 in infrastructure.html + 1 in ten.html), persona "draft PRD/SRS" chip rephrases. Preserved: the two intentional github.com canonical-spec links in `fr-catalog.html` lines 56–57.
- Grep verification: `\bPRD\b|\bSRS\b` across `website/docs/*.html` → 2 hits, both intentional.

Verified: memory.html Mermaid no longer has `<kind>/<hex>/<file>` patterns; styles.css line counts went from 1018 → 1085. The fix should ship cleanly to Cloudflare Pages on next deploy.

---

## 2026-05-14 — Code-block contrast fix + PRD/SRS sweep + repair regression + Research Mode brief

- **Fixed code-block invisible-text bug.** A late-stage override in `assets/styles.css` (`.codeblock { background: var(--bg-code) }`) was flipping the dark `--neutral-900` background to a light `--bg-code` while leaving text colour at light `--neutral-100` → code invisible on auth.html and other module pages. Removed the `background` override; kept the `backdrop-filter: none` (which prevents glass-leakage from a glass parent).
- **Swept PRD/SRS back-references out of the docs site.** The docs site is now the single source of truth — removed every `PRD §X.Y`, `SRS §X.Y`, "per PRD", "see PRD", "sourced from PRD" reference across 33 HTML files. Replaced `Source: PRD §...` / `Reference: SRS §...` labels with `(covered on this page)`. Net 29,710 substitutions.
- **Repaired regex over-strip regression.** The sweep's separator-collapse regex had a false-positive: `(/)\s*(/)` matched `://` in URLs and collapsed them to `:/`. 175 URLs (Google Fonts, jsdelivr CDN, GitHub repo links, SVG xmlns, etc.) were silently broken across all HTML files. Wrote a repair pass that restored `https?:/` → `https?://` plus cleaned up 83 empty `<strong></strong>` / `<em></em>` / `<code></code>` tags and orphan-separator artifacts. Zero broken URLs verified after repair.
- **Added `docs/RESEARCH_MODE_BRIEF.md`** — canonical brief for the pre-lock comprehensive review via Claude Chat's Research Mode. Contains the full prompt covering 8 review dimensions (strategic coherence, architecture, spec quality, UX, info architecture, compliance, GTM, next-7-days actions), the 10-file input bundle (~250 KB total of curated source-of-truth markdown), why we DON'T attach the docs HTML (token waste + visual UX requires live URL crawl), how to drive the mid-review conversation, and how to operationalize the returned document.

---

## 2026-05-14 — Heading line-height fix + FR authoring workflow guide

- Fixed heading collision on H2 elements caused by the Be-Vietnam-Pro font swap. BVP has taller ascenders + descenders than Inter at the same `font-size`. The previous Inter-tuned `line-height: 1.05` (h-display), `1.15` (h-1), `1.25` (h-2) values were too tight and let the heading bounding box collide with the following paragraph (visible on the "The substrate · the catalog · the orchestrator" H2 on index.html). Updated `assets/styles.css` heading rhythm: h-display 1.05→1.1, h-1 1.15→1.25, h-2 1.25→1.4, h-3 (added) 1.45. Added explicit `margin-block-end` on each + an `h-* + * { margin-block-start: 0 }` rule to neutralise Tailwind `mb-*` collapse.
- Added `feature-request-audit skill` — canonical playbook for the post-strip FR re-authoring lifecycle. Covers the mental model, file layout, standalone vs chained flows, the standard module-slice-1 recipe (5–7 FRs per slice), how FRs surface back to the docs site, status state machine, task integration paths, and a fully worked FR-AUTH-001 example. Designed to keep open while authoring.

---

## 2026-05-14 — Comprehensive audit + FR catalog strip + Mermaid mass-fix

Added `docs/AUDIT_AND_PLAN_2026_05_14.md` — single comprehensive audit + build-readiness plan covering UI glitches (severity-ranked), FR landscape, per-module build sequence for the 19 unbuilt modules with slice-1 outlines, and strategic followups. Designed as the source of truth for the next 2 weeks of work.

**FR catalog strip (per user decision: strip-everything).** Stripped:
- All 22 module pages: each "Functional Requirements" section (the `<section id="functional-requirements">` block, lines ~789–820 across modules) replaced with a stub linking to the `feature-request-author` Agent Skill workflow. 23/23 pages patched cleanly via regex sweep.
- `website/docs/reference/fr-catalog.html`: 1006-line generated catalog replaced with a 70-line stub explaining the rebuild + how to author new FRs via the skill module.

**Partially stripped (cross-refs remain — call to extend):**
- `website/docs/reference/nfr-catalog.html` — still has 137 FR refs (NFRs are described in terms of which FRs they constrain)
- `website/docs/reference/risk-register.html` — still has 51 FR refs (risks reference the FRs they affect)
- Module pages — still have inline FR refs in Dependencies tables, NFR descriptions, KPIs, References footers (~200 total across all)
- `docs/prd/PRD.md` (393 FR refs) and `docs/srs/SRS.md` (206 FR refs) — preserved as authoritative spec narrative; .docx originals also preserved

The "strip-everything" decision affects ~434 remaining FR cross-references — these are inline within sentences and tables. They become broken references until re-authored. To clean them up, separate decisions are needed on whether to: keep them as broken refs (will rewrite organically as new FRs come online), replace with `(FR pending)` markers, or remove the surrounding sentences entirely.

**Mermaid mass-fix across 28 pages:**
- `<br/>` → `<br>` — 754 instances replaced, ALL inside `<div class="mermaid">` blocks (zero outside, verified). This fixes the "Cursorvia MCP tool" text-collapse bug seen on `modules/memory.html` where Mermaid 11.4.1 strips self-closed `<br/>` tags inside quoted node labels.
- Pastel `classDef` palette → Umber/Ochre brand: 127 instances recolored across all non-index module + architecture pages. Map: emerald-100→umber-50, blue-100→umber-100, purple-100→ochre-300, amber-100→ochre-50, pink-100→ochre-100, indigo-100→umber-200, slate-100→neutral-100, yellow-100→ochre-50, violet-100→ochre-50. Strokes likewise mapped to umber-500 / ochre-700 / neutral-400.
- 6 broken internal links to non-existent architecture pages fixed: `architecture/services.html` (5 refs from learn/hr/esop/rew/inv) and `architecture/runtime.html` (1 ref from chat) redirect to `architecture/infrastructure.html` (the closest topical match).

Net code change: 36 files, ~1,417 insertions / ~2,641 deletions. Plus new files `docs/AUDIT_AND_PLAN_2026_05_14.md` (the master plan) and `website/docs/assets/tailwind.min.css` (16.7 KB vendored from prior commit).

Open items pending Stephen's call (per audit doc):
1. Whether to strip the remaining 434 inline FR cross-refs (in NFR catalog / risk register / module sub-sections) or let them rewrite organically.
2. AUTH RFC's 5 open questions need answers before slice 1 codes.
3. Redeploy `website/docs/` via wrangler so the brand + Tailwind + Mermaid + strip fixes go live.

---

## 2026-05-14 — Vendor Tailwind (CDN was silently failing on Cloudflare Pages)

After the brand-rebuild deploy at https://5cc09eb6.cyberos-docs.pages.dev/, the layout was still broken: hero text and SVG stacked, bento stats stacked one-per-row, 22-module catalog stacked one-per-row, the three shipped-module cards stacked one-per-row. Every `grid`, `grid-cols-*`, `lg:grid-cols-*`, `flex`, `gap-*`, `mt-*` utility was dead because the Tailwind CDN script (`https://cdn.tailwindcss.com`) was loading (200, 14 KB body, no console errors) but **never injected its generated utility CSS** — confirmed by `getComputedStyle` showing `.grid` resolving to `display:block` and `typeof window.tailwind === 'undefined'`. No CSP headers, no module/MIME errors, just a silent failure of Tailwind Play CDN's runtime JIT inside Cloudflare Pages.

Fix in this commit:

- Generated a 16.7 KB static `assets/tailwind.min.css` via `npx tailwindcss@3.4.17` with content-paths covering all 32 HTML files (index + 22 modules + 4 architecture + 4 reference + 1 nav asset). Preflight disabled (we already have `assets/styles.css` setting base styles). All classes the pages actually use are baked in: `.grid`, `.flex`, `.container`, `.grid-cols-{2,3,5,6}`, `.lg:grid-cols-{4,5,6,8,12}`, `.md:grid-cols-{2,3,4}`, `.gap-{1..10}`, `.mt-{0..16}`, `.py-*`, `.text-{xs..2xl}`, `.font-{medium,semibold,bold,black}`, `.items-center`, `.justify-between`, etc.
- Replaced `<script src="https://cdn.tailwindcss.com"></script>` with `<link rel="stylesheet" href="assets/tailwind.min.css">` across all 32 HTML files (relative paths corrected: `assets/...` from index, `../assets/...` from subdirs).
- Result: layout works without runtime JavaScript, no third-party CDN dependency, faster (16.7 KB CSS gzips to ~4 KB vs the CDN's 14 KB JS + runtime compile + style injection).

To regenerate when classes change:

```bash
cd /tmp && cat > input.css <<'CSS'
@tailwind base; @tailwind components; @tailwind utilities;
CSS
cat > tailwind.config.js <<'JS'
const docs = '/path/to/cyberos/website/docs';
module.exports = {
  content: [`${docs}/*.html`, `${docs}/modules/*.html`, `${docs}/architecture/*.html`, `${docs}/reference/*.html`, `${docs}/assets/*.html`],
  corePlugins: { preflight: false },
};
JS
npx tailwindcss@3.4.17 -c tailwind.config.js -i input.css -o /path/to/cyberos/website/docs/assets/tailwind.min.css --minify
```

Once the docs site moves to a real build pipeline (Vite, Astro, or just a Makefile), this becomes one-line in the build command.

---

## 2026-05-14 — Docs site brand rebuild

Live deploy at https://fe8d68ee.cyberos-docs.pages.dev/ was off-brand: hero triangle used pastel purple/blue/green/yellow Mermaid-default palette; bento stats used per-stat blue/purple/emerald/amber/rose; phase strips used five different pastels; persona accents were purple; compliance ring was blue/green/yellow concentric; tech-stack Mermaid `classDef` was pastel-rainbow. None of these aligned with the design-system DESIGN.md anchors (Umber `#45210e` + Ochre `#f4ba17`) or with Part 21 Liquid Glass defaults.

Root cause: page authoring drift, not design-system fault. Glass classes (`.surface-light/.surface-standard/.surface-heavy`) and `--glass-*` tokens were already defined in `assets/styles.css` and `assets/tokens.css`, but `index.html` hand-coded inline Tailwind palette utilities (`bg-blue-50`, `text-purple-700`, etc.) instead of consuming them.

Fixes in this commit:

- `website/docs/index.html` — 534 lines changed. All inline pastel hex fills in the hero SVG triangle, phase strips, and compliance ring SVG converted to Umber/Ochre tints (`#f5ede6`, `#e8d4c2`, `#fef6e0`, `#fde7b3`, `#f9c64f`, `#cba88a`). All Tailwind palette utilities (`bg-blue-*`, `text-purple-*`, `bg-emerald-*`, `text-amber-*`, `text-rose-*`) replaced with `style="color:var(--umber-700)"` / `style="background:var(--ochre-50)"`. Tech-stack Mermaid `classDef` repainted to brand palette. CyberOS wordmark gradient changed from `blue→purple→emerald` to `umber→ochre`. v2026.05 pill changed from `bg-blue-50 text-blue-700` to `ochre-50 + umber-700`. Phase summary gradient changed from `from-blue-50 via-purple-50 to-emerald-50` to `umber-50 → ochre-50`. Compliance ring concentric gradients changed from `blue→green→yellow` to `neutral→umber→ochre` (warmest at the inner Vietnam home regime).
- `website/docs/assets/tokens.css` — `--font-sans`/`--font-body`/`--font-display` reordered: Be Vietnam Pro listed before Inter per design-system mandate. Comment notes the Vietnamese-first commitment.
- `website/docs/assets/styles.css` — added the `@import` for Be Vietnam Pro so the font actually loads. Added `+101 lines` of design-system utilities: `.ds-modpill` + `.ds-modpill--future` (module navigator pills), `.pill--brand`, `.tile` + `.tile--accent`. Added a transitional-safety-net override block that converts any remaining Tailwind palette utilities on the 22 module pages + 4 architecture pages + 4 reference pages to brand tokens (`bg-blue-*` → `--umber-100`, `bg-purple-*` → `--ochre-50`, etc.) so the brand wins site-wide even before each page is hand-cleaned. Saves ~620 individual edit operations.
- `website/docs/assets/scripts.js` — Mermaid `themeVariables.fontFamily` reordered to Be Vietnam Pro first.

Zero Tailwind palette leaks remain in `index.html` (was 13). Across the rest of the docs site there are still 620 leaks but the new safety-net rules in `styles.css` neutralise them visually until each page is cleaned.

Design-system suggested followups (not landed in this commit):
1. Add Part-21 sub-section "§21.x — Theming third-party renderers" with the Mermaid `themeVariables` recipe, so the next docs author doesn't re-invent it.
2. Promote `.tile`, `.pill--brand`, `.ds-modpill` from the docs site into `design-system/DESIGN.md` Part 3 as first-class component specs.
3. Ship `tools/design-system-lint.{ts,py}` per Part 15 — flag Tailwind palette utilities (`bg-blue-*` etc.) and off-anchor `fill:#` hexes at commit time.

---

## 2026-05-14 — Consolidation pass

Moved all CyberOS-related artifacts into a single umbrella at `cyberos/`:

- `workbench/CyberOS-docs/` → `cyberos/website/docs/`
- `workbench/CYBEROS_STRATEGY.md` → `cyberos/strategy/CYBEROS_STRATEGY.md`
- `workbench/cyberskill-vn-skills/` → `cyberos/public-skills/`
- `/design-system/` → `cyberos/design-system/`
- `/landing-page/` → `cyberos/website/landing/`

This enables clone-and-go for new sessions and keeps strategic + technical + design content co-located.

See per-module CHANGELOG.md files for module-specific history:
- `memory/docs/CHANGELOG.md`
- `skill/docs/CHANGELOG.md`
- `cuo/docs/CHANGELOG.md`
- `design-system/CHANGELOG.md`
- `website/docs/index.html` (the rendered changelog page)

