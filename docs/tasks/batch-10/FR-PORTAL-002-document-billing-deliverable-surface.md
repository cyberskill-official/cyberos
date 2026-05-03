---
title: "PORTAL — document signing surface, invoice viewer, deliverable hub for external clients"
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
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Build the three "high-value action surfaces" inside PORTAL on top of FR-PORTAL-001's foundation: (1) the **document signing surface** that lets a counterparty review and sign a CyberOS-managed document (NDA, MSA, SOW, change-order) using FR-DOC-001's three signing tiers (SES/AdES/QES); (2) the **invoice viewer** that shows AR-side invoices the tenant has issued + the payment status + a deep-link to a hosted Stripe/VNPay/Wise checkout (FR-INV-003); (3) the **deliverable hub** that lets the counterparty download tenant-published files via presigned URLs with TTL + watermarking. These three surfaces are where PORTAL stops being "a status read" and starts carrying transactional weight: a counterparty who signs a contract or pays an invoice through PORTAL has produced a legally-binding outcome that ripples back into the tenant's CyberOS instance (DOC, INV, AUDIT chain). The FR encodes the architectural constraints that make this safe at multi-tenant scale: signing is always step-up-authenticated; invoice payment never exposes payment instrument details to PORTAL; deliverable downloads are scoped to the workspace + watermarked + audit-logged.

## Problem

PRD §7.1 PORTAL scope explicitly names "signed contracts" and "invoice hub" as P4 deliverables. PRD §14.5.1 P4 entry-gate criterion includes "first paying CyberSkill client has paid ≥ 1 invoice via PORTAL deep-link". Without these surfaces, PORTAL is read-only and provides too little value to justify counterparty adoption.

Three failure modes if not built carefully:

- **Cross-workspace document leakage.** A counterparty can be invited to multiple workspaces (for tenants who serve them on multiple engagements). Without strict per-workspace publication scoping, the user could see documents from a workspace they're meant to access in a workspace they're not.
- **Payment instrument leakage.** PORTAL is at the public-Internet edge; if it ever loaded card data into its DOM, the tenant becomes PCI-DSS-in-scope. The architectural contract: PORTAL never loads payment data, only deep-links to the payment processor's hosted checkout.
- **Deliverable URL replay.** A presigned URL leaked outside the workspace context (forwarded email, log file copy) becomes an unauthenticated access vector. Mitigation: short TTL (24 hours) + per-download audit + watermark binding the file to the user.

## Proposed Solution

Three independent surfaces, each backed by an existing internal module via a publication adapter.

**(1) Document signing surface.**

When a tenant employee initiates an FR-DOC-001 signing flow targeting an external counterparty, the flow's "external signer" leg renders in PORTAL:

- Counterparty navigates to `/portal/<slug>/documents/<opaque-doc-slug>` (slug is opaque; the underlying `doc.document_id` is never in the URL).
- Step-up auth required: passkey or magic-link with explicit "I am about to sign a document" prompt.
- Document renders in the in-browser PDF viewer (FR-PDF-VIEWER pattern reused; not the same code, but same UX).
- The signing UI shows: signing tier badge (SES/AdES/QES), counterparty signing block, the signing certificate's intent, the consequences ("by signing, you bind <client legal entity> to <agreement>"), and a checkbox "I have read and understood".
- On sign action: the FR-DOC-001 signing engine is invoked server-side (not client-side); the signature is computed; the signed PDF is generated; the audit-chain entry is written.
- Counterparty sees a "Signed" confirmation; the signed PDF is published back to the workspace as a `doc_signed` publication.
- The originating tenant employee receives a Notify (FR-CHAT-001 channel + email) with the signed-doc link.
- Failure mode: the counterparty refuses to sign (they reject). The doc stays in `awaiting_external_signature` state; tenant employee is Notified.

QES-tier signing (the strongest, eIDAS-recognised tier) requires the counterparty to step through their QTSP flow (handled by the eIDAS QTSP integration in FR-DOC-001); PORTAL hosts the redirect-and-return pattern.

