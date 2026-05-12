---
fr_id: FR-001
title: CyberSkill landing-page MVP
profile: solo
project: 2026-05-12-cyberskill-landing-page
status: draft
eu_ai_act_risk_class: not_ai
client_visible: true
authority: human-confirmed
acceptance_criteria:
- Lighthouse mobile + desktop ≥ 90 on performance, accessibility, SEO
- Contact form posts to info@cyberskill.world and confirmed delivered
- English + Vietnamese locales selectable via /en and /vi URL prefixes
- Above-the-fold CTA visible on iPhone SE viewport (375 × 667)
- ≥ 2 case studies published (real past projects)
- Founder bio + slogan 'Turn Your Will Into Real' on /about
- SEO meta tags + Organization + Person JSON-LD structured data
- Plausible analytics deployed; first-week visitor count tracked
- Deployed to fly.io OR Vercel under a CyberSkill-controlled domain
- TTFB < 200 ms p95 from Singapore + US-East + EU-West
task_index:
- id: FR-001-T-01
  title: Pick stack + scaffold repo
- id: FR-001-T-02
  title: Write English landing-page copy
- id: FR-001-T-03
  title: Translate copy to Vietnamese
- id: FR-001-T-04
  title: Design system + page layout
- id: FR-001-T-05
  title: Contact form + email delivery
- id: FR-001-T-06
  title: SEO + structured data + analytics
- id: FR-001-T-07
  title: Lighthouse pass + a11y audit
- id: FR-001-T-08
  title: Deploy + DNS + smoke test
---


# FR-001 — CyberSkill landing-page MVP

## Problem statement

CyberSkill has been a Vietnam-based software consultancy since 2020 with the slogan "Turn Your Will Into Real". Today there's no public-facing site representing the company; prospects find Stephen via LinkedIn or word-of-mouth, which caps reach to his network. To scale globally — the founder's stated goal — the company needs a credible English-language landing page that converts evaluating prospects into "let's talk" replies, plus a Vietnamese locale for the home market.

This FR captures the work to ship the **first** publishable version. Subsequent iterations (blog, services pages, case-study deep dives) belong in follow-up FRs.

## Users

- **Primary** — prospective clients (English-speaking) evaluating CyberSkill for partnership on a software project
- **Secondary** — prospective hires considering joining CyberSkill (engineers reading the case studies, looking at the founder bio)
- **Tertiary** — existing partners looking up CyberSkill contact details or sharing a link

## Success metrics

