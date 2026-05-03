---
title: "TEN — self-service tenant onboarding flow: signup, residency selection, provisioning, first-run wizard"
author: "@stephen-cheng"
department: engineering
status: ready_for_review
priority: p1
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: full_stack
eu_ai_act_risk_class: not_ai
target_release: "P4 / 2028-Q2"
client_visible: true
template: feature_request@1
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Build the **self-service tenant onboarding flow** — the surface where a prospective customer (a small consultancy in any of the supported jurisdictions) creates a new tenant in CyberOS without anyone from the CyberSkill team being in the loop. Five steps: (1) **signup form** at `cyberos.world/start` collecting org name + admin email + jurisdiction selection (which determines residency shard); (2) **email verification + magic link**; (3) **plan tier selection** (T1/T2/T3 from FR-BILL-001) + payment-method capture via Stripe Setup Intent (no charge yet — 14-day free trial); (4) **automated tenant provisioning** that runs FR-TEN-002's `provision` lifecycle: create the per-tenant Postgres schema, KMS keys, S3 buckets, Keycloak realm, Apollo subgraph slot, brand-kit defaults, and seed the C-suite personas (CUO/CEO/COO/CTO from P0); (5) **first-run wizard** that walks the new tenant admin through importing their initial team (CSV or 5 manual entries) + configuring their two highest-priority modules (typically PROJ + CRM for a consultancy) + meeting the CUO. The flow is deliberately optimised so that "from /start to first useful CyberOS interaction" is ≤ 30 minutes for an admin who has the basic info ready. The FR also encodes the abuse-prevention layer: rate-limited per-IP + per-email signup, fraud-signal collection (Stripe Radar), automatic tenant suspension on bounce of payment-method validation.

## Problem

PRD §14.5.1 P4 entry-gate criterion: "Self-service tenant onboarding via cyberos.world/start completes in ≤ 30 minutes from form submission to first authenticated CyberOS use, with zero CyberSkill-employee involvement, for a typical 5-10 person consultancy in any supported residency." PRD §7.1 TEN module includes "self-service onboarding" as a P4 deliverable.

Without this flow, every new tenant is hand-provisioned by Founder or HR/Ops Lead — which doesn't scale beyond a handful of design partners. P4 launch needs the onboarding surface to be the default acquisition path.

Three failure modes:

- **Long onboarding times.** If steps are blocking (e.g. waiting for a human to verify the org), drop-off is high. Target: 30-min end-to-end.
- **Abuse / fraudulent signups.** Spammy accounts, content farms, abusive AI-usage attempts. Mitigation: rate limits + Stripe Radar + manual review for "suspicious" first 30 days (held in a sandbox shard).
- **Wrong residency selection.** A user from VN selects EU shard by mistake → cross-border-transfer compliance issue. Mitigation: auto-suggest based on IP + explicit confirmation step + irreversible-after-provisioning warning.

## Customer Quotes

<!-- Required when client_visible: true. Verbatim, attributed where possible. Paraphrasing here costs you the signal. -->

<untrusted_content source="other">
…paste verbatim customer quote here…
</untrusted_content>

<!-- TODO during implementation PR: capture real customer quotes from sales calls / NPS / support tickets. -->

## Proposed Solution

A linear, deterministic flow + an asynchronous provisioner + a first-run wizard.

**Step 1: Signup form (`cyberos.world/start`).**

Single-page form, no JavaScript framework heavy enough to slow first paint:
- Organization name (required).
- Admin email (required, must verify).
- Jurisdiction (radio + auto-suggest by GeoIP):
  - Vietnam → vn-shard.
  - Singapore + ASEAN → sg-shard.
  - EU + UK + Switzerland → eu-shard.
  - US + Canada + LATAM → us-shard.
  - Other → defaults to sg-shard with a "let us know if this isn't right" link.
- Team size estimate (5-10, 11-20, 21-50, 51+).
- Industry selector (Consultancy, Software Studio, Design Studio, Other) — drives module-recommendation defaults.
- Marketing-attribution dropdown ("How did you hear about us?") — optional.
- Terms-of-service checkbox + Privacy-policy link.
- Submit button.

Backend: rate-limited to 3/hour per IP, 5/hour per email-domain. CAPTCHA challenge on suspicious patterns (rapid-fire submission, known-spam IPs from Spamhaus).

**Step 2: Email verification + magic link.**

- Email sent within 30 seconds with a single-use, 24-hour-TTL magic link.
- Link lands at `/start/verify?token=…`; consumes the token; logs admin into a "pre-provisioning" state.
- If link expires: re-send option, 1 retry per hour.

**Step 3: Plan tier + payment method.**

