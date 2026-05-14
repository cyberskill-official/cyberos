# CyberOS — Comprehensive Audit + Build Readiness Plan

**Date:** 2026-05-14
**Scope:** Deep UI audit · FR landscape cleanup proposal · per-module build readiness for the 19 unbuilt modules · strategic followups
**Reading order:** §1 (UI fixes) → §2 (FR cleanup, blocking) → §3 (per-module plan) → §4 (strategic followups)

---

## §1 — UI audit

Method: live-page DOM inspection + source pattern sweeps across all 32 HTML files. Live target: `https://5cc09eb6.cyberos-docs.pages.dev/` (this is the pre-Tailwind-vendor-fix deploy; redeploy from `website/docs/` to pick up the vendored CSS before re-verifying).

### §1.1 Critical (blocks shipping public)

| # | Issue | Where | Fix |
|---|---|---|---|
| C1 | Vendored Tailwind not yet on live deploy | live URL above is from before my fix; `grid`/`grid-cols-*`/`flex` resolve to `display: block` | Redeploy via `wrangler pages deploy .` from `website/docs/` |
| C2 | Mermaid `<br/>` self-close fails on Mermaid 11.4.1 with `htmlLabels: true` → labels collapse ("Cursorvia MCP tool") | 754 occurrences across **all 27 pages** with Mermaid (index + 22 modules + 4 architecture) | Bulk replace `<br/>` → `<br>` in `.mermaid` blocks only (zero outside Mermaid, verified) |
| C3 | "Syntax error in text" Mermaid render failure | At least one diagram on `modules/brain.html` (and possibly others) throws hard-error | Audit each diagram against Mermaid 11.4.1 syntax — likely culprits: `subgraph X ["title with (parens)"]`, missing class on undefined nodes, escape gaps |
| C4 | 6 broken internal links to non-existent architecture pages | 5× `architecture/services.html` (from modules/learn, hr, esop, rew, inv); 1× `architecture/runtime.html` (from modules/chat) | Either create the two pages, or redirect refs to `architecture/infrastructure.html` / `architecture/tech-stack.html` |

### §1.2 High (brand drift + polish)

| # | Issue | Where | Fix |
|---|---|---|---|
| H1 | Mermaid `classDef` still uses pastel palette on 26 non-index pages | 127 occurrences (`fill:#d1fae5`, `#dbeafe`, `#f3e8ff`, `#fef3c7`, `#fce7f3`, `#e0e7ff`, `#f1f5f9`) | Bulk sed across `.mermaid` blocks — map green→`#f5ede6`, blue→`#e8d4c2`, purple→`#f9c64f`, yellow→`#fef6e0`, pink→`#fde7b3`, indigo→`#cba88a`, neutral→`#f0eee9`. The safety-net in styles.css won't catch these because they're hardcoded in Mermaid syntax, not Tailwind utilities. |
| H2 | TOC card sits inline at all viewports instead of as a left sidebar at `lg`+ | Every page (`<details><summary>On this page` etc.) | Wrap content + TOC in a `lg:grid lg:grid-cols-[240px_1fr] lg:gap-10` shell so the TOC becomes a sticky left rail above `lg`. Vertical scroll preserves it on screen. Bytebytego-style. |
| H3 | "Active section" not highlighted in TOC during scroll | Every page | Add Intersection Observer in `assets/scripts.js` that toggles `.toc-active` on the TOC item whose section is in viewport; style: Ochre left-accent border. |
| H4 | Tab-style top buttons inconsistent: first is heavy umber/black, siblings are pill secondary | `index.html` ("What is CyberOS / All 22 modules / 12-month roadmap"), `architecture/milestones.html` ("Horizontal timeline / Dependency graph / Headcount + revenue"), `architecture/compliance.html` ("Ring 1 Vietnam / Compliance gates by phase / Breach notification matrix"), many module pages | Standardise: when the buttons are visual tabs of the same level, all of them should be `.btn--ghost` size pills. Reserve `.btn--primary` for the single primary CTA on a page (e.g. "Read full spec"). |
| H5 | Sticky nav too washed out — barely visible against the gradient body bg | Every page | Bump nav `--glass-light-bg` alpha to 0.85 (was 0.7), or add a 2px subtle Umber-tinted bottom border at all times (currently it appears only on scroll). |
| H6 | Anchor links in TOC + body are bare blue underlines | Every reference + architecture + some module pages (compliance.html visible at the screenshot) | Apply `.text-brand-link { color: var(--umber-700); text-decoration-color: var(--ochre-300); }` site-wide via styles.css. |