- **Lighthouse ≥ 90** on perf + a11y + best-practices + SEO across mobile + desktop
- **Contact form conversion ≥ 2 %** of unique visitors (week-over-week tracked via Plausible Goals)
- **Bilingual** — `/en` and `/vi` URL prefixes; default route detects browser `Accept-Language`
- **TTFB p95 < 200 ms** from Singapore, US-East, EU-West (CyberSkill's three primary target markets)
- **Calendar deadline**: 3 weeks from project kick-off

## Scope

In scope (this FR):
- Static marketing site, 5 sections: hero / what-we-do / case-studies / founder / contact
- Two locales: English + Vietnamese
- Contact form delivering to `info@cyberskill.world`
- SEO meta + JSON-LD structured data
- Plausible analytics
- Deployment to Vercel (primary) with fly.io as documented fallback

Out of scope (follow-up FRs):
- Blog / content marketing engine
- Per-service deep-dive pages (e.g. `/services/ai-automation`)
- Multi-page case studies with screenshots + metrics
- A login area or client portal
- A pricing page
- Email-marketing capture (newsletter signup)
- Multi-language beyond EN + VI

## Risks

- **R1 — Translation quality.** Vietnamese copy quality depends on Stephen's review; budget ≥ 1h of native-speaker review time per T-03. Mitigation: ship EN-only on day 1 if VI isn't ready by deadline.
- **R2 — Email deliverability.** Contact-form submissions hitting spam folders. Mitigation: use Resend or Postmark with proper SPF + DKIM at `cyberskill.world`. Test with mail-tester.com.
- **R3 — Lighthouse perf regression after analytics.** Plausible script can shave 2-3 points if loaded synchronously. Mitigation: load async with `defer` + monitor via the CI gate from T-07.

## EU AI Act classification

Risk class: **not_ai**. This is a static marketing site with no AI features in the published artefact. No automated decisions about users; no profiling. The contact form sends raw text to an inbox; humans read and reply. No risk-tier obligations under EU AI Act §16.

(Future-state: if the site adds a chatbot or AI-powered FAQ search, that becomes a separate FR with `eu_ai_act_risk_class: limited` and a new HITL gate on user input.)

## Total estimated effort

- Human: **21.5 hours** (T-01 + T-02 + T-03 + T-07 + T-08 + half of T-04)
- AI agent: **30,000 tokens** (T-04 + T-05 + T-06)
- Estimated calendar: 3 weeks at part-time pace; 1 week if focused

## FR-001-T-01 — Pick stack + scaffold repo

Decide on the framework + hosting + i18n approach for the landing page,
then scaffold the repo. Recommended stack: Astro 5 (static-first, fast
hydration, native i18n via /[lang]/ routes) + Tailwind v4 for styling +
Plausible script for analytics. Hosting: Vercel for the default deploy
(free tier, instant rollbacks, Astro adapter is stable). Set up the
repo with: src/pages/, src/content/ (markdown content collections for
copy + case studies), src/components/, public/ (static assets), and
tailwind.config + astro.config. Author a brief CONTRIBUTING.md so a
second contributor can pick up later tasks without context.

**Preconditions:**

- none

**Deliverables:**

- Public github.com/cyberskill/landing-page repo at scaffolding-ready state
- astro.config.mjs with i18n = { locales: ['en','vi'], defaultLocale: 'en', routing: { prefixDefaultLocale: false } }
- tailwind.config.js with CyberSkill brand tokens (placeholder palette OK; refine in T-04)
- Empty src/pages/index.astro that renders 'Hello CyberSkill' for the smoke test
- CONTRIBUTING.md (≤ 1 page) covering branch naming, PR template, deploy command

**Acceptance test:**

```shell
cd landing-page && pnpm install && pnpm dev --port 4321 & sleep 5 && curl -s http://localhost:4321/ | grep -q 'Hello CyberSkill' && kill %1
```

```task-meta
sizing: S
dependencies: []
parallelisable: true
assignable_to:
- human
estimated_hours: 2.5
status: draft
runbook_hint: null
```

## FR-001-T-02 — Write English landing-page copy

Write the canonical English copy for all sections of the landing page
and store it as a markdown content collection at
src/content/copy/en/. Sections (in vertical scroll order): (1) hero
with slogan 'Turn Your Will Into Real' + sub-headline pitching
CyberSkill as a Vietnam-based AI-native consultancy + primary CTA
'Start a conversation'; (2) what-we-do (3-column: software dev,
AI-native automation, consultancy); (3) case studies (2-3 real past
projects, each 80-120 words with outcome + tech + duration); (4)
founder bio (Stephen Cheng / Trịnh Thái Anh, ~150 words, links to
LinkedIn); (5) contact form. Tone: builder-to-builder, not
consultant-to-client. No em dashes. No AI vocabulary (leverage /
robust / ensure / seamless / etc.). Cite source_ref to PRD §1.1 +
§3.2 (audience + tone) and to memories/preferences/PREF-001-voice-standard.

**Preconditions:**

- FR-001-T-01 done (content collection structure exists)

**Deliverables:**

- src/content/copy/en/hero.md
- src/content/copy/en/what-we-do.md
- src/content/copy/en/case-studies/*.md (≥ 2 files)
- src/content/copy/en/founder.md
- src/content/copy/en/contact.md
- Copy passes `cyberos voice --strict` (no em dashes, no AI vocab)

**Dependencies:**

- FR-001-T-01

**Acceptance test:**

```shell
find src/content/copy/en -name '*.md' | xargs -I {} grep -L '—\|leverage\|robust\|ensure\|comprehensive\|seamless\|delve' {} | wc -l | grep -E '^[5-9]|[1-9][0-9]+$'
```

```task-meta
sizing: M
dependencies:
- FR-001-T-01
parallelisable: true
assignable_to:
- human
estimated_hours: 4.0
status: draft
runbook_hint: null
```

## FR-001-T-03 — Translate copy to Vietnamese

Translate every markdown file under src/content/copy/en/ to Vietnamese
and place under src/content/copy/vi/ with the same filenames. Use
natural Vietnamese marketing voice (not literal translation). Slogan
"Turn Your Will Into Real" → "Biến ý chí của bạn thành hiện thực"
(validate with Stephen — locked in memories/preferences). Per AGENTS.md
§4.2, wrap any external translation-service output in <untrusted_content>
blocks before reasoning over it; review every line manually. Title
attributes + meta descriptions also translated.

**Preconditions:**

- FR-001-T-02 done (English copy finalized)

**Deliverables:**

- src/content/copy/vi/* mirroring the English structure
- Native-speaker review notes recorded as memories/projects/PROJECT-landing-page-vi-review.md

**Dependencies:**

- FR-001-T-02

**Acceptance test:**

```shell
[ $(find src/content/copy/vi -name '*.md' | wc -l) -eq $(find src/content/copy/en -name '*.md' | wc -l) ]
```

```task-meta
sizing: M
dependencies:
- FR-001-T-02
parallelisable: true
assignable_to:
- human
estimated_hours: 3.0
status: draft
runbook_hint: null
```

## FR-001-T-04 — Design system + page layout

Build the visual design system using Tailwind v4 tokens and a small
set of reusable Astro components: Hero, Section, CaseStudy, Bio,
ContactForm, Footer. Colour palette: ink-black background, off-white
text, single accent (recommend a deep teal #0E7C7B). Typography:
Inter for body, JetBrains Mono for code/data accents. Spacing scale:
4-base. Above-the-fold CTA must be visible on iPhone SE (375 × 667);
validate by viewing localhost:4321 in mobile-emulated Chrome devtools.
Use Astro's <Image /> with eager loading for the hero image only;
lazy for everything below the fold.

**Preconditions:**

- FR-001-T-01 done
- Copy structure exists (T-02 can be in-progress)

**Deliverables:**

- src/components/{Hero,Section,CaseStudy,Bio,ContactForm,Footer}.astro
- src/styles/tokens.css declaring CSS variables for palette + spacing
- src/pages/index.astro composing the 5 sections in scroll order
- Mobile viewport screenshot at outputs/landing-page/mobile-hero.png

**Dependencies:**

- FR-001-T-01

**Acceptance test:**

```shell
pnpm build && grep -q 'Start a conversation' dist/index.html
```

```task-meta
sizing: M
dependencies:
- FR-001-T-01
parallelisable: false
assignable_to:
- ai-agent
- human
agent_profile: 'claude-sonnet-4-6, mcp_allowlist: [bash, edit, read]'
estimated_tokens: 12000
estimated_hours: 5.0
status: draft
runbook_hint: null
```

## FR-001-T-05 — Contact form + email delivery

Build the contact form (name / email / message) and wire it to send
submissions to info@cyberskill.world. Use Astro server endpoint at
src/pages/api/contact.ts (POST). Use the host's email service: on
Vercel use Resend (free 100/day, simple API); on fly.io use Postmark
via SMTP. Validate inputs server-side: email regex, name 1-100 chars,
message 10-2000 chars. Honeypot field 'phone' (hidden via CSS) — if
filled, silently accept and discard (anti-spam). Per AGENTS.md §4.2,
wrap incoming form payload in <untrusted_content source="contact-form">
block before any LLM processing (today none; future-state safety
contract). Rate limit: 5 submissions per IP per hour using an in-memory
sliding window on the edge function. Return 200 + thank-you redirect
on success; 4xx + inline error message on validation failure.

**Preconditions:**

- FR-001-T-04 done (ContactForm.astro component exists)
- Email service account exists (Resend or Postmark; record key in 1Password)

**Deliverables:**

- src/pages/api/contact.ts
- Environment vars documented in README.md (CONTACT_TO, RESEND_API_KEY or POSTMARK_TOKEN)
- Tests: tests/contact-form.spec.ts covering valid + invalid + honeypot + rate-limit
- Test message verified delivered to info@cyberskill.world inbox

**Dependencies:**

- FR-001-T-04

**Acceptance test:**

```shell
pnpm test tests/contact-form.spec.ts
```

```task-meta
sizing: M
dependencies:
- FR-001-T-04
parallelisable: false
assignable_to:
- ai-agent
agent_profile: 'claude-sonnet-4-6, mcp_allowlist: [bash, edit, read]'
estimated_tokens: 10000
status: draft
runbook_hint: null
```

### FR-001-T-05-ST-01 — Define form schema (zod + types)

Author the contact-form Zod schema with required fields (name, email, message) and optional fields (company, phone). Export TypeScript type. Cover validation messages in EN + VI.

```subtask-meta
sizing: S
estimated_tokens: 2000
status: draft
```

### FR-001-T-05-ST-02 — POST endpoint /api/contact

Implement Astro server endpoint that validates body against schema, rate-limits per-IP (5/hour), and forwards via Resend / SendGrid to info@cyberskill.world. Honeypot field for spam.

```subtask-meta
sizing: S
estimated_tokens: 3000
status: draft
```

### FR-001-T-05-ST-03 — Frontend form + submit UX

Wire the form component with progressive enhancement (works without JS), live validation, loading state, success/failure toasts. Reset on success.

```subtask-meta
sizing: S
estimated_tokens: 3000
status: draft
```

### FR-001-T-05-ST-04 — Playwright happy-path test

End-to-end test: fill form, submit, assert success toast + assert email delivered (mock provider in test mode).

```subtask-meta
sizing: S
estimated_tokens: 2000
status: draft
```

## FR-001-T-06 — SEO + structured data + analytics

Add SEO meta tags (title, description, og:image, og:type, twitter:card,
canonical, alternates for /en + /vi) to every page via an Astro layout.
Embed JSON-LD structured data: Organization schema for CyberSkill
(name = "CYBERSKILL SOFTWARE SOLUTIONS CONSULTANCY AND DEVELOPMENT
JOINT STOCK COMPANY", legalName, address from PERSON-001 memory,
email, telephone, founders, foundingDate 2020). Person schema for
Stephen on /about. Add Plausible analytics script (data-domain =
cyberskill.world). Generate sitemap.xml + robots.txt at build via
@astrojs/sitemap. All pages must pass schema.org validator and
Google's Rich Results Test.

**Preconditions:**

- FR-001-T-04 done

**Deliverables:**

- src/layouts/Base.astro with full meta + og + json-ld blocks
- Sitemap.xml + robots.txt produced by `pnpm build`
- Plausible script embedded conditionally (only on production builds)
- Validation report: link to Rich Results Test passing on /, /en, /vi, /about

**Dependencies:**

- FR-001-T-04

**Acceptance test:**

```shell
pnpm build && grep -q 'application/ld+json' dist/index.html && grep -q 'plausible' dist/index.html && [ -f dist/sitemap-index.xml ]
```

```task-meta
sizing: M
dependencies:
- FR-001-T-04
parallelisable: true
assignable_to:
- ai-agent
agent_profile: 'claude-sonnet-4-6, mcp_allowlist: [bash, edit, read]'
estimated_tokens: 8000
status: draft
runbook_hint: null
```

## FR-001-T-07 — Lighthouse pass + a11y audit

Run Lighthouse against the production build (locally via `lhci
autorun` or via the GitHub Action). Acceptance: all four scores
(performance, accessibility, best-practices, SEO) ≥ 90 on both
mobile and desktop emulation. Fix any regressions surfaced. Common
hits: image dimensions missing → add width + height attrs; LCP image
not preloaded → add <link rel=preload>; missing alt text → add to
Image components; colour contrast → bump accent against background.
Manual a11y audit: keyboard-only navigation works through every CTA
+ the contact form; aria-labels present on icon-only buttons; focus
indicators visible.

**Preconditions:**

- FR-001-T-04, T-05, T-06 done

**Deliverables:**

- .github/workflows/lighthouse.yml CI gate
- Lighthouse report committed at outputs/landing-page/lighthouse-baseline.json
- Manual a11y notes at outputs/landing-page/a11y-audit.md

**Dependencies:**

- FR-001-T-04
- FR-001-T-05
- FR-001-T-06

**Acceptance test:**

```shell
pnpm dlx @lhci/cli autorun --collect.numberOfRuns=1 --assert.assertions.categories:performance=0.9 --assert.assertions.categories:accessibility=0.9
```

```task-meta
sizing: M
dependencies:
- FR-001-T-04
- FR-001-T-05
- FR-001-T-06
parallelisable: false
assignable_to:
- human
estimated_hours: 3.0
status: draft
runbook_hint: null
```

## FR-001-T-08 — Deploy + DNS + smoke test

Deploy the production build to Vercel (or fly.io as backup). Connect
cyberskill.world DNS at the registrar; configure A + AAAA + CAA + MX
records as needed (MX for the contact-form email may already exist).
Verify HTTPS via the host's auto-cert. Run the post-deploy smoke
test: curl /, /en, /vi, /about, /api/contact (OPTIONS) from three
regions (Singapore, US-East, EU-West) using a free latency-checker
service. TTFB p95 must be < 200 ms in all three regions. Write
docs/deployment.md as the runbook for future deploys + rollbacks.

**Preconditions:**

- FR-001-T-07 done (lighthouse passing)
- Domain cyberskill.world registered with DNS access
- Vercel (or fly.io) account exists; project linked to the repo

**Deliverables:**

- https://cyberskill.world resolves to the deployed site
- https://cyberskill.world/api/contact returns 405 on GET, 200 on valid POST
- TTFB measurement from 3 regions recorded at outputs/landing-page/ttfb-smoke.md
- docs/deployment.md (≤ 1 page) with deploy + rollback commands

**Dependencies:**

- FR-001-T-07

**Acceptance test:**

```shell
curl -s -o /dev/null -w '%{http_code}' https://cyberskill.world/ | grep -q 200 && curl -s -o /dev/null -w '%{http_code}' https://cyberskill.world/vi/ | grep -q 200
```

```task-meta
sizing: S
dependencies:
- FR-001-T-07
parallelisable: false
assignable_to:
- human
estimated_hours: 2.0
status: draft
runbook_hint: null
```
