---
title: "CRM — schema (Accounts, Contacts, Deals, Activities, Signals); Apollo subgraph; RLS; Engagement linkage"
author: "@stephen-cheng"
department: engineering
status: ready_for_review
priority: p1
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: infrastructure
eu_ai_act_risk_class: not_ai
target_release: "P1 / 2026-Q4"
client_visible: false
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Stand up the CRM module's schema and Apollo Federation subgraph. Five primitives: **Account** (the company), **Contact** (the person at the account), **Deal** (the in-flight or closed sales opportunity), **Activity** (the structured record of every touchpoint — email out, email in, meeting, call, demo, proposal sent, contract signed, internal note), and **Signal** (the asynchronous record of customer-side events — incoming email surfaced from FR-EMAIL-006, website visit when web tracking arrives in P3, support ticket reference). The schema reciprocally links to **Engagement** (FR-PROJ-007 — when a Deal closes-won, a `proj.engagement.client_account_id` references the CRM Account). Subsequent batch-05 FRs ship the pipeline UX (FR-CRM-002), the AI features (FR-CRM-003), and the HubSpot migration (FR-CRM-004).

## Problem

CyberSkill's HubSpot today is "mostly empty" (PRD §1.1's Origin) — Account Manager activities sit in spreadsheets, deal stage is in someone's head, and proposal acknowledgements live in email threads no one consolidates. The PRD §9.11 commits to "pipeline, accounts, contacts, deals; agent-loggable activity; integration with EMAIL and CHAT" as the CRM scope.

Three failure modes a small team must avoid:

- **No system of record.** Without structured accounts + contacts + deals, "who's our primary at Acme?" varies by who you ask. The PRD's strategic-bet 1 (agent parity) requires *agents* to answer that question; agents need typed data.
- **Activity decay.** A customer dinner happens; the receipt-photo lives in TIME (FR-TIME-003); the conversation summary is in a Slack DM; the follow-up is in a Notion doc. Three months later, no Member can reconstruct the relationship.
- **Deal-stage opacity.** "Acme is at proposal" and "Acme is closing-won" are different commitments to plan around; without a pipeline, sales forecasting is gut-feel.

## Proposed Solution

The shape of the answer is a `crm` schema, an Apollo Federation v2 subgraph, RLS + per-Member ACL, and the Engagement / EMAIL / CHAT seam stubs.

**Schema.**

