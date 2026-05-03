---
title: "TEN — per-tenant theme overrides, custom domains, brandability for external tenants"
author: "@stephen-cheng"
department: design
status: ready_for_review
priority: p3
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: user_facing
eu_ai_act_risk_class: not_ai
target_release: "P3 / 2027-Q4"
client_visible: true
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Ship per-tenant brand customisation for external tenants: **per-tenant theme overrides** layered on top of FR-DESIGN-001's `@cyberskill/tokens` (tenant logo + per-tenant colour-anchor overrides + per-tenant favicon + per-tenant Genie-mascot variant); **custom domains** (a tenant can set up `app.theirCompany.com` mapped to their CyberOS instance via Cloudflare for SaaS); **per-tenant Trust Center** rebranding so the customer's own status page lives at `status.theirCompany.com`; **white-label option** for T3 enterprise tenants where the "Powered by CyberOS" footer is removed; **brand kit upload** UI where the tenant admin uploads logo + colour swatches + the platform validates contrast ratios + renders preview; **PDF + email branding** so invoices, payslips (when shared), KB exports, and outbound emails carry the tenant's brand. CyberSkill's own brand remains the platform-default; tenant overrides only affect the tenant's own surfaces, never cross into platform-shared spaces (the marketing site at cyberos.world remains CyberSkill-branded).

## Customer Quotes

<untrusted_content source="founder_anticipation">
"For our enterprise customers, the platform needs to feel like it's part of *their* operations, not a third-party SaaS. They put their logo on it, their colours, their domain. The platform's job is to make that easy without breaking the consistency that the design system protects." — anticipated by Stephen
</untrusted_content>

## Problem

Without brand customisation, every external tenant's interface looks like CyberSkill. PRD §14.4.1 P3 scope: "per-tenant theme overrides; per-tenant DPIA artifacts; per-tenant CUO persona-version isolation." Three failure modes:

- **Tenant identity opacity.** A Member of Tenant Acme logs into the platform; without Acme's logo + colours, "is this our tool?" is ambiguous; adoption suffers.
- **Custom-domain absence.** Enterprise procurement at T3 expects the platform to live on the customer's own domain. Without Cloudflare-for-SaaS plumbing, every customer routes through `acme.cyberos.world` — a B2B SaaS deal-breaker.
- **Brand drift across surfaces.** Tenant logo on the app but CyberSkill on the invoice; consistency is the floor.

## Proposed Solution

The shape of the answer is per-tenant theme override layer + custom-domain plumbing + per-tenant Trust Center + brand-kit UI + cross-surface brand application.

**Per-tenant theme overrides.**

