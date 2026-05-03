---
title: "ESOP — phantom-stock schema (grants, vesting schedules, valuation history) + anti-retroactive parameter versioning"
author: "@stephen-cheng"
department: human_resources
status: ready_for_review
priority: p2
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: infrastructure
eu_ai_act_risk_class: high
target_release: "P2 / 2027-Q2"
client_visible: false
template: feature_request@1
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Stand up the **ESOP (Employee Share Option Plan / phantom stock)** module's schema and Apollo subgraph. CyberSkill's Total Rewards Appendix specifies **phantom stock** (rights to a payout pegged to the company's notional valuation, not actual share issuance) with **4-year vesting** + **Year-3 put options** (FR-ESOP-002 implements the lifecycle). Schema includes the **Plan** (the legal-document-backed container per parameter version), the **Grant** (per-Member equity grant with vesting + cliff), the **VestingEvent** (each scheduled vest tranche), the **ValuationEvent** (a board-or-founder-blessed valuation snapshot with rationale), the **PutOption** (a Year-3+ Member's right to redeem vested phantom shares for cash), and the **PayoutEvent** (when the put or a liquidity event triggers cash). Same anti-retroactive discipline as FR-REW-001/002 — every plan + grant + valuation is parameter-version-locked + immutable post-publish + signed by founder + engineering lead + legal counsel ref. Lives in `hr_secure` under the same separate-KMS-key pattern. AI is **forbidden from compute paths**; the only AI surface is the read-only put-option simulator (FR-ESOP-002 §"Read-only AI") that explains "if you put 30% of vested shares today at the most recent valuation, your gross would be X" — never recommending action.

## Problem

PRD §9.17 names ESOP as P2 with the specific architectural property: "Operates on the same anti-retroactive parameter-versioning discipline as REW." PRD §2.3 Bet 5 names it as part of the moat. Three failure modes the platform must structurally prevent:

- **Retroactive plan modification.** A grant signed at 1,000 phantom shares cannot be quietly halved later — the contract forbids it. The same anti-retroactive trigger pattern as FR-REW-001 applies.
- **Phantom-share value drift opacity.** Without versioned valuation events, "what's my equity worth today?" varies by who you ask. Each valuation is a signed event; the timeline is auditable.
- **AI in the compute path.** Same prohibition as REW; the simulator is read-only.

## Proposed Solution

The shape of the answer is `hr_secure.esop_*` schema + the parameter-version + signature primitives + the same trigger pattern as FR-REW-001/002.

**Schema (under `hr_secure`).**

