---
title: "TEN — tenant admin console: billing, users, theme, custom domain, audit log, data export"
author: "@stephen-cheng"
department: engineering
status: ready_for_review
priority: p1
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: full_stack
eu_ai_act_risk_class: not_ai
target_release: "P4 / 2028-Q3"
client_visible: true
template: feature_request@1
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Build the **tenant admin console** at `/admin` — the surface where the tenant's Founder/CEO + Engineering Lead manage their tenant: billing + plan changes (FR-BILL-001 surface), user/team management (invites, role assignment, suspension, removal), brand/theme configuration (FR-TEN-003 surface), custom domain configuration (Cloudflare for SaaS DNS records walkthrough), audit-log access (FR-AUTH-002 read surface, scoped to the tenant), data export + DSAR fulfilment (FR-CP-002 + FR-CP-004 surface), tenant lifecycle controls (FR-TEN-002 admin actions: suspend, archive, delete with crypto-shred), and AI usage budget management (per-module + per-persona caps with the 80/100/110 Notify ladder). The admin console is the **single coherent surface** that ties every cross-cutting tenant-level concern together; without it, tenants would be hunting through 6+ different module surfaces to do basic admin tasks. Step-up auth is required for every destructive or financial action; an "admin actions" timeline surfaces a rolling 90-day record of what the admin team has changed; emergency-revoke flow lets a Founder pull every active session in 30 seconds in case of a security incident.

## Problem

PRD §7.1 (TEN module) lists "tenant admin console" as a P4 deliverable; PRD §14.5.1 names "tenant admin can complete all common admin tasks without contacting CyberSkill support" as an exit-gate criterion. Without a unified console:

- Tenant admins hunt through `/auth`, `/inv`, `/obs`, `/cp`, `/ten`, etc. to do their job; UX is terrible.
- Audit-log access is per-module; tenant admin can't get a single view of "who did what in my tenant".
- Tenant lifecycle controls (suspend a leaving employee, archive an old engagement, delete the tenant) need a consistent surface — duplicating them per-module would lead to drift.
- Emergency-response (rotate keys, revoke all sessions) needs a single fast path; embedding it in a deep menu is dangerous.

Three failure modes if not built carefully:

- **Unauthorised admin escalation.** A Member-role user shouldn't see admin pages. RBAC enforced at every endpoint, with UI hiding paths the user can't reach.
- **Destructive action without confirmation.** Crypto-shred is permanent; suspending an active employee blocks all their work. Multi-step confirmation + step-up + audit log entries.
- **AI budget runaway.** Without per-tenant + per-persona caps, an LLM-heavy month can produce a surprise invoice. Budget controls + Notify ladder + auto-pause prevent surprise.

## Customer Quotes

<!-- Required when client_visible: true. Verbatim, attributed where possible. Paraphrasing here costs you the signal. -->

<untrusted_content source="other">
…paste verbatim customer quote here…
</untrusted_content>

<!-- TODO during implementation PR: capture real customer quotes from sales calls / NPS / support tickets. -->

## Proposed Solution

A single `/admin` route with seven panes: Overview, Billing, Team, Brand & Theme, Custom Domain, Security & Audit, Data & Privacy. Each pane is a Module Federation remote slot loaded into the host shell. RBAC enforced at the gateway + every panel.

**Pane 1: Overview.**

- Tenant snapshot: plan tier, days into billing cycle, active users, AI usage % of budget.
- Recent admin actions (last 7 days): up to 20 lines, "<actor> did <action> on <object> at <time>".
- Health card: Compliance Cockpit summary (FR-CP-001), incident count (FR-OBS-002), CUO acceptance trend.
- Quick actions: invite user, change plan, view audit log.

**Pane 2: Billing.**

Surfaces FR-BILL-001 + FR-INV-002:
- Current plan tier (T1/T2/T3) with comparison + "Upgrade" / "Downgrade" CTAs.
- Current period: cumulative bill, AI usage %, days remaining.
- Payment method (last 4 of card, or SEPA mandate ID, or Wise account); "Update" CTA → Stripe Customer Portal.
- Invoices table: past 12 months; download PDF; status (paid/due/overdue).
- AI usage breakdown: per-module + per-persona; chart over time; per-user heatmap.
- Budget caps: per-module $ cap; per-persona $ cap; per-counterparty-user $ cap (for CXO inside PORTAL).
- Notify settings: who gets the 80/100/110 alerts (Founder + Engineering Lead by default).

**Pane 3: Team.**

