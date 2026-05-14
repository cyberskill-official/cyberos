# CyberOS — Strategic Push Forward

*Where CyberOS sits today, where world-class ecosystems are heading, and how to turn CyberOS into the internal-then-commercial Ecosystem-as-a-Service moat for CyberSkill.*

---

## 1 — Where CyberOS sits today (May 2026)

Three modules shipped, twenty more in plan. The architectural foundation is real:

- **Memory module** — local-first, audit-chained, cryptographically verifiable personal memory store. 245 tests green.
- **Skill module** — Anthropic Agent Skills open-standard compliant. 20 SKILL.md bundles indexed, 6 Vietnamese-market skills shipped, Rust+Wasmtime+Bun toolchain. All 7 audit phases done.
- **CUO module** — rule-based router. 15/15 routing fixtures + 15/15 pytest tests. Phase 1 (rule-based) shipped, Phases 2–4 (LLM, multi-skill chains, persona switching) designed.
- **Documentation site** — 31 pages, 226 Mermaid diagrams, 341 FRs, 100 NFRs, 199 glossary terms, 42 risks. Multi-page Path C.

The remaining 19 modules (AUTH, AI Gateway, MCP Gateway, OBS, CHAT, EMAIL, PROJ, TIME, CRM, KB, HR, REW, LEARN, INV, ESOP, RES, OKR, DOC, PORTAL, TEN) are scaffolded in the docs but not built.

## 2 — World-class ecosystem landscape (2026 snapshot)

Here is what CyberOS is competing with, organized by where they sit on the **closed ↔ open** + **horizontal ↔ vertical** axes:

### The horizontal-closed giants

| Player | Revenue | What they own | Their agentic move |
|---|---|---|---|
| **Microsoft 365** | ~$80B/yr | Office + Teams + SharePoint + Outlook + Power Platform | Copilot baked into every product; closed agent layer |
| **Google Workspace** | ~$30B/yr | Gmail + Docs + Drive + Meet + Calendar | Gemini baked in; closed agent layer |
| **Salesforce** | ~$35B/yr | Sales Cloud + Service Cloud + Marketing Cloud + Slack | Einstein/Agentforce; closed agent layer |
| **Atlassian** | ~$5B/yr | Jira + Confluence + Bitbucket + Loom + Browser Co (Arc/Dia, Oct 2025 $610M acq) | Rovo; building agentic dev platform |