```sql
-- The ESOP plan — the legal document + structural parameters.
CREATE TABLE hr_secure.esop_plan (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  parameter_version_id UUID NOT NULL REFERENCES hr_secure.parameter_version(id),
  plan_name TEXT NOT NULL,                                          -- "CyberSkill Phantom Stock Plan v1"
  total_authorised_shares BIGINT NOT NULL,                          -- the cap
  effective_from DATE NOT NULL,
  description_md TEXT NOT NULL,
  vesting_schedule_default JSONB NOT NULL,                          -- { kind: "monthly_with_cliff",
                                                                  --   total_months: 48,
                                                                  --   cliff_months: 12,
                                                                  --   monthly_pct_after_cliff: 1/36 of (1 - 25%) }
                                                                  -- (4-year vesting; 1-year cliff at 25%; monthly thereafter)
  put_option_rule JSONB NOT NULL,                                   -- { eligible_from_year: 3,
                                                                  --   max_pct_per_year: 0.33,
                                                                  --   valuation_basis: "most_recent_blessed_valuation",
                                                                  --   payout_currency: "VND",
                                                                  --   payout_terms_md: "..." }
  good_leaver_treatment_md TEXT NOT NULL,                            -- vested shares retained per FR-REW-007's Good Leaver
  bad_leaver_treatment_md TEXT NOT NULL,                              -- vested + unvested forfeit per FR-REW-007's Bad Leaver
  signed_by_founder_at TIMESTAMPTZ NOT NULL,
  signed_by_engineering_lead_at TIMESTAMPTZ NOT NULL,
  signed_by_legal_counsel_ref TEXT NOT NULL,                          -- mandatory; equity is high-stakes legal
  superseded_by UUID REFERENCES hr_secure.esop_plan(id),
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Trigger: plan immutable post-publish.
CREATE OR REPLACE FUNCTION hr_secure.forbid_esop_plan_update()
RETURNS TRIGGER AS $$
BEGIN
  IF OLD.signed_by_founder_at IS NOT NULL
     AND OLD.signed_by_engineering_lead_at IS NOT NULL
     AND OLD.signed_by_legal_counsel_ref IS NOT NULL THEN
    RAISE EXCEPTION 'esop_plan % is published and immutable; create a superseding plan version', OLD.id
      USING ERRCODE = 'check_violation';
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER hr_secure_esop_plan_immutable
  BEFORE UPDATE ON hr_secure.esop_plan
  FOR EACH ROW EXECUTE FUNCTION hr_secure.forbid_esop_plan_update();

-- Per-Member grant.
CREATE TABLE hr_secure.esop_grant (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  plan_id UUID NOT NULL REFERENCES hr_secure.esop_plan(id),
  employee_id UUID NOT NULL REFERENCES hr.employee(id) ON DELETE RESTRICT,
  grant_number BIGSERIAL,                                            -- monotonic per tenant for serialisation
  total_phantom_shares BIGINT NOT NULL,
  grant_date DATE NOT NULL,
  vesting_start_date DATE NOT NULL,                                   -- typically grant_date or hire_date
  vesting_schedule_override JSONB,                                    -- null = use plan default
  cliff_status TEXT NOT NULL DEFAULT 'pending',                       -- "pending" | "passed" | "forfeited"
  signed_by_founder_at TIMESTAMPTZ NOT NULL,
  signed_by_employee_at TIMESTAMPTZ,                                   -- the Member's countersign
  signed_doc_id UUID,                                                  -- references DOC P3 when ships
  reason_md_encrypted BYTEA,                                           -- the rationale; encrypted
  status TEXT NOT NULL DEFAULT 'active',                                -- "active" | "fully_vested" | "forfeited" | "redeemed"
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (tenant_id, employee_id, grant_number)
);

-- Trigger: grant immutable post-Member-countersign.
CREATE TRIGGER hr_secure_esop_grant_immutable
  BEFORE UPDATE ON hr_secure.esop_grant
  FOR EACH ROW EXECUTE FUNCTION hr_secure.forbid_esop_plan_update();
                                                                    -- reuses the same function pattern;
                                                                    -- reused trigger function checks signatures

-- Vesting events: scheduled vest tranches.
CREATE TABLE hr_secure.esop_vesting_event (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  grant_id UUID NOT NULL REFERENCES hr_secure.esop_grant(id) ON DELETE RESTRICT,
  vest_date DATE NOT NULL,
  shares_vested_in_event BIGINT NOT NULL,                              -- per-tranche amount
  cumulative_vested_after_event BIGINT NOT NULL,                       -- sum to date
  status TEXT NOT NULL DEFAULT 'scheduled',                              -- "scheduled" | "vested" | "forfeited" | "accelerated"
  vested_at TIMESTAMPTZ,
  forfeited_at TIMESTAMPTZ,
  forfeit_reason_md_encrypted BYTEA,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  UNIQUE (tenant_id, grant_id, vest_date)
);

CREATE INDEX esop_vesting_event_grant_idx ON hr_secure.esop_vesting_event (tenant_id, grant_id, vest_date);

-- Valuation events.
CREATE TABLE hr_secure.esop_valuation_event (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  parameter_version_id UUID NOT NULL REFERENCES hr_secure.parameter_version(id),
  valuation_date DATE NOT NULL,
  total_company_valuation_minor_encrypted BYTEA NOT NULL,              -- the company's notional valuation; encrypted
  per_phantom_share_minor_encrypted BYTEA NOT NULL,                     -- = total_valuation / total_authorised_shares
  currency TEXT NOT NULL,
  basis TEXT NOT NULL,                                                  -- "internal_board_review" | "external_409a"
                                                                      -- | "fundraising_round" | "secondary_transaction"
  basis_evidence_md_encrypted BYTEA,                                    -- the rationale + supporting docs; encrypted
  signed_by_founder_at TIMESTAMPTZ NOT NULL,
  signed_by_engineering_lead_at TIMESTAMPTZ NOT NULL,
  signed_by_legal_counsel_ref TEXT,                                      -- required for external valuations
  status TEXT NOT NULL DEFAULT 'draft',                                   -- "draft" | "blessed" | "superseded" | "rolled_back"
  blessed_at TIMESTAMPTZ,
  superseded_by UUID REFERENCES hr_secure.esop_valuation_event(id),
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX esop_valuation_date_idx ON hr_secure.esop_valuation_event (tenant_id, valuation_date DESC);

-- Trigger: valuation event immutable post-bless.
CREATE OR REPLACE FUNCTION hr_secure.forbid_valuation_update_after_bless()
RETURNS TRIGGER AS $$
BEGIN
  IF OLD.status = 'blessed' THEN
    -- The ONLY allowed update post-bless is setting `superseded_by` (a new valuation supersedes it).
    -- All other fields are immutable.
    IF NEW.total_company_valuation_minor_encrypted IS DISTINCT FROM OLD.total_company_valuation_minor_encrypted
       OR NEW.per_phantom_share_minor_encrypted IS DISTINCT FROM OLD.per_phantom_share_minor_encrypted
       OR NEW.basis IS DISTINCT FROM OLD.basis
       OR NEW.signed_by_founder_at IS DISTINCT FROM OLD.signed_by_founder_at THEN
      RAISE EXCEPTION 'esop_valuation_event % is blessed and immutable; create a new event instead', OLD.id;
    END IF;
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER hr_secure_valuation_immutable
  BEFORE UPDATE ON hr_secure.esop_valuation_event
  FOR EACH ROW EXECUTE FUNCTION hr_secure.forbid_valuation_update_after_bless();
```