- T1 / T2 / T3 plan tier comparison table (FR-BILL-001) with "best fit for your team size" suggestion highlighted.
- Stripe Setup Intent flow: collect payment method (card or SEPA debit for EU; Wise + Vietnamese banks for VN); no charge.
- 14-day free trial; explicit "you won't be charged until <date>" message.
- Optional: skip-payment-method-for-now (defaults to T1 with auto-suspension at trial end if not added).
- Submit → triggers Step 4.

**Step 4: Automated provisioning (asynchronous).**

The provisioner is a NATS-driven job that runs via FR-TEN-002's `provision` flow:

1. Allocate `tenant_id` (UUID).
2. On the chosen shard:
   - Create `tenant_<slug>` Postgres schema.
   - Apply baseline schema migrations (all P0 + P1 + P2 + P3 module schemas).
   - Insert the founding tenant admin user as the first AUTH principal.
3. Provision per-tenant resources:
   - KMS key (per-tenant data-key + persona-key, AWS KMS or equivalent in shard's region).
   - S3 buckets (`cyberos-<tenant>-blobs`, `cyberos-<tenant>-archives`).
   - Keycloak realm `internal-<tenant>` for tenant employees + reserved namespace `portal-<tenant>` for FR-PORTAL-001 (created later when first workspace is needed).
   - Apollo subgraph slot for the tenant's mutations.
4. Seed defaults:
   - Brand-kit defaults (placeholder logo, default tokens — admin customises in first-run wizard).
   - C-suite personas (CUO/CEO/COO/CTO from FR-GENIE-001) with the tenant's name substituted.
   - 5 default member roles (Founder/CEO, HR/Ops Lead, Account Manager, Engineering Lead, Member).
   - Default working hours (08:00-17:00 ICT for vn-shard, locale-appropriate elsewhere).
   - Default OBS dashboards.
5. Provisioning takes ~3-5 minutes typical. Status surface at `/start/provisioning` polls every 5 seconds; shows progress: schema → keys → buckets → personas → done.
6. On completion: redirect to first-run wizard.
7. On failure: revert (drop schema, destroy keys, etc.); alert ENG-LEAD on-call; surface friendly "we hit a snag, we'll email you within 1 hour" page.

**Step 5: First-run wizard.**

5-7 minute wizard, gentle-but-deliberate:

1. **Welcome from the CUO** — a short text from the tenant's CUO persona ("Hi, I'm the CUO for <Org>. Let me walk you through getting started."). Shows the persona-version badge.
2. **Brand kit** — upload logo, pick primary + accent colors; optional, skippable.
3. **Invite team** — CSV upload (name, email, role) OR 5 manual entries. Each invitee gets a magic-link email.
4. **Pick first 2 modules** — radio: PROJ, CRM, KB, EMAIL, CHAT, TIME. Suggested defaults based on industry (consultancy → PROJ + CRM). Other modules are still available; this is just where the wizard does extra setup.
5. **Configure first PROJ project** (if PROJ chosen) — project name, target date, first 5 tasks. Or "skip and explore on your own".
6. **Configure first CRM account** (if CRM chosen) — account name, primary contact, deal stage. Or skip.
7. **Meet the CUO** — opens a CUO chat with a starter question pre-filled: "What should I focus on this week?". The CUO uses BRAIN's tenant-scoped retrieval (only what's been entered so far).
8. **Done** — lands on `/dashboard`.

The wizard uses the design system from FR-DESIGN-001; vi-VN locale by default for vn-shard tenants.

**Abuse + fraud-prevention.**

- IP-rate-limit: 3 signups/hour per /24.
- Email-domain-rate-limit: 5 signups/hour per email domain (excludes major free providers via allowlist).
- Stripe Radar fraud signal: high-risk → tenant created in "sandbox" mode (limited features, manual review within 24 hours).
- Bounced payment-method validation: tenant auto-suspended via FR-TEN-002's `suspend` flow.
- Trial-end auto-suspension: at day 14, no payment method = tenant suspended; 30-day grace then archive (FR-TEN-002).
- Disposable-email-domain blocklist (mailinator, etc.).

## Alternatives Considered

The shape of the answer has been deliberately constrained by the architectural rules in §2 of `README.md` and the locked decisions cited in *Dependencies*. Notable rejected approaches:

- Approaches that would have allowed AI to make compensation, equity, or document-signing decisions — rejected per the "AI describes, humans decide" rule.
- Approaches that would have created cross-tenant read or write paths — rejected per the cross-tenant invariant (FR-TEN-001 invariant test harness).
- Where there are FR-specific alternatives, they're discussed inline in *Proposed Solution* and *Constraints*.

<!-- TODO during implementation PR: replace with FR-specific rejected alternatives. -->

## Out of Scope

