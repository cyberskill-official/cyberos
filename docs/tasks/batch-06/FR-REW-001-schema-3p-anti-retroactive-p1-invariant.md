---
title: "REW — schema + 3P income model (P1/P2/P3) + anti-retroactive parameter versioning + P1-protection invariant at DB-policy level"
author: "@stephen-cheng"
department: human_resources
status: ready_for_review
priority: p2
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: infrastructure
eu_ai_act_risk_class: high
target_release: "P2 / 2027-Q1"
client_visible: false
template: feature_request@1
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Stand up the REW (Total Rewards) module — the legal heart of CyberSkill's social contract per PRD §2.3 Bet 5 and §9.14. Encodes the **3P income model**: **P1 Base** (the contractual floor; cash-paid monthly), **P2 Allowance** (cash-paid monthly; tied to role + seniority + company performance), **P3 Performance** (variable; quarterly; cash-collected from the Bonus Points fund modelled in FR-REW-002). Implements **anti-retroactive parameter versioning** — every salary parameter (rate, multiplier, threshold) has a version with `effective_from` + `published_at` + `signed_by_*` fields; once published, a parameter version is immutable; modifications create new versions superseding the old. Implements the **P1-protection invariant** ("evaluation never reduces base salary in cash") at the **database-policy level** — a `BEFORE UPDATE` trigger rejects any mutation that decreases `salary.p1_base_monthly_minor` for an active employee unless an explicit `force_reduction_with_legal_review_id` is set (signed off by founder + DPO + legal counsel reference). Lives in the `hr_secure` schema (FR-HR-001 §"Comp-secure schema") under the per-tenant `hr_secure` KMS key. Subsequent batch-06 FRs add Bonus Points (FR-REW-002), payroll close (FR-REW-003), Vietnamese SI/PIT (FR-REW-004), frontend + payslip narrator (FR-REW-005), migration (FR-REW-006), and Good/Bad Leaver (FR-REW-007).

## Problem

PRD §2.3 Bet 5 names this directly: "Most platforms cannot model 3P income with a cash-collected pool, BP overflow, anti-inflation interest, four-year phantom-stock vesting with put options from Year 3, sabbaticals, anti-retroactive parameter versioning, and Good/Bad Leaver branches. CyberOS does — because it is required for the founder's own company to function."

The Total Rewards Appendix is the founder's signed contract with every Member. Three failure modes the platform must structurally prevent:

- **Retroactive parameter change.** A bonus multiplier set at 1.2× in Q2 cannot be unilaterally changed to 1.0× in Q3 retroactively — that breaks the contract. The PRD §4.3 anti-metric: "Parameter version retroactively modified after publish = 0 — immutable by construction at the database-policy level."
- **P1 base salary reduction by AI / process error.** PRD §4.3: "P1 base salary reduced as penalty by the system = 0 — legal commitment from Total Rewards Appendix Article 2a; sev-0 if violated." This must be enforced at the database, not at the application.
- **Compensation values leaking into BRAIN.** PRD §4.3: "Compensation / equity values appearing in BRAIN vector or filesystem layer = 0". The `hr_secure` schema with separate KMS key + ingestion-side denylist (FR-HR-001 + FR-BRAIN-002) is the architectural floor.

## Proposed Solution

The shape of the answer is a `hr_secure.salary_*` schema with envelope-encrypted columns, a parameter-version log with append-only semantics + immutability triggers, a P1-protection trigger, and the GraphQL contract.

**Schema (under `hr_secure`).**