### §1.3 Medium (polish + accessibility)

| # | Issue | Where | Fix |
|---|---|---|---|
| M1 | No "back to top" affordance on long pages (fr-catalog, glossary, milestones, every module) | Pages > 2,000 px tall | Floating bottom-right `↑` button shown after `scroll-y > 800px`; uses `surface-light` glass |
| M2 | Code blocks lack syntax highlighting | Module pages (`<pre><code>`) | Add `highlight.js` 11.x light-mode build with `atom-one-light` (or vendor the bare ~12 KB CSS for the languages used: bash, json, sql, py, rust, ts) |
| M3 | Pagefind search widget styling doesn't pick up Umber/Ochre | Top-right of every page | Override `pagefind/pagefind-ui.css` defaults via a custom theme block in `styles.css` (Pagefind exposes `--pagefind-ui-*` CSS vars) |
| M4 | `:focus-visible` rings missing/inconsistent | Buttons, links, inputs | Single rule: `*:focus-visible { outline: 3px solid var(--ochre-500); outline-offset: 2px; border-radius: inherit; }` |
| M5 | No `<meta property="og:*">` open graph tags | All pages | Add OG title + description + image per page — affects link previews when sharing the docs URL |
| M6 | No `prefers-color-scheme: dark` styling | All pages | Defer until P1 — dark mode is a polish-tier feature; document the deferral |
| M7 | No print stylesheet polish | All pages | The strategy claims "every page printable" — confirm by hitting `?print=1` or `print:` media query. Add page-break-avoid on h2/h3, hide sticky nav + search + footer-arrows in print |
| M8 | No `<link rel="canonical">` | All pages | One-liner per page: `<link rel="canonical" href="https://docs.cyberskill.world/{path}">` — needed for SEO before public launch |
| M9 | No 404 page | Site-wide | Cloudflare Pages auto-serves a generic one; add a branded `404.html` |
| M10 | No sitemap.xml | Site-wide | Generate from page list — needed for SEO. `cd website/docs && python3 -c "..."` one-liner generator |
| M11 | Site-wide `data-pagefind-filter` declarations exist per page but no filter UI surfaces them in search | Pagefind widget | Either remove filter declarations OR expose a filter dropdown via `pagefind-ui` config |
| M12 | "On this page" TOC drops anchors that have hard-coded text but no rendered destination (e.g. milestones.html anchors `#horizontal-timeline`, `#dependency-graph`, etc. — confirm all targets exist) | Each page individually | Sweep |

### §1.4 Low (nice-to-have)

