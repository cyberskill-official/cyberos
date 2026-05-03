---
title: "GTM — marketing site at cyberos.world + Trust Center publish + content + launch playbook"
author: "@stephen-cheng"
department: marketing
status: ready_for_review
priority: p1
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: full_stack
eu_ai_act_risk_class: limited
target_release: "P4 / 2028-Q3"
client_visible: true
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Build the **marketing site + Trust Center** at `cyberos.world` — the public surface where prospects discover, evaluate, and start trial of CyberOS. Three asset clusters: (1) **marketing pages** (homepage, product, pricing, customer stories, blog, contact, careers) built as a static-site generated stack (Astro + MDX) with Vietnamese + English locales; (2) **Trust Center** at `cyberos.world/trust` publishing the canonical compliance posture: SOC 2 Type I report (under NDA), ISO/IEC 27001 certificate, sub-processor list, security overview, DPIA summary, vulnerability disclosure policy, status page link, audit-log architecture overview — most of these auto-generated from FR-CP-001/002/003/004/005 outputs; (3) **launch playbook artefacts**: launch sequence checklist, partner-readiness pack (pitch decks, integration docs, co-marketing templates), sales enablement (battlecards, ICP definition, scripted demo flow), founder-led PR plan (Product Hunt, Hacker News, TechCrunch press kit, Vietnamese tech press, Singapore startup ecosystem). The marketing site is the front door; the Trust Center is what the security-conscious prospect needs to complete a security review without contacting CyberSkill; the launch playbook is what turns "we're live" into "first paying customer". This FR is the only marketing-shaped FR in the entire backlog because most of CyberOS is the platform itself; this FR is the platform meeting the market.

## Problem

PRD §14.5.1 P4 entry-gate criterion includes "marketing site live in vi-VN + en-US" + "Trust Center publishes the SOC 2 Type I report (NDA-gated) + ISO 27001 certificate + sub-processor list" + "first 3 paying customers acquired via the public site (not founder-network)". PRD §2.5 anti-positioning: "Not a marketing-first company — but the marketing surface must be honest, sharp, and developer-credible".

Three failure modes if the marketing surface is built poorly:

- **Honest-but-bland.** A site that's compliance-perfect but reads like a vendor brochure. Mitigation: founder-voice copy + concrete examples + showing real product.
- **Trust Center that's just lip service.** Posting "we're SOC 2 compliant" without the artefacts is what most SaaS does. CyberOS Trust Center auto-generates the actual content from the compliance plane.
- **Launch sequence collapses.** Launching to crickets — no warm lead funnel, no PR amplification, no inbound playbook. Mitigation: pre-build the launch playbook before "go live"; rehearse internally.

## Proposed Solution

### Marketing site (Astro + MDX)

**Stack.**

- Astro static-site generator + MDX for blog content + Tailwind CSS using FR-DESIGN-001 tokens (CyberOS design system extended for marketing aesthetic).
- Hosted on Cloudflare Pages with edge-caching globally; tens of milliseconds to first byte everywhere.
- vi-VN + en-US locales with sub-path routing (`/vi/`, `/en/`); Vietnamese is default for visitors from VN GeoIP.
- Image optimisation via Cloudflare Image Resizing.
- Open Graph + Twitter Card metadata for every page.
- Schema.org JSON-LD for `Organization`, `Product`, `Article` (blog).
- robots.txt + sitemap.xml.

**Pages.**