```sql
-- Per-Member current salary state (3P income model).
CREATE TABLE hr_secure.salary (
  tenant_id UUID NOT NULL,
  employee_id UUID NOT NULL UNIQUE REFERENCES hr.employee(id) ON DELETE RESTRICT,
                                                                 -- RESTRICT, never CASCADE — cannot delete employee with salary record
  -- All amount fields are envelope-encrypted bytea via the hr_secure KMS key.
  -- For documentation, conceptually they hold integer minor-currency values.
  p1_base_monthly_minor_encrypted BYTEA NOT NULL,                  -- Article 2a base; cash-paid; the protected floor
  p2_allowance_monthly_minor_encrypted BYTEA NOT NULL,             -- Article 2b allowance; cash-paid
  -- p3 is computed quarterly from BP fund; not a stored monthly value
  currency TEXT NOT NULL,                                          -- ISO 4217; tenant primary
  effective_from DATE NOT NULL,
  effective_to DATE,                                                -- null for current; populated when superseded
  parameter_version_id UUID NOT NULL REFERENCES hr_secure.parameter_version(id),
                                                                   -- the version of REW parameters in force
  superseded_by UUID REFERENCES hr_secure.salary,                   -- the next salary record (audit chain)
  signed_by_founder_at TIMESTAMPTZ NOT NULL,
  signed_by_employee_at TIMESTAMPTZ NOT NULL,                       -- the Member's countersign (acceptance)
  signed_doc_id UUID,                                                -- DOC module's signed-PDF when DOC ships P3
  reason_md_encrypted BYTEA,                                         -- human-language rationale; encrypted
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (tenant_id, employee_id, effective_from)
);

-- Salary history is the chain of all `hr_secure.salary` rows over time per employee — by virtue of the composite PK.
-- The `superseded_by` link makes the audit chain explicit; old rows are preserved (never deleted).

-- Parameter version log (anti-retroactive; immutable post-publish).
CREATE TABLE hr_secure.parameter_version (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  version_number INT NOT NULL,                                     -- monotonic per tenant
  description_md TEXT NOT NULL,                                    -- the changelog entry
  parameters JSONB NOT NULL,                                        -- the full parameter snapshot:
                                                                   -- {
                                                                   --   p1_base_floors_per_level: { ... },
                                                                   --   p2_allowance_per_role: { ... },
                                                                   --   p3_target_quarterly: { ... },
                                                                   --   bp_fund_rules: { ... },
                                                                   --   bp_anti_inflation_rate_source: "ACB-savings-rate",
                                                                   --   acb_rate_snapshot_pct: 5.4,
                                                                   --   acb_rate_snapshot_taken_at: "2026-Q2",
                                                                   --   sabbatical_rules: { ... },
                                                                   --   evaluation_rules: { ... }
                                                                   -- }
  effective_from DATE NOT NULL,
  published_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  signed_by_founder_at TIMESTAMPTZ NOT NULL,
  signed_by_engineering_lead_at TIMESTAMPTZ,                        -- secondary signature for engineering-grade decisions
  signed_by_legal_counsel_ref TEXT,                                  -- external legal reference (reviewed at adoption)
  superseded_by UUID REFERENCES hr_secure.parameter_version(id),
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  UNIQUE (tenant_id, version_number)
);

-- Forbid any UPDATE to a published parameter_version (the version is immutable).
CREATE OR REPLACE FUNCTION hr_secure.forbid_param_version_update()
RETURNS TRIGGER AS $$
BEGIN
  IF OLD.published_at IS NOT NULL THEN
    RAISE EXCEPTION 'parameter_version % is published and immutable; create a new version', OLD.id
      USING ERRCODE = 'check_violation';
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER hr_secure_param_version_immutable
  BEFORE UPDATE ON hr_secure.parameter_version
  FOR EACH ROW
  EXECUTE FUNCTION hr_secure.forbid_param_version_update();

-- Forbid any DELETE on parameter_version (audit-grade preservation).
CREATE TRIGGER hr_secure_param_version_no_delete
  BEFORE DELETE ON hr_secure.parameter_version
  FOR EACH ROW
  EXECUTE FUNCTION cyberos_audit_owner.raise_no_delete();
```

**The P1-protection invariant — at trigger level.**

```sql
CREATE OR REPLACE FUNCTION hr_secure.protect_p1_base()
RETURNS TRIGGER AS $$
DECLARE
  prev_p1_value BIGINT;
  new_p1_value BIGINT;
  is_force_reduction BOOLEAN;
BEGIN
  -- Only checks when superseding an existing salary row for an active employee.
  IF NEW.effective_from <= CURRENT_DATE THEN
    SELECT (pgp_sym_decrypt(prev_row.p1_base_monthly_minor_encrypted, kms_key()))::bigint
      INTO prev_p1_value
      FROM hr_secure.salary prev_row
     WHERE prev_row.tenant_id = NEW.tenant_id
       AND prev_row.employee_id = NEW.employee_id
       AND prev_row.effective_to IS NULL
       AND prev_row.effective_from < NEW.effective_from
     ORDER BY prev_row.effective_from DESC
     LIMIT 1;

    new_p1_value := (pgp_sym_decrypt(NEW.p1_base_monthly_minor_encrypted, kms_key()))::bigint;
    is_force_reduction := COALESCE((NEW.metadata->>'force_reduction_with_legal_review_id') IS NOT NULL, false);

    IF prev_p1_value IS NOT NULL
       AND new_p1_value < prev_p1_value
       AND NOT is_force_reduction THEN
      RAISE EXCEPTION 'P1-protection invariant violated: cannot reduce P1 base from % to % without force_reduction_with_legal_review_id', prev_p1_value, new_p1_value
        USING ERRCODE = 'check_violation';
    END IF;
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER hr_secure_p1_protection
  BEFORE INSERT OR UPDATE ON hr_secure.salary
  FOR EACH ROW
  EXECUTE FUNCTION hr_secure.protect_p1_base();
```