Schema (extending FR-TEN-001's `cyberos_meta.tenant`):

```sql
CREATE TABLE cyberos_meta.tenant_brand (
  tenant_id UUID PRIMARY KEY REFERENCES cyberos_meta.tenant(id) ON DELETE CASCADE,
  display_name TEXT NOT NULL,                                  -- the customer-facing brand name
  logo_blob_id UUID,                                           -- the primary logo (SVG required; PNG fallback)
  logo_dark_blob_id UUID,                                      -- logo for dark mode
  favicon_blob_id UUID,
  primary_colour_hex TEXT,                                     -- override Umber #45210E
  accent_colour_hex TEXT,                                      -- override Ochre #F4BA17
  surface_canvas_light_hex TEXT,
  surface_canvas_dark_hex TEXT,
  custom_domain TEXT,                                          -- "app.acme.com"
  custom_status_domain TEXT,                                   -- "status.acme.com"
  remove_powered_by_footer BOOLEAN NOT NULL DEFAULT false,      -- T3 plan only
  brand_kit_validated_at TIMESTAMPTZ,
  brand_kit_validated_by UUID,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

The `tenant_brand` row is read at request time by the host shell (FR-INFRA-001 §"Module-Federation host shell") — the design tokens are computed as the platform-default merged with the tenant's overrides. The merge is conservative: only the explicitly-overridden tokens change; everything else (typography, spacing, motion, elevation) remains platform-default to preserve the design system's structural consistency.

The brand-kit validation enforces:
- Logo is SVG (Tiny SVG profile preferred for BIMI compatibility per FR-EMAIL-001).
- Logo dimensions: 24-512 px in any direction.
- Colour overrides meet WCAG 2.1 AAA contrast against text (the validator computes contrast vs. text colours; rejects insufficient).
- Logo's "transparent vs. solid background" handled correctly for both dark + light modes.

**Custom domains via Cloudflare for SaaS.**

Provisioning flow:

1. Tenant admin enters their custom domain (e.g. `app.acme.com`) in `/tenant/admin/branding`.
2. The platform displays DNS records the customer must add:
   - `CNAME app.acme.com → custom-domain.cyberos.world`
   - `CNAME _acme-challenge.app.acme.com → <verification-token>.cyberos.world`
3. The customer adds the records to their DNS.
4. The platform's Cloudflare-for-SaaS integration verifies the records + provisions an SSL certificate (Cloudflare manages cert lifecycle).
5. The custom domain is bound to the tenant's slug; HTTP requests to `app.acme.com` route to the tenant's shard.
6. The platform's Apollo Router accepts the custom-domain `Host` header + resolves the tenant.

The status-domain `status.acme.com` follows the same pattern; routes to the tenant's per-tenant Trust Center (which shows only the tenant's own SLA + incident history scoped to their service experience, not platform-wide aggregates).

**Per-tenant Trust Center.**

The platform's Trust Center (FR-OBS-001) is shard-aggregate. Per-tenant Trust Centers are *tenant-scoped* slices showing:
- Tenant's tenant-specific uptime (their actual experience).
- Tenant's incident history (scoped to incidents that affected them).
- Tenant's branded compliance summary (their plan's regime status — PDPL/GDPR/SOC 2 etc.).
- Tenant's signed export public key (for verifying their data exports).

Customer's auditors + customers' customers can subscribe to the per-tenant Trust Center for incident notifications.

**White-label option (T3 only).**

T3 enterprise tenants can opt to remove the "Powered by CyberOS" footer from in-product UI + emails + PDFs. The setting is part of the tenant brand config but only enabled for T3. The audit log records the white-label flip + the customer's contractual acceptance.

**Brand-kit UI.**

`/tenant/admin/branding` provides:
- Logo upload (SVG primary; PNG fallback) with live preview in dark + light modes.
- Colour-anchor pickers with contrast-validation warnings.
- Custom-domain provisioning wizard with DNS-record copy buttons + verification status.
- Custom-status-domain (same flow).
- Mascot variant selector (customers choose a Genie variant from a curated set; FR-GENIE-001's mascot architecture supports per-tenant variants; the variants are pre-published by CyberSkill, never customer-uploaded — the genie is CyberSkill's IP).
- Preview pane showing the rebranded UI live + sample emails + sample PDFs.

**Cross-surface brand application.**

Brand applies consistently to:
- Module-Federation host shell.
- All module remotes (per-module remote reads `tenant_brand` at host-shell mount time).
- Outbound emails (FR-EMAIL-001 PDF + email templates incorporate brand).
- Invoices (FR-INV-001 PDF templates).
- Payslips when surfaced externally (rare; usually internal-only).
- KB exports (FR-KB-003 export bundles include brand).
- Trust Center page.
- Login pages.

Surfaces NOT branded by tenant (remain CyberSkill brand):
- The marketing site at `cyberos.world`.
- Platform-shared documentation.
- Cross-shard administrative dashboards (only CyberSkill team sees).

**Brand cache.**

The `tenant_brand` row is cached at the host shell + Apollo Router + module remote level for performance; a brand-update mutation broadcasts a cache invalidation NATS event; new request inherits the new brand within 30 seconds.

**Persona scope contract.**

CUO and all skills declare in `tools_allowed` the read of `cyberos.tenant.my_brand` so the persona's responses can reference the tenant's brand contextually ("Welcome to <tenant_brand.display_name>'s instance"). No mutation access.

**MCP tool surface.**

- `cyberos.tenant.my_brand` — read; everyone in the tenant.
- `cyberos.tenant.update_brand(patch)` — `destructive: true; requires_confirmation: true; sensitivity: medium`; tenant admin only.
- `cyberos.tenant.provision_custom_domain(domain, kind)` — `destructive: true; requires_confirmation: true; sensitivity: high; step_up_required: true`.
- `cyberos.tenant.list_my_custom_domain_status` — read.

## Alternatives Considered

- **Skip per-tenant theming; force everyone to CyberSkill brand.** Rejected: T2/T3 enterprise customers won't accept; competitive deal-breaker.
- **Full theme customisation including typography + spacing.** Rejected: design system structural consistency is the floor; customers override colour + logo, not typography.
- **Customer-supplied mascots.** Rejected: Genie is CyberSkill's IP per PRD §13.1; customers select from CyberSkill-published variants.
- **Custom domains via tenant-managed certificates.** Rejected: Cloudflare for SaaS handles cert lifecycle automatically; reduces customer-side ops burden + improves security posture.
- **Skip per-tenant Trust Center.** Rejected: enterprise procurement expects scoped status pages.

## Sales/CS Summary

Make CyberOS feel like part of your team's tools, not a third-party SaaS. Upload your logo, set your colour anchors, and use your own domain (`app.yourcompany.com`). Your auditors get a status page on `status.yourcompany.com` showing exactly your service experience. The design system stays consistent under the hood so quality + accessibility never break — you customise the surface, not the bones. Enterprise plan removes the "Powered by CyberOS" footer entirely.

## Success Metrics

- **Primary metric.** P3 sprint demo passes: (1) a synthetic external tenant uploads brand kit; logo + colours render correctly across host shell + sample email + sample invoice PDF; (2) custom-domain provisioning end-to-end via Cloudflare for SaaS; the synthetic `app.acme-test.com` routes correctly to the tenant's shard; (3) per-tenant Trust Center renders for the synthetic tenant; (4) T3 white-label option correctly removes the "Powered by" footer.
- **Adoption metric.** ≥ 70% of external tenants at P3 → P4 have configured a custom logo + colour overrides; ≥ 30% have configured custom domains.
- **Compliance metric.** No platform-shared surface (marketing site, cross-shard dashboards) ever displays a customer's brand.

## Scope

**In-scope.**
- `cyberos_meta.tenant_brand` schema.
- Brand-kit upload UI with contrast validation.
- Cloudflare for SaaS custom-domain plumbing.
- Per-tenant Trust Center surface.
- T3 white-label option.
- Cross-surface brand application (host shell + remotes + emails + PDFs).
- Brand-cache invalidation pipeline.
- Mascot-variant selector with curated variants.
- The 4 MCP tools.

**Out-of-scope (deferred).**
- Customer-uploaded custom mascots (forbidden by IP architecture).
- Per-Member theme overrides (P4+ — accessibility / personal preference).
- Custom typography (P4+ — likely never; design system protection).
- Custom-domain DKIM + SPF for email-from (P4 — when EMAIL goes white-label per tenant; today emails-from-platform use platform-managed cyberskill.world or per-tenant configured domain via FR-EMAIL-001).
- Embedded white-label of CyberOS inside the customer's own product (P4 PORTAL; this FR is for customer-of-CyberOS branding, not customer-of-customer).

## Dependencies

- FR-TEN-001 (multi-tenancy substrate).
- FR-INFRA-001 (host shell + Module Federation).
- FR-DESIGN-001 (`@cyberskill/tokens` + components).
- FR-AUTH-001 / FR-AUTH-002 / FR-AUTH-003 (admin step-up).
- FR-MCP-001.
- FR-OBS-001 (Trust Center substrate).
- Cloudflare for SaaS subscription tier on the platform's Cloudflare account.
- Compliance: trademark + brand-IP separation between CyberSkill (Genie + tokens + design system) and tenant (their logo + colours).
- Locked decisions referenced: DEC-266 (per-tenant overrides limited to colour + logo + domain; not typography or spacing), DEC-267 (mascot variants curated by CyberSkill), DEC-268 (white-label only on T3).

## AI Risk Assessment

Not applicable. `eu_ai_act_risk_class: not_ai`. Theming + custom-domain plumbing is deterministic.
