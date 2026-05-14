# Research-Mode brief for CyberOS pre-lock review

**Use:** Paste the **Prompt** below into Claude Chat's Research Mode. Attach the **Input artefacts** as files. Research Mode will crawl the live docs site and the attached materials and return a single comprehensive review.

**Why this shape:** Zipping `website/docs/` (~3.5 MB of static HTML + 1.7 MB Pagefind index + ~30 KB images) and feeding it as one blob defeats Research Mode's strengths — it does shallow per-file reads on big zips and can't actually exercise the interactive elements. Instead we feed Research Mode **(a)** a small bundle of curated source-of-truth markdown (~250 KB total) so it knows the architecture without ambiguity, and **(b)** the deployed URL so it crawls and renders the actual UX a human will see.

---

## Part 1 — The Prompt to paste

```
ROLE
You are a senior product-and-engineering reviewer doing a final pre-launch
audit on CyberOS, an AI-native internal-operations platform built by
CyberSkill (a 10-person Vietnam-based software consultancy). I'm about to
LOCK the strategy and start building 19 unbuilt modules over the next
24 weeks. Before I lock, I want one comprehensive, opinionated review.

CONTEXT
- The CyberOS docs site is the single source of truth. Deployed at:
    https://docs.cyberskill.world/                (target custom domain)
    https://5cc09eb6.cyberos-docs.pages.dev/      (current preview — use this URL for crawls)
- 32 HTML pages: index, 23 module pages (3 shipped: BRAIN/Skill/CUO;
  19 designed-but-unbuilt), 4 architecture pages, 4 reference pages.
- The strategy doc, the build-readiness plan, the FR-authoring workflow,
  and the design system are attached. Read them all before you crawl.
- The PRD/SRS markdown still exists in the repo but FR identifiers have
  been stripped. The docs site is now authoritative; PRD/SRS are
  narrative-only background.

YOUR TASK
Produce a single review document covering EIGHT dimensions, in this order:

1. STRATEGIC COHERENCE
   - Does the "ecosystem-as-a-service" thesis (Strategy doc §4) hold up?
   - Are the five productization levels (Internal → OSS → Hosted SaaS →
     Marketplace → Vertical Packs → Ecosystem-as-a-Service) sequenced
     correctly? Where does the bet seem most fragile?
   - The 12-month markers (3mo / 6mo / 9mo / 12mo / 18mo / 24mo) —
     realistic given a 10-person team? What's missing?
   - The Vietnamese-market wedge — does it actually compound into global
     reach, or does it trap the company regionally?

2. ARCHITECTURE & MODULE BOUNDARIES
   - The three-layer BRAIN (FS + pgvector + archival corpus) — sensible?
     Where does it break at scale?
   - The CUO router as the single Genie persona with 10 C-level
     sub-personas hot-loaded via Anthropic Agent Skills — is this
     architecturally cleaner than multi-persona alternatives, or is it
     hiding complexity that will leak in P1+?
   - The 22-module decomposition — any modules that should be merged?
     Split? Reordered in the build sequence?
   - Cross-cutting infrastructure (AUTH / AI Gateway / MCP Gateway / OBS
     / GraphQL Federation / NATS) — does AUTH really belong as P0 #1,
     or is one of the others a better unlock?

3. SPEC QUALITY (per-module pages)
   - For each of the 22 module pages, check: 5W1H2C5M completeness,
     architecture diagram presence, data model ERD, API surface
     (GraphQL/MCP/CLI), key flows, dependencies, compliance scope,
     risk entries, KPIs, RACI, planned CLI surface.
   - Which modules are spec-complete, which are skeletal? Rank.
   - Are there obvious contradictions or gaps that will trip up an
     engineer reading the page cold?

4. UX & VISUAL DESIGN
   - The design system is Vietnamese-first (Be Vietnam Pro), Liquid
     Glass default (Part 21), Umber + Ochre anchors.
   - Inspect the live site for: brand consistency, typography rhythm,
     spacing, code blocks, Mermaid renders, TOC behavior, anchor links,
     focus states, mobile responsiveness, print stylesheet, FOUC,
     accessibility (WCAG 2.2 AA + APCA Lc ≥ 75 body).
   - List every visible glitch with page + severity (Critical / High /
     Medium / Low).

5. INFORMATION ARCHITECTURE
   - Is the site navigable? Can a first-time visitor reach any module
     spec in ≤ 3 clicks?
   - Is Pagefind search useful? Test 10 representative queries.
   - Are the architecture pages cross-cutting enough or do they repeat
     content from the module pages?
   - Are the reference pages (FR-catalog stub, NFR catalog, glossary,
     risk register) appropriately weighted?

6. COMPLIANCE & RISK POSTURE
   - The three-ring compliance model (Vietnam / Cross-border /
     International standards). Is the timeline (P0 → P3 → P4) tight
     enough? Loose enough?
   - The risk register — comprehensive? What's missing?
   - GDPR / Vietnamese PDPL / Singapore PDPA mapping — does the spec
     hold against actual regulator language? Flag anything that looks
     vulnerable.
   - EU AI Act Annex III §4 mapping — is REW + LEARN correctly classified?

7. GO-TO-MARKET POSTURE
   - The dogfooding bet ("internal first, external P4") — sound?
   - The Vietnamese-market launch sequence (HCMC → HN → SG → ID → TH →
     PH) — realistic?
   - The marketplace play vs. Salesforce AppExchange / Atlassian
     Marketplace / Notion templates — defensible?
   - Pricing model (Free / Pro / Enterprise + per-module vertical packs)
     — comparable to anchor competitors?

8. CONCRETE NEXT-7-DAY RECOMMENDATIONS
   - Five concrete things I should do THIS WEEK before locking the
     strategy. Ranked by leverage.
   - Five things I should explicitly DEFER even if tempted.

CONSTRAINTS ON YOUR REVIEW
- Be opinionated. Don't hedge.
- Cite specific page URLs + section anchors when you make a claim.
- When you flag a defect, propose a concrete fix.
- When you flag a gap, propose what to fill it with.
- Compare to anchor competitors by name (Linear, Slack, Notion, Carta,
  Stripe Billing, Auth0, LiteLLM, Honeycomb, etc.) — don't just say
  "industry standard".
- Length: as long as it needs to be. Plan on ~6,000–10,000 words.
- Output in markdown. Use H2 for each of the 8 dimensions, H3 for
  sub-topics, tables where useful.
- End with a one-page executive summary suitable for forwarding to a
  Series-A investor.

DELIVER
One markdown document. No interim updates needed.
```

