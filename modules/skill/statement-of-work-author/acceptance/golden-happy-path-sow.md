---
template: statement-of-work@1
title: Acme Corporation — Customer Portal Modernisation Phase 1
client_name: Acme Corporation
client_legal_entity: Acme Corporation Inc., Delaware USA
engagement_model: fixed_price
effective_date: 2026-06-15
target_close_date: 2026-11-30
sow_version: 1.0.0
cs_signer: "@pat.acme"
em_signer: "@em.tba"
cyberskill_signer: "@stephen.cheng"
governing_law: Vietnam
provenance:
  source_path: ./golden-happy-path-brief.md
  source_hash: sha256:8a3f1c0d2e4b56789f1234567890abcdef0123456789abcdef0123456789abcd
---

# Statement of Work — Acme Corporation — Customer Portal Modernisation Phase 1

## 1. Objectives and Success Criteria

<!-- authority: human-confirmed --> Modernise the Acme customer portal so that:
the project-list page p95 load time drops below 800ms; ≥95% of Acme customers
can log in via SSO (SAML 2.0 + OIDC); the portal achieves a Lighthouse Mobile
score ≥ 90. <!-- source: golden-happy-path-brief.md §"Rough scope" + §"Problem statement" -->

## 2. Scope

### In Scope
- Greenfield customer portal (React + tRPC + Postgres).
- SSO integration (SAML 2.0 for Okta + Azure AD; OIDC for Google Workspace).
- Responsive mobile-first design.
- Data migration from the legacy Rails 6 portal's Postgres database (~5M rows).
- Performance hardening (p95 page-load <800 ms on project-list).
- GDPR-compliant data-handling for three EU-based customer firms.

### Out of Scope
- Audit-log feature (Phase 2 — separate SOW).
- Mobile native apps (web-responsive only).
- Migration of customer-uploaded files older than 24 months (separate archival plan).
- Customer training materials and webinars.

## 3. Deliverables

| # | Deliverable | Format | Owner | Target date |
|---|---|---|---|---|
| 1 | New customer portal (greenfield) | live system + source code | @em.tba | 2026-10-31 |
| 2 | SSO integration (SAML + OIDC) | production-deployed | @em.tba | 2026-09-30 |
| 3 | Data migration completion certificate | signed PDF + runbook | @tl.tba | 2026-10-15 |
| 4 | Performance test report | Markdown + k6 fixtures | @qa.tba | 2026-10-31 |
| 5 | Acme staff onboarding pack | runbook + Loom walkthroughs | @em.tba | 2026-11-15 |
| 6 | Phase-1 closure certificate | signed PDF (per `closure@1`) | @stephen.cheng | 2026-11-30 |

## 4. Assumptions and Constraints

- **Assumptions:** Acme provides production database snapshot by 2026-07-01;
  Acme's identity-provider admins respond within 5 business days on SAML
  configuration handshakes.
- **Constraints:** Budget USD $180k–$240k; demo deadline 2026-09 user
  conference; GA by 2026-11-30; GDPR applies (three EU customer firms).

## 5. Engagement Model

`fixed_price`. <!-- source: golden-happy-path-brief.md §"Engagement preferences" — Pat asked for milestone-based pricing -->

### Fixed-Price Terms
Milestone-tied invoice schedule:

| Milestone | % of fixed-fee | Trigger |
|---|---|---|
| M1 — Kickoff complete (SOW signed, env provisioned) | 15% | T+7 days |
| M2 — Architecture + design sign-off | 15% | 2026-07-31 |
| M3 — SSO live in staging | 20% | 2026-09-15 |
| M4 — Conference-demo cut | 15% | 2026-09-25 |
| M5 — Data migration verified | 15% | 2026-10-31 |
| M6 — GA (acceptance criteria met) | 20% | 2026-11-30 |

## 6. Team and Roles

