---
title: "PORTAL — client portal foundation: external auth, project visibility, document hub, status timeline"
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

Stand up the **PORTAL** module — the first surface in CyberOS that is read by *external counterparties* (clients of a tenant, not the tenant's own employees). PORTAL is a **multi-tenant external surface** that lets a client of a CyberSkill-or-other-consultancy tenant see exactly the slices of that tenant's CyberOS data that have been explicitly published to them: project status, deliverables, invoices, signed documents, knowledge-base pages, and a timeline of what's been done. Five primitives ship in this FR: (1) a separate AUTH realm `portal.<tenant>.cyberos.world` with passkey-first + magic-link fallback (no SSO required for clients); (2) a `portal.workspace` schema scoped to "external counterparty × tenant × engagement", with explicit allow-listing of which CyberOS objects are visible; (3) a frontend remote at `/portal` (different host shell from the internal `/` shell) running Module Federation with the public design tokens (per-tenant theme via FR-TEN-003); (4) a deliberately narrow MCP surface — read-only, with the `client` persona-scope contract — so any AI-touchpoints are agent-parity-safe; (5) the **publication-and-revocation** flow that lets a tenant employee push (or revoke) an internal CyberOS object's visibility into a counterparty's workspace, with full audit. PORTAL is the *unique* CyberOS surface that runs both inside the tenant's residency shard and at the edge of the public Internet — it is the place where the multi-tenant + residency + crypto-shred work from FR-TEN-001..003 meets a real human typing into a browser they don't own.

## Problem

PRD §7.1 (PORTAL — module 21 of 22) names this as the "external client portal" with explicit scope: "letters, deliverables, signed contracts, invoice hub, status timeline, and Q&A — all read-only by default and explicitly published from inside CyberOS by a tenant employee." PRD §14.5.1 P4 entry-gate criterion includes "PORTAL has been used by ≥ 1 paying CyberSkill client for ≥ 30 consecutive days with NPS ≥ 7 from the client side."

Three failure modes if this is not built carefully on the multi-tenant substrate:

- **Cross-tenant client leakage.** Without a hard `portal.workspace` × residency-shard × tenant constraint, a client of TenantA could in principle see content from TenantB. This is the most reputation-destroying class of bug for a SaaS at this stage.
- **Publication accidents.** An internal employee accidentally drags an internal-only document (e.g. a project retrospective with frank assessments of the client) into the wrong publication scope. Once a counterparty has loaded the page, it's been read; revocation is a partial control.
- **External AUTH attack surface.** External clients don't use the tenant's identity provider. A separate AUTH realm has to be just as hardened as the internal one — without falling back to "username + password" which is the default failure mode in lazy implementations.

## Customer Quotes

<!-- Required when client_visible: true. Verbatim, attributed where possible. Paraphrasing here costs you the signal. -->

<untrusted_content source="other">
…paste verbatim customer quote here…
</untrusted_content>

<!-- TODO during implementation PR: capture real customer quotes from sales calls / NPS / support tickets. -->

## Proposed Solution

The shape of the answer is a tightly-scoped `portal.*` schema + a separate AUTH realm + a separate frontend remote on a separate host shell + a narrow MCP surface + an explicit publication-revocation lifecycle.

**Schema.**

```sql
-- The external counterparty's workspace, scoped to a tenant + an engagement.
CREATE TABLE portal.workspace (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,                                                  -- the publishing tenant (e.g. CyberSkill JSC)
  engagement_id UUID NOT NULL REFERENCES proj.engagement(id),                -- 1:1 with a CRM Engagement
  client_account_id UUID NOT NULL REFERENCES crm.account(id),                -- the receiving client account
  display_name TEXT NOT NULL,                                                -- "CyberSkill ↔ Acme — Project Alpha"
  vanity_slug TEXT NOT NULL,                                                 -- portal.<tenant>.cyberos.world/<slug>
  brand_kit_id UUID REFERENCES tenant.brand_kit(id),                         -- inherits tenant brand kit by default; can override
  status TEXT NOT NULL DEFAULT 'active',                                     -- "active" | "paused" | "archived" | "revoked"
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  archived_at TIMESTAMPTZ,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  UNIQUE (tenant_id, vanity_slug)
);

-- An external user (a counterparty's employee) authorised to access a workspace.
CREATE TABLE portal.workspace_user (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  workspace_id UUID NOT NULL REFERENCES portal.workspace(id),
  email CITEXT NOT NULL,                                                      -- their work email
  display_name TEXT NOT NULL,
  role TEXT NOT NULL DEFAULT 'viewer',                                        -- "viewer" | "approver" | "admin" (rarely "admin"; reserved for the client-side counterparty admin)
  passkey_credential_ids JSONB NOT NULL DEFAULT '[]'::jsonb,                  -- WebAuthn credential IDs
  magic_link_email_enabled BOOLEAN NOT NULL DEFAULT true,                     -- fallback when no passkey set
  invited_by_tenant_user_id UUID NOT NULL,
  invited_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  last_login_at TIMESTAMPTZ,
  status TEXT NOT NULL DEFAULT 'invited',                                     -- "invited" | "active" | "suspended" | "removed"
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  UNIQUE (workspace_id, email)
);

-- A published object: any CyberOS internal object that has been explicitly made visible.
CREATE TABLE portal.publication (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  workspace_id UUID NOT NULL REFERENCES portal.workspace(id),
  object_kind TEXT NOT NULL,                                                  -- "kb_page" | "doc_signed" | "invoice" | "project_status" | "deliverable" | "qa_thread"
  object_id UUID NOT NULL,                                                    -- the underlying internal object's ID
  source_module TEXT NOT NULL,                                                -- "kb" | "doc" | "inv" | "proj" | …
  published_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  published_by_tenant_user_id UUID NOT NULL,
  visibility_window TSTZRANGE,                                                -- when published; NULL = "from publish until revocation"
  revoked_at TIMESTAMPTZ,
  revoked_by_tenant_user_id UUID,
  revoke_reason_md TEXT,
  pinned BOOLEAN NOT NULL DEFAULT false,                                      -- pinned to top of workspace home
  display_order INT,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);

-- Per-publication access log (audit at the read-side; complements FR-AUTH-002 audit chain).
CREATE TABLE portal.publication_access_log (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  publication_id UUID NOT NULL REFERENCES portal.publication(id),
  workspace_user_id UUID NOT NULL REFERENCES portal.workspace_user(id),
  accessed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  ip_address INET,
  user_agent TEXT,
  duration_seconds INT,                                                       -- if measurable; NULL otherwise
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);

-- Q&A thread: a counterparty asks a question; tenant answers.
CREATE TABLE portal.qa_thread (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  workspace_id UUID NOT NULL REFERENCES portal.workspace(id),
  subject TEXT NOT NULL,
  asked_by_workspace_user_id UUID NOT NULL,
  asked_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  status TEXT NOT NULL DEFAULT 'open',                                        -- "open" | "answered" | "closed"
  closed_at TIMESTAMPTZ,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);

CREATE TABLE portal.qa_message (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  thread_id UUID NOT NULL REFERENCES portal.qa_thread(id),
  author_kind TEXT NOT NULL,                                                  -- "workspace_user" | "tenant_user"
  author_id UUID NOT NULL,
  body_md TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  attachment_blob_ids UUID[],
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);
```

**RLS** is enabled on every `portal.*` table with two policies: (a) the workspace's tenant employees see all rows where `tenant_id = current_tenant()`; (b) external counterparty users see only rows where `workspace_id IN (SELECT workspace_id FROM portal.workspace_user WHERE id = current_external_user())`. The `current_external_user()` function is set per-request via the JWT issued by the portal AUTH realm.

**External AUTH realm.**

A separate Keycloak realm `portal-{tenant_slug}` per tenant, served at `portal.<tenant_vanity_domain>` (e.g. `portal.cyberskill.world`). The realm is configured for:

- **Passkey-first.** WebAuthn registration on first login; second-factor not required (the passkey is the credential).
- **Magic-link fallback.** Email-link with 15-min expiry, single-use, IP-pinned for the link consumption (relax by configuration).
- **No password authentication.** Period. This is a hard architectural rule because counterparty users are typically email + passkey + browser-based; passwords add reuse-attack risk without value.
- **Session length.** 30 days idle, 90 days absolute. Step-up required for "view sensitive doc" (signed contracts) and "submit Q&A".
- **Federation: no.** This realm does not federate to the counterparty's IdP at MVP; FR-PORTAL-004 (P4 follow-up, not in this batch) adds optional SAML/OIDC federation per workspace.
- **Brute-force protection.** Per-email + per-IP rate limit on magic-link issuance (5/hour). Anomaly Notify to the tenant admin if a workspace user has > 3 failed attempts in 1 hour.

**Frontend remote.**

`portal-shell` is a separate Module Federation **host** (not a remote inside the internal shell). It runs at `portal.<tenant_vanity_domain>` and is served from the tenant's residency shard. The shell:

- Loads only `portal-views`, `auth-views`, and `design-system` remotes — no internal modules. This is a hard rule: no `proj`, `crm`, `kb`, `email`, `chat`, `okr`, `hr`, `rew`, `learn`, `esop`, `inv`, `res`, `time`, `genie`, or `brain` remote can be loaded into `portal-shell`.
- Renders the workspace home (status timeline, pinned publications), the publication detail views (one per object kind), the Q&A thread list, and the user's account settings.
- Uses the per-tenant brand kit (FR-TEN-003); tenants may further override via `portal.workspace.brand_kit_id`.
- Vietnamese-locale supported as for internal surfaces (Be Vietnam Pro typography, vi-VN as default for VN-shard tenants, Anh/Chị honorifics).

**Publication-revocation lifecycle.**

A tenant employee with the `client_publisher` role can publish an internal object to a specific workspace. Each object kind has a publish-side adapter:

- `kb_page` → renders the page as a snapshot; outbound links to other internal pages are either also published or rendered as "internal — not available to you".
- `doc_signed` → renders the signed PDF + the signature certificate (FR-DOC-001); never the unsigned drafts.
- `invoice` → renders the invoice PDF + payment status; never AR aging or other AP context.
- `project_status` → renders a curated status summary (Done / In-progress / Next / Risks) authored by the tenant employee; never the raw PROJ board.
- `deliverable` → renders a file or a list of files; clicking downloads through a presigned URL with a 24-hour TTL.
- `qa_thread` → not a publication per se; it's a created object within the workspace.

Publication is reversible via `revoked_at` + `revoke_reason_md`; once revoked, the object is removed from the workspace UI immediately, but the audit trail (who-saw-what-when) is preserved. The system surfaces "this publication was revoked on YYYY-MM-DD" if the user follows a stale link — never silently 404.

**Narrow MCP surface.**

A new persona-scope contract `client` is added to the AI Gateway + MCP Gateway:
- Tools available: `portal.list_publications`, `portal.get_publication`, `portal.list_qa_threads`, `portal.post_qa_message` (write, narrow).
- Tools NOT available: anything from internal modules. The defence-in-depth check is enforced at AI Gateway, MCP Gateway, AND every server.
- The CUO/CXO emergent skill (FR-PORTAL-003) operates within this scope when invoked from the portal.

**Status timeline.**

The portal home renders a vertical timeline of "what's been published, what's been signed, what's been delivered, what's pending response from you". Each item is sourced from `portal.publication` or `portal.qa_thread` rows. The timeline is the primary surface; it answers the typical client question "what's the latest?".

## Alternatives Considered

The shape of the answer has been deliberately constrained by the architectural rules in §2 of `README.md` and the locked decisions cited in *Dependencies*. Notable rejected approaches:

- Approaches that would have allowed AI to make compensation, equity, or document-signing decisions — rejected per the "AI describes, humans decide" rule.
- Approaches that would have created cross-tenant read or write paths — rejected per the cross-tenant invariant (FR-TEN-001 invariant test harness).
- Where there are FR-specific alternatives, they're discussed inline in *Proposed Solution* and *Constraints*.

<!-- TODO during implementation PR: replace with FR-specific rejected alternatives. -->

## Out of Scope

- Real-time chat with the client (PORTAL is async-only at MVP; client-tenant chat is deferred — it would need CHAT to expose external rooms, which is not in PRD).
- File upload from the client side (FR-PORTAL-005, deferred to P4 follow-up).
- E-signature initiated by the client (the client signs documents created by the tenant via FR-DOC-001's signing surface; the surface is shared but the workflow originates inside the tenant).
- Financial transactions initiated by the client (paying invoices is a deep-link to an external Stripe/VNPay/Wise checkout, not a payment surface inside PORTAL).
- White-label PORTAL hosting on the client's own domain (`portal.client.com` instead of `portal.tenant.com`) — this is a T3 white-label feature in FR-TEN-003, not duplicated here.
- Federated SSO with the client's IdP (deferred to FR-PORTAL-004).
- Multi-language dynamic translation for publications (publications are rendered in their authored language; a tenant employee chooses to publish a Vietnamese version + an English version separately if both are needed).

## Dependencies

- FR-AUTH-001 (RBAC + RLS — extended with the `external_user` row-class).
- FR-AUTH-002 (Merkle audit chain — `portal.publication_access_log` cross-references).
- FR-AUTH-003 (step-up auth — required for publication of sensitive docs + signing).
- FR-INFRA-001 (Module Federation host shell pattern reused for `portal-shell`).
- FR-DESIGN-001 (design tokens + component library — public subset for PORTAL).
- FR-TEN-001 (residency partitioning — `portal-shell` runs on the tenant's shard).
- FR-TEN-003 (per-tenant theming + custom domains — `portal.<tenant>.cyberos.world` routing).
- FR-DOC-001 (e-signature — `doc_signed` publication adapter).
- FR-INV-004 (`invoice` publication adapter — invoice PDF + status).
- FR-PROJ-005 (`project_status` publication adapter — curated status surfacing).
- FR-KB-003 (`kb_page` publication adapter — page snapshot).
- FR-CRM-001 (`portal.workspace.client_account_id` references).
- FR-PROJ-001 (`portal.workspace.engagement_id` references).
- FR-MCP-001 (the `client` persona-scope contract addition).
- DEC-001 multi-tenant from day one; DEC-013 schema-per-tenant + RLS; DEC-016 passkey-first.

## Constraints

- **No internal-module remote can load in `portal-shell`.** Enforced at Module Federation manifest level + at the shell's runtime guard.
- **External AUTH realm cannot use passwords.** Configured at Keycloak realm level; CI test asserts no password authenticator is enabled.
- **Cross-tenant invariant.** The workspace × tenant constraint is at row level via RLS; FR-TEN-001's invariant test harness extends to cover `portal.*` tables.
- **No raw internal object IDs leak to PORTAL URLs.** Every publication URL uses an opaque slug; client-side cannot guess `/portal/kb/<internal_id>`.
- **Magic-link fallback required.** Even with passkey enabled, every workspace user must be able to fall back to magic-link if their device is lost. Accepted security trade-off for usability.
- **Anti-screenshot warning, not enforcement.** PORTAL renders a watermark on sensitive pages (signed docs, deliverables) with the client user's email; not a true DRM control, but a deterrent + forensic signal.
- **No AI in publication decisions.** The CUO/CXO can suggest "this KB page might be useful to publish" but the tenant employee always makes the call.

## Compliance / Privacy

- **PDPL Decree 13/2023:** workspace user emails + access logs are personal data of the *counterparty's* employees, not the tenant's; PDPL applies; the tenant's DPIA must enumerate the counterparty as a separate data subject category.
- **Cross-border transfer:** if the workspace is on EU shard but the counterparty is in VN, or vice versa, a Schrems II TIA applies (already in FR-CP-004 scope).
- **Data subject rights for external users:** workspace users are fully entitled to DSAR via the external-subject DSAR portal (FR-CP-004); the FR-CP-004 portal accepts requests scoped by counterparty email.
- **GDPR Article 5 storage limitation:** `portal.publication_access_log` is retained for 24 months then auto-archived to S3 for the regulatory minimum; configurable per tenant within statutory floors.
- **EU AI Act:** no AI surface in this FR (the CXO surface is in FR-PORTAL-003); `eu_ai_act_risk_class: not_ai`.
- **ISO/IEC 27001 A.5.10/A.8.10:** "external party access controls" — this FR is the canonical implementation; documented in the SoA (Statement of Applicability) at FR-CP-005.

## Risk Assessment (AI-emitting features)

No AI surface in this FR. The portal is read-mostly + Q&A. AI surface is in FR-PORTAL-003.

## Vietnamese-locale considerations

- Default locale for VN-shard portals: `vi-VN`. Counterparty user can override in their account settings.
- Honorifics: Anh/Chị + first name in tenant-employee → workspace-user salutations.
- Be Vietnam Pro typography for vi-VN; Inter for en-US; per the design tokens.
- Vietnamese e-invoice fapiao numbers + VAT IDs render correctly in `invoice` publications (FR-INV-004 + FR-REW-004).
- Q&A threads support Vietnamese tokenisation via PGroonga at the search layer.

## Scope (acceptance criteria — auditable)

- [ ] `portal.*` schema migration applied; all tables have RLS enabled with the (tenant, workspace) two-policy structure.
- [ ] FR-TEN-001 invariant test harness extended to verify zero cross-tenant leakage for every `portal.*` table.
- [ ] `portal-shell` is a separate Module Federation host; CI test asserts no internal remote (proj/crm/kb/etc.) can load inside it.
- [ ] Per-tenant Keycloak realm `portal-{tenant_slug}` provisioned automatically when a tenant first creates a workspace; password authenticator disabled at realm config level.
- [ ] WebAuthn passkey registration flow + magic-link fallback flow both work end-to-end on the demo workspace.
- [ ] First publication adapters live: `kb_page`, `doc_signed`, `invoice`, `project_status`, `deliverable`. Each renders in the workspace UI.
- [ ] Publication revocation: revoking a publication removes it from the UI within 60 seconds (cache TTL ceiling); audit trail preserved.
- [ ] Stale revoked link returns a "this publication has been removed" page, not a 404.
- [ ] Q&A thread create + post-message flow works for both directions (workspace user ↔ tenant employee).
- [ ] Per-publication access log captures every read with IP + UA + duration estimate.
- [ ] Tenant brand kit applied via FR-TEN-003 by default; `portal.workspace.brand_kit_id` override works when set.
- [ ] vi-VN locale renders Be Vietnam Pro; date format ISO 8601; honorifics in salutations.
- [ ] DSAR portal (FR-CP-004) accepts requests from counterparty emails and resolves them across `portal.workspace_user` + `portal.publication_access_log`.
- [ ] First end-to-end test: a CyberSkill internal employee provisions a workspace for a synthetic Acme Corp account, invites a synthetic acme.com counterparty user, the user completes passkey registration, opens the workspace, sees a published `kb_page` + a published `doc_signed`, asks a Q&A question, the employee replies, the user reads the reply.

**Gherkin (PRD §19.18).**

```gherkin
Feature: PORTAL never leaks data across tenants

  Scenario: Counterparty user A of Tenant1 attempts to fetch a publication of Tenant2
    Given Tenant1 workspace W1 has a published kb_page P1
    And Tenant2 workspace W2 has a published kb_page P2
    And Counterparty user A is authorised on W1 only
    When A's session sends GET /portal/p/<P2 opaque slug>
    Then the response is 404
    And the audit log records "cross_workspace_access_denied" with A's user_id and P2's id
    And no portal.publication_access_log row is inserted (the read never reached P2)

Feature: Publication revocation removes immediately

  Scenario: Tenant employee revokes a publication after counterparty has loaded but not closed the tab
    Given counterparty user A has the publication detail page open
    And Tenant employee E presses Revoke on the publication
    Then within 60 seconds, A's UI re-fetch returns 410
    And A's UI shows "This publication has been removed by the publisher; reason: <revoke_reason_md>"
    And the publication is no longer listed on A's workspace home
    And the original audit log of A's reads BEFORE the revoke is preserved
```

## Success Metrics

- First-pilot acceptance: tenant employee onboards 1 client to PORTAL, publishes ≥ 5 objects across ≥ 3 object kinds, the client returns to the portal ≥ 5 times in the first 14 days.
- Cross-tenant leakage events in invariant tests: zero.
- Magic-link issuance rate: ≤ 5 per workspace user per hour without anomaly trigger.
- Mean time to publish: ≤ 30 seconds from "click publish" to publication-visible-in-portal.
- Publication revocation SLO: ≤ 60 seconds from revoke action to disappearance from UI.
- vi-VN render correctness: zero typography or honorific bugs in QA pass.

## Sales/CS Summary

<!-- Required when client_visible: true. One paragraph written so a non-engineer can pitch the feature. Plain English. No internal jargon, no module codes, no speculation about future scope. -->

<!-- TODO during implementation PR: write the customer-facing pitch. -->

## Open Questions

- **OQ-PORTAL-001-01.** Should PORTAL support file uploads from the counterparty (FR-PORTAL-005) at MVP or as a follow-up? Default: follow-up (post-batch-10).
- **OQ-PORTAL-001-02.** Should the `client_publisher` role be assignable to the CUO via a "draft + ask Founder to confirm" pattern, or kept human-only? Default: human-only at MVP.
- **OQ-PORTAL-001-03.** Should PORTAL access logs be exposed to the counterparty as a "who from your team has seen what" view? Default: no (security through opacity); revisit in a P4 follow-up.

## AI Authorship Disclosure

- **Tools used:** Claude Cowork (Anthropic).
- **Scope:** drafted the FR end-to-end against the PRD + SRS; founder reviews and edits before status changes from `ready_for_review`.
- **Human review:** founder (`@stephen-cheng`) — final wording is the founder's responsibility.

## References

- PRD §7.1 (PORTAL — module 21).
- PRD §14.5.1 P4 entry-gate criterion (PORTAL pilot ≥ 30 days NPS ≥ 7).
- SRS Decisions Log: DEC-001, DEC-013, DEC-016.
- FR-AUTH-001/002/003, FR-INFRA-001, FR-DESIGN-001, FR-TEN-001/003, FR-DOC-001, FR-INV-004, FR-PROJ-005, FR-KB-003, FR-CRM-001, FR-PROJ-001, FR-MCP-001, FR-CP-004.

---

*ai_authorship: co_authored — drafted by Claude Cowork on 2026-05-03 against PRD §7.1 + §14.5.1.*
