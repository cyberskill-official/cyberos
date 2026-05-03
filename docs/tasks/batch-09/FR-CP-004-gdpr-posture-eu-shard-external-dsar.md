---
title: "CP — GDPR posture for eu-shard, EU AI Act Article-by-Article evidence map, external-subject DSAR portal, DPA template library"
author: "@stephen-cheng"
department: operations
status: ready_for_review
priority: p3
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: internal_tooling
eu_ai_act_risk_class: not_ai
target_release: "P3 / 2027-Q4"
client_visible: false
template: feature_request@1
---

# Summary

Activate **GDPR posture** on the eu-shard (FR-TEN-001) so EU-resident tenants can sign + operate without legal exposure. Deliverables: **per-eu-shard DPIA library extension** to GDPR Article 35 standard (the FR-CP-003 PDPL DPIAs are extended with GDPR-specific lawful-basis assessments + transfer impact assessments + DPO contact information per Article 13/14); **EU AI Act Article-by-Article evidence map** covering Articles 5 (prohibited practices — none in scope), 6-7 (high-risk classification — REW/ESOP/LEARN evidence), 9-15 (high-risk system requirements — risk management + data governance + technical documentation + record-keeping + human oversight + accuracy + robustness + cybersecurity), 50 (transparency obligations — already enforced through persona-version stamping + disclosure chips); the **external-subject DSAR portal** at `https://privacy.cyberos.world/dsar/{tenant-slug}` finally graduating from FR-CP-003's Member-self-only stub; **DPA template library** for tenant-to-customer DPAs (Vietnamese tenants serving EU customers + EU tenants serving anywhere); **transfer impact assessments** (TIAs) per Schrems II for any cross-border data flows; the **right-to-be-forgotten** path through FR-TEN-002's deletion lifecycle with GDPR Article 17 statutory-floor handling. The posture is the structural answer to "can we sell to EU-headquartered customers?".

# Problem

PRD §14.4.1 P3 scope: "GDPR posture turned on for eu-shard; DSAR workflow surfaced via CP module." Three failure modes the platform must structurally avoid:

- **EU customer procurement blocked.** Without GDPR posture (DPA + DPIA + DSAR external + transfer-impact), any EU customer's procurement-due-diligence rejects the platform.
- **Schrems II violation.** Cross-border transfers from EU to non-EU countries require Standard Contractual Clauses + Transfer Impact Assessment per the CJEU's Schrems II ruling. Without TIA evidence, the EU customer's data flowing to AWS Singapore (vn-shard) or AWS Ohio (us-shard) is illegal.
- **EU AI Act high-risk uncertainty.** The REW + ESOP + LEARN modules are high-risk under EU AI Act Articles 6-7; without an explicit evidence map demonstrating compliance with Articles 9-15, the platform's market access in the EU collapses on enforcement (effective August 2026).

# Proposed Solution

**Per-eu-shard DPIA library extension.**

The FR-CP-003 PDPL-focused DPIA library is extended with GDPR-specific fields per Article 35:

- **Article 35(1)(a):** systematic description of envisaged processing operations + purposes.
- **Article 35(1)(b):** assessment of necessity + proportionality.
- **Article 35(1)(c):** assessment of risks to data subject rights + freedoms.
- **Article 35(1)(d):** measures to address the risks (technical + organisational).
- **Article 13/14:** the privacy notice rendered to data subjects (per language).

Each module's DPIA gets an EU-shard-specific addendum filed at the eu-shard's per-tenant `cp.dpia` row.

**EU AI Act Article-by-Article evidence map.**