Surfaces FR-AUTH-001 + FR-HR-001 (limited admin view; full HR view stays in `/hr`):
- User list: email, name, role, status (invited/active/suspended/removed), last login, AI usage YTD.
- Add user: email + name + role; sends invite via FR-AUTH-001 magic-link; tracks acceptance.
- Suspend / Reactivate / Remove: each is a step-up-auth-gated action; remove triggers FR-HR-001 + FR-CP-002 termination + DSAR flow.
- Role-change: assign or revoke roles; surfaces the role's permissions in plain-text.
- Bulk actions: suspend all from a department; CSV export of users.
- SSO/SCIM (T2 + T3 plans): configure SAML/OIDC IdP federation + SCIM 2.0 user provisioning (FR-AUTH-001 extension).

**Pane 4: Brand & Theme.**

Surfaces FR-TEN-003:
- Brand kit: logo upload (light + dark variants), favicon, primary + accent + neutral colors, typography selection (Be Vietnam Pro / Inter / custom upload), email-template branding.
- Preview: sample dashboard, sample email, sample invoice rendered with the kit.
- Theme variant: light / dark / system.
- T3 white-label: hide CyberSkill watermark + use custom logo on the auth + invoice surfaces (T3 plan only).

**Pane 5: Custom Domain.**

Surfaces FR-TEN-003's Cloudflare for SaaS integration:
- Internal domain: `<slug>.cyberos.world` (always available, never removable).
- Custom domain: `app.acme.com` (T2 + T3 plans). Walkthrough:
  1. User enters target domain.
  2. CyberOS shows DNS records to add (CNAME to Cloudflare for SaaS hostname).
  3. CyberOS polls every 60 seconds; surfaces verification status.
  4. On verification success: TLS cert auto-issued via Cloudflare; routing live within 5 minutes.
- Custom PORTAL domain (T3 plan): `portal.acme.com` (white-label PORTAL).
- DNS propagation troubleshooting tools (DNS lookup, "what we see", "what you typed").

**Pane 6: Security & Audit.**

Surfaces FR-AUTH-002 + FR-AUTH-003:
- Audit log: filterable by actor / action-kind / time range / object kind. Renders the Merkle-chained audit chain entries. Export to CSV/JSON.
- Active sessions: list all active sessions across users; revoke individually or all at once.
- **Emergency revoke**: red CTA "Revoke all sessions"; step-up + 2-of-2 (Founder + Engineering Lead) for active production tenants; logs reason; pages on-call.
- Passkey + MFA enforcement: per-role policy ("CEO must use passkey", "all members must have MFA enabled within 30 days"); surfaces non-compliant users for follow-up.
- Federated auth (T2 + T3): IdP configuration; certificate rotation reminders.
- Persona-scope contracts: list of active personas; their allowed tools; persona-version + skill-version chain; signed-by metadata.
- API tokens (FR-API-001): list + create + revoke; scoped to per-tenant API keys.

**Pane 7: Data & Privacy.**

Surfaces FR-CP-002 + FR-CP-004:
- DSAR queue: list of incoming DSARs (internal user + external counterparty); status; resolve actions.
- Export tenant data: full export (signed .zip via S3 presigned, 7-day TTL, FR-BRAIN-003 archival pattern).
- Per-user data export: subject-rights export for any user (GDPR Article 20 + PDPL Article 26); DPIA-aligned.
- Retention policies: per-module retention period; configurable downward (within statutory floor); regulatory floor enforced.
- BRAIN denylist: list of fields blocked from BRAIN ingestion (DEC-036); read-only display.
- Sub-processor list: list of GDPR sub-processors (FR-CP-004); "subscribe to changes" flow.
- Trust Center deep-link: link to the tenant's Trust Center page (cyberos.world/trust/<slug>).
- **Tenant deletion**: red CTA "Delete this tenant"; multi-step confirmation; step-up + 2-of-2 + 30-day grace period; triggers FR-TEN-002's `delete` lifecycle (crypto-shred).

**RBAC.**

| Role | Overview | Billing | Team | Brand | Custom Dom | Security | Data |
|---|---|---|---|---|---|---|---|
| Founder/CEO | rw | rw | rw | rw | rw | rw | rw |
| Engineering Lead | r | r | rw (no remove) | r | rw | rw | r |
| HR/Ops Lead | r | r | rw (no role-change) | r | r | r | r |
| Account Manager | r | - | r | - | - | - | - |
| Member | (own profile only — different surface) | - | - | - | - | - | - |

Each panel checks against this matrix at the gateway + at the UI layer.

**Audit + step-up.**

Every destructive or financial action: step-up auth + audit-chain entry with full context. The "admin actions timeline" on the Overview pane is just a query against the chain filtered to admin-action kinds.

**Anti-foot-gun: tenant deletion confirmation flow.**