- **Homepage** (`/`): hero ("CyberOS — the AI-native operations platform for boutique consultancies"), 3 value props, "see it in action" demo video (silent, 90s), social proof carousel (logos of pilot customers + CyberSkill itself), CTA "Start free trial" → cyberos.world/start.
- **Product** (`/product`): scrollytelling through the 22 modules grouped by use-case (run a project, manage a team, close the books, talk to clients). Each module shown with a screenshot + 1-paragraph description. Anchored ToC.
- **Pricing** (`/pricing`): T1 / T2 / T3 plan tiers (FR-BILL-001) with feature comparison table; FAQ; "calculate my monthly cost" interactive. Vietnamese pricing in VND with VAT note for vn-shard tenants.
- **Customer stories** (`/customers`): one per pilot customer; format: challenge → CyberOS solution → outcome with metrics. Founder-quoted. Each story has a "use these" badge for tenants who agree to be featured.
- **Blog** (`/blog`): MDX-authored posts; founder + team contributors; topics: building CyberOS in public, AI-for-operations patterns, multi-tenant + residency lessons, Vietnamese B2B SaaS playbook, customer interviews.
- **Contact** (`/contact`): contact form + Calendly embed for founder office hours; office hours weekly.
- **Careers** (`/careers`): roles + culture deck + Vietnamese remote-first context. Auto-syncs with FR-HR-001 open-roles list when available.
- **About** (`/about`): founder story, team, mission, "why we built this".
- **Legal**: ToS, Privacy, Cookies, AUP, DPA-as-a-PDF.

**Performance.**

- Core Web Vitals: LCP ≤ 1.5s, INP ≤ 100ms, CLS ≤ 0.05 — at p75.
- Lighthouse score: ≥ 95 in all four categories.
- TTI: ≤ 2s.

**SEO.**

- Per-page meta tags + OG + structured data.
- Vietnamese-keyword research baked into Vietnamese-locale content.
- Internal linking strategy.
- Sitemap submission to Google Search Console + Bing Webmaster.

### Trust Center (`/trust`)

**Auto-generated content (from compliance plane).**

- **Security overview** (markdown rendered from a template + FR-CP-001 metadata): high-level architecture, data residency, encryption-at-rest + in-transit, key management, MFA enforcement.
- **Compliance certifications**: ISO/IEC 27001 certificate (PDF embed); SOC 2 Type I report (NDA-gated download — visitor requests access, receives DocuSign NDA, signs, gets PDF).
- **Sub-processor list**: rendered from FR-CP-004's sub-processor table; "subscribe to changes" form (email opt-in for change notifications).
- **DPIA summaries**: GDPR Article 35 DPIAs (PDPL DPIAs equivalent) — public-safe summaries (full versions DPO-only).
- **Audit-log architecture**: 1-page explainer of FR-AUTH-002's Merkle-chained audit + how customers can request their own log slice.
- **Vulnerability disclosure policy**: HackerOne-style; PGP key + email; 90-day disclosure window; bounty (eventually).
- **Penetration test attestation**: latest pentest summary (auto-generated; full report under NDA).
- **Status page link**: cyberos.statuspage.io or self-hosted.
- **Data residency map**: visual showing 4-shard topology (vn/sg/eu/us) + which jurisdictions land where.
- **Privacy policy**: full version + summary version.
- **AI usage policy**: how AI is used + Article 50 transparency + persona-version + skill-version surface explanation.

**Tenant-specific Trust Center pages.**

Each tenant gets `cyberos.world/trust/<slug>` with the platform-level Trust Center extended with:
- Tenant-specific: their data residency, their sub-processor list (= platform list scoped), their DPO contact.
- Tenant-customisable: "About <Tenant>" section (their compliance posture in their own words).

### Launch playbook artefacts

Stored at `OUTPUTS/launch-playbook/`:

- **Launch checklist** (Markdown): 80-item checklist of pre-launch + day-of + post-launch tasks; T-30 / T-14 / T-7 / T-0 / T+1 / T+7 / T+30 milestones.
- **Pitch decks**: 3 versions — partner pitch (15 slides), customer pitch (10 slides), investor pitch (12 slides).
- **Sales enablement**: ICP definition, battlecards vs Notion + Asana + Monday + Linear + custom-built consultancy stacks.
- **Demo script**: 20-minute scripted demo flow for the founder; key moments highlighted (CUO interaction, BRAIN cross-module retrieval, multi-tenant residency demo).
- **PR plan**: Product Hunt launch playbook, Hacker News launch playbook, press release template, Vietnamese tech press contact list, Singapore startup ecosystem contacts (e83, Tech in Asia, etc.).
- **Co-marketing templates**: partner blog post template, joint webinar deck template, joint case study template.
- **Onboarding playbook**: first-3-week customer success playbook (post-trial-signup).

## Out of Scope