---

## Part 2 — Input artefacts to attach

Attach these **eight files** in the Research Mode chat. Total ≈ 250 KB.

| # | File | Why it's in the bundle |
|---|---|---|
| 1 | [`strategy/CYBEROS_STRATEGY.md`](../strategy/CYBEROS_STRATEGY.md) | The strategic thesis — the bet, the competitive landscape, the 12-month arc. Required for Dimensions 1 + 7. |
| 2 | [`docs/AUDIT_AND_PLAN_2026_05_14.md`](AUDIT_AND_PLAN_2026_05_14.md) | The current build-readiness state. Required for Dimensions 2 + 3 + 8. Includes the per-module table the reviewer will fact-check against. |
| 3 | [`docs/FR_AUTHORING_WORKFLOW.md`](FR_AUTHORING_WORKFLOW.md) | How the team plans to actually build forward (one FR per task per PR via the fr-author skill). Required for Dimension 8. |
| 4 | [`README.md`](../README.md) | Umbrella view — three shipped modules + 19 designed. Sanity-check ground truth. |
| 5 | [`memory/README.md`](../memory/README.md), [`skill/README.md`](../skill/README.md), [`cuo/README.md`](../cuo/README.md) | The three shipped modules' status. Anchors the reviewer's mental model. |
| 6 | [`services/auth/RFC.md`](../services/auth/RFC.md) | The first RFC for the keystone P0 module. Template for the other 18. Required for Dimensions 2 + 3 + 8. |
| 7 | [`docs/prd/PRD.md`](prd/PRD.md) | Long-form narrative spec (~4,287 lines). Background reference for Dimensions 2 + 6. Reviewer can grep into it but shouldn't read end-to-end. |
| 8 | (optional but ideal) [`design-system/DESIGN.md`](../../design-system/DESIGN.md) | The brand doctrine — Umber/Ochre anchors, Liquid Glass Part 21, Vietnamese-first commitment, voice axes. Required for Dimension 4. The design-system repo is a sibling — attach the file directly. |