1. Founder presses "Delete tenant".
2. Modal: "This will permanently destroy the tenant's data via crypto-shred after a 30-day grace period."
3. Type-to-confirm: tenant slug.
4. Step-up auth.
5. 2-of-2: Engineering Lead receives Notify; must approve within 7 days.
6. On both signs: tenant moves to `pending_deletion` state for 30 days; daily countdown email; visible in admin console.
7. Within 30 days, Founder can cancel.
8. After 30 days: FR-TEN-002 `delete` flow runs; KMS keys destroyed; tenant becomes unrecoverable.

## Alternatives Considered

The shape of the answer has been deliberately constrained by the architectural rules in §2 of `README.md` and the locked decisions cited in *Dependencies*. Notable rejected approaches:

- Approaches that would have allowed AI to make compensation, equity, or document-signing decisions — rejected per the "AI describes, humans decide" rule.
- Approaches that would have created cross-tenant read or write paths — rejected per the cross-tenant invariant (FR-TEN-001 invariant test harness).
- Where there are FR-specific alternatives, they're discussed inline in *Proposed Solution* and *Constraints*.

<!-- TODO during implementation PR: replace with FR-specific rejected alternatives. -->

## Out of Scope

- Cross-tenant admin (a CyberSkill internal admin overseeing all tenants) — that's a separate `cyberskill-admin` surface, FR-TEN-007 in P4 follow-up.
- Per-module deep configuration (e.g. configuring the PROJ board layout) — stays in the module's own surface.
- Customer support ticket system inside the admin console — handled via direct email or a future support-portal FR.
- Bulk import of users from third-party HRIS — handled by SCIM (T2/T3) or CSV.
- Cost allocation per-cost-center inside the tenant — out of scope; costs are tenant-level only.

## Dependencies

- FR-TEN-001/002/003 (tenancy + lifecycle + theme).
- FR-AUTH-001/002/003 (RBAC + audit + step-up).
- FR-BILL-001 (billing + Stripe Customer Portal).
- FR-INV-002/004 (invoice list + AR aging surface).
- FR-HR-001 (user provisioning).
- FR-CP-001/002/004 (compliance cockpit; DSAR; sub-processor list).
- FR-CP-005 (audit-package generator surface for ISO/SOC).
- FR-INFRA-001 (Module Federation host).
- FR-DESIGN-001 (design tokens + components).
- FR-OBS-002 (admin actions timeline = audit-chain query).
- FR-API-001 (API key management — Pane 6).
- FR-MCP-001 + FR-AI-001 (persona-scope contracts surfaced in Pane 6; AI budget caps in Pane 2).
- FR-BRAIN-003 (archival export pattern).
- FR-CHAT-001 (Notify channels for budget + emergency events).

## Constraints

- **Step-up required for all destructive + financial actions.** Architectural rule.
- **2-of-2 for emergency revoke + tenant delete.** Production tenants only; sandbox tenants 1-of-1 for dev convenience.
- **30-day grace period on tenant deletion.** Cannot be shortened from the UI; requires a manual support escalation to override (and even that is gated).
- **No raw audit-chain export to non-Founder roles.** Members can request their own data via DSAR; the full chain is Founder + ENG-LEAD + DPO only.
- **Custom domain DNS verification cannot be bypassed.** Even Founder cannot skip it; this is a fundamental security control.
- **API token revocation propagates within 60 seconds** to all gateways.

## Compliance / Privacy

- **PDPL Decree 13/2023:** admin console is the surface where the tenant DPO controls retention + sub-processor + DSAR flow.
- **GDPR Article 30 (records of processing):** generated as a downloadable artefact from Pane 7 by FR-CP-001.
- **GDPR Article 32 + ISO 27001 A.5.7:** the audit log + emergency revoke + persona-scope contract surface are core security controls; demonstrated in the SoA.
- **EU AI Act Article 14 + 50:** persona-version + skill-version transparency in Pane 6.
- **SOC 2 CC6 (Logical Access):** the admin console + RBAC matrix is the canonical demonstration; auditor walks through this surface.
- **No AI surface in this FR.** `eu_ai_act_risk_class: not_ai`.

## Risk Assessment (AI-emitting features)

No AI in this FR. The CUO/CFO can answer questions about a tenant ("what was my AI spend last month?") via the standard Genie chat — but the admin console itself is deterministic. Persona-scope contract listing is metadata, not AI.

## Vietnamese-locale considerations

- Full vi-VN translation; all admin language reviewed.
- Anh/Chị honorifics in Notify messages.
- Vietnamese date format (ISO 8601 + dd/mm/yyyy alt).
- VND currency display + Vietnamese tax ID (MST) shown on Billing pane for vn-shard tenants.
- Vietnamese e-invoice download links per Decree 123.

## Scope (acceptance criteria — auditable)