- Paid ad campaigns + ad creative (handled in FR-GTM-003 P4 follow-up if needed).
- Gated whitepapers / lead magnets (Phase 2).
- A/B testing infrastructure on marketing site (Phase 2).
- Customer testimonial video production (Phase 2).
- Conference booth + event budget (Phase 2).
- Multi-language beyond vi-VN + en-US at MVP.

## Dependencies

- FR-DESIGN-001 (design tokens; marketing aesthetic extended).
- FR-CP-001/002/003/004/005 (compliance plane outputs feed Trust Center).
- FR-AUTH-002 (audit chain — referenced in Trust Center).
- FR-TEN-003 (per-tenant Trust Center customisation surface).
- FR-BILL-001 (pricing page sourced from plan-tier definitions).
- FR-INV-002 (Vietnamese e-invoice mention in vi-VN pricing page).
- FR-PORTAL-001 (customer stories may include PORTAL screenshots).
- FR-GENIE-001 (CUO/CXO references in product page).
- FR-HR-001 (Careers page open-roles auto-sync).
- DEC-052 Trust Center is public.

## Constraints

- **Founder voice mandatory in copy review.** Every page passes founder review; no agency-tone "we are passionate about innovation" copy.
- **No fake social proof.** Logos only from confirmed customers + with permission.
- **No false certifications claimed.** Trust Center asserts only what's actually certified.
- **vi-VN parity at launch.** Not "English now, Vietnamese later".
- **No third-party analytics scripts beyond first-party (Plausible-style).** Privacy-by-default.
- **Cookie banner is honest.** Strictly-necessary cookies only by default; no dark patterns.
- **AI in marketing copy must be disclosed.** If any blog post or page is AI-drafted, the byline reflects that.
- **Site loads without JavaScript** for the homepage + pricing + Trust Center key pages — progressive enhancement.

## Compliance / Privacy

- **GDPR Article 13:** privacy policy at point of collection; cookie banner.
- **PDPL Decree 13/2023:** vi-VN privacy policy + DPO contact.
- **EU AI Act Article 50:** if any visitor-facing AI is on the site (e.g. chat support widget), it's clearly labelled.
- **Cookie compliance:** strictly necessary by default; opt-in for analytics; opt-out for sales-tracking.
- **CCPA + DMA + e-Privacy Directive:** acknowledged + summarised in privacy policy.
- **WCAG 2.1 AA** accessibility:
  - Alt text on every image.
  - Keyboard navigation works.
  - Color contrast ≥ 4.5:1.
  - Skip-to-content link.
  - Lang attribute correct per locale.
  - aria-* attributes correct.

## Risk Assessment (AI-emitting features)

- **EU AI Act risk class:** `limited` — visitor-facing chat support widget (if shipped) uses CXO read-only patterns from FR-PORTAL-003; Article 50 transparency disclosed at first interaction.
- **AI-drafted blog content:** disclosed in byline; Founder approves before publish; ai_authorship field on each post.

## Vietnamese-locale considerations

- Vietnamese copy authored + reviewed by native speaker; not auto-translated from English.
- Vietnamese SEO keyword research; vi-VN-specific long-tail.
- Vietnamese case studies favouring vi-VN customers.
- Be Vietnam Pro typography mandatory in vi-VN locale.
- Vietnamese pricing in VND + VAT note; e-invoice availability flagged.
- Vietnamese press contacts in launch playbook (Tuoi Tre, VnEconomy, ICTNews, GenK, TechSignals).
- Tone: lịch sự + chuyên nghiệp; founder-first ("anh Cheng / Trịnh Thái Anh / nhà sáng lập").

## Scope (acceptance criteria — auditable)

