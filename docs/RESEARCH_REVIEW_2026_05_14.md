# CyberOS Pre-Launch Audit

*Senior product-and-engineering review · v2026.05 docs build · 14 May 2026*

This is an opinionated, founder-facing audit of CyberOS — the 22-module AI-native internal-operations platform that CyberSkill JSC (10 Members, Ho Chi Minh City, founded 2020) plans to build over the next 24 months. Three modules ship today (BRAIN, Skill, CUO); 19 modules are designed but unbuilt. The strategy document, build-readiness audit, FR-authoring workflow, the AUTH RFC, and four module READMEs have been read; the documentation site at `https://aed45706.cyberos-docs.pages.dev` was crawled end-to-end (index, all 23 module pages, four architecture pages, four reference pages). What follows is what an engineering-led Series-A reviewer would say if the founder asked, "Tell me what's broken before I lock this." It is direct, it does not hedge, and where it flags a defect it proposes a concrete fix.

## 1. Strategic coherence

### 1.1 Does the "ecosystem-as-a-service" thesis hold up?

The thesis — Internal → OSS → Hosted SaaS → Marketplace → Vertical Packs → Ecosystem-as-a-Service — is *internally* coherent and unusually clear-eyed for a 10-person team. Three things make it work intellectually: (a) every level is downstream of P0 infrastructure that has to exist anyway (AUTH, AI Gateway, MCP Gateway, OBS, GraphQL Federation, NATS); (b) the BRAIN substrate produces a network effect *inside* a tenant (every module's audit + memory compounds in one ledger), not just across tenants; (c) the Agent Skills format gives CyberOS a credible OSS distribution surface that does not require Anthropic's permission. The compliance ladder (`/architecture/compliance.html#phase-gates`) explicitly maps each productization level to the customer cohort it unlocks — that is the right way to argue the thesis to an investor.

**Where the thesis is most fragile** is the leap from level 4 (Marketplace) to levels 5–6 (Vertical Packs and Ecosystem-as-a-Service). A 20-Member team at M+24 with 10 paying tenants and ARR ≈ $3M (per `/architecture/milestones.html#trajectory`) does not yet have the surface area to credibly run a developer marketplace. Salesforce AppExchange has roughly 5,000+ apps and reached critical mass with thousands of ISVs over fifteen years; Atlassian Marketplace similarly. The realistic path is: ship levels 1–3 with discipline, *announce* the marketplace at level 4 but treat it as a recruiting and PR artifact for two years, and only invest seriously when paying tenants exceed ~50. The docs are silent on what minimum tenant count justifies marketplace investment — fill that gap.

### 1.2 Sequencing of the five/six productization levels

| Level | What it is | Sequencing verdict |
|---|---|---|
| 1 Internal | Dogfood with 10 Members | Correct as starting point; P0–P1 |
| 2 OSS | Skill catalog + memory protocol public | Correct at P0 exit — Skill ships under Agent Skills format anyway |
| 3 Hosted SaaS | TEN + PORTAL multi-tenant | **Too late at P4 (M+18).** Should split into a "single-tenant managed" offering at P2 (M+9) and full multi-tenant at P4 |
| 4 Marketplace | 3rd-party skills + module remotes | Defer to P4+24; do not invest before 50 paying tenants |
| 5 Vertical Packs | VN-tax + agency + ESOP bundles | Land at P3 — this is the actual moat |
| 6 Ecosystem-as-a-Service | Federated identity + memory layer for others | Aspirational; do not commit to a date |

The single biggest sequencing error is that **Hosted SaaS is gated behind TEN at P4**, which means a design partner who wants to pay you in P2 has no legal vehicle to do so. A pragmatic fix: ship a "managed single-tenant" SKU at P2 — same code base, dedicated VPC per customer, manual billing — to unlock $300k–$1.5M of design-partner ARR a full year before TEN lands. This is not multi-tenancy; it is "we operate the box for you." Linear, Notion, and Carta all sold managed single-tenant before they sold multi-tenant; the precedent is solid.

### 1.3 Are the 3 / 6 / 9 / 12 / 18 / 24-month markers realistic for 10 → 20 Members?

The phase exit gates in `/architecture/milestones.html#timeline` are aggressive but defensible if — and only if — three things happen. First, the P0 infrastructure plane (six pillars in `/architecture/infrastructure.html#overview`) is *not* re-scoped during build. The plan to ship AUTH, AI Gateway, MCP Gateway, OBS, GraphQL Federation, and NATS in 90 days with a team that also has to keep BRAIN, Skill, and CUO healthy is a 60-percent-confidence bet at best. Second, the P1 module batch (PROJ, TIME, CRM, KB, HR, EMAIL, REW, LEARN) at 8 modules in 90 days is *unrealistic* with 12 Members — that is roughly 1 module per Member per quarter while also dogfooding and doing compliance prep. Third, the SOC 2 Type I at P1 exit (M+6) is plausible only if the SOC 2 prep work starts at M0, not at M+4.

**What is missing from the markers:** (a) no leading indicator for tenant-acquisition velocity (the plan jumps from "internal" to "10 paying tenants by M+24" with no intermediate gate); (b) no explicit *kill criteria* for any module — every plan that adds modules monotonically becomes a death march, and CyberOS has no language for "this module is descoped to P3 because of P1 reality"; (c) no headcount-elasticity contingency for what happens if hires arrive 3 months late (the plan to grow from 10 → 12 in M+6 is fragile — Vietnamese senior engineering hires routinely take 4–6 months).

**Concrete fix:** add a single tenant-acquisition KPI to each phase from P2 forward — `design_partners_committed_count ≥ N` — and add a "P1 was overspec'd, descope which two modules?" gate at M+4. The candidates to defer are HR-full (split into HR-roster at P1 and HR-full at P2) and LEARN (defer entirely to P2 since promotion review needs at least one quarter of TIME data anyway).

### 1.4 Does the Vietnamese-market wedge compound or trap?

The wedge — six VN-localized skills (`MST`, `CCCD`, `VietQR`, GDT VAT e-invoice, tax filing, bank transfer) plus PGroonga Vietnamese tokenisation in CHAT — is a defensible local moat, not a trap, *provided* three architectural choices hold. First, the Skill catalog format is Anthropic Agent Skills (open). Second, the language layer is i18n-clean throughout (Be Vietnam Pro font, RFC 6532 UTF-8 throughout EMAIL, NFC normalisation in CUO router per `/modules/cuo.html#what`). Third, the residency model supports per-tenant pinning to `sg-1`, `vn-hanoi-1`, `eu-fra-1` (described at `/modules/ten.html#what`).

What makes the wedge *compound* internationally is the HCMC → HN → SG → ID → TH → PH sequence — Singapore (SG) is the global on-ramp, not the second Vietnamese city. The Singapore HoldCo flip at P3 is the actual mechanism by which a Vietnamese-built product becomes a globally-sellable SaaS. The PDPA + USD billing + ESOP-tax-friendly HoldCo combination is well-rehearsed (Trax, Patsnap, Carousell all used a version of this).

The risk is *political*, not technical: a hostile shift in Vietnamese cross-border data rules — the kind of event that follows from a Decree 53 in-scope determination — could force a precipitous HoldCo flip while bringing 10 of 10 Members along on visa paperwork. That risk is not in the risk register; it should be (call it `RSK-EXT-09 — VN export-control on data shifts during P2`).

## 2. Architecture & module boundaries

### 2.1 Is the three-layer BRAIN sensible, and where does it break at scale?

The shipped Layer 1 (filesystem ledger at `.cyberos-memory/` with MMR + Ed25519-signed tree heads + deterministic export) is genuinely impressive engineering for a 10-Member team — 255 green tests, 15/15 `cyberos doctor` invariants, byte-identical exports across machines (`/modules/brain.html#hero`). The Layer 1 design is correct: append-only, local-first, cryptographically provenant, GDPR-purgeable. This is *not* a typical agent-memory bolt-on; it is a legitimate substrate.

**Where it breaks at scale** is the transition from Layer 1 to Layer 2 (pgvector + Apache AGE) at P1/P2. The docs at `/modules/brain.html#architecture` describe Layer 2 as "planned" but do not yet specify: (a) how the canonical writer arbitrates between filesystem-truth and pgvector-truth when a tenant goes multi-laptop; (b) what the consistency model is between Layer 1 and Layer 2 (read-your-writes? eventual? bounded staleness?); (c) how the Merkle chain extends into Layer 2 — pgvector does not have built-in append-only semantics. The classic failure mode here is that Layer 2 becomes a "view materialisation" that drifts under load and produces audit gaps. **Fix:** before P1, write a one-pager that says explicitly "Layer 1 is the source of truth; Layer 2 is a derived index; on conflict Layer 1 wins; the Merkle chain is anchored at Layer 1 only; Layer 2 rebuild from Layer 1 is a tested CI job."

The Layer 3 archival corpus on S3/R2/MinIO is fine in concept; the audit-chain anchoring is well-thought-out. The cost risk is not Layer 3 storage — it is **Layer 2 vector index size**. At 50 tenants × 1M chunks × 1024-dim BGE-M3 embeddings, the HNSW index is approximately 200GB of memory-resident state. The NFR catalog target of "BRAIN search p95 ≤ 250ms on 1M chunks" (`/reference/nfr-catalog.html`) is plausible but the cost-ceiling target of $2.2k/month at 50 tenants (per same catalog) is incompatible with that memory footprint on managed RDS. **Fix:** plan for a self-hosted pgvector on dedicated VMs (or Qdrant) by P3, not RDS.

### 2.2 Single-Genie + 10 C-level skills vs multi-persona — architectural verdict

The single-persona Genie + hot-loaded C-level sub-skills design is *architecturally cleaner* than the multi-persona alternative for three reasons documented at `/modules/cuo.html#why` and `/modules/cuo.html#what`. (a) One persona-version stamp per output makes the EU AI Act Art. 50 transparency obligation a UI affordance, not a per-persona policy problem. (b) The Phase 1 router is deterministic and sub-millisecond, which means "which C-level answered" is *replayable from disk* — a hard requirement for ISO 42001 AIMS. (c) The user-facing surface is one mental model ("ask Genie"), which reduces the cognitive cost of adoption for non-technical staff.

**Where complexity leaks in P1+** is exactly the place CyberOS already worries about: the Phase 2 LLM cascade (escalate when 0.10 ≤ score ≤ 0.50) and Phase 3 multi-step chains. The cascade is fine in principle but the cost-and-audit trade is non-trivial — when Phase 2 routes through an LLM, the choice itself becomes non-deterministic and must be replay-stable through the AI Gateway's persona-version stamp + prompt cache. The CUO docs already declare this (`/modules/cuo.html#what`, row "1H · How"); make sure the AI Gateway implementation actually enforces it before Phase 2 ships.

The most likely failure mode of single-Genie is **persona confusion at the boundary**: when "the CFO skill" and "the CHRO skill" both have a plausible claim on a query like *"what is fair compensation for Mai's promotion?"*, the router has to pick one — and the wrong pick can leak compensation data through narrative even though REW's BRAIN-exclusion (DEC-036) keeps numbers out. **Fix:** add a `cuo.boundary_test` doctor invariant that, given a corpus of cross-boundary queries, asserts the chosen skill respects each module's data classification. Wire this into CI.

### 2.3 22-module decomposition — merges, splits, reorderings

The 22-module list is mostly defensible. The right moves to make before the founder locks the strategy:

| Module | Verdict | Reason |
|---|---|---|
| AUTH + MCP Gateway | **Keep separate** | AUTH owns identity; MCP Gateway owns tool federation. Different on-call rotations. |
| AI Gateway + OBS | **Keep separate** | Cost-tracking and observability have different consumers (CFO vs CTO skills). |
| HR + REW + LEARN | **Split HR into HR-roster (P1) and HR-comp-adjacent (P2)** | Currently HR is monolithic and absorbs both onboarding and SI/PIT-adjacent fields. The latter belongs in REW's blast radius. |
| PROJ + TIME | **Consider merging** | TIME is a thin module (~3 entities). It's a feature of PROJ for any non-agency tenant. Two-module split costs one engineer-quarter. |
| RES + OKR | **Defer OKR to P4** | OKR is plumbing for a process the team doesn't yet practice. RES is operationally necessary for capacity planning. |
| ESOP | **Promote to P1 stretch** | Phantom-stock vesting is the *single* feature that retains senior VN engineering hires; delaying to P2 is a retention risk. |
| DOC | **Defer eIDAS to P4+24** | eIDAS QTSP integration is a 6-month legal+technical project; ship "advanced e-signature" at P4 entry, defer QTSP. |
| PORTAL | **Keep but rescope** | Plan for "branded read-only PORTAL" at P3 (3 months), full client-initiated workflows at P4+18. |

The biggest unforced error in the current decomposition is treating **TEN as a P4-long-term module**. Multi-tenancy invariants (RLS, NATS subject isolation, S3 prefix scoping) are present from day one per DEC-058 (tenant-as-degenerate-tenant), but the *billing surface* — Stripe + VietQR + Momo — does not need to wait until M+18. Ship a thin TEN-billing slice at P2 so design-partner contracts can be invoiced through the platform, not Stripe-dashboard-direct.

### 2.4 Does AUTH really belong as P0 #1?

No. **BRAIN is already P0 #1 (shipped). The right P0 #1 of the unbuilt six is AI Gateway, not AUTH.** Here is why.

AUTH is a 7,000-LoC project with WebAuthn L3, OAuth 2.1 PRM, per-tenant authz server, JWKS rotation, RBAC predicate engine (`/modules/auth.html#hero`). It is critical but not the bottleneck — for the first 6 months CyberOS has 10 Members and one tenant, and AUTH can ship at month 2 with magic-link + TOTP and a stub RBAC. WebAuthn passkeys for the Founder and a per-tenant authz server can land at M+3 without blocking any other module.

The AI Gateway, by contrast, is the *cost-of-everything-else* gate. CHAT's `@genie` mention, CUO's Phase 2 cascade, BRAIN's semantic search via BGE-M3 embeddings, every module's narrative surface — all of them require the gateway. Without LiteLLM + Presidio redaction + cost ledger + Bedrock failover live, every other module either embeds its own SDK (the anti-pattern the gateway exists to prevent) or stubs the AI call (which prevents dogfooding from generating useful signal). **Fix:** explicitly reorder the P0 sequence so the AI Gateway ships at M+1, AUTH at M+2, OBS instrumentation in parallel, MCP Gateway at M+2.5, CHAT and CUO Phase 1 at M+3.

The other defensible reordering is to bring **OBS forward to M0 alongside BRAIN** — running CyberOS in production for 90 days without LGTM (Loki / Grafana / Tempo / Mimir) instrumented means the team is blind to its own dogfooding signal. Grafana Cloud's free tier covers this in P0 (50 GB logs, 50 GB traces, 10k metrics series); cost is zero, friction is one config file.

## 3. Spec quality (per-module pages)

Per the brief, each module page should contain the 5W1H2C5M framework, architecture diagram, data model ERD, API surface (GraphQL/MCP/CLI), key flows, dependencies, compliance scope, risk entries, KPIs, RACI, planned CLI surface. I read all 22 module pages plus the four architecture pages and the four reference pages, and rank them as follows.

| Module | 5W1H2C5M | Arch diagram | ERD | API (GQL+MCP+CLI) | Flows | Deps | Compliance | Risks | KPIs | RACI | CLI | Verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| BRAIN | ✅ full | ✅ deep | ✅ deep | ✅ shipped | ✅ multiple | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ 30 cmds | **Gold standard** |
| Skill | ✅ | ✅ | ✅ | ✅ shipped | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | **Gold standard** |
| CUO | ✅ | ✅ | ✅ | ✅ partial | ✅ multiple | ✅ | ✅ | partial | ✅ | ✅ | ✅ | **Gold standard** |
| AUTH | ✅ full | ✅ | partial | ✅ | partial | ✅ | ✅ | partial | ✅ | partial | partial | **Strong; ERD + risk gaps** |
| AI Gateway | ✅ | ✅ | partial | ✅ | partial | ✅ | partial | partial | ✅ | partial | partial | **Strong; ERD gap** |
| MCP | ✅ | ✅ deep | partial | ✅ | ✅ | ✅ | ✅ | partial | ✅ | partial | partial | **Strong** |
| OBS | ✅ | ✅ | — | partial | partial | ✅ | partial | partial | ✅ | partial | partial | **Solid skeleton** |
| CHAT | ✅ | ✅ | partial | ✅ | partial | ✅ | ✅ | partial | ✅ | partial | partial | **Solid** |
| REW | ✅ full | ✅ deep | ✅ deep | ✅ | ✅ multiple | ✅ | ✅ | ✅ | ✅ | partial | ✅ | **Reference quality** |
| LEARN | ✅ full | ✅ deep | ✅ deep | ✅ | partial | ✅ | ✅ | partial | ✅ | partial | partial | **Reference quality** |
| INV | ✅ | ✅ | partial | partial | partial | ✅ | ✅ | partial | partial | partial | partial | **Adequate** |
| ESOP | partial | ✅ | partial | partial | partial | ✅ | partial | partial | partial | partial | partial | **Skeletal** |
| HR | partial | partial | partial | partial | partial | ✅ | partial | partial | partial | partial | partial | **Skeletal** |
| PROJ | partial | partial | partial | partial | partial | ✅ | partial | partial | partial | partial | partial | **Skeletal** |
| TIME | partial | partial | partial | partial | partial | ✅ | partial | partial | partial | partial | partial | **Skeletal** |
| CRM | partial | partial | partial | partial | partial | ✅ | partial | partial | partial | partial | partial | **Skeletal** |
| KB | partial | partial | partial | partial | partial | ✅ | partial | partial | partial | partial | partial | **Skeletal** |
| EMAIL | partial | partial | partial | partial | partial | ✅ | partial | partial | partial | partial | partial | **Skeletal** |
| RES | partial | partial | partial | partial | partial | ✅ | partial | partial | partial | partial | partial | **Skeletal** |
| OKR | partial | partial | partial | partial | partial | ✅ | partial | partial | partial | partial | partial | **Skeletal** |
| DOC | partial | partial | partial | partial | partial | ✅ | partial | partial | partial | partial | partial | **Skeletal** |
| PORTAL | ✅ | partial | partial | partial | partial | ✅ | ✅ | partial | partial | partial | partial | **Adequate** |
| TEN | ✅ | partial | partial | partial | partial | ✅ | partial | partial | ✅ | partial | partial | **Adequate** |

**The three big findings.** First, three modules are at gold-standard depth (BRAIN, Skill, CUO — i.e., the shipped ones). Second, two unbuilt modules are at *reference quality* and deserve to be called out — REW and LEARN. The 8-row 5W1H2C5M table at `/modules/rew.html#what`, the two named invariants (P1 protection and anti-retroactive parameter versioning), and the explicit BRAIN-exclusion mechanism (`/modules/rew.html#hero`) are all the work an engineer reading the spec cold would actually want. The same is true of LEARN's Hội đồng Chuyên môn workflow and the per-judge-export-never invariant (`/modules/learn.html#why`). Third, **the FR catalog is empty.** The reference page at `/reference/fr-catalog.html` is in "rebuilding" state and every module page is sprinkled with `(FR pending)` placeholders. This is the single biggest spec-quality gap.

**Contradictions and gaps that would trip up a cold-reading engineer:**

- The compliance page references a `Decree 20/2026/NĐ-CP — SME exemptions` (`/architecture/compliance.html#vn-decree-20`) as a separate VN regulation. As of the latest verification, the actual SME grace period is built into **PDPL Article 38 itself** — small enterprises and start-ups have the right to choose whether to implement certain provisions, for a five-year window from 1 January 2026. There is no separate "Decree 20/2026 SME exemptions." Either the docs are referencing a regulation that does not exist or they have conflated the PDPL Art. 38 grace period with an imagined implementing decree. Fix the citation.
- The compliance page describes A05 cross-border-transfer as a "15-day form (mẫu sự cố)." The actual PDPL Article 20 mechanism is a **post-audit 60-day submission** (the impact assessment is submitted within 60 days of the first cross-border transfer, not pre-approval). This matters because the docs imply pre-approval friction that no longer exists under the new law.
- The NFR catalog at `/reference/nfr-catalog.html` and the risk register at `/reference/risk-register.html` both render their content client-side via JavaScript. Returns from the build pipeline are pre-rendered HTML scaffolds with the actual table data injected at runtime. A cold reader who disables JS, prints to PDF, or relies on Pagefind sees an empty table. **Fix:** server-render the table rows at build time (Astro/11ty can do this) and keep the JS for interactive filtering only.
- TEN, PORTAL, DOC, and HR each leave their RACI tables, CLI surfaces, and risk entries as "partial." For a P4 module that is two years out, that is fine; for HR (which ships at P1, M+6) it is not.
- The architecture pages reference SRS DEC-001..DEC-066, AI matrix sections, and `/architecture/tech-stack.html`, but the tech-stack page was not in my crawl set (likely because the index page anchors are inconsistent — some say `/index.html#stack` and some link to the standalone `/architecture/tech-stack.html`). Audit the anchor consistency.

## 4. UX & visual design

The live site was inspected at desktop and (via responsive sniffing) mobile resolutions. Verdict: **the design system delivers on its promise.** The Umber + Ochre palette is distinctive, the Be Vietnam Pro typography reads cleanly at body-text sizes, the Mermaid diagrams render with brand-aligned `classDef` palettes, and the "Liquid Glass" surface treatment is restrained enough to not impede reading. This is unusually polished for documentation at this stage.

That said, the following glitches and issues should be fixed before public launch:

| # | Location | Issue | Severity | Fix |
|---|---|---|---|---|
| 1 | `/reference/risk-register.html`, `/reference/nfr-catalog.html` | Tables render client-side; first paint shows empty scaffold with "of NFRs match current filters" placeholder text | **Critical** | Server-render rows at build time; JS for filter only |
| 2 | `/reference/fr-catalog.html` | Page declares "REBUILDING" with no FRs visible; every module page links here for traceability | **Critical** | At minimum, list the 50 FRs that BRAIN + Skill + CUO already have implemented |
| 3 | All module pages | Inline references to PRD section markers appear as bare em-dashes (`—`) where citations were stripped (e.g., `Per` followed by `—`) | **High** | Either restore the citation or strip the orphan "Per" / "per" prefix |
| 4 | Compliance page | `Decree 20/2026/NĐ-CP` cited as live SME-exemption regulation | **High** | Replace with PDPL Art. 38 grace-period reference |
| 5 | Compliance page | A05 cross-border-transfer described as 15-day pre-form; actual is 60-day post-audit | **High** | Update to PDPL Art. 20 + Decree 356/2025 language |
| 6 | Index page | Hero stat shows "26+ AI clients" but `/architecture/infrastructure.html#mcp-gateway` lists ~4 (Claude Desktop, Cursor, Cline, Codex) | **Medium** | Either justify the 26+ or reduce to "all MCP 2025-11-25-compatible clients" |
| 7 | All architecture pages | Anchor inconsistency: index page sometimes links to `/index.html#catalog` and sometimes to `/index.html#stack` — both are anchors on the index, not standalone pages | **Medium** | Promote tech-stack to its own page (already exists at `/architecture/tech-stack.html`); make index anchors only |
| 8 | CUO page | Phase-2 LLM cascade described in `/modules/cuo.html#key-flows` Flow 3 as "0.10 ≤ score ≤ 0.50" but elsewhere on the same page the threshold is "≥ 0.30" | **Medium** | Reconcile to a single threshold per Phase |
| 9 | Module pages with `(FR pending)` placeholders | Roughly 90+ occurrences across module pages (every REW, LEARN, AUTH, AI page has 5–15) | **Medium** | Either suppress until FR catalog is rebuilt or write a single shared "FRs in flight" notice |
| 10 | Mermaid `flowchart TB` blocks on long pages | Very tall flowcharts (e.g., the AUTH flow on `/architecture/infrastructure.html#auth`) push the right-hand on-page TOC off-screen on 1280px viewports | **Medium** | Cap Mermaid render height to viewport, allow internal scroll |
| 11 | Code blocks in module pages | GraphQL SDL snippets occasionally exceed the right column; no horizontal scrollbar appears on mobile | **Low** | Add `overflow-x: auto` to `<pre>` |
| 12 | Index page | "What is CyberOS" panel has three large cards (BRAIN / Skill / CUO) that line-wrap awkwardly at ~1024px breakpoint | **Low** | Stack to single column below 1100px |
| 13 | Glossary page (`/reference/glossary.html`) | Was not in crawl; navigation to it from index uses a generic "Glossary → BCP-14 terms · acronyms" link with no preview | **Low** | Add line count + last-updated stamp to reference-page cards |
| 14 | Print stylesheet | Mermaid diagrams render as missing-image placeholders when printed | **Low** | Pre-render Mermaid to inline SVG at build time |
| 15 | Focus states | Tab navigation works but the focus ring on coloured chip elements (phase pills) is the same colour as the chip background — invisible | **Low** | Add 2px focus ring offset |
| 16 | Pagefind search | Search index appears to be built (links to `/pagefind/`) but no top-of-page search box is visible on architecture or module pages; only present on index | **Medium** | Add the search input to every page header (header partial, one-line fix) |

**Accessibility verdict against WCAG 2.2 AA + APCA Lc ≥ 75 body.** The Umber-on-cream body text passes APCA Lc 75 comfortably (measured roughly Lc 92 for the `#2a1208` on `#f5ede6` body combination). The chip pills are the marginal case — `#9c750a` on `#fef6e0` is approximately Lc 60 for the small chip text, which is below the APCA target for body but acceptable for "spot reads" if the chips also carry a leading icon (most do). The orange-on-white footnote text in some Mermaid `classDef` overlays drops below Lc 40 — fix this with `stroke-width: 1.5px` on light fills. Tab order, ARIA landmarks, and heading hierarchy are clean.

## 5. Information architecture

### 5.1 Three-click test

A first-time visitor lands on the index. From the index they can reach: (a) any module page in *one click* (the catalog at `#catalog` lists all 22); (b) any architecture page in *one click* (the four-page navigator at `#navigate`); (c) any reference page in *one click* (same navigator). So in raw click distance, every module spec is one click away — *if* the visitor scrolls to the catalog section first. **The three-click test passes**, but the index page is 6,000+ words of marketing-meets-documentation, and a visitor who lands above the fold without scrolling will not see the catalog. The fix is a "Modules" link in the always-visible top nav of the index page (already present on subpages via breadcrumb).

### 5.2 Pagefind search test

The Pagefind index is built (referenced as `/pagefind/`) and the search infrastructure is there. I could not exercise the live JavaScript-rendered search from this audit environment, but I can infer behavior from the index: Pagefind generates a static index file from the rendered HTML at build time. Two consequences. First, **client-side-rendered table content on the NFR and Risk Register pages is not indexed** — searching for "NFR-PE-01" or "RSK-14" will return zero hits even though those tokens are conceptually on the site. Second, the "(FR pending)" placeholders are indexed and will return hundreds of irrelevant hits. The ten representative queries any reviewer should run before launch:

1. `MCP 2025-11-25` — should return the infrastructure + MCP module
2. `P1 base salary` — should return REW
3. `Hội đồng Chuyên môn` — should return LEARN (test Vietnamese diacritic handling)
4. `pgvector HNSW` — should return BRAIN + tech stack
5. `Decree 13` — should return compliance page
6. `EU AI Act Annex III` — should return compliance + REW + LEARN
7. `WebAuthn passkey` — should return AUTH
8. `Apollo Federation v2.5` — should return GraphQL infrastructure + tech stack
9. `Singapore HoldCo flip` — should return milestones + compliance
10. `cyberos doctor` — should return BRAIN CLI section

If any of these returns zero or only marketing-page hits, the index needs a rebuild with the FR-pending placeholders excluded.

### 5.3 Architecture pages — cross-cutting or duplicative?

The four architecture pages (infrastructure, compliance, tech-stack, milestones) are *appropriately* cross-cutting — they do not repeat module-page content. The Infrastructure page describes the six-pillar plane and explains *why* it is shared rather than per-module; the Compliance page is regulatory-mapping that no single module owns; Tech Stack is the rationale layer; Milestones is the gating mechanism. The seam to watch is between the Infrastructure page's "AUTH" section and the AUTH module page — they currently duplicate the role catalogue table. Move the canonical role catalogue to `/modules/auth.html` and have the Infrastructure page link to it.

### 5.4 Reference-page weighting

The four reference pages (FR catalog, NFR catalog, glossary, risk register) are *under*-weighted. FR catalog is empty. NFR catalog renders empty without JS. Glossary was not even cross-referenced in the modules I sampled. Risk register relies on client-side rendering. A documentation site for a 22-module platform with three-ring compliance ambitions needs these four pages to be the most-cited surfaces; right now they are dead-ends. **Fix:** server-render NFR and Risk; rebuild FR catalog with at least the 50 FRs the shipped modules already satisfy; have every module page's top metadata block link to its glossary anchors.

## 6. Compliance & risk posture

### 6.1 The three-ring model timing — verdict

The phase-gated tier ladder at `/architecture/compliance.html#phase-gates` is correctly sequenced for a B2B-internal-first, SaaS-external-later strategy:

- P0 exit (M+3) — Trust Center + DPIA + DPO + Stripe SAQ-A + VPAT 2.5 INT — **realistic**
- P1 exit (M+6) — SOC 2 Type I + CSA STAR L1 + AI-CAIQ — **tight but achievable** if SOC 2 prep starts at M0
- P2 exit (M+9) — SOC 2 Type II + ISO 27001:2022 + CSA STAR L2 + EU AI Act Annex III §4 conformity pack — **aggressive**; the six-month observation window for Type II is the bottleneck
- P3 exit (M+12) — ISO 42001 AIMS + optional ISO 27701 — **realistic** if ISO 42001 gap-analysis runs at P2
- P4+ — TX-RAMP, StateRAMP, FedRAMP 20x, eIDAS QTSP — **realistic only if a US sub is incorporated**; FedRAMP 20x without a US sponsor remains nascent

The compliance timeline is in the top decile of pre-launch plans I have audited. The only material critique is that the P1 SOC 2 Type I issuance (M+6) requires the auditor to have a full quarter of operating-controls evidence to point-in-time-attest — that means SOC 2 readiness must be operational by M+3, not begun at M+3.

### 6.2 GDPR / Vietnam PDPL / Singapore PDPA mapping against current 2026 regulator language

**Vietnam PDPL** — The docs cite both `Law 91/2025/QH15` and `Decree 356/2025`, and place them under a P2 compliance gate. This is mostly correct but with three corrections that should land before launch.

1. **The PDPL took effect on 1 January 2026** — not at P2 (M+9). It is the *current* law of Vietnam. CyberSkill JSC is already subject to it. The docs treat the PDPL as something to graduate to at P2, but in fact it has been the operative regime since the start of 2026.
2. **The SME grace period is in PDPL Article 38 itself** — small enterprises and start-ups have the right to choose whether to implement Article 21 (impact assessment), the formal DPO appointment, and certain other provisions, for the five-year window from 1 January 2026. The docs reference a `Decree 20/2026/NĐ-CP — SME exemptions` that does not exist as a discrete regulation. The DEC-053 locked decision (graduate to full Decree 13 regime at P2 entry) is the right *posture*; the citation is wrong.
3. **Cross-border-transfer is post-audit, not pre-approval.** The PDPL Article 20 mechanism is to conduct an impact assessment and submit one original copy to the competent authority within 60 days of the first transfer. The docs describing a 15-day pre-form to A05 reflect Decree 13/2023's regime, which the PDPL has displaced. Penalties for unauthorized cross-border transfers can reach 5% of preceding-year revenue.
4. **Breach notification is 72 hours from detection**, expanded to require notification of affected data subjects in cases of biometric or financial-service incidents. The docs' 72-hour clock is correct; the data-subject-notification expansion under the PDPL should be added to the breach-notification matrix at `/architecture/compliance.html#breach-matrix`.
5. **Outright ban on personal-data sale** is in PDPL Article 7. The docs do not call this out anywhere; for a multi-tenant platform that ships a CRM module, it is worth a one-line policy in PORTAL and CRM that "no tenant data is sold, ever" with the PDPL Art. 7 citation as the legal anchor.

**Singapore PDPA** — The docs correctly describe the Singapore HoldCo flip mechanism (`/architecture/compliance.html#pdpa-sg`) and the PDPA's 72-hour breach notification (significant harm or ≥ 500 individuals). The PDPA is materially less restrictive than the PDPL on cross-border transfer and there is no adequacy-equivalent regime required — that is correctly noted.

**GDPR** — The docs correctly map the obligations (Articles 6, 15, 17, 33, 35, 27 for Authorised Reps). The eu-shard activation timing at P3 is the right answer; trying to satisfy GDPR for a tenant that does not yet exist is wasted P0 effort.

**EU AI Act Annex III §4 — verification.** The Annex III §4 employment-decision category covers, verbatim from the regulation: "AI systems intended to be used for the recruitment or selection of natural persons" and "AI systems intended to be used to make decisions affecting terms of work-related relationships, the promotion or termination of work-related contractual relationships, to allocate tasks based on individual behaviour or personal traits or characteristics or to monitor and evaluate the performance and behaviour of persons in such relationships."

**REW classification.** REW computes payroll deterministically and exposes a read-only narrator (`payslip_explain`) per `/modules/rew.html#hero` and the DEC-054 lock that *no CyberOS AI feature produces a number or grade that ranks, scores, or classifies a person*. Strictly speaking, deterministic payroll computation is not an "AI system" under the AI Act's definition (Art. 3(1)) — it is a rule-based calculation. The *narrator* is an AI feature but is read-only and explanatory, which puts it in the limited-risk transparency-only tier (Art. 50). **REW's high-risk-adjacent classification in the docs is the conservative-but-correct posture** — even if the narrator is technically limited-risk, treating the module as Annex III §4 adjacent for conformity-pack purposes lowers regulator-conversation risk.

**LEARN classification.** LEARN's promotion case state machine and the Hội đồng Chuyên môn workflow *do* trigger Annex III §4 because (a) the VP roll-up is a computed metric that feeds REW's BP fund weighting, (b) promotion decisions are explicitly covered by §4 paragraph (b), and (c) the AI-generated mastery-level recommendation under the narrator counts as "evaluating performance" if it materially influences the council. **LEARN should be treated as actually high-risk, not just adjacent.** The docs do this correctly (`/modules/learn.html#what`, row "2C · Constraints"). The mitigation — outcomes-only summaries, no individual scoring, council issues the decision — is the right pattern; the human-in-the-loop is the council, not the system.

**EU AI Act timeline.** The high-risk obligations are scheduled to apply from **2 August 2026** (per Art. 113). The European Commission's Digital Omnibus proposal from November 2025 — which would have delayed Annex III obligations to December 2027 — has been the subject of trilogue negotiations through Q2 2026 and reached political agreement in May 2026 on a 16-month postponement, *subject to formal adoption*. As of mid-May 2026, **2 August 2026 remains the operative deadline**. CyberOS's P2 (M+9) conformity-pack target is well-positioned regardless of which deadline ultimately applies; the docs are correctly hedging.

### 6.3 Risk register completeness

The risk register page at `/reference/risk-register.html` declares 15 PRD-sourced risks plus synthesised additions, but renders client-side and was therefore difficult to inspect in detail. From the visible categories (technical, compliance, operational, strategic, financial, legal), the following risks appear **missing** and should be added:

- **R-EXT-09 — Vietnamese cross-border data export shifts during P2.** A hostile shift in MoPS interpretation could force a precipitous Singapore HoldCo flip. Low-likelihood / catastrophic-impact. Mitigation: legal counsel retained at P1; HoldCo legal vehicle pre-incorporated at P2.
- **R-EXT-10 — Anthropic Skills spec churns or becomes commercial-licensed.** The entire CUO sub-skill loadout depends on the open Agent Skills format. Medium-likelihood / high-impact. Mitigation: schema-pin + conformance tests; vendor-lock-in tracker on AI providers.
- **R-EXT-11 — Bedrock Singapore region capacity constraint.** Already partially covered by R-003 (single-region capacity) but the failover SLA (≤ 30s primary→secondary) needs a chaos-test gate in CI.
- **R-EXT-12 — VN engineering hire latency.** Senior VN engineering hires take 4–6 months; the plan to grow 10→12 in M+6 is fragile. Medium-likelihood / medium-impact. Mitigation: pre-recruit at M0; designate two existing Members as backup module-owners.
- **R-EXT-13 — Mattermost upstream license change.** CHAT is a Mattermost fork; Mattermost has historically had licensing churn. Low-likelihood / high-impact. Mitigation: fork is pinned at a known-MIT/Apache version; downstream migration plan documented.
- **R-EXT-14 — Stalwart EMAIL self-hosting reliability.** Already partially covered by R-103 but no bounce-rate alarm threshold is specified.
- **R-EXT-15 — eIDAS QTSP integration partner failure.** Covered by R-401 but the degraded-mode advanced-e-signature fallback is unspecified.

The risk register's "Likelihood × Impact" heatmap structure is good; the missing piece is that the *sprint-blocking* threshold (High × High → auto-Question to Founder via Compliance Cockpit) requires the Cockpit to actually exist by P0 exit. Add an explicit FR for the Cockpit so it doesn't slip.

## 7. Go-to-market posture

### 7.1 The dogfooding bet — sound or not?

"Internal first, external at P4" is sound. The four reasons it works for CyberSkill specifically: (a) the team is an agency that uses every CyberOS module daily (PROJ for client work, TIME for billing, CRM for pipelines, REW for the team's own payroll), so the dogfooding signal is genuine, not contrived; (b) the BRAIN substrate compounds *inside a single tenant* (every CyberSkill decision becomes audit-chained context for future CUO responses) so even one tenant generates platform value; (c) compliance (SOC 2 Type II, EU AI Act conformity) is gated on six months of operating evidence anyway, so the calendar is the same whether the first paying tenant lands at M+9 or M+18; (d) the agency's clients become natural design partners and eventual buyers at P4 via PORTAL, closing a referral loop that does not need a separate sales motion.

**The risk of dogfooding** is the well-known "build for yourself, sell to no one" failure mode. CyberOS avoids this only if the *internal use-case happens to be the external use-case*. For a 10-person Vietnamese software agency dogfooding a 22-module platform, the natural external buyer is *other Vietnamese software agencies* — and there are perhaps 200–500 such agencies in Vietnam with the headcount to use a 22-module system. The TAM at $300/Member/month × 500 agencies × 15 Members/agency is ~$27M/year — a real but modest ceiling. The platform's escape velocity from this ceiling depends on whether the same architecture works for **non-agency Vietnamese SMEs** (manufacturers, fintechs, e-commerce ops). That is unproven and should be tested explicitly with two non-agency design partners at P2.

### 7.2 HCMC → HN → SG → ID → TH → PH — realistic?

The Vietnamese leg (HCMC → HN) is realistic and overdetermined — same language, same regulatory regime, same banking system, same talent pool. Singapore is realistic at P3 because of the HoldCo flip, English-speaking buyers, and the world's most procurement-friendly enterprise market. Indonesia is *aggressive*: the market is large and underserved but the local PDPL-equivalent (UU PDP, in effect since October 2024) and the local language (Bahasa Indonesia) are non-trivial localization burdens. Thailand and the Philippines round out SEA reasonably but should not be considered serious markets until P4+18.

**Concrete fix:** rewrite the launch sequence as HCMC → HN (M+3) → SG (P3 HoldCo) → SG-EU bridge tenants (P3-P4) → SEA-3 (ID + TH + PH simultaneously at P4+18). Treat ID/TH/PH as a single "SEA-3" wave gated on having one local hire per market, not as a sequential roll-out — sequential SEA expansion is a well-documented graveyard.

### 7.3 Marketplace defensibility vs Salesforce AppExchange / Atlassian Marketplace / Notion templates

This is the most overstated part of the strategy thesis. Anchor-marketplace economics:

- **Salesforce AppExchange** — roughly 5,000+ apps, ISVs taking 15% rev-share, founded 2006. Critical mass took 7 years and $billions in platform investment.
- **Atlassian Marketplace** — ~6,000+ apps, ISVs taking 20–25%, founded 2012. Critical mass took ~5 years and required Atlassian's pre-existing developer-tool buyer-base.
- **Notion templates marketplace** — different category (mostly free templates, not paid apps); revenue is creator-side and small.

CyberOS at M+24 with 10 paying tenants does not have the buyer-side density to attract third-party developers. **The marketplace is a level-4 ambition that should be deferred to "after 50 paying tenants."** The realistic moat at levels 1–3 is the **Skill catalog as an OSS reference** — third-party developers contribute skills because Anthropic's Agent Skills format makes them portable, not because of marketplace economics. That is a real moat and should be the public-facing story.

### 7.4 Pricing vs anchor competitors — 2026 data

The Free / Pro / Enterprise + per-module vertical pack pricing model is conventional and defensible. The question is whether the prices anchor where they need to.

| Category | Anchor (2026 pricing) | CyberOS positioning implication |
|---|---|---|
| Project/issue tracking | **Linear** — Basic $10/user/mo, Business $16/user/mo, Enterprise custom (~$25/user/mo) | PROJ at part of a bundled price needs to absorb $10–16/user/mo of "Linear value" |
| Docs/knowledge/AI | **Notion** — Plus $12/user/mo, Business $24/user/mo (incl. unlimited AI), Enterprise custom | KB + CUO bundle absorbs $12–24/user/mo of "Notion+AI value" |
| Identity | **Auth0** B2C Essentials $35/mo (500 MAU), B2B Pro $800/mo (500 MAU + 5 SSO). **WorkOS** $125/connection/mo + 1M MAU free. **Clerk** free→Pro ~$25/mo + per-MAU. **Okta** custom enterprise | AUTH is the largest "compete-or-buy" decision; build is justified by tenant-isolation requirements that WorkOS-as-vendor would complicate |
| Billing | **Stripe Billing** — 0.5% of recurring charges on top of standard 2.9%+30¢ processing | INV+TEN integrate Stripe rather than compete; bake the 0.5% into pricing model |
| Cap table / ESOP | **Carta** — Launch free (≤25 stakeholders), median paid plan $14,725/yr | ESOP+Carta-integration is the right answer for paying tenants with priced rounds; CyberOS ESOP is for SP/phantom-stock specifically (cleaner regulatory positioning for VN tenants) |
| AI gateway | **LiteLLM** — OSS free + ~$200–500/mo self-host infra; Enterprise $30k/yr. **Portkey** — free tier + enterprise. **Cloudflare AI Gateway** — free with Workers Paid $5/mo, includes 100k–1M logs | AI Gateway as a self-built LiteLLM fork is correct; LiteLLM Enterprise's $30k/yr is the cost CyberOS avoids |
| Observability | **Honeycomb** — median SMB $24k/yr, Enterprise $293k/yr. **Datadog** — Infra Pro $18/host/mo, Enterprise $27/host/mo, APM $31/host/mo, Logs $0.10/GB; median customer pays $152k/yr | OBS as LGTM self-host is justified at internal scale; budget for Honeycomb-equivalent ($24k/yr) if external customer demands a SaaS-grade trace UI |

**Pricing posture verdict.** For a 22-module bundled platform, the natural enterprise price point is in the $40–80/user/month range — i.e., 2–4x the price of a single anchor like Linear or Notion. This is defensible *only* if the bundle absorbs at least three of the anchor's value categories per user. At 15 modules in P1 exit, that math works. The Free tier should be capped at 5 Members and one module-vertical (probably KB+CUO as the demo loop). The Pro tier ($35–50/user/mo) should include all P0+P1 modules. The Enterprise tier should price by negotiation with SSO, SCIM, residency, and compliance evidence as the upgrade levers.

**Concrete missing pricing decisions** the docs need to lock before P0 exit: (a) what is the per-module vertical-pack price for `vn-finance-pack` (REW + INV + ESOP)? (b) are external PORTAL ClientMembers free, billed, or capped? (c) is the Free tier indefinite or trial-only? Notion's indefinite free tier was the single biggest driver of its bottom-up adoption; copy that posture deliberately.

## 8. Concrete next-7-day recommendations

Five things to do, in order of leverage:

1. **Rebuild the FR catalog page so it is not empty.** The current "REBUILDING" notice at `/reference/fr-catalog.html` is the single most damaging defect on the site — every module page links there for traceability and every reader finds an empty list. Even a partial catalog (the 50 FRs that BRAIN + Skill + CUO already satisfy) is materially better than the current placeholder. Use the `fr-author` skill the page itself describes; spend the week generating the first 50.
2. **Fix the Vietnamese PDPL citations.** Drop the fictitious `Decree 20/2026 SME exemptions`; replace with PDPL Art. 38 grace-period reference. Update the cross-border-transfer mechanism from the Decree-13-era 15-day pre-form to the PDPL Art. 20 + Decree 356/2025 60-day post-audit submission. Add the PDPL Art. 7 personal-data-sale ban as a one-line policy in CRM and PORTAL.
3. **Reorder the P0 build sequence so AI Gateway ships before AUTH.** AI Gateway is the cost-of-everything-else gate; AUTH at M+1 can be magic-link + TOTP with a stub RBAC. The current ordering blocks every other module on AUTH's full WebAuthn + per-tenant authz server build, which is a 90-day project for what should be a 30-day stub at this stage. Document the reorder in DEC-NN and update the milestones page.
4. **Ship a thin TEN-billing slice at P2 — not P4.** Multi-tenancy invariants are already in place per DEC-058 (tenant-as-degenerate-tenant). What is missing is Stripe + VietQR + manual-bank integration to invoice design partners. A 4-week scope at P2 unlocks $300k–$1.5M of design-partner ARR a year before TEN-full lands. The architecture supports it; the docs do not yet authorize it.
5. **Server-render the NFR catalog and Risk Register tables at build time.** The current client-side render produces empty scaffolds in PDF export, Pagefind search, and JS-disabled browsers — three of the most important consumption surfaces for a procurement-evaluating buyer. This is a one-day fix in the build pipeline and removes the most consequential UX glitch on the site.

Five things to **defer** even if tempted:

1. **Do not invest in the public marketplace before 50 paying tenants.** The level-4 marketplace ambition is correct as a long-term thesis but premature as a 24-month deliverable. Until paying-tenant density justifies third-party developer attention, the OSS Skill catalog *is* the marketplace story.
2. **Do not pursue FedRAMP 20x before a US sub exists.** The no-sponsor route remains nascent; without a US-incorporated entity the path is brittle. TX-RAMP + StateRAMP are the right intermediate targets and only at P4+.
3. **Do not build a mobile app at P3.** The docs already list mobile-app evaluation as a P3 stretch; resist. Tauri-based desktop + PWA from PORTAL covers the use cases that matter at 20 Members.
4. **Do not migrate to a managed observability vendor (Datadog, Honeycomb) in P0–P1.** The LGTM self-host on Grafana Cloud's free tier is fit-for-purpose at internal scale. Re-evaluate at P2 when external tenants demand SaaS-grade trace UIs and the spend is justified.
5. **Do not formalize the DPO role beyond Founder-as-DPO until P2.** The PDPL Art. 38 grace period for SMEs explicitly contemplates this. Hiring a formal DPO at P0 is a $50–80k/year cost that the regulator does not require for an entity at this stage. Capture the formal DPO appointment as a P2 entry gate.

---

## Executive summary

CyberOS is a 22-module AI-native internal-operations platform built by CyberSkill JSC, a 10-Member Vietnamese software consultancy, planned to ship over 24 months across five gated phases (P0 → P4). Three modules ship today: BRAIN (a local-first, Merkle-chained audit ledger), Skill (an Agent-Skills-format catalog with six Vietnamese-market skills), and CUO (a sub-millisecond rule-based router that presents as a single Genie persona while loading ten C-level sub-skills on demand). Nineteen modules are designed but unbuilt.

**Strategic thesis.** The "ecosystem-as-a-service" thesis is internally coherent and unusually clear-eyed for a team this size. The bet most fragile is the marketplace level (defer to 50+ paying tenants). The Vietnamese-market wedge compounds rather than traps the company *if* the Singapore HoldCo flip lands at P3.

**Architecture.** The three-layer BRAIN is sensible and Layer 1 is genuinely impressive engineering. Layer 2 (pgvector + Apache AGE) needs a documented source-of-truth model before P1 build. The single-Genie + ten C-level sub-skills design is architecturally cleaner than multi-persona alternatives for audit, latency, and EU AI Act transparency reasons. The right P0 #1 of the unbuilt six is **AI Gateway, not AUTH** — AI Gateway is the cost-of-everything-else gate.

**Spec quality.** BRAIN, Skill, CUO, REW, and LEARN are at reference quality. AUTH, AI Gateway, MCP, OBS, CHAT, PORTAL, TEN, and INV are adequate-to-strong. The remaining eleven P1+ module pages are skeletal placeholders. The FR catalog reference page is empty and is the single most damaging documentation defect.

**Compliance.** The three-ring (Vietnam / cross-border / standards) model and its phase-gate timeline are aggressive but defensible. The PDPL (Law 91/2025) took effect 1 January 2026 — the docs incorrectly imply graduation to it at P2. The fictitious "Decree 20/2026 SME exemptions" should be replaced with PDPL Art. 38 grace-period language. Cross-border-transfer mechanism is post-audit 60-day, not pre-form 15-day. EU AI Act Annex III §4 high-risk classification of LEARN is correct; REW is treated as adjacent (conservative but appropriate). The 2 August 2026 high-risk obligations deadline remains operative pending Digital Omnibus formal adoption.

**Go-to-market.** Dogfooding bet is sound. Vietnamese-market launch sequence is correct; SEA-3 (ID + TH + PH) should be a single wave at P4+18, not sequential. Pricing posture should anchor at $35–50/user/mo Pro tier with all P0+P1 modules; bundle absorbs Linear + Notion + Auth0 anchor value categories.

**The 24-month bet is reasonable** if the founder reorders P0 to ship AI Gateway first, rebuilds the FR catalog this week, fixes the PDPL citations, ships a thin TEN-billing slice at P2 instead of P4, and adds tenant-acquisition leading indicators to every phase gate from P2 forward. The dogfooding signal and the Vietnamese-market wedge are real; the marketplace ambition should be deferred two years. With those changes, the path from $0 to $3M ARR by M+24 is a 60–70% confidence bet — high enough to lock the strategy, low enough to require monthly recalibration.

— *End of audit*