| Stage | CS | EM | PO | TL | AR | DEV | QA | DO | SEC |
|---|---|---|---|---|---|---|---|---|---|
| (a) Discovery | @pat.acme | @em.tba (A/R) | — | C | C | — | — | — | C |
| (b) Requirements | @pat.acme | A | R | C | C | I | C | — | C |
| (c) Planning | I | A/R | C | C | C | I | C | C | C |
| (d) Architecture | I | A | C | R | A/R | C | C | C | C |
| (e) Detailed design | I | A | C | A | R | R | C | C | C |
| (f) Implementation | I | A | I | A | I | R | I | I | I |
| (g) Code review | I | A | — | A | C | R | — | I | C |
| (h) Testing | C | A | C | C | I | I | A/R | I | C |
| (i) Deploy | I | A | C | C | I | I | C | A/R | C |
| (j) Operations | I | A | I | I | I | I | I | A/R | C |
| (l) Closure | A | R | C | C | I | I | C | C | C |

## 7. Schedule and Milestones

See §5 Fixed-Price Terms milestone table for fee triggers. Acceptance gates:

| Milestone | Target date | Acceptance gate |
|---|---|---|
| M1 Kickoff | 2026-06-22 | DoR/DoD signed by Acme |
| M2 Design sign-off | 2026-07-31 | PRD + SRS + ADRs at 10/10 audit |
| M3 SSO live (staging) | 2026-09-15 | ≥3 IdP integrations green; threat-model 10/10 |
| M4 Demo cut | 2026-09-25 | Acme rehearsal recorded |
| M5 Data migration verified | 2026-10-31 | Row-count + checksum match against staging |
| M6 GA acceptance | 2026-11-30 | All §1 success criteria met in production |

## 8. Pricing and Invoicing

- **Fixed fee:** USD $210,000 (mid-band; subject to scope change-control per §11).
- **Invoice cadence:** per milestone in §5; net-30 payment terms.
- **Currency:** USD. Vietnam VAT not applicable (Acme is foreign-domiciled).
- **Late-payment policy:** 1.5%/month after 30 days net; work pause after 60 days.

## 9. Acceptance Criteria

Per-deliverable in §3 + the three measurable targets in §1. Definition of Done
applies as declared in `./dor-dod.md` (to be authored at M2).

## 10. IP and Confidentiality

- **IP assignment on payment:** all CyberSkill-authored Phase-1 source code,
  designs, and documentation transfer to Acme upon receipt of the M6 invoice.
- **Pre-existing IP carve-out:** CyberSkill retains the CyberOS skill module,
  the project-cleanup utility, and any cyberskill-vn skills used in support of
  the engagement.
- **Background-IP licensing:** CyberSkill grants Acme a perpetual, irrevocable,
  royalty-free, worldwide licence to use any background IP embedded in Phase-1
  deliverables for Acme's own internal business purposes.
- **NDA scope and term:** mutual NDA, two-year survival post-engagement.
- **Sub-processor list:** Cloudflare (CDN), Vercel (hosting), Sentry (errors),
  Datadog (APM), Postgres-as-a-service via Neon.
- **Data-processing addendum:** `./acme-dpa-2026.md` (GDPR-aligned; per
  golden-happy-path-brief §Constraints).
- **AI-tool usage disclosure:** CyberSkill engineers may use Claude Code,
  Cursor, and GitHub Copilot during Phase 1. AI-generated code is reviewed by
  a human per modules/cuo/docs/appendices.md (§13 Software Development Process) §5; every PR carries an
  `ai-assisted: yes/no` label. No Acme proprietary data is fed to AI providers
  without ZDR (zero-data-retention) attestation on file.

## 11. Change Control

Any change to scope, deliverables, schedule, or pricing requires a written
change-order, signed by CS + EM. Change-orders are priced at CyberSkill's
standard T&M rates (USD $95/hr senior, $70/hr mid, $50/hr junior). The default
change-order template lives at `./change-order-template.md` (created at M1).

## 12. Warranty, Support, and Governance Cadence

- **Warranty:** 90 days post-M6 acceptance — defects against §1 criteria fixed
  at no additional cost.
- **Support tier:** business-hours email + 24h response SLA during warranty;
  post-warranty handled via separate managed-services SOW or hand-over to
  Acme's internal team.
- **Governance cadence:** daily standup (internal); weekly written status to
  Pat + Sam + 30-min Friday call; fortnightly demo to Acme product team;
  monthly steering committee (Pat + @stephen.cheng); single QBR in December
  2026 covering DORA metrics, NPS, and Phase-2 roadmap.