- [ ] cyberos.world live with all listed pages in vi-VN + en-US.
- [ ] Trust Center at /trust live with auto-generated content from FR-CP-001..005.
- [ ] SOC 2 Type I + ISO 27001 certificates accessible (NDA-gated for SOC 2).
- [ ] Sub-processor list rendered from FR-CP-004 source-of-truth; "subscribe to changes" works.
- [ ] Privacy policy + ToS + DPA live in both locales.
- [ ] Per-tenant /trust/<slug> pages live for all active tenants; customisation slot works.
- [ ] Core Web Vitals: LCP ≤ 1.5s, INP ≤ 100ms, CLS ≤ 0.05 at p75.
- [ ] Lighthouse score ≥ 95 across all four categories on key pages.
- [ ] WCAG 2.1 AA passes; pa11y CI test in pipeline.
- [ ] OG + Twitter Card + JSON-LD schemas on every page; tested in Twitter card validator + Facebook debugger.
- [ ] Sitemap.xml + robots.txt live + submitted to Google + Bing.
- [ ] Cookie banner: strictly-necessary by default; opt-in works.
- [ ] Plausible-style first-party analytics live; no third-party trackers.
- [ ] Launch playbook artefacts complete + reviewed: checklist, decks, sales enablement, PR plan, onboarding playbook.
- [ ] Vietnamese tech press list verified + contact attempts logged.
- [ ] Customer stories: 3 published at launch with customer permission.
- [ ] Founder review pass: every page approved by Founder before launch.
- [ ] Pricing page numbers match FR-BILL-001 source-of-truth.

**Gherkin (PRD §19.18).**

```gherkin
Feature: Trust Center auto-updates from compliance plane

  Scenario: Sub-processor list updates after FR-CP-004 source-of-truth changes
    Given sub-processor X is removed from cp.sub_processor table
    When the Trust Center static-site rebuilds (nightly or on-demand)
    Then cyberos.world/trust no longer lists X
    And the previous version is preserved in the change-log section
    And subscribers receive a change notification email within 24 hours

Feature: Vietnamese-default for VN visitors

  Scenario: Visitor from VN GeoIP lands at cyberos.world
    Given a visitor with VN IP and Accept-Language vi-VN
    When they GET cyberos.world
    Then they are redirected to /vi/ (Vietnamese homepage)
    And the page renders in Be Vietnam Pro
    And the language switcher shows "EN" + "VI" with VI selected
    When they click "EN"
    Then they are redirected to /en/ and the language preference is stored in cookie

Feature: SOC 2 Type I requires NDA before download

  Scenario: Visitor requests SOC 2 report
    Given a visitor on cyberos.world/trust
    When they click "Download SOC 2 Type I report"
    Then they're prompted to fill: name + email + company + use case
    And on submit, a DocuSign envelope is sent for NDA signing
    And the SOC 2 report is automatically delivered to the same email after NDA signing
    And the request is logged in cp.report_request for audit
```

## Success Metrics

- Marketing site launch traffic: ≥ 5,000 unique visitors in first 30 days.
- /start signup conversion: ≥ 2% of unique visitors.
- First 3 paying customers acquired via the public site within 90 days.
- Trust Center visit-to-trial-signup conversion: ≥ 5% (Trust Center is mid-funnel).
- vi-VN traffic share: ≥ 40% of total in first 90 days.
- Lighthouse score sustained ≥ 95 over time.
- Zero accessibility regressions caught in pa11y CI.

## Open Questions

- **OQ-GTM-001-01.** Should we offer a public sandbox tenant for prospects to "try before signup"? Default: no at MVP (signup is short, signup-then-cancel is the better flow); revisit if conversion data suggests otherwise.
- **OQ-GTM-001-02.** Should the Trust Center include the tenants' own SOC 2 / ISO certifications when they have them, on the per-tenant /trust/<slug> page? Default: yes if the tenant opts in; show their certifications + their data residency.
- **OQ-GTM-001-03.** Should we run a Product Hunt launch on Day 0 or wait until 30+ days post-launch with traction? Default: Day 0 — Product Hunt rewards first-day traction.

## References

- PRD §14.5.1 P4 entry-gate; PRD §2.5 anti-positioning.
- SRS Decisions Log: DEC-052.
- FR-DESIGN-001, FR-CP-001/002/003/004/005, FR-AUTH-002, FR-TEN-003, FR-BILL-001, FR-INV-002, FR-PORTAL-001, FR-GENIE-001, FR-HR-001.

---

*ai_authorship: co_authored — drafted by Claude Cowork on 2026-05-03.*