**(2) Invoice viewer.**

When a tenant employee publishes an `invoice` to a workspace (via FR-PORTAL-001's publication flow):

- The publication adapter pulls from `inv.invoice` + `inv.payment` + the rendered PDF.
- PORTAL renders an invoice detail view: invoice header (number, date, due date, amount, currency), line items, totals, VAT/tax breakdown (Vietnamese e-invoice format if VN-shard), payment status, "Pay this invoice" CTA.
- The "Pay" CTA launches a deep-link to the appropriate processor's hosted checkout:
  - VN-shard tenants + VND amounts → VNPay hosted checkout.
  - Non-VND amounts + non-VN counterparties → Stripe Checkout Sessions.
  - Cross-border invoices → Wise hosted payment link.
- The deep-link includes a short-lived (15-min) token that scopes the checkout to this specific invoice + amount + workspace + counterparty user.
- On checkout completion, the processor's webhook lands in FR-INV-003's webhook handler, which marks `inv.payment.status = 'completed'` and triggers a Notify back to the workspace.
- PORTAL never loads payment instrument data into its DOM. PCI-DSS scope stays at the processor.

The invoice viewer also shows historical invoices (only those published to this workspace) + their payment status. AR aging context (e.g. how late this invoice is) is shown only if the tenant employee opted to publish that field; default off, to avoid unnecessary financial disclosure.

**(3) Deliverable hub.**

A `deliverable` publication is a tenant-published file or list of files (typically the actual project outputs: code archive, design files, reports, datasets):

- File storage: per-tenant S3 bucket (FR-INFRA-001 + FR-TEN-001).
- Publication renders a list of files with name, size, type, last-modified.
- "Download" generates a presigned URL with **24-hour TTL** + a workspace-user binding (the URL's signing context includes the user's portal session).
- The downloaded file is **watermarked** server-side at download time:
  - PDF, DOCX, PPTX: a footer "<workspace_name> — <user_email> — <download_ts>" is rendered into every page.
  - Images: a corner watermark with the same metadata.
  - Code archives: a `WATERMARK.txt` file is added to the zip with the same metadata + a SHA-256 hash of the original archive.
  - Other binary files: no watermark; logged but not modified.
- Each download is logged in `portal.publication_access_log` with `kind = 'deliverable_download'` + `bytes_served` + `presigned_url_id`.
- Re-downloads within the 24-hour TTL window do not generate a new presigned URL; the same one is returned (idempotent).

If a deliverable file is updated, the tenant employee creates a new publication; the old presigned URLs continue to work for 24 hours then expire; the workspace UI shows "Updated 2 hours ago — see latest version".

**Cross-workspace constraint enforced.**

Every operation in this FR includes a `workspace_id` parameter sourced from the URL slug + cross-checked against `portal.workspace_user.workspace_id` for the requesting external user. RLS at the database layer is the floor; application-layer assertions are belt-and-braces. The FR-TEN-001 invariant test harness is extended to cover all three surfaces.

## Out of Scope

- Counterparty-initiated document creation (FR-PORTAL-005, follow-up).
- Counterparty-uploaded files (FR-PORTAL-005, follow-up).
- Recurring invoice / subscription payment management surfaced in PORTAL (handled in tenant's billing surface, not counterparty's; FR-BILL-001 stays internal).
- Cryptocurrency payment options (not in PRD).
- Detailed AR aging history (intentionally hidden; tenant chooses what to publish).
- Document collaborative editing in PORTAL (PORTAL is review-and-sign only; collaborative redline lives in FR-DOC-002 internal surface).

## Dependencies

- FR-PORTAL-001 (workspace + publication framework).
- FR-DOC-001 (e-signature schema + AATL + eIDAS).
- FR-DOC-002 (redline review — same in-browser PDF viewer pattern).
- FR-INV-001/002/003/004 (invoice schema + lifecycle + payment integrations + frontend — payment status surfaced in PORTAL).
- FR-AUTH-003 (step-up auth — required for sign action and view-sensitive-doc).
- FR-INFRA-001 (per-tenant S3 + presigned URL pattern).
- FR-TEN-001 (residency partitioning + cross-tenant invariant tests).
- FR-CHAT-001 (Notify on signed/paid events).
- FR-AUTH-002 (audit chain — every sign + every download captured).
- DEC-001, DEC-013, DEC-016, DEC-017 (audit chain), DEC-051 (storage layout per tenant).

## Constraints

- **Step-up auth mandatory before sign action.** Cannot be bypassed.
- **Payment data never in PORTAL DOM.** Architectural rule; CI test asserts no PCI-relevant fields are ever returned by portal API.
- **Presigned URL TTL ≤ 24 hours.** Configurable downward by tenant; never upward.
- **Watermark cannot be disabled at MVP.** Configurability deferred to a follow-up.
- **No deliverable size > 5 GB at MVP.** Larger files via direct S3 link with separate access flow (FR-PORTAL-006, follow-up).
- **No bulk download (zip-of-zips) at MVP.** Per-file downloads only.

## Compliance / Privacy

- **PDPL Decree 13/2023:** invoices contain counterparty's contact + financial details; classified as personal + commercial data; access logs preserved for the regulatory minimum.
- **GDPR Article 32:** technical + organisational measures — TLS 1.3 only on PORTAL; presigned URLs use AWS SigV4; watermarking adds traceability.
- **EU AI Act:** no AI in this FR.
- **eIDAS Regulation:** QES signing flow is implemented per FR-DOC-001 spec; PORTAL is the user-facing leg.
- **Vietnamese Decree 130/2018 e-signature:** SES/AdES tiers comply; QES via Vietnamese qualified provider (FR-DOC-001 has a VN QTSP option).
- **PCI-DSS:** PORTAL is **out of scope** for PCI by architectural design (no card data ever loaded). The payment processor is in scope; the tenant's PCI questionnaire (SAQ A) is the appropriate fit.
- **ISO 27001 A.5.34 (privacy + PII protection):** counterparty access logs encrypted at rest; access to logs scoped by tenant + DPO.
- **Watermark consent:** the workspace ToS (presented at first login) names watermarking explicitly; no separate consent surface needed.

## Risk Assessment (AI-emitting features)

No AI surface in this FR. Document drafting is internal (FR-DOC-002); invoice generation is internal (FR-INV-002 with CUO/CFO read-only assist); PORTAL is delivery-only.

## Vietnamese-locale considerations

- Vietnamese e-invoice (Decree 123/2020/NĐ-CP) compliant rendering for VN-shard tenants: invoice number prefix, fapiao layout, VAT breakdown, e-invoice signing certificate display.
- VNPay hosted checkout deep-link uses the bank's preferred locale (vi-VN by default).
- Document signing tier names localised (vi: "Chữ ký số đơn giản" / "tiến cấp" / "đủ điều kiện"; en: "Simple / Advanced / Qualified").
- Watermark text supports Vietnamese characters (Be Vietnam Pro family).

## Scope (acceptance criteria — auditable)

- [ ] Document signing flow end-to-end: tenant employee initiates signing, counterparty receives invite (workspace publication + email + magic-link), step-up auth, document renders in viewer, counterparty signs, signed PDF generated by FR-DOC-001 server-side, signed publication appears in workspace, audit-chain entries written, tenant employee Notified.
- [ ] QES-tier signing: redirect-and-return flow with at least one eIDAS QTSP integration tested end-to-end.
- [ ] Invoice viewer: published invoice renders correctly with line items + totals + VAT breakdown; "Pay" CTA launches the right processor based on currency + counterparty residency; processor webhook lands and updates `inv.payment.status`.
- [ ] Cross-currency case: USD invoice for an EU-shard tenant routes to Stripe; VND invoice for a VN-shard tenant routes to VNPay; cross-border (VND collected from a US counterparty) routes to Wise.
- [ ] Deliverable download with watermark: PDF + DOCX + PPTX get watermark-stamped with workspace + user + timestamp; image files get corner watermark; code archives get `WATERMARK.txt` added.
- [ ] Presigned URL: 24-hour TTL enforced; expired URL returns 403 with a "request a new download link" prompt that re-checks workspace membership.
- [ ] CI test: PORTAL API never returns a string matching credit-card regex / `cvv` / `card_number` from any endpoint.
- [ ] CI test: every URL of the form `/portal/<slug>/documents/<id>` or `/portal/<slug>/invoices/<id>` is opaque (slug, not internal UUID).
- [ ] FR-TEN-001 invariant tests extended for `portal.publication` + presigned URL leakage scenarios.
- [ ] PCI-DSS SAQ-A scoping document drafted by FR-CP-005's compliance plane and confirmed.
- [ ] vi-VN locale: e-invoice rendering matches Decree 123 exactly (manual QA pass + automated regex on key fields).

**Gherkin (PRD §19.18).**

```gherkin
Feature: Counterparty signs a document via PORTAL

  Scenario: Counterparty signs an MSA at AdES tier
    Given Tenant employee E has initiated an AdES-tier signing flow targeting Counterparty C
    And C has been invited to workspace W
    When C navigates to W's home and opens the signing-pending publication
    And C completes step-up auth via passkey
    And C reviews the document in the in-browser viewer
    And C ticks "I have read and understood" and presses "Sign"
    Then the FR-DOC-001 signing engine generates the AdES signature server-side
    And the signed PDF is published back to W as a doc_signed publication
    And the FR-AUTH-002 audit chain entry is written with C's user_id + IP + UA + persona-version
    And E receives a Notify in their FR-CHAT channel
    And the doc.document state transitions to "fully_signed"

Feature: PORTAL never carries payment data

  Scenario: Counterparty pays an invoice via PORTAL deep-link
    Given Tenant T has published invoice I to workspace W
    When Counterparty C presses "Pay this invoice" in the PORTAL invoice viewer
    Then PORTAL responds with a 302 redirect to the appropriate processor's hosted checkout
    And the redirect URL contains a short-lived (≤ 15 min) one-time token
    And no payment instrument fields appear in any PORTAL HTML, JSON, or log
    When C completes payment on the processor
    Then the processor's webhook lands at FR-INV-003 within 5 minutes
    And inv.payment.status flips to "completed"
    And W is updated with the new payment status within 60 seconds
```

## Success Metrics

- First paying CyberSkill client pays ≥ 1 invoice via PORTAL deep-link within 30 days of P4 launch.
- ≥ 1 contract signed via PORTAL at AdES tier (the typical tier for SaaS contracting).
- Zero PCI-DSS leakage events.
- Presigned URL anomaly rate (downloads from unexpected IPs) ≤ 0.5% of total.
- Document signing flow median completion time ≤ 7 minutes.

## Open Questions

- **OQ-PORTAL-002-01.** Should AR aging context be hidden by default and opt-in per tenant, or shown by default and opt-out? Default proposal: hidden by default; tenant opts in when they want to apply collection pressure.
- **OQ-PORTAL-002-02.** Should the QES-tier flow support all major eIDAS QTSPs at MVP, or just the 2-3 most likely (e.g. Adobe Sign QTSP, DocuSign EU QTSP, Trustpro)? Default: 2-3 at MVP; expand later based on customer demand.
- **OQ-PORTAL-002-03.** Should watermark text be customisable per tenant (e.g. logo overlay)? Default: no at MVP; revisit post-launch.

## References

- PRD §7.1 PORTAL scope.
- PRD §14.5.1 P4 entry-gate.
- SRS Decisions Log: DEC-013, DEC-016, DEC-017.
- FR-PORTAL-001, FR-DOC-001/002, FR-INV-001/002/003/004, FR-AUTH-002/003, FR-INFRA-001, FR-TEN-001.

---

*ai_authorship: co_authored — drafted by Claude Cowork on 2026-05-03.*
