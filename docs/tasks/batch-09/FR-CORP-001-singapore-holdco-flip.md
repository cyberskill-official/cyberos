---
title: "CORP — Singapore HoldCo flip: legal entity tags, IP licence re-issuance, tenant ownership migration, audit-log discipline"
author: "@stephen-cheng"
department: legal
status: ready_for_review
priority: p3
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: infrastructure
eu_ai_act_risk_class: not_ai
target_release: "P3 / 2027-Q4"
client_visible: false
---

# Summary

Execute the **Singapore HoldCo flip** that PRD §14.4.1 schedules for P3 entry: incorporate `cyberskill_pte_ltd` (Singapore HoldCo) as the new top-of-stack entity; flip the existing `cyberskill_jsc` (Vietnamese OpCo) to a wholly-owned subsidiary of the HoldCo; **re-issue all IP licences** so the platform's IP (CyberOS source code, design system, persona Skills, the Genie mascot trademark) is held by the HoldCo and licensed to the OpCo; **migrate tenant ownership** — Vietnamese tenants continue contracting with the OpCo (Vietnamese VAT-eligible invoicing); international tenants contract with the HoldCo (Singapore-tax-resident invoicing); **per-shard tenant-billing-entity routing** — vn-shard tenants billed through the OpCo; sg/eu/us-shard tenants billed through the HoldCo; **audit-log discipline** preserves the entity-of-record at every audit row; the **legal entity tag** on every contract + invoice + employee record + customer-facing artefact reflects the post-flip state. The flip is **for fundraising + tax efficiency only** per PRD §2.5 — the centre of gravity stays in Ho Chi Minh City; the Vietnamese OpCo retains the engineering talent + product development.

# Problem

PRD §14.4.1 P3 scope: "Singapore HoldCo flip executed; legal entity tags migrated; IP licences re-issued; tenant ownership flipped in audit log." PRD §14.4.2 P3 → P4 exit gate: "Singapore HoldCo flip is fully closed; cyberskill_pte is the canonical owner; cyberskill_jsc is a wholly-owned subsidiary." PRD §2.5 Anti-positioning: "Not a US-headquartered company. CyberSkill JSC is and will remain a Vietnamese-incorporated company with Vietnamese cultural identity. The Singapore HoldCo flip at P3 entry is for fundraising and tax efficiency only; the centre of gravity stays in Ho Chi Minh City."

Three failure modes the platform must structurally avoid:

- **Entity-tag drift on existing artefacts.** Pre-flip contracts + invoices + employee records reference `cyberskill_jsc`; post-flip new artefacts reference `cyberskill_pte`; without explicit migration + tagging, audit reconstruction across the flip date becomes impossible.
- **IP-licensing gap.** If IP isn't formally re-issued from JSC → PTE → JSC (with the OpCo licensing back from the HoldCo), the company structure is fragile + investors will not accept it during diligence.
- **Tenant-billing-entity confusion.** A Vietnamese tenant invoiced from Singapore violates Vietnamese tax law; an EU tenant invoiced from Vietnam misses Singapore-tax-treaty advantages. Routing per-shard is the structural answer.

# Proposed Solution

**Schema additions.**