The trigger is the architectural floor — the application cannot reduce P1 by mistake, by AI, or by an admin's oversight. The `force_reduction_with_legal_review_id` escape hatch is the only path; it requires (per application policy + audit) the founder's signature, the DPO's signature, and an external legal-counsel reference. Every force-reduction writes a high-prominence audit row visible in the Compliance Cockpit.

**3P income model encoded.**

- **P1 Base** (Article 2a of the Total Rewards Appendix): the contractual floor; cash-paid monthly. Stored in `hr_secure.salary.p1_base_monthly_minor_encrypted`. Subject to the protection invariant.
- **P2 Allowance** (Article 2b): role + seniority + company-performance-tied; cash-paid monthly. Stored in `hr_secure.salary.p2_allowance_monthly_minor_encrypted`. *Not* subject to P1-protection (allowance can move with company performance per the contract).
- **P3 Performance** (Article 2c): variable; quarterly; cash-collected from the Bonus Points fund (FR-REW-002). Computed at quarter-close as a function of (the Member's points × the BP fund's per-point value at close). *Never* stored as a static monthly value — always derived from the BP fund.

**Parameter publishing flow.**

1. Founder + Engineering Lead author a new parameter version: edit a draft in `hr_secure.parameter_version` with `published_at IS NULL`.
2. Legal counsel reviews; HR/Ops Lead reviews.
3. Founder signs (`signed_by_founder_at`); Engineering Lead signs (`signed_by_engineering_lead_at`); the legal counsel reference is recorded.
4. Founder publishes — the row's `published_at` is set; the immutability trigger thereafter rejects updates.
5. The new version is auto-consumed by ongoing payroll cycles whose `effective_from` falls inside the new version's window.

**Salary publishing flow.**

A change to a Member's salary (raise, allowance bump, role-change-driven reset):

1. HR/Ops Lead drafts a new `hr_secure.salary` row with `effective_from` in the future.
2. The current salary row remains active; the new row is `effective_from`-future.
3. Founder signs (`signed_by_founder_at`); the Member countersigns (`signed_by_employee_at`).
4. On `effective_from` arrival, the old row's `effective_to` is set to `effective_from - 1` and `superseded_by` is the new row.
5. The P1-protection trigger fires on the INSERT/UPDATE; rejects on a P1 reduction without legal escape hatch.

A reduction for cause (rare; e.g. role-downgrade with employee consent) requires the legal-review escape hatch, populated in `metadata.force_reduction_with_legal_review_id`. Audit-row + DPO + Founder sign-off + an explicit consent record from the Member are all required policy-side; the trigger only enforces the data-integrity floor.

**RLS + access controls.**