- [ ] `/admin` route exists; loads in Module Federation host shell; lazy-loads each pane on tab click.
- [ ] All 7 panes render with the correct data + actions for the Founder role; RBAC matrix enforced for non-Founder roles.
- [ ] Billing pane: plan tier change (T1 → T2) flows through FR-BILL-001 + Stripe Customer Portal.
- [ ] Team pane: invite + suspend + remove flows work; remove triggers FR-HR-001 + FR-CP-002 termination + DSAR.
- [ ] Brand pane: logo upload + color customisation persists in `tenant.brand_kit`; preview renders correctly.
- [ ] Custom Domain pane: DNS walkthrough completes for `<custom>.cyberos-test.com`; verification + cert auto-issuance works.
- [ ] Security pane: audit log filterable + exportable; emergency revoke flow with 2-of-2 works (mock Engineering Lead approval); API token list/create/revoke works.
- [ ] Data pane: DSAR queue surfaces; full tenant export downloads as signed .zip; per-user export works; tenant-delete confirmation flow with 30-day grace works.
- [ ] Step-up auth: all destructive + financial actions require it; CI test asserts no destructive endpoint accepts a non-step-up token.
- [ ] Audit chain: every admin action appears in the chain within 5 seconds.
- [ ] Admin actions timeline: surfaces the last 20 actions on the Overview pane.
- [ ] Trust Center deep-link works.
- [ ] vi-VN translation complete; native-speaker QA pass.

**Gherkin (PRD §19.18).**

```gherkin
Feature: Tenant deletion has 30-day grace period and 2-of-2 sign-off

  Scenario: Founder initiates tenant deletion
    Given the Founder is on /admin/data
    When they press "Delete this tenant"
    And confirm by typing the tenant slug
    And complete step-up auth via passkey
    Then the tenant moves to "pending_deletion" status
    And the Engineering Lead receives a Notify with the deletion request
    And a 30-day countdown begins
    And the daily countdown email is sent to Founder + Engineering Lead
    When the Engineering Lead approves within 7 days
    Then both signatures are captured in audit-chain
    When 30 days pass without cancellation
    Then FR-TEN-002 delete lifecycle is triggered
    And per-tenant KMS keys are destroyed (crypto-shred)
    And the tenant is unrecoverable

Feature: Emergency revoke pulls all sessions

  Scenario: Founder presses emergency revoke after suspected breach
    Given there are 50 active sessions across the tenant
    When the Founder presses "Revoke all sessions" with reason "breach suspected"
    And step-up + 2-of-2 (Engineering Lead) is completed
    Then within 30 seconds all 50 sessions are invalidated
    And every active user receives a re-login prompt within 60 seconds
    And the audit chain has a "session_emergency_revoke" entry with reason + actors
    And the on-call rota is paged
```

## Success Metrics

- Self-service rate: ≥ 90% of common admin tasks (invite user, change plan, view audit log, etc.) completed without CyberSkill support contact.
- Median time-to-complete a common admin task: ≤ 2 minutes.
- Custom domain verification success rate: ≥ 95% (failures = DNS issues outside CyberOS control).
- Step-up coverage: 100% of destructive/financial endpoints require step-up.
- Emergency revoke SLO: ≤ 30 seconds from CTA to all sessions invalid.

## Sales/CS Summary

<!-- Required when client_visible: true. One paragraph written so a non-engineer can pitch the feature. Plain English. No internal jargon, no module codes, no speculation about future scope. -->

<!-- TODO during implementation PR: write the customer-facing pitch. -->

## Open Questions

- **OQ-TEN-005-01.** Should the audit-log export be per-day file (rotated) or a single rolling file? Default: per-day.
- **OQ-TEN-005-02.** Should we offer "delegated admin" (Founder grants a third party temporary admin access for support)? Default: no at MVP; revisit when partner ecosystem matures.
- **OQ-TEN-005-03.** Should "tenant deletion" include the 7-year financial-records retention for tax / regulator? Default: yes — financial records are exported to cold-storage S3 (separate KMS key, retained 7-10y) before crypto-shred runs on the rest.

## AI Authorship Disclosure

- **Tools used:** Claude Cowork (Anthropic).
- **Scope:** drafted the FR end-to-end against the PRD + SRS; founder reviews and edits before status changes from `ready_for_review`.
- **Human review:** founder (`@stephen-cheng`) — final wording is the founder's responsibility.

## References

- PRD §7.1 TEN module; PRD §14.5.1 P4 entry-gate.
- SRS Decisions Log: DEC-001, DEC-013, DEC-016, DEC-019..DEC-023.
- FR-TEN-001/002/003, FR-AUTH-001/002/003, FR-BILL-001, FR-INV-002/004, FR-HR-001, FR-CP-001/002/004/005, FR-INFRA-001, FR-DESIGN-001, FR-OBS-002, FR-API-001, FR-MCP-001, FR-AI-001, FR-BRAIN-003, FR-CHAT-001.

---

*ai_authorship: co_authored — drafted by Claude Cowork on 2026-05-03.*