```sql
CREATE TABLE cyberos_meta.legal_entity (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  entity_code TEXT NOT NULL UNIQUE,                                       -- "cyberskill_jsc" | "cyberskill_pte_ltd"
  legal_name TEXT NOT NULL,                                                -- "CYBERSKILL SOFTWARE SOLUTIONS CONSULTANCY..."
                                                                         -- | "CYBERSKILL PTE. LTD."
  jurisdiction_country TEXT NOT NULL,                                      -- "VN" | "SG"
  registered_address TEXT NOT NULL,
  duns_number TEXT,
  tax_id TEXT NOT NULL,                                                    -- VN MST or SG UEN
  bank_account_metadata JSONB,                                              -- bank details for AR receivables
  is_active BOOLEAN NOT NULL DEFAULT true,
  effective_from DATE NOT NULL,
  superseded_role TEXT,                                                    -- "operating_company" | "holding_company" | "subsidiary"
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Per-tenant + per-shard billing entity assignment.
CREATE TABLE cyberos_meta.tenant_billing_entity (
  tenant_id UUID NOT NULL REFERENCES cyberos_meta.tenant(id) ON DELETE RESTRICT,
  effective_from DATE NOT NULL,
  effective_to DATE,
  legal_entity_id UUID NOT NULL REFERENCES cyberos_meta.legal_entity(id),
  reason_md TEXT NOT NULL,
  signed_off_at TIMESTAMPTZ NOT NULL,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  PRIMARY KEY (tenant_id, effective_from)
);

-- IP licence registry between entities.
CREATE TABLE cyberos_meta.ip_licence (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  licensor_entity_id UUID NOT NULL REFERENCES cyberos_meta.legal_entity(id),
  licensee_entity_id UUID NOT NULL REFERENCES cyberos_meta.legal_entity(id),
  ip_kind TEXT NOT NULL,                                                    -- "platform_source_code" | "design_system"
                                                                          -- | "persona_skills" | "genie_mascot_trademark"
                                                                          -- | "process_documentation"
  licence_kind TEXT NOT NULL,                                                -- "exclusive" | "non_exclusive" | "perpetual" | "term"
  effective_from DATE NOT NULL,
  effective_to DATE,
  consideration_md TEXT NOT NULL,                                            -- the consideration (royalty, equity, intercompany)
  signed_doc_id UUID NOT NULL,                                               -- the FR-DOC-001 envelope ref
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

**Entity tagging on artefacts.**

Every artefact-producing module (DOC + INV + REW + ESOP + LEARN + HR contracts) gets a `legal_entity_id` field added (or a `metadata.legal_entity_id` for less-modified tables). At provisioning + at flip-execution-time, the field is populated.

The `legal_entity_id` is a function of:
- The artefact's date.
- The artefact's tenant's residency-shard.
- The artefact kind.
- The flip-execution date.

Pre-flip: all artefacts reference `cyberskill_jsc`.
Post-flip:
- Vietnamese tenants → all artefacts reference `cyberskill_jsc` (the OpCo continues; tax-residency unchanged).
- International tenants → all artefacts reference `cyberskill_pte` (Singapore HoldCo).
- Internal artefacts (employment contracts of Vietnamese employees) → `cyberskill_jsc` (the OpCo employs the team).

**Flip execution flow.**

1. **Pre-flip preparation (3-6 months before).**
   - Singapore legal counsel engaged; HoldCo incorporation initiated.
   - SG bank account opened in HoldCo's name.
   - SG UEN issued.
   - DUNS number for HoldCo.
2. **Day-0 corporate actions.**
   - Founder transfers OpCo shares to HoldCo (the founder receives equivalent HoldCo shares; OpCo becomes 100% subsidiary of HoldCo).
   - Vietnamese regulator notification + approval (typically 30-60 days; Department of Investment in HCMC for foreign-investment notification).
   - IP licences signed: HoldCo licences platform IP to OpCo for OpCo's customer-facing operations.
3. **Day-N platform actions (after corporate actions complete).**
   - `cyberos_meta.legal_entity` rows for both entities created + active.
   - Tenant billing-entity routing pre-computed: vn-shard tenants → `jsc`; sg/eu/us-shard tenants → `pte`.
   - Per-tenant Notify card explaining the change ("Your billing entity has changed from Vietnamese OpCo to Singapore HoldCo; your invoices going forward will reference the HoldCo. Vietnamese tenants are unaffected.").
   - `cyberos_meta.tenant_billing_entity` rows created with `effective_from: <flip-date>` and the prior implicit `cyberskill_jsc` row gets `effective_to: <flip-date - 1>`.
4. **Audit-log discipline.**
   - Every audit row from flip-date onward includes the `legal_entity_id` of the issuing party.
   - The `audit.entry.metadata` includes `legal_entity_at_event_time` for chain-of-custody.
   - Pre-flip audit rows are preserved as-is (immutable); a synthetic audit row at flip-date documents the transition.
5. **Tenant-facing migration.**
   - Each existing tenant is offered a "novation": their existing service agreement transfers from JSC → PTE if they're international (with their consent + co-signing). Vietnamese tenants stay on JSC contracts.
   - Tenants with active subscriptions: Stripe customer is migrated from JSC's Stripe account to PTE's Stripe account; subscriptions continue without interruption.
6. **Post-flip ongoing.**
   - All new tenants billed per the routing rules.
   - Vietnamese-tax-eligible invoices via VNPay routing through JSC remain unchanged.
   - The OpCo's Vietnamese employee compensation continues unchanged (employment continuity preserved).

**IP licence re-issuance.**

Specific IP licences signed at flip:

| IP | Licensor | Licensee | Kind | Term |
|---|---|---|---|---|
| Platform source code | HoldCo (PTE) | OpCo (JSC) | Non-exclusive perpetual | Perpetual; royalty-free for intercompany |
| Design system + tokens | HoldCo (PTE) | OpCo (JSC) | Non-exclusive perpetual | Perpetual |
| Persona Skills + CUO IP | HoldCo (PTE) | OpCo (JSC) | Exclusive (per OpCo's customer base) perpetual | Perpetual |
| Genie mascot trademark | HoldCo (PTE) | OpCo (JSC) | Non-exclusive perpetual | Perpetual; OpCo customer-facing use |

The licence terms are signed via FR-DOC-001 envelopes (using QES tier for cross-border legal validity).

**Tax + accounting flow.**

- **Vietnamese tenants → JSC.** Invoiced in VND by JSC; VAT eligible per Vietnamese rules; CIT (corporate income tax) applies to JSC's profits.
- **International tenants → PTE.** Invoiced in USD by PTE; PTE's profit subject to Singapore CIT (currently 17%; with possible startup tax exemptions for first 3 years).
- **Royalty + management fee structure.** PTE licenses platform IP to JSC for a market-rate royalty; JSC pays PTE the royalty; this is the legitimate transfer-pricing flow that funnels international-customer revenue (collected by PTE) to operational costs (paid by JSC; including the team's salaries).
- The royalty is set at fair-market levels per Singapore + Vietnamese transfer-pricing rules; reviewed annually by both jurisdictions' accountants.

**Frontend additions.**

`/platform/admin/corporate-structure` (founder + Engineering Lead + Auditor only):

- Active legal entities + their roles + tax IDs.
- Tenant billing-entity assignment table.
- IP licence library.
- Flip execution audit-log timeline.

# Alternatives Considered

- **Skip the HoldCo flip.** Rejected: PRD §14.4 explicitly schedules; international fundraising + customer procurement at scale require a non-Vietnamese-incorporated holding entity.
- **Delaware C-Corp instead of Singapore PTE.** Considered + rejected: Singapore is the natural choice for Asia-Pacific operations + closer to the team's time zone + tax treaty with Vietnam + ASEAN integration; Delaware would bias toward US fundraising at the cost of regional alignment.
- **Skip the tenant-novation step; let existing tenants stay on JSC contracts.** Rejected: international tenants on JSC contracts means international revenue accruing to Vietnamese tax-residency; the flip becomes economically pointless without novation.
- **AI-suggest tenant-novation language.** Rejected: legal commitments are out of CUO scope.

# Success Metrics

- **Primary metric.** P3 → P4 exit-gate (PRD §14.4.2): "Singapore HoldCo flip is fully closed; cyberskill_pte is the canonical owner; cyberskill_jsc is a wholly-owned subsidiary."
- **Compliance metric.** 100% of post-flip invoices reference the correct legal entity; 100% of audit rows include `legal_entity_id`.
- **Operational metric.** Zero tenant-facing billing disruption during the migration window.

# Scope

**In-scope.**
- The 3 schema additions (`legal_entity`, `tenant_billing_entity`, `ip_licence`).
- Entity-tagging on DOC + INV + REW + ESOP + LEARN + HR + audit-log artefacts.
- Pre-flip / Day-0 / Day-N execution playbook.
- IP licence registry + 4 key IP licences signed via FR-DOC-001.
- Tenant-novation flow for international tenants.
- Per-shard tenant-billing-entity routing.
- Stripe + VNPay account migration scripts.
- `/platform/admin/corporate-structure` frontend.
- Audit integration in scope `platform.corporate.{global}`.
- Royalty-flow accounting + transfer-pricing review cadence (annual).

**Out-of-scope (deferred).**
- US Delaware subsidiary if needed for SOC 2 compliance optics or US-specific contracts (P4 if signal warrants).
- Multi-tier holding structure (e.g. PTE → BVI → JSC) — unnecessary at this scale.
- Cross-border employee transfers (the OpCo continues to employ the Vietnamese team; international hires are explicit FR-HR-INTL-001 in P4+).

# Dependencies

- FR-TEN-001 / FR-TEN-002 / FR-TEN-003.
- FR-INV-001 / FR-INV-003 (per-tenant invoicing routing).
- FR-DOC-001 / FR-DOC-002 (IP licence + tenant-novation envelopes).
- FR-AUTH-002 (audit chain).
- FR-OBS-001 / FR-OBS-002 / FR-OBS-003 (compliance evidence).
- Singapore legal counsel + Vietnamese legal counsel (transfer-pricing + corporate-actions).
- Singapore accountant for HoldCo books.
- Vietnamese accountant for OpCo continuity + transfer-pricing review.
- Department of Investment HCMC for the foreign-investment notification.
- ACRA (Singapore registrar) for HoldCo incorporation.
- Compliance: Singapore Companies Act + Income Tax Act; Vietnamese Foreign Investment Law + transfer-pricing regulations; cross-border tax treaty between Singapore + Vietnam (1994 + amendments).
- Locked decisions referenced: DEC-285 (Singapore HoldCo at P3 entry per PRD §2.5), DEC-286 (royalty + management fee transfer-pricing structure), DEC-287 (per-tenant-shard legal-entity routing), DEC-288 (Vietnamese tenants stay on JSC contracts to preserve VAT eligibility).

# AI Risk Assessment

Not applicable. `eu_ai_act_risk_class: not_ai`. Corporate-structure migration is deterministic legal + accounting actions.