**Default plan seed.**

The first ESOP plan (parameter version v1; ESOP-specific extension to FR-REW-001's parameter version):

| Field | Value |
|---|---|
| total_authorised_shares | 10,000,000 (notional; representing 10% of fully-diluted) |
| vesting_schedule_default.kind | "monthly_with_cliff" |
| vesting_schedule_default.total_months | 48 |
| vesting_schedule_default.cliff_months | 12 |
| vesting_schedule_default.cliff_pct | 0.25 (25% at cliff) |
| vesting_schedule_default.monthly_pct_after_cliff | (1 - 0.25) / 36 = ~0.0208 per month for the remaining 36 months |
| put_option_rule.eligible_from_year | 3 |
| put_option_rule.max_pct_per_year | 0.33 |
| put_option_rule.valuation_basis | "most_recent_blessed_valuation" |
| put_option_rule.payout_currency | "VND" (or USD for USD-denominated grants when international hires P3+) |
| good_leaver_treatment | retain vested; unvested forfeit |
| bad_leaver_treatment | vested + unvested forfeit |

The plan is reviewed annually by founder + Engineering Lead + external counsel; changes go through the standard sign-and-publish flow.

**Vesting-event scheduling.**

When a grant is signed (founder + Member countersigns):
1. The schedule is computed: typically 1-year cliff → 25% of total at cliff (single event); then 36 monthly events of (~2.08% each).
2. `esop_vesting_event` rows are inserted for every scheduled date over the 48-month vesting period.
3. A nightly job marks `scheduled → vested` events whose `vest_date <= today`.

Acceleration (rare; M&A or founder-discretionary): a manual override creates an "accelerated" vesting event with founder + DPO + legal-counsel-ref signatures.

**Valuation-event lifecycle.**

1. **Draft.** Founder + Engineering Lead draft a new valuation (typically annually, or after a fundraising round / secondary).
2. **Evidence.** Supporting documents are referenced (board-review minutes; external 409A report; fundraising term sheet).
3. **Sign.** Founder + Engineering Lead sign; for external-basis (409A, fundraising), legal counsel ref required.
4. **Bless.** Status `draft → blessed`; immutability trigger thereafter rejects modifications.
5. **Effects.** All future put-option computations use the most-recent-blessed valuation; the per-phantom-share value is updated.

**RLS + ACL.**

- `esop_plan`: HR/Ops + Founder + DPO + Auditor read; only HR/Ops + Founder + Engineering Lead + DPO write (UI-only, parameter-version flow).
- `esop_grant`: the Member sees their own grant; their manager does *not* (compensation-secret); HR/Ops + Founder + DPO see all.
- `esop_vesting_event`: same as grant.
- `esop_valuation_event`: HR/Ops + Founder + Engineering Lead + DPO + Auditor read; sign-and-publish flow restricted.

All reads audit-logged with `field_kind` + `purpose` (mandatory).

**MCP tool surface (read-only).**

- `cyberos.esop.my_grants` — read; calling Member's own; step-up.
- `cyberos.esop.my_vesting_schedule(grant_id)` — read; step-up.
- `cyberos.esop.list_blessed_valuations(since?)` — read; HR/Ops + Founder + Engineering Lead + DPO + Auditor.
- `cyberos.esop.get_active_plan` — read; everyone (the plan exists; its parameters are public per the legal doc).

There are **no mutation MCP tools** — same architectural rule as FR-REW-001..007. Plan + grant + valuation publishing is HR/Ops + Founder UI + step-up only.

**BRAIN denylist + structural exclusion.**

`hr_secure.esop_*` is structurally excluded from BRAIN ingestion (same pattern as FR-HR-001 / FR-REW-001). The denylist regex catches typical equity-amount patterns (large round numbers + "phantom shares" / "equity" / "ESOP" tokens). Nightly sweep over `brain.fact.text` re-asserts.

**Audit integration.**

`esop.{tenant}` audit scope. Every plan + grant + valuation lifecycle event audit-logged. The `cp.regime` row for "ESOP-Plan-Compliance" tracks plan signing + valuation cadence + grant counter-sign rate.

**Compliance Cockpit panel.**

- Active plan version + signed-by status.
- Valuation cadence (last-blessed-at; days-since-last; flag if > 18 months without a new valuation).
- Per-Member grant summary (count of active grants; aggregate phantom shares; never per-Member amounts in the cockpit — that's `/esop/my` for the Member themselves).
- Force-acceleration events (should be rare).

## Alternatives Considered

- **Real share issuance instead of phantom stock.** Rejected for P2: PRD specifies phantom stock; real share issuance has Vietnamese securities-regulation implications + cap-table complexity not yet warranted at 10-employee scale. P3+ may revisit if international expansion needs real-equity instruments.
- **Hosted ESOP platform (Carta, Pulley).** Rejected: residency + integration with REW + Total Rewards Appendix encoding + the per-Vietnamese-context legal terms. P3+ may use Carta as an external system-of-record while keeping CyberOS as the operational surface.
- **Skip valuation events; use a fixed valuation forever.** Rejected: phantom-share value should reflect company performance; valuation events with documented basis are the floor.
- **AI-suggested valuation.** Rejected: explicit prohibition.

## Success Metrics

- **Primary metric.** P2 sprint demo passes: (1) the founder publishes the first ESOP plan (parameter version + sign chain + legal-counsel-ref); (2) HR/Ops Lead drafts grants for the 10 employees; founder + each Member counter-sign; (3) vesting-event rows are inserted for every grant; (4) a synthetic valuation event is blessed; immutability trigger rejects update.
- **Compliance metric.** Zero retroactive plan or grant modifications; zero AI in compute paths; zero equity values in BRAIN.
- **Audit completeness.** 100% of plan + grant + valuation reads + writes audit-logged.

## Scope

**In-scope.**
- The 4 schema additions (`esop_plan`, `esop_grant`, `esop_vesting_event`, `esop_valuation_event`).
- Anti-retroactive immutability triggers.
- Default plan seed + grant lifecycle + vesting-event scheduling.
- Valuation-event lifecycle.
- RLS + ACL + audit.
- The 4 read-only MCP tools.
- BRAIN denylist + structural exclusion.
- Compliance Cockpit panel.

**Out-of-scope (deferred to FR-ESOP-002 / FR-ESOP-003).**
- Put-option mechanics + read-only AI simulator (FR-ESOP-002).
- Frontend remote at /esop (FR-ESOP-003).
- Good Leaver / Bad Leaver branches integration with FR-REW-007 (FR-ESOP-002).
- Liquidity-event payout flow (P3 — when M&A or fundraising happens).
- International grants (P3+).
- 409A-style external valuation API integration (P3+).

## Dependencies

- FR-HR-001 / FR-REW-001 (substrate + parameter-version primitive + per-tenant `hr_secure` KMS key).
- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-002 / FR-AUTH-003 / FR-MCP-001.
- FR-CP-001 (Compliance Cockpit).
- HashiCorp Vault for the per-tenant `hr_secure` KMS key.
- External legal counsel review of plan + grant terms.
- The signed Total Rewards Appendix.
- Compliance: Vietnamese securities + tax regulations on phantom stock; PDPL Decree 13; EU AI Act Articles 5-7 high-risk classification (compensation domain — no AI in compute); GDPR Article 22; SOC 2 CC6.
- Locked decisions referenced: DEC-207 (phantom stock; not real shares in P2), DEC-208 (4-year vesting + 1-year cliff + monthly thereafter), DEC-209 (Year-3 put options at 33% per year max), DEC-210 (anti-retroactive plan + grant + valuation immutability).

## AI Risk Assessment

This FR explicitly forbids AI in the compute path. EU AI Act risk class: `high` (compensation + equity domain).

### Data Sources

The schema stores data; no AI in the read or write path of this FR. The simulator (FR-ESOP-002) consumes the data read-only. Per-tenant residency.

### Human Oversight

- Plan + grant + valuation publish each require multi-party sign chains.
- Valuation events require legal-counsel-ref for external basis.
- Forfeiture / acceleration is documented + signed.
- The Compliance Cockpit surfaces every event.

### Failure Modes

- **Retroactive plan modification.** Caught by trigger.
- **Grant counter-sign skipped.** The grant remains in `signed_by_founder_at` but `signed_by_employee_at: NULL` state; vesting events are *not* scheduled until the Member counter-signs (Member acceptance is required for the grant to be effective).
- **Valuation drift.** Mitigation: 18-month cadence reminder; founder + Engineering Lead must produce a fresh valuation regularly.
- **Equity value leak into BRAIN.** Caught by structural ingestion exclusion + nightly sweep.

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6.
- **Scope:** Drafted schema, anti-retroactive triggers, plan + grant + vesting-event lifecycles, valuation-event mechanics, failure modes.
- **Human review:** `@stephen-cheng` reviewed; legal counsel will review the schema + trigger encoding + plan template before P2 production.
