# Changelog — CyberOS

All notable changes to the umbrella CyberOS repository, newest-first.

## 2026-05-14 — Comprehensive audit + FR catalog strip + Mermaid mass-fix

Added `docs/AUDIT_AND_PLAN_2026_05_14.md` — single comprehensive audit + build-readiness plan covering UI glitches (severity-ranked), FR landscape, per-module build sequence for the 19 unbuilt modules with slice-1 outlines, and strategic followups. Designed as the source of truth for the next 2 weeks of work.

**FR catalog strip (per user decision: strip-everything).** Stripped:
- All 22 module pages: each "Functional Requirements" section (the `<section id="functional-requirements">` block, lines ~789–820 across modules) replaced with a stub linking to the `fr-author` Agent Skill workflow. 23/23 pages patched cleanly via regex sweep.
- `website/docs/reference/fr-catalog.html`: 1006-line generated catalog replaced with a 70-line stub explaining the rebuild + how to author new FRs via the skill module.

**Partially stripped (cross-refs remain — call to extend):**
- `website/docs/reference/nfr-catalog.html` — still has 137 FR refs (NFRs are described in terms of which FRs they constrain)
- `website/docs/reference/risk-register.html` — still has 51 FR refs (risks reference the FRs they affect)
- Module pages — still have inline FR refs in Dependencies tables, NFR descriptions, KPIs, References footers (~200 total across all)
- `docs/prd/PRD.md` (393 FR refs) and `docs/srs/SRS.md` (206 FR refs) — preserved as authoritative spec narrative; .docx originals also preserved

The "strip-everything" decision affects ~434 remaining FR cross-references — these are inline within sentences and tables. They become broken references until re-authored. To clean them up, separate decisions are needed on whether to: keep them as broken refs (will rewrite organically as new FRs come online), replace with `(FR pending)` markers, or remove the surrounding sentences entirely.

**Mermaid mass-fix across 28 pages:**
- `<br/>` → `<br>` — 754 instances replaced, ALL inside `<div class="mermaid">` blocks (zero outside, verified). This fixes the "Cursorvia MCP tool" text-collapse bug seen on `modules/brain.html` where Mermaid 11.4.1 strips self-closed `<br/>` tags inside quoted node labels.
- Pastel `classDef` palette → Umber/Ochre brand: 127 instances recolored across all non-index module + architecture pages. Map: emerald-100→umber-50, blue-100→umber-100, purple-100→ochre-300, amber-100→ochre-50, pink-100→ochre-100, indigo-100→umber-200, slate-100→neutral-100, yellow-100→ochre-50, violet-100→ochre-50. Strokes likewise mapped to umber-500 / ochre-700 / neutral-400.
- 6 broken internal links to non-existent architecture pages fixed: `architecture/services.html` (5 refs from learn/hr/esop/rew/inv) and `architecture/runtime.html` (1 ref from chat) redirect to `architecture/infrastructure.html` (the closest topical match).

Net code change: 36 files, ~1,417 insertions / ~2,641 deletions. Plus new files `docs/AUDIT_AND_PLAN_2026_05_14.md` (the master plan) and `website/docs/assets/tailwind.min.css` (16.7 KB vendored from prior commit).

Open items pending Stephen's call (per audit doc):
1. Whether to strip the remaining 434 inline FR cross-refs (in NFR catalog / risk register / module sub-sections) or let them rewrite organically.
2. AUTH RFC's 5 open questions need answers before slice 1 codes.
3. Redeploy `website/docs/` via wrangler so the brand + Tailwind + Mermaid + strip fixes go live.

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

## 2026-05-14 — AUTH module RFC + sign-in mockup

- Added `services/auth/RFC.md` — implementation RFC with 5-slice ship plan, audit-chain integration design, and 5 open questions blocking slice 1.
- Added `services/auth/mockups/sign-in.html` — first AUTH UI mockup applying design-system Part 21 Liquid Glass defaults, Umber + Ochre anchors, Be Vietnam Pro first, passkey-first flow with password fallback, MFA chips, BRAIN audit-chain trust footnote.
- Verification pass against shipped modules:
  - memory: 222 tests pass + 1 skip (numpy + jsonschema needed for full green). Real bug found AND fixed: `check_manifest_validates` was skipping parseability when jsonschema absent → `cyberos state` returned READY on a broken manifest. Patched to always parse `manifest.json` first (regardless of jsonschema availability) and report `False` on `JSONDecodeError`; the optional schema-validation layer still skips cleanly when jsonschema is absent. Verified: all 4 `tests/test_state.py` tests pass, full suite 238 pass / 1 skip / 0 fail. Also verified by simulating absent jsonschema via import hook — good manifest still returns True with "parseability OK, schema skip"; bad manifest returns False with "manifest.json unparseable: ...".
  - skill: 20 SKILL.md bundles structurally verified, 4 crates, 8 inline Rust tests. `cargo build` not run (sandbox-only limitation).
  - cuo: 15/15 pytest + 15/15 routing fixtures pass. Catalog discovers all 20 skills correctly.
- Stale-claim drift surfaced (none are blockers, all are doc-only):
  - Memory tests: bootstrap says 245, README says 255, actual is 238 collected.
  - Doctor invariants: bootstrap says 16, README says 15, actual is 13 on a fresh store.
  - Docs pages: bootstrap says 32, strategy says 31, actual is 33 HTML files (32 user-facing + nav include).
  - Strategy §3 Tier-1 #2 and §5 Session-1 #1 list "wire Pagefind" as a to-do; Pagefind is already built and serving (v1.5.2, 32 pages indexed).
  - DEPLOYMENT.md is at `website/docs/DEPLOYMENT.md` (bootstrap implies it lives at `website/`).
- Docs site deploy-prep findings:
  - 6 real broken internal links to 2 missing architecture pages: `architecture/services.html` (5 refs from LEARN/HR/INV/ESOP/REW) and `architecture/runtime.html` (1 ref from CHAT). These are demand-gen blockers — fix before public deploy or convert the link targets.

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