### Shell command to bundle them

```bash
cd /Users/stephencheng/Projects/CyberSkill/cyberos

mkdir -p /tmp/cyberos-research-input
cp strategy/CYBEROS_STRATEGY.md /tmp/cyberos-research-input/01-strategy.md
cp docs/AUDIT_AND_PLAN_2026_05_14.md /tmp/cyberos-research-input/02-audit-and-plan.md
cp docs/FR_AUTHORING_WORKFLOW.md /tmp/cyberos-research-input/03-fr-workflow.md
cp README.md /tmp/cyberos-research-input/04-umbrella-readme.md
cp memory/README.md /tmp/cyberos-research-input/05-memory-readme.md
cp skill/README.md  /tmp/cyberos-research-input/06-skill-readme.md
cp cuo/README.md    /tmp/cyberos-research-input/07-cuo-readme.md
cp services/auth/RFC.md /tmp/cyberos-research-input/08-auth-rfc.md
cp docs/prd/PRD.md  /tmp/cyberos-research-input/09-prd.md
cp ../design-system/DESIGN.md /tmp/cyberos-research-input/10-design-system.md

ls -la /tmp/cyberos-research-input/
du -sh /tmp/cyberos-research-input/
```

Then drag all 10 files into the Research Mode chat alongside the prompt.

---

## Part 3 — How to drive the conversation

1. **First message** (paste-once):
   - The Prompt from Part 1
   - All 10 files attached
   - One line at the end: *"Begin the review."*

2. **Mid-review interaction** — Research Mode will pull up the live URLs itself. You generally don't need to interrupt. Two exceptions where a short follow-up helps:
   - If it asks for clarification on a specific module's intended scope (e.g. "is `REW.pool calculation` deterministic or LLM-assisted?"), answer briefly and let it continue.
   - If it surfaces a finding you want to chase deeper, say *"Expand on §X.Y — give me three sub-findings."*

3. **Closing** — After the document arrives, ask:
   - *"Score the strategy 1–10 on each of the 8 dimensions and aggregate. Justify the lowest score."* — forces a defensible call.
   - *"List the three things in your review you are least confident about."* — surfaces where Research Mode's web-search was thin.
   - *"If I lock the strategy as-is, what is the single most likely failure mode in the next 6 months?"* — the post-mortem-from-the-future question. Most useful answer.

---

## Part 4 — What to do with the review

After Research Mode returns the document:

1. Save it as `docs/RESEARCH_REVIEW_2026_05_14.md` in the repo.
2. Walk through the **§8 (next-7-days)** list with me — we'll convert each item into a TaskCreate.
3. Walk through **§3 (spec quality)** — for each module flagged "skeletal", we decide: defer-the-page, queue-an-RFC, or rewrite-spec-on-the-page.
4. Walk through **§4 (UX)** — every Critical + High glitch goes into a fix sprint before the public-launch redeploy.
5. **Lock the strategy** after the §1-§7 dimensions clear. §8 just becomes the working backlog.

---

## Part 5 — Why NOT to attach the docs HTML

The docs HTML pages are derivative — they render the same source-of-truth that's already in the attached markdown. Feeding them directly would:

- **Waste tokens** on Tailwind utility classes, Pagefind chunks, inline SVGs, and embedded fonts (~3.5 MB of mostly-useless tokens).
- **Confuse the reviewer** — the same module is described in both the HTML and the markdown; the reviewer may flag "inconsistency" when really it's just rendering drift.
- **Miss the visual UX** — even with HTML, Research Mode reads source, not rendered pages. The visual review (Dimension 4) requires the live URL crawl.

The live URL gives the reviewer the rendered experience. The 10 markdown files give the canonical content. Together that's the full picture without bloat.

---

*Ready when you are. Once Research Mode returns the review, paste it back to me and we'll plan the post-lock sprints.*
