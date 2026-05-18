# Acme Corp — Discovery Brief

> Synthetic fixture for statement-of-work-author smoke test. Fictional client. No real
> persons, no real money, no real PII. Used by the SKILL module parity harness
> per `skill/statement-of-work-author/acceptance/README.md`.

**Date:** 2026-05-17
**Source:** lead-intake call notes + email thread
**Intake by:** @stephen.cheng

## Client

- Trading name: Acme Corporation
- Legal entity: Acme Corporation Inc., Delaware USA
- Primary contact: Pat Acme (CEO)
- Secondary contact: Sam Smith (Head of Engineering)
- Industry: SaaS — B2B project management for landscape architecture firms
- Headcount: ~80; ~12 in engineering

## Problem statement

Acme has a customer portal built in 2021 on a legacy stack (Rails 6 + jQuery).
The portal handles customer login, project status, file uploads, and invoice
viewing. Current pain points:

- Page-load times >3 s on the project-list page (200+ projects per customer).
- Mobile experience broken (no responsive design).
- No SSO support — customers are asking for SAML / OIDC.
- No audit log for customer access — Acme's enterprise prospects keep asking
  for one.

Acme wants a Phase-1 modernisation focused on the most-impactful three: speed,
mobile, SSO. Audit log is Phase 2.

## Rough scope

- New customer portal (greenfield), React + tRPC + Postgres.
- SSO via SAML 2.0 (Okta, Azure AD) and OIDC (Google Workspace).
- Responsive design — mobile-first.
- Data migration from current Rails app's Postgres DB (~5M rows).
- Acceptance: 95% of customers can log in via SSO; project-list page p95 <800ms;
  Lighthouse mobile >90.

## Constraints

- Budget: USD $180k–$240k for Phase 1.
- Timeline: kick off mid-June 2026; demo at Acme's Sept-2026 user conference;
  general availability by end of November 2026.
- Compliance: customers include three EU-based architecture firms — GDPR
  applies. No PHI, no PCI.
- Acme retains all data-residency for US-based customers in `us-east`; the
  three EU customers can stay in `us-east` per their own data-processing
  addenda already on file.

## Engagement preferences

Pat asked for "milestone-based pricing with clear go/no-go gates so we know
where we are." This reads as fixed-price with stage gates.

Sam noted they want one Acme engineer embedded part-time with CyberSkill so
they can take over support post-launch. This reads more as dedicated-team or
T&M for that engineer.

→ HITL: pricing-terms ambiguity. Operator needs to choose engagement_model.