```sql
CREATE SCHEMA crm;

-- Account: the company (or organisation).
CREATE TABLE crm.account (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  name TEXT NOT NULL,
  slug TEXT NOT NULL,                          -- "acme-corp"
  domain TEXT,                                  -- "acme.example"
  parent_account_id UUID REFERENCES crm.account(id),  -- subsidiaries
  industry TEXT,
  employee_count INT,
  annual_revenue_usd_minor BIGINT,
  region TEXT NOT NULL,                         -- "VN" | "US-CA" | "EU-DE" | etc.
  hq_address TEXT,
  status TEXT NOT NULL DEFAULT 'prospect',      -- "prospect" | "active" | "former" | "churned" | "do_not_contact"
  health_score TEXT,                            -- "green" | "yellow" | "red"; derived (FR-CRM-003)
  primary_contact_id UUID,                      -- references crm.contact (after that table exists)
  primary_owner_member_id UUID NOT NULL,        -- the Account Manager
  vi_honorific_default TEXT,                    -- "Anh" | "Chị" | etc.; for vi-VN composition (FR-EMAIL-005)
  notes_md TEXT,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  archived_at TIMESTAMPTZ,
  UNIQUE (tenant_id, slug)
);

CREATE INDEX account_status_idx ON crm.account (tenant_id, status);
CREATE INDEX account_owner_idx  ON crm.account (tenant_id, primary_owner_member_id);

-- Contact: a person at an account.
CREATE TABLE crm.contact (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  account_id UUID NOT NULL REFERENCES crm.account(id) ON DELETE CASCADE,
  full_name TEXT NOT NULL,
  preferred_name TEXT,
  vi_honorific TEXT,                            -- "Anh"/"Chị"/etc.; overrides account default
  email TEXT,
  phone TEXT,
  linkedin_url TEXT,
  title TEXT,
  role TEXT,                                    -- "decision_maker" | "champion" | "user" | "blocker" | "influencer"
  language_default TEXT,                        -- "vi-VN" | "en-US" | etc.
  timezone TEXT,                                -- IANA
  is_active BOOLEAN NOT NULL DEFAULT true,
  do_not_contact BOOLEAN NOT NULL DEFAULT false,
  notes_md TEXT,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX contact_account_idx ON crm.contact (tenant_id, account_id);
CREATE INDEX contact_email_idx   ON crm.contact (tenant_id, lower(email)) WHERE email IS NOT NULL;

-- Deal: an in-flight sales opportunity.
CREATE TABLE crm.deal (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  account_id UUID NOT NULL REFERENCES crm.account(id) ON DELETE CASCADE,
  primary_contact_id UUID REFERENCES crm.contact(id),
  name TEXT NOT NULL,                           -- "Acme – Onboarding Q3 launch"
  stage TEXT NOT NULL,                          -- "lead" | "discovery" | "proposal" | "negotiation"
                                                -- | "closed_won" | "closed_lost"
  amount_minor BIGINT,
  currency TEXT,
  probability_band TEXT,                        -- "low" | "medium" | "high" — *bands not single values* (FR-CRM-003)
  expected_close_date DATE,
  actual_close_date DATE,
  loss_reason TEXT,                             -- when closed_lost
  competitor TEXT,
  primary_owner_member_id UUID NOT NULL,
  engagement_id UUID,                            -- populated when closed_won → references proj.engagement
  notes_md TEXT,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX deal_stage_idx ON crm.deal (tenant_id, stage);
CREATE INDEX deal_account_idx ON crm.deal (tenant_id, account_id);
CREATE INDEX deal_owner_idx ON crm.deal (tenant_id, primary_owner_member_id);

-- Activity: a structured record of a touchpoint.
CREATE TABLE crm.activity (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  account_id UUID NOT NULL REFERENCES crm.account(id),
  contact_id UUID REFERENCES crm.contact(id),
  deal_id UUID REFERENCES crm.deal(id),
  kind TEXT NOT NULL,                           -- "email_out" | "email_in" | "meeting" | "call"
                                                -- | "demo" | "proposal_sent" | "contract_signed"
                                                -- | "internal_note" | "task" | "review"
  subject TEXT NOT NULL,
  body_md TEXT,
  occurred_at TIMESTAMPTZ NOT NULL,
  created_by_member_id UUID NOT NULL,
  external_refs JSONB NOT NULL DEFAULT '[]'::jsonb,  -- e.g. [{kind: "email_thread", id: ..., url: ...}]
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX activity_account_idx ON crm.activity (tenant_id, account_id, occurred_at DESC);
CREATE INDEX activity_deal_idx    ON crm.activity (tenant_id, deal_id, occurred_at DESC);

-- Signal: asynchronous customer-side events; less structured than activities.
CREATE TABLE crm.signal (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  account_id UUID NOT NULL REFERENCES crm.account(id),
  contact_id UUID REFERENCES crm.contact(id),
  deal_id UUID REFERENCES crm.deal(id),
  kind TEXT NOT NULL,                           -- "email_in" | "support_ticket" | "website_visit" (P3+)
                                                -- | "renewal_due" | "anniversary" | "press_mention"
  summary TEXT NOT NULL,
  sentiment TEXT,                               -- "positive" | "neutral" | "negative"
  source_module TEXT,                           -- "email" | "support" | "external_webhook"
  source_ref TEXT,
  occurred_at TIMESTAMPTZ NOT NULL,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX signal_account_idx ON crm.signal (tenant_id, account_id, occurred_at DESC);
CREATE INDEX signal_kind_idx    ON crm.signal (tenant_id, kind);

-- Stage history (deal stage transitions; queryable for time-in-stage analysis).
CREATE TABLE crm.deal_stage_transition (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  deal_id UUID NOT NULL REFERENCES crm.deal(id) ON DELETE CASCADE,
  from_stage TEXT,
  to_stage TEXT NOT NULL,
  actor_member_id UUID,
  reason_md TEXT,
  occurred_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

**RLS + ACL.**

Every `crm.*` table has tenant RLS. Per-Member ACL: a Member sees CRM data if any of (a) the Member is the `primary_owner_member_id` of the account/deal, (b) the Member has the `crm.read.*` predicate (Founder, Account Manager, Auditor, DPO), (c) the Member has been explicitly added via per-record ACL (introduced in FR-CRM-002 — `crm.account_acl{account_id, member_id, role}`).

**Federation directives.**

- `CrmAccount @key(fields: "id")` — exposes a stub for PROJ (`extend type ProjEngagement { account: CrmAccount @requires("clientAccountId")` lives in PROJ-007).
- `CrmContact @key(fields: "id")` — exposed for EMAIL (FR-EMAIL-006 references contacts).
- `Member @key(fields: "id") @external` — references the AUTH subgraph for `primary_owner_member_id`.

**GraphQL subgraph.**

```graphql
type Query {
  crmAccounts(status: [String!], ownerId: ID, region: String, first: Int = 50): CrmAccountConnection!
  crmAccount(id: ID, slug: String): CrmAccount
  crmAccountByDomain(domain: String!): CrmAccount      # used by EMAIL-006 contact resolver
  crmContacts(accountId: ID, first: Int = 50): CrmContactConnection!
  crmContact(id: ID, email: String): CrmContact
  crmContactByEmail(email: String!): CrmContact        # used by EMAIL-006
  crmDeals(stage: [String!], ownerId: ID, accountId: ID, first: Int = 50): CrmDealConnection!
  crmDeal(id: ID!): CrmDeal
  crmActivities(accountId: ID, dealId: ID, contactId: ID, since: DateTime, first: Int = 50): CrmActivityConnection!
  crmSignals(accountId: ID, kind: [String!], since: DateTime, first: Int = 50): CrmSignalConnection!
  crmSearch(query: String!, kinds: [String!], first: Int = 50): [CrmSearchHit!]!
  crmPipelineForecast(stage: [String!], ownerId: ID): CrmPipelineForecast!  # FR-CRM-003 deals; this returns null in P1 base
}

type Mutation {
  crmCreateAccount(input: CrmAccountInput!): CrmAccount!
  crmUpdateAccount(id: ID!, patch: CrmAccountPatch!): CrmAccount!
  crmArchiveAccount(id: ID!): CrmAccount!

  crmCreateContact(input: CrmContactInput!): CrmContact!
  crmUpdateContact(id: ID!, patch: CrmContactPatch!): CrmContact!
  crmMergeContacts(sourceIds: [ID!]!, targetId: ID!): CrmContact!

  crmCreateDeal(input: CrmDealInput!): CrmDeal!
  crmUpdateDeal(id: ID!, patch: CrmDealPatch!): CrmDeal!
  crmTransitionDealStage(id: ID!, toStage: String!, reasonMd: String): CrmDeal!
  crmCloseDealWon(id: ID!, actualCloseDate: Date!, engagementInputForCreation: ProjEngagementInput): CrmDeal!
  crmCloseDealLost(id: ID!, actualCloseDate: Date!, lossReason: String!, competitor: String): CrmDeal!

  crmCreateActivity(input: CrmActivityInput!): CrmActivity!
  crmUpdateActivity(id: ID!, patch: CrmActivityPatch!): CrmActivity!
  crmDeleteActivity(id: ID!, reason: String!): Boolean!

  crmAddAccountMember(accountId: ID!, memberId: ID!, role: String!): Boolean!
  crmRemoveAccountMember(accountId: ID!, memberId: ID!): Boolean!
}

type Subscription {
  crmAccountStream(accountId: ID!): CrmAccountEvent!
  crmDealStream(ownerId: ID): CrmDealEvent!
}
```

Persisted-queries discipline applies (FR-INFRA-001).

**Engagement linkage.**

When a Deal moves to `closed_won`:
1. The mutation `crmCloseDealWon` requires either an `engagementInputForCreation` (creates a new Engagement) or links to an existing Engagement.
2. New-Engagement path: PROJ creates the Engagement (the call traverses the federation to the PROJ subgraph); the resulting Engagement ID is stored in `crm.deal.engagement_id`.
3. Existing-Engagement path: the `engagement_id` is set; the Engagement's `client_account_id` is updated to point back to this Account.

CUO/CRO surfaces a Notify card "Deal closed-won → create Engagement?" via FR-PROJ-007's hook (already specified there).

**MCP tool surface (read tools in this FR; mutation tools in FR-CRM-002).**

- `cyberos.crm.list_accounts`
- `cyberos.crm.get_account`
- `cyberos.crm.account_by_domain`
- `cyberos.crm.list_contacts`
- `cyberos.crm.get_contact`
- `cyberos.crm.contact_by_email`
- `cyberos.crm.list_deals`
- `cyberos.crm.get_deal`
- `cyberos.crm.list_activities`
- `cyberos.crm.list_signals`
- `cyberos.crm.search`

All `read_only: true`.

**Audit integration.** Every mutation writes an audit row in `crm.{tenant}` scope; deal-stage transitions also write to `crm.deal_stage_transition` for queryable history. Activity writes are not auto-audited individually (volume too high) but their aggregate counts feed the pipeline-health dashboards.

**Seed data.** P1 seed creates 2 Accounts (CyberSkill's two long-term clients per PRD §1.1) with their primary contacts; each is linked to its existing Engagement by `proj.engagement.client_account_id`. The full HubSpot migration (FR-CRM-004) populates the rest.

## Alternatives Considered

- **Hosted CRM (HubSpot, Salesforce, Pipedrive).** Rejected: residency + the Engagement linkage + persona-aware retrieval cannot be enforced with a hosted provider; lock-in is the failure mode.
- **Skip Signal as a separate primitive.** Rejected: signals are asynchronous + lower-confidence; mixing with Activity would muddy the structured-touchpoint semantic.
- **Single people-scoring number on contacts ("lead score").** Rejected: PRD §14.2.1 explicitly says "no scoring of people"; the Bet 1 + EU AI Act concerns about automated scoring are real. The probability bands on Deals (low/medium/high) are intentional; people are not scored.
- **Auto-create Engagement on every closed_won without confirmation.** Rejected: the human-in-the-loop floor; the founder reviews every deal-to-engagement promotion.

## Success Metrics

- **Primary metric.** P1 sprint demo passes: (1) the founder creates an Account + Contact + Deal end-to-end via GraphQL; (2) RLS denies a non-owner Member's read of a private account; (3) `crmAccountByDomain` resolves correctly (used by EMAIL-006 contact resolver); (4) closing a deal-won creates the Engagement linkage atomically across the federation.
- **Coverage metric.** 100% of mutations write an audit row + deal-stage-transition rows.
- **Latency NFR.** `crmAccounts` query p95 ≤ 120 ms; federation cross-subgraph join (Account → Engagement) p95 ≤ 200 ms.

## Scope

**In-scope.**
- The `crm` schema with the seven tables (Account, Contact, Deal, Activity, Signal, deal_stage_transition, account_acl-pre).
- RLS + per-Member ACL composition.
- Apollo Federation v2 subgraph with all queries + mutations + subscriptions.
- Engagement ↔ Account bidirectional linkage.
- Federated entity references for Member / ProjEngagement.
- Seed data for the 2 long-term clients.
- Audit integration in scope `crm.{tenant}`.
- The 11 read-only MCP tools.

**Out-of-scope (deferred to FR-CRM-002 / FR-CRM-003 / FR-CRM-004).**
- Pipeline UX + forecast confidence-bands rendering (FR-CRM-002).
- AI features: next-action drafter, deal-aware reply suggestions, "what's the typical hold-up at proposal stage" (FR-CRM-003).
- Mutation MCP tools (FR-CRM-002).
- HubSpot migration (FR-CRM-004).
- Per-record ACL UX (FR-CRM-002 ships the table; UX in same FR).
- Signal-source webhook receivers beyond EMAIL (P2/P3).

## Dependencies

- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-002 / FR-MCP-001.
- FR-PROJ-001 / FR-PROJ-007 (Engagement linkage).
- FR-DESIGN-001 (no UI in this FR; subsequent FRs consume).
- Compliance: PDPL Decree 13 (CRM data is heavily personal-data-loaded; per-tenant residency + RLS + the BRAIN ingestion denylist on activity texts apply); GDPR (P3) for EU contacts.
- Locked decisions referenced: DEC-145 (five primitives: Account/Contact/Deal/Activity/Signal), DEC-146 (no people-scoring; only deal probability bands), DEC-147 (deal-won → Engagement requires human confirmation).

## AI Risk Assessment

Not applicable. `eu_ai_act_risk_class: not_ai`. The schema + GraphQL are deterministic; AI-derived health-score + forecast-bands surfaces live in FR-CRM-003.