- White-label onboarding (a partner running CyberOS under their own brand) — FR-TEN-006 in P4 follow-up.
- Self-service custom-domain configuration (handled in tenant admin console, FR-TEN-005, not the onboarding flow itself).
- Migrating data from another tenant or another platform during onboarding (handled via per-module migration FRs: FR-EMAIL-009, FR-PROJ-009, FR-CRM-004, FR-REW-006).
- Invitee onboarding (the team members invited in step 5 follow a separate magic-link flow; this FR covers the founding admin's flow only).
- Multi-organization onboarding (one user, multiple tenants) — possible, but each is a separate signup from this FR's perspective.

## Dependencies

- FR-TEN-001 (residency partitioning + shard topology).
- FR-TEN-002 (tenant lifecycle + `provision` flow).
- FR-TEN-003 (per-tenant theming — brand kit defaults).
- FR-AUTH-001 (RBAC + RLS — first AUTH principal creation).
- FR-AUTH-003 (passkey-first; magic-link).
- FR-BILL-001 (plan tier + Stripe Setup Intent + 14-day trial).
- FR-INFRA-001 (Module Federation host + per-tenant resource provisioning).
- FR-GENIE-001 (CUO persona seeding).
- FR-OBS-002 (default OBS dashboards seeded).
- FR-DESIGN-001 (design tokens + wizard component library).
- FR-MCP-001 (CUO chat in the wizard's "Meet the CUO" step).
- DEC-001 multi-tenant from day one; DEC-013 schema-per-tenant; DEC-016 passkey-first.

## Constraints

- **30-minute target end-to-end.** If a step takes > 5 minutes typical, the step is broken down or moved post-onboarding.
- **No human in the loop in the happy path.** CyberSkill team is not paged unless provisioning fails or fraud-signal triggers manual review.
- **Residency selection is irreversible after provisioning.** UI displays this clearly before submit. Migration to a different shard is a separate, paid, manual operation.
- **Trial without payment method** is allowed; auto-suspension at trial end is the safety net.
- **First-run wizard is skippable** at any step; tenant lands on dashboard with empty state + a "resume setup" banner.
- **No cross-residency cross-checks.** A vn-shard tenant cannot be queried about by a non-vn-shard surface; respects FR-TEN-001's hard partition.

## Compliance / Privacy

- **PDPL Decree 13/2023:** signup form collects personal data (admin email, name); processing basis = service provision; consent surface integrated with terms-of-service checkbox; DPIA refresh.
- **GDPR Article 13:** info-at-collection notice at the form (linked privacy policy + summary).
- **Cross-border transfer:** if admin is in a third country but selects vn-shard for the org, FR-TEN-001's residency rules apply; Schrems II TIA at the platform level (FR-CP-004) covers the standard cases; flagged exotic combinations require manual review.
- **EU AI Act:** no AI in the onboarding flow itself except the "Meet the CUO" closing step, which is CUO chat — already covered by FR-GENIE-001's risk classification.
- **PCI-DSS:** Stripe Setup Intent runs in Stripe's hosted iframe; CyberOS doesn't load card data; PCI scope = SAQ A.
- **Decree 130/2018 (VN e-signature):** ToS acceptance is a Simple Electronic Signature; logged + audit-chained.

## Risk Assessment (AI-emitting features)

The onboarding flow itself has no AI surface (other than the closing CUO chat which inherits risk classification from FR-GENIE-001). `eu_ai_act_risk_class: not_ai`.

## Vietnamese-locale considerations

- `cyberos.world/start` auto-detects vi-VN by Accept-Language header for VN-shard pre-selection; explicit toggle available.
- Vietnamese form copy authored + reviewed by native speaker (cultural appropriateness, formality level — `Anh/Chị` register).
- Vietnamese ToS + Privacy Policy translated by the legal counsel; legal-entity references match Decree 13.
- Magic-link emails localised: vi-VN for vn-shard, en-US for sg/eu/us-shard.
- Vietnamese e-receipts (Decree 123) integrated when first invoice issues post-trial.
- Be Vietnam Pro typography on every wizard screen.

## Scope (acceptance criteria — auditable)

- [ ] Signup form lives at `cyberos.world/start`; form submits successfully; rate limits enforced (CI test: 4th submission in an hour from same IP returns 429).
- [ ] Email verification flow: magic-link issued ≤ 30s; consumed-once; 24-hour TTL.
- [ ] Plan tier selection + Stripe Setup Intent: payment method captured; no charge; trial countdown shown.
- [ ] Provisioning job completes in ≤ 5 minutes for the median case; the schema, KMS keys, S3 buckets, Keycloak realms, Apollo slot, brand-kit, C-suite personas, and default seeds all exist and are health-checked.
- [ ] Provisioning failure → automatic rollback; tenant admin gets a friendly error page; on-call ENG-LEAD paged.
- [ ] First-run wizard: 7 steps surface in order; skip-able; CSV team import works; first PROJ + first CRM seeded if chosen; CUO chat opens with starter question.
- [ ] End-to-end median time-to-first-CyberOS-interaction: ≤ 30 minutes (measured via instrumentation).
- [ ] Residency mismatch warning: if GeoIP country ≠ selected jurisdiction, warning surface before submit.
- [ ] Disposable-email blocklist: signup with mailinator email is rejected.
- [ ] Sandbox-mode tenant: high-risk Stripe Radar signal → tenant created in sandbox; manual-review queue surfaces in CyberSkill internal admin.
- [ ] Trial-end suspension: tenant at day 14 with no payment method → auto-suspended via FR-TEN-002.
- [ ] Audit chain: every step (signup, verification, plan select, provisioning start/end, wizard completion) is logged with timestamps + actor.
- [ ] vi-VN regression: complete the entire flow in Vietnamese; copy + UX reviewed by native speaker.

**Gherkin (PRD §19.18).**

```gherkin
Feature: Self-service onboarding completes in ≤ 30 minutes

  Scenario: Vietnamese consultancy admin onboards via cyberos.world/start
    Given the admin lands at cyberos.world/start from a VN IP at 09:00
    And the admin selects "Vietnam" jurisdiction (auto-suggested)
    And the admin completes the signup form with org name "Acme VN" + admin email
    When the admin verifies their email via magic link within 5 minutes
    And the admin selects T1 plan + adds a payment method
    And the provisioner runs to completion in 4 minutes
    And the admin completes 4 of the 7 first-run wizard steps (skipping 3)
    And the admin lands on /dashboard at 09:25
    Then the entire flow completes in ≤ 30 minutes
    And the tenant exists in vn-shard with a fully provisioned schema
    And the admin user has passkey registered
    And the CUO/CEO/COO/CTO personas are seeded
    And the audit chain has 7 events for this onboarding
    And the time-to-first-interaction metric is recorded as 25 minutes

Feature: Sandbox-mode for high-risk signups

  Scenario: Stripe Radar flags a high-risk signup
    Given an admin completes signup with a credit card flagged by Stripe Radar
    When the provisioner is invoked
    Then the tenant is created in sandbox mode (limited shard, limited features)
    And the tenant admin sees a "Your account is being reviewed; we'll respond within 24h" page
    And a CyberSkill internal admin sees the manual-review item in the queue
    And no full-feature provisioning happens until the admin approves the tenant
```

## Success Metrics

- Self-service onboarding completion rate: ≥ 60% of started signups complete the wizard (gate-window).
- Median time from /start to first authenticated CyberOS interaction: ≤ 30 minutes.
- Provisioning failure rate: ≤ 1% (rolling 7-day).
- Sandbox-mode flag rate: ≤ 5% of signups.
- Trial-to-paid conversion rate: ≥ 25% (P4 + 90 days; will improve with iteration).

## Sales/CS Summary

<!-- Required when client_visible: true. One paragraph written so a non-engineer can pitch the feature. Plain English. No internal jargon, no module codes, no speculation about future scope. -->

<!-- TODO during implementation PR: write the customer-facing pitch. -->

## Open Questions

- **OQ-TEN-004-01.** Should the trial be 14 days or 30 days for non-VN-shard tenants? Default: 14 across the board, revisit post-launch based on conversion data.
- **OQ-TEN-004-02.** Should a credit card be required at signup, or can users start without one and add later? Default: payment method optional at signup; auto-suspend at trial end if missing.
- **OQ-TEN-004-03.** Should we offer founder-led "white-glove" onboarding for tenants with > 50 seats at signup? Default: yes; flag at form-submit time and route to a "schedule onboarding call" step instead of self-service.

## AI Authorship Disclosure

- **Tools used:** Claude Cowork (Anthropic).
- **Scope:** drafted the FR end-to-end against the PRD + SRS; founder reviews and edits before status changes from `ready_for_review`.
- **Human review:** founder (`@stephen-cheng`) — final wording is the founder's responsibility.

## References

- PRD §7.1 TEN module; PRD §14.5.1 P4 entry-gate.
- SRS Decisions Log: DEC-001, DEC-013, DEC-016.
- FR-TEN-001/002/003, FR-AUTH-001/003, FR-BILL-001, FR-INFRA-001, FR-GENIE-001, FR-OBS-002, FR-DESIGN-001, FR-MCP-001.

---

*ai_authorship: co_authored — drafted by Claude Cowork on 2026-05-03.*