- Every `hr_secure.salary` row: tenant RLS + a strict ACL — only HR/Ops Lead, Founder, and DPO can SELECT (Auditor cannot; the Auditor's read is via aggregate roll-ups in payroll period reports, not per-row).
- The Member can read their *own* current salary record via a special `cyberos.rew.my_salary` MCP tool (read-only; with step-up auth required).
- All reads write audit rows with `field_kind: salary.<sub_field>`, `purpose: <text>` (mandatory).

**GraphQL surface.**

```graphql
type Query {
  rewMyCurrentSalary: RewSalary  # requires step-up; returns the calling Member's own current salary
  rewSalary(employeeId: ID!, asOfDate: Date): RewSalary  # HR/Ops + Founder + DPO; step-up required
  rewSalaryHistory(employeeId: ID!): [RewSalary!]!       # same access requirements
  rewParameterVersions(activeOnly: Boolean = false): [RewParameterVersion!]!
  rewActiveParameterVersion(asOfDate: Date): RewParameterVersion
}

type Mutation {
  # All mutations require step-up auth + dual-sign at policy level (the trigger enforces immutability).
  rewDraftParameterVersion(input: RewParameterVersionInput!): RewParameterVersion!
  rewPublishParameterVersion(id: ID!, founderSignature: String!, engineeringLeadSignature: String!,
                             legalCounselRef: String!): RewParameterVersion!
  rewDraftSalary(input: RewSalaryInput!): RewSalary!
  rewPublishSalary(id: ID!, founderSignatureToken: String!, employeeSignatureToken: String!): RewSalary!
  rewForceP1Reduction(employeeId: ID!, newP1Minor: BigInt!, legalReviewRef: String!,
                      founderSig: String!, dpoSig: String!, consentRecordRef: String!): RewSalary!
  # Note: there are no `update` or `delete` mutations on parameter_version after publish. Drafting + publishing is the only path.
}
```

Every secure-tier mutation carries an `@stepUp` directive (FR-AUTH-003).

**MCP tool surface.**

- `cyberos.rew.my_salary` — read; the calling Member's own salary; step-up required at the gateway.
- `cyberos.rew.list_parameter_versions` — read; HR/Ops + Founder + DPO; informational.
- `cyberos.rew.get_parameter_version(version_number)` — read; same audience.

There are **no mutation MCP tools** for REW. Compensation paths are non-MCP per PRD §2.5 anti-positioning + FR-MCP-001 §"Tool annotations enforced at the proxy" §"Irreversible tools are not registered". Salary drafts and publishes happen exclusively through the HR/Ops Lead frontend with explicit step-up + dual-sign. CUO can *narrate* a payslip (FR-REW-005) but never compute or write.

**BRAIN denylist + ingestion exclusion.**

- The `hr_secure.salary` table is structurally excluded from BRAIN ingestion (FR-BRAIN-002 source-listener allowlist does not include `hr_secure.*`).
- The denylist regex in FR-AI-001's redaction layer matches typical compensation-amount patterns (e.g. `\b(?:\d{1,3}[,.]?){2,}\s*(?:VND|đ|USD|\$)\b`) so that *if* a stray comp value appears in any other surface (CHAT, EMAIL, KB), it never reaches an external provider.
- Nightly sweep over `brain.fact.text` re-asserts (FR-BRAIN-002 §"Compensation / equity values..."). A single match is sev-0.

**Audit integration.**

Every read of `hr_secure.salary` writes an audit row in scope `rew.{tenant}` with `field_kind`, `purpose`, `actor_subject`. Every mutation writes audit rows + chain into the canonical Merkle audit (FR-AUTH-002). The `parameter_version` lifecycle is the most-audited path in the platform — every draft, every signature, every publish is captured.

**Compliance Cockpit.**

A dedicated panel in the cockpit (FR-CP-001) shows:
- Current parameter version + when last published.
- Per-Member salary "as-of-now-vs-prior" deltas (heat map).
- P1-protection trigger violations attempted (should always be 0).
- BRAIN comp-leakage sweep results (should always be 0).
- Force-reduction events (should be exceedingly rare; any occurrence triggers alarm).

## Alternatives Considered

- **Application-level enforcement of P1-protection.** Rejected: the trigger is the database-level floor; application bugs cannot bypass.
- **Skip parameter versioning; mutate parameter values in place.** Rejected (also PRD §4.3 anti-metric): retroactive parameter change is the explicit anti-pattern.
- **Single salary table with mutable rows.** Rejected: history must be queryable + immutable for audit + dispute resolution.
- **Allow MCP write tools for salary with step-up.** Rejected: PRD §2.5 anti-positioning + the architectural rule. The HR/Ops Lead UI is the only write surface.
- **Use a third-party payroll provider (Gusto, ADP, Justworks, Payroll-VN).** Rejected: residency + the Total Rewards Appendix's specific structure (3P + BP + anti-retroactive + P1-protection) cannot be encoded in any hosted provider we evaluated. The platform is the system of record; a downstream provider may consume the platform's output for actual bank disbursement.

## Success Metrics

- **Primary metric.** P2 sprint demo passes: (1) the founder authors + signs + publishes parameter version v1; immutability trigger rejects update; (2) HR/Ops Lead drafts + signs a Member salary record; the Member countersigns; the row publishes correctly; (3) a synthetic P1 reduction without `force_reduction_with_legal_review_id` is rejected by the trigger; (4) BRAIN nightly sweep reports zero comp-leakage.
- **Compliance metric.** Zero parameter-version-retroactive-modification events for the lifetime of the platform (sev-0 if observed). Zero unauthorised P1 reductions. Zero comp values in BRAIN.
- **Audit completeness.** 100% of `hr_secure.salary` reads + writes audit-logged with `purpose` non-empty.

## Scope

**In-scope.**
- The `hr_secure.salary` and `hr_secure.parameter_version` tables.
- Anti-retroactive immutability trigger.
- P1-protection trigger.
- Parameter publishing flow with dual-sign.
- Salary publishing flow with founder + Member dual-sign.
- Force-reduction escape hatch (audit-heavy).
- Apollo Federation v2 read + draft + publish mutations with `@stepUp` directives.
- 3 read-only MCP tools (no mutation MCP for compensation).
- BRAIN ingestion structural exclusion + nightly denylist sweep.
- Compliance Cockpit panel.
- Audit integration in scope `rew.{tenant}`.
- Seed migration: parameter version v1 with the current Total Rewards Appendix values; salary records for the 10 employees.

**Out-of-scope (deferred).**
- Bonus Points fund mechanics (FR-REW-002).
- Payroll cycle close + payslip generation (FR-REW-003).
- Vietnamese SI/PIT calculations (FR-REW-004).
- Frontend remote + payslip narrator (FR-REW-005).
- Migration from the existing Excel payroll (FR-REW-006).
- Good Leaver / Bad Leaver branches (FR-REW-007).
- Phantom Stock (P3 — separate FR cluster in batch-07).

## Dependencies

- FR-HR-001 (the `hr_secure` schema substrate; per-tenant separate KMS key).
- FR-INFRA-001 (Postgres + pgcrypto).
- FR-AUTH-001 / FR-AUTH-002 / FR-AUTH-003 (identity, audit, step-up).
- FR-CP-001 / FR-CP-002 (Compliance Cockpit panel; per-tenant KMS key).
- FR-BRAIN-002 (ingestion-side denylist + nightly sweep).
- HashiCorp Vault for the per-tenant `hr_secure` KMS key.
- A signed copy of the current Total Rewards Appendix (legal source-of-truth).
- Compliance: Vietnamese Labour Code Articles on wage protection (Article 95+); PDPL Decree 13 (compensation is sensitive personal data); EU AI Act Articles 6-7 (high-risk classification for AI used in compensation decisions; this FR explicitly forbids AI in compensation compute paths); GDPR Article 22 (no fully automated decisions on individuals); SOC 2 CC6 + CC8.
- Locked decisions referenced: DEC-165 (P1-protection at trigger level), DEC-166 (parameter-version immutability at trigger level), DEC-167 (no MCP write tools for REW), DEC-168 (force-reduction requires founder + DPO + legal counsel ref).

## AI Risk Assessment

This FR explicitly forbids AI in compensation compute paths but operates under the EU AI Act high-risk classification because the data underpins later AI-aware features (the read-only payslip narrator in FR-REW-005). EU AI Act risk class: `high`.

### Data Sources

The schema stores data; no AI is in the read or write path of this FR. The eventual narrator (FR-REW-005) consumes the data read-only and runs through the AI Gateway with persona-stamping. No third-party data; no cross-tenant data; the per-tenant `hr_secure` KMS key prevents inter-tenant data exposure.

### Human Oversight

- Every parameter-version publish requires founder + engineering-lead dual-sign + legal counsel reference.
- Every salary publish requires founder + Member dual-sign.
- Force-reduction requires founder + DPO + legal counsel + Member consent.
- No AI surface in the compute path; AI cannot reduce, increase, or modify a compensation value.
- The HR/Ops Lead frontend is the only mutation surface; MCP cannot mutate.
- The kill switch from FR-GENIE-002 is irrelevant here — REW does not have AI; the explicit prohibition is the architectural floor.

### Failure Modes

- **P1-protection trigger bypassed by direct DB access.** Mitigation: the application Postgres role does not have `BYPASSRLS`; the `cyberos_app` role grant on `hr_secure.salary` does not include `TRIGGER` privileges; the only way to bypass is `cyberos_audit_owner` access (sealed credential), which itself is logged.
- **Parameter version edited post-publish.** Caught by the immutability trigger.
- **Comp value leak into BRAIN.** Caught by the structural ingestion exclusion + nightly sweep + denylist redaction; sev-0.
- **KMS key compromise.** Mitigation: per-tenant separate `hr_secure` KMS key; rotation procedure documented; key-access audit-logged; quarterly key-rotation drill.
- **Force-reduction abuse.** Mitigation: requires three signatures; logged with high prominence; founder + DPO + Member counter-sign; the audit row is reviewed by external counsel quarterly.

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6.
- **Scope:** Drafted schema, anti-retroactive trigger, P1-protection trigger, parameter + salary publishing flows, failure modes.
- **Human review:** `@stephen-cheng` reviewed; legal counsel will review the schema + triggers + Total Rewards Appendix encoding before P2 production deployment; SRS DEC-165..168 to be finalised.