| # | Issue | Fix |
|---|---|---|
| L1 | No "edit this page" link to the source MD/HTML | Add `<a href="https://github.com/cyberskill/cyberos/edit/main/website/docs/{path}">Edit this page →</a>` in footer |
| L2 | No reading-time / difficulty marker | Skip until content reorganization |
| L3 | No live module-status badges (per Strategy Tier-1 #1) | Add `<status-badge>` web component fed by per-module JSON; show pass/fail of tests on most-recent commit |
| L4 | No public changelog page | Aggregate `*/CHANGELOG.md` into `reference/changelog.html` + `feed.xml` (Strategy Tier-1 #4) |
| L5 | No decision log (ADRs) page | New `reference/decisions.html` per Strategy Tier-1 #3 |
| L6 | No per-FR anchors (Strategy Tier-1 #5) | Add `id="FR-{MOD}-{NNN}"` to each FR card in `fr-catalog.html` — depends on FR cleanup decision (§2) |

### §1.5 What I already fixed this session (no action needed)

- All Tailwind palette utility leaks on `index.html` (0 remaining); transitional safety-net in `styles.css` neutralises the 620 leaks on other pages.
- Hero SVG triangle, compliance ring SVG, tech-stack Mermaid `classDef` — Umber/Ochre.
- Be Vietnam Pro font added to `@import` + reordered ahead of Inter in tokens.
- Mermaid `themeVariables` reordered to BVP first.
- `cyberos doctor` invariant bug (`check_manifest_validates` skipping parseability when jsonschema absent) — fixed + committed.
- Vendored `assets/tailwind.min.css` (16.7 KB) replacing the silently-failing CDN.
- Added `.ds-modpill`, `.tile`, `.pill--brand` utility classes to `styles.css`.

---

## §2 — FR landscape cleanup proposal (BLOCKING — needs your call)

You said: *"for feature requests, i want to use skill module to create one by one later, so cleanup all current ones (if exist)"*.

### §2.1 What exists today

| Location | Count | Source-of-truth? | Type |
|---|---|---|---|
| `docs/prd/PRD.md` | 303 unique FR-IDs (in body text) | yes — authoritative spec | markdown (converted from .docx) |
| `docs/prd/PRD.docx` | mirrors PRD.md | yes — original Word doc | binary |
| `docs/srs/SRS.md` | overlaps with PRD | yes — authoritative spec | markdown (converted from .docx) |
| `docs/srs/SRS.docx` | mirrors SRS.md | yes — original Word doc | binary |
| `website/docs/reference/fr-catalog.html` | 348 unique FR-IDs | no — generated/curated | HTML (1006 lines) |
| `website/docs/modules/*.html` × 22 | ~700 total FR refs (each module page has a "Functional Requirements" section averaging 40 FR cards) | no — copies of PRD content | HTML |
| Anywhere in `skill/`, `cuo/`, `memory/`, `scripts/` | **0** FR references | n/a — code doesn't reference FRs | — |

**Total**: ~5,000–8,000 lines of FR content across 27 files. Zero of it is referenced from running code (skill/, cuo/, memory/). All of it is read-only documentation.

The 45-FR gap (348 in HTML vs 303 in PRD+SRS) means **45 FRs in the catalog have no source-of-truth in PRD/SRS** — they were inferred from module pages during catalog generation. These would die in cleanup either way.

### §2.2 The three options

| Option | Effect | Reversible? | Risk |
|---|---|---|---|
| **A. Archive everything** (recommended) | Move all FR-bearing files to `docs/archive/feature-requests-pre-cleanup-2026-05-14/` (preserves PRD.md, SRS.md, fr-catalog.html, and each module page's FR section verbatim). The .docx originals stay untouched in `docs/prd/` and `docs/srs/`. Add a stub `fr-catalog.html` that says "Catalog is being rebuilt one feature at a time via the skill module — see `skill/skills/cuo/cpo/fr-author/`." Same stub treatment for each module page's "Functional Requirements" subsection. | Yes — single `mv` command back | Low. PRD/SRS narrative still tells the story; only the FR identifiers go quiet. |
| **B. Surgical strip** | Edit PRD.md, SRS.md, fr-catalog.html, and each module page to remove FR sections but keep all other content. Discard the FR text entirely. | No — git history only | Medium. Cross-references in NFR catalog + risk register break and need rewriting. Module dependency tables may reference FRs. |
| **C. Keep current FRs, author new ones additively** | Don't delete anything. Use `fr-author` skill to extend the catalog forward. | Trivial. | Low — but doesn't match what you asked for. |

**My recommendation: Option A (archive).** Reasoning:
1. PRD.md + SRS.md are *normative spec docs*. Even if you re-author each FR, the spec narrative around it (background, rationale, dependencies) stays the source of truth. Archiving preserves that for reference; the docs site can rebuild fresh.
2. The .docx originals remain untouched in `docs/prd/` and `docs/srs/`. No legal/audit risk.
3. NFR catalog + risk register currently cross-link to FRs. If you strip FRs without rewriting those, they 404 — archiving lets the rewrites happen incrementally as you re-author each FR.
4. Reversible via one `git mv` if the workflow doesn't pan out.

### §2.3 What an Option-A archive looks like concretely

```
docs/
├── prd/
│   ├── PRD.md             ← stays (narrative is still authoritative)
│   ├── PRD.docx           ← stays
│   ├── CHANGELOG.md
│   └── README.md
├── srs/
│   ├── SRS.md             ← stays
│   ├── SRS.docx           ← stays
│   ├── CHANGELOG.md
│   └── README.md
└── archive/
    └── feature-requests-pre-cleanup-2026-05-14/
        ├── README.md                          ← "Archived 2026-05-14. Use skill/skills/cuo/cpo/fr-author/ going forward."
        ├── fr-catalog.html                    ← copy of the pre-cleanup HTML catalog (348 FRs)
        ├── modules-fr-sections/               ← extracted FR sections from each of 22 module pages
        │   ├── auth-frs.html (or .md)
        │   ├── brain-frs.html
        │   └── ... 22 files ...
        └── prd-srs-fr-only.md                 ← greppable list of just the FR text from PRD+SRS (informational)

website/docs/
├── reference/
│   └── fr-catalog.html       ← REPLACED with stub: "Catalog is being rebuilt..."
└── modules/
    └── *.html × 22           ← "Functional Requirements" section in each REPLACED with a one-line stub linking to the archive + the fr-author skill
```

### §2.4 What happens after cleanup

- Run `cd /Users/stephencheng/Projects/CyberSkill/cyberos/skill && cargo run -p cyberos-skill-cli -- run fr-author --executor script` (per the `skill/skills/cuo/cpo/fr-author/SKILL.md` spec)
- For each authored FR, the skill emits a `feature_request@1` artefact → a new entry into the fresh `fr-catalog.html` and a new line into the relevant module page's FR section
- BRAIN records the auth chain entry per the skill module's `allowed_brain_scopes`

If you want, I can also pre-build an `fr-author` workflow harness that drives the catalog regeneration end-to-end.

### §2.5 Confirmation gate

This is destructive enough that I won't act without your explicit say-so. Pick one of:
- "archive" (Option A — recommended)
- "strip" (Option B — destructive)
- "keep" (Option C — no cleanup)
- "different" (you want a different shape — tell me)

---

## §3 — Per-module build readiness (the 19 unbuilt modules)

Below: for each unbuilt module, spec depth, dependencies, position on the critical path, slice-1 outline. The strategy doc orders work by "internal productivity is the moat" — that ordering holds. The table here is the *engineering* sequence (what unlocks the most downstream work), which sometimes differs.

### §3.1 The dependency truth (one diagram you should hold in your head)

```
                     ┌────────────────────────────────────────┐
                     │                                        │
   ┌── BRAIN ────────┴─── SKILL ─── CUO ────── (all P1+ modules)
   │   (shipped)        (shipped)  (Phase 1)
   │                                  │
   │                                  ▼
   └── AUTH ──► AI Gateway ──► MCP Gateway ──► OBS
       (P0)     (P0)            (P0)            (P0)
        │         │               │              │
        │         │               │              ▼
        │         │               │     (every module pipes logs/metrics here)
        │         │               │
        │         │               ▼
        │         │     (CHAT / KB / docs-search / MCP tools all flow through)
        │         │
        │         ▼
        │  (every LLM call from CUO + every module)
        ▼
   (every cross-module call needs a verified subject; every audit row needs an actor)
```

**Net consequence:** AUTH is the single highest-leverage P0 module. Until it lands, every other module ships with mock auth — fine for dogfooding, blocking for tenant launch. The other 5 P0 cross-cutting modules (AI Gateway, MCP Gateway, OBS, CHAT, plus the GraphQL Federation router which is more infra than module) all depend on AUTH for tenant scoping.

### §3.2 The 19 modules, ranked by build leverage

| Rank | Module | Tier | Spec depth (today) | Open Qs blocking slice 1 | Critical-path role | Suggested slice 1 | Comparison anchor |
|---|---|---|---|---|---|---|---|
| 1 | **AUTH** | P0 | RFC drafted (`services/auth/RFC.md`), 5 slices, 7,000 LoC est | 5 open Qs in RFC §6 (workspace, memory-bridge timing, tenant-0 bootstrap, HIBP default, OBS deferral) | Keystone. Unblocks every module's tenant-aware call. | Tenant + subject CRUD with RLS (week 1, ~15 tests) | Auth0 / Clerk / WorkOS |
| 2 | **AI Gateway** | P0 | Spec page exists; LiteLLM-based; ~3,000 LoC est | Multi-provider key routing? Per-tenant budget caps? Rate-limit storage (Redis vs Postgres)? | Every CUO/skill LLM call traverses this. Mock today = raw provider keys in env vars. | LiteLLM proxy + tenant-scoped key vault + Postgres budget tracker | LiteLLM gateway / Helicone |
| 3 | **MCP Gateway** | P0 | Spec page exists; 2025-11-25 MCP spec compliance | Should this be one process per tenant or one shared with tenant_id in protocol? Tool-allowlist storage? | Every external MCP tool call traverses this. Mock today = direct MCP server connections. | MCP transport multiplexer + tool-allowlist resolver + audit-row emitter | n/a (we are pioneering this) |
| 4 | **OBS** | P0 | Spec page exists; OpenTelemetry SDK + LiteLLM logs + audit chain | Where do traces land? (Honeycomb / Datadog / self-host? Probably Grafana Tempo + Loki for cost.) | Every module emits logs + metrics here. Mock today = stdout. | OTLP receiver + Loki sink + Tempo sink + Grafana dashboards | Honeycomb / Datadog / Grafana |
| 5 | **CHAT** | P0 | Spec page exists; internal chat replacing Slack/Zalo for the CyberSkill team | Real-time (NATS+WebSockets vs Matrix vs Phoenix-Channels)? Persistence schema? Encryption (E2EE or not)? @genie integration? | Dogfooding gate — at P0 exit, Slack + Zalo are decommissioned. Without CHAT, you can't decommission them. | Channel + thread + message + reaction + presence over NATS; @genie callback to CUO | Slack / Matrix / Discord |
| 6 | **EMAIL** | P1 | Spec page exists; Mail + Inbox | IMAP/SMTP vs SES/Postmark API? Threading model? Bayesian or LLM classification? | Replaces Gmail for CyberSkill internal use; sales-CRM funnel ingestion. | Per-tenant mailbox + IMAP poll + thread reconstruction + @inbox CUO integration | Front / Missive / Superhuman |
| 7 | **PROJ** | P1 | Spec page exists; Linear analogue | Issue lifecycle states? OKR cascade integration? PR-bot wiring? | Replaces Jira/Linear for CyberSkill internal use. | Project + issue + cycle + assignee + state-machine + GitHub-webhook bridge | Linear / Plane / Jira |
| 8 | **TIME** | P1 | Spec page exists; Time + Expense | Approval flow? Currency handling? Vietnamese tax-receipt OCR? | Pre-payroll dependency for REW; expense compliance | Time entry + approval state-machine + per-project rollup + VN-receipt OCR via skill | Harvest / Toggl / Tempo |
| 9 | **CRM** | P1 | Spec page exists; Clients | Pipeline stages closed catalog vs free-form? Lead-scoring model? CUO-CMO persona integration? | Sales-side of the demand-gen plays the strategy doc mentions | Account + contact + deal + stage + activity-log + @genie call-prep | HubSpot / Close / Pipedrive |
| 10 | **KB** | P1 | Spec page exists; Knowledge | Markdown + page hierarchy vs wiki-style backlinks? BRAIN ingestion path? Search vs Pagefind reuse? | Internal docs — replaces Notion for CyberSkill internal use. Becomes the substrate other modules link into. | Page + space + version + backlink graph + BGE-M3 search + @genie Q&A | Notion / Confluence / Outline |
| 11 | **HR** | P1 | Spec page exists; Human Resources | Vietnamese BHXH/BHYT compliance handlers? Org-chart data shape? Performance review cycle integration with REW + LEARN? | Pre-payroll dependency. PII storage governance. | Employee record + org-chart + onboarding/offboarding state-machine + leave-balance + VN-compliance skill bridge | BambooHR / Personio / Rippling |
| 12 | **REW** | P1 | Spec page exists; Total Rewards (payroll + bonus + equity prep) | Pool calculation rubric? Phantom-stock vs real ESOP at P2? Tax-withholding handler? Vietnamese-IRS API? | Payroll = the P1 exit gate ("First payroll cycle through REW") | Salary table + bonus pool calc + tax-withholding + payslip generator + VN-tax-API skill bridge | Gusto / Justworks / Deel |
| 13 | **LEARN** | P1 | Spec page exists; Learning + Promotion | Hội đồng Chuyên môn institutional review format? Promotion-readiness rubric? Performance-review integration? | Career-track motivator; ties HR + REW; LEARN-stake measurement informs ESOP at P2 | Skill-map per role + course catalogue + assessment + Hội đồng Chuyên môn workflow | Lattice / 15Five / Workday |
| 14 | **INV** | P2 | Spec page exists; Invoicing (Vietnamese e-invoice = MST + VAT + GDT submission) | Tax-software API ownership: in-house or partner? Per-line tax-rate calculation? Cross-currency? | Bill-to-cash. ARR recognition. Vietnamese GDT submission via existing `vn-tax-filing` skill. | Invoice + line-item + customer + GDT-submission state-machine + payment-reconciliation | Stripe Billing / Chargebee / Xero |
| 15 | **ESOP** | P2 | Spec page exists; Phantom Stock | Grant math (cliff, vest, accelerate, ratchet)? Strike price vs phantom unit pricing? Tax-event handler? | First SP grant issued = P2 exit gate; key talent-retention lever | Plan + grant + vesting-schedule + tax-event ledger + audit-chain anchor | Carta / Pulley / Capdesk |
| 16 | **RES** | P3 | Spec page exists; Resource Plan | Forecasting model (rules vs LLM)? Project-to-people assignment rubric? Capacity-vs-utilisation thresholds? | First quarterly OKR cycle depends on this | Capacity model + assignment + utilization + forecasting + OKR cascade | Float / Resource Guru / Forecast |
| 17 | **OKR** | P3 | Spec page exists; OKR + Strategy | Cascade depth? KR-vs-metric automation? CEO/COO persona integration for reviews? | Strategy → execution loop closer. P3 exit gate. | Objective + key-result + cycle + cascade-walk + persona-review skill bridge | Lattice / Ally / Workboard |
| 18 | **DOC** | P4 | Spec page exists; Document Signing (eIDAS QTSP) | eIDAS QTSP partner (Adobe Approved Trust List? VN-AATL?) Signature payload format? Audit-chain anchoring? | External GA gate (P4) | Document + signature + audit-chain anchor + QTSP integration | DocuSign / Adobe Sign / SignWell |
| 19 | **PORTAL** | P4 | Spec page exists; Client Portal (external-facing surface) | Per-tenant subdomain or path? Brand-customisation depth? Auth mode (passwordless? magic-link?) | First external paying tenant gate | Tenant subdomain + branded shell + auth gate + module embed adapter | Notion guest sites / Slite portals |
| 20 | **TEN** | P4 | Spec page exists; Tenancy + Billing | Subscription plan model? Per-module pricing? Self-serve vs sales-led signup? | Final P4 module — enables the "First paying tenant" exit | Tenant lifecycle state-machine + plan catalogue + Stripe-billing bridge + onboarding wizard | Stripe / Chargebee + Auth0 |

### §3.3 Recommended build sequence

**Phase A (now → +6 weeks):**
1. **AUTH slice 1** (Cargo crate + tenant + subject CRUD) — *unblocks everything*
2. **AUTH slice 2** (password + session + JWT)
3. **OBS slice 1** (OTLP receiver + Loki + Tempo) — *because every subsequent slice will be invisible without traces*
4. **AUTH slice 3** (WebAuthn + TOTP)

**Phase B (+7 → +12 weeks):**
5. **AI Gateway slice 1** (LiteLLM proxy + per-tenant key vault)
6. **MCP Gateway slice 1** (multiplexer + tool-allowlist)
7. **AUTH slice 4** (RBAC + Scope Contract + audit-chain bridge)
8. **CHAT slice 1** (channels + messages + @genie callback)
9. **AUTH slice 5** (KMS + impossible-travel + device + OIDC + MCP)

**Phase C (+13 → +18 weeks):**
10. **KB slice 1** (page + space + version + BGE-M3 search) — *cyberskill.world internal docs land here*
11. **PROJ slice 1** (project + issue + cycle + state-machine) — *replaces our Linear-in-spirit workflow*
12. **CHAT slice 2** (file uploads + reactions + presence)

**Phase D (+19 → +26 weeks):**
13. **HR slice 1** (employee record + org-chart + onboarding)
14. **TIME slice 1** (entry + approval + rollup)
15. **REW slice 1** (salary table + payslip) — *P1 exit gate*

After phase D you hit the strategy doc's `M+6 first payroll through REW` milestone. Phases E–H cover the remaining P1–P4 modules at roughly the same cadence (~2 slices / 2 weeks per module).

### §3.4 What every unbuilt module's slice 1 needs from you

A short RFC like `services/auth/RFC.md` *before* code lands. Template:

1. **Decision summary** — language, framework, DB, deps (use AUTH RFC as a template).
2. **Module layout** — `services/<mod>/Cargo.toml` + `src/` skeleton + `migrations/`.
3. **5-slice ship plan** — each slice mergeable in 1 week, ~5–8 PRs total.
4. **Audit-chain integration** — every state-changing op MUST land on BRAIN.
5. **5 open Qs** — explicit decisions you need to lock before slice 1.
6. **DoD** — test count, conformance suite, BRAIN-bridge integration test.

I can draft these for the next 5–10 modules on request (~30 min each).

---

## §4 — Strategic followups (do these in parallel)

### §4.1 Design-system additions (you asked for these in the prior audit)

1. **Mermaid theming recipe in `design-system/DESIGN.md` Part 21** — new sub-section `§21.x — Theming third-party renderers` with the Mermaid `themeVariables` JSON we already wrote. Prevents the next docs author from re-inventing it.
2. **Component spec for `.tile` + `.pill--brand` + `.ds-modpill` + `.btn--*`** in Part 3 — promote from `assets/styles.css` (project-local) to `design-system/DESIGN.md` (cross-product canonical).
3. **`design-system-lint` tool** per Part 15 — Python/TS script that scans HTML/JSX and flags: Tailwind palette utilities (`bg-blue-*` etc.), off-anchor `fill:#` hexes, missing `surface-*` class on top-level `<article>/<section>`, code blocks missing `var(--font-mono)`. Wire to pre-commit + Cloudflare Pages build.

### §4.2 Docs site Tier-1 (Strategy §3 Tier 1)

Already done:
- ✅ Pagefind site-wide search (was claimed "to-do" — actually shipped earlier)

Pending:
- **#1 Live module-status dashboard** — per-module status JSON + badge component
- **#3 Decision log / ADRs page** — `reference/decisions.html`
- **#4 Public changelog + RSS** — aggregate per-module CHANGELOG.md → `reference/changelog.html` + `feed.xml`
- **#5 Per-FR anchors** — depends on FR cleanup decision (§2)

### §4.3 Docs site Tier-2 (Strategy §3 Tier 2)

- **Comparison matrices** — "CyberOS PROJ vs Linear", "CHAT vs Slack", "KB vs Notion" — *highest demand-gen leverage*; Stephen flagged in Strategy §5 Session 3
- **Migration guides** — "From Slack to CHAT" / "From Notion to KB" / "From Jira to PROJ" — includes import-tool spec
- **Pricing calculator** — interactive cost model
- **Interactive dependency graph** — D3 force-directed graph of 22 modules

### §4.4 Repo plumbing

- **Cloudflare Pages auto-deploy on push to `main`** — DEPLOYMENT.md describes the wiring; not yet done.
- **GitHub Actions CI** — `cargo test`, `pytest`, `pagefind --site`, deploy preview.
- **`.gitignore` update** — `website/docs/.wrangler/` (already-untracked scratchpad)
- **CONTRIBUTING.md** — refresh: where to add modules, FR-author workflow, design-system-lint usage.

---

## §5 — Recommended next-session action queue

1. **Decide on §2 (FR cleanup)** — pick A/B/C. Blocks everything downstream.
2. **Redeploy `website/docs/`** — pick up vendored Tailwind + brand fixes. Verify on a fresh URL.
3. **Pick a Mermaid-fix scope** — bulk `<br/>`→`<br>` is mechanical (15 min). Brand-aligning `classDef` palettes across 127 instances is also mechanical (30 min).
4. **Answer the 5 AUTH RFC open questions** so slice 1 can land.
5. **Tell me whether to draft RFCs for the next 5 modules** (AI Gateway, MCP Gateway, OBS, CHAT, KB) — I can produce these to the same quality as the AUTH RFC.
6. **(Optional) commit the work-in-progress** I've left in the worktree:
   - `services/auth/` (RFC + mockup)
   - `website/docs/assets/tailwind.min.css` (new)
   - 32 HTML files (CDN script → vendored link)
   - `website/docs/index.html` (brand rebuild)
   - `assets/styles.css`, `assets/tokens.css`, `assets/scripts.js` (BVP + utilities + Mermaid font)
   - `CHANGELOG.md` (entries)
   - `docs/AUDIT_AND_PLAN_2026_05_14.md` (this file)

---

*End of audit + plan.* Single source of truth for the next 2 weeks of work.