A new `cp.eu_ai_act_evidence_map` table (extending FR-OBS-003's evidence-map pattern):

```sql
CREATE TABLE cp.eu_ai_act_evidence_map (
  tenant_id UUID NOT NULL,
  article_id TEXT NOT NULL,                                            -- "Article-9" | "Article-10" | "Article-13" | "Article-14"
                                                                       -- | "Article-15" | "Article-50" etc.
  applies_to_module TEXT NOT NULL,                                     -- "REW" | "ESOP" | "LEARN" | "any"
  status TEXT NOT NULL,                                                -- "implemented" | "in_progress" | "not_applicable"
  evidence_blob_ids UUID[],
  cross_reference_fr_ids TEXT[],                                        -- "FR-REW-001" | "FR-LEARN-002" etc.
  notes_md TEXT,
  signed_off_by_dpo_at TIMESTAMPTZ,
  signed_off_by_legal_counsel_ref TEXT,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  PRIMARY KEY (tenant_id, article_id, applies_to_module)
);
```

Sample mapping (REW high-risk module):

| Article | REW Implementation Evidence |
|---|---|
| 9 (Risk management) | FR-REW-001 P1-protection + anti-retroactive triggers + FR-REW-003 anomaly detection + audit-row completeness |
| 10 (Data governance) | FR-HR-001 separate hr_secure schema + structural BRAIN exclusion + denylist sweep |
| 11 (Technical documentation) | FR-CP-003 DPIA + parameter-version log + computation-trace per payslip |
| 12 (Record-keeping) | FR-AUTH-002 Merkle audit chain + FR-REW-001 sign chain |
| 13 (Transparency) | FR-REW-005 read-only narrator + EU AI Act disclosure chips |
| 14 (Human oversight) | FR-REW-001 P1-protection trigger + FR-REW-003 dual-sign + FR-LEARN-002 Council ratification |
| 15 (Accuracy + robustness) | FR-REW-004 deterministic SI/PIT engine + regression test suite + FR-REW-006 migration drill |

**External-subject DSAR portal.**

`https://privacy.cyberos.world/dsar/{tenant-slug}` — the public-facing DSAR portal:

1. **Identity verification.** External subjects upload an identity document; the platform's KYC service (extending FR-AUTH-001 patterns) verifies.
2. **Request submission.** The subject describes their request in free text + selects scope (access / erasure / rectification / objection / restriction / portability).
3. **DPO routing.** The request lands in the tenant's DPO queue with the same workflow as Member-self DSARs.
4. **30-day SLA enforcement** per GDPR Article 12.
5. **Fulfilment** via the same DSAR enumerator as FR-CP-003.
6. **Audit.** Comprehensive trail; GDPR Article 30 records-of-processing alignment.

The portal is per-tenant per-shard; eu-shard tenants surface it actively; vn-shard tenants surface it at the customer's option.

**DPA template library.**

A `cp.dpa_template` library with pre-authored DPA terms appropriate for:
- **Tenant-as-controller, customer-as-controller** (joint controller arrangement; rare).
- **Tenant-as-controller, customer-as-processor** (customer processes their end-users' data via tenant's CyberOS instance — most common pattern).
- **Tenant-as-processor for the platform** (the platform is the sub-processor; the platform's master DPA covers).

Each template is parameter-version-locked + signed by external counsel + filed in DOC (FR-DOC-001) for use in tenant-to-customer service-agreement envelopes.

**Transfer Impact Assessments (TIAs).**

For cross-border data flows from eu-shard to non-EU regions, a per-flow TIA documents:
- Source country + destination country.
- Legal mechanism (Standard Contractual Clauses + supplementary measures + adequacy decisions where available).
- Practical assessment: encryption at rest + in transit + per-tenant KMS keys + the Vietnamese-vendor exemption analysis.
- Signed by founder + DPO + legal counsel ref.

Schema:
```sql
CREATE TABLE cp.transfer_impact_assessment (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  source_country TEXT NOT NULL,                                          -- ISO 3166-1
  destination_country TEXT NOT NULL,
  data_classes TEXT[] NOT NULL,
  legal_mechanism TEXT NOT NULL,                                          -- "SCCs_2021_module_2" | "adequacy_decision"
                                                                         -- | "binding_corporate_rules" | "consent_explicit"
  supplementary_measures_md TEXT NOT NULL,
  practical_assessment_md TEXT NOT NULL,
  signed_off_by_dpo_at TIMESTAMPTZ NOT NULL,
  signed_off_by_legal_counsel_ref TEXT NOT NULL,
  effective_from DATE NOT NULL,
  reviewed_at DATE,
  reviewed_due_at DATE NOT NULL,                                          -- annual review
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);
```

A typical eu-shard tenant has zero cross-border transfers (data stays in eu-shard). Cross-shard transfers (e.g. an EU tenant whose Members work from Vietnam → the Vietnamese-Member's CHAT messages on eu-shard) are per-Member's-residence + need TIAs only when actual data transfer crosses borders.

**Right-to-be-forgotten through deletion lifecycle.**

Article 17 erasure requests flow through FR-TEN-002's deletion lifecycle with these GDPR-specific overrides:
- **Statutory-retention-floor exceptions per Article 17(3):** legal compliance retention (Vietnamese SI/PIT 10y, accounting 7y) overrides erasure for those specific records; the records are *pseudonymised* (FR-AUTH-002 pattern) but not deleted.
- **Counter-claim during the process:** if the controller has documented public-interest or legal-defence ground, the request may be partially declined with explicit DPO + legal-counsel sign + Article 21 right-to-object route open.
- **30-day SLA enforcement.**

**MCP tool surface (extending FR-CP-003).**

- `cyberos.cp.eu_ai_act_evidence_status(article?)` — read; HR/Ops + Founder + DPO + Auditor.
- `cyberos.cp.list_tias` — read.
- `cyberos.cp.list_dpa_templates(jurisdiction?)` — read.
- `cyberos.cp.external_dsar_status(tenant_slug)` — read; tenant DPO.

# Alternatives Considered

- **Skip GDPR posture; only sell to non-EU customers.** Rejected: eliminates a major P3+ market.
- **Use a hosted compliance platform (Vanta GDPR module).** Considered for evidence-collection automation; we'll layer a hosted tool *on top of* the platform's data, but the canonical evidence + DPA + DPIA must live in CyberOS for residency reasons.
- **Skip EU AI Act evidence map; rely on FR-CP-003 alone.** Rejected: EU AI Act enforcement effective August 2026; the article-level evidence map is structurally required for high-risk modules.

# Success Metrics

- **Primary metric.** P3 → P4 exit-gate progress: eu-shard fully operational; first synthetic EU tenant provisioned with full DPIA + DPA + TIA + EU AI Act evidence map; external-subject DSAR portal handles a synthetic subject request end-to-end within 30 days.
- **Compliance metric.** EU AI Act evidence map shows `implemented` for all Articles 9-15 across REW/ESOP/LEARN modules; 100% of cross-border data flows have a signed TIA.

# Scope

**In-scope.**
- Per-eu-shard DPIA library extension with Article 35 fields.
- `cp.eu_ai_act_evidence_map` schema + per-Article evidence per high-risk module.
- External-subject DSAR portal at `privacy.cyberos.world/dsar/{tenant}`.
- `cp.dpa_template` library + per-template parameter-version sign chain.
- `cp.transfer_impact_assessment` schema + sign chain + annual-review cadence.
- GDPR Article 17 erasure with statutory-floor pseudonymisation.
- The 4 read-only MCP tools.
- Audit integration in scope `cp.gdpr.{tenant}`.

**Out-of-scope (deferred).**
- Auto-generated Article 30 records of processing (P4 — currently DPO-authored).
- ISO/IEC 42001 (AI Management System) certification (P4 — voluntary; alongside ISO 27001).
- Per-region data-portability format (P4 — current export bundle is JSON + Markdown, sufficient).

# Dependencies

- FR-TEN-001 / FR-TEN-002.
- FR-CP-001 / FR-CP-002 / FR-CP-003.
- FR-AUTH-001 / FR-AUTH-003.
- FR-DOC-001 (DPA template envelope storage).
- FR-OBS-001 / FR-OBS-002 / FR-OBS-003.
- External EU legal counsel for DPIA + DPA + TIA + EU AI Act evidence-map review.
- Per-jurisdiction local counsel (Germany / France / etc. as customer base develops).
- Compliance: GDPR Regulation 2016/679 + EU AI Act Regulation 2024/1689 (effective August 2026) + Schrems II rulings + per-EU-MS data protection authorities + ISO/IEC 27001/42001.
- Locked decisions referenced: DEC-276 (eu-shard at AWS Frankfurt; cross-border transfers require TIA), DEC-277 (external-subject DSAR portal in P3), DEC-278 (statutory-retention floors override Article 17 erasure with pseudonymisation).

# AI Risk Assessment

Not applicable. `eu_ai_act_risk_class: not_ai`. The compliance plane is workflow + evidence storage. The EU AI Act evidence map is the platform's structural answer to the regulation; the platform's AI surfaces (which use this evidence) are classified at their own FRs.

## AI Authorship Disclosure

- **Tools used:** Claude Cowork (Anthropic).
- **Scope:** drafted the FR end-to-end against the PRD + SRS; founder reviews and edits before status changes from `ready_for_review`.
- **Human review:** founder (`@stephen-cheng`) — final wording is the founder's responsibility.