These are entrenched. CyberOS cannot win horizontally against any of them. The only viable plays are: (a) interop via MCP, (b) regional vertical wedge (Vietnam), (c) net-new categories (agentic-native ops, where they're playing catch-up).

### The vertical-open challengers

| Player | Bet | Why they matter to CyberOS |
|---|---|---|
| **Notion** | Knowledge ops + AI ($10B val) | Closest analog for KB + ad-hoc workflow; lacks agentic substrate |
| **Linear** | PM done right (~$1B val) | Performance bar for PROJ; modern stack reference |
| **Plane.so** | Open-source Linear | Open-source playbook |
| **Retool** | Internal tools builder | Vertical CyberOS could absorb |
| **HuggingFace** | Open AI hub | The "agent registry" CyberOS skills could one day publish to |

### The agent-spec wars

| Player | Status | CyberOS posture |
|---|---|---|
| **Anthropic Agent Skills** (Dec 2025 spec release) | Open standard; 26+ clients adopting | **CyberOS is a citizen.** Skills compatible with Claude, Codex, Cursor, Goose, Amp, etc. |
| **OpenAI Apps SDK + ChatGPT Custom GPTs** | Proprietary; ecosystem-locked | OpenAI itself adopting SKILL.md format inside Codex CLI (Dec 2025) — the standard won |
| **MCP (Model Context Protocol)** | LF-donated Dec 2025; 10K+ public servers | **CyberOS speaks MCP natively.** MCP Gateway is a P0 module |
| **Sigstore Rekor / transparency logs** | Open; growing | CyberOS audit chain anchors here long-term |

### The ecosystem-as-a-service playbooks (2024–2026 lessons)

| Platform | Marketplace size | Key lesson for CyberOS |
|---|---|---|
| **Salesforce AppExchange** | ~7,000 apps | Vertical packs unlock enterprise sales |
| **Microsoft AppSource** | ~50,000 apps | Compliance certifications drive adoption |
| **Shopify App Store** | ~10,000 apps | Revenue share + dev-friendly tooling matters |
| **Atlassian Marketplace** | ~5,000 apps | Forge platform took 5 years; trust + sandbox is hard |
| **Notion Templates Gallery** | ~30,000 templates | Free templates = top-of-funnel for paid product |
| **agentskills.io directory** | ~500 skills (still scaling) | **CyberOS publishes Vietnamese pack here = early-mover advantage** |

### What's new in 2026 worth tracking

- **Atlassian Rovo** — agentic teammates baked across Jira/Confluence/Bitbucket. Threat to PROJ + KB modules.
- **Sierra (Bret Taylor)** — vertical AI agents for customer service. Threat to CHAT-as-support.
- **Lindy AI / Cognosys** — agent builders for ops automation. Indirect competitors.
- **Devin AI (Cognition)** — autonomous coding agent. Adjacent — could integrate via MCP.
- **Browser Co (Dia, Arc 2.0)** — Atlassian-owned, becoming a browser-native agent OS. Direct threat to the desktop-shell layer.
- **Anthropic Claude Code + MCP** — the model of agentic-CLI-meets-codebase. Worth deep study; CyberOS CLI surface should match its DX.

## 3 — Specific recommendations to push CyberOS-docs deeper

The docs site at `cyberos/website/docs/` is now structurally complete (31 pages, 22 modules, FR/NFR catalogs). Here is the **next-tier feature list** to turn it from "comprehensive reference" into "the canonical CyberOS wiki + roadmap tracker".

### Tier 1 — Critical (next 1–2 sessions)

| # | Feature | Why | Implementation |
|---|---|---|---|
| 1 | **Live module-status dashboard** | Each module shows real-time pass/fail status, test count, last-deploy date | Per-module status JSON file, Alpine fetches + renders |
| 2 | **Site-wide search** (Lunr.js or Pagefind) | 30,400 lines of docs — Cmd+F per page doesn't scale | Pagefind (Rust, fast, no backend); build at deploy time |
| 3 | **Decision log / ADRs** | Track every architectural decision with date + author + rationale | New `reference/decisions.html` with chronological list + per-ADR pages |
| 4 | **Public changelog** with RSS | Every module ships changes; one place to subscribe | Aggregate per-module CHANGELOGs into `reference/changelog.html` + `feed.xml` |
| 5 | **Cross-link tightening (per-FR anchors)** | Currently FR refs point at module sections; should anchor per-FR | Add `id="FR-{MOD}-{NNN}"` to each FR card in `fr-catalog.html` |

### Tier 2 — Substantial (3–5 sessions)

| # | Feature | Why |
|---|---|---|
| 6 | **Interactive dependency graph** | D3 force-directed graph of all 22 modules + their relationships; click a node to drill in |
| 7 | **API playground** (Stripe-docs-style) | For modules with GraphQL/MCP surface, try requests inline with sample data |
| 8 | **Comparison matrices** | "CyberOS PROJ vs Linear vs Jira" feature-by-feature; ditto for every module |
| 9 | **Migration guides** | "From Slack to CyberOS CHAT" / "From Notion to CyberOS KB" / "From Jira to PROJ" |
| 10 | **Pricing calculator** | Interactive: select modules + seats + tenant size → estimated monthly cost. Includes infra cost ($380/mo internal / $2.2k for 50-tenant) |
| 11 | **Customer stories / case studies** | Start with CyberSkill-itself; add Vietnamese partner case studies as deals close |
| 12 | **Roadmap kanban view** | Drag-drop FR cards across "Backlog / This Sprint / In Progress / Done"; per-phase swimlanes |
| 13 | **SLA/SLO dashboards** | Per-module reliability targets + actuals (when production data exists) |

### Tier 3 — Long-horizon (multi-month)

| # | Feature | Why |
|---|---|---|
| 14 | **Vietnamese full-language version** | Every page bilingual; `Be Vietnam Pro` already in the font stack |
| 15 | **Versioned docs** | v0.1 / v0.2 / v1.0; diff view; deprecated banners |
| 16 | **i18n for English variants** | US English vs International English (date formats, currency) |
| 17 | **Embeddable widgets** | "Embed CyberOS module status on your blog/dashboard" via iframe |
| 18 | **PDF generation** | One-click PDF export per page or for whole site (use Paged.js) |
| 19 | **Video walkthroughs** | One 60-second video per module embedded above the fold |
| 20 | **Interactive tutorials** | Web-based "Try CyberOS" with a sandboxed BRAIN + skills (use Pyodide for the Python tier) |

### Tier 4 — Wiki-style depth

Atlassian-Confluence-class depth requires:

| # | Feature |
|---|---|
| 21 | **Hierarchical TOCs per module** — drill several levels deep without losing context |
| 22 | **Inline annotations / comments** — like Google Docs sidebars; reviewers can leave notes |
| 23 | **"Edit this page" links** — every page links to its source markdown for direct edits |
| 24 | **AI Q&A over the whole docs corpus** — like Notion AI; uses BGE-M3 over the entire site |
| 25 | **Notebook-style live examples** — embed runnable Python/JS in skill explanations |
| 26 | **Glossary popovers** — hover any glossary term anywhere in the docs to see its definition |
| 27 | **Cross-page breadcrumb trails** — show how related pages link together |
| 28 | **Reading time + difficulty markers** — "This page: 8 min read · Intermediate" |

## 4 — Ecosystem-as-a-Service strategy

The big idea: **CyberOS isn't just a product; it's the substrate for OTHER products.** Same playbook as Salesforce (not just CRM, but a platform for ISVs), Notion (not just notes, but a platform for templates), Shopify (not just storefronts, but a platform for apps).

Five levels of ecosystem productization, in order:

### Level 0 — Internal (today)

CyberSkill uses CyberOS for everything internally. Dogfooding. Bet 4 from the PRD. Status: **shipped for memory/skill/cuo**, in progress for the rest.

### Level 1 — Open-source distribution (next 6 months)

CyberOS is on GitHub. Anyone can clone, run their own instance, contribute modules. Tactics:

- Apache 2.0 license throughout
- One-command install (`curl …/install.sh | bash`)
- Public agentskills.io presence for the cyberskill-vn collection
- Public docs site (cyberos/website/docs/ deployed to docs.cyberskill.world)
- Open RFC process for protocol changes
- Public weekly office hours / community calls
- Public ROADMAP.md updated weekly

This is the credibility play. Without OSS distribution, no developer takes CyberOS seriously as a platform.

### Level 2 — Hosted SaaS (months 6–18)

CyberSkill runs CyberOS for paying tenants. Each tenant gets isolated infra (tenant_id RLS Postgres, tenant-scoped NATS, tenant S3 prefix). Pricing tiers:

- **Free** — 5 seats, 100 MB BRAIN, 50K AI tokens/mo, community support
- **Pro** ($29/seat/mo) — unlimited seats, 5 GB BRAIN, 5M tokens/mo, email support, all P0+P1 modules
- **Enterprise** ($99/seat/mo + setup) — bring-your-own-LLM-keys, dedicated tenant, SSO, audit log retention, SLA, all 22 modules including ESOP+DOC

This unlocks ARR. Vietnam-market launch first (HCMC tech scene, then HN), then SEA expansion (Singapore, Indonesia, Thailand, Philippines).

### Level 3 — Marketplace (months 12–24)

3rd parties publish skills + module integrations to the CyberSkill marketplace. Tactics:

- Skill publish workflow: `cyberos-skill publish` pushes to `agentskills.io/cyberskill/<author>/<skill>`
- Revenue share (70% to skill author, 30% to CyberSkill) for paid skills
- Marketplace UI in the docs site at `marketplace.cyberskill.world`
- Curated "Vetted by CyberSkill" badge for security-reviewed skills
- "Built on CyberOS" co-marketing

The marketplace converts CyberOS from a product into a **platform**. This is what Salesforce did in 2005 with AppExchange — and 21 years later it's still the moat.

### Level 4 — Vertical packs (months 18–36)

Beyond Vietnamese skills, build complete vertical packs:

- **cyberskill-vn** (already shipping) — VN compliance, e-invoice, banking, identity, legal
- **cyberskill-sg** — Singapore tax (IRAS), local bank APIs, PDPA, ACRA filings
- **cyberskill-id** — Indonesia (BPJS, NPWP, OJK compliance)
- **cyberskill-th** — Thailand (RD VAT, PDPA-Thailand)
- **cyberskill-eu** — EU compliance (GDPR-native, eIDAS DOC integration, EU AI Act helpers)
- **cyberskill-us** — US compliance (SOC 2 reports, HIPAA helpers, state tax)
- **cyberskill-hr** — HR-specific (US W-2, EU contracts, VN BHXH)
- **cyberskill-legal** — Legal practice (contract review, litigation tracking, billable hours)
- **cyberskill-accounting** — Accounting (GAAP/IFRS reports, audit trail, year-end close)

Each vertical pack is a saleable product on top of the base CyberOS. Margins: 70%+ since the base is open-source.

### Level 5 — Ecosystem-as-a-Service (months 24+)

The endgame: **sell the CyberOS framework itself to enterprises** who want their own branded internal-ops platform.

- "Acme Corp Operating System, powered by CyberOS"
- Enterprise pays CyberSkill to deploy, customize, and operate a private-cloud or on-prem CyberOS instance
- White-label everything (logo, colors via design system, custom modules)
- ISVs publish into the enterprise's private marketplace, not the public one
- Margins: 80%+ on multi-year contracts; recurring services revenue stacks

This is the Confluent / Databricks / Snowflake playbook applied to agentic ops. CyberSkill becomes the consultancy AND the platform — exactly the position your 2020 charter aimed for.

### Comparative positioning

| Dimension | Microsoft 365 / Google Workspace | Salesforce | Notion | Linear | **CyberOS** |
|---|---|---|---|---|---|
| Horizontal vs vertical | Horizontal | Vertical (CRM-first) | Horizontal (KB) | Vertical (PM) | **Horizontal (ops) + vertical packs** |
| Closed vs open | Closed | Semi-closed (AppExchange) | Closed | Closed | **Open standard + Apache 2.0 base** |
| AI-native | Bolted on (Copilot) | Bolted on (Einstein) | Bolted on (Notion AI) | Native-ish | **Agentic substrate from day one** |
| Regional moat | None | Localized regions | None | None | **Vietnamese-first, then SEA** |
| Marketplace | Yes (50K apps) | Yes (7K apps) | Yes (30K templates) | No | **Planned (agentskills.io citizen + own marketplace)** |
| Open audit chain | No | No | No | No | **Yes (MMR + STH on every action)** |

CyberOS's defensible position: **the only platform that's agentic-native + open-standard + audit-chained + regionally-localized**. None of the giants have all four.

## 5 — Concrete next-session priorities

Three actionable next steps, in order:

### Session 1 — Push the docs site to public-ready

1. Wire site-wide search (Pagefind — 30 minutes; Rust-fast; builds at deploy time)
2. Add per-FR anchors in fr-catalog (cross-link tightening)
3. Add decision log + RSS-able changelog page
4. Polish remaining Tailwind utility colors to match Umber/Ochre tokens
5. Deploy to `cyberskill.world/docs` (Cloudflare Pages or GitHub Pages, either works)
6. Announce on LinkedIn + Vietnam dev communities

### Session 2 — Begin the AUTH module

AUTH is the keystone for everything else. Building it unlocks: AI Gateway, MCP Gateway, OBS, every P1 module. The docs already specify the design. Build it:

- Postgres-backed identity service (Rust or Python, your call)
- JWT RS256 with tenant_id claim
- OAuth 2.1 + RFC 7636 PKCE
- WebAuthn L3 for MFA
- RBAC with role catalogue per PRD §8.6.1
- Audit log integration (every auth decision → memory audit chain)

### Session 3 — Comparison matrices + migration guides

The fastest demand-generation play:
- "CyberOS PROJ vs Linear" — feature table + migration script
- "CyberOS CHAT vs Slack" — feature table + import tool
- "CyberOS KB vs Notion" — feature table + import tool

These pages bring search traffic (everyone Googling "Linear vs alternative", "Notion alternative", etc. lands on CyberOS docs).

## 6 — What success looks like

12-month markers if this strategy works:

- **3 months**: docs site live publicly; agentskills.io listing live; LinkedIn/Vietnam tech community awareness; first 100 OSS users
- **6 months**: AUTH + AI Gateway + MCP Gateway + OBS + CHAT shipped; 10 OSS contributors; 1,000+ docs site weekly visitors
- **9 months**: PROJ + TIME + CRM + KB + HR shipped; SaaS tier launched (Free + Pro); first 50 paying tenants in Vietnam
- **12 months**: REW + LEARN + EMAIL shipped; 500+ paying tenants; ARR ≥ $500K; first enterprise customer signed
- **18 months**: 22-module catalog complete; ARR ≥ $1.5M (HoldCo flip trigger per PRD §1.3); marketplace launched with 50+ third-party skills
- **24 months**: First white-label enterprise deal (Level 5 — ecosystem-as-a-service); CyberSkill team grown to 20–30; SEA market expansion underway

This is the ambition. The architectural substrate (memory/skill/cuo + docs site) is in place. The remaining 19 modules are designed. What's left is execution discipline + distribution.

## 7 — Risks worth pre-empting

| # | Risk | Mitigation |
|---|---|---|
| 1 | **Anthropic deprecates or restructures Agent Skills spec** | Tracks the open agentskills.io spec; contribute upstream to have voice in governance |
| 2 | **OpenAI / Microsoft / Google build a competing "agentic OS"** | Differentiate on: open + regional + audit-chained + multi-vendor. They cannot copy all four. |
| 3 | **CyberSkill team can't ship 19 more modules in 18 months** | Modular ownership (Bet 6); each module is one owner; hiring pace per PRD §1.3 (10→12→14→16→20 over 18 months) |
| 4 | **Vietnamese market too small to justify the investment** | Vietnam is the wedge; full TAM is global. Vertical packs unlock global pricing on local content. |
| 5 | **Open-source contributors fork CyberOS away from CyberSkill** | Standard OSS playbook: trademark "CyberOS" name; CyberSkill keeps consultancy + hosted SaaS + private marketplace as commercial moat |
| 6 | **EU AI Act compliance becomes more onerous than expected** | REW + LEARN designed for Annex III §4 from day one; head-start vs competitors retro-fitting |
| 7 | **AGI accelerates faster than CyberOS can ship** | The substrate stays valuable regardless of model capability; memory + audit + capability sandbox + Vietnamese localization don't go away |

## 8 — Closing summary

CyberOS is at an unusual moment. The architectural substrate is real. The Vietnamese-market wedge is shipping. The Anthropic Agent Skills open standard is settling. The competition is bolt-on AI; CyberOS is agent-native from day one.

The next 12 months are about **distribution, not architecture**. Ship the docs publicly. Ship AUTH so the rest of the modules can land. Ship vertical packs. Build the marketplace. Start the Level 5 enterprise conversations early — they take 6–9 months to close.

CyberSkill the consultancy becomes CyberSkill the platform. The Vietnamese tech scene gets an internationally-credible product company headquartered in HCMC. CyberOS becomes the substrate other Vietnamese (then SEA, then global) businesses run their agentic ops on.

That is the bet.

---

*Strategic recommendations prepared 2026-05-14 alongside the CyberOS docs site Wave 1 upgrade. Companion document to `../website/docs/index.html`. For roadmap detail see `../website/docs/architecture/milestones.html`.*